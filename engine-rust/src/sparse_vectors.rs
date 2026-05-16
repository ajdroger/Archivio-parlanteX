/// BM25 sparse vector generation using tantivy tokenizer
///
/// Converts text into sparse vector representation for hybrid search.

use std::collections::HashMap;
use tantivy::tokenizer::{SimpleTokenizer, TextAnalyzer, TokenStream};

use crate::clients::qdrant::SparseVector;

/// BM25 sparse vector generator
pub struct BM25VectorGenerator {
    tokenizer: TextAnalyzer,
    vocab_size: u32,
}

impl BM25VectorGenerator {
    /// Create new BM25 generator
    pub fn new() -> Self {
        // Use simple tokenizer (splits on whitespace/punctuation, lowercases)
        let tokenizer = TextAnalyzer::builder(SimpleTokenizer::default())
            .filter_dynamic(tantivy::tokenizer::LowerCaser)
            .build();

        Self {
            tokenizer,
            vocab_size: 100_000, // Reasonable vocab size for Italian legal text
        }
    }

    /// Generate sparse vector from text
    ///
    /// Uses simple term frequency (TF) weighting.
    /// For production BM25, would need IDF from corpus statistics.
    pub fn generate(&mut self, text: &str) -> SparseVector {
        let mut term_freq: HashMap<String, usize> = HashMap::new();

        // Count term frequencies (in a scope to release borrow)
        {
            let mut token_stream = self.tokenizer.token_stream(text);
            while let Some(token) = token_stream.next() {
                let term = token.text.to_string();
                *term_freq.entry(term).or_insert(0) += 1;
            }
        }

        // Convert to sparse vector (term hash → weight)
        let mut indices = Vec::new();
        let mut values = Vec::new();

        for (term, freq) in term_freq.iter() {
            let index = self.hash_term(term);
            let weight = (*freq as f32).sqrt(); // Simple TF weighting (sqrt for smoothing)

            indices.push(index);
            values.push(weight);
        }

        // Sort by indices (Qdrant requirement)
        let mut pairs: Vec<_> = indices.into_iter().zip(values.into_iter()).collect();
        pairs.sort_by_key(|(idx, _)| *idx);

        let (sorted_indices, sorted_values): (Vec<u32>, Vec<f32>) = pairs.into_iter().unzip();

        SparseVector {
            indices: sorted_indices,
            values: sorted_values,
        }
    }

    /// Hash term to vocabulary index (deterministic)
    fn hash_term(&self, term: &str) -> u32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        term.hash(&mut hasher);
        (hasher.finish() % self.vocab_size as u64) as u32
    }
}

impl Default for BM25VectorGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bm25_generation() {
        let mut generator = BM25VectorGenerator::new();
        let text = "contratto di fornitura servizi professionali clausola 7.2 penale";

        let sparse = generator.generate(text);

        assert!(!sparse.indices.is_empty());
        assert_eq!(sparse.indices.len(), sparse.values.len());

        // Check sorted order
        for i in 1..sparse.indices.len() {
            assert!(sparse.indices[i] >= sparse.indices[i - 1]);
        }

        // Check non-zero weights
        assert!(sparse.values.iter().all(|&v| v > 0.0));
    }

    #[test]
    fn test_empty_text() {
        let mut generator = BM25VectorGenerator::new();
        let sparse = generator.generate("");

        assert!(sparse.indices.is_empty());
        assert!(sparse.values.is_empty());
    }

    #[test]
    fn test_repeated_terms() {
        let mut generator = BM25VectorGenerator::new();
        let text = "contratto contratto contratto"; // Repeated term

        let sparse = generator.generate(text);

        // Should have single entry with higher weight
        assert_eq!(sparse.indices.len(), 1);
        assert!(sparse.values[0] > 1.0); // sqrt(3) ≈ 1.73
    }

    #[test]
    fn test_deterministic_hashing() {
        let mut generator = BM25VectorGenerator::new();
        let text = "test document";

        let sparse1 = generator.generate(text);
        let sparse2 = generator.generate(text);

        assert_eq!(sparse1.indices, sparse2.indices);
        assert_eq!(sparse1.values, sparse2.values);
    }
}
