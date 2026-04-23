/// Multi-contract comparison route handler

use axum::{extract::State, Json};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use crate::clients::qdrant::QdrantWrapper;
use crate::errors::{AppError, Result};
use crate::models::comparison::{CompareRequest, CompareResponse};
use crate::rag::multi_contract::MultiContractComparator;
use crate::routes::ingest::AppState;

/// Compare multiple contracts
///
/// Pipeline:
/// 1. Validate request (min 2 docs, kb_id valid)
/// 2. Create Qdrant client for KB
/// 3. Create MultiContractComparator
/// 4. Run comparison
/// 5. Generate Markdown result
/// 6. Optionally save to database (TODO: Phase 1.6)
/// 7. Return response
///
/// # Errors
/// Returns appropriate AppError if any step fails
pub async fn handle_compare_contracts(
    State(state): State<AppState>,
    Json(req): Json<CompareRequest>,
) -> Result<Json<CompareResponse>> {
    let start = Instant::now();

    tracing::info!(
        kb_id = %req.kb_id,
        doc_count = req.doc_ids.len(),
        question = %req.question,
        save_analysis = req.save_analysis,
        "Starting contract comparison"
    );

    // Step 1: Validate request
    validate_request(&req)?;

    // Step 2: Create Qdrant client for this KB
    let collection_name = format!("ap_kb_{}", req.kb_id);
    let qdrant = Arc::new(
        QdrantWrapper::new(state.config.qdrant_url.clone(), collection_name, 768).await?,
    );

    // Step 3: Create comparator
    let llm = state.llm_registry.default();
    let comparator = MultiContractComparator::new(qdrant, llm);

    // Step 4: Run comparison
    let mut comparison_result = comparator.compare(req.doc_ids.clone(), &req.question).await?;

    // Update processing time to include total request handling
    let total_processing_ms = start.elapsed().as_millis() as u64;
    comparison_result.processing_ms = total_processing_ms;

    // Step 5: Generate Markdown
    let doc_names = build_doc_names_map(&req.doc_ids);
    let markdown_result = comparison_result.to_markdown(&doc_names);

    // Step 6: Optionally save to database
    // TODO: Phase 1.6 - save to ap_contract_analyses table
    let analysis_id = if req.save_analysis {
        // Placeholder for future implementation
        tracing::warn!("save_analysis=true but persistence not yet implemented (Phase 1.6)");
        None
    } else {
        None
    };

    tracing::info!(
        processing_ms = total_processing_ms,
        aspects_count = comparison_result.aspects.len(),
        "Comparison completed successfully"
    );

    Ok(Json(CompareResponse {
        markdown_result,
        structured: comparison_result,
        processing_ms: total_processing_ms,
        analysis_id,
    }))
}

/// Validate comparison request
fn validate_request(req: &CompareRequest) -> Result<()> {
    if req.kb_id.is_empty() {
        return Err(AppError::BadRequest("kb_id cannot be empty".to_string()));
    }

    if req.doc_ids.len() < 2 {
        return Err(AppError::BadRequest(
            "At least 2 documents required for comparison".to_string(),
        ));
    }

    if req.doc_ids.len() > 10 {
        return Err(AppError::BadRequest(
            "Maximum 10 documents can be compared at once".to_string(),
        ));
    }

    // Check for duplicate doc_ids
    let mut seen = std::collections::HashSet::new();
    for doc_id in &req.doc_ids {
        if !seen.insert(doc_id) {
            return Err(AppError::BadRequest(format!(
                "Duplicate document ID: {}",
                doc_id
            )));
        }
    }

    if req.question.trim().is_empty() {
        return Err(AppError::BadRequest(
            "question cannot be empty".to_string(),
        ));
    }

    Ok(())
}

/// Build doc_id -> display_name mapping
fn build_doc_names_map(doc_ids: &[String]) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for (i, doc_id) in doc_ids.iter().enumerate() {
        // For now, use doc_id as display name
        // In Phase 1.6, fetch from database
        let display_name = format!("Contratto {}", (b'A' + i as u8) as char);
        map.insert(doc_id.clone(), display_name);
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_request_valid() {
        let req = CompareRequest {
            kb_id: "kb_123".to_string(),
            doc_ids: vec!["doc_1".to_string(), "doc_2".to_string()],
            question: "Confronta le clausole di riservatezza".to_string(),
            save_analysis: false,
        };

        assert!(validate_request(&req).is_ok());
    }

    #[test]
    fn test_validate_request_empty_kb_id() {
        let req = CompareRequest {
            kb_id: "".to_string(),
            doc_ids: vec!["doc_1".to_string(), "doc_2".to_string()],
            question: "test".to_string(),
            save_analysis: false,
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
    fn test_validate_request_insufficient_docs() {
        let req = CompareRequest {
            kb_id: "kb_123".to_string(),
            doc_ids: vec!["doc_1".to_string()],
            question: "test".to_string(),
            save_analysis: false,
        };

        let result = validate_request(&req);
        assert!(result.is_err());

        if let Err(AppError::BadRequest(msg)) = result {
            assert!(msg.contains("At least 2 documents"));
        } else {
            panic!("Expected BadRequest error");
        }
    }

    #[test]
    fn test_validate_request_too_many_docs() {
        let req = CompareRequest {
            kb_id: "kb_123".to_string(),
            doc_ids: (0..11).map(|i| format!("doc_{}", i)).collect(),
            question: "test".to_string(),
            save_analysis: false,
        };

        let result = validate_request(&req);
        assert!(result.is_err());

        if let Err(AppError::BadRequest(msg)) = result {
            assert!(msg.contains("Maximum 10 documents"));
        } else {
            panic!("Expected BadRequest error");
        }
    }

    #[test]
    fn test_validate_request_duplicate_docs() {
        let req = CompareRequest {
            kb_id: "kb_123".to_string(),
            doc_ids: vec![
                "doc_1".to_string(),
                "doc_2".to_string(),
                "doc_1".to_string(),
            ],
            question: "test".to_string(),
            save_analysis: false,
        };

        let result = validate_request(&req);
        assert!(result.is_err());

        if let Err(AppError::BadRequest(msg)) = result {
            assert!(msg.contains("Duplicate document ID"));
        } else {
            panic!("Expected BadRequest error");
        }
    }

    #[test]
    fn test_validate_request_empty_question() {
        let req = CompareRequest {
            kb_id: "kb_123".to_string(),
            doc_ids: vec!["doc_1".to_string(), "doc_2".to_string()],
            question: "   ".to_string(),
            save_analysis: false,
        };

        let result = validate_request(&req);
        assert!(result.is_err());

        if let Err(AppError::BadRequest(msg)) = result {
            assert!(msg.contains("question cannot be empty"));
        } else {
            panic!("Expected BadRequest error");
        }
    }

    #[test]
    fn test_build_doc_names_map() {
        let doc_ids = vec![
            "doc_123".to_string(),
            "doc_456".to_string(),
            "doc_789".to_string(),
        ];

        let map = build_doc_names_map(&doc_ids);

        assert_eq!(map.len(), 3);
        assert_eq!(map.get("doc_123").unwrap(), "Contratto A");
        assert_eq!(map.get("doc_456").unwrap(), "Contratto B");
        assert_eq!(map.get("doc_789").unwrap(), "Contratto C");
    }
}
