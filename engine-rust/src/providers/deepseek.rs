/// DeepSeek provider implementation (OpenAI-compatible API)
///
/// Supports DeepSeek-V3, DeepSeek-Coder models.

use async_trait::async_trait;
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::errors::{AppError, Result};
use super::{
    LlmProvider, ChatRequest, ChatResponse, Message as ProviderMessage, Role, Usage,
    FinishReason, ModelInfo, StreamEvent,
};

const DEEPSEEK_API_BASE: &str = "https://api.deepseek.com/v1";

pub struct DeepSeekProvider {
    api_key: String,
    client: Client,
    semaphore: Arc<Semaphore>,
    default_model: String,
}

impl DeepSeekProvider {
    pub fn new(api_key: String, max_concurrent: usize, default_model: Option<String>) -> Self {
        Self {
            api_key,
            client: Client::new(),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            default_model: default_model.unwrap_or_else(|| "deepseek-chat".to_string()),
        }
    }

    fn calculate_cost(_model: &str, prompt_tokens: u32, completion_tokens: u32) -> f64 {
        // DeepSeek V3 pricing (Jan 2025, EUR): $0.27/M input, $1.10/M output
        let input_cost = (prompt_tokens as f64 / 1_000_000.0) * 0.27;
        let output_cost = (completion_tokens as f64 / 1_000_000.0) * 1.10;
        input_cost + output_cost
    }
}

#[async_trait]
impl LlmProvider for DeepSeekProvider {
    fn name(&self) -> &str {
        "deepseek"
    }

    async fn available_models(&self) -> Result<Vec<ModelInfo>> {
        Ok(vec![
            ModelInfo {
                id: "deepseek-chat".to_string(),
                name: "DeepSeek V3".to_string(),
                context_window: 64_000,
                input_cost_per_1m: 0.27,
                output_cost_per_1m: 1.10,
            },
            ModelInfo {
                id: "deepseek-coder".to_string(),
                name: "DeepSeek Coder".to_string(),
                context_window: 16_000,
                input_cost_per_1m: 0.27,
                output_cost_per_1m: 1.10,
            },
        ])
    }

    async fn is_available(&self) -> bool {
        !self.api_key.is_empty()
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

        let response = self
            .client
            .post(format!("{}/chat/completions", DEEPSEEK_API_BASE))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("DeepSeek API request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::InternalError(format!("DeepSeek API error ({}): {}", status, error_text)));
        }

        let openai_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to parse DeepSeek response: {}", e)))?;

        let content = openai_response["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| AppError::InternalError("No content in DeepSeek response".to_string()))?
            .to_string();

        let usage = Usage {
            prompt_tokens: openai_response["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: openai_response["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: openai_response["usage"]["total_tokens"].as_u64().unwrap_or(0) as u32,
        };

        let cost_eur = Self::calculate_cost(&model, usage.prompt_tokens, usage.completion_tokens);

        let finish_reason = match openai_response["choices"][0]["finish_reason"].as_str() {
            Some("stop") => FinishReason::Stop,
            Some("length") => FinishReason::Length,
            _ => FinishReason::Stop,
        };

        Ok(ChatResponse {
            content,
            provider: "deepseek".to_string(),
            model,
            usage,
            cost_eur,
            finish_reason,
        })
    }

    async fn generate(&self, prompt: &str, max_tokens: usize, temperature: f32) -> Result<String> {
        let request = ChatRequest {
            model: self.default_model.clone(),
            messages: vec![ProviderMessage { role: Role::User, content: prompt.to_string() }],
            response_format: None,
            temperature: Some(temperature),
            max_tokens: Some(max_tokens as u32),
            stop: None,
        };
        let response = self.chat(request).await?;
        Ok(response.content)
    }

    async fn chat_stream(&self, _request: ChatRequest) -> Result<Box<dyn futures::Stream<Item = Result<StreamEvent>> + Send + Unpin>> {
        Err(AppError::InternalError("Streaming not yet implemented for DeepSeek provider".to_string()))
    }

    async fn embed(&self, _texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        Err(AppError::InternalError("DeepSeek does not support embeddings".to_string()))
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
            serde_json::json!({"role": role, "content": msg.content})
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deepseek_provider_creation() {
        let provider = DeepSeekProvider::new("test-key".to_string(), 4, None);
        assert_eq!(provider.name(), "deepseek");
        assert_eq!(provider.default_model, "deepseek-chat");
    }

    #[tokio::test]
    async fn test_available_models() {
        let provider = DeepSeekProvider::new("test-key".to_string(), 4, None);
        let models = provider.available_models().await.unwrap();
        assert_eq!(models.len(), 2);
    }
}
