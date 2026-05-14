/// Python AI Worker HTTP client

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::errors::{AppError, Result};

/// Python worker client for ML operations
pub struct PythonWorkerClient {
    base_url: String,
    client: Client,
}

impl PythonWorkerClient {
    /// Create new Python worker client
    ///
    /// # Arguments
    /// * `base_url` - Python worker URL (e.g., http://python-worker:8091)
    pub fn new(base_url: String) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: Client::new(),
        }
    }

    /// Parse document (PDF/DOCX) to structured text
    ///
    /// # Arguments
    /// * `file_path` - Path to file in shared volume
    /// * `doc_id` - Document UUID
    pub async fn parse_document(
        &self,
        file_path: String,
        doc_id: String,
        kb_id: String,
        mime_type: String,
    ) -> Result<ParseResponse> {
        let url = format!("{}/parse", self.base_url);

        let request = ParseDocumentRequest { file_path, doc_id, kb_id, mime_type };

        tracing::debug!(url = %url, "Sending parse request to Python worker");

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::PythonWorker(format!("Parse request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::PythonWorker(format!(
                "Parse failed with status {}: {}",
                status, body
            )));
        }

        let parsed = response
            .json::<ParseResponse>()
            .await
            .map_err(|e| AppError::PythonWorker(format!("Failed to parse response: {}", e)))?;

        tracing::info!(
            doc_id = %parsed.doc_id,
            chunks = parsed.chunks.len(),
            parsing_method = %parsed.parsing_method,
            processing_ms = parsed.processing_ms,
            "Document parsed successfully"
        );

        Ok(parsed)
    }

    /// Add contextual retrieval prefix to chunks
    ///
    /// Uses LLM to generate document context for each chunk (Anthropic technique).
    ///
    /// # Arguments
    /// * `doc_id` - Document UUID
    /// * `chunks` - Raw chunks
    pub async fn contextualize_chunks(
        &self,
        doc_id: String,
        chunks: Vec<RawChunk>,
    ) -> Result<Vec<ContextualizedChunk>> {
        let url = format!("{}/contextualize", self.base_url);

        let request = ContextualizeRequest { doc_id, chunks };

        tracing::debug!(
            url = %url,
            chunks_count = request.chunks.len(),
            "Sending contextualize request"
        );

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                AppError::PythonWorker(format!("Contextualize request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::PythonWorker(format!(
                "Contextualize failed with status {}: {}",
                status, body
            )));
        }

        let result = response
            .json::<ContextualizeResponse>()
            .await
            .map_err(|e| AppError::PythonWorker(format!("Failed to parse response: {}", e)))?;

        tracing::info!(
            chunks_count = result.chunks.len(),
            "Chunks contextualized successfully"
        );

        Ok(result.chunks)
    }

    /// Rerank chunks with BGE cross-encoder
    ///
    /// # Arguments
    /// * `query` - User query
    /// * `chunks` - Candidate chunks from hybrid search
    /// * `top_k` - Number of chunks to return
    pub async fn rerank(
        &self,
        query: String,
        chunks: Vec<String>,
        top_k: usize,
    ) -> Result<Vec<RerankResult>> {
        let url = format!("{}/rerank", self.base_url);

        let request = RerankRequest {
            query,
            texts: chunks,
            top_k,
        };

        tracing::debug!(
            url = %url,
            texts_count = request.texts.len(),
            top_k = request.top_k,
            "Sending rerank request"
        );

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::PythonWorker(format!("Rerank request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::PythonWorker(format!(
                "Rerank failed with status {}: {}",
                status, body
            )));
        }

        let result = response
            .json::<RerankResponse>()
            .await
            .map_err(|e| AppError::PythonWorker(format!("Failed to parse response: {}", e)))?;

        tracing::info!(
            results_count = result.results.len(),
            "Chunks reranked successfully"
        );

        Ok(result.results)
    }

    /// Extract knowledge graph entities and relations
    ///
    /// # Arguments
    /// * `doc_id` - Document UUID
    /// * `text` - Full document text
    pub async fn extract_knowledge_graph(
        &self,
        doc_id: String,
        text: String,
    ) -> Result<KnowledgeGraph> {
        let url = format!("{}/extract-kg", self.base_url);

        let request = ExtractKgRequest { doc_id, text };

        tracing::debug!(url = %url, "Sending KG extraction request");

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::PythonWorker(format!("KG extract request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::PythonWorker(format!(
                "KG extract failed with status {}: {}",
                status, body
            )));
        }

        let kg = response
            .json::<KnowledgeGraph>()
            .await
            .map_err(|e| AppError::PythonWorker(format!("Failed to parse response: {}", e)))?;

        tracing::info!(
            nodes = kg.nodes.len(),
            edges = kg.edges.len(),
            "Knowledge graph extracted"
        );

        Ok(kg)
    }

    /// Verify hallucinations in LLM-generated answer
    ///
    /// # Arguments
    /// * `answer` - LLM-generated answer to verify
    /// * `sources` - Source documents with text_quote
    pub async fn verify_hallucination(
        &self,
        answer: &str,
        sources: &[crate::rag::citation_validator::SourceDocument],
    ) -> Result<crate::rag::citation_validator::ValidationResult> {
        let url = format!("{}/verify_hallucination", self.base_url);

        let request = VerifyHallucinationRequest {
            answer: answer.to_string(),
            sources: sources
                .iter()
                .map(|s| VerifyHallucinationSource {
                    text_quote: s.text_quote.clone(),
                    doc_id: s.doc_id.clone(),
                })
                .collect(),
        };

        tracing::debug!(
            url = %url,
            sources_count = request.sources.len(),
            "Sending hallucination verification request"
        );

        let response = self
            .client
            .post(&url)
            .timeout(std::time::Duration::from_secs(60)) // Long timeout for claim extraction
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                AppError::PythonWorker(format!("Hallucination verification request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::PythonWorker(format!(
                "Hallucination verification failed with status {}: {}",
                status, body
            )));
        }

        let result = response
            .json::<crate::rag::citation_validator::ValidationResult>()
            .await
            .map_err(|e| AppError::PythonWorker(format!("Failed to parse response: {}", e)))?;

        tracing::info!(
            hallucination_score = result.hallucination_score,
            flagged_claims = result.flagged_claims.len(),
            total_claims = result.total_claims,
            "Hallucination verification completed"
        );

        Ok(result)
    }
}

// === Request/Response types ===

#[derive(Debug, Serialize)]
struct ParseDocumentRequest {
    file_path: String,
    doc_id: String,
    kb_id: String,
    mime_type: String,
}

/// Response from Python worker /parse endpoint
#[derive(Debug, Deserialize)]
pub struct ParseResponse {
    pub doc_id: String,
    pub kb_id: String,
    pub chunks: Vec<ParsedChunk>,
    pub total_chunks: usize,
    pub total_pages: Option<usize>,
    pub parsing_method: String,
    pub processing_ms: usize,
}

/// A single parsed chunk of text
#[derive(Debug, Clone, Deserialize)]
pub struct ParsedChunk {
    pub text: String,
    pub page_number: Option<usize>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawChunk {
    pub index: usize,
    pub text: String,
}

#[derive(Debug, Serialize)]
struct ContextualizeRequest {
    doc_id: String,
    chunks: Vec<RawChunk>,
}

#[derive(Debug, Deserialize)]
struct ContextualizeResponse {
    chunks: Vec<ContextualizedChunk>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContextualizedChunk {
    pub index: usize,
    pub original_text: String,
    pub context_prefix: String,
    pub full_text: String, // context_prefix + original_text
}

#[derive(Debug, Serialize)]
struct RerankRequest {
    query: String,
    texts: Vec<String>,
    top_k: usize,
}

#[derive(Debug, Deserialize)]
struct RerankResponse {
    results: Vec<RerankResult>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RerankResult {
    pub index: usize,
    pub score: f32,
}

#[derive(Debug, Serialize)]
struct ExtractKgRequest {
    doc_id: String,
    text: String,
}

#[derive(Debug, Deserialize)]
pub struct KnowledgeGraph {
    pub nodes: Vec<KgNode>,
    pub edges: Vec<KgEdge>,
}

#[derive(Debug, Deserialize)]
pub struct KgNode {
    pub id: String,
    pub entity_type: String, // PARTY, DATE, AMOUNT, CLAUSE, JURISDICTION, PENALTY
    pub name: String,
    pub properties: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct KgEdge {
    pub source_id: String,
    pub target_id: String,
    pub relation_type: String,
    pub properties: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct VerifyHallucinationRequest {
    answer: String,
    sources: Vec<VerifyHallucinationSource>,
}

#[derive(Debug, Serialize)]
struct VerifyHallucinationSource {
    text_quote: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    doc_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_worker_client_creation() {
        let client = PythonWorkerClient::new("http://localhost:8091/".to_string());
        assert_eq!(client.base_url, "http://localhost:8091");
    }

    #[test]
    fn test_raw_chunk_serialization() {
        let chunk = RawChunk {
            index: 0,
            text: "Sample text".to_string(),
        };

        let json = serde_json::to_string(&chunk).expect("Serialization failed");
        assert!(json.contains("\"index\":0"));
        assert!(json.contains("Sample text"));
    }
}
