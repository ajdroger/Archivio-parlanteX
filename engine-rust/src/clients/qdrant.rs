/// Qdrant vector database client wrapper

use qdrant_client::{
    client::QdrantClient as RawQdrantClient,
    qdrant::{
        vectors_config::Config as VectorsConfig, CreateCollectionBuilder, Distance,
        PointStruct, ScoredPoint, SearchPointsBuilder, VectorParamsBuilder, VectorsConfigBuilder,
        SearchParamsBuilder, SparseVectorParamsBuilder, SparseIndices, SparseVectorConfig,
        NamedVectors, Vector as QdrantVector, Value, Condition, Filter,
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
        let dense_config = VectorParamsBuilder::new(self.dense_vector_size, Distance::Cosine);

        // Sparse vector config (BM25 for keyword search)
        let sparse_config = SparseVectorParamsBuilder::default();

        let mut vectors_config = HashMap::new();
        vectors_config.insert("dense".to_string(), dense_config.build());

        let create_request = CreateCollectionBuilder::new(&self.collection_name)
            .vectors_config(VectorsConfigBuilder::default().params_map(vectors_config))
            .sparse_vectors_config([("sparse".to_string(), sparse_config)])
            .build();

        self.client
            .create_collection(create_request)
            .await
            .map_err(|e| AppError::Qdrant(format!("Failed to create collection: {}", e)))?;

        tracing::info!(collection = %self.collection_name, "Collection created successfully");

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
                let mut payload = HashMap::new();
                payload.insert("text".to_string(), chunk.text.into());
                payload.insert("doc_id".to_string(), chunk.doc_id.into());
                payload.insert("chunk_index".to_string(), (chunk.chunk_index as i64).into());

                if let Some(metadata) = chunk.metadata {
                    payload.insert("metadata".to_string(), serde_json::to_string(&metadata).unwrap_or_default().into());
                }

                // Dense vector
                let mut named_vectors = HashMap::new();
                named_vectors.insert("dense".to_string(), chunk.dense_embedding.into());

                // Sparse vector (BM25 term weights from Python worker)
                if let Some(sparse) = chunk.sparse_vector {
                    let sparse_vec = qdrant_client::qdrant::Vector {
                        data: Some(qdrant_client::qdrant::vector::Data::Sparse(
                            qdrant_client::qdrant::SparseVector {
                                indices: sparse.indices,
                                values: sparse.values,
                            },
                        )),
                    };
                    named_vectors.insert("sparse".to_string(), sparse_vec);
                }

                PointStruct::new(
                    chunk.id.clone(),
                    named_vectors,
                    payload,
                )
            })
            .collect();

        tracing::debug!(points_count = points.len(), "Upserting chunks to Qdrant");

        self.client
            .upsert_points(&self.collection_name, None, points, None)
            .await
            .map_err(|e| AppError::Qdrant(format!("Failed to upsert chunks: {}", e)))?;

        tracing::info!(
            collection = %self.collection_name,
            chunks_count = points.len(),
            "Chunks upserted successfully"
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
        let sparse_vec = QdrantVector {
            data: Some(qdrant_client::qdrant::vector::Data::Sparse(
                qdrant_client::qdrant::SparseVector {
                    indices: query_sparse.indices,
                    values: query_sparse.values,
                },
            )),
        };

        let search = SearchPointsBuilder::new(
            &self.collection_name,
            sparse_vec,
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

        self.client
            .delete_points(&self.collection_name, None, &filter.into(), None)
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
            .unwrap_or("")
            .to_string();

        let doc_id = payload
            .get("doc_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let chunk_index = payload
            .get("chunk_index")
            .and_then(|v| v.as_integer())
            .unwrap_or(0) as usize;

        ScoredChunk {
            id: point.id.map(|id| id.to_string()).unwrap_or_default(),
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
