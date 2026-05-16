// ============================================================================
// Archivio Parlante — Graph-Guided Retrieval
// ============================================================================
// Fase 6.1 - Enhanced Knowledge Graph RAG
//
// Expands query entities 2-hop via knowledge graph to find related chunks,
// then merges with traditional hybrid search using Reciprocal Rank Fusion.

use std::collections::HashSet;
use std::sync::Arc;
use sqlx::{MySqlPool, Row};
use serde::{Deserialize, Serialize};
use crate::errors::AppError;

/// Knowledge graph node from database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub node_type: String,
    pub kb_id: String,
}

/// Knowledge graph edge from database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source_id: String,
    pub target_id: String,
    pub relation_type: String,
    pub confidence: f32,
}

/// Chunk retrieved via graph expansion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphChunk {
    pub chunk_id: String,
    pub text: String,
    pub doc_id: String,
    pub score: f32,
    pub retrieval_path: Vec<String>,  // Entities traversed to reach this chunk
}

/// Graph-guided retrieval component
pub struct GraphRetriever {
    db_pool: Arc<MySqlPool>,
}

impl GraphRetriever {
    /// Create new graph retriever
    pub fn new(db_pool: Arc<MySqlPool>) -> Self {
        Self { db_pool }
    }

    /// Expand entities via knowledge graph
    ///
    /// Given a list of entity labels (from query NER), expand N hops through
    /// the knowledge graph to find related entities.
    ///
    /// # Arguments
    /// * `entity_labels` - Initial entities from query
    /// * `kb_id` - Knowledge base to search within
    /// * `depth` - Number of hops to expand (default: 2)
    ///
    /// # Returns
    /// All entity labels within depth hops (including initial entities)
    pub async fn expand_entities(
        &self,
        entity_labels: Vec<String>,
        kb_id: &str,
        depth: u8,
    ) -> Result<Vec<String>, AppError> {
        if entity_labels.is_empty() {
            return Ok(vec![]);
        }

        let mut visited = HashSet::new();
        let mut current_layer: HashSet<String> = entity_labels.iter().cloned().collect();
        let mut all_entities: HashSet<String> = current_layer.clone();

        for hop in 0..depth {
            if current_layer.is_empty() {
                break;
            }

            tracing::debug!(
                "Graph expansion hop {}/{}, current layer size: {}",
                hop + 1,
                depth,
                current_layer.len()
            );

            // Find nodes matching current layer entities
            let node_ids = self.find_nodes_by_labels(
                current_layer.iter().cloned().collect(),
                kb_id,
            ).await?;

            if node_ids.is_empty() {
                tracing::debug!("No nodes found for current layer, stopping expansion");
                break;
            }

            // Find neighbors of current nodes
            let neighbors = self.find_neighbors(&node_ids, kb_id).await?;

            // Mark current layer as visited
            visited.extend(current_layer.clone());

            // Prepare next layer (neighbors not yet visited)
            let next_layer: HashSet<String> = neighbors
                .into_iter()
                .filter(|label| !visited.contains(label))
                .collect();

            all_entities.extend(next_layer.clone());
            current_layer = next_layer;
        }

        let result: Vec<String> = all_entities.into_iter().collect();
        tracing::info!(
            "Expanded {} entities to {} entities via {}-hop graph traversal",
            entity_labels.len(),
            result.len(),
            depth
        );

        Ok(result)
    }

    /// Retrieve chunks associated with expanded entities
    ///
    /// # Arguments
    /// * `entity_labels` - Entity labels to find chunks for
    /// * `kb_id` - Knowledge base ID
    ///
    /// # Returns
    /// Chunks that mention these entities, ranked by relevance
    pub async fn retrieve_chunks_by_entities(
        &self,
        entity_labels: Vec<String>,
        kb_id: &str,
    ) -> Result<Vec<GraphChunk>, AppError> {
        if entity_labels.is_empty() {
            return Ok(vec![]);
        }

        // Build query to find chunks mentioning these entities
        // Uses LIKE for fuzzy matching (entity mentions might not be exact)
        let _placeholders: Vec<String> = entity_labels
            .iter()
            .map(|_| "?".to_string())
            .collect();

        let like_conditions: Vec<String> = entity_labels
            .iter()
            .map(|_| "chunk_text LIKE CONCAT('%', ?, '%')".to_string())
            .collect();

        let query_str = format!(
            r#"
            SELECT DISTINCT
                c.id as chunk_id,
                c.chunk_text as text,
                c.document_id as doc_id,
                COUNT(DISTINCT n.label) as entity_match_count
            FROM ap_chunks c
            CROSS JOIN ap_graph_nodes n
            WHERE c.kb_id = ?
              AND n.kb_id = ?
              AND ({})
            GROUP BY c.id, c.chunk_text, c.document_id
            ORDER BY entity_match_count DESC, c.id
            LIMIT 50
            "#,
            like_conditions.join(" OR ")
        );

        let mut query = sqlx::query(&query_str)
            .bind(kb_id)
            .bind(kb_id);

        // Bind entity labels for LIKE conditions
        for label in &entity_labels {
            query = query.bind(label);
        }

        let rows = query
            .fetch_all(self.db_pool.as_ref())
            .await
            .map_err(|e| {
                tracing::error!("Database error retrieving chunks by entities: {}", e);
                AppError::InternalError(format!("Database error: {}", e))
            })?;

        let chunks: Vec<GraphChunk> = rows
            .into_iter()
            .map(|row| {
                let match_count: i64 = row.get::<i64, _>("entity_match_count");
                let score = (match_count as f32) / (entity_labels.len() as f32);

                GraphChunk {
                    chunk_id: row.get::<String, _>("chunk_id"),
                    text: row.get::<String, _>("text"),
                    doc_id: row.get::<String, _>("doc_id"),
                    score,
                    retrieval_path: entity_labels.clone(),  // Simplified: all entities
                }
            })
            .collect();

        tracing::info!(
            "Retrieved {} chunks via graph-guided search ({} entities)",
            chunks.len(),
            entity_labels.len()
        );

        Ok(chunks)
    }

    /// Find graph node IDs by entity labels
    async fn find_nodes_by_labels(
        &self,
        labels: Vec<String>,
        kb_id: &str,
    ) -> Result<Vec<String>, AppError> {
        if labels.is_empty() {
            return Ok(vec![]);
        }

        let placeholders: Vec<&str> = labels.iter().map(|_| "?").collect();
        let query_str = format!(
            "SELECT id FROM ap_graph_nodes WHERE kb_id = ? AND label IN ({})",
            placeholders.join(",")
        );

        let mut query = sqlx::query(&query_str).bind(kb_id);
        for label in labels {
            query = query.bind(label);
        }

        let rows = query
            .fetch_all(self.db_pool.as_ref())
            .await
            .map_err(|e| AppError::InternalError(format!("Database error: {}", e)))?;

        let node_ids: Vec<String> = rows
            .into_iter()
            .map(|row| row.get::<String, _>("id"))
            .collect();

        Ok(node_ids)
    }

    /// Find neighbor entity labels for given node IDs
    async fn find_neighbors(
        &self,
        node_ids: &[String],
        kb_id: &str,
    ) -> Result<Vec<String>, AppError> {
        if node_ids.is_empty() {
            return Ok(vec![]);
        }

        let placeholders: Vec<&str> = node_ids.iter().map(|_| "?").collect();

        // Find both outgoing and incoming edges
        let query_str = format!(
            r#"
            SELECT DISTINCT n.label
            FROM ap_graph_edges e
            INNER JOIN ap_graph_nodes n ON (e.target_id = n.id OR e.source_id = n.id)
            WHERE (e.source_id IN ({}) OR e.target_id IN ({}))
              AND n.kb_id = ?
            "#,
            placeholders.join(","),
            placeholders.join(",")
        );

        let mut query = sqlx::query(&query_str);

        // Bind node_ids twice (for source and target)
        for node_id in node_ids {
            query = query.bind(node_id);
        }
        for node_id in node_ids {
            query = query.bind(node_id);
        }
        query = query.bind(kb_id);

        let rows = query
            .fetch_all(self.db_pool.as_ref())
            .await
            .map_err(|e| AppError::InternalError(format!("Database error: {}", e)))?;

        let labels: Vec<String> = rows
            .into_iter()
            .map(|row| row.get::<String, _>("label"))
            .collect();

        Ok(labels)
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_graph_retriever_creation() {
        // Test that GraphRetriever can be created (requires DB connection in real tests)
        // This is a placeholder - real tests would use test database
    }
}
