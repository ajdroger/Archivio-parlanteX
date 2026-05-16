-- ==========================================================================
-- Migration 009: Hallucination Detection Tracking
-- Fase 6.2 - Advanced Hallucination Detection
--
-- Adds hallucination detection metrics to ap_chat_messages for tracking
-- claim verification and flagged claims in LLM-generated answers.
-- ==========================================================================

USE archivio_parlante_x;

-- Add hallucination detection fields to chat messages
ALTER TABLE ap_chat_messages
  ADD COLUMN hallucination_score DECIMAL(3,2) DEFAULT NULL
    COMMENT 'Hallucination score 0.00-1.00 (higher = more hallucination)',
  ADD COLUMN flagged_claims_count INT DEFAULT 0
    COMMENT 'Number of claims without citation support',
  ADD COLUMN verified_at DATETIME DEFAULT NULL
    COMMENT 'Timestamp of hallucination verification';

-- Index for monitoring hallucination rates
CREATE INDEX idx_hallucination_score ON ap_chat_messages(hallucination_score);

-- Index for querying verified messages
CREATE INDEX idx_verified_at ON ap_chat_messages(verified_at);

-- ==========================================================================
-- Migration applied successfully
-- ==========================================================================
