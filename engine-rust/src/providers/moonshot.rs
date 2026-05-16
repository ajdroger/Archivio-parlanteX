/// Moonshot AI provider implementation (Chinese LLM, OpenAI-compatible)

use async_trait::async_trait;
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::errors::{AppError, Result};
use super::{LlmProvider, ChatRequest, ChatResponse, Message as ProviderMessage, Role, Usage, FinishReason, ModelInfo, StreamEvent};

const MOONSHOT_API_BASE: &str = "https://api.moonshot.cn/v1";

pub struct MoonshotProvider {
    api_key: String,
    client: Client,
    semaphore: Arc<Semaphore>,
    default_model: String,
}

impl MoonshotProvider {
    pub fn new(api_key: String, max_concurrent: usize, default_model: Option<String>) -> Self {
        Self { api_key, client: Client::new(), semaphore: Arc::new(Semaphore::new(max_concurrent)), default_model: default_model.unwrap_or_else(|| "moonshot-v1-8k".to_string()) }
    }

    fn calculate_cost(_model: &str, prompt_tokens: u32, completion_tokens: u32) -> f64 {
        let input_cost = (prompt_tokens as f64 / 1_000_000.0) * 0.11; // ~$0.12/M
        let output_cost = (completion_tokens as f64 / 1_000_000.0) * 0.11;
        input_cost + output_cost
    }
}

#[async_trait]
impl LlmProvider for MoonshotProvider {
    fn name(&self) -> &str { "moonshot" }

    async fn available_models(&self) -> Result<Vec<ModelInfo>> {
        Ok(vec![
            ModelInfo { id: "moonshot-v1-8k".to_string(), name: "Moonshot 8K".to_string(), context_window: 8_000, input_cost_per_1m: 0.11, output_cost_per_1m: 0.11 },
            ModelInfo { id: "moonshot-v1-32k".to_string(), name: "Moonshot 32K".to_string(), context_window: 32_000, input_cost_per_1m: 0.23, output_cost_per_1m: 0.23 },
        ])
    }

    async fn is_available(&self) -> bool { !self.api_key.is_empty() }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let _permit = self.semaphore.acquire().await.map_err(|e| AppError::InternalError(format!("Semaphore: {}", e)))?;
        let model = if request.model.is_empty() { self.default_model.clone() } else { request.model.clone() };
        let messages = convert_messages(request.messages);

        let mut body = serde_json::json!({"model": model, "messages": messages});
        if let Some(temp) = request.temperature { body["temperature"] = serde_json::json!(temp); }
        if let Some(max_tokens) = request.max_tokens { body["max_tokens"] = serde_json::json!(max_tokens); }

        let response = self.client.post(format!("{}/chat/completions", MOONSHOT_API_BASE)).header("Authorization", format!("Bearer {}", self.api_key)).json(&body).send().await.map_err(|e| AppError::InternalError(format!("Moonshot API failed: {}", e)))?;

        if !response.status().is_success() { return Err(AppError::InternalError(format!("Moonshot error ({})", response.status()))); }

        let resp: serde_json::Value = response.json().await.map_err(|e| AppError::InternalError(format!("Parse: {}", e)))?;
        let content = resp["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
        let usage = Usage { prompt_tokens: resp["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32, completion_tokens: resp["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32, total_tokens: resp["usage"]["total_tokens"].as_u64().unwrap_or(0) as u32 };
        let cost_eur = Self::calculate_cost(&model, usage.prompt_tokens, usage.completion_tokens);

        Ok(ChatResponse { content, provider: "moonshot".to_string(), model, usage, cost_eur, finish_reason: FinishReason::Stop })
    }

    async fn generate(&self, prompt: &str, max_tokens: usize, temperature: f32) -> Result<String> {
        let request = ChatRequest { model: self.default_model.clone(), messages: vec![ProviderMessage { role: Role::User, content: prompt.to_string() }], response_format: None, temperature: Some(temperature), max_tokens: Some(max_tokens as u32), stop: None };
        let response = self.chat(request).await?;
        Ok(response.content)
    }

    async fn chat_stream(&self, _request: ChatRequest) -> Result<Box<dyn futures::Stream<Item = Result<StreamEvent>> + Send + Unpin>> { Err(AppError::InternalError("Streaming not implemented".to_string())) }

    async fn embed(&self, _texts: Vec<String>) -> Result<Vec<Vec<f32>>> { Err(AppError::InternalError("Moonshot embeddings not supported".to_string())) }
}

fn convert_messages(messages: Vec<ProviderMessage>) -> Vec<serde_json::Value> {
    messages.into_iter().map(|msg| {
        let role = match msg.role { Role::System => "system", Role::User => "user", Role::Assistant => "assistant" };
        serde_json::json!({"role": role, "content": msg.content})
    }).collect()
}
