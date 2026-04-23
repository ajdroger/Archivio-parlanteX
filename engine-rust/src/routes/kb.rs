/// Knowledge base management routes

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

use crate::clients::qdrant::QdrantWrapper;
use crate::errors::{AppError, Result};
use crate::routes::ingest::AppState;

/// Document metadata
#[derive(Debug, Serialize)]
pub struct DocumentInfo {
    pub doc_id: String,
    pub source_name: String,
    pub mime_type: String,
    pub chunks_count: usize,
    pub uploaded_at: String,
    pub tags: Vec<String>,
}

/// Knowledge graph node
#[derive(Debug, Serialize)]
pub struct GraphNode {
    pub id: String,
    pub entity_type: String,
    pub name: String,
    pub properties: serde_json::Value,
}

/// Knowledge graph edge
#[derive(Debug, Serialize)]
pub struct GraphEdge {
    pub source_id: String,
    pub target_id: String,
    pub relation_type: String,
    pub properties: serde_json::Value,
}

/// Knowledge base statistics
#[derive(Debug, Serialize)]
pub struct KbStats {
    pub kb_id: String,
    pub documents_count: usize,
    pub chunks_count: usize,
    pub collection_size_bytes: u64,
    pub nodes_count: usize,
    pub edges_count: usize,
}

/// Query parameters for graph endpoint
#[derive(Debug, Deserialize)]
pub struct GraphQuery {
    /// Filter by document IDs (comma-separated)
    #[serde(default)]
    pub doc_ids: Option<String>,
}

/// List documents in KB
///
/// GET /kb/{kb_id}/documents
pub async fn list_documents(
    State(_state): State<AppState>,
    Path(kb_id): Path<String>,
) -> Result<Json<Vec<DocumentInfo>>> {
    tracing::info!(kb_id = %kb_id, "Listing documents");

    // TODO: Query MySQL ap_documents table
    // For now, return placeholder
    tracing::warn!("list_documents not fully implemented (requires MySQL integration)");

    Ok(Json(vec![]))
}

/// Delete document from KB
///
/// DELETE /kb/{kb_id}/documents/{doc_id}
///
/// Removes from:
/// - Qdrant (all chunks)
/// - MySQL (ap_documents, ap_graph_nodes, ap_graph_edges)
pub async fn delete_document(
    State(state): State<AppState>,
    Path((kb_id, doc_id)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    tracing::info!(kb_id = %kb_id, doc_id = %doc_id, "Deleting document");

    // Step 1: Delete from Qdrant
    let collection_name = format!("ap_kb_{}", kb_id);
    let qdrant = QdrantWrapper::new(
        state.config.qdrant_url.clone(),
        collection_name,
        768,
    )
    .await?;

    qdrant.delete_by_doc_id(doc_id.clone()).await?;

    tracing::info!(kb_id = %kb_id, doc_id = %doc_id, "Document deleted from Qdrant");

    // TODO: Step 2: Delete from MySQL
    // DELETE FROM ap_documents WHERE doc_id = ?
    // DELETE FROM ap_graph_nodes WHERE doc_id = ?
    // DELETE FROM ap_graph_edges WHERE source_doc_id = ? OR target_doc_id = ?

    tracing::warn!("MySQL deletion not implemented yet");

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Document deleted successfully",
            "doc_id": doc_id,
            "kb_id": kb_id
        })),
    ))
}

/// Get knowledge graph for KB
///
/// GET /kb/{kb_id}/graph?doc_ids=X,Y
pub async fn get_graph(
    State(_state): State<AppState>,
    Path(kb_id): Path<String>,
    Query(params): Query<GraphQuery>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        kb_id = %kb_id,
        doc_ids = ?params.doc_ids,
        "Fetching knowledge graph"
    );

    // TODO: Query MySQL ap_graph_nodes and ap_graph_edges
    // Filter by doc_ids if provided

    tracing::warn!("get_graph not fully implemented (requires MySQL + KG extraction)");

    // Placeholder response
    Ok(Json(json!({
        "nodes": [],
        "edges": [],
        "filtered_by_docs": params.doc_ids
    })))
}

/// Get KB statistics
///
/// GET /kb/{kb_id}/stats
pub async fn get_stats(
    State(state): State<AppState>,
    Path(kb_id): Path<String>,
) -> Result<Json<KbStats>> {
    tracing::info!(kb_id = %kb_id, "Fetching KB statistics");

    // Query Qdrant for collection info
    let collection_name = format!("ap_kb_{}", kb_id);
    let qdrant = QdrantWrapper::new(
        state.config.qdrant_url.clone(),
        collection_name.clone(),
        768,
    )
    .await?;

    // TODO: Get actual collection stats from Qdrant
    // For now, return placeholder
    let chunks_count = 0; // qdrant.count_points().await?;

    // TODO: Query MySQL for documents count
    let documents_count = 0;

    // TODO: Query MySQL for graph stats
    let nodes_count = 0;
    let edges_count = 0;

    tracing::warn!("get_stats returning placeholder data (requires full integration)");

    Ok(Json(KbStats {
        kb_id,
        documents_count,
        chunks_count,
        collection_size_bytes: 0,
        nodes_count,
        edges_count,
    }))
}

/// Reindex KB (background job)
///
/// POST /admin/reindex/{kb_id}
///
/// Triggers re-embedding of all chunks with current model
pub async fn reindex_kb(
    State(_state): State<AppState>,
    Path(kb_id): Path<String>,
) -> Result<impl IntoResponse> {
    tracing::info!(kb_id = %kb_id, "Reindex KB requested");

    // TODO: Implement background job
    // 1. Fetch all documents from MySQL
    // 2. For each document:
    //    - Fetch chunks from Qdrant (or re-chunk)
    //    - Re-embed with current model
    //    - Update Qdrant
    // 3. Return job ID for tracking

    tracing::warn!("reindex_kb not implemented (requires background job system)");

    Ok((
        StatusCode::ACCEPTED,
        Json(json!({
            "message": "Reindex job queued (not implemented yet)",
            "kb_id": kb_id,
            "job_id": "placeholder_job_123"
        })),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_info_serialization() {
        let doc = DocumentInfo {
            doc_id: "doc_123".to_string(),
            source_name: "contract.pdf".to_string(),
            mime_type: "application/pdf".to_string(),
            chunks_count: 15,
            uploaded_at: "2024-01-15T10:30:00Z".to_string(),
            tags: vec!["nda".to_string(), "2024".to_string()],
        };

        let json = serde_json::to_string(&doc).expect("Should serialize");
        assert!(json.contains("doc_123"));
        assert!(json.contains("contract.pdf"));
    }

    #[test]
    fn test_kb_stats_serialization() {
        let stats = KbStats {
            kb_id: "kb_456".to_string(),
            documents_count: 10,
            chunks_count: 250,
            collection_size_bytes: 1024000,
            nodes_count: 45,
            edges_count: 78,
        };

        let json = serde_json::to_string(&stats).expect("Should serialize");
        assert!(json.contains("kb_456"));
        assert!(json.contains("250"));
    }

    #[test]
    fn test_graph_query_parsing() {
        let query_str = "doc_ids=doc_1,doc_2,doc_3";
        let parsed: GraphQuery =
            serde_urlencoded::from_str(query_str).expect("Should parse");

        assert!(parsed.doc_ids.is_some());
        assert_eq!(parsed.doc_ids.unwrap(), "doc_1,doc_2,doc_3");
    }

    #[test]
    fn test_graph_query_empty() {
        let query_str = "";
        let parsed: GraphQuery =
            serde_urlencoded::from_str(query_str).expect("Should parse");

        assert!(parsed.doc_ids.is_none());
    }
}
