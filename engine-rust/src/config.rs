/// Configuration module for Archivio Parlante Rust Engine
///
/// Loads configuration from environment variables with sane defaults.

use anyhow::{Context, Result};
use std::env;

/// Application configuration loaded from environment
#[derive(Debug, Clone)]
pub struct Config {
    /// Server listen address (default: 0.0.0.0:8090)
    pub listen_addr: String,

    /// Ollama base URL
    pub ollama_url: String,

    /// Ollama chat model (default)
    pub ollama_model_chat: String,

    /// Ollama chat model (lightweight for massive tasks)
    pub ollama_model_chat_small: String,

    /// Ollama embedding model
    pub ollama_model_embed: String,

    /// Qdrant base URL
    pub qdrant_url: String,

    /// Python worker base URL
    pub python_worker_url: String,

    /// MySQL connection string
    pub mysql_url: String,

    /// MySQL database name
    pub mysql_db: String,

    /// Internal auth token for PHP → Rust communication
    pub rust_engine_internal_token: String,

    // === Chunking parameters ===
    /// Target chunk size in tokens
    pub chunk_size_tokens: usize,

    /// Overlap percentage between chunks (0-100)
    pub chunk_overlap_percent: u8,

    // === Retrieval parameters ===
    /// Top-k for dense search
    pub top_k_dense: usize,

    /// Top-k for sparse (BM25) search
    pub top_k_sparse: usize,

    /// Top-k after reranking
    pub top_k_rerank: usize,

    /// RRF k parameter for hybrid fusion
    pub hybrid_fusion_k: u32,

    // === Concurrency limits ===
    /// Max concurrent embedding requests
    pub max_concurrent_embeddings: usize,

    /// Max concurrent LLM calls
    pub max_concurrent_llm_calls: usize,

    // === Provider API keys (opt-in) ===
    pub anthropic_api_key: Option<String>,
    pub google_api_key: Option<String>,
    pub openai_api_key: Option<String>,
    pub deepseek_api_key: Option<String>,

    // === Budget guard ===
    /// Daily cost budget in EUR (0.00 = disabled)
    pub daily_cost_budget_eur: f64,
}

impl Config {
    /// Load configuration from environment variables
    ///
    /// # Errors
    /// Returns error if required env vars are missing or invalid
    pub fn from_env() -> Result<Self> {
        // Load .env file if present (development)
        dotenvy::dotenv().ok();

        Ok(Config {
            listen_addr: env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8090".to_string()),

            ollama_url: env::var("OLLAMA_URL")
                .context("OLLAMA_URL must be set")?,

            ollama_model_chat: env::var("OLLAMA_MODEL_CHAT")
                .unwrap_or_else(|_| "qwen2.5:7b-instruct-q4_K_M".to_string()),

            ollama_model_chat_small: env::var("OLLAMA_MODEL_CHAT_SMALL")
                .unwrap_or_else(|_| "qwen2.5:3b-instruct-q4_K_M".to_string()),

            ollama_model_embed: env::var("OLLAMA_MODEL_EMBED")
                .unwrap_or_else(|_| "nomic-embed-text".to_string()),

            qdrant_url: env::var("QDRANT_URL")
                .context("QDRANT_URL must be set")?,

            python_worker_url: env::var("PYTHON_WORKER_URL")
                .context("PYTHON_WORKER_URL must be set")?,

            mysql_db: env::var("MYSQL_DB")
                .unwrap_or_else(|_| "archivio_parlante_x".to_string()),

            mysql_url: format!(
                "mysql://{}:{}@{}/{}",
                env::var("MYSQL_USER").unwrap_or_else(|_| "root".to_string()),
                env::var("MYSQL_PASSWORD").unwrap_or_default(),
                env::var("MYSQL_HOST").unwrap_or_else(|_| "mysql".to_string()),
                env::var("MYSQL_DB").unwrap_or_else(|_| "archivio_parlante_x".to_string())
            ),

            rust_engine_internal_token: env::var("RUST_ENGINE_INTERNAL_TOKEN")
                .context("RUST_ENGINE_INTERNAL_TOKEN must be set for auth")?,

            // Chunking
            chunk_size_tokens: env::var("CHUNK_SIZE_TOKENS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(800),

            chunk_overlap_percent: env::var("CHUNK_OVERLAP_PERCENT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(15),

            // Retrieval
            top_k_dense: env::var("TOP_K_DENSE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),

            top_k_sparse: env::var("TOP_K_SPARSE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),

            top_k_rerank: env::var("TOP_K_RERANK")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5),

            hybrid_fusion_k: env::var("HYBRID_FUSION_K")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(60),

            // Concurrency
            max_concurrent_embeddings: env::var("MAX_CONCURRENT_EMBEDDINGS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(16),

            max_concurrent_llm_calls: env::var("MAX_CONCURRENT_LLM_CALLS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(8),

            // Provider API keys (opt-in, empty = disabled)
            anthropic_api_key: env::var("ANTHROPIC_API_KEY")
                .ok()
                .filter(|s| !s.is_empty()),

            google_api_key: env::var("GOOGLE_API_KEY")
                .ok()
                .filter(|s| !s.is_empty()),

            openai_api_key: env::var("OPENAI_API_KEY")
                .ok()
                .filter(|s| !s.is_empty()),

            deepseek_api_key: env::var("DEEPSEEK_API_KEY")
                .ok()
                .filter(|s| !s.is_empty()),

            // Budget guard
            daily_cost_budget_eur: env::var("DAILY_COST_BUDGET_EUR")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.0),
        })
    }

    /// Check if cloud providers are enabled (at least one API key present)
    pub fn has_cloud_providers(&self) -> bool {
        self.anthropic_api_key.is_some()
            || self.google_api_key.is_some()
            || self.openai_api_key.is_some()
            || self.deepseek_api_key.is_some()
    }

    /// Check if daily budget allows cloud API calls
    pub fn cloud_budget_available(&self) -> bool {
        self.daily_cost_budget_eur > 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        // Set required env vars for test
        std::env::set_var("OLLAMA_URL", "http://localhost:11434");
        std::env::set_var("QDRANT_URL", "http://localhost:6333");
        std::env::set_var("PYTHON_WORKER_URL", "http://localhost:8091");
        std::env::set_var("RUST_ENGINE_INTERNAL_TOKEN", "test-token-123");

        let config = Config::from_env().expect("Config should load from env");

        assert_eq!(config.chunk_size_tokens, 800);
        assert_eq!(config.chunk_overlap_percent, 15);
        assert_eq!(config.top_k_dense, 30);
        assert_eq!(config.top_k_rerank, 5);
        assert!(!config.has_cloud_providers());
        assert!(!config.cloud_budget_available());
    }
}
