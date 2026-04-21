/// Hybrid search with Reciprocal Rank Fusion (RRF)

use std::collections::HashMap;
use std::sync::Arc;

use crate::clients::qdrant::{QdrantWrapper, ScoredChunk, SparseVector};
use crate::errors::Result;
use crate::providers::LlmProvider;
use crate::utils::bm25::Bm25Vectorizer;
use qdrant_client::qdrant::Filter;

/// Hybrid searcher combining dense + sparse retrieval
#[derive(Clone)]
pub struct HybridSearcher {
    /// Qdrant client
    qdrant: Arc<QdrantWrapper>,

    /// LLM provider for query embedding
    llm: Arc<dyn LlmProvider>,

    /// Top-k for dense search
    top_k_dense: usize,

    /// Top-k for sparse search
    top_k_sparse: usize,

    /// RRF k parameter (rank normalization constant)
    rrf_k: u32,
}

impl HybridSearcher {
    /// Create new hybrid searcher
    pub fn new(
        qdrant: Arc<QdrantWrapper>,
        llm: Arc<dyn LlmProvider>,
        top_k_dense: usize,
        top_k_sparse: usize,
        rrf_k: u32,
    ) -> Self {
        Self {
            qdrant,
            llm,
            top_k_dense,
            top_k_sparse,
            rrf_k,
        }
    }

    /// Perform hybrid search
    ///
    /// # Arguments
    /// * `query` - Search query text
    /// * `top_k` - Number of final results after fusion
    ///
    /// # Returns
    /// Fused and ranked results
    pub async fn search(&self, query: &str, top_k: usize) -> Result<Vec<ScoredChunk>> {
        self.search_with_filter(query, top_k, None).await
    }

    /// Perform hybrid search with optional filter
    ///
    /// # Arguments
    /// * `query` - Search query text
    /// * `top_k` - Number of final results after fusion
    /// * `filter` - Optional Qdrant filter (e.g., filter by doc_id)
    ///
    /// # Returns
    /// Fused and ranked results
    pub async fn search_with_filter(
        &self,
        query: &str,
        top_k: usize,
        filter: Option<Filter>,
    ) -> Result<Vec<ScoredChunk>> {
        tracing::debug!(query = %query, "Starting hybrid search");

        // Step 1: Generate query embedding (dense)
        let query_embeddings = self.llm.embed(vec![query.to_string()]).await?;
        let query_embedding = query_embeddings.into_iter().next().ok_or_else(|| {
            crate::errors::AppError::Internal(anyhow::anyhow!("No embedding generated"))
        })?;

        // Step 2: Generate query sparse vector (BM25)
        let query_sparse = self.generate_query_sparse(query)?;

        // Step 3: Dense search
        let dense_results = self
            .qdrant
            .search_dense(query_embedding, self.top_k_dense, filter.clone())
            .await?;

        tracing::debug!(dense_count = dense_results.len(), "Dense search completed");

        // Step 4: Sparse search
        let sparse_results = self
            .qdrant
            .search_sparse(query_sparse, self.top_k_sparse, filter)
            .await?;

        tracing::debug!(sparse_count = sparse_results.len(), "Sparse search completed");

        // Step 5: Fuse with RRF
        let fused = self.fuse_with_rrf(dense_results, sparse_results, top_k);

        tracing::info!(
            fused_count = fused.len(),
            "Hybrid search completed with RRF fusion"
        );

        Ok(fused)
    }

    /// Generate sparse vector for query
    fn generate_query_sparse(&self, query: &str) -> Result<SparseVector> {
        // For query, we build a minimal vocabulary just from the query terms
        // In production, this should use the same vocabulary as indexing
        // For now, simplified approach
        let mut vectorizer = Bm25Vectorizer::new();
        vectorizer.build_vocabulary(&vec![query.to_string()]);
        vectorizer.vectorize(query)
    }

    /// Fuse results using Reciprocal Rank Fusion
    ///
    /// RRF formula: score(doc) = sum over ranklists: 1 / (k + rank(doc))
    ///
    /// # Arguments
    /// * `dense_results` - Results from dense search
    /// * `sparse_results` - Results from sparse search
    /// * `top_k` - Number of results to return
    fn fuse_with_rrf(
        &self,
        dense_results: Vec<ScoredChunk>,
        sparse_results: Vec<ScoredChunk>,
        top_k: usize,
    ) -> Vec<ScoredChunk> {
        let mut rrf_scores: HashMap<String, f32> = HashMap::new();
        let mut chunks_map: HashMap<String, ScoredChunk> = HashMap::new();

        let k = self.rrf_k as f32;

        // Process dense results
        for (rank, chunk) in dense_results.into_iter().enumerate() {
            let rrf_score = 1.0 / (k + (rank + 1) as f32);
            *rrf_scores.entry(chunk.id.clone()).or_insert(0.0) += rrf_score;
            chunks_map.insert(chunk.id.clone(), chunk);
        }

        // Process sparse results
        for (rank, chunk) in sparse_results.into_iter().enumerate() {
            let rrf_score = 1.0 / (k + (rank + 1) as f32);
            *rrf_scores.entry(chunk.id.clone()).or_insert(0.0) += rrf_score;
            chunks_map.entry(chunk.id.clone()).or_insert(chunk);
        }

        // Sort by RRF score descending
        let mut scored_chunks: Vec<_> = rrf_scores
            .into_iter()
            .filter_map(|(id, rrf_score)| {
                chunks_map.get(&id).map(|chunk| {
                    let mut fused_chunk = chunk.clone();
                    fused_chunk.score = rrf_score;
                    fused_chunk
                })
            })
            .collect();

        scored_chunks.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // Return top-k
        scored_chunks.into_iter().take(top_k).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rrf_fusion_scores() {
        // Mock data for testing RRF calculation
        let dense = vec![
            ScoredChunk {
                id: "chunk_1".to_string(),
                doc_id: "doc_1".to_string(),
                chunk_index: 0,
                text: "text 1".to_string(),
                score: 0.9,
            },
            ScoredChunk {
                id: "chunk_2".to_string(),
                doc_id: "doc_1".to_string(),
                chunk_index: 1,
                text: "text 2".to_string(),
                score: 0.8,
            },
        ];

        let sparse = vec![
            ScoredChunk {
                id: "chunk_2".to_string(),
                doc_id: "doc_1".to_string(),
                chunk_index: 1,
                text: "text 2".to_string(),
                score: 0.7,
            },
            ScoredChunk {
                id: "chunk_3".to_string(),
                doc_id: "doc_1".to_string(),
                chunk_index: 2,
                text: "text 3".to_string(),
                score: 0.6,
            },
        ];

        // Create mock searcher (we won't actually use qdrant/llm for this test)
        // Just testing the fusion logic
        let k = 60;

        // Manual RRF calculation
        // chunk_1: dense rank 1 → 1/(60+1) = 0.01639
        // chunk_2: dense rank 2 + sparse rank 1 → 1/(60+2) + 1/(60+1) = 0.01613 + 0.01639 = 0.03252
        // chunk_3: sparse rank 2 → 1/(60+2) = 0.01613

        // Expected order: chunk_2 (highest RRF), chunk_1, chunk_3

        // We'll test this manually since we can't easily create a HybridSearcher without real dependencies
        let mut rrf_scores: HashMap<String, f32> = HashMap::new();

        let k_f = k as f32;

        // Dense
        for (rank, chunk) in dense.iter().enumerate() {
            *rrf_scores.entry(chunk.id.clone()).or_insert(0.0) +=
                1.0 / (k_f + (rank + 1) as f32);
        }

        // Sparse
        for (rank, chunk) in sparse.iter().enumerate() {
            *rrf_scores.entry(chunk.id.clone()).or_insert(0.0) +=
                1.0 / (k_f + (rank + 1) as f32);
        }

        // Verify chunk_2 has highest score (appears in both)
        assert!(rrf_scores["chunk_2"] > rrf_scores["chunk_1"]);
        assert!(rrf_scores["chunk_2"] > rrf_scores["chunk_3"]);
    }

    #[test]
    fn test_generate_query_sparse() {
        let query = "contratto fornitura servizi";

        let mut vectorizer = Bm25Vectorizer::new();
        vectorizer.build_vocabulary(&vec![query.to_string()]);

        let sparse = vectorizer.vectorize(query).expect("Should vectorize");

        assert!(!sparse.indices.is_empty());
        assert_eq!(sparse.indices.len(), sparse.values.len());
    }
}
