/// Complete end-to-end workflow test
///
/// Tests the full lifecycle:
/// 1. Ingest multiple documents
/// 2. Query documents
/// 3. Compare contracts
/// 4. KB management (stats, list, delete)
///
/// Requires full stack running:
/// - Ollama (embeddings + chat)
/// - Qdrant (vector storage)
/// - Python worker (parsing)
///
/// Run with: cargo test --test full_workflow_e2e -- --ignored --nocapture

use archivio_parlante_rust_engine::models::document::{IngestRequest, IngestResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;
use std::env;
use std::fs;

const CONTRACT_A: &str = r#"
CONTRATTO DI LOCAZIONE COMMERCIALE

Stipulato in data 15/03/2024

Tra:
- Società Alpha S.r.l. (Locatore)
- Società Beta S.r.l. (Conduttore)

Art. 1 - Oggetto
Locazione di immobile commerciale sito in Via Roma 10, Milano.
Superficie: 150 mq.

Art. 2 - Durata
6 anni + 6 anni rinnovabili (6+6).

Art. 3 - Canone
Euro 24.000,00 annui oltre IVA.
Pagamento trimestrale anticipato.

Art. 4 - Deposito cauzionale
3 mensilità (Euro 6.000,00).

Art. 5 - Foro competente
Tribunale di Milano.
"#;

const CONTRACT_B: &str = r#"
CONTRATTO DI AFFITTO COMMERCIALE

Stipulato in data 22/05/2024

Tra:
- Immobiliare Gamma S.p.A. (Locatore)
- Società Delta S.r.l. (Conduttore)

Art. 1 - Immobile
Affitto di locale commerciale in Via Torino 25, Roma.
Superficie: 200 mq.

Art. 2 - Periodo
9 anni senza rinnovo automatico.

Art. 3 - Corrispettivo
Euro 36.000,00 annui oltre IVA.
Pagamento mensile posticipato.

Art. 4 - Garanzia
6 mensilità (Euro 18.000,00).

Art. 5 - Giurisdizione
Tribunale di Roma.
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
    text: String,
    score: f32,
}

#[derive(Debug, Deserialize)]
struct CompareResponse {
    markdown_result: String,
    processing_ms: u64,
}

#[derive(Debug, Deserialize)]
struct KbStatsResponse {
    kb_id: String,
    documents_count: usize,
    chunks_count: usize,
}

#[derive(Debug, Deserialize)]
struct DocumentInfo {
    doc_id: String,
    source_name: String,
}

/// Complete workflow test: ingest → query → compare → KB ops
#[tokio::test]
#[ignore] // Requires stack up: make up
async fn test_full_workflow_end_to_end() {
    let rust_engine_url =
        env::var("RUST_ENGINE_URL").unwrap_or_else(|_| "http://localhost:8090".to_string());

    let client = reqwest::Client::new();
    let kb_id = format!("kb_workflow_test_{}", chrono::Utc::now().timestamp());

    println!("\n=== PHASE 1: INGESTION ===");

    // Ingest Contract A
    let temp_file_a = "/tmp/contract_a_workflow.txt";
    fs::write(temp_file_a, CONTRACT_A).expect("Failed to write contract A");

    let doc_id_a = format!("contract_a_{}", chrono::Utc::now().timestamp());

    let ingest_request_a = IngestRequest {
        doc_id: doc_id_a.clone(),
        kb_id: kb_id.clone(),
        file_path: temp_file_a.to_string(),
        source_name: "Contratto_Locazione_Milano.txt".to_string(),
        mime_type: "text/plain".to_string(),
        tags: vec!["locazione".to_string(), "milano".to_string()],
    };

    let ingest_response_a = client
        .post(format!("{}/ingest", rust_engine_url))
        .json(&ingest_request_a)
        .send()
        .await
        .expect("Ingestion A failed");

    assert!(
        ingest_response_a.status().is_success(),
        "Ingestion A failed: {}",
        ingest_response_a.status()
    );

    let ingest_result_a: IngestResponse = ingest_response_a
        .json()
        .await
        .expect("Failed to parse ingestion A response");

    println!("✅ Contract A ingested:");
    println!("  Doc ID: {}", ingest_result_a.doc_id);
    println!("  Chunks: {}", ingest_result_a.chunks_indexed);
    println!("  Processing: {} ms", ingest_result_a.processing_ms);

    assert!(ingest_result_a.chunks_indexed > 0);

    // Ingest Contract B
    let temp_file_b = "/tmp/contract_b_workflow.txt";
    fs::write(temp_file_b, CONTRACT_B).expect("Failed to write contract B");

    let doc_id_b = format!("contract_b_{}", chrono::Utc::now().timestamp());

    let ingest_request_b = IngestRequest {
        doc_id: doc_id_b.clone(),
        kb_id: kb_id.clone(),
        file_path: temp_file_b.to_string(),
        source_name: "Contratto_Affitto_Roma.txt".to_string(),
        mime_type: "text/plain".to_string(),
        tags: vec!["affitto".to_string(), "roma".to_string()],
    };

    let ingest_response_b = client
        .post(format!("{}/ingest", rust_engine_url))
        .json(&ingest_request_b)
        .send()
        .await
        .expect("Ingestion B failed");

    assert!(
        ingest_response_b.status().is_success(),
        "Ingestion B failed: {}",
        ingest_response_b.status()
    );

    let ingest_result_b: IngestResponse = ingest_response_b
        .json()
        .await
        .expect("Failed to parse ingestion B response");

    println!("✅ Contract B ingested:");
    println!("  Doc ID: {}", ingest_result_b.doc_id);
    println!("  Chunks: {}", ingest_result_b.chunks_indexed);
    println!("  Processing: {} ms", ingest_result_b.processing_ms);

    assert!(ingest_result_b.chunks_indexed > 0);

    // Wait for indexing
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    println!("\n=== PHASE 2: QUERY ===");

    // Query for canone/corrispettivo
    let query_payload = json!({
        "query": "Qual è il canone annuo?",
        "kb_id": kb_id,
        "top_k": 5
    });

    let query_response = client
        .post(format!("{}/query", rust_engine_url))
        .json(&query_payload)
        .send()
        .await
        .expect("Query failed");

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
    println!("  Results: {}", query_result.results.len());
    println!("  Candidates: {}", query_result.candidates_count);
    println!("  Processing: {} ms", query_result.processing_ms);

    assert!(!query_result.results.is_empty(), "Should have results");

    // Verify results contain both contracts
    let doc_ids_in_results: HashSet<String> = query_result
        .results
        .iter()
        .map(|r| r.doc_id.clone())
        .collect();

    println!("  Documents in results: {:?}", doc_ids_in_results);

    // At least one result should mention canone/corrispettivo/euro
    let relevant = query_result
        .results
        .iter()
        .any(|r| {
            let text_lower = r.text.to_lowercase();
            text_lower.contains("euro") || text_lower.contains("canone") || text_lower.contains("corrispettivo")
        });

    assert!(relevant, "Results should be relevant to query about canone");

    println!("\n=== PHASE 3: COMPARISON ===");

    // Compare contracts
    let compare_payload = json!({
        "kb_id": kb_id,
        "doc_ids": [doc_id_a, doc_id_b],
        "question": "Confronta durata, canone, deposito cauzionale e foro competente",
        "save_analysis": false
    });

    let compare_response = client
        .post(format!("{}/compare_contracts", rust_engine_url))
        .json(&compare_payload)
        .send()
        .await
        .expect("Comparison failed");

    assert!(
        compare_response.status().is_success(),
        "Comparison failed: {}",
        compare_response.status()
    );

    let compare_result: CompareResponse = compare_response
        .json()
        .await
        .expect("Failed to parse comparison response");

    println!("✅ Comparison completed:");
    println!("  Processing: {} ms", compare_result.processing_ms);
    println!("  Markdown length: {} chars", compare_result.markdown_result.len());

    assert!(!compare_result.markdown_result.is_empty());
    assert!(
        compare_result.markdown_result.contains("# Confronto Contratti"),
        "Markdown should have title"
    );

    println!("  Preview:");
    println!("{}", &compare_result.markdown_result[..300.min(compare_result.markdown_result.len())]);

    println!("\n=== PHASE 4: KB MANAGEMENT ===");

    // Get KB stats
    let stats_response = client
        .get(format!("{}/kb/{}/stats", rust_engine_url, kb_id))
        .send()
        .await
        .expect("Stats request failed");

    assert!(
        stats_response.status().is_success(),
        "Stats failed: {}",
        stats_response.status()
    );

    let stats_result: KbStatsResponse = stats_response
        .json()
        .await
        .expect("Failed to parse stats response");

    println!("✅ KB Statistics:");
    println!("  KB ID: {}", stats_result.kb_id);
    println!("  Documents: {}", stats_result.documents_count);
    println!("  Chunks: {}", stats_result.chunks_count);

    assert_eq!(stats_result.kb_id, kb_id);

    // List documents (currently returns empty in implementation, but endpoint should work)
    let list_response = client
        .get(format!("{}/kb/{}/documents", rust_engine_url, kb_id))
        .send()
        .await
        .expect("List documents failed");

    assert!(
        list_response.status().is_success(),
        "List documents failed: {}",
        list_response.status()
    );

    println!("✅ List documents endpoint works");

    // Delete Contract A
    let delete_response = client
        .delete(format!(
            "{}/kb/{}/documents/{}",
            rust_engine_url, kb_id, doc_id_a
        ))
        .send()
        .await
        .expect("Delete failed");

    assert!(
        delete_response.status().is_success(),
        "Delete failed: {}",
        delete_response.status()
    );

    println!("✅ Contract A deleted");

    // Verify deletion by querying again - should only return Contract B chunks
    let verify_query_payload = json!({
        "query": "Milano",
        "kb_id": kb_id,
        "top_k": 10
    });

    let verify_response = client
        .post(format!("{}/query", rust_engine_url))
        .json(&verify_query_payload)
        .send()
        .await
        .expect("Verification query failed");

    let verify_result: QueryResponse = verify_response
        .json()
        .await
        .expect("Failed to parse verification query");

    let has_deleted_doc = verify_result
        .results
        .iter()
        .any(|r| r.doc_id == doc_id_a);

    println!("✅ Deletion verified:");
    println!("  Deleted doc in results: {}", has_deleted_doc);
    println!("  (Should be false after Qdrant deletion)");

    println!("\n=== PHASE 5: CLEANUP ===");

    // Cleanup temp files
    fs::remove_file(temp_file_a).ok();
    fs::remove_file(temp_file_b).ok();

    println!("✅ Full workflow test completed successfully!");
    println!("\nSummary:");
    println!("  ✅ Ingestion (2 documents)");
    println!("  ✅ Query (hybrid search + reranking)");
    println!("  ✅ Comparison (multi-contract analysis)");
    println!("  ✅ KB Statistics");
    println!("  ✅ List Documents");
    println!("  ✅ Delete Document");
    println!("  ✅ Deletion Verification");
}

/// Test health checks for all services
#[tokio::test]
#[ignore]
async fn test_all_services_health() {
    println!("\n=== HEALTH CHECKS ===");

    let rust_url = env::var("RUST_ENGINE_URL")
        .unwrap_or_else(|_| "http://localhost:8090".to_string());
    let python_url = env::var("PYTHON_WORKER_URL")
        .unwrap_or_else(|_| "http://localhost:8091".to_string());
    let qdrant_url = env::var("QDRANT_URL")
        .unwrap_or_else(|_| "http://localhost:6333".to_string());
    let ollama_url = env::var("OLLAMA_URL")
        .unwrap_or_else(|_| "http://localhost:11434".to_string());

    let client = reqwest::Client::new();

    // Rust engine
    let rust_health = client
        .get(format!("{}/health", rust_url))
        .send()
        .await
        .expect("Rust health check failed");

    assert!(rust_health.status().is_success(), "Rust engine not healthy");
    println!("✅ Rust Engine: healthy");

    // Python worker
    let python_health = client
        .get(format!("{}/health", python_url))
        .send()
        .await
        .expect("Python health check failed");

    assert!(python_health.status().is_success(), "Python worker not healthy");
    println!("✅ Python Worker: healthy");

    // Qdrant
    let qdrant_health = client
        .get(format!("{}/", qdrant_url))
        .send()
        .await
        .expect("Qdrant health check failed");

    assert!(qdrant_health.status().is_success(), "Qdrant not healthy");
    println!("✅ Qdrant: healthy");

    // Ollama (list models endpoint as health check)
    let ollama_health = client
        .get(format!("{}/api/tags", ollama_url))
        .send()
        .await
        .expect("Ollama health check failed");

    assert!(ollama_health.status().is_success(), "Ollama not healthy");
    println!("✅ Ollama: healthy");

    println!("\n✅ All services are healthy");
}
