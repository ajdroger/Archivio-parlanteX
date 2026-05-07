/// Multi-provider LLM abstraction layer
///
/// Supports runtime switching between Ollama (local, default) and cloud providers
/// (Anthropic, Google, OpenAI, DeepSeek, etc.) with unified interface.

pub mod types;
pub mod ollama;
pub mod registry;

use async_trait::async_trait;
use crate::errors::Result;
pub use types::*;

/// LLM provider capability trait
///
/// All providers (local and cloud) implement this trait for seamless switching.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Provider identifier (e.g., "ollama", "anthropic", "openai")
    fn name(&self) -> &str;

    /// List available models for this provider
    async fn available_models(&self) -> Result<Vec<ModelInfo>>;

    /// Check if provider is currently available (API key set, service reachable)
    async fn is_available(&self) -> bool;

    /// Non-streaming chat completion
    ///
    /// # Errors
    /// Returns `AppError::Ollama`, `AppError::Internal`, or provider-specific errors
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;

    /// Simple text generation (convenience method)
    ///
    /// # Errors
    /// Returns `AppError::Ollama`, `AppError::Internal`, or provider-specific errors
    async fn generate(&self, prompt: &str, max_tokens: usize, temperature: f32) -> Result<String> {
        let request = ChatRequest {
            model: String::new(), // Provider implementations will override with actual model
            messages: vec![Message {
                role: Role::User,
                content: prompt.to_string(),
            }],
            response_format: None,
            temperature: Some(temperature),
            max_tokens: Some(max_tokens as u32),
            stop: None,
        };
        let response = self.chat(request).await?;
        Ok(response.content)
    }

    /// Streaming chat completion (for real-time UX)
    ///
    /// Returns async stream of events (delta text, tool calls, finish reason)
    ///
    /// # Errors
    /// Returns `AppError::Ollama`, `AppError::Internal`, or provider-specific errors
    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<Box<dyn futures::Stream<Item = Result<StreamEvent>> + Send + Unpin>>;

    /// Generate embeddings for text chunks
    ///
    /// # Errors
    /// Returns `AppError::Ollama`, `AppError::Internal`, or provider-specific errors
    async fn embed(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>>;
}
