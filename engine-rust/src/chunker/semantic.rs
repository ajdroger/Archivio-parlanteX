/// Semantic chunker with Italian legal contract awareness

use regex::Regex;
use serde_json::json;

use crate::errors::Result;
use crate::models::chunk::Chunk;
use crate::utils::tokenizer;

/// Semantic chunker for legal contracts
///
/// Splits text respecting:
/// - Markdown headers (# ## ###)
/// - Italian legal clauses (Art., Articolo, CAPO, Sezione)
/// - Sentence boundaries
/// - Token limits with overlap
pub struct SemanticChunker {
    /// Target chunk size in tokens
    pub chunk_size: usize,

    /// Overlap percentage (0.0-1.0)
    pub overlap_pct: f32,
}

impl SemanticChunker {
    /// Create new semantic chunker
    pub fn new(chunk_size: usize, overlap_pct: f32) -> Self {
        Self {
            chunk_size,
            overlap_pct: overlap_pct.clamp(0.0, 1.0),
        }
    }

    /// Chunk document with semantic boundaries
    ///
    /// # Arguments
    /// * `text` - Full document text
    /// * `doc_id` - Document identifier
    ///
    /// # Returns
    /// Vector of chunks with metadata
    pub fn chunk(&self, text: &str, doc_id: &str) -> Result<Vec<Chunk>> {
        if text.trim().is_empty() {
            return Ok(Vec::new());
        }

        // Step A: Split by markdown headers
        let sections = self.split_by_headers(text);

        // Step B: Further split by legal clauses
        let mut segments = Vec::new();
        for section in sections {
            segments.extend(self.split_by_clauses(&section.text, section.header.as_deref()));
        }

        // Step C: Split large segments by sentences
        let mut final_segments = Vec::new();
        for segment in segments {
            final_segments.extend(self.split_by_sentences(&segment)?);
        }

        // Step D: Apply overlap and create chunks
        let chunks = self.create_chunks_with_overlap(doc_id, final_segments)?;

        tracing::info!(
            doc_id = %doc_id,
            chunks_count = chunks.len(),
            avg_tokens = if !chunks.is_empty() {
                chunks.iter().map(|c| c.token_count).sum::<usize>() / chunks.len()
            } else {
                0
            },
            "Document chunked"
        );

        Ok(chunks)
    }

    /// Split text by markdown headers
    fn split_by_headers(&self, text: &str) -> Vec<Section> {
        let header_regex = Regex::new(r"^(#{1,6})\s+(.+)$").expect("Invalid regex");

        let mut sections = Vec::new();
        let mut current_header: Option<String> = None;
        let mut current_text = String::new();

        for line in text.lines() {
            if let Some(caps) = header_regex.captures(line) {
                // Save previous section
                if !current_text.trim().is_empty() {
                    sections.push(Section {
                        header: current_header.clone(),
                        text: current_text.trim().to_string(),
                    });
                }

                // Start new section
                current_header = Some(caps[2].to_string());
                current_text = String::new();
            } else {
                current_text.push_str(line);
                current_text.push('\n');
            }
        }

        // Save last section
        if !current_text.trim().is_empty() {
            sections.push(Section {
                header: current_header,
                text: current_text.trim().to_string(),
            });
        }

        // If no headers found, return whole text as single section
        if sections.is_empty() {
            sections.push(Section {
                header: None,
                text: text.to_string(),
            });
        }

        sections
    }

    /// Split by Italian legal clause markers
    fn split_by_clauses(&self, text: &str, header: Option<&str>) -> Vec<Segment> {
        let clause_regex = Regex::new(
            r"(?m)^(\d+\.\d*\.?|Art\.?\s*\d+|Articolo\s+\d+|CAPO\s+[IVX]+|Sezione\s+\d+)"
        ).expect("Invalid regex");

        let mut segments = Vec::new();
        let mut last_pos = 0;
        let mut last_clause: Option<String> = None;

        for mat in clause_regex.find_iter(text) {
            // Save previous segment
            if mat.start() > last_pos {
                let seg_text = text[last_pos..mat.start()].trim();
                if !seg_text.is_empty() {
                    segments.push(Segment {
                        text: seg_text.to_string(),
                        header: header.map(|h| h.to_string()),
                        clause: last_clause.clone(),
                    });
                }
            }

            last_clause = Some(mat.as_str().to_string());
            last_pos = mat.start();
        }

        // Save last segment
        let remaining = text[last_pos..].trim();
        if !remaining.is_empty() {
            segments.push(Segment {
                text: remaining.to_string(),
                header: header.map(|h| h.to_string()),
                clause: last_clause,
            });
        }

        // If no clauses found, return whole text
        if segments.is_empty() {
            segments.push(Segment {
                text: text.to_string(),
                header: header.map(|h| h.to_string()),
                clause: None,
            });
        }

        segments
    }

    /// Split segment by sentences if exceeds chunk_size
    fn split_by_sentences(&self, segment: &Segment) -> Result<Vec<Segment>> {
        let token_count = tokenizer::count_tokens(&segment.text)?;

        if token_count <= self.chunk_size {
            return Ok(vec![segment.clone()]);
        }

        // Split by sentence boundaries (Italian-aware)
        let sentence_regex = Regex::new(r"[.!?]\s+(?=[A-ZÀ-Ü])").expect("Invalid regex");

        let mut segments = Vec::new();
        let mut current = String::new();

        for sentence in sentence_regex.split(&segment.text) {
            let test_text = if current.is_empty() {
                sentence.to_string()
            } else {
                format!("{} {}", current, sentence)
            };

            let test_count = tokenizer::count_tokens(&test_text)?;

            if test_count > self.chunk_size && !current.is_empty() {
                // Save current segment
                segments.push(Segment {
                    text: current.trim().to_string(),
                    header: segment.header.clone(),
                    clause: segment.clause.clone(),
                });
                current = sentence.to_string();
            } else {
                current = test_text;
            }
        }

        // Save last segment
        if !current.trim().is_empty() {
            segments.push(Segment {
                text: current.trim().to_string(),
                header: segment.header.clone(),
                clause: segment.clause.clone(),
            });
        }

        // If still too large, force split by token limit
        let mut final_segments = Vec::new();
        for seg in segments {
            let seg_tokens = tokenizer::count_tokens(&seg.text)?;
            if seg_tokens > self.chunk_size * 12 / 10 {
                // 20% tolerance
                tracing::warn!(
                    tokens = seg_tokens,
                    limit = self.chunk_size,
                    "Segment exceeds limit, force-splitting by tokens"
                );

                let split_texts = tokenizer::split_by_token_limit(&seg.text, self.chunk_size)?;
                for text in split_texts {
                    final_segments.push(Segment {
                        text,
                        header: seg.header.clone(),
                        clause: seg.clause.clone(),
                    });
                }
            } else {
                final_segments.push(seg);
            }
        }

        Ok(final_segments)
    }

    /// Create chunks with overlap
    fn create_chunks_with_overlap(&self, doc_id: &str, segments: Vec<Segment>) -> Result<Vec<Chunk>> {
        if segments.is_empty() {
            return Ok(Vec::new());
        }

        let mut chunks = Vec::new();
        let overlap_chars = (self.chunk_size as f32 * self.overlap_pct) as usize;

        let mut char_offset = 0;
        let mut previous_tail = String::new();

        for (idx, segment) in segments.into_iter().enumerate() {
            let text = if idx == 0 {
                segment.text.clone()
            } else {
                // Prepend overlap from previous chunk
                let overlap = if previous_tail.len() > overlap_chars {
                    &previous_tail[previous_tail.len() - overlap_chars..]
                } else {
                    &previous_tail
                };

                format!("{}{}", overlap, segment.text)
            };

            let token_count = tokenizer::count_tokens(&text)?;
            let start_offset = char_offset;
            let end_offset = char_offset + segment.text.len();

            let mut metadata = json!({
                "chunk_idx": idx,
            });

            if let Some(header) = segment.header {
                metadata["section_header"] = json!(header);
            }

            if let Some(clause) = segment.clause {
                metadata["clause_marker"] = json!(clause);
                metadata["is_clause_start"] = json!(true);
            }

            let chunk = Chunk::new(
                doc_id.to_string(),
                idx,
                text.clone(),
                token_count,
                start_offset,
                end_offset,
                metadata,
            );

            chunks.push(chunk);

            previous_tail = segment.text.clone();
            char_offset = end_offset;
        }

        Ok(chunks)
    }
}

/// Document section with header
#[derive(Debug, Clone)]
struct Section {
    header: Option<String>,
    text: String,
}

/// Text segment with metadata
#[derive(Debug, Clone)]
struct Segment {
    text: String,
    header: Option<String>,
    clause: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunker_creation() {
        let chunker = SemanticChunker::new(800, 0.15);
        assert_eq!(chunker.chunk_size, 800);
        assert!((chunker.overlap_pct - 0.15).abs() < 0.01);
    }

    #[test]
    fn test_chunker_empty_text() {
        let chunker = SemanticChunker::new(800, 0.15);
        let chunks = chunker.chunk("", "doc_001").expect("Should handle empty text");
        assert_eq!(chunks.len(), 0);
    }

    #[test]
    fn test_chunker_short_text() {
        let chunker = SemanticChunker::new(800, 0.15);
        let text = "This is a short contract clause.";
        let chunks = chunker.chunk(text, "doc_002").expect("Should handle short text");

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].doc_id, "doc_002");
        assert_eq!(chunks[0].chunk_idx, 0);
        assert!(chunks[0].token_count < 800);
    }

    #[test]
    fn test_split_by_headers() {
        let chunker = SemanticChunker::new(800, 0.15);
        let text = "# Introduction\nSome text here.\n## Section 1\nMore text.";

        let sections = chunker.split_by_headers(text);

        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].header, Some("Introduction".to_string()));
        assert_eq!(sections[1].header, Some("Section 1".to_string()));
    }

    #[test]
    fn test_split_by_legal_clauses() {
        let chunker = SemanticChunker::new(800, 0.15);
        let text = "Art. 1 - Prima clausola.\nArt. 2 - Seconda clausola.\nArticolo 3 - Terza clausola.";

        let segments = chunker.split_by_clauses(text, None);

        assert!(segments.len() >= 3);
        assert!(segments.iter().any(|s| s.clause.as_ref().map_or(false, |c| c.contains("Art. 1"))));
        assert!(segments.iter().any(|s| s.clause.as_ref().map_or(false, |c| c.contains("Articolo 3"))));
    }

    #[test]
    fn test_overlap_pct_clamping() {
        let chunker1 = SemanticChunker::new(800, 1.5);
        assert_eq!(chunker1.overlap_pct, 1.0);

        let chunker2 = SemanticChunker::new(800, -0.5);
        assert_eq!(chunker2.overlap_pct, 0.0);
    }
}
