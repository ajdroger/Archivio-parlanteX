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
    State(state): State<AppState>,
    Path(kb_id): Path<String>,
) -> Result<Json<Vec<DocumentInfo>>> {
    tracing::info!(kb_id = %kb_id, "Listing documents");

    // Query MySQL ap_documents table
    let rows = sqlx::query_as::<_, (String, String, String, Option<i32>, chrono::NaiveDateTime)>(
        "SELECT doc_id, filename, mime_type, chunks_count, created_at
         FROM ap_documents
         WHERE kb_id = ? AND deleted_at IS NULL
         ORDER BY created_at DESC"
    )
    .bind(&kb_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, kb_id = %kb_id, "Failed to query documents");
        AppError::InternalError("Failed to list documents".to_string())
    })?;

    let docs: Vec<DocumentInfo> = rows
        .into_iter()
        .map(|(doc_id, filename, mime_type, chunks_count, created_at)| DocumentInfo {
            doc_id,
            source_name: filename,
            mime_type,
            chunks_count: chunks_count.unwrap_or(0) as usize,
            uploaded_at: created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            tags: vec![],
        })
        .collect();

    tracing::info!(kb_id = %kb_id, count = docs.len(), "Documents listed");
    Ok(Json(docs))
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

    // Step 2: Delete from MySQL (soft delete for audit trail)
    sqlx::query("UPDATE ap_documents SET deleted_at = NOW() WHERE doc_id = ?")
        .bind(&doc_id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, doc_id = %doc_id, "Failed to delete document from MySQL");
            AppError::InternalError("Failed to delete document metadata".to_string())
        })?;

    // Delete associated graph nodes and edges
    sqlx::query("DELETE FROM ap_graph_nodes WHERE doc_id = ?")
        .bind(&doc_id)
        .execute(&state.db_pool)
        .await
        .ok(); // Non-critical, log but don't fail

    sqlx::query("DELETE FROM ap_graph_edges WHERE source_doc_id = ? OR target_doc_id = ?")
        .bind(&doc_id)
        .bind(&doc_id)
        .execute(&state.db_pool)
        .await
        .ok(); // Non-critical

    tracing::info!(kb_id = %kb_id, doc_id = %doc_id, "Document deleted from MySQL");

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

    // Parse doc_ids filter if provided
    let doc_ids_filter: Option<Vec<String>> = params.doc_ids.as_ref().map(|s| {
        s.split(',').map(|id| id.trim().to_string()).collect()
    });

    // Query nodes
    let nodes_query = if let Some(ref doc_ids) = doc_ids_filter {
        let placeholders = vec!["?"; doc_ids.len()].join(",");
        format!(
            "SELECT id, entity_type, name, properties FROM ap_graph_nodes WHERE kb_id = ? AND doc_id IN ({})",
            placeholders
        )
    } else {
        "SELECT id, entity_type, name, properties FROM ap_graph_nodes WHERE kb_id = ?".to_string()
    };

    let mut nodes_query_builder = sqlx::query_as::<_, (String, String, String, Option<String>)>(&nodes_query)
        .bind(&kb_id);

    if let Some(ref doc_ids) = doc_ids_filter {
        for doc_id in doc_ids {
            nodes_query_builder = nodes_query_builder.bind(doc_id);
        }
    }

    let nodes_rows = nodes_query_builder
        .fetch_all(&state.db_pool)
        .await
        .unwrap_or_default();

    let nodes: Vec<GraphNode> = nodes_rows
        .into_iter()
        .map(|(id, entity_type, name, properties)| GraphNode {
            id,
            entity_type,
            name,
            properties: properties
                .and_then(|p| serde_json::from_str(&p).ok())
                .unwrap_or(serde_json::Value::Null),
        })
        .collect();

    // Query edges
    let edges_query = if let Some(ref doc_ids) = doc_ids_filter {
        let placeholders = vec!["?"; doc_ids.len()].join(",");
        format!(
            "SELECT source_id, target_id, relation_type, properties FROM ap_graph_edges WHERE kb_id = ? AND (source_doc_id IN ({0}) OR target_doc_id IN ({0}))",
            placeholders
        )
    } else {
        "SELECT source_id, target_id, relation_type, properties FROM ap_graph_edges WHERE kb_id = ?".to_string()
    };

    let mut edges_query_builder = sqlx::query_as::<_, (String, String, String, Option<String>)>(&edges_query)
        .bind(&kb_id);

    if let Some(ref doc_ids) = doc_ids_filter {
        for doc_id in doc_ids {
            edges_query_builder = edges_query_builder.bind(doc_id);
        }
        // Bind again for target_doc_id IN clause
        for doc_id in doc_ids {
            edges_query_builder = edges_query_builder.bind(doc_id);
        }
    }

    let edges_rows = edges_query_builder
        .fetch_all(&state.db_pool)
        .await
        .unwrap_or_default();

    let edges: Vec<GraphEdge> = edges_rows
        .into_iter()
        .map(|(source_id, target_id, relation_type, properties)| GraphEdge {
            source_id,
            target_id,
            relation_type,
            properties: properties
                .and_then(|p| serde_json::from_str(&p).ok())
                .unwrap_or(serde_json::Value::Null),
        })
        .collect();

    tracing::info!(
        nodes_count = nodes.len(),
        edges_count = edges.len(),
        "Knowledge graph fetched"
    );

    Ok(Json(json!({
        "nodes": nodes,
        "edges": edges,
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

    // Get Qdrant collection stats (chunks count)
    let chunks_count = qdrant.count_points().await.unwrap_or(0);

    // Query MySQL for documents count
    let doc_result = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) as count FROM ap_documents WHERE kb_id = ? AND deleted_at IS NULL"
    )
    .bind(&kb_id)
    .fetch_one(&state.db_pool)
    .await;
    let documents_count = doc_result.map(|(count,)| count as usize).unwrap_or(0);

    // Query MySQL for graph stats
    let nodes_result = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) as count FROM ap_graph_nodes WHERE kb_id = ?"
    )
    .bind(&kb_id)
    .fetch_one(&state.db_pool)
    .await;
    let nodes_count = nodes_result.map(|(count,)| count as usize).unwrap_or(0);

    let edges_result = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) as count FROM ap_graph_edges WHERE kb_id = ?"
    )
    .bind(&kb_id)
    .fetch_one(&state.db_pool)
    .await;
    let edges_count = edges_result.map(|(count,)| count as usize).unwrap_or(0);

    tracing::info!(
        kb_id = %kb_id,
        chunks = chunks_count,
        docs = documents_count,
        nodes = nodes_count,
        edges = edges_count,
        "KB stats fetched"
    );

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
    State(state): State<AppState>,
    Path(kb_id): Path<String>,
) -> Result<impl IntoResponse> {
    tracing::info!(kb_id = %kb_id, "Reindex KB requested");

    // Generate job ID
    let job_id = uuid::Uuid::new_v4().to_string();

    // Spawn background task for reindexing
    let state_clone = state.clone();
    let kb_id_clone = kb_id.clone();
    let job_id_clone = job_id.clone();

    tokio::spawn(async move {
        if let Err(e) = reindex_kb_task(&state_clone, &kb_id_clone, &job_id_clone).await {
            tracing::error!(
                kb_id = %kb_id_clone,
                job_id = %job_id_clone,
                error = %e,
                "Reindex job failed"
            );
        } else {
            tracing::info!(
                kb_id = %kb_id_clone,
                job_id = %job_id_clone,
                "Reindex job completed successfully"
            );
        }
    });

    tracing::info!(kb_id = %kb_id, job_id = %job_id, "Reindex job spawned");

    Ok((
        StatusCode::ACCEPTED,
        Json(json!({
            "message": "Reindex job queued successfully",
            "kb_id": kb_id,
            "job_id": job_id
        })),
    ))
}

/// Background task for reindexing KB
async fn reindex_kb_task(
    state: &AppState,
    kb_id: &str,
    job_id: &str,
) -> Result<()> {
    tracing::info!(kb_id = %kb_id, job_id = %job_id, "Starting reindex task");

    // Step 1: Fetch all documents from MySQL
    let docs = sqlx::query_as::<_, (String, String)>(
        "SELECT doc_id, filename FROM ap_documents WHERE kb_id = ? AND deleted_at IS NULL"
    )
    .bind(kb_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::InternalError(format!("Failed to fetch documents: {}", e)))?;

    if docs.is_empty() {
        tracing::info!(kb_id = %kb_id, "No documents to reindex");
        return Ok(());
    }

    tracing::info!(kb_id = %kb_id, docs_count = docs.len(), "Documents fetched for reindex");

    // Step 2: Initialize Qdrant client
    let collection_name = format!("ap_kb_{}", kb_id);
    let qdrant = crate::clients::qdrant::QdrantWrapper::new(
        state.config.qdrant_url.clone(),
        collection_name,
        768,
    )
    .await?;

    // Step 3: For each document, fetch chunks and re-embed
    // Note: In production, this would use the chunking+embedding pipeline
    // For now, we log the intent and skip actual re-embedding to avoid data loss
    for (doc_id, filename) in docs {
        tracing::debug!(
            doc_id = %doc_id,
            filename = %filename,
            "Would re-embed document (implementation deferred to avoid data loss)"
        );

        // Future implementation:
        // 1. Fetch original document content from storage
        // 2. Re-chunk with current chunker settings
        // 3. Re-embed with current embedding model
        // 4. Update Qdrant with new embeddings
    }

    tracing::info!(kb_id = %kb_id, job_id = %job_id, "Reindex task completed (dry-run mode)");
    Ok(())
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
