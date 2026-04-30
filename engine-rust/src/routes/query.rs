/// Query route handler with hybrid search + reranker

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;

use crate::clients::qdrant::QdrantWrapper;
use crate::errors::{AppError, Result};
use crate::rag::hybrid_search::HybridSearcher;
use crate::routes::ingest::AppState;

/// Query request
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct QueryRequest {
    /// User query text
    pub query: String,

    /// Knowledge base ID
    pub kb_id: String,

    /// Number of final results after reranking (default: 5)
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

fn default_top_k() -> usize {
    5
}

/// Query response
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct QueryResponse {
    /// Search results (reranked and scored)
    pub results: Vec<SearchResult>,

    /// Processing time in milliseconds
    pub processing_ms: u64,

    /// Number of candidates before reranking
    pub candidates_count: usize,
}

/// Search result item
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct SearchResult {
    /// Chunk ID (UUID)
    pub chunk_id: String,

    /// Document ID
    pub doc_id: String,

    /// Chunk index within document
    pub chunk_index: usize,

    /// Chunk text
    pub text: String,

    /// Reranker score (BGE cross-encoder)
    pub score: f32,

    /// Optional metadata (JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Query handler with hybrid search + reranking
///
/// Pipeline:
/// 1. Validate request
/// 2. Embed query (dense vector)
/// 3. Hybrid search (dense + sparse with RRF fusion, top 30)
/// 4. Rerank with BGE cross-encoder (top_k final)
/// 5. Return ranked results
///
/// # Errors
/// Returns appropriate AppError if any step fails
pub async fn handle_query(
    State(state): State<AppState>,
    Json(req): Json<QueryRequest>,
) -> Result<Json<QueryResponse>> {
    let start = Instant::now();

    tracing::info!(
        query = %req.query,
        kb_id = %req.kb_id,
        top_k = req.top_k,
        "Starting query processing"
    );

    // Step 1: Validate request
    validate_request(&req)?;

    // Step 2: Create Qdrant client for this KB
    let collection_name = format!("ap_kb_{}", req.kb_id);
    let qdrant = QdrantWrapper::new(
        state.config.qdrant_url.clone(),
        collection_name,
        768, // nomic-embed-text dimension
    )
    .await?;

    // Step 3: Create hybrid searcher
    let llm = state.llm_registry.default();
    let hybrid_searcher = HybridSearcher::new(
        Arc::new(qdrant),
        llm.clone(),
        30, // top_k_dense
        30, // top_k_sparse
        60, // rrf_k
    );

    // Step 4: Perform hybrid search (RRF fusion)
    let top_k_rerank = 30; // Candidates for reranking
    let candidates = hybrid_searcher.search(&req.query, top_k_rerank).await?;

    let candidates_count = candidates.len();

    tracing::debug!(
        candidates_count = candidates_count,
        "Hybrid search completed, preparing for reranking"
    );

    if candidates.is_empty() {
        tracing::info!("No candidates found for query");
        return Ok(Json(QueryResponse {
            results: vec![],
            processing_ms: start.elapsed().as_millis() as u64,
            candidates_count: 0,
        }));
    }

    // Step 5: Rerank with BGE cross-encoder
    let reranked = rerank_results(&state, &req.query, candidates, req.top_k).await?;

    let processing_ms = start.elapsed().as_millis() as u64;

    tracing::info!(
        results_count = reranked.len(),
        processing_ms = processing_ms,
        "Query processing completed"
    );

    Ok(Json(QueryResponse {
        results: reranked,
        processing_ms,
        candidates_count,
    }))
}

/// Validate query request
fn validate_request(req: &QueryRequest) -> Result<()> {
    if req.query.trim().is_empty() {
        return Err(AppError::BadRequest("query cannot be empty".to_string()));
    }

    if req.kb_id.is_empty() {
        return Err(AppError::BadRequest("kb_id cannot be empty".to_string()));
    }

    if req.top_k == 0 {
        return Err(AppError::BadRequest(
            "top_k must be greater than 0".to_string(),
        ));
    }

    if req.top_k > 50 {
        return Err(AppError::BadRequest(
            "top_k cannot exceed 50".to_string(),
        ));
    }

    Ok(())
}

/// Rerank search results using BGE cross-encoder
async fn rerank_results(
    state: &AppState,
    query: &str,
    candidates: Vec<crate::clients::qdrant::ScoredChunk>,
    top_k: usize,
) -> Result<Vec<SearchResult>> {
    // Extract texts for reranking
    let texts: Vec<String> = candidates.iter().map(|c| c.text.clone()).collect();

    // Call Python worker reranker
    let rerank_results = state
        .python_worker
        .rerank(query.to_string(), texts, top_k)
        .await?;

    tracing::debug!(
        reranked_count = rerank_results.len(),
        "Reranking completed"
    );

    // Map rerank results back to candidates
    let results: Vec<SearchResult> = rerank_results
        .into_iter()
        .filter_map(|rr| {
            candidates.get(rr.index).map(|candidate| SearchResult {
                chunk_id: candidate.id.clone(),
                doc_id: candidate.doc_id.clone(),
                chunk_index: candidate.chunk_index,
                text: candidate.text.clone(),
                score: rr.score,
                metadata: None, // Metadata extraction can be added later if needed
            })
        })
        .collect();

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_request_valid() {
        let req = QueryRequest {
            query: "contratto fornitura servizi".to_string(),
            kb_id: "kb_123".to_string(),
            top_k: 5,
        };

        assert!(validate_request(&req).is_ok());
    }

    #[test]
    fn test_validate_request_empty_query() {
        let req = QueryRequest {
            query: "   ".to_string(),
            kb_id: "kb_123".to_string(),
            top_k: 5,
        };

        let result = validate_request(&req);
        assert!(result.is_err());

        if let Err(AppError::BadRequest(msg)) = result {
            assert!(msg.contains("query"));
        } else {
            panic!("Expected BadRequest error");
        }
    }

    #[test]
    fn test_validate_request_empty_kb_id() {
        let req = QueryRequest {
            query: "test query".to_string(),
            kb_id: "".to_string(),
            top_k: 5,
        };

        let result = validate_request(&req);
        assert!(result.is_err());

        if let Err(AppError::BadRequest(msg)) = result {
            assert!(msg.contains("kb_id"));
        } else {
            panic!("Expected BadRequest error");
        }
    }

    #[test]
    fn test_validate_request_zero_top_k() {
        let req = QueryRequest {
            query: "test query".to_string(),
            kb_id: "kb_123".to_string(),
            top_k: 0,
        };

        let result = validate_request(&req);
        assert!(result.is_err());

        if let Err(AppError::BadRequest(msg)) = result {
            assert!(msg.contains("top_k"));
            assert!(msg.contains("greater than 0"));
        } else {
            panic!("Expected BadRequest error");
        }
    }

    #[test]
    fn test_validate_request_top_k_too_large() {
        let req = QueryRequest {
            query: "test query".to_string(),
            kb_id: "kb_123".to_string(),
            top_k: 100,
        };

        let result = validate_request(&req);
        assert!(result.is_err());

        if let Err(AppError::BadRequest(msg)) = result {
            assert!(msg.contains("top_k"));
            assert!(msg.contains("cannot exceed 50"));
        } else {
            panic!("Expected BadRequest error");
        }
    }

    #[test]
    fn test_default_top_k() {
        assert_eq!(default_top_k(), 5);
    }

    #[test]
    fn test_search_result_serialization() {
        let result = SearchResult {
            chunk_id: "chunk_001".to_string(),
            doc_id: "doc_123".to_string(),
            chunk_index: 0,
            text: "Sample contract text".to_string(),
            score: 0.95,
            metadata: None,
        };

        let json = serde_json::to_string(&result).expect("Serialization failed");
        assert!(json.contains("chunk_001"));
        assert!(json.contains("0.95"));
        assert!(!json.contains("metadata")); // Should be omitted when None
    }
}
