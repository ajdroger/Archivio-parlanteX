/// Document model and ingestion types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Document {
    /// Document identifier (UUID format)
    pub doc_id: String,

    /// Parent knowledge base identifier
    pub kb_id: String,

    /// Original filename or source reference
    pub source_name: String,

    /// MIME type (e.g., application/pdf, text/plain)
    pub mime_type: String,

    /// File path in shared volume
    pub file_path: String,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// Additional metadata (JSON)
    pub metadata: serde_json::Value,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

impl Document {
    /// Create new document
    pub fn new(
        doc_id: String,
        kb_id: String,
        source_name: String,
        mime_type: String,
        file_path: String,
        tags: Vec<String>,
        metadata: serde_json::Value,
    ) -> Self {
        Self {
            doc_id,
            kb_id,
            source_name,
            mime_type,
            file_path,
            tags,
            metadata,
            created_at: Utc::now(),
        }
    }
}

/// Ingestion request
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct IngestRequest {
    /// Document identifier
    pub doc_id: String,

    /// Knowledge base identifier
    pub kb_id: String,

    /// File path in shared volume
    pub file_path: String,

    /// Original filename
    pub source_name: String,

    /// MIME type
    pub mime_type: String,

    /// Optional tags
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Ingestion response
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct IngestResponse {
    /// Document identifier
    pub doc_id: String,

    /// Number of chunks indexed in Qdrant
    pub chunks_indexed: usize,

    /// Processing time in milliseconds
    pub processing_ms: u64,

    /// Number of knowledge graph entities extracted
    pub entities_extracted: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_document_creation() {
        let doc = Document::new(
            "doc_123".to_string(),
            "kb_456".to_string(),
            "contract.pdf".to_string(),
            "application/pdf".to_string(),
            "/shared/uploads/contract.pdf".to_string(),
            vec!["legal".to_string(), "2024".to_string()],
            json!({"author": "Company XYZ"}),
        );

        assert_eq!(doc.doc_id, "doc_123");
        assert_eq!(doc.kb_id, "kb_456");
        assert_eq!(doc.tags.len(), 2);
        assert!(doc.created_at <= Utc::now());
    }

    #[test]
    fn test_ingest_request_serialization() {
        let req = IngestRequest {
            doc_id: "doc_789".to_string(),
            kb_id: "kb_abc".to_string(),
            file_path: "/shared/uploads/test.pdf".to_string(),
            source_name: "test.pdf".to_string(),
            mime_type: "application/pdf".to_string(),
            tags: vec!["urgent".to_string()],
        };

        let json = serde_json::to_string(&req).expect("Serialization failed");
        assert!(json.contains("\"doc_id\":\"doc_789\""));
        assert!(json.contains("\"kb_id\":\"kb_abc\""));
    }

    #[test]
    fn test_ingest_response() {
        let resp = IngestResponse {
            doc_id: "doc_999".to_string(),
            chunks_indexed: 42,
            processing_ms: 1500,
            entities_extracted: 15,
        };

        assert_eq!(resp.chunks_indexed, 42);
        assert_eq!(resp.processing_ms, 1500);
        assert_eq!(resp.entities_extracted, 15);
    }
}
