/// Document ingestion route handler

use axum::{extract::State, Json};
use std::sync::Arc;
use std::time::Instant;

use crate::chunker::contextual::ContextualRetrievalEnricher;
use crate::chunker::semantic::SemanticChunker;
use crate::clients::qdrant::{ChunkInsert, SparseVector};
use crate::errors::{AppError, Result};
use crate::models::document::{IngestRequest, IngestResponse};
use crate::utils::bm25::Bm25Vectorizer;

/// Application state for routes
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<crate::config::Config>,
    pub llm_registry: Arc<crate::providers::registry::LlmRegistry>,
    pub python_worker: Arc<crate::clients::python_worker::PythonWorkerClient>,
}

/// Ingest document into knowledge base
///
/// Pipeline:
/// 1. Validate request
/// 2. Parse document (Python worker)
/// 3. Semantic chunking
/// 4. Contextual enrichment
/// 5. Embedding generation (Ollama)
/// 6. Store in Qdrant
///
/// # Errors
/// Returns appropriate AppError if any step fails
pub async fn handle_ingest(
    State(state): State<AppState>,
    Json(req): Json<IngestRequest>,
) -> Result<Json<IngestResponse>> {
    let start = Instant::now();

    tracing::info!(
        doc_id = %req.doc_id,
        kb_id = %req.kb_id,
        file_path = %req.file_path,
        "Starting document ingestion"
    );

    // Step 1: Validate request
    validate_request(&req)?;

    // Step 2: Parse document via Python worker
    tracing::debug!("Calling Python worker to parse document");
    let parsed = state
        .python_worker
        .parse_document(req.file_path.clone(), req.doc_id.clone())
        .await?;

    let full_text = parsed
        .pages
        .iter()
        .map(|p| p.text.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");

    tracing::info!(
        pages = parsed.pages.len(),
        text_length = full_text.len(),
        "Document parsed successfully"
    );

    // Step 3: Semantic chunking
    let chunker = SemanticChunker::new(
        state.config.chunk_size_tokens,
        state.config.chunk_overlap_percent as f32 / 100.0,
    );

    let mut chunks = chunker.chunk(&full_text, &req.doc_id)?;

    tracing::info!(chunks_count = chunks.len(), "Document chunked");

    // Step 4: Contextual enrichment
    let llm = state.llm_registry.default();
    let enricher = ContextualRetrievalEnricher::new(
        llm.clone(),
        state.config.ollama_model_chat_small.clone(),
        state.config.max_concurrent_llm_calls,
    );

    enricher.enrich(&full_text, &mut chunks).await?;

    let enriched_count = chunks.iter().filter(|c| c.is_contextualized()).count();
    tracing::info!(
        enriched_count = enriched_count,
        total = chunks.len(),
        "Contextual enrichment completed"
    );

    // Step 4.5: Generate BM25 sparse vectors
    let sparse_vectors = generate_sparse_vectors(&chunks)?;

    // Step 5: Embedding generation
    let embeddings = generate_embeddings(&state, &chunks, llm).await?;

    // Step 6: Store in Qdrant
    let chunks_indexed = store_in_qdrant(&state.config, &req, chunks, embeddings, sparse_vectors).await?;

    // TODO: Step 7: Extract knowledge graph (Phase 1.4)
    let entities_extracted = 0;

    let processing_ms = start.elapsed().as_millis() as u64;

    tracing::info!(
        doc_id = %req.doc_id,
        chunks_indexed = chunks_indexed,
        processing_ms = processing_ms,
        "Ingestion completed successfully"
    );

    Ok(Json(IngestResponse {
        doc_id: req.doc_id,
        chunks_indexed,
        processing_ms,
        entities_extracted,
    }))
}

/// Validate ingestion request
fn validate_request(req: &IngestRequest) -> Result<()> {
    if req.doc_id.is_empty() {
        return Err(AppError::BadRequest("doc_id cannot be empty".to_string()));
    }

    if req.kb_id.is_empty() {
        return Err(AppError::BadRequest("kb_id cannot be empty".to_string()));
    }

    if req.file_path.is_empty() {
        return Err(AppError::BadRequest(
            "file_path cannot be empty".to_string(),
        ));
    }

    // Validate MIME type
    let valid_mimes = [
        "application/pdf",
        "text/plain",
        "application/msword",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    ];

    if !valid_mimes.contains(&req.mime_type.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Unsupported MIME type: {}. Supported: PDF, TXT, DOC, DOCX",
            req.mime_type
        )));
    }

    Ok(())
}

/// Generate BM25 sparse vectors for chunks
fn generate_sparse_vectors(chunks: &[crate::models::chunk::Chunk]) -> Result<Vec<SparseVector>> {
    tracing::debug!(chunks_count = chunks.len(), "Generating BM25 sparse vectors");

    let mut vectorizer = Bm25Vectorizer::new();

    // Build vocabulary from all chunks
    let texts: Vec<String> = chunks.iter().map(|c| c.embedding_text().to_string()).collect();
    vectorizer.build_vocabulary(&texts);

    // Generate sparse vector for each chunk
    let sparse_vectors: Result<Vec<_>> = texts
        .iter()
        .map(|text| vectorizer.vectorize(text))
        .collect();

    let sparse_vectors = sparse_vectors?;

    tracing::info!(
        vocab_size = vectorizer.vocab_size(),
        vectors_generated = sparse_vectors.len(),
        "BM25 sparse vectors generated"
    );

    Ok(sparse_vectors)
}

/// Generate embeddings for chunks
async fn generate_embeddings(
    state: &AppState,
    chunks: &[crate::models::chunk::Chunk],
    llm: Arc<dyn crate::providers::LlmProvider>,
) -> Result<Vec<Vec<f32>>> {
    tracing::debug!(chunks_count = chunks.len(), "Generating embeddings");

    let texts: Vec<String> = chunks.iter().map(|c| c.embedding_text().to_string()).collect();

    // Batch embeddings to avoid overwhelming Ollama
    let batch_size = state.config.max_concurrent_embeddings;
    let mut all_embeddings = Vec::with_capacity(texts.len());

    for (batch_idx, chunk_batch) in texts.chunks(batch_size).enumerate() {
        tracing::debug!(
            batch = batch_idx + 1,
            size = chunk_batch.len(),
            "Processing embedding batch"
        );

        let batch_embeddings = llm.embed(chunk_batch.to_vec()).await.map_err(|e| {
            tracing::error!(error = %e, "Embedding generation failed");
            e
        })?;

        all_embeddings.extend(batch_embeddings);
    }

    tracing::info!(
        embeddings_generated = all_embeddings.len(),
        "Embeddings generation completed"
    );

    Ok(all_embeddings)
}

/// Store chunks in Qdrant
async fn store_in_qdrant(
    config: &crate::config::Config,
    req: &IngestRequest,
    chunks: Vec<crate::models::chunk::Chunk>,
    embeddings: Vec<Vec<f32>>,
    sparse_vectors: Vec<SparseVector>,
) -> Result<usize> {
    if chunks.len() != embeddings.len() || chunks.len() != sparse_vectors.len() {
        return Err(AppError::Internal(anyhow::anyhow!(
            "Chunks count ({}) != embeddings count ({}) or sparse vectors count ({})",
            chunks.len(),
            embeddings.len(),
            sparse_vectors.len()
        )));
    }

    // Create Qdrant client for this KB
    let collection_name = format!("ap_kb_{}", req.kb_id);
    let qdrant = crate::clients::qdrant::QdrantWrapper::new(
        config.qdrant_url.clone(),
        collection_name,
        768, // nomic-embed-text dimension
    )
    .await?;

    // Ensure collection exists
    qdrant.ensure_collection().await?;

    // Prepare chunk inserts
    let chunk_inserts: Vec<ChunkInsert> = chunks
        .into_iter()
        .zip(embeddings)
        .zip(sparse_vectors)
        .map(|((chunk, embedding), sparse)| ChunkInsert {
            id: chunk.id.to_string(),
            doc_id: req.doc_id.clone(),
            chunk_index: chunk.chunk_idx,
            text: chunk.text,
            dense_embedding: embedding,
            sparse_vector: Some(sparse),
            metadata: Some(chunk.metadata),
        })
        .collect();

    let count = chunk_inserts.len();

    qdrant.upsert_chunks(chunk_inserts).await?;

    tracing::info!(chunks_stored = count, "Chunks stored in Qdrant");

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_request_valid() {
        let req = IngestRequest {
            doc_id: "doc_123".to_string(),
            kb_id: "kb_456".to_string(),
            file_path: "/shared/test.pdf".to_string(),
            source_name: "test.pdf".to_string(),
            mime_type: "application/pdf".to_string(),
            tags: vec![],
        };

        assert!(validate_request(&req).is_ok());
    }

    #[test]
    fn test_validate_request_empty_doc_id() {
        let req = IngestRequest {
            doc_id: "".to_string(),
            kb_id: "kb_456".to_string(),
            file_path: "/shared/test.pdf".to_string(),
            source_name: "test.pdf".to_string(),
            mime_type: "application/pdf".to_string(),
            tags: vec![],
        };

        let result = validate_request(&req);
        assert!(result.is_err());

        if let Err(AppError::BadRequest(msg)) = result {
            assert!(msg.contains("doc_id"));
        } else {
            panic!("Expected BadRequest error");
        }
    }

    #[test]
    fn test_validate_request_invalid_mime() {
        let req = IngestRequest {
            doc_id: "doc_123".to_string(),
            kb_id: "kb_456".to_string(),
            file_path: "/shared/test.mp4".to_string(),
            source_name: "test.mp4".to_string(),
            mime_type: "video/mp4".to_string(),
            tags: vec![],
        };

        let result = validate_request(&req);
        assert!(result.is_err());

        if let Err(AppError::BadRequest(msg)) = result {
            assert!(msg.contains("Unsupported MIME type"));
        } else {
            panic!("Expected BadRequest error");
        }
    }
}
