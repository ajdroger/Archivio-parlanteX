/// Ollama integration smoke test
///
/// Verifies connectivity and basic operations with Ollama service.
/// Requires Ollama running at OLLAMA_URL from .env

use archivio_parlante_rust_engine::providers::{
    ollama::OllamaProvider, types::*, LlmProvider,
};

#[tokio::test]
#[ignore] // Run with: cargo test --test ollama_smoke -- --ignored
async fn test_ollama_connectivity() {
    let ollama_url = std::env::var("OLLAMA_URL")
        .unwrap_or_else(|_| "http://localhost:11434".to_string());

    let provider = OllamaProvider::new(ollama_url, 8, "nomic-embed-text".to_string());

    // Test 1: Check availability
    let available = provider.is_available().await;
    assert!(
        available,
        "Ollama is not available. Is the service running?"
    );

    println!("✅ Ollama is available");

    // Test 2: List models
    let models = provider
        .available_models()
        .await
        .expect("Failed to fetch models");

    assert!(!models.is_empty(), "No models available in Ollama");

    println!("✅ Found {} models:", models.len());
    for model in &models {
        println!("  - {}", model.id);
    }

    // Test 3: Simple chat completion
    let chat_request = ChatRequest {
        model: "qwen2.5:7b-instruct-q4_K_M".to_string(),
        messages: vec![
            Message::system("You are a helpful assistant."),
            Message::user("Say 'Hello from Rust!' in one sentence."),
        ],
        response_format: None,
        temperature: Some(0.7),
        max_tokens: Some(50),
        stop: None,
    };

    let response = provider
        .chat(chat_request)
        .await
        .expect("Chat request failed");

    assert!(!response.content.is_empty(), "Empty response from Ollama");
    assert_eq!(response.provider, "ollama");
    assert_eq!(response.cost_eur, 0.0);
    assert!(response.usage.total_tokens > 0);

    println!("✅ Chat completion successful");
    println!("  Response: {}", response.content.trim());
    println!(
        "  Tokens: {} prompt + {} completion = {} total",
        response.usage.prompt_tokens,
        response.usage.completion_tokens,
        response.usage.total_tokens
    );
}

#[tokio::test]
#[ignore]
async fn test_ollama_embeddings() {
    let ollama_url = std::env::var("OLLAMA_URL")
        .unwrap_or_else(|_| "http://localhost:11434".to_string());

    let provider = OllamaProvider::new(ollama_url, 8, "nomic-embed-text".to_string());

    let texts = vec![
        "Contratto di fornitura servizi".to_string(),
        "Clausola penale".to_string(),
    ];

    let embeddings = provider
        .embed(texts.clone())
        .await
        .expect("Embedding failed");

    assert_eq!(embeddings.len(), texts.len());

    for (i, emb) in embeddings.iter().enumerate() {
        assert_eq!(emb.len(), 768, "nomic-embed-text produces 768-dim vectors");
        println!("✅ Embedding {} dimension: {}", i, emb.len());
    }
}
