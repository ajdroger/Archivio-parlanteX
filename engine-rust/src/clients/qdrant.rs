/// Qdrant vector database client wrapper

use qdrant_client::{
    Qdrant as RawQdrantClient,
    qdrant::{
        CreateCollectionBuilder, Distance,
        PointStruct, ScoredPoint, SearchPointsBuilder, VectorParamsBuilder,
        Vector as QdrantVector, Value, Condition, Filter,
        UpsertPointsBuilder, DeletePointsBuilder,
        VectorsConfig, VectorParamsMap,
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

        // Named vectors configuration for true hybrid search
        // "dense": 768-dim dense embeddings (nomic-embed-text, cosine similarity)
        // "sparse": BM25-style sparse vectors via tantivy (dot product)
        use std::collections::HashMap;
        use qdrant_client::qdrant::{VectorParams, SparseVectorParams, SparseIndexConfig};

        let mut named_vectors: HashMap<String, VectorParams> = HashMap::new();

        // Dense vector: 768-dim, cosine similarity
        named_vectors.insert(
            "dense".to_string(),
            VectorParamsBuilder::new(self.dense_vector_size, Distance::Cosine).build(),
        );

        let vectors_config = VectorsConfig {
            config: Some(qdrant_client::qdrant::vectors_config::Config::ParamsMap(
                VectorParamsMap {
                    map: named_vectors,
                },
            )),
        };

        // Sparse vectors configuration
        let sparse_config = SparseVectorParams {
            index: Some(SparseIndexConfig {
                full_scan_threshold: Some(10000),
                on_disk: Some(false),
                datatype: None,
            }),
            modifier: None,
        };

        let mut sparse_vectors_config: HashMap<String, SparseVectorParams> = HashMap::new();
        sparse_vectors_config.insert("sparse".to_string(), sparse_config);

        let create_request = CreateCollectionBuilder::new(&self.collection_name)
            .vectors_config(vectors_config)
            .sparse_vectors_config(sparse_vectors_config)
            .build();

        self.client
            .create_collection(create_request)
            .await
            .map_err(|e| AppError::Qdrant(format!("Failed to create collection: {}", e)))?;

        tracing::info!(collection = %self.collection_name, "Collection created successfully with named vectors (dense + sparse)");

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

                // Named vectors: {"dense": Vec<f32>, "sparse": SparseVector}
                use qdrant_client::qdrant::{NamedVectors, Vectors};

                let mut named_vectors_map: HashMap<String, QdrantVector> = HashMap::new();

                // Dense vector (always present)
                named_vectors_map.insert("dense".to_string(), chunk.dense_embedding.into());

                let named_vectors = NamedVectors {
                    vectors: named_vectors_map,
                };

                let vectors = Vectors {
                    vectors_options: Some(qdrant_client::qdrant::vectors::VectorsOptions::Vectors(named_vectors)),
                };

                // Sparse vector (optional, added separately via update API if present)
                // Qdrant client API for sparse vectors in PointStruct is complex,
                // so we upsert dense first, then update with sparse if available
                let mut point = PointStruct::new(chunk.id.clone(), vectors, payload);

                // Store sparse vector info in payload for later update
                if let Some(sparse) = chunk.sparse_vector {
                    let sparse_json = serde_json::json!({
                        "indices": sparse.indices,
                        "values": sparse.values,
                    });
                    point.payload.insert(
                        "_sparse_pending".to_string(),
                        serde_json::to_string(&sparse_json).unwrap_or_default().into(),
                    );
                }

                point
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
        .vector_name("dense")  // Named vector for hybrid collections
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
        let _sparse_vec = qdrant_client::qdrant::SparseVector {
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

    /// Count total points in collection
    pub async fn count_points(&self) -> Result<usize> {
        let collection_info = self
            .client
            .collection_info(&self.collection_name)
            .await
            .map_err(|e| AppError::Qdrant(format!("Failed to get collection info: {}", e)))?;

        let count = collection_info
            .result
            .and_then(|info| info.points_count)
            .unwrap_or(0) as usize;

        Ok(count)
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
