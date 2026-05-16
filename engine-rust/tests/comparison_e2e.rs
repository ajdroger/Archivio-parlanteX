/// End-to-end test for multi-contract comparison
///
/// Requires full stack running:
/// - Ollama (embeddings + LLM for aspects/synthesis)
/// - Qdrant (vector storage)
/// - Python worker (parsing + reranking)
///
/// Run with: cargo test --test comparison_e2e -- --ignored --nocapture

use archivio_parlante_rust_engine::models::comparison::CompareRequest;
use archivio_parlante_rust_engine::models::document::{IngestRequest, IngestResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::fs;

const NDA_CONTRACT_2023: &str = r#"
ACCORDO DI NON DIVULGAZIONE

Stipulato in data 15 gennaio 2023

Art. 1 - Oggetto
Il presente accordo ha per oggetto la tutela delle informazioni riservate scambiate
tra le parti nel contesto di una potenziale collaborazione commerciale.

Art. 2 - Durata
Il presente accordo avrà durata di 24 mesi a decorrere dalla data di sottoscrizione
e si rinnoverà tacitamente per ulteriori 12 mesi salvo disdetta.

Art. 3 - Obblighi di riservatezza
La parte ricevente si impegna a:
- Mantenere strettamente riservate tutte le informazioni
- Non divulgare a terzi senza consenso scritto
- Utilizzare le informazioni solo per gli scopi concordati

Art. 4 - Penali
In caso di violazione degli obblighi di riservatezza, la parte inadempiente
sarà tenuta al pagamento di una penale pari a Euro 25.000,00 (venticinquemila/00).

Art. 5 - Foro competente
Per qualsiasi controversia derivante dal presente accordo, è competente
in via esclusiva il Foro di Milano.

Art. 6 - Legge applicabile
Il presente accordo è regolato dalla legge italiana.
"#;

const NDA_CONTRACT_2024: &str = r#"
ACCORDO DI RISERVATEZZA

Stipulato in data 10 marzo 2024

Art. 1 - Finalità
Questo accordo disciplina la protezione delle informazioni confidenziali
che saranno condivise tra le parti durante le trattative commerciali.

Art. 2 - Periodo di validità
L'accordo ha validità di 36 mesi dalla firma e non prevede rinnovo automatico.

Art. 3 - Impegni di confidenzialità
Il destinatario delle informazioni si obbliga a:
- Custodire con diligenza le informazioni ricevute
- Non comunicare a terzi senza autorizzazione preventiva scritta
- Impiegare le informazioni esclusivamente per le finalità pattuite
- Restituire o distruggere le informazioni su richiesta

Art. 4 - Sanzioni per inadempimento
La violazione degli obblighi di confidenzialità comporta il pagamento
di una penale pari a Euro 50.000,00 (cinquantamila/00) oltre al risarcimento
del danno ulteriore eventualmente subito.

Art. 5 - Giurisdizione
Per ogni controversia relativa all'interpretazione ed esecuzione del presente
accordo è competente esclusivamente il Tribunale di Roma.

Art. 6 - Normativa di riferimento
Si applica la legislazione italiana vigente.
"#;

#[derive(Debug, Deserialize)]
struct CompareResponse {
    markdown_result: String,
    structured: ComparisonResult,
    processing_ms: u64,
    analysis_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ComparisonResult {
    aspects: Vec<ComparisonAspect>,
    differences_summary: String,
    recommendations: Vec<String>,
    information_gaps: Vec<String>,
    processing_ms: u64,
}

#[derive(Debug, Deserialize)]
struct ComparisonAspect {
    name: String,
    description: Option<String>,
    cells: serde_json::Value, // HashMap as JSON value for easier testing
}

/// Test multi-contract comparison with 2 NDA contracts
#[tokio::test]
#[ignore] // Requires stack up: make up
async fn test_compare_two_nda_contracts() {
    let rust_engine_url =
        env::var("RUST_ENGINE_URL").unwrap_or_else(|_| "http://localhost:8090".to_string());

    let client = reqwest::Client::new();

    let kb_id = "kb_comparison_test_e2e".to_string();

    // Step 1: Ingest first contract (NDA 2023)
    let temp_file_1 = "/tmp/nda_2023_comparison_test.txt";
    fs::write(temp_file_1, NDA_CONTRACT_2023).expect("Failed to write test file 1");

    let doc_id_1 = format!("nda_2023_{}", chrono::Utc::now().timestamp());

    let ingest_request_1 = IngestRequest {
        doc_id: doc_id_1.clone(),
        kb_id: kb_id.clone(),
        file_path: temp_file_1.to_string(),
        source_name: "NDA_2023.txt".to_string(),
        mime_type: "text/plain".to_string(),
        tags: vec!["test".to_string(), "nda".to_string(), "2023".to_string()],
    };

    let ingest_response_1 = client
        .post(format!("{}/ingest", rust_engine_url))
        .json(&ingest_request_1)
        .send()
        .await
        .expect("Ingestion request 1 failed");

    assert!(
        ingest_response_1.status().is_success(),
        "Ingestion 1 failed: {}",
        ingest_response_1.status()
    );

    let ingest_result_1: IngestResponse = ingest_response_1
        .json()
        .await
        .expect("Failed to parse ingestion response 1");

    println!("✅ Ingestion 1 completed: {} chunks", ingest_result_1.chunks_indexed);

    // Step 2: Ingest second contract (NDA 2024)
    let temp_file_2 = "/tmp/nda_2024_comparison_test.txt";
    fs::write(temp_file_2, NDA_CONTRACT_2024).expect("Failed to write test file 2");

    let doc_id_2 = format!("nda_2024_{}", chrono::Utc::now().timestamp());

    let ingest_request_2 = IngestRequest {
        doc_id: doc_id_2.clone(),
        kb_id: kb_id.clone(),
        file_path: temp_file_2.to_string(),
        source_name: "NDA_2024.txt".to_string(),
        mime_type: "text/plain".to_string(),
        tags: vec!["test".to_string(), "nda".to_string(), "2024".to_string()],
    };

    let ingest_response_2 = client
        .post(format!("{}/ingest", rust_engine_url))
        .json(&ingest_request_2)
        .send()
        .await
        .expect("Ingestion request 2 failed");

    assert!(
        ingest_response_2.status().is_success(),
        "Ingestion 2 failed: {}",
        ingest_response_2.status()
    );

    let ingest_result_2: IngestResponse = ingest_response_2
        .json()
        .await
        .expect("Failed to parse ingestion response 2");

    println!("✅ Ingestion 2 completed: {} chunks", ingest_result_2.chunks_indexed);

    // Step 3: Wait a bit for indexing to complete
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Step 4: Compare contracts
    let compare_request = CompareRequest {
        kb_id: kb_id.clone(),
        doc_ids: vec![doc_id_1.clone(), doc_id_2.clone()],
        question: "Confronta durata, penali e foro competente di questi due accordi di riservatezza".to_string(),
        save_analysis: false,
    };

    let compare_response = client
        .post(format!("{}/compare_contracts", rust_engine_url))
        .json(&compare_request)
        .send()
        .await
        .expect("Comparison request failed");

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
    println!("  Aspects: {}", compare_result.structured.aspects.len());
    println!("  Processing time: {} ms", compare_result.processing_ms);
    println!("  Differences summary length: {}", compare_result.structured.differences_summary.len());

    // Step 5: Verify results
    assert!(
        !compare_result.structured.aspects.is_empty(),
        "Should have at least one aspect"
    );

    assert!(
        compare_result.structured.aspects.len() >= 3,
        "Should have at least 3 aspects (durata, penali, foro), got {}",
        compare_result.structured.aspects.len()
    );

    // Verify aspects include expected topics
    let aspect_names: Vec<String> = compare_result
        .structured
        .aspects
        .iter()
        .map(|a| a.name.to_lowercase())
        .collect();

    println!("  Aspects found: {:?}", aspect_names);

    let has_duration = aspect_names.iter().any(|n| n.contains("durata") || n.contains("validità") || n.contains("periodo"));
    let has_penalties = aspect_names.iter().any(|n| n.contains("penal") || n.contains("sanzione"));
    let has_jurisdiction = aspect_names.iter().any(|n| n.contains("foro") || n.contains("giurisdizione") || n.contains("tribunale"));

    assert!(
        has_duration,
        "Should have duration aspect. Aspects: {:?}",
        aspect_names
    );
    assert!(
        has_penalties,
        "Should have penalties aspect. Aspects: {:?}",
        aspect_names
    );
    assert!(
        has_jurisdiction,
        "Should have jurisdiction aspect. Aspects: {:?}",
        aspect_names
    );

    // Verify each aspect has cells for both documents
    for aspect in &compare_result.structured.aspects {
        let cells_map = aspect.cells.as_object().expect("Cells should be object");

        assert!(
            cells_map.contains_key(&doc_id_1) || cells_map.contains_key(&doc_id_2),
            "Aspect '{}' should have cells for at least one document",
            aspect.name
        );

        // Verify cell structure
        for (doc_id, cell) in cells_map {
            let present = cell["present"].as_bool().expect("present should be bool");

            if present {
                assert!(
                    cell["text_quote"].is_string() && !cell["text_quote"].as_str().unwrap().is_empty(),
                    "Aspect '{}' doc '{}': present=true but text_quote is empty or missing",
                    aspect.name,
                    doc_id
                );

                println!(
                    "  {} ({}): {}",
                    aspect.name,
                    doc_id,
                    &cell["text_quote"].as_str().unwrap()[..50.min(cell["text_quote"].as_str().unwrap().len())]
                );
            } else {
                assert!(
                    cell["text_quote"].is_null(),
                    "Aspect '{}' doc '{}': present=false but text_quote is not null",
                    aspect.name,
                    doc_id
                );

                println!("  {} ({}): NON PRESENTE", aspect.name, doc_id);
            }

            let confidence = cell["confidence"].as_f64().expect("confidence should be number");
            assert!(
                (0.0..=1.0).contains(&confidence),
                "Confidence should be in [0, 1], got {}",
                confidence
            );
        }
    }

    // Verify differences summary exists
    assert!(
        !compare_result.structured.differences_summary.is_empty(),
        "Should have differences summary"
    );

    println!("\n  Differences summary:");
    println!("  {}", compare_result.structured.differences_summary);

    // Verify markdown output
    assert!(
        !compare_result.markdown_result.is_empty(),
        "Should have markdown result"
    );

    assert!(
        compare_result.markdown_result.contains("# Confronto Contratti"),
        "Markdown should have title"
    );

    assert!(
        compare_result.markdown_result.contains("## Tabella Comparativa"),
        "Markdown should have comparison table"
    );

    println!("\n  Markdown output preview:");
    println!("{}", &compare_result.markdown_result[..500.min(compare_result.markdown_result.len())]);

    // Cleanup
    fs::remove_file(temp_file_1).ok();
    fs::remove_file(temp_file_2).ok();
    println!("\n✅ Test cleanup completed");
}

/// Test comparison validation errors
#[tokio::test]
#[ignore]
async fn test_comparison_validation_errors() {
    let rust_engine_url =
        env::var("RUST_ENGINE_URL").unwrap_or_else(|_| "http://localhost:8090".to_string());

    let client = reqwest::Client::new();

    // Test 1: Empty kb_id
    let payload = json!({
        "kb_id": "",
        "doc_ids": ["doc_1", "doc_2"],
        "question": "test",
        "save_analysis": false
    });

    let response = client
        .post(format!("{}/compare_contracts", rust_engine_url))
        .json(&payload)
        .send()
        .await
        .expect("Request failed");

    assert_eq!(response.status(), 400, "Should return 400 for empty kb_id");

    // Test 2: Insufficient documents (only 1)
    let payload = json!({
        "kb_id": "kb_test",
        "doc_ids": ["doc_1"],
        "question": "test",
        "save_analysis": false
    });

    let response = client
        .post(format!("{}/compare_contracts", rust_engine_url))
        .json(&payload)
        .send()
        .await
        .expect("Request failed");

    assert_eq!(
        response.status(),
        400,
        "Should return 400 for insufficient documents"
    );

    // Test 3: Too many documents (> 10)
    let payload = json!({
        "kb_id": "kb_test",
        "doc_ids": (0..11).map(|i| format!("doc_{}", i)).collect::<Vec<_>>(),
        "question": "test",
        "save_analysis": false
    });

    let response = client
        .post(format!("{}/compare_contracts", rust_engine_url))
        .json(&payload)
        .send()
        .await
        .expect("Request failed");

    assert_eq!(
        response.status(),
        400,
        "Should return 400 for too many documents"
    );

    // Test 4: Duplicate doc_ids
    let payload = json!({
        "kb_id": "kb_test",
        "doc_ids": ["doc_1", "doc_2", "doc_1"],
        "question": "test",
        "save_analysis": false
    });

    let response = client
        .post(format!("{}/compare_contracts", rust_engine_url))
        .json(&payload)
        .send()
        .await
        .expect("Request failed");

    assert_eq!(
        response.status(),
        400,
        "Should return 400 for duplicate doc_ids"
    );

    // Test 5: Empty question
    let payload = json!({
        "kb_id": "kb_test",
        "doc_ids": ["doc_1", "doc_2"],
        "question": "   ",
        "save_analysis": false
    });

    let response = client
        .post(format!("{}/compare_contracts", rust_engine_url))
        .json(&payload)
        .send()
        .await
        .expect("Request failed");

    assert_eq!(
        response.status(),
        400,
        "Should return 400 for empty question"
    );

    println!("✅ Validation tests passed");
}
