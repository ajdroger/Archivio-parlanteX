/// BM25 sparse vector generation using tantivy

use std::collections::HashMap;
use tantivy::tokenizer::{SimpleTokenizer, TextAnalyzer, TokenStream};

use crate::clients::qdrant::SparseVector;
use crate::errors::Result;

/// BM25 sparse vectorizer
///
/// Generates sparse vectors (term indices + weights) for BM25 retrieval
pub struct Bm25Vectorizer {
    /// Term to index mapping
    vocabulary: HashMap<String, u32>,

    /// Next term index
    next_index: u32,
}

impl Bm25Vectorizer {
    /// Create new BM25 vectorizer
    pub fn new() -> Self {
        Self {
            vocabulary: HashMap::new(),
            next_index: 0,
        }
    }

    /// Build vocabulary from corpus
    ///
    /// Processes all texts to build term→index mapping
    pub fn build_vocabulary(&mut self, texts: &[String]) {
        let tokenizer = TextAnalyzer::from(SimpleTokenizer::default());

        for text in texts {
            let mut token_stream = tokenizer.token_stream(text);
            while let Some(token) = token_stream.next() {
                let term = token.text.to_lowercase();

                if !self.vocabulary.contains_key(&term) {
                    self.vocabulary.insert(term, self.next_index);
                    self.next_index += 1;
                }
            }
        }

        tracing::debug!(
            vocabulary_size = self.vocabulary.len(),
            "BM25 vocabulary built"
        );
    }

    /// Generate sparse vector for text
    ///
    /// Returns term indices and BM25 weights
    pub fn vectorize(&self, text: &str) -> Result<SparseVector> {
        let tokenizer = TextAnalyzer::from(SimpleTokenizer::default());

        // Count term frequencies
        let mut term_freq: HashMap<String, f32> = HashMap::new();
        let mut token_stream = tokenizer.token_stream(text);
        let mut total_tokens = 0;

        while let Some(token) = token_stream.next() {
            let term = token.text.to_lowercase();
            *term_freq.entry(term).or_insert(0.0) += 1.0;
            total_tokens += 1;
        }

        if total_tokens == 0 {
            return Ok(SparseVector {
                indices: vec![],
                values: vec![],
            });
        }

        // Convert to BM25 weights (simplified, no IDF for now)
        let mut indices = Vec::new();
        let mut values = Vec::new();

        let k1 = 1.2; // BM25 k1 parameter
        let b = 0.75; // BM25 b parameter
        let avg_doc_len = 100.0; // Approximate average document length
        let doc_len = total_tokens as f32;

        for (term, freq) in term_freq {
            if let Some(&index) = self.vocabulary.get(&term) {
                // BM25 term weight (without IDF component)
                let weight = (freq * (k1 + 1.0))
                    / (freq + k1 * (1.0 - b + b * (doc_len / avg_doc_len)));

                indices.push(index);
                values.push(weight);
            }
        }

        // Sort by index for consistency
        let mut pairs: Vec<_> = indices.into_iter().zip(values).collect();
        pairs.sort_by_key(|(idx, _)| *idx);

        let (indices, values): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();

        Ok(SparseVector { indices, values })
    }

    /// Get vocabulary size
    pub fn vocab_size(&self) -> usize {
        self.vocabulary.len()
    }
}

impl Default for Bm25Vectorizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bm25_vectorizer_creation() {
        let vectorizer = Bm25Vectorizer::new();
        assert_eq!(vectorizer.vocab_size(), 0);
    }

    #[test]
    fn test_build_vocabulary() {
        let mut vectorizer = Bm25Vectorizer::new();

        let texts = vec![
            "contratto di fornitura servizi".to_string(),
            "servizi di consulenza".to_string(),
        ];

        vectorizer.build_vocabulary(&texts);

        assert!(vectorizer.vocab_size() > 0);
        assert!(vectorizer.vocabulary.contains_key("contratto"));
        assert!(vectorizer.vocabulary.contains_key("servizi"));
        assert!(vectorizer.vocabulary.contains_key("consulenza"));
    }

    #[test]
    fn test_vectorize_text() {
        let mut vectorizer = Bm25Vectorizer::new();

        let corpus = vec![
            "articolo uno della legge".to_string(),
            "articolo due della normativa".to_string(),
        ];

        vectorizer.build_vocabulary(&corpus);

        let sparse = vectorizer
            .vectorize("articolo della legge")
            .expect("Vectorization should succeed");

        assert!(!sparse.indices.is_empty());
        assert_eq!(sparse.indices.len(), sparse.values.len());

        // Verify indices are sorted
        for i in 1..sparse.indices.len() {
            assert!(sparse.indices[i] > sparse.indices[i - 1]);
        }

        // Verify weights are positive
        for weight in &sparse.values {
            assert!(*weight > 0.0);
        }
    }

    #[test]
    fn test_vectorize_empty_text() {
        let vectorizer = Bm25Vectorizer::new();

        let sparse = vectorizer
            .vectorize("")
            .expect("Empty text should work");

        assert!(sparse.indices.is_empty());
        assert!(sparse.values.is_empty());
    }

    #[test]
    fn test_vectorize_unknown_terms() {
        let mut vectorizer = Bm25Vectorizer::new();

        vectorizer.build_vocabulary(&vec!["known term".to_string()]);

        let sparse = vectorizer
            .vectorize("unknown word")
            .expect("Unknown terms should be skipped");

        // Unknown terms not in vocabulary → empty vector
        assert!(sparse.indices.is_empty());
    }
}
