/// Chat route handler with RAG + hallucination detection
///
/// Fase 6.2 - Advanced Hallucination Detection
///
/// Pipeline:
/// 1. Retrieve relevant chunks (hybrid search + reranking)
/// 2. Generate answer using LLM
/// 3. Validate answer for hallucinations
/// 4. Store message in ap_chat_messages
/// 5. Return answer with verification results

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;

use crate::clients::qdrant::QdrantWrapper;
use crate::errors::{AppError, Result};
use crate::rag::citation_validator::{CitationValidator, SourceDocument, ValidationResult};
use crate::rag::hybrid_search::HybridSearcher;
use crate::routes::ingest::AppState;

/// Chat request
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ChatRequest {
    /// User query/question
    pub query: String,

    /// Knowledge base ID
    pub kb_id: String,

    /// Session ID (UUID for conversation grouping)
    pub session_id: String,

    /// User ID (for access control and audit)
    pub user_id: u64,

    /// Number of chunks to retrieve (default: 5)
    #[serde(default = "default_top_k")]
    pub top_k: usize,

    /// Enable hallucination detection (default: true)
    #[serde(default = "default_verify")]
    pub verify_hallucinations: bool,
}

fn default_top_k() -> usize {
    5
}

fn default_verify() -> bool {
    true
}

/// Chat response
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ChatResponse {
    /// LLM-generated answer
    pub answer: String,

    /// Citations supporting the answer
    pub citations: Vec<Citation>,

    /// Hallucination verification result (if enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification: Option<VerificationInfo>,

    /// Processing time in milliseconds
    pub processing_ms: u64,

    /// LLM provider used
    pub llm_provider: String,

    /// LLM model used
    pub llm_model: String,

    /// Message ID (for reference)
    pub message_id: i64,
}

/// Citation with source reference
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct Citation {
    /// Document ID
    pub doc_id: String,

    /// Chunk index within document
    pub chunk_index: usize,

    /// Verbatim text quote from source
    pub text_quote: String,

    /// Relevance score
    pub score: f32,
}

/// Hallucination verification info
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct VerificationInfo {
    /// Hallucination score (0-1, higher = more hallucination)
    pub hallucination_score: f64,

    /// Claims that could not be verified
    pub flagged_claims: Vec<String>,

    /// Number of verified claims
    pub supported_claims_count: usize,

    /// Total claims extracted
    pub total_claims: usize,

    /// Overall verification status
    pub verified: bool,
}

/// Chat handler with RAG and hallucination detection
///
/// # Errors
/// Returns appropriate AppError if any step fails
pub async fn handle_chat(
    State(state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>> {
    let start = Instant::now();

    tracing::info!(
        query = %req.query,
        kb_id = %req.kb_id,
        session_id = %req.session_id,
        user_id = req.user_id,
        "Starting chat processing"
    );

    // Step 1: Validate request
    validate_request(&req)?;

    // Step 2: Retrieve relevant chunks
    let collection_name = format!("ap_kb_{}", req.kb_id);
    let qdrant = QdrantWrapper::new(
        state.config.qdrant_url.clone(),
        collection_name,
        768, // nomic-embed-text dimension
    )
    .await?;

    let llm = state.llm_registry.default();
    let hybrid_searcher = HybridSearcher::new(
        Arc::new(qdrant),
        llm.clone(),
        30, // top_k_dense
        30, // top_k_sparse
        60, // rrf_k
    );

    let candidates = hybrid_searcher.search(&req.query, 30).await?;

    if candidates.is_empty() {
        tracing::info!("No relevant chunks found");
        return Err(AppError::NotFound(
            "No relevant information found in the knowledge base".to_string(),
        ));
    }

    // Step 3: Rerank with BGE cross-encoder
    let texts: Vec<String> = candidates.iter().map(|c| c.text.clone()).collect();
    let rerank_results = state
        .python_worker
        .rerank(req.query.clone(), texts, req.top_k)
        .await?;

    // Step 4: Build citations
    let citations: Vec<Citation> = rerank_results
        .iter()
        .filter_map(|rr| {
            candidates.get(rr.index).map(|candidate| Citation {
                doc_id: candidate.doc_id.clone(),
                chunk_index: candidate.chunk_index,
                text_quote: candidate.text.clone(),
                score: rr.score,
            })
        })
        .collect();

    if citations.is_empty() {
        return Err(AppError::InternalError(
            "Reranking produced no results".to_string(),
        ));
    }

    // Step 5: Generate answer using LLM
    let context = citations
        .iter()
        .map(|c| c.text_quote.as_str())
        .collect::<Vec<_>>()
        .join("\n\n---\n\n");

    let answer = generate_answer(&state, &req.query, &context).await?;

    // Step 6: Verify hallucinations (if enabled)
    let verification = if req.verify_hallucinations {
        let sources: Vec<SourceDocument> = citations
            .iter()
            .map(|c| SourceDocument {
                text_quote: c.text_quote.clone(),
                doc_id: Some(c.doc_id.clone()),
            })
            .collect();

        let validator = CitationValidator::new(
            state.python_worker.clone(),
            state.config.redis_url.clone(),
            None, // Use default TTL (1 hour)
        )?;

        let result = validator.validate(&answer, &sources).await?;

        Some(VerificationInfo {
            hallucination_score: result.hallucination_score,
            flagged_claims: result.flagged_claims,
            supported_claims_count: result.supported_claims.len(),
            total_claims: result.total_claims,
            verified: result.hallucination_score < 0.3, // Threshold: 30% unsupported claims
        })
    } else {
        None
    };

    // Step 7: Store message in database
    let message_id = store_chat_message(&state, &req, &answer, &citations, &verification).await?;

    let processing_ms = start.elapsed().as_millis() as u64;

    tracing::info!(
        message_id = message_id,
        processing_ms = processing_ms,
        hallucination_score = ?verification.as_ref().map(|v| v.hallucination_score),
        "Chat processing completed"
    );

    Ok(Json(ChatResponse {
        answer,
        citations,
        verification,
        processing_ms,
        llm_provider: "ollama".to_string(), // TODO: Get from LLM provider
        llm_model: state.config.ollama_model_chat.clone(),
        message_id,
    }))
}

/// Validate chat request
fn validate_request(req: &ChatRequest) -> Result<()> {
    if req.query.trim().is_empty() {
        return Err(AppError::BadRequest("query cannot be empty".to_string()));
    }

    if req.kb_id.is_empty() {
        return Err(AppError::BadRequest("kb_id cannot be empty".to_string()));
    }

    if req.session_id.is_empty() {
        return Err(AppError::BadRequest(
            "session_id cannot be empty".to_string(),
        ));
    }

    if req.top_k == 0 || req.top_k > 50 {
        return Err(AppError::BadRequest(
            "top_k must be between 1 and 50".to_string(),
        ));
    }

    Ok(())
}

/// Generate answer using LLM
async fn generate_answer(state: &AppState, query: &str, context: &str) -> Result<String> {
    let llm = state.llm_registry.default();

    let prompt = format!(
        r#"You are a legal document analysis assistant. Answer the following question based ONLY on the provided context.

IMPORTANT RULES:
1. Use ONLY information from the context below
2. Quote verbatim from the context using "..." when citing specific clauses
3. If the context doesn't contain the answer, say "Le informazioni richieste non sono presenti nei documenti caricati."
4. Always cite which document section your answer comes from

Context:
{context}

Question: {query}

Answer (in Italian, with citations):"#
    );

    let answer = llm.generate(&prompt, 2000, 0.7).await.map_err(|e| {
        AppError::InternalError(format!("LLM generation failed: {}", e))
    })?;

    let answer = answer.trim();

    if answer.is_empty() {
        return Err(AppError::InternalError(
            "LLM generated empty answer".to_string(),
        ));
    }

    Ok(answer.to_string())
}

/// Store chat message in database
async fn store_chat_message(
    state: &AppState,
    req: &ChatRequest,
    answer: &str,
    citations: &[Citation],
    verification: &Option<VerificationInfo>,
) -> Result<i64> {
    let citations_json = serde_json::to_value(citations)
        .map_err(|e| AppError::InternalError(format!("Failed to serialize citations: {}", e)))?;

    let (hallucination_score, flagged_claims_count, verified_at) = match verification {
        Some(v) => (
            Some(v.hallucination_score),
            v.flagged_claims.len() as i32,
            Some(chrono::Utc::now().naive_utc()),
        ),
        None => (None, 0, None),
    };

    let result = sqlx::query(
        r#"
        INSERT INTO ap_chat_messages (
            session_id, kb_id, user_id, role, content, citations,
            llm_provider, llm_model, verified,
            hallucination_score, flagged_claims_count, verified_at
        ) VALUES (?, ?, ?, 'assistant', ?, ?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(&req.session_id)
    .bind(&req.kb_id)
    .bind(req.user_id as i64)
    .bind(answer)
    .bind(&citations_json)
    .bind("ollama")
    .bind(&state.config.ollama_model_chat)
    .bind(verification.as_ref().map(|v| v.verified).unwrap_or(false))
    .bind(hallucination_score)
    .bind(flagged_claims_count)
    .bind(verified_at)
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::InternalError(format!("Failed to store message: {}", e)))?;

    Ok(result.last_insert_id() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_request_valid() {
        let req = ChatRequest {
            query: "Test query".to_string(),
            kb_id: "kb_123".to_string(),
            session_id: "session_456".to_string(),
            user_id: 1,
            top_k: 5,
            verify_hallucinations: true,
        };

        assert!(validate_request(&req).is_ok());
    }

    #[test]
    fn test_validate_request_empty_query() {
        let req = ChatRequest {
            query: "   ".to_string(),
            kb_id: "kb_123".to_string(),
            session_id: "session_456".to_string(),
            user_id: 1,
            top_k: 5,
            verify_hallucinations: true,
        };

        let result = validate_request(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_request_invalid_top_k() {
        let req = ChatRequest {
            query: "Test".to_string(),
            kb_id: "kb_123".to_string(),
            session_id: "session_456".to_string(),
            user_id: 1,
            top_k: 0,
            verify_hallucinations: true,
        };

        let result = validate_request(&req);
        assert!(result.is_err());
    }
}
