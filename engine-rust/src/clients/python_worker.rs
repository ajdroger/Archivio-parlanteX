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
    ) -> Result<ParsedDocument> {
        let url = format!("{}/parse", self.base_url);

        let request = ParseDocumentRequest { file_path, doc_id };

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
            .json::<ParsedDocument>()
            .await
            .map_err(|e| AppError::PythonWorker(format!("Failed to parse response: {}", e)))?;

        tracing::info!(
            doc_id = %parsed.doc_id,
            pages = parsed.pages.len(),
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
}

// === Request/Response types ===

#[derive(Debug, Serialize)]
struct ParseDocumentRequest {
    file_path: String,
    doc_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ParsedDocument {
    pub doc_id: String,
    pub pages: Vec<Page>,
    pub metadata: DocumentMetadata,
}

#[derive(Debug, Deserialize)]
pub struct Page {
    pub page_num: usize,
    pub text: String,
    pub tables: Option<Vec<Table>>,
}

#[derive(Debug, Deserialize)]
pub struct Table {
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub created_at: Option<String>,
    pub page_count: usize,
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
