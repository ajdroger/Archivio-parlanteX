/// Intent classification for routing queries

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::errors::{AppError, Result};
use crate::providers::LlmProvider;

/// User intent for query routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Intent {
    /// Standard RAG query (default)
    RagQuery,

    /// Multi-contract comparison
    ContractComparison,

    /// Document summarization
    Summarize,

    /// General chat without RAG
    GeneralChat,
}

impl Default for Intent {
    fn default() -> Self {
        Intent::RagQuery
    }
}

/// Intent classifier using LLM
pub struct IntentClassifier {
    /// LLM provider for classification
    llm: Arc<dyn LlmProvider>,
}

impl IntentClassifier {
    /// Create new intent classifier
    pub fn new(llm: Arc<dyn LlmProvider>) -> Self {
        Self { llm }
    }

    /// Classify user intent from question
    ///
    /// # Arguments
    /// * `question` - User's question
    /// * `kb_has_docs` - Whether the KB has documents (affects routing)
    ///
    /// # Returns
    /// Classified intent, defaults to RagQuery on parse failure
    pub async fn classify(&self, question: &str, kb_has_docs: bool) -> Result<Intent> {
        tracing::debug!(
            question = %question,
            kb_has_docs = kb_has_docs,
            "Classifying intent"
        );

        let prompt = build_classification_prompt(question, kb_has_docs);

        // Use small model with low temperature for classification
        let response = self
            .llm
            .generate(&prompt, 100, 0.1)
            .await
            .map_err(|e| {
                tracing::warn!(error = %e, "Intent classification LLM call failed");
                e
            })?;

        // Parse JSON response
        match serde_json::from_str::<IntentResponse>(&response) {
            Ok(parsed) => {
                tracing::info!(
                    intent = ?parsed.intent,
                    confidence = parsed.confidence,
                    "Intent classified"
                );
                Ok(parsed.intent)
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    response = %response,
                    "Failed to parse intent JSON, defaulting to RagQuery"
                );
                // Fallback to RagQuery on parse error
                Ok(Intent::RagQuery)
            }
        }
    }
}

/// Build classification prompt
fn build_classification_prompt(question: &str, kb_has_docs: bool) -> String {
    let context = if kb_has_docs {
        "Il knowledge base contiene documenti indicizzati."
    } else {
        "Il knowledge base è vuoto (nessun documento indicizzato)."
    };

    format!(
        r#"Classifica l'intento dell'utente dalla domanda fornita.

Contesto: {}

Domanda utente: "{}"

Intenti possibili:
- "rag_query": Ricerca informazioni nei documenti (es. "Qual è la durata del contratto?", "Trova clausole di riservatezza")
- "contract_comparison": Confronto tra più contratti (es. "Confronta i contratti A e B", "Quali differenze tra NDA 2023 e 2024?")
- "summarize": Riassunto di uno o più documenti (es. "Riassumi il contratto", "Sintesi dei punti chiave")
- "general_chat": Conversazione generale senza bisogno di documenti (es. "Ciao", "Come funziona questo sistema?", "Cosa puoi fare?")

Rispondi SOLO con un JSON valido in questo formato:
{{
  "intent": "rag_query" | "contract_comparison" | "summarize" | "general_chat",
  "confidence": 0.0-1.0
}}

REGOLE:
- Se la domanda menziona "confronta", "differenze", "paragona" → contract_comparison
- Se la domanda chiede "riassumi", "sintesi", "punti chiave" → summarize
- Se la domanda è saluti, domande sul sistema, o non richiede documenti → general_chat
- Altrimenti → rag_query (default)

JSON:"#,
        context, question
    )
}

/// JSON response from LLM
#[derive(Debug, Deserialize)]
struct IntentResponse {
    intent: Intent,
    confidence: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_serde() {
        let json = r#"{"intent": "rag_query", "confidence": 0.95}"#;
        let parsed: IntentResponse = serde_json::from_str(json).expect("Should parse");
        assert_eq!(parsed.intent, Intent::RagQuery);
        assert_eq!(parsed.confidence, 0.95);
    }

    #[test]
    fn test_intent_serde_comparison() {
        let json = r#"{"intent": "contract_comparison", "confidence": 0.88}"#;
        let parsed: IntentResponse = serde_json::from_str(json).expect("Should parse");
        assert_eq!(parsed.intent, Intent::ContractComparison);
    }

    #[test]
    fn test_intent_serde_summarize() {
        let json = r#"{"intent": "summarize", "confidence": 0.92}"#;
        let parsed: IntentResponse = serde_json::from_str(json).expect("Should parse");
        assert_eq!(parsed.intent, Intent::Summarize);
    }

    #[test]
    fn test_intent_serde_general_chat() {
        let json = r#"{"intent": "general_chat", "confidence": 0.99}"#;
        let parsed: IntentResponse = serde_json::from_str(json).expect("Should parse");
        assert_eq!(parsed.intent, Intent::GeneralChat);
    }

    #[test]
    fn test_intent_default() {
        assert_eq!(Intent::default(), Intent::RagQuery);
    }

    #[test]
    fn test_build_classification_prompt_with_docs() {
        let prompt = build_classification_prompt("Qual è la durata?", true);

        assert!(prompt.contains("Il knowledge base contiene documenti"));
        assert!(prompt.contains("Qual è la durata?"));
        assert!(prompt.contains("rag_query"));
        assert!(prompt.contains("contract_comparison"));
    }

    #[test]
    fn test_build_classification_prompt_empty_kb() {
        let prompt = build_classification_prompt("Ciao", false);

        assert!(prompt.contains("Il knowledge base è vuoto"));
        assert!(prompt.contains("Ciao"));
    }
}
