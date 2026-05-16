/// End-to-end test for query route with hybrid search + reranker
///
/// Requires full stack running:
/// - Ollama (embeddings)
/// - Qdrant (vector storage)
/// - Python worker (parsing + reranker)
///
/// Run with: cargo test --test query_e2e -- --ignored --nocapture

use archivio_parlante_rust_engine::models::document::{IngestRequest, IngestResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::fs;

const SAMPLE_CONTRACT: &str = r#"
CONTRATTO DI FORNITURA SERVIZI

Art. 1 - Oggetto del contratto
La presente scrittura ha per oggetto la fornitura di servizi di consulenza informatica
da parte del Fornitore alla Società Cliente per la realizzazione di un sistema software.

Art. 2 - Durata
Il contratto avrà durata di 12 mesi a decorrere dalla data di sottoscrizione.

Art. 3 - Corrispettivo
Il corrispettivo pattuito ammonta a Euro 50.000,00 (cinquantamila/00) oltre IVA di legge.
Tale importo sarà corrisposto in rate trimestrali di pari importo.

Art. 4 - Modalità di pagamento
Il pagamento avverrà in rate trimestrali posticipate entro 30 giorni dalla emissione
di regolare fattura da parte del Fornitore.

Art. 5 - Obblighi del Fornitore
Il Fornitore si impegna a svolgere i servizi con la massima diligenza professionale
e nel rispetto dei termini concordati.
"#;

#[derive(Debug, Deserialize)]
struct QueryResponse {
    results: Vec<SearchResult>,
    processing_ms: u64,
    candidates_count: usize,
}

#[derive(Debug, Deserialize)]
struct SearchResult {
    chunk_id: String,
    doc_id: String,
    chunk_index: usize,
    text: String,
    score: f32,
}

/// Test successful query after ingestion
#[tokio::test]
#[ignore] // Requires stack up: make up
async fn test_query_after_ingestion_e2e() {
    let rust_engine_url =
        env::var("RUST_ENGINE_URL").unwrap_or_else(|_| "http://localhost:8090".to_string());

    let client = reqwest::Client::new();

    // Step 1: Ingest sample document
    let temp_file = "/tmp/test_contract_query_e2e.txt";
    fs::write(temp_file, SAMPLE_CONTRACT).expect("Failed to write test file");

    let doc_id = format!("doc_query_e2e_{}", chrono::Utc::now().timestamp());
    let kb_id = "kb_query_test_e2e".to_string();

    let ingest_request = IngestRequest {
        doc_id: doc_id.clone(),
        kb_id: kb_id.clone(),
        file_path: temp_file.to_string(),
        source_name: "test_contract.txt".to_string(),
        mime_type: "text/plain".to_string(),
        tags: vec!["test".to_string(), "query_e2e".to_string()],
    };

    let ingest_response = client
        .post(format!("{}/ingest", rust_engine_url))
        .json(&ingest_request)
        .send()
        .await
        .expect("Ingestion request failed");

    assert!(
        ingest_response.status().is_success(),
        "Ingestion failed: {}",
        ingest_response.status()
    );

    let ingest_result: IngestResponse = ingest_response
        .json()
        .await
        .expect("Failed to parse ingestion response");

    println!("✅ Ingestion completed:");
    println!("  Chunks indexed: {}", ingest_result.chunks_indexed);
    println!("  Processing time: {} ms", ingest_result.processing_ms);

    // Step 2: Query the ingested document
    let query_payload = json!({
        "query": "Qual è il corrispettivo del contratto?",
        "kb_id": kb_id,
        "top_k": 3
    });

    let query_response = client
        .post(format!("{}/query", rust_engine_url))
        .json(&query_payload)
        .send()
        .await
        .expect("Query request failed");

    assert!(
        query_response.status().is_success(),
        "Query failed: {}",
        query_response.status()
    );

    let query_result: QueryResponse = query_response
        .json()
        .await
        .expect("Failed to parse query response");

    println!("✅ Query completed:");
    println!("  Results count: {}", query_result.results.len());
    println!("  Candidates before rerank: {}", query_result.candidates_count);
    println!("  Processing time: {} ms", query_result.processing_ms);

    // Step 3: Verify results
    assert!(
        !query_result.results.is_empty(),
        "Should have at least one result"
    );
    assert!(
        query_result.results.len() <= 3,
        "Should not exceed top_k=3"
    );

    // Verify first result structure
    let first_result = &query_result.results[0];
    assert_eq!(first_result.doc_id, doc_id, "doc_id should match");
    assert!(!first_result.chunk_id.is_empty(), "chunk_id should exist");
    assert!(!first_result.text.is_empty(), "text should exist");
    assert!(first_result.score > 0.0, "score should be positive");

    // Verify scores are descending
    if query_result.results.len() > 1 {
        for i in 0..query_result.results.len() - 1 {
            assert!(
                query_result.results[i].score >= query_result.results[i + 1].score,
                "Scores should be in descending order"
            );
        }
    }

    // Verify relevance
    let first_text = first_result.text.to_lowercase();
    assert!(
        first_text.contains("corrispettivo")
            || first_text.contains("euro")
            || first_text.contains("50.000"),
        "First result should be relevant to query about 'corrispettivo'. Got: {}",
        first_text
    );

    println!("  Top result: {:?}", &first_result.text[..100.min(first_result.text.len())]);
    println!("  Top score: {:.4}", first_result.score);

    // Cleanup
    fs::remove_file(temp_file).ok();
    println!("✅ Test cleanup completed");
}

/// Test query validation errors
#[tokio::test]
#[ignore]
async fn test_query_validation_errors() {
    let rust_engine_url =
        env::var("RUST_ENGINE_URL").unwrap_or_else(|_| "http://localhost:8090".to_string());

    let client = reqwest::Client::new();

    // Test 1: Empty query
    let payload = json!({
        "query": "   ",
        "kb_id": "kb_test",
        "top_k": 5
    });

    let response = client
        .post(format!("{}/query", rust_engine_url))
        .json(&payload)
        .send()
        .await
        .expect("Request failed");

    assert_eq!(
        response.status(),
        400,
        "Should return 400 for empty query"
    );

    // Test 2: Empty kb_id
    let payload = json!({
        "query": "test query",
        "kb_id": "",
        "top_k": 5
    });

    let response = client
        .post(format!("{}/query", rust_engine_url))
        .json(&payload)
        .send()
        .await
        .expect("Request failed");

    assert_eq!(response.status(), 400, "Should return 400 for empty kb_id");

    // Test 3: Zero top_k
    let payload = json!({
        "query": "test query",
        "kb_id": "kb_test",
        "top_k": 0
    });

    let response = client
        .post(format!("{}/query", rust_engine_url))
        .json(&payload)
        .send()
        .await
        .expect("Request failed");

    assert_eq!(response.status(), 400, "Should return 400 for top_k = 0");

    // Test 4: Excessive top_k
    let payload = json!({
        "query": "test query",
        "kb_id": "kb_test",
        "top_k": 100
    });

    let response = client
        .post(format!("{}/query", rust_engine_url))
        .json(&payload)
        .send()
        .await
        .expect("Request failed");

    assert_eq!(response.status(), 400, "Should return 400 for top_k > 50");

    println!("✅ Validation tests passed");
}

/// Test query with non-existent KB (should return empty results gracefully)
#[tokio::test]
#[ignore]
async fn test_query_nonexistent_kb() {
    let rust_engine_url =
        env::var("RUST_ENGINE_URL").unwrap_or_else(|_| "http://localhost:8090".to_string());

    let client = reqwest::Client::new();

    let payload = json!({
        "query": "test query",
        "kb_id": "kb_does_not_exist_12345",
        "top_k": 5
    });

    let response = client
        .post(format!("{}/query", rust_engine_url))
        .json(&payload)
        .send()
        .await
        .expect("Request failed");

    // Should succeed with empty results, not error
    assert_eq!(
        response.status(),
        200,
        "Should return 200 for non-existent KB"
    );

    let result: QueryResponse = response.json().await.expect("Failed to parse response");

    assert!(
        result.results.is_empty(),
        "Should return empty results for non-existent KB"
    );

    println!("✅ Non-existent KB test passed");
}

/// Test query with default top_k
#[tokio::test]
#[ignore]
async fn test_query_default_top_k() {
    let rust_engine_url =
        env::var("RUST_ENGINE_URL").unwrap_or_else(|_| "http://localhost:8090".to_string());

    let client = reqwest::Client::new();

    let payload = json!({
        "query": "test query",
        "kb_id": "kb_test_default"
        // top_k omitted, should default to 5
    });

    let response = client
        .post(format!("{}/query", rust_engine_url))
        .json(&payload)
        .send()
        .await
        .expect("Request failed");

    // Should succeed with default
    assert_eq!(
        response.status(),
        200,
        "Should succeed with default top_k"
    );

    println!("✅ Default top_k test passed");
}
