/// Citation Validator for Hallucination Detection
///
/// Verifies LLM-generated answers against source documents by:
/// 1. Extracting atomic claims from the answer
/// 2. Checking each claim has citation support in sources
/// 3. Calculating hallucination score (ratio of unsupported claims)
///
/// Uses Redis cache to avoid redundant verification calls.

use redis::{AsyncCommands, Client as RedisClient};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::Duration;

use crate::clients::python_worker::PythonWorkerClient;
use crate::errors::{AppError, Result};

/// Validation result from hallucination detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Hallucination score (0-1, higher = more hallucination)
    pub hallucination_score: f64,

    /// Claims that could not be verified against sources
    pub flagged_claims: Vec<String>,

    /// Claims that were successfully verified
    pub supported_claims: Vec<String>,

    /// Total number of claims extracted
    pub total_claims: usize,
}

/// Citation validator with Redis caching
pub struct CitationValidator {
    python_worker: Arc<PythonWorkerClient>,
    redis_client: RedisClient,
    cache_ttl: Duration,
}

impl CitationValidator {
    /// Create new citation validator
    ///
    /// # Arguments
    /// * `python_worker` - Python worker client for hallucination detection
    /// * `redis_url` - Redis connection URL
    /// * `cache_ttl` - Cache TTL (default: 1 hour for validation results)
    pub fn new(
        python_worker: Arc<PythonWorkerClient>,
        redis_url: String,
        cache_ttl: Option<Duration>,
    ) -> Result<Self> {
        let redis_client = RedisClient::open(redis_url)
            .map_err(|e| AppError::InternalError(format!("Redis connection failed: {}", e)))?;

        Ok(Self {
            python_worker,
            redis_client,
            cache_ttl: cache_ttl.unwrap_or(Duration::from_secs(3600)), // 1 hour default
        })
    }

    /// Validate answer against sources for hallucinations
    ///
    /// Pipeline:
    /// 1. Generate cache key from answer + sources
    /// 2. Check Redis cache
    /// 3. If miss, call Python worker /verify_hallucination
    /// 4. Cache result
    /// 5. Return validation result
    pub async fn validate(
        &self,
        answer: &str,
        sources: &[SourceDocument],
    ) -> Result<ValidationResult> {
        // Step 1: Generate cache key
        let cache_key = self.generate_cache_key(answer, sources);

        // Step 2: Check Redis cache
        if let Some(cached) = self.get_cached(&cache_key).await? {
            tracing::debug!(cache_key = %cache_key, "Cache hit for hallucination validation");
            return Ok(cached);
        }

        tracing::debug!(cache_key = %cache_key, "Cache miss, calling Python worker");

        // Step 3: Call Python worker
        let result = self
            .python_worker
            .verify_hallucination(answer, sources)
            .await
            .map_err(|e| {
                AppError::InternalError(format!("Hallucination verification failed: {}", e))
            })?;

        // Step 4: Cache result
        self.set_cached(&cache_key, &result).await?;

        Ok(result)
    }

    /// Generate cache key from answer + sources
    ///
    /// Uses SHA-256 hash of concatenated answer and source texts
    fn generate_cache_key(&self, answer: &str, sources: &[SourceDocument]) -> String {
        let mut hasher = Sha256::new();

        // Hash answer
        hasher.update(answer.as_bytes());

        // Hash all source quotes in deterministic order
        for source in sources {
            hasher.update(source.text_quote.as_bytes());
        }

        let hash = hasher.finalize();
        let hex_hash = hash.iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<String>();
        format!("hallucination:v1:{}", hex_hash)
    }

    /// Get cached validation result
    async fn get_cached(&self, key: &str) -> Result<Option<ValidationResult>> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| {
                AppError::InternalError(format!("Redis connection failed: {}", e))
            })?;

        let cached: Option<String> = conn.get(key).await.map_err(|e| {
            AppError::InternalError(format!("Redis GET failed: {}", e))
        })?;

        match cached {
            Some(json) => {
                let result: ValidationResult = serde_json::from_str(&json).map_err(|e| {
                    AppError::InternalError(format!("Failed to deserialize cached result: {}", e))
                })?;
                Ok(Some(result))
            }
            None => Ok(None),
        }
    }

    /// Cache validation result
    async fn set_cached(&self, key: &str, result: &ValidationResult) -> Result<()> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| {
                AppError::InternalError(format!("Redis connection failed: {}", e))
            })?;

        let json = serde_json::to_string(result).map_err(|e| {
            AppError::InternalError(format!("Failed to serialize validation result: {}", e))
        })?;

        let ttl_secs = self.cache_ttl.as_secs();

        conn.set_ex::<_, _, ()>(key, json, ttl_secs)
            .await
            .map_err(|e| AppError::InternalError(format!("Redis SETEX failed: {}", e)))?;

        Ok(())
    }
}

/// Source document for citation verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceDocument {
    /// Text quote from source document
    pub text_quote: String,

    /// Document ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_generation() {
        let python_worker = Arc::new(PythonWorkerClient::new("http://localhost:8091".to_string()));
        let validator = CitationValidator::new(
            python_worker,
            "redis://localhost:6379".to_string(),
            None,
        )
        .expect("Failed to create validator");

        let answer = "Acme Corp deve pagare €10,000 di penale.";
        let sources = vec![SourceDocument {
            text_quote: "Art. 5.2: Penale di €10,000 in caso di inadempimento.".to_string(),
            doc_id: Some("doc_001".to_string()),
        }];

        let key1 = validator.generate_cache_key(answer, &sources);
        let key2 = validator.generate_cache_key(answer, &sources);

        // Same input = same key
        assert_eq!(key1, key2);
        assert!(key1.starts_with("hallucination:v1:"));
        assert_eq!(key1.len(), 81); // "hallucination:v1:" (17 chars) + 64 hex chars
    }

    #[test]
    fn test_cache_key_different_sources() {
        let python_worker = Arc::new(PythonWorkerClient::new("http://localhost:8091".to_string()));
        let validator = CitationValidator::new(
            python_worker,
            "redis://localhost:6379".to_string(),
            None,
        )
        .expect("Failed to create validator");

        let answer = "Test answer";
        let sources1 = vec![SourceDocument {
            text_quote: "Source A".to_string(),
            doc_id: None,
        }];
        let sources2 = vec![SourceDocument {
            text_quote: "Source B".to_string(),
            doc_id: None,
        }];

        let key1 = validator.generate_cache_key(answer, &sources1);
        let key2 = validator.generate_cache_key(answer, &sources2);

        // Different sources = different keys
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_validation_result_serialization() {
        let result = ValidationResult {
            hallucination_score: 0.15,
            flagged_claims: vec!["Unsupported claim".to_string()],
            supported_claims: vec![
                "Supported claim 1".to_string(),
                "Supported claim 2".to_string(),
            ],
            total_claims: 3,
        };

        let json = serde_json::to_string(&result).expect("Serialization failed");
        let deserialized: ValidationResult =
            serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(result.hallucination_score, deserialized.hallucination_score);
        assert_eq!(result.flagged_claims, deserialized.flagged_claims);
        assert_eq!(result.supported_claims, deserialized.supported_claims);
        assert_eq!(result.total_claims, deserialized.total_claims);
    }
}
