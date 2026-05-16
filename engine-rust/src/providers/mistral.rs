/// Mistral AI provider implementation (OpenAI-compatible API)

use async_trait::async_trait;
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::errors::{AppError, Result};
use super::{LlmProvider, ChatRequest, ChatResponse, Message as ProviderMessage, Role, Usage, FinishReason, ModelInfo, StreamEvent};

const MISTRAL_API_BASE: &str = "https://api.mistral.ai/v1";

pub struct MistralProvider {
    api_key: String,
    client: Client,
    semaphore: Arc<Semaphore>,
    default_model: String,
}

impl MistralProvider {
    pub fn new(api_key: String, max_concurrent: usize, default_model: Option<String>) -> Self {
        Self {
            api_key,
            client: Client::new(),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            default_model: default_model.unwrap_or_else(|| "mistral-large-latest".to_string()),
        }
    }

    fn calculate_cost(model: &str, prompt_tokens: u32, completion_tokens: u32) -> f64 {
        let (input_per_1m, output_per_1m) = match model {
            m if m.contains("large") => (3.64, 10.91), // $4/M, $12/M
            m if m.contains("medium") => (2.45, 7.27), // $2.7/M, $8/M
            m if m.contains("small") => (0.91, 2.73), // $1/M, $3/M
            _ => (3.64, 10.91),
        };
        let input_cost = (prompt_tokens as f64 / 1_000_000.0) * input_per_1m;
        let output_cost = (completion_tokens as f64 / 1_000_000.0) * output_per_1m;
        input_cost + output_cost
    }
}

#[async_trait]
impl LlmProvider for MistralProvider {
    fn name(&self) -> &str {
        "mistral"
    }

    async fn available_models(&self) -> Result<Vec<ModelInfo>> {
        Ok(vec![
            ModelInfo { id: "mistral-large-latest".to_string(), name: "Mistral Large".to_string(), context_window: 128_000, input_cost_per_1m: 3.64, output_cost_per_1m: 10.91 },
            ModelInfo { id: "mistral-medium-latest".to_string(), name: "Mistral Medium".to_string(), context_window: 32_000, input_cost_per_1m: 2.45, output_cost_per_1m: 7.27 },
            ModelInfo { id: "mistral-small-latest".to_string(), name: "Mistral Small".to_string(), context_window: 32_000, input_cost_per_1m: 0.91, output_cost_per_1m: 2.73 },
        ])
    }

    async fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let _permit = self.semaphore.acquire().await.map_err(|e| AppError::InternalError(format!("Semaphore: {}", e)))?;
        let model = if request.model.is_empty() { self.default_model.clone() } else { request.model.clone() };
        let messages = convert_messages(request.messages);

        let mut body = serde_json::json!({"model": model, "messages": messages});
        if let Some(temp) = request.temperature { body["temperature"] = serde_json::json!(temp); }
        if let Some(max_tokens) = request.max_tokens { body["max_tokens"] = serde_json::json!(max_tokens); }

        let response = self.client.post(format!("{}/chat/completions", MISTRAL_API_BASE))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("Mistral API failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::InternalError(format!("Mistral error ({})", response.status())));
        }

        let resp: serde_json::Value = response.json().await.map_err(|e| AppError::InternalError(format!("Parse: {}", e)))?;
        let content = resp["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
        let usage = Usage {
            prompt_tokens: resp["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: resp["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: resp["usage"]["total_tokens"].as_u64().unwrap_or(0) as u32,
        };
        let cost_eur = Self::calculate_cost(&model, usage.prompt_tokens, usage.completion_tokens);

        Ok(ChatResponse { content, provider: "mistral".to_string(), model, usage, cost_eur, finish_reason: FinishReason::Stop })
    }

    async fn generate(&self, prompt: &str, max_tokens: usize, temperature: f32) -> Result<String> {
        let request = ChatRequest { model: self.default_model.clone(), messages: vec![ProviderMessage { role: Role::User, content: prompt.to_string() }], response_format: None, temperature: Some(temperature), max_tokens: Some(max_tokens as u32), stop: None };
        let response = self.chat(request).await?;
        Ok(response.content)
    }

    async fn chat_stream(&self, _request: ChatRequest) -> Result<Box<dyn futures::Stream<Item = Result<StreamEvent>> + Send + Unpin>> {
        Err(AppError::InternalError("Streaming not implemented".to_string()))
    }

    async fn embed(&self, _texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        Err(AppError::InternalError("Mistral embeddings not supported".to_string()))
    }
}

fn convert_messages(messages: Vec<ProviderMessage>) -> Vec<serde_json::Value> {
    messages.into_iter().map(|msg| {
        let role = match msg.role { Role::System => "system", Role::User => "user", Role::Assistant => "assistant" };
        serde_json::json!({"role": role, "content": msg.content})
    }).collect()
}
