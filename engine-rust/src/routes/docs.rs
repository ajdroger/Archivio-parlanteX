/// OpenAPI documentation with Swagger UI

use utoipa::OpenApi;

use crate::models::comparison::{CompareRequest, CompareResponse};
use crate::models::document::{IngestRequest, IngestResponse};
use crate::routes::query::{QueryRequest, QueryResponse, SearchResult};

/// OpenAPI specification
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Archivio Parlante REST API",
        version = "0.1.0",
        description = "RAG enterprise engine for Italian legal contract forensic analysis with zero hallucinations",
        contact(
            name = "Archivio Parlante",
            email = "support@example.com"
        )
    ),
    paths(
        // OpenAPI paths are auto-generated from route handlers
        // See: routes/ingest.rs, routes/query.rs, routes/chat.rs, routes/compare.rs, routes/kb.rs
        // Each handler is annotated with #[utoipa::path(...)]
    ),
    components(
        schemas(
            IngestRequest,
            IngestResponse,
            QueryRequest,
            QueryResponse,
            SearchResult,
            CompareRequest,
            CompareResponse,
        )
    ),
    tags(
        (name = "ingestion", description = "Document ingestion endpoints"),
        (name = "query", description = "Query and search endpoints"),
        (name = "comparison", description = "Multi-contract comparison endpoints"),
        (name = "kb_management", description = "Knowledge base management endpoints"),
        (name = "health", description = "Health and metrics endpoints"),
    )
)]
pub struct ApiDoc;

/// Get OpenAPI JSON spec
pub fn get_openapi_spec() -> String {
    ApiDoc::openapi().to_json().unwrap_or_else(|e| {
        tracing::error!(error = %e, "Failed to serialize OpenAPI spec");
        r#"{"error": "Failed to generate OpenAPI spec"}"#.to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openapi_spec_generation() {
        let spec = get_openapi_spec();

        assert!(spec.contains("Archivio Parlante"));
        assert!(spec.contains("0.1.0"));
        assert!(!spec.contains("error"));
    }

    #[test]
    fn test_openapi_has_components() {
        let spec = ApiDoc::openapi();

        // Verify schemas are registered
        let spec_json = serde_json::to_value(&spec).unwrap();

        assert!(spec_json["components"]["schemas"]["IngestRequest"].is_object());
        assert!(spec_json["components"]["schemas"]["QueryRequest"].is_object());
        assert!(spec_json["components"]["schemas"]["CompareRequest"].is_object());
    }

    #[test]
    fn test_openapi_has_tags() {
        let spec = ApiDoc::openapi();
        let spec_json = serde_json::to_value(&spec).unwrap();

        let tags = spec_json["tags"].as_array().unwrap();

        let tag_names: Vec<String> = tags
            .iter()
            .filter_map(|t| t["name"].as_str())
            .map(|s| s.to_string())
            .collect();

        assert!(tag_names.contains(&"ingestion".to_string()));
        assert!(tag_names.contains(&"query".to_string()));
        assert!(tag_names.contains(&"comparison".to_string()));
    }
}
