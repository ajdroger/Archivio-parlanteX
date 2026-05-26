/// End-to-end ingestion test
///
/// Requires full stack running:
/// - Ollama (embeddings + contextual enrichment)
/// - Qdrant (vector storage)
/// - Python worker (document parsing)
///
/// Run with: cargo test --test ingestion_e2e -- --ignored --nocapture

use archivio_parlante_rust_engine::models::document::{IngestRequest, IngestResponse};
use archivio_parlante_rust_engine::clients::qdrant::QdrantWrapper;
use std::fs;
use std::env;

const SAMPLE_TEXT_DOCUMENT: &str = r#"
# CONTRATTO DI CONSULENZA

Art. 1 - Oggetto
Il presente contratto ha per oggetto la fornitura di servizi di consulenza strategica
per il periodo di 12 mesi.

Art. 2 - Corrispettivo
Il corrispettivo è fissato in Euro 50.000,00 oltre IVA.

Art. 3 - Obblighi delle parti
Il Consulente si impegna a fornire servizi con la massima diligenza professionale.
Il Cliente corrisponderà i compensi pattuiti entro 30 giorni.
"#;

#[tokio::test]
#[ignore] // Requires stack up: make up
async fn test_ingestion_e2e_text_document() {
    // Setup: write sample document to shared volume
    let temp_file = "/tmp/test_contract_e2e.txt";
    fs::write(temp_file, SAMPLE_TEXT_DOCUMENT).expect("Failed to write test file");

    let doc_id = format!("doc_e2e_test_{}", chrono::Utc::now().timestamp());
    let kb_id = "kb_test_e2e".to_string();

    // Prepare request
    let request = IngestRequest {
        doc_id: doc_id.clone(),
        kb_id: kb_id.clone(),
        file_path: temp_file.to_string(),
        source_name: "test_contract.txt".to_string(),
        mime_type: "text/plain".to_string(),
        tags: vec!["test".to_string(), "e2e".to_string()],
    };

    // Call ingestion endpoint
    let rust_engine_url = env::var("RUST_ENGINE_URL")
        .unwrap_or_else(|_| "http://localhost:8090".to_string());

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/ingest", rust_engine_url))
        .json(&request)
        .send()
        .await
        .expect("Ingestion request failed");

    assert!(
        response.status().is_success(),
        "Ingestion failed with status: {}",
        response.status()
    );

    let ingest_response: IngestResponse = response
        .json()
        .await
        .expect("Failed to parse response");

    println!("✅ Ingestion completed:");
    println!("  Doc ID: {}", ingest_response.doc_id);
    println!("  Chunks indexed: {}", ingest_response.chunks_indexed);
    println!("  Processing time: {} ms", ingest_response.processing_ms);
    println!("  Entities extracted: {}", ingest_response.entities_extracted);

    // Verify response
    assert_eq!(ingest_response.doc_id, doc_id);
    assert!(
        ingest_response.chunks_indexed > 0,
        "Should have indexed at least 1 chunk"
    );
    assert!(
        ingest_response.chunks_indexed < 20,
        "Sample document should produce < 20 chunks, got {}",
        ingest_response.chunks_indexed
    );
    assert!(
        ingest_response.processing_ms > 0,
        "Processing time should be > 0"
    );

    // Verify chunks exist in Qdrant
    let qdrant_url = env::var("QDRANT_URL")
        .unwrap_or_else(|_| "http://localhost:6335".to_string());

    let collection_name = format!("ap_kb_{}", kb_id);
    let qdrant = QdrantWrapper::new(qdrant_url, collection_name, 768)
        .await
        .expect("Failed to connect to Qdrant");

    // Search for chunks from this document
    let dummy_vector = vec![0.1; 768]; // Dummy vector for test search
    let results = qdrant
        .search_dense(dummy_vector, 10, None)
        .await
        .expect("Qdrant search failed");

    println!("✅ Qdrant verification:");
    println!("  Chunks found: {}", results.len());

    // Verify at least some chunks were stored
    assert!(
        !results.is_empty(),
        "Should find chunks in Qdrant after ingestion"
    );

    // Verify chunks belong to our document
    let our_chunks: Vec<_> = results
        .iter()
        .filter(|c| c.doc_id == doc_id)
        .collect();

    assert!(
        !our_chunks.is_empty(),
        "Should find at least one chunk from our document"
    );

    println!("  Our document chunks: {}", our_chunks.len());

    // Cleanup
    fs::remove_file(temp_file).ok();
    println!("✅ Test cleanup completed");
}

#[tokio::test]
#[ignore]
async fn test_ingestion_validation_errors() {
    let rust_engine_url = env::var("RUST_ENGINE_URL")
        .unwrap_or_else(|_| "http://localhost:8090".to_string());

    let client = reqwest::Client::new();

    // Test 1: Empty doc_id
    let request = IngestRequest {
        doc_id: "".to_string(),
        kb_id: "kb_test".to_string(),
        file_path: "/tmp/test.pdf".to_string(),
        source_name: "test.pdf".to_string(),
        mime_type: "application/pdf".to_string(),
        tags: vec![],
    };

    let response = client
        .post(format!("{}/ingest", rust_engine_url))
        .json(&request)
        .send()
        .await
        .expect("Request failed");

    assert_eq!(response.status(), 400, "Should return 400 for empty doc_id");

    // Test 2: Invalid MIME type
    let request = IngestRequest {
        doc_id: "doc_test".to_string(),
        kb_id: "kb_test".to_string(),
        file_path: "/tmp/test.mp4".to_string(),
        source_name: "test.mp4".to_string(),
        mime_type: "video/mp4".to_string(),
        tags: vec![],
    };

    let response = client
        .post(format!("{}/ingest", rust_engine_url))
        .json(&request)
        .send()
        .await
        .expect("Request failed");

    assert_eq!(response.status(), 400, "Should return 400 for invalid MIME type");

    println!("✅ Validation tests passed");
}
