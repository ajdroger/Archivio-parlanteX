/// Ollama local LLM provider implementation

use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::errors::{AppError, Result};
use super::{LlmProvider, ChatRequest, ChatResponse, StreamEvent, ModelInfo, Usage, FinishReason, Message};

/// Ollama provider (local, zero-cost)
pub struct OllamaProvider {
    base_url: String,
    client: Client,
    /// Rate limiter (max concurrent requests)
    semaphore: Arc<Semaphore>,
    /// Embedding model name (from config)
    embed_model: String,
}

impl OllamaProvider {
    /// Create new Ollama provider
    ///
    /// # Arguments
    /// * `base_url` - Ollama API base URL (e.g., http://ollama:11434)
    /// * `max_concurrent` - Max concurrent requests (from config)
    /// * `embed_model` - Embedding model name (e.g., nomic-embed-text)
    pub fn new(base_url: String, max_concurrent: usize, embed_model: String) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: Client::new(),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            embed_model,
        }
    }

    /// Build full API URL
    fn api_url(&self, path: &str) -> String {
        format!("{}/api/{}", self.base_url, path.trim_start_matches('/'))
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    async fn available_models(&self) -> Result<Vec<ModelInfo>> {
        let url = self.api_url("tags");

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Ollama(format!("Failed to fetch models: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::Ollama(format!(
                "Ollama API error: {}",
                response.status()
            )));
        }

        let body: OllamaTagsResponse = response
            .json()
            .await
            .map_err(|e| AppError::Ollama(format!("Failed to parse models: {}", e)))?;

        Ok(body
            .models
            .into_iter()
            .map(|m| ModelInfo {
                id: m.name.clone(),
                name: m.name,
                context_window: 32768, // Qwen2.5 default, adjust per model if needed
                input_cost_per_1m: 0.0,
                output_cost_per_1m: 0.0,
            })
            .collect())
    }

    async fn is_available(&self) -> bool {
        let url = self.api_url("tags");
        self.client.get(&url).send().await.is_ok()
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let _permit = self.semaphore.acquire().await.map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Semaphore error: {}", e))
        })?;

        let url = self.api_url("chat");

        let ollama_req = OllamaChatRequest {
            model: request.model.clone(),
            messages: request.messages.iter().map(|m| OllamaMessage {
                role: match m.role {
                    super::types::Role::System => "system".to_string(),
                    super::types::Role::User => "user".to_string(),
                    super::types::Role::Assistant => "assistant".to_string(),
                },
                content: m.content.clone(),
            }).collect(),
            stream: false,
            options: Some(OllamaOptions {
                temperature: request.temperature,
                num_predict: request.max_tokens.map(|t| t as i32),
                stop: request.stop.clone(),
            }),
        };

        tracing::debug!(
            model = %request.model,
            messages_count = request.messages.len(),
            "Sending Ollama chat request"
        );

        let response = self
            .client
            .post(&url)
            .json(&ollama_req)
            .send()
            .await
            .map_err(|e| AppError::Ollama(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Ollama(format!(
                "Ollama API error {}: {}",
                status, body
            )));
        }

        let ollama_resp: OllamaChatResponse = response
            .json()
            .await
            .map_err(|e| AppError::Ollama(format!("Failed to parse response: {}", e)))?;

        let usage = Usage {
            prompt_tokens: ollama_resp.prompt_eval_count.unwrap_or(0) as u32,
            completion_tokens: ollama_resp.eval_count.unwrap_or(0) as u32,
            total_tokens: (ollama_resp.prompt_eval_count.unwrap_or(0)
                + ollama_resp.eval_count.unwrap_or(0)) as u32,
        };

        tracing::info!(
            model = %request.model,
            prompt_tokens = usage.prompt_tokens,
            completion_tokens = usage.completion_tokens,
            "Ollama chat completed"
        );

        Ok(ChatResponse {
            content: ollama_resp.message.content,
            provider: "ollama".to_string(),
            model: request.model,
            usage,
            cost_eur: 0.0, // Ollama is free
            finish_reason: if ollama_resp.done {
                FinishReason::Stop
            } else {
                FinishReason::Length
            },
        })
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<Box<dyn Stream<Item = Result<StreamEvent>> + Send + Unpin>> {
        let _permit = self.semaphore.acquire().await.map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Semaphore error: {}", e))
        })?;

        let url = self.api_url("chat");

        let ollama_req = OllamaChatRequest {
            model: request.model.clone(),
            messages: request.messages.iter().map(|m| OllamaMessage {
                role: match m.role {
                    super::types::Role::System => "system".to_string(),
                    super::types::Role::User => "user".to_string(),
                    super::types::Role::Assistant => "assistant".to_string(),
                },
                content: m.content.clone(),
            }).collect(),
            stream: true,
            options: Some(OllamaOptions {
                temperature: request.temperature,
                num_predict: request.max_tokens.map(|t| t as i32),
                stop: request.stop.clone(),
            }),
        };

        let response = self
            .client
            .post(&url)
            .json(&ollama_req)
            .send()
            .await
            .map_err(|e| AppError::Ollama(format!("Stream request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(AppError::Ollama(format!("Ollama stream error: {}", status)));
        }

        let stream = response.bytes_stream().map(|chunk_result| {
            let chunk = chunk_result.map_err(|e| {
                AppError::Ollama(format!("Stream chunk error: {}", e))
            })?;

            let line = String::from_utf8_lossy(&chunk);
            for json_line in line.lines() {
                if json_line.trim().is_empty() {
                    continue;
                }

                let event: OllamaChatResponse = serde_json::from_str(json_line).map_err(|e| {
                    AppError::Ollama(format!("Failed to parse stream event: {}", e))
                })?;

                if event.done {
                    return Ok(StreamEvent::Done {
                        usage: Usage {
                            prompt_tokens: event.prompt_eval_count.unwrap_or(0) as u32,
                            completion_tokens: event.eval_count.unwrap_or(0) as u32,
                            total_tokens: (event.prompt_eval_count.unwrap_or(0)
                                + event.eval_count.unwrap_or(0)) as u32,
                        },
                        cost_eur: 0.0,
                    });
                } else {
                    return Ok(StreamEvent::Delta {
                        content: event.message.content,
                    });
                }
            }

            Err(AppError::Ollama("Empty stream chunk".to_string()))
        });

        Ok(Box::new(Box::pin(stream)))
    }

    async fn embed(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        let _permit = self.semaphore.acquire().await.map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Semaphore error: {}", e))
        })?;

        let url = self.api_url("embeddings");

        // Ollama only supports one text per request, batch manually
        let texts_count = texts.len();
        let mut embeddings = Vec::with_capacity(texts_count);

        for text in texts {
            let req = OllamaEmbedRequest {
                model: self.embed_model.clone(),
                prompt: text.clone(),
            };

            let response = self
                .client
                .post(&url)
                .json(&req)
                .send()
                .await
                .map_err(|e| AppError::Ollama(format!("Embed request failed: {}", e)))?;

            if !response.status().is_success() {
                return Err(AppError::Ollama(format!(
                    "Ollama embed error: {}",
                    response.status()
                )));
            }

            let resp: OllamaEmbedResponse = response
                .json()
                .await
                .map_err(|e| AppError::Ollama(format!("Failed to parse embed: {}", e)))?;

            embeddings.push(resp.embedding);
        }

        tracing::debug!(texts_count, "Ollama embeddings generated");

        Ok(embeddings)
    }
}

// === Ollama API types ===

#[derive(Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct OllamaChatResponse {
    message: OllamaMessage,
    done: bool,
    #[serde(default)]
    prompt_eval_count: Option<i64>,
    #[serde(default)]
    eval_count: Option<i64>,
}

#[derive(Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Deserialize)]
struct OllamaModel {
    name: String,
}

#[derive(Serialize)]
struct OllamaEmbedRequest {
    model: String,
    prompt: String,
}

#[derive(Deserialize)]
struct OllamaEmbedResponse {
    embedding: Vec<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_provider_creation() {
        let provider = OllamaProvider::new(
            "http://localhost:11434".to_string(),
            8,
            "nomic-embed-text".to_string(),
        );
        assert_eq!(provider.name(), "ollama");
        assert_eq!(provider.base_url, "http://localhost:11434");
        assert_eq!(provider.embed_model, "nomic-embed-text");
    }

    #[test]
    fn test_api_url_builder() {
        let provider = OllamaProvider::new(
            "http://localhost:11434/".to_string(),
            8,
            "nomic-embed-text".to_string(),
        );
        assert_eq!(provider.api_url("chat"), "http://localhost:11434/api/chat");
        assert_eq!(provider.api_url("/tags"), "http://localhost:11434/api/tags");
    }
}
