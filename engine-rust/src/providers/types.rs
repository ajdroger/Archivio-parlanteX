/// Shared types for LLM provider communication

use serde::{Deserialize, Serialize};

/// Chat message role
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

/// Single message in conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
        }
    }
}

/// Chat completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    /// Model identifier (provider-specific format)
    pub model: String,

    /// Conversation messages
    pub messages: Vec<Message>,

    /// Optional JSON schema for structured output (Self-RAG)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,

    /// Sampling temperature (0.0 = deterministic, 1.0 = creative)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Max tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
}

/// Response format constraint (for JSON mode)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    pub format_type: String, // "json_object"

    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
}

/// Chat completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// Generated text
    pub content: String,

    /// Provider identifier
    pub provider: String,

    /// Model used
    pub model: String,

    /// Token usage stats
    pub usage: Usage,

    /// Estimated cost in EUR (0.0 for local Ollama)
    pub cost_eur: f64,

    /// Finish reason
    pub finish_reason: FinishReason,
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Completion finish reason
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,          // Natural completion
    Length,        // Hit max_tokens
    ContentFilter, // Blocked by safety filter
    ToolCalls,     // Function calling (future)
}

/// Streaming event (for chat_stream)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamEvent {
    Delta { content: String },
    Done { usage: Usage, cost_eur: f64 },
    Error { message: String },
}

/// Model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub context_window: u32,
    pub input_cost_per_1m: f64,  // EUR per 1M tokens (0.0 for local)
    pub output_cost_per_1m: f64, // EUR per 1M tokens (0.0 for local)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_constructors() {
        let sys = Message::system("You are a helpful assistant");
        assert_eq!(sys.role, Role::System);

        let user = Message::user("Hello");
        assert_eq!(user.role, Role::User);
        assert_eq!(user.content, "Hello");
    }

    #[test]
    fn test_chat_request_serialization() {
        let req = ChatRequest {
            model: "qwen2.5:7b".to_string(),
            messages: vec![Message::user("Test")],
            response_format: None,
            temperature: Some(0.7),
            max_tokens: Some(500),
            stop: None,
        };

        let json = serde_json::to_string(&req).expect("Serialization failed");
        assert!(json.contains("\"model\":\"qwen2.5:7b\""));
        assert!(json.contains("\"temperature\":0.7"));
    }
}
