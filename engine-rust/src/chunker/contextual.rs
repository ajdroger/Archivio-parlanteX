/// Contextual Retrieval enrichment (Anthropic technique)
///
/// Adds context prefix to each chunk using LLM to improve retrieval accuracy

use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::errors::{AppError, Result};
use crate::models::chunk::Chunk;
use crate::providers::{types::*, LlmProvider};
use crate::utils::tokenizer;

/// Contextual retrieval enricher using LLM
///
/// Implements Anthropic's contextual retrieval technique:
/// https://www.anthropic.com/news/contextual-retrieval
pub struct ContextualRetrievalEnricher {
    /// LLM provider for context generation
    llm: Arc<dyn LlmProvider>,

    /// Model to use for enrichment
    model: String,

    /// Rate limiter (max concurrent requests)
    semaphore: Arc<Semaphore>,

    /// Cache: chunk text hash -> generated context
    cache: Arc<DashMap<u64, String>>,
}

impl ContextualRetrievalEnricher {
    /// Create new contextual enricher
    ///
    /// # Arguments
    /// * `llm` - LLM provider
    /// * `model` - Model identifier (e.g., "qwen2.5:7b")
    /// * `max_concurrent` - Max parallel enrichment requests
    pub fn new(llm: Arc<dyn LlmProvider>, model: String, max_concurrent: usize) -> Self {
        Self {
            llm,
            model,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            cache: Arc::new(DashMap::new()),
        }
    }

    /// Enrich chunks with contextual prefixes
    ///
    /// # Arguments
    /// * `full_document_text` - Complete document text for summary
    /// * `chunks` - Mutable slice of chunks to enrich
    ///
    /// # Returns
    /// Ok(()) if enrichment succeeds, Err if critical failure
    ///
    /// Note: Individual chunk enrichment failures are logged but don't fail the whole operation
    pub async fn enrich(&self, full_document_text: &str, chunks: &mut [Chunk]) -> Result<()> {
        let doc_tokens = tokenizer::count_tokens(full_document_text)?;

        // Skip if document too short
        if doc_tokens < 2000 {
            tracing::debug!(
                tokens = doc_tokens,
                "Document too short for contextual enrichment, skipping"
            );
            return Ok(());
        }

        tracing::info!(
            doc_tokens = doc_tokens,
            chunks_count = chunks.len(),
            "Starting contextual enrichment"
        );

        // Generate document summary if large
        let summary = if doc_tokens > 8000 {
            self.generate_summary(full_document_text).await?
        } else {
            // For medium docs, use truncated version as summary
            let chars_limit = (full_document_text.len() * 8000 / doc_tokens).min(full_document_text.len());
            full_document_text[..chars_limit].to_string()
        };

        // Enrich chunks in parallel
        let mut tasks = Vec::new();

        for (i, chunk) in chunks.iter().enumerate() {
            let chunk_text = chunk.text.clone();
            let summary_clone = summary.clone();
            let enricher = self.clone_for_task();

            let task = tokio::spawn(async move {
                enricher.enrich_single(&summary_clone, &chunk_text).await
            });

            tasks.push((i, task));
        }

        // Collect results
        let mut success_count = 0;
        let mut failure_count = 0;

        for (i, task) in tasks {
            match task.await {
                Ok(Ok(context)) => {
                    let chunk = &mut chunks[i];
                    chunk.contextual_text = Some(format!("{}\n\n{}", context, chunk.text));
                    success_count += 1;
                }
                Ok(Err(e)) => {
                    tracing::warn!(error = %e, "Failed to enrich chunk, using original text");
                    failure_count += 1;
                }
                Err(e) => {
                    tracing::error!(error = %e, "Task panic during enrichment");
                    failure_count += 1;
                }
            }
        }

        tracing::info!(
            success = success_count,
            failed = failure_count,
            "Contextual enrichment completed"
        );

        Ok(())
    }

    /// Generate document summary
    async fn generate_summary(&self, text: &str) -> Result<String> {
        let _permit = self.semaphore.acquire().await.map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Semaphore error: {}", e))
        })?;

        let prompt = format!(
            "Riassumi questo documento in italiano in 2-3 frasi (~100 parole), \
             evidenziando il tipo di documento, le parti coinvolte, e l'oggetto principale:\n\n{}",
            text.chars().take(20000).collect::<String>()
        );

        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![Message::user(prompt)],
            response_format: None,
            temperature: Some(0.3),
            max_tokens: Some(300),
            stop: None,
        };

        let response = self.llm.chat(request).await?;

        tracing::debug!(
            summary_length = response.content.len(),
            "Document summary generated"
        );

        Ok(response.content.trim().to_string())
    }

    /// Enrich single chunk
    async fn enrich_single(&self, summary: &str, chunk_text: &str) -> Result<String> {
        // Check cache
        let cache_key = self.hash_text(chunk_text);
        if let Some(cached) = self.cache.get(&cache_key) {
            tracing::debug!("Using cached context");
            return Ok(cached.clone());
        }

        let _permit = self.semaphore.acquire().await.map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Semaphore error: {}", e))
        })?;

        let prompt = format!(
            "<document_summary>{}</document_summary>\n\
             <chunk>{}</chunk>\n\n\
             Fornisci un breve contesto (1-2 frasi, max 80 token) che situi questo chunk \
             nel documento intero per migliorare il retrieval. Rispondi SOLO con il contesto, \
             senza preamboli.",
            summary,
            chunk_text.chars().take(2000).collect::<String>()
        );

        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![Message::user(prompt)],
            response_format: None,
            temperature: Some(0.2),
            max_tokens: Some(100),
            stop: None,
        };

        let response = self.llm.chat(request).await?;
        let context = response.content.trim().to_string();

        // Cache result
        self.cache.insert(cache_key, context.clone());

        Ok(context)
    }

    /// Hash text for caching
    fn hash_text(&self, text: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        hasher.finish()
    }

    /// Clone for async task (Arc-based, cheap)
    fn clone_for_task(&self) -> Self {
        Self {
            llm: self.llm.clone(),
            model: self.model.clone(),
            semaphore: self.semaphore.clone(),
            cache: self.cache.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enricher_creation() {
        use crate::providers::ollama::OllamaProvider;

        let provider = Arc::new(OllamaProvider::new(
            "http://localhost:11434".to_string(),
            8,
            "nomic-embed-text".to_string(),
        ));
        let enricher = ContextualRetrievalEnricher::new(
            provider,
            "qwen2.5:7b".to_string(),
            16,
        );

        assert_eq!(enricher.model, "qwen2.5:7b");
        assert_eq!(enricher.cache.len(), 0);
    }

    #[test]
    fn test_hash_consistency() {
        use crate::providers::ollama::OllamaProvider;

        let provider = Arc::new(OllamaProvider::new(
            "http://localhost:11434".to_string(),
            8,
            "nomic-embed-text".to_string(),
        ));
        let enricher = ContextualRetrievalEnricher::new(
            provider,
            "qwen2.5:7b".to_string(),
            16,
        );

        let text = "Same text for hashing";
        let hash1 = enricher.hash_text(text);
        let hash2 = enricher.hash_text(text);

        assert_eq!(hash1, hash2);
    }
}
