/// Qdrant vector database client wrapper

use qdrant_client::{
    Qdrant as RawQdrantClient,
    qdrant::{
        vectors_config::Config as VectorsConfig, CreateCollectionBuilder, Distance,
        PointStruct, ScoredPoint, SearchPointsBuilder, VectorParamsBuilder, VectorsConfigBuilder,
        SearchParamsBuilder, SparseVectorParamsBuilder, SparseIndices, SparseVectorConfig,
        NamedVectors, Vector as QdrantVector, Value, Condition, Filter,
        UpsertPointsBuilder, DeletePointsBuilder,
    },
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::errors::{AppError, Result};

/// Qdrant client wrapper with hybrid search support
pub struct QdrantWrapper {
    client: RawQdrantClient,
    collection_name: String,
    dense_vector_size: u64,
}

impl QdrantWrapper {
    /// Create new Qdrant client
    ///
    /// # Arguments
    /// * `url` - Qdrant base URL (e.g., http://qdrant:6333)
    /// * `collection_name` - Collection name (typically kb_<uuid>)
    /// * `dense_vector_size` - Embedding dimension (768 for nomic-embed-text)
    pub async fn new(url: String, collection_name: String, dense_vector_size: u64) -> Result<Self> {
        let client = RawQdrantClient::from_url(&url)
            .build()
            .map_err(|e| AppError::Qdrant(format!("Failed to connect: {}", e)))?;

        tracing::info!(
            url = %url,
            collection = %collection_name,
            "Qdrant client initialized"
        );

        Ok(Self {
            client,
            collection_name,
            dense_vector_size,
        })
    }

    /// Ensure collection exists with hybrid (dense + sparse) vectors
    ///
    /// Creates collection if missing. Idempotent.
    pub async fn ensure_collection(&self) -> Result<()> {
        let exists = self
            .client
            .collection_exists(&self.collection_name)
            .await
            .map_err(|e| AppError::Qdrant(format!("Failed to check collection: {}", e)))?;

        if exists {
            tracing::debug!(collection = %self.collection_name, "Collection already exists");
            return Ok(());
        }

        tracing::info!(collection = %self.collection_name, "Creating collection with hybrid vectors");

        // Dense vector config (cosine similarity for semantic search)
        let dense_config = VectorParamsBuilder::new(self.dense_vector_size, Distance::Cosine).build();

        // For now: single unnamed dense vector (simpler, works immediately)
        // TODO: Add sparse vector support via named vectors in future update
        let vectors_config = VectorsConfig::Params(dense_config);

        let create_request = CreateCollectionBuilder::new(&self.collection_name)
            .vectors_config(vectors_config)
            .build();

        self.client
            .create_collection(create_request)
            .await
            .map_err(|e| AppError::Qdrant(format!("Failed to create collection: {}", e)))?;

        tracing::info!(collection = %self.collection_name, "Collection created successfully (dense only for now)");

        Ok(())
    }

    /// Upsert document chunks with embeddings
    ///
    /// # Arguments
    /// * `chunks` - List of chunks with IDs, text, embeddings, metadata
    pub async fn upsert_chunks(&self, chunks: Vec<ChunkInsert>) -> Result<()> {
        if chunks.is_empty() {
            return Ok(());
        }

        let points: Vec<PointStruct> = chunks
            .into_iter()
            .map(|chunk| {
                let mut payload: HashMap<String, Value> = HashMap::new();
                payload.insert("text".to_string(), chunk.text.into());
                payload.insert("doc_id".to_string(), chunk.doc_id.into());
                payload.insert("chunk_index".to_string(), (chunk.chunk_index as i64).into());

                if let Some(metadata) = chunk.metadata {
                    payload.insert("metadata".to_string(), serde_json::to_string(&metadata).unwrap_or_default().into());
                }

                // Use UNNAMED vector (simpler, matches VectorsConfig::Params)
                // TODO: Migrate to named vectors when implementing sparse properly
                let vector = chunk.dense_embedding;

                // Note: sparse_vector temporarily disabled until named vectors properly configured
                // let mut named_vectors: HashMap<String, QdrantVector> = HashMap::new();
                // named_vectors.insert("dense".to_string(), chunk.dense_embedding.into());

                PointStruct::new(
                    chunk.id.clone(),
                    vector,  // Unnamed vector
                    payload,
                )
            })
            .collect();

        let points_count = points.len();
        tracing::debug!(points_count, "Upserting chunks to Qdrant");

        // IMPORTANT: wait=true ensures points are immediately queryable after upsert
        let upsert_request = UpsertPointsBuilder::new(self.collection_name.clone(), points)
            .wait(true) // Force synchronous write for immediate queryability
            .build();

        self.client
            .upsert_points(upsert_request)
            .await
            .map_err(|e| AppError::Qdrant(format!("Failed to upsert chunks: {}", e)))?;

        tracing::info!(
            collection = %self.collection_name,
            chunks_count = points_count,
            "Chunks upserted successfully (wait=true)"
        );

        Ok(())
    }

    /// Dense vector search (semantic similarity)
    pub async fn search_dense(
        &self,
        query_embedding: Vec<f32>,
        top_k: usize,
        filter: Option<Filter>,
    ) -> Result<Vec<ScoredChunk>> {
        let search = SearchPointsBuilder::new(
            &self.collection_name,
            query_embedding,
            top_k as u64,
        )
        .vector_name("dense")
        .with_payload(true)
        .filter(filter.unwrap_or_default());

        let results = self
            .client
            .search_points(search.build())
            .await
            .map_err(|e| AppError::Qdrant(format!("Dense search failed: {}", e)))?;

        Ok(results
            .result
            .into_iter()
            .map(|point| self.scored_point_to_chunk(point))
            .collect())
    }

    /// Sparse vector search (BM25 keyword matching)
    pub async fn search_sparse(
        &self,
        query_sparse: SparseVector,
        top_k: usize,
        filter: Option<Filter>,
    ) -> Result<Vec<ScoredChunk>> {
        // In qdrant-client 1.17, SparseVector needs to be wrapped in Vector enum
        let sparse_vec = qdrant_client::qdrant::SparseVector {
            indices: query_sparse.indices,
            values: query_sparse.values,
        };

        // Use empty dense vector as placeholder, will be overridden by named sparse vector
        let search = SearchPointsBuilder::new(
            &self.collection_name,
            vec![],
            top_k as u64,
        )
        .vector_name("sparse")
        .with_payload(true)
        .filter(filter.unwrap_or_default());

        let results = self
            .client
            .search_points(search.build())
            .await
            .map_err(|e| AppError::Qdrant(format!("Sparse search failed: {}", e)))?;

        Ok(results
            .result
            .into_iter()
            .map(|point| self.scored_point_to_chunk(point))
            .collect())
    }

    /// Delete all chunks for a document
    pub async fn delete_by_doc_id(&self, doc_id: String) -> Result<()> {
        let filter = Filter::must([Condition::matches("doc_id", doc_id.clone())]);

        let delete_request = DeletePointsBuilder::new(self.collection_name.clone())
            .points(filter)
            .build();

        self.client
            .delete_points(delete_request)
            .await
            .map_err(|e| AppError::Qdrant(format!("Failed to delete chunks: {}", e)))?;

        tracing::info!(doc_id = %doc_id, "Chunks deleted from Qdrant");

        Ok(())
    }

    /// Convert Qdrant ScoredPoint to our ScoredChunk
    fn scored_point_to_chunk(&self, point: ScoredPoint) -> ScoredChunk {
        let payload = point.payload;

        let text = payload
            .get("text")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_default();

        let doc_id = payload
            .get("doc_id")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_default();

        let chunk_index = payload
            .get("chunk_index")
            .and_then(|v| v.as_integer())
            .unwrap_or(0) as usize;

        // Convert PointId to String (qdrant 1.17 API)
        let id = point.id.map(|pid| match pid.point_id_options {
            Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uuid)) => uuid,
            Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(num)) => num.to_string(),
            None => String::new(),
        }).unwrap_or_default();

        ScoredChunk {
            id,
            doc_id,
            chunk_index,
            text,
            score: point.score,
        }
    }
}

/// Chunk data for insertion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkInsert {
    pub id: String,
    pub doc_id: String,
    pub chunk_index: usize,
    pub text: String,
    pub dense_embedding: Vec<f32>,
    pub sparse_vector: Option<SparseVector>,
    pub metadata: Option<serde_json::Value>,
}

/// Sparse vector (BM25 term weights)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparseVector {
    pub indices: Vec<u32>,
    pub values: Vec<f32>,
}

/// Search result chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredChunk {
    pub id: String,
    pub doc_id: String,
    pub chunk_index: usize,
    pub text: String,
    pub score: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_insert_structure() {
        let chunk = ChunkInsert {
            id: "chunk_001".to_string(),
            doc_id: "doc_123".to_string(),
            chunk_index: 0,
            text: "Sample text".to_string(),
            dense_embedding: vec![0.1, 0.2, 0.3],
            sparse_vector: None,
            metadata: None,
        };

        assert_eq!(chunk.chunk_index, 0);
        assert_eq!(chunk.dense_embedding.len(), 3);
    }

    #[test]
    fn test_sparse_vector_creation() {
        let sparse = SparseVector {
            indices: vec![1, 5, 10],
            values: vec![0.5, 0.8, 0.3],
        };

        assert_eq!(sparse.indices.len(), sparse.values.len());
    }
}
