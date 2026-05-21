/// Google Gemini provider implementation
///
/// Supports Gemini 1.5 Pro, Gemini 1.5 Flash via Generative Language API.
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

/// Google AI API endpoint
const GOOGLE_API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta";

/// Google provider
pub struct GoogleProvider {
    api_key: String,
    client: Client,
    semaphore: Arc<Semaphore>,
    default_model: String,
}

impl GoogleProvider {
    pub fn new(api_key: String, max_concurrent: usize, default_model: Option<String>) -> Self {
        Self {
            api_key,
            client: Client::new(),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            default_model: default_model.unwrap_or_else(|| "gemini-1.5-pro".to_string()),
        }
    }

    fn calculate_cost(model: &str, prompt_tokens: u32, completion_tokens: u32) -> f64 {
        // Pricing as of Jan 2025 (EUR, converted from USD at ~1.1 rate)
        let (input_per_1m, output_per_1m) = match model {
            m if m.contains("gemini-1.5-pro") => (3.18, 12.73), // $3.50/M, $14/M (128k+ context)
            m if m.contains("gemini-1.5-flash") => (0.27, 1.09), // $0.30/M, $1.20/M
            m if m.contains("gemini-pro") => (0.45, 1.36), // $0.50/M, $1.50/M
            _ => (3.18, 12.73), // Default to Pro pricing
        };

        let input_cost = (prompt_tokens as f64 / 1_000_000.0) * input_per_1m;
        let output_cost = (completion_tokens as f64 / 1_000_000.0) * output_per_1m;

        input_cost + output_cost
    }
}

#[async_trait]
impl LlmProvider for GoogleProvider {
    fn name(&self) -> &str {
        "google"
    }

    async fn available_models(&self) -> Result<Vec<ModelInfo>> {
        Ok(vec![
            ModelInfo {
                id: "gemini-1.5-pro".to_string(),
                name: "Gemini 1.5 Pro".to_string(),
                context_window: 1_048_576, // 1M tokens
                input_cost_per_1m: 3.18,
                output_cost_per_1m: 12.73,
            },
            ModelInfo {
                id: "gemini-1.5-flash".to_string(),
                name: "Gemini 1.5 Flash".to_string(),
                context_window: 1_048_576,
                input_cost_per_1m: 0.27,
                output_cost_per_1m: 1.09,
            },
            ModelInfo {
                id: "gemini-pro".to_string(),
                name: "Gemini Pro".to_string(),
                context_window: 30_720,
                input_cost_per_1m: 0.45,
                output_cost_per_1m: 1.36,
            },
        ])
    }

    async fn is_available(&self) -> bool {
        !self.api_key.is_empty() && self.api_key.starts_with("AIzaSy")
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

        let contents = convert_messages(request.messages)?;

        let mut body = serde_json::json!({
            "contents": contents,
        });

        if let Some(temp) = request.temperature {
            body["generationConfig"] = serde_json::json!({
                "temperature": temp,
            });
        }
        if let Some(max_tokens) = request.max_tokens {
            if body["generationConfig"].is_null() {
                body["generationConfig"] = serde_json::json!({});
            }
            body["generationConfig"]["maxOutputTokens"] = serde_json::json!(max_tokens);
        }

        tracing::debug!(model = %model, "Sending request to Google Gemini");

        let url = format!(
            "{}/models/{}:generateContent?key={}",
            GOOGLE_API_BASE, model, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("Google API request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::InternalError(format!(
                "Google API error ({}): {}",
                status, error_text
            )));
        }

        let google_response: GoogleChatResponse = response
            .json()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to parse Google response: {}", e)))?;

        let candidate = google_response
            .candidates
            .first()
            .ok_or_else(|| AppError::InternalError("No candidates in Google response".to_string()))?;

        let content = candidate
            .content
            .parts
            .iter()
            .map(|p| p.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let usage = Usage {
            prompt_tokens: google_response.usage_metadata.prompt_token_count,
            completion_tokens: google_response.usage_metadata.candidates_token_count,
            total_tokens: google_response.usage_metadata.total_token_count,
        };

        let cost_eur = Self::calculate_cost(
            &model,
            google_response.usage_metadata.prompt_token_count,
            google_response.usage_metadata.candidates_token_count,
        );

        let finish_reason = match candidate.finish_reason.as_str() {
            "STOP" => FinishReason::Stop,
            "MAX_TOKENS" => FinishReason::Length,
            "SAFETY" => FinishReason::ContentFilter,
            _ => FinishReason::Stop,
        };

        tracing::info!(
            model = %model,
            input_tokens = usage.prompt_tokens,
            output_tokens = usage.completion_tokens,
            cost_eur = cost_eur,
            "Google API call completed"
        );

        Ok(ChatResponse {
            content,
            provider: "google".to_string(),
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
            "Streaming not yet implemented for Google provider".to_string(),
        ))
    }

    async fn embed(&self, _texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        // Google has embedding models but different API endpoint
        Err(AppError::InternalError(
            "Google embeddings not yet implemented (use Ollama nomic-embed-text instead)".to_string(),
        ))
    }
}

fn convert_messages(messages: Vec<ProviderMessage>) -> Result<Vec<serde_json::Value>> {
    let mut contents = Vec::new();
    let mut system_instruction = None;

    for msg in messages {
        match msg.role {
            Role::System => {
                system_instruction = Some(msg.content);
            }
            Role::User => {
                contents.push(serde_json::json!({
                    "role": "user",
                    "parts": [{"text": msg.content}],
                }));
            }
            Role::Assistant => {
                contents.push(serde_json::json!({
                    "role": "model",
                    "parts": [{"text": msg.content}],
                }));
            }
        }
    }

    // Gemini requires system instruction in separate field (not implemented here for simplicity)
    if system_instruction.is_some() {
        tracing::warn!("System messages not fully supported in Google provider yet");
    }

    Ok(contents)
}

// ============================================================================
// Google API types
// ============================================================================

#[derive(Debug, Deserialize)]
struct GoogleChatResponse {
    candidates: Vec<GoogleCandidate>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: GoogleUsageMetadata,
}

#[derive(Debug, Deserialize)]
struct GoogleCandidate {
    content: GoogleContent,
    #[serde(rename = "finishReason")]
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct GoogleContent {
    parts: Vec<GooglePart>,
}

#[derive(Debug, Deserialize)]
struct GooglePart {
    text: String,
}

#[derive(Debug, Deserialize)]
struct GoogleUsageMetadata {
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: u32,
    #[serde(rename = "candidatesTokenCount")]
    candidates_token_count: u32,
    #[serde(rename = "totalTokenCount")]
    total_token_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_provider_creation() {
        let provider = GoogleProvider::new("AIzaSyTest".to_string(), 4, None);
        assert_eq!(provider.name(), "google");
        assert_eq!(provider.default_model, "gemini-1.5-pro");
    }

    #[tokio::test]
    async fn test_available_models() {
        let provider = GoogleProvider::new("AIzaSyTest".to_string(), 4, None);
        let models = provider.available_models().await.expect("Should list models");
        assert_eq!(models.len(), 3);
        assert!(models.iter().any(|m| m.id.contains("gemini-1.5-pro")));
    }

    #[tokio::test]
    async fn test_is_available() {
        let provider = GoogleProvider::new("AIzaSyTest".to_string(), 4, None);
        assert!(provider.is_available().await);

        let invalid_provider = GoogleProvider::new("invalid-key".to_string(), 4, None);
        assert!(!invalid_provider.is_available().await);
    }

    #[test]
    fn test_cost_calculation() {
        let cost = GoogleProvider::calculate_cost("gemini-1.5-pro", 1000, 500);
        assert!((cost - 0.00954).abs() < 0.0001);

        let cost_flash = GoogleProvider::calculate_cost("gemini-1.5-flash", 1000, 500);
        assert!(cost_flash < cost);
    }
}
