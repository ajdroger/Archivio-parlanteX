/// OpenAI provider implementation
///
/// Supports GPT-4o, GPT-4-turbo, GPT-3.5-turbo, o1 models via Chat Completions API.
/// Cost tracking enabled for daily budget enforcement.

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::errors::{AppError, Result};
use super::{
    LlmProvider, ChatRequest, ChatResponse, Message as ProviderMessage, Role, Usage,
    FinishReason, ModelInfo, StreamEvent,
};

/// OpenAI API endpoint
const OPENAI_API_BASE: &str = "https://api.openai.com/v1";

/// OpenAI provider
pub struct OpenAIProvider {
    api_key: String,
    client: Client,
    semaphore: Arc<Semaphore>,
    default_model: String,
}

impl OpenAIProvider {
    /// Create new OpenAI provider
    pub fn new(api_key: String, max_concurrent: usize, default_model: Option<String>) -> Self {
        Self {
            api_key,
            client: Client::new(),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            default_model: default_model.unwrap_or_else(|| "gpt-4o-2024-11-20".to_string()),
        }
    }

    /// Calculate cost in EUR for OpenAI API call
    fn calculate_cost(model: &str, prompt_tokens: u32, completion_tokens: u32) -> f64 {
        // Pricing as of Jan 2025 (EUR, converted from USD at ~1.1 rate)
        let (input_per_1m, output_per_1m) = match model {
            m if m.starts_with("gpt-4o-2024-11-20") || m == "gpt-4o" => (2.27, 9.09), // $2.50/M, $10/M
            m if m.starts_with("gpt-4o-mini") => (0.14, 0.55), // $0.150/M, $0.600/M
            m if m.starts_with("gpt-4-turbo") || m.starts_with("gpt-4-0125") => (9.09, 27.27), // $10/M, $30/M
            m if m.starts_with("gpt-4-32k") => (54.55, 109.09), // $60/M, $120/M
            m if m.starts_with("gpt-4") => (27.27, 54.55), // $30/M, $60/M
            m if m.starts_with("gpt-3.5-turbo") => (0.45, 1.36), // $0.50/M, $1.50/M
            m if m.starts_with("o1-preview") => (13.64, 54.55), // $15/M, $60/M
            m if m.starts_with("o1-mini") => (2.73, 10.91), // $3/M, $12/M
            m if m.starts_with("o1") => (13.64, 54.55), // $15/M, $60/M
            _ => (2.27, 9.09), // Default to GPT-4o pricing
        };

        let input_cost = (prompt_tokens as f64 / 1_000_000.0) * input_per_1m;
        let output_cost = (completion_tokens as f64 / 1_000_000.0) * output_per_1m;

        input_cost + output_cost
    }
}

#[async_trait]
impl LlmProvider for OpenAIProvider {
    fn name(&self) -> &str {
        "openai"
    }

    async fn available_models(&self) -> Result<Vec<ModelInfo>> {
        Ok(vec![
            ModelInfo {
                id: "gpt-4o-2024-11-20".to_string(),
                name: "GPT-4o".to_string(),
                context_window: 128_000,
                input_cost_per_1m: 2.27,
                output_cost_per_1m: 9.09,
            },
            ModelInfo {
                id: "gpt-4o-mini".to_string(),
                name: "GPT-4o mini".to_string(),
                context_window: 128_000,
                input_cost_per_1m: 0.14,
                output_cost_per_1m: 0.55,
            },
            ModelInfo {
                id: "gpt-4-turbo".to_string(),
                name: "GPT-4 Turbo".to_string(),
                context_window: 128_000,
                input_cost_per_1m: 9.09,
                output_cost_per_1m: 27.27,
            },
            ModelInfo {
                id: "gpt-4".to_string(),
                name: "GPT-4".to_string(),
                context_window: 8_192,
                input_cost_per_1m: 27.27,
                output_cost_per_1m: 54.55,
            },
            ModelInfo {
                id: "gpt-3.5-turbo".to_string(),
                name: "GPT-3.5 Turbo".to_string(),
                context_window: 16_385,
                input_cost_per_1m: 0.45,
                output_cost_per_1m: 1.36,
            },
            ModelInfo {
                id: "o1-preview".to_string(),
                name: "o1 Preview".to_string(),
                context_window: 128_000,
                input_cost_per_1m: 13.64,
                output_cost_per_1m: 54.55,
            },
            ModelInfo {
                id: "o1-mini".to_string(),
                name: "o1 Mini".to_string(),
                context_window: 128_000,
                input_cost_per_1m: 2.73,
                output_cost_per_1m: 10.91,
            },
        ])
    }

    async fn is_available(&self) -> bool {
        !self.api_key.is_empty() && self.api_key.starts_with("sk-")
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

        let messages = convert_messages(request.messages);

        let mut body = serde_json::json!({
            "model": model,
            "messages": messages,
        });

        if let Some(temp) = request.temperature {
            body["temperature"] = serde_json::json!(temp);
        }
        if let Some(max_tokens) = request.max_tokens {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }
        if let Some(stop) = request.stop {
            body["stop"] = serde_json::json!(stop);
        }
        if let Some(response_format) = request.response_format {
            if response_format.format_type == "json_object" {
                body["response_format"] = serde_json::json!({"type": "json_object"});
            }
        }

        tracing::debug!(model = %model, "Sending request to OpenAI");

        let response = self
            .client
            .post(format!("{}/chat/completions", OPENAI_API_BASE))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("OpenAI API request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::InternalError(format!(
                "OpenAI API error ({}): {}",
                status, error_text
            )));
        }

        let openai_response: OpenAIChatResponse = response
            .json()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to parse OpenAI response: {}", e)))?;

        let choice = openai_response
            .choices
            .first()
            .ok_or_else(|| AppError::InternalError("No choices in OpenAI response".to_string()))?;

        let content = choice.message.content.clone();

        let usage = Usage {
            prompt_tokens: openai_response.usage.prompt_tokens,
            completion_tokens: openai_response.usage.completion_tokens,
            total_tokens: openai_response.usage.total_tokens,
        };

        let cost_eur = Self::calculate_cost(
            &model,
            openai_response.usage.prompt_tokens,
            openai_response.usage.completion_tokens,
        );

        let finish_reason = match choice.finish_reason.as_str() {
            "stop" => FinishReason::Stop,
            "length" => FinishReason::Length,
            "content_filter" => FinishReason::ContentFilter,
            "tool_calls" => FinishReason::ToolCalls,
            _ => FinishReason::Stop,
        };

        tracing::info!(
            model = %model,
            input_tokens = usage.prompt_tokens,
            output_tokens = usage.completion_tokens,
            cost_eur = cost_eur,
            "OpenAI API call completed"
        );

        Ok(ChatResponse {
            content,
            provider: "openai".to_string(),
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
        Err(AppError::InternalError(
            "Streaming not yet implemented for OpenAI provider".to_string(),
        ))
    }

    async fn embed(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let body = serde_json::json!({
            "model": "text-embedding-3-small",
            "input": texts,
        });

        let response = self
            .client
            .post(format!("{}/embeddings", OPENAI_API_BASE))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("OpenAI embeddings request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::InternalError(format!("OpenAI embeddings error ({}): {}", status, error_text)));
        }

        let embedding_response: OpenAIEmbeddingResponse = response
            .json()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to parse OpenAI embedding response: {}", e)))?;

        let embeddings = embedding_response
            .data
            .into_iter()
            .map(|item| item.embedding)
            .collect();

        Ok(embeddings)
    }
}

fn convert_messages(messages: Vec<ProviderMessage>) -> Vec<serde_json::Value> {
    messages
        .into_iter()
        .map(|msg| {
            let role = match msg.role {
                Role::System => "system",
                Role::User => "user",
                Role::Assistant => "assistant",
            };
            serde_json::json!({
                "role": role,
                "content": msg.content,
            })
        })
        .collect()
}

// ============================================================================
// OpenAI API types
// ============================================================================

#[derive(Debug, Deserialize)]
struct OpenAIChatResponse {
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct OpenAIEmbeddingResponse {
    data: Vec<OpenAIEmbeddingData>,
}

#[derive(Debug, Deserialize)]
struct OpenAIEmbeddingData {
    embedding: Vec<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_provider_creation() {
        let provider = OpenAIProvider::new("sk-test-key".to_string(), 4, None);
        assert_eq!(provider.name(), "openai");
        assert_eq!(provider.default_model, "gpt-4o-2024-11-20");
    }

    #[tokio::test]
    async fn test_available_models() {
        let provider = OpenAIProvider::new("sk-test-key".to_string(), 4, None);
        let models = provider.available_models().await.expect("Should list models");
        assert!(models.len() >= 6);
        assert!(models.iter().any(|m| m.id.contains("gpt-4o")));
    }

    #[tokio::test]
    async fn test_is_available() {
        let provider = OpenAIProvider::new("sk-test-key".to_string(), 4, None);
        assert!(provider.is_available().await);

        let invalid_provider = OpenAIProvider::new("invalid-key".to_string(), 4, None);
        assert!(!invalid_provider.is_available().await);
    }

    #[test]
    fn test_cost_calculation() {
        let cost = OpenAIProvider::calculate_cost("gpt-4o", 1000, 500);
        assert!((cost - 0.006815).abs() < 0.0001);

        let cost_mini = OpenAIProvider::calculate_cost("gpt-4o-mini", 1000, 500);
        assert!(cost_mini < cost);
    }
}
