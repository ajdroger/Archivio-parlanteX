/// Anthropic Claude provider implementation
///
/// Supports Claude 3.5 Sonnet, Opus, Haiku models via Messages API.
/// Cost tracking enabled for daily budget enforcement.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::errors::{AppError, Result};
use super::{
    LlmProvider, ChatRequest, ChatResponse, Message as ProviderMessage, Role, Usage,
    FinishReason, ModelInfo, StreamEvent,
};

/// Anthropic API endpoint
const ANTHROPIC_API_BASE: &str = "https://api.anthropic.com/v1";
const ANTHROPIC_API_VERSION: &str = "2023-06-01";

/// Anthropic provider
pub struct AnthropicProvider {
    api_key: String,
    client: Client,
    semaphore: Arc<Semaphore>,
    default_model: String,
}

impl AnthropicProvider {
    /// Create new Anthropic provider
    ///
    /// # Arguments
    /// * `api_key` - Anthropic API key (sk-ant-...)
    /// * `max_concurrent` - Max concurrent requests
    /// * `default_model` - Default model (e.g., "claude-3-5-sonnet-20241022")
    pub fn new(api_key: String, max_concurrent: usize, default_model: Option<String>) -> Self {
        Self {
            api_key,
            client: Client::new(),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            default_model: default_model.unwrap_or_else(|| "claude-3-5-sonnet-20241022".to_string()),
        }
    }

    /// Calculate cost in EUR for Anthropic API call
    fn calculate_cost(model: &str, prompt_tokens: u32, completion_tokens: u32) -> f64 {
        // Pricing as of Jan 2025 (EUR, converted from USD at ~1.1 rate)
        let (input_per_1m, output_per_1m) = match model {
            m if m.contains("claude-3-5-sonnet") => (2.73, 13.64), // $3/M input, $15/M output
            m if m.contains("claude-3-5-haiku") => (0.73, 3.64),   // $0.80/M, $4/M
            m if m.contains("claude-3-opus") => (13.64, 68.18),    // $15/M, $75/M
            m if m.contains("claude-3-sonnet") => (2.73, 13.64),   // $3/M, $15/M
            m if m.contains("claude-3-haiku") => (0.23, 1.14),     // $0.25/M, $1.25/M
            _ => (2.73, 13.64), // Default to Sonnet pricing
        };

        let input_cost = (prompt_tokens as f64 / 1_000_000.0) * input_per_1m;
        let output_cost = (completion_tokens as f64 / 1_000_000.0) * output_per_1m;

        input_cost + output_cost
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    async fn available_models(&self) -> Result<Vec<ModelInfo>> {
        // Anthropic doesn't have a models list endpoint, return hardcoded list
        Ok(vec![
            ModelInfo {
                id: "claude-3-5-sonnet-20241022".to_string(),
                name: "Claude 3.5 Sonnet".to_string(),
                context_window: 200_000,
                input_cost_per_1m: 2.73,
                output_cost_per_1m: 13.64,
            },
            ModelInfo {
                id: "claude-3-5-haiku-20241022".to_string(),
                name: "Claude 3.5 Haiku".to_string(),
                context_window: 200_000,
                input_cost_per_1m: 0.73,
                output_cost_per_1m: 3.64,
            },
            ModelInfo {
                id: "claude-3-opus-20240229".to_string(),
                name: "Claude 3 Opus".to_string(),
                context_window: 200_000,
                input_cost_per_1m: 13.64,
                output_cost_per_1m: 68.18,
            },
            ModelInfo {
                id: "claude-3-sonnet-20240229".to_string(),
                name: "Claude 3 Sonnet".to_string(),
                context_window: 200_000,
                input_cost_per_1m: 2.73,
                output_cost_per_1m: 13.64,
            },
            ModelInfo {
                id: "claude-3-haiku-20240307".to_string(),
                name: "Claude 3 Haiku".to_string(),
                context_window: 200_000,
                input_cost_per_1m: 0.23,
                output_cost_per_1m: 1.14,
            },
        ])
    }

    async fn is_available(&self) -> bool {
        !self.api_key.is_empty() && self.api_key.starts_with("sk-ant-")
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let _permit = self.semaphore.acquire().await.map_err(|e| {
            AppError::InternalError(format!("Failed to acquire semaphore: {}", e))
        })?;

        let model = if request.model.is_empty() {
            self.default_model.clone()
        } else {
            request.model.clone()
        };

        // Convert messages to Anthropic format
        let (system, messages) = convert_messages(request.messages)?;

        let body = AnthropicChatRequest {
            model: model.clone(),
            max_tokens: request.max_tokens.unwrap_or(4096),
            messages,
            system,
            temperature: request.temperature,
            stop_sequences: request.stop,
        };

        tracing::debug!(model = %model, "Sending request to Anthropic");

        let response = self
            .client
            .post(format!("{}/messages", ANTHROPIC_API_BASE))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_API_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("Anthropic API request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::InternalError(format!(
                "Anthropic API error ({}): {}",
                status, error_text
            )));
        }

        let anthropic_response: AnthropicChatResponse = response
            .json()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to parse Anthropic response: {}", e)))?;

        // Extract text content
        let content = anthropic_response
            .content
            .into_iter()
            .filter_map(|c| match c {
                ContentBlock::Text { text } => Some(text),
            })
            .collect::<Vec<_>>()
            .join("\n");

        let usage = Usage {
            prompt_tokens: anthropic_response.usage.input_tokens,
            completion_tokens: anthropic_response.usage.output_tokens,
            total_tokens: anthropic_response.usage.input_tokens + anthropic_response.usage.output_tokens,
        };

        let cost_eur = Self::calculate_cost(
            &model,
            anthropic_response.usage.input_tokens,
            anthropic_response.usage.output_tokens,
        );

        let finish_reason = match anthropic_response.stop_reason.as_str() {
            "end_turn" => FinishReason::Stop,
            "max_tokens" => FinishReason::Length,
            "stop_sequence" => FinishReason::Stop,
            _ => FinishReason::Stop,
        };

        tracing::info!(
            model = %model,
            input_tokens = usage.prompt_tokens,
            output_tokens = usage.completion_tokens,
            cost_eur = cost_eur,
            "Anthropic API call completed"
        );

        Ok(ChatResponse {
            content,
            provider: "anthropic".to_string(),
            model,
            usage,
            cost_eur,
            finish_reason,
        })
    }

    async fn generate(&self, prompt: &str, max_tokens: usize, temperature: f32) -> Result<String> {
        let request = ChatRequest {
            model: self.default_model.clone(),
            messages: vec![ProviderMessage {
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

    async fn chat_stream(
        &self,
        _request: ChatRequest,
    ) -> Result<Box<dyn futures::Stream<Item = Result<StreamEvent>> + Send + Unpin>> {
        // Streaming implementation deferred (Phase 6.4 - Real-time collaboration)
        Err(AppError::InternalError(
            "Streaming not yet implemented for Anthropic provider".to_string(),
        ))
    }

    async fn embed(&self, _texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        // Anthropic doesn't provide embeddings API
        Err(AppError::InternalError(
            "Anthropic does not support embeddings (use Ollama nomic-embed-text instead)".to_string(),
        ))
    }
}

/// Convert provider messages to Anthropic format
fn convert_messages(messages: Vec<ProviderMessage>) -> Result<(Option<String>, Vec<AnthropicMessage>)> {
    let mut system = None;
    let mut anthropic_messages = Vec::new();

    for msg in messages {
        match msg.role {
            Role::System => {
                // Anthropic uses separate system field
                system = Some(msg.content);
            }
            Role::User => {
                anthropic_messages.push(AnthropicMessage {
                    role: "user".to_string(),
                    content: msg.content,
                });
            }
            Role::Assistant => {
                anthropic_messages.push(AnthropicMessage {
                    role: "assistant".to_string(),
                    content: msg.content,
                });
            }
        }
    }

    Ok((system, anthropic_messages))
}

// ============================================================================
// Anthropic API types
// ============================================================================

#[derive(Debug, Serialize)]
struct AnthropicChatRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicChatResponse {
    content: Vec<ContentBlock>,
    stop_reason: String,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anthropic_provider_creation() {
        let provider = AnthropicProvider::new(
            "sk-ant-test-key".to_string(),
            4,
            None,
        );
        assert_eq!(provider.name(), "anthropic");
        assert_eq!(provider.default_model, "claude-3-5-sonnet-20241022");
    }

    #[tokio::test]
    async fn test_available_models() {
        let provider = AnthropicProvider::new(
            "sk-ant-test-key".to_string(),
            4,
            None,
        );
        let models = provider.available_models().await.expect("Should list models");
        assert!(models.len() >= 5);
        assert!(models.iter().any(|m| m.id.contains("claude-3-5-sonnet")));
    }

    #[tokio::test]
    async fn test_is_available() {
        let provider = AnthropicProvider::new(
            "sk-ant-test-key".to_string(),
            4,
            None,
        );
        assert!(provider.is_available().await);

        let invalid_provider = AnthropicProvider::new(
            "invalid-key".to_string(),
            4,
            None,
        );
        assert!(!invalid_provider.is_available().await);
    }

    #[test]
    fn test_cost_calculation() {
        // Claude 3.5 Sonnet: 1000 input + 500 output tokens
        let cost = AnthropicProvider::calculate_cost("claude-3-5-sonnet-20241022", 1000, 500);
        assert!((cost - 0.00955).abs() < 0.0001); // (1000/1M * 2.73) + (500/1M * 13.64)

        // Claude 3.5 Haiku (cheaper)
        let cost_haiku = AnthropicProvider::calculate_cost("claude-3-5-haiku-20241022", 1000, 500);
        assert!(cost_haiku < cost);
    }

    #[test]
    fn test_convert_messages() {
        let messages = vec![
            ProviderMessage {
                role: Role::System,
                content: "You are helpful".to_string(),
            },
            ProviderMessage {
                role: Role::User,
                content: "Hello".to_string(),
            },
            ProviderMessage {
                role: Role::Assistant,
                content: "Hi there!".to_string(),
            },
        ];

        let (system, anthropic_messages) = convert_messages(messages).expect("Conversion should succeed");

        assert_eq!(system, Some("You are helpful".to_string()));
        assert_eq!(anthropic_messages.len(), 2);
        assert_eq!(anthropic_messages[0].role, "user");
        assert_eq!(anthropic_messages[1].role, "assistant");
    }
}
