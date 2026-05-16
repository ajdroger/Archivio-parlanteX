/// Chunk demo CLI tool
///
/// Usage: cargo run --example chunk_demo -- <file_path>

use std::env;
use std::fs;

use archivio_parlante_rust_engine::chunker::semantic::SemanticChunker;

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Get file path from args
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        eprintln!("\nExample:");
        eprintln!("  cargo run --example chunk_demo -- contract.txt");
        std::process::exit(1);
    }

    let file_path = &args[1];

    // Read file
    let text = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", file_path, e);
            std::process::exit(1);
        }
    };

    println!("📄 File: {}", file_path);
    println!("📏 Size: {} bytes", text.len());
    println!();

    // Create chunker
    let chunker = SemanticChunker::new(800, 0.15);

    // Chunk document
    let doc_id = format!("demo_{}", chrono::Utc::now().timestamp());
    let chunks = match chunker.chunk(&text, &doc_id) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error chunking document: {}", e);
            std::process::exit(1);
        }
    };

    println!("✂️  Chunked into {} parts\n", chunks.len());
    println!("{}", "=".repeat(80));

    // Display chunks
    for (idx, chunk) in chunks.iter().enumerate() {
        println!("\n📌 Chunk #{} (ID: {})", idx + 1, chunk.id);
        println!("   Tokens: {}", chunk.token_count);
        println!("   Offsets: {} → {}", chunk.start_offset, chunk.end_offset);

        if let Some(header) = chunk.metadata.get("section_header") {
            println!("   Header: {}", header);
        }

        if let Some(clause) = chunk.metadata.get("clause_marker") {
            println!("   Clause: {}", clause);
        }

        println!();
        println!("   Text preview:");
        let preview = chunk.text.chars().take(200).collect::<String>();
        for line in preview.lines().take(5) {
            println!("   | {}", line);
        }

        if chunk.text.len() > 200 {
            println!("   | ... ({} more chars)", chunk.text.len() - 200);
        }

        println!("{}", "-".repeat(80));
    }

    // Summary statistics
    println!("\n📊 Statistics:");
    println!("   Total chunks: {}", chunks.len());

    let total_tokens: usize = chunks.iter().map(|c| c.token_count).sum();
    let avg_tokens = if !chunks.is_empty() {
        total_tokens / chunks.len()
    } else {
        0
    };

    println!("   Total tokens: {}", total_tokens);
    println!("   Average tokens/chunk: {}", avg_tokens);

    let min_tokens = chunks.iter().map(|c| c.token_count).min().unwrap_or(0);
    let max_tokens = chunks.iter().map(|c| c.token_count).max().unwrap_or(0);

    println!("   Min tokens: {}", min_tokens);
    println!("   Max tokens: {}", max_tokens);

    let clauses_count = chunks
        .iter()
        .filter(|c| c.metadata.get("clause_marker").is_some())
        .count();

    println!("   Chunks with clause markers: {}", clauses_count);

    let headers_count = chunks
        .iter()
        .filter(|c| c.metadata.get("section_header").is_some())
        .count();

    println!("   Chunks with section headers: {}", headers_count);
}
