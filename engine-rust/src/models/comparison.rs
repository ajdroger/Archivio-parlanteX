/// Multi-contract comparison models

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete comparison result
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ComparisonResult {
    /// Aspects being compared (e.g., "Durata", "Penali", "Foro competente")
    pub aspects: Vec<ComparisonAspect>,

    /// Narrative summary of key differences
    pub differences_summary: String,

    /// Neutral recommendations (no legal advice)
    pub recommendations: Vec<String>,

    /// Information gaps (missing data in contracts)
    pub information_gaps: Vec<String>,

    /// Processing time in milliseconds
    pub processing_ms: u64,
}

/// Single aspect across multiple contracts
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ComparisonAspect {
    /// Aspect name (e.g., "Durata del contratto")
    pub name: String,

    /// Description of this aspect
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Evidence cells per document (key = doc_id)
    pub cells: HashMap<String, ComparisonCell>,
}

/// Evidence cell for one document in one aspect
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ComparisonCell {
    /// Document ID
    pub doc_id: String,

    /// Document name (for display)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_name: Option<String>,

    /// Whether this aspect is present in the document
    pub present: bool,

    /// Verbatim text quote from document (MUST be present if present=true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_quote: Option<String>,

    /// Confidence score (0.0-1.0)
    pub confidence: f32,

    /// Source chunk ID for traceability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_chunk_id: Option<String>,

    /// Additional context or notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl ComparisonCell {
    /// Validate that the cell is consistent
    pub fn validate(&self) -> Result<(), String> {
        // If present=true, text_quote must exist
        if self.present && self.text_quote.is_none() {
            return Err(format!(
                "ComparisonCell for doc {} is marked present=true but has no text_quote",
                self.doc_id
            ));
        }

        // If present=false, text_quote should be None
        if !self.present && self.text_quote.is_some() {
            return Err(format!(
                "ComparisonCell for doc {} is marked present=false but has text_quote",
                self.doc_id
            ));
        }

        // Confidence must be in [0, 1]
        if !(0.0..=1.0).contains(&self.confidence) {
            return Err(format!(
                "ComparisonCell confidence {} is out of range [0, 1]",
                self.confidence
            ));
        }

        // Text quote should not be empty if present
        if let Some(ref quote) = self.text_quote {
            if quote.trim().is_empty() {
                return Err(format!(
                    "ComparisonCell for doc {} has empty text_quote",
                    self.doc_id
                ));
            }
        }

        Ok(())
    }
}

impl ComparisonAspect {
    /// Validate all cells in this aspect
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("ComparisonAspect name cannot be empty".to_string());
        }

        for (doc_id, cell) in &self.cells {
            cell.validate().map_err(|e| {
                format!(
                    "Validation failed for aspect '{}', doc '{}': {}",
                    self.name, doc_id, e
                )
            })?;
        }

        Ok(())
    }
}

impl ComparisonResult {
    /// Validate the entire comparison result
    pub fn validate(&self) -> Result<(), String> {
        if self.aspects.is_empty() {
            return Err("ComparisonResult must have at least one aspect".to_string());
        }

        for aspect in &self.aspects {
            aspect.validate()?;
        }

        Ok(())
    }

    /// Render as Markdown table + sections
    pub fn to_markdown(&self, doc_names: &HashMap<String, String>) -> String {
        let mut md = String::new();

        // Title
        md.push_str("# Confronto Contratti\n\n");

        // Comparison table
        md.push_str("## Tabella Comparativa\n\n");

        // Build table header
        if let Some(first_aspect) = self.aspects.first() {
            let doc_ids: Vec<&String> = first_aspect.cells.keys().collect();

            md.push_str("| Aspetto |");
            for doc_id in &doc_ids {
                let doc_name = doc_names.get(*doc_id).map(|s| s.as_str()).unwrap_or(doc_id);
                md.push_str(&format!(" {} |", doc_name));
            }
            md.push_str("\n");

            // Header separator
            md.push_str("|---------|");
            for _ in &doc_ids {
                md.push_str("---------|");
            }
            md.push_str("\n");

            // Table rows
            for aspect in &self.aspects {
                md.push_str(&format!("| **{}** |", aspect.name));

                for doc_id in &doc_ids {
                    if let Some(cell) = aspect.cells.get(*doc_id) {
                        if cell.present {
                            let quote = cell
                                .text_quote
                                .as_ref()
                                .map(|s| {
                                    let truncated = if s.len() > 150 {
                                        format!("{}...", &s[..150])
                                    } else {
                                        s.clone()
                                    };
                                    truncated.replace('\n', " ").replace('|', "\\|")
                                })
                                .unwrap_or_else(|| "—".to_string());
                            md.push_str(&format!(" {} |", quote));
                        } else {
                            md.push_str(" ❌ *Non presente* |");
                        }
                    } else {
                        md.push_str(" — |");
                    }
                }

                md.push_str("\n");
            }
        }

        md.push_str("\n");

        // Differences summary
        if !self.differences_summary.is_empty() {
            md.push_str("## Differenze Chiave\n\n");
            md.push_str(&self.differences_summary);
            md.push_str("\n\n");
        }

        // Recommendations
        if !self.recommendations.is_empty() {
            md.push_str("## Raccomandazioni\n\n");
            for (i, rec) in self.recommendations.iter().enumerate() {
                md.push_str(&format!("{}. {}\n", i + 1, rec));
            }
            md.push_str("\n");
        }

        // Information gaps
        if !self.information_gaps.is_empty() {
            md.push_str("## Lacune Informative\n\n");
            for gap in &self.information_gaps {
                md.push_str(&format!("- {}\n", gap));
            }
            md.push_str("\n");
        }

        // Footer
        md.push_str("---\n\n");
        md.push_str("*Questo confronto è generato automaticamente e riporta solo il contenuto testuale dei contratti. Non costituisce parere legale.*\n");

        md
    }
}

/// Request for contract comparison
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CompareRequest {
    /// Knowledge base ID
    pub kb_id: String,

    /// List of document IDs to compare (min 2)
    pub doc_ids: Vec<String>,

    /// Comparison question/query
    pub question: String,

    /// Whether to save the analysis to database
    #[serde(default)]
    pub save_analysis: bool,
}

/// Response for contract comparison
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CompareResponse {
    /// Markdown-formatted result
    pub markdown_result: String,

    /// Structured comparison data
    pub structured: ComparisonResult,

    /// Processing time in milliseconds
    pub processing_ms: u64,

    /// Analysis ID if saved to database
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analysis_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comparison_cell_validation_present_without_quote() {
        let cell = ComparisonCell {
            doc_id: "doc_1".to_string(),
            doc_name: Some("Contract A".to_string()),
            present: true,
            text_quote: None,
            confidence: 0.95,
            source_chunk_id: None,
            notes: None,
        };

        assert!(cell.validate().is_err());
    }

    #[test]
    fn test_comparison_cell_validation_not_present_with_quote() {
        let cell = ComparisonCell {
            doc_id: "doc_1".to_string(),
            doc_name: Some("Contract A".to_string()),
            present: false,
            text_quote: Some("Some text".to_string()),
            confidence: 0.95,
            source_chunk_id: None,
            notes: None,
        };

        assert!(cell.validate().is_err());
    }

    #[test]
    fn test_comparison_cell_validation_valid() {
        let cell = ComparisonCell {
            doc_id: "doc_1".to_string(),
            doc_name: Some("Contract A".to_string()),
            present: true,
            text_quote: Some("Il contratto avrà durata di 12 mesi".to_string()),
            confidence: 0.95,
            source_chunk_id: Some("chunk_123".to_string()),
            notes: None,
        };

        assert!(cell.validate().is_ok());
    }

    #[test]
    fn test_comparison_cell_validation_confidence_out_of_range() {
        let cell = ComparisonCell {
            doc_id: "doc_1".to_string(),
            doc_name: Some("Contract A".to_string()),
            present: true,
            text_quote: Some("Text".to_string()),
            confidence: 1.5,
            source_chunk_id: None,
            notes: None,
        };

        assert!(cell.validate().is_err());
    }

    #[test]
    fn test_comparison_result_to_markdown() {
        let mut cells1 = HashMap::new();
        cells1.insert(
            "doc_1".to_string(),
            ComparisonCell {
                doc_id: "doc_1".to_string(),
                doc_name: Some("NDA 2023".to_string()),
                present: true,
                text_quote: Some("Durata: 24 mesi dalla firma".to_string()),
                confidence: 0.95,
                source_chunk_id: Some("chunk_1".to_string()),
                notes: None,
            },
        );
        cells1.insert(
            "doc_2".to_string(),
            ComparisonCell {
                doc_id: "doc_2".to_string(),
                doc_name: Some("NDA 2024".to_string()),
                present: true,
                text_quote: Some("Durata: 36 mesi dalla data di sottoscrizione".to_string()),
                confidence: 0.92,
                source_chunk_id: Some("chunk_2".to_string()),
                notes: None,
            },
        );

        let result = ComparisonResult {
            aspects: vec![ComparisonAspect {
                name: "Durata del contratto".to_string(),
                description: None,
                cells: cells1,
            }],
            differences_summary: "Il contratto 2024 ha durata maggiore di 12 mesi".to_string(),
            recommendations: vec![
                "Verificare la compatibilità con i requisiti aziendali".to_string()
            ],
            information_gaps: vec![],
            processing_ms: 1500,
        };

        let mut doc_names = HashMap::new();
        doc_names.insert("doc_1".to_string(), "NDA 2023".to_string());
        doc_names.insert("doc_2".to_string(), "NDA 2024".to_string());

        let markdown = result.to_markdown(&doc_names);

        assert!(markdown.contains("# Confronto Contratti"));
        assert!(markdown.contains("## Tabella Comparativa"));
        assert!(markdown.contains("Durata del contratto"));
        assert!(markdown.contains("24 mesi"));
        assert!(markdown.contains("36 mesi"));
        assert!(markdown.contains("## Differenze Chiave"));
        assert!(markdown.contains("## Raccomandazioni"));
    }

    #[test]
    fn test_comparison_result_validation_empty_aspects() {
        let result = ComparisonResult {
            aspects: vec![],
            differences_summary: String::new(),
            recommendations: vec![],
            information_gaps: vec![],
            processing_ms: 0,
        };

        assert!(result.validate().is_err());
    }
}
