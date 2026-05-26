/// LLM provider registry for runtime switching

use std::collections::HashMap;
use std::sync::Arc;

use crate::config::Config;
use crate::errors::{AppError, Result};
use super::{
    LlmProvider,
    ollama::OllamaProvider,
    anthropic::AnthropicProvider,
    openai::OpenAIProvider,
    google::GoogleProvider,
    deepseek::DeepSeekProvider,
    qwen::QwenProvider,
    moonshot::MoonshotProvider,
    zhipu::ZhipuProvider,
    mistral::MistralProvider,
    groq::GroqProvider,
    openrouter::OpenRouterProvider,
    together::TogetherProvider,
    fireworks::FireworksProvider,
};

/// Registry of available LLM providers
///
/// Manages lifecycle and routing of requests to appropriate provider.
pub struct LlmRegistry {
    providers: HashMap<String, Arc<dyn LlmProvider>>,
    default_provider: String,
}

impl LlmRegistry {
    /// Initialize registry from config
    ///
    /// Registers Ollama (always) + cloud providers (if API keys present).
    pub fn from_config(config: &Config) -> Self {
        let mut providers: HashMap<String, Arc<dyn LlmProvider>> = HashMap::new();

        // Ollama (always available, zero-cost default)
        let ollama = Arc::new(OllamaProvider::new(
            config.ollama_url.clone(),
            config.max_concurrent_llm_calls,
            config.ollama_model_chat.clone(),
            config.ollama_model_embed.clone(),
        ));
        providers.insert("ollama".to_string(), ollama);

        // Cloud providers: Register if API keys present
        let max_concurrent = config.max_concurrent_llm_calls;

        if let Some(key) = &config.anthropic_api_key {
            if !key.is_empty() {
                providers.insert("anthropic".to_string(), Arc::new(AnthropicProvider::new(key.clone(), max_concurrent, None)));
            }
        }

        if let Some(key) = &config.openai_api_key {
            if !key.is_empty() {
                providers.insert("openai".to_string(), Arc::new(OpenAIProvider::new(key.clone(), max_concurrent, None)));
            }
        }

        if let Some(key) = &config.google_api_key {
            if !key.is_empty() {
                providers.insert("google".to_string(), Arc::new(GoogleProvider::new(key.clone(), max_concurrent, None)));
            }
        }

        if let Some(key) = &config.deepseek_api_key {
            if !key.is_empty() {
                providers.insert("deepseek".to_string(), Arc::new(DeepSeekProvider::new(key.clone(), max_concurrent, None)));
            }
        }

        if let Some(key) = &config.qwen_api_key {
            if !key.is_empty() {
                providers.insert("qwen".to_string(), Arc::new(QwenProvider::new(key.clone(), max_concurrent, None)));
            }
        }

        if let Some(key) = &config.moonshot_api_key {
            if !key.is_empty() {
                providers.insert("moonshot".to_string(), Arc::new(MoonshotProvider::new(key.clone(), max_concurrent, None)));
            }
        }

        if let Some(key) = &config.zhipu_api_key {
            if !key.is_empty() {
                providers.insert("zhipu".to_string(), Arc::new(ZhipuProvider::new(key.clone(), max_concurrent, None)));
            }
        }

        if let Some(key) = &config.mistral_api_key {
            if !key.is_empty() {
                providers.insert("mistral".to_string(), Arc::new(MistralProvider::new(key.clone(), max_concurrent, None)));
            }
        }

        if let Some(key) = &config.groq_api_key {
            if !key.is_empty() {
                providers.insert("groq".to_string(), Arc::new(GroqProvider::new(key.clone(), max_concurrent, None)));
            }
        }

        if let Some(key) = &config.openrouter_api_key {
            if !key.is_empty() {
                providers.insert("openrouter".to_string(), Arc::new(OpenRouterProvider::new(key.clone(), max_concurrent, None)));
            }
        }

        if let Some(key) = &config.together_api_key {
            if !key.is_empty() {
                providers.insert("together".to_string(), Arc::new(TogetherProvider::new(key.clone(), max_concurrent, None)));
            }
        }

        if let Some(key) = &config.fireworks_api_key {
            if !key.is_empty() {
                providers.insert("fireworks".to_string(), Arc::new(FireworksProvider::new(key.clone(), max_concurrent, None)));
            }
        }

        tracing::info!(
            provider_count = providers.len(),
            "LLM registry initialized"
        );

        Self {
            providers,
            default_provider: "ollama".to_string(),
        }
    }

    /// Get provider by name
    ///
    /// # Errors
    /// Returns `AppError::NotFound` if provider not registered
    pub fn get(&self, name: &str) -> Result<Arc<dyn LlmProvider>> {
        self.providers
            .get(name)
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("LLM provider '{}' not found", name)))
    }

    /// Get default provider (Ollama)
    pub fn default(&self) -> Arc<dyn LlmProvider> {
        self.providers
            .get(&self.default_provider)
            .expect("Default provider must exist")
            .clone()
    }

    /// List all registered provider names
    pub fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    /// Check if provider is registered
    pub fn has_provider(&self, name: &str) -> bool {
        self.providers.contains_key(name)
    }

    /// Get provider by name or default if not found
    pub fn get_or_default(&self, name: Option<&str>) -> Arc<dyn LlmProvider> {
        name.and_then(|n| self.providers.get(n).cloned())
            .unwrap_or_else(|| self.default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Config {
        Config {
            listen_addr: "0.0.0.0:8090".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            ollama_model_chat: "qwen2.5:7b".to_string(),
            ollama_model_chat_small: "qwen2.5:3b".to_string(),
            ollama_model_embed: "nomic-embed-text".to_string(),
            qdrant_url: "http://localhost:6335".to_string(),
            python_worker_url: "http://localhost:8091".to_string(),
            mysql_url: "mysql://root@localhost/test".to_string(),
            mysql_db: "test".to_string(),
            rust_engine_internal_token: "test-token".to_string(),
            chunk_size_tokens: 800,
            chunk_overlap_percent: 15,
            top_k_dense: 30,
            top_k_sparse: 30,
            top_k_rerank: 5,
            hybrid_fusion_k: 60,
            max_concurrent_embeddings: 16,
            max_concurrent_llm_calls: 8,
            anthropic_api_key: None,
            google_api_key: None,
            openai_api_key: None,
            deepseek_api_key: None,
            qwen_api_key: None,
            moonshot_api_key: None,
            zhipu_api_key: None,
            mistral_api_key: None,
            groq_api_key: None,
            openrouter_api_key: None,
            together_api_key: None,
            fireworks_api_key: None,
            redis_url: "redis://localhost:6379".to_string(),
            daily_cost_budget_eur: 0.0,
            app_env: "dev".to_string(),
            cors_origins: vec!["http://localhost:3000".to_string()],
        }
    }

    #[test]
    fn test_registry_initialization() {
        let config = test_config();
        let registry = LlmRegistry::from_config(&config);

        assert!(registry.has_provider("ollama"));
        assert_eq!(registry.list_providers().len(), 1);
    }

    #[test]
    fn test_get_provider() {
        let config = test_config();
        let registry = LlmRegistry::from_config(&config);

        let provider = registry.get("ollama").expect("Ollama should exist");
        assert_eq!(provider.name(), "ollama");
    }

    #[test]
    fn test_get_missing_provider() {
        let config = test_config();
        let registry = LlmRegistry::from_config(&config);

        let result = registry.get("anthropic");
        assert!(result.is_err());

        if let Err(AppError::NotFound(msg)) = result {
            assert!(msg.contains("anthropic"));
        } else {
            panic!("Expected NotFound error");
        }
    }

    #[test]
    fn test_default_provider() {
        let config = test_config();
        let registry = LlmRegistry::from_config(&config);

        let provider = registry.default();
        assert_eq!(provider.name(), "ollama");
    }

    #[test]
    fn test_get_or_default() {
        let config = test_config();
        let registry = LlmRegistry::from_config(&config);

        // Existing provider
        let provider = registry.get_or_default(Some("ollama"));
        assert_eq!(provider.name(), "ollama");

        // Missing provider → default
        let provider = registry.get_or_default(Some("nonexistent"));
        assert_eq!(provider.name(), "ollama");

        // None → default
        let provider = registry.get_or_default(None);
        assert_eq!(provider.name(), "ollama");
    }
}
