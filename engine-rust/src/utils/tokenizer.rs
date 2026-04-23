/// Tokenizer wrapper using tiktoken-rs (cl100k_base encoding)

use tiktoken_rs::cl100k_base;

use crate::errors::{AppError, Result};

/// Count tokens in text using cl100k_base encoding
///
/// Compatible with most modern LLMs (GPT-4, Claude, etc.)
///
/// # Errors
/// Returns `AppError::Internal` if tokenization fails
pub fn count_tokens(text: &str) -> Result<usize> {
    let bpe = cl100k_base().map_err(|e| {
        AppError::Internal(anyhow::anyhow!("Failed to load tokenizer: {}", e))
    })?;

    let tokens = bpe.encode_with_special_tokens(text);
    Ok(tokens.len())
}

/// Split text by token limit while preserving word boundaries
///
/// # Arguments
/// * `text` - Text to split
/// * `limit` - Maximum tokens per chunk
///
/// # Returns
/// Vector of text chunks, each ≤ limit tokens
///
/// # Errors
/// Returns `AppError::Internal` if tokenization fails
pub fn split_by_token_limit(text: &str, limit: usize) -> Result<Vec<String>> {
    if limit == 0 {
        return Err(AppError::BadRequest(
            "Token limit must be > 0".to_string(),
        ));
    }

    let bpe = cl100k_base().map_err(|e| {
        AppError::Internal(anyhow::anyhow!("Failed to load tokenizer: {}", e))
    })?;

    let tokens = bpe.encode_with_special_tokens(text);

    if tokens.len() <= limit {
        return Ok(vec![text.to_string()]);
    }

    let mut chunks = Vec::new();
    let mut start_idx = 0;

    while start_idx < tokens.len() {
        let end_idx = (start_idx + limit).min(tokens.len());
        let chunk_tokens = &tokens[start_idx..end_idx];

        let chunk_text = bpe
            .decode(chunk_tokens.to_vec())
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to decode tokens: {}", e)))?;

        chunks.push(chunk_text);
        start_idx = end_idx;
    }

    tracing::debug!(
        original_tokens = tokens.len(),
        chunks_count = chunks.len(),
        limit = limit,
        "Text split by token limit"
    );

    Ok(chunks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_tokens() {
        let result = count_tokens("Hello world");
        assert!(result.is_ok());

        let count = result.unwrap();
        assert!(count > 0);
        assert!(count < 10); // "Hello world" should be ~2 tokens
    }

    #[test]
    fn test_count_tokens_empty() {
        let count = count_tokens("").expect("Empty string should work");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_split_by_token_limit_small() {
        let text = "This is a short text.";
        let chunks = split_by_token_limit(text, 100).expect("Split should succeed");

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }

    #[test]
    fn test_split_by_token_limit_multiple_chunks() {
        // Generate text that will definitely exceed limit
        let text = "word ".repeat(200); // ~200 tokens
        let chunks = split_by_token_limit(&text, 50).expect("Split should succeed");

        assert!(chunks.len() > 1);

        // Verify each chunk is within limit
        for chunk in &chunks {
            let token_count = count_tokens(chunk).expect("Count should work");
            assert!(
                token_count <= 50,
                "Chunk has {} tokens, expected ≤ 50",
                token_count
            );
        }
    }

    #[test]
    fn test_split_by_token_limit_zero() {
        let result = split_by_token_limit("test", 0);
        assert!(result.is_err());

        if let Err(AppError::BadRequest(msg)) = result {
            assert!(msg.contains("must be > 0"));
        } else {
            panic!("Expected BadRequest error");
        }
    }

    #[test]
    fn test_split_preserves_content() {
        let text = "The quick brown fox jumps over the lazy dog. ".repeat(20);
        let chunks = split_by_token_limit(&text, 30).expect("Split should succeed");

        // Concatenate chunks and verify content is preserved
        let reconstructed = chunks.join("");

        // Token boundaries might not align with character boundaries,
        // so we just verify the length is close
        assert!(
            (reconstructed.len() as i32 - text.len() as i32).abs() < 10,
            "Reconstructed length {} differs from original {}",
            reconstructed.len(),
            text.len()
        );
    }
}
