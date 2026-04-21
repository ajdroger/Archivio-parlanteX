/// Chunk model for document segmentation

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Document chunk with semantic boundaries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    /// Unique chunk identifier
    pub id: Uuid,

    /// Parent document ID
    pub doc_id: String,

    /// Chunk index in document (0-based)
    pub chunk_idx: usize,

    /// Original chunk text
    pub text: String,

    /// Contextual retrieval enriched text (prefix + original)
    pub contextual_text: Option<String>,

    /// Token count (cl100k_base encoding)
    pub token_count: usize,

    /// Character offset start in original document
    pub start_offset: usize,

    /// Character offset end in original document
    pub end_offset: usize,

    /// Additional metadata (section headers, clause markers, etc.)
    pub metadata: serde_json::Value,
}

impl Chunk {
    /// Create new chunk with auto-generated UUID
    ///
    /// # Arguments
    /// * `doc_id` - Parent document identifier
    /// * `chunk_idx` - Index in document
    /// * `text` - Chunk text content
    /// * `token_count` - Token count
    /// * `start_offset` - Start position in original text
    /// * `end_offset` - End position in original text
    /// * `metadata` - Additional metadata (JSON)
    pub fn new(
        doc_id: String,
        chunk_idx: usize,
        text: String,
        token_count: usize,
        start_offset: usize,
        end_offset: usize,
        metadata: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            doc_id,
            chunk_idx,
            text,
            contextual_text: None,
            token_count,
            start_offset,
            end_offset,
            metadata,
        }
    }

    /// Get effective text for embedding (contextual if available, else original)
    pub fn embedding_text(&self) -> &str {
        self.contextual_text.as_deref().unwrap_or(&self.text)
    }

    /// Check if chunk has contextual enrichment
    pub fn is_contextualized(&self) -> bool {
        self.contextual_text.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_chunk_creation() {
        let chunk = Chunk::new(
            "doc_123".to_string(),
            0,
            "Sample text".to_string(),
            3,
            0,
            11,
            json!({"section": "Introduction"}),
        );

        assert_eq!(chunk.doc_id, "doc_123");
        assert_eq!(chunk.chunk_idx, 0);
        assert_eq!(chunk.text, "Sample text");
        assert_eq!(chunk.token_count, 3);
        assert_eq!(chunk.start_offset, 0);
        assert_eq!(chunk.end_offset, 11);
        assert!(!chunk.is_contextualized());
    }

    #[test]
    fn test_embedding_text() {
        let mut chunk = Chunk::new(
            "doc_123".to_string(),
            0,
            "Original text".to_string(),
            2,
            0,
            13,
            json!({}),
        );

        // Without contextual text
        assert_eq!(chunk.embedding_text(), "Original text");

        // With contextual text
        chunk.contextual_text = Some("Context prefix.\n\nOriginal text".to_string());
        assert_eq!(chunk.embedding_text(), "Context prefix.\n\nOriginal text");
        assert!(chunk.is_contextualized());
    }

    #[test]
    fn test_chunk_serialization() {
        let chunk = Chunk::new(
            "doc_456".to_string(),
            5,
            "Test chunk".to_string(),
            2,
            100,
            110,
            json!({"is_clause_start": true}),
        );

        let json = serde_json::to_string(&chunk).expect("Serialization failed");
        assert!(json.contains("\"doc_id\":\"doc_456\""));
        assert!(json.contains("\"chunk_idx\":5"));

        let deserialized: Chunk = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(deserialized.doc_id, chunk.doc_id);
        assert_eq!(deserialized.chunk_idx, chunk.chunk_idx);
    }
}
