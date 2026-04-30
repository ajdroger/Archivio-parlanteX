/// Multi-contract comparison engine

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use crate::clients::qdrant::{QdrantWrapper, ScoredChunk};
use crate::errors::{AppError, Result};
use crate::models::comparison::{ComparisonAspect, ComparisonCell, ComparisonResult};
use crate::providers::LlmProvider;
use crate::rag::hybrid_search::HybridSearcher;
use qdrant_client::qdrant::{Condition, Filter};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Multi-contract comparator
pub struct MultiContractComparator {
    /// Qdrant client
    qdrant: Arc<QdrantWrapper>,

    /// LLM provider
    llm: Arc<dyn LlmProvider>,

    /// Top-k for per-document retrieval (dense)
    top_k_dense: usize,

    /// Top-k for per-document retrieval (sparse)
    top_k_sparse: usize,

    /// Top-k after reranking per document
    top_k_rerank: usize,

    /// RRF k parameter
    rrf_k: u32,
}

impl MultiContractComparator {
    /// Create new multi-contract comparator
    pub fn new(
        qdrant: Arc<QdrantWrapper>,
        llm: Arc<dyn LlmProvider>,
    ) -> Self {
        Self {
            qdrant,
            llm,
            top_k_dense: 15,
            top_k_sparse: 15,
            top_k_rerank: 8,
            rrf_k: 60,
        }
    }

    /// Compare multiple contracts
    ///
    /// # Arguments
    /// * `doc_ids` - List of document IDs to compare (min 2)
    /// * `question` - Comparison question
    ///
    /// # Returns
    /// ComparisonResult with aspects, differences, recommendations
    pub async fn compare(
        &self,
        doc_ids: Vec<String>,
        question: &str,
    ) -> Result<ComparisonResult> {
        let start = Instant::now();

        tracing::info!(
            doc_count = doc_ids.len(),
            question = %question,
            "Starting multi-contract comparison"
        );

        // Validate input
        if doc_ids.len() < 2 {
            return Err(AppError::BadRequest(
                "At least 2 documents required for comparison".to_string(),
            ));
        }

        // Step 1: Parallel per-document retrieval
        let per_doc_chunks = self.retrieve_per_document(&doc_ids, question).await?;

        tracing::info!(
            total_chunks = per_doc_chunks.values().map(|v| v.len()).sum::<usize>(),
            "Per-document retrieval completed"
        );

        // Step 2: Extract aspects to compare
        let aspects = self.extract_aspects(question, &per_doc_chunks).await?;

        tracing::info!(aspects_count = aspects.len(), "Aspects extracted");

        // Step 3: Per-aspect evidence collection
        let comparison_aspects = self
            .collect_evidence_per_aspect(aspects, &doc_ids, &per_doc_chunks)
            .await?;

        tracing::info!(
            aspects_count = comparison_aspects.len(),
            "Evidence collection completed"
        );

        // Step 4: Generate synthesis (differences + recommendations)
        let (differences_summary, recommendations, information_gaps) = self
            .generate_synthesis(&comparison_aspects, &doc_ids, question)
            .await?;

        tracing::info!("Synthesis generated");

        // Step 5: Validate with Self-RAG (simplified: check all cells have quotes)
        for aspect in &comparison_aspects {
            aspect.validate().map_err(|e| {
                AppError::Internal(anyhow::anyhow!("Comparison validation failed: {}", e))
            })?;
        }

        let processing_ms = start.elapsed().as_millis() as u64;

        let result = ComparisonResult {
            aspects: comparison_aspects,
            differences_summary,
            recommendations,
            information_gaps,
            processing_ms,
        };

        // Final validation
        result.validate().map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Final comparison validation failed: {}", e))
        })?;

        tracing::info!(processing_ms = processing_ms, "Comparison completed");

        Ok(result)
    }

    /// Retrieve relevant chunks per document (parallel)
    async fn retrieve_per_document(
        &self,
        doc_ids: &[String],
        question: &str,
    ) -> Result<HashMap<String, Vec<ScoredChunk>>> {
        // Create hybrid searcher
        let hybrid_searcher = HybridSearcher::new(
            self.qdrant.clone(),
            self.llm.clone(),
            self.top_k_dense,
            self.top_k_sparse,
            self.rrf_k,
        );

        // Parallel retrieval for each document
        let mut tasks = Vec::new();

        for doc_id in doc_ids {
            let doc_id_clone = doc_id.clone();
            let question_clone = question.to_string();
            let searcher = hybrid_searcher.clone();

            let task = tokio::spawn(async move {
                // Create filter for this specific document
                let filter = Filter::must([Condition::matches("doc_id", doc_id_clone.clone())]);

                // Embed query
                // Note: We're calling the same embedding multiple times. In production,
                // embed once and reuse, but for simplicity we let HybridSearcher handle it
                let results = searcher
                    .search_with_filter(&question_clone, 30, Some(filter))
                    .await?;

                Ok::<(String, Vec<ScoredChunk>), AppError>((doc_id_clone, results))
            });

            tasks.push(task);
        }

        // Wait for all tasks
        let results = futures::future::join_all(tasks).await;

        let mut per_doc_chunks = HashMap::new();

        for result in results {
            let (doc_id, chunks) = result
                .map_err(|e| AppError::Internal(anyhow::anyhow!("Task join error: {}", e)))??;
            per_doc_chunks.insert(doc_id, chunks);
        }

        Ok(per_doc_chunks)
    }

    /// Extract aspects to compare using LLM
    async fn extract_aspects(
        &self,
        question: &str,
        per_doc_chunks: &HashMap<String, Vec<ScoredChunk>>,
    ) -> Result<Vec<String>> {
        // Build context from top chunks
        let mut context_samples = Vec::new();

        for (doc_id, chunks) in per_doc_chunks {
            let sample = chunks
                .iter()
                .take(3)
                .map(|c| c.text.as_str())
                .collect::<Vec<_>>()
                .join("\n\n");

            context_samples.push(format!("Documento {}:\n{}", doc_id, sample));
        }

        let context = context_samples.join("\n\n---\n\n");

        let prompt = format!(
            r#"Data questa domanda comparativa e questi estratti di contratto, identifica 4-8 dimensioni/aspetti principali da confrontare.

Domanda: {}

Estratti:
{}

Rispondi SOLO con un JSON valido in questo formato:
{{
  "aspects": ["Aspetto 1", "Aspetto 2", "Aspetto 3", ...]
}}

Esempi di aspetti: "Durata del contratto", "Penali", "Foro competente", "Modalità di pagamento", "Clausola di riservatezza".

JSON:"#,
            question, context
        );

        tracing::debug!(prompt_length = prompt.len(), "Extracting aspects");

        let response = self.llm.generate(&prompt, 500, 0.3).await?;

        // Parse JSON
        let parsed: AspectExtractionResponse = serde_json::from_str(&response)
            .map_err(|e| {
                tracing::warn!(response = %response, error = %e, "Failed to parse aspects JSON");
                AppError::Internal(anyhow::anyhow!("Failed to parse aspects: {}. Response: {}", e, response))
            })?;

        if parsed.aspects.is_empty() {
            return Err(AppError::Internal(anyhow::anyhow!(
                "No aspects extracted from question"
            )));
        }

        Ok(parsed.aspects)
    }

    /// Collect evidence for each aspect in each document
    async fn collect_evidence_per_aspect(
        &self,
        aspects: Vec<String>,
        doc_ids: &[String],
        per_doc_chunks: &HashMap<String, Vec<ScoredChunk>>,
    ) -> Result<Vec<ComparisonAspect>> {
        let mut comparison_aspects = Vec::new();

        for aspect_name in aspects {
            let mut cells = HashMap::new();

            // Parallel evidence collection for this aspect across all documents
            let mut tasks = Vec::new();

            for doc_id in doc_ids {
                let doc_id_clone = doc_id.clone();
                let aspect_clone = aspect_name.clone();
                let chunks = per_doc_chunks
                    .get(doc_id)
                    .map(|c| c.clone())
                    .unwrap_or_default();
                let llm = self.llm.clone();

                let task = tokio::spawn(async move {
                    Self::extract_aspect_evidence(&llm, &aspect_clone, &doc_id_clone, &chunks)
                        .await
                });

                tasks.push(task);
            }

            // Wait for all evidence collection tasks
            let results = futures::future::join_all(tasks).await;

            for result in results {
                let (doc_id, cell) = result
                    .map_err(|e| {
                        AppError::Internal(anyhow::anyhow!("Task join error: {}", e))
                    })??;
                cells.insert(doc_id, cell);
            }

            comparison_aspects.push(ComparisonAspect {
                name: aspect_name,
                description: None,
                cells,
            });
        }

        Ok(comparison_aspects)
    }

    /// Extract evidence for one aspect in one document
    async fn extract_aspect_evidence(
        llm: &Arc<dyn LlmProvider>,
        aspect: &str,
        doc_id: &str,
        chunks: &[ScoredChunk],
    ) -> Result<(String, ComparisonCell)> {
        if chunks.is_empty() {
            // No chunks found for this document
            return Ok((
                doc_id.to_string(),
                ComparisonCell {
                    doc_id: doc_id.to_string(),
                    doc_name: Some(doc_id.to_string()),
                    present: false,
                    text_quote: None,
                    confidence: 1.0,
                    source_chunk_id: None,
                    notes: Some("Nessun chunk rilevante trovato".to_string()),
                },
            ));
        }

        // Build context from chunks
        let context = chunks
            .iter()
            .take(5)
            .map(|c| c.text.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");

        let prompt = format!(
            r#"Estrai il testo esatto che riguarda l'aspetto specificato dal seguente contratto. Se l'aspetto non è presente, ritorna null.

Aspetto da cercare: {}

Testo del contratto:
{}

Rispondi SOLO con un JSON valido in questo formato:
{{
  "present": true/false,
  "text_quote": "testo esatto dal contratto" oppure null se non presente,
  "confidence": 0.0-1.0
}}

IMPORTANTE:
- Se present=true, text_quote DEVE contenere il testo verbatim dal contratto
- Se present=false, text_quote DEVE essere null
- Non inventare testo, riporta solo ciò che è effettivamente scritto

JSON:"#,
            aspect, context
        );

        tracing::debug!(
            aspect = %aspect,
            doc_id = %doc_id,
            "Extracting aspect evidence"
        );

        let response = llm.generate(&prompt, 500, 0.1).await?;

        // Parse JSON
        let parsed: EvidenceExtractionResponse = serde_json::from_str(&response)
            .map_err(|e| {
                tracing::warn!(
                    response = %response,
                    error = %e,
                    "Failed to parse evidence JSON"
                );
                AppError::Internal(anyhow::anyhow!(
                    "Failed to parse evidence for aspect '{}' in doc '{}': {}. Response: {}",
                    aspect,
                    doc_id,
                    e,
                    response
                ))
            })?;

        // Find source chunk ID if present
        let source_chunk_id = if parsed.present && parsed.text_quote.is_some() {
            chunks.first().map(|c| c.id.clone())
        } else {
            None
        };

        let cell = ComparisonCell {
            doc_id: doc_id.to_string(),
            doc_name: Some(doc_id.to_string()),
            present: parsed.present,
            text_quote: parsed.text_quote,
            confidence: parsed.confidence,
            source_chunk_id,
            notes: None,
        };

        // Validate the cell
        cell.validate().map_err(|e| {
            AppError::Internal(anyhow::anyhow!(
                "Evidence extraction produced invalid cell: {}",
                e
            ))
        })?;

        Ok((doc_id.to_string(), cell))
    }

    /// Generate synthesis: differences summary + recommendations
    async fn generate_synthesis(
        &self,
        aspects: &[ComparisonAspect],
        doc_ids: &[String],
        question: &str,
    ) -> Result<(String, Vec<String>, Vec<String>)> {
        // Build context from comparison table
        let mut context = String::new();
        context.push_str("TABELLA COMPARATIVA:\n\n");

        for aspect in aspects {
            context.push_str(&format!("Aspetto: {}\n", aspect.name));

            for doc_id in doc_ids {
                if let Some(cell) = aspect.cells.get(doc_id) {
                    if cell.present {
                        context.push_str(&format!(
                            "  - {}: {}\n",
                            doc_id,
                            cell.text_quote.as_ref().unwrap_or(&"—".to_string())
                        ));
                    } else {
                        context.push_str(&format!("  - {}: NON PRESENTE\n", doc_id));
                    }
                }
            }

            context.push('\n');
        }

        let prompt = format!(
            r#"Data questa tabella comparativa di contratti, genera un'analisi strutturata.

Domanda originale: {}

{}

Rispondi SOLO con un JSON valido in questo formato:
{{
  "differences_summary": "Sintesi narrativa delle differenze chiave (2-4 frasi)",
  "recommendations": ["Raccomandazione 1", "Raccomandazione 2", ...],
  "information_gaps": ["Gap 1", "Gap 2", ...] oppure [] se non ci sono gap
}}

IMPORTANTE:
- Le raccomandazioni devono essere neutrali, NON fornire pareri legali
- Riporta solo ciò che è presente nei contratti
- Se un aspetto manca in alcuni contratti, menzionalo nei gap informativi

JSON:"#,
            question, context
        );

        tracing::debug!("Generating synthesis");

        let response = self.llm.generate(&prompt, 1000, 0.5).await?;

        // Parse JSON
        let parsed: SynthesisResponse = serde_json::from_str(&response).map_err(|e| {
            tracing::warn!(response = %response, error = %e, "Failed to parse synthesis JSON");
            AppError::Internal(anyhow::anyhow!(
                "Failed to parse synthesis: {}. Response: {}",
                e,
                response
            ))
        })?;

        Ok((
            parsed.differences_summary,
            parsed.recommendations,
            parsed.information_gaps,
        ))
    }
}

// === JSON response types ===

#[derive(Debug, Deserialize)]
struct AspectExtractionResponse {
    aspects: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct EvidenceExtractionResponse {
    present: bool,
    text_quote: Option<String>,
    confidence: f32,
}

#[derive(Debug, Deserialize)]
struct SynthesisResponse {
    differences_summary: String,
    recommendations: Vec<String>,
    information_gaps: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aspect_extraction_response_parsing() {
        let json = r#"{"aspects": ["Durata", "Penali", "Foro competente"]}"#;

        let parsed: AspectExtractionResponse =
            serde_json::from_str(json).expect("Should parse");

        assert_eq!(parsed.aspects.len(), 3);
        assert_eq!(parsed.aspects[0], "Durata");
    }

    #[test]
    fn test_evidence_extraction_response_parsing_present() {
        let json = r#"{"present": true, "text_quote": "Durata: 12 mesi", "confidence": 0.95}"#;

        let parsed: EvidenceExtractionResponse =
            serde_json::from_str(json).expect("Should parse");

        assert!(parsed.present);
        assert_eq!(parsed.text_quote.unwrap(), "Durata: 12 mesi");
        assert_eq!(parsed.confidence, 0.95);
    }

    #[test]
    fn test_evidence_extraction_response_parsing_absent() {
        let json = r#"{"present": false, "text_quote": null, "confidence": 1.0}"#;

        let parsed: EvidenceExtractionResponse =
            serde_json::from_str(json).expect("Should parse");

        assert!(!parsed.present);
        assert!(parsed.text_quote.is_none());
    }

    #[test]
    fn test_synthesis_response_parsing() {
        let json = r#"{
            "differences_summary": "I contratti differiscono principalmente nella durata.",
            "recommendations": ["Verificare requisiti aziendali", "Consultare legale"],
            "information_gaps": ["Penali non specificate nel contratto B"]
        }"#;

        let parsed: SynthesisResponse = serde_json::from_str(json).expect("Should parse");

        assert!(parsed.differences_summary.contains("durata"));
        assert_eq!(parsed.recommendations.len(), 2);
        assert_eq!(parsed.information_gaps.len(), 1);
    }
}
