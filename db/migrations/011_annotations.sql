-- ==========================================================================
-- Migration 011: Collaborative Annotation Schema
-- Fase 6.4 - Real-time Collaborative Annotation
--
-- Creates tables for collaborative annotations on PDF chunks with real-time
-- updates via WebSocket. Supports annotation threads (replies), soft delete,
-- and position tracking.
-- ==========================================================================

USE archivio_parlante_x;

-- ==========================================================================
-- Annotations (Main Table)
-- ==========================================================================
CREATE TABLE IF NOT EXISTS ap_annotations (
    id CHAR(36) PRIMARY KEY COMMENT 'UUID',
    kb_id CHAR(36) NOT NULL,
    doc_id CHAR(36) NOT NULL,
    chunk_id VARCHAR(255) NOT NULL COMMENT 'Qdrant point ID',
    user_id BIGINT UNSIGNED NOT NULL,
    text TEXT NOT NULL COMMENT 'Annotation content',
    position_start INT NOT NULL COMMENT 'Character position start in chunk',
    position_end INT NOT NULL COMMENT 'Character position end in chunk',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_at DATETIME NULL COMMENT 'Soft delete timestamp',

    FOREIGN KEY (kb_id) REFERENCES ap_knowledge_bases(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES ap_users(id) ON DELETE CASCADE,

    INDEX idx_doc_chunk (doc_id, chunk_id),
    INDEX idx_kb_id (kb_id),
    INDEX idx_user_id (user_id),
    INDEX idx_created_at (created_at),
    INDEX idx_deleted_at (deleted_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
  COMMENT='Collaborative annotations on PDF chunks';

-- ==========================================================================
-- Annotation Threads (Replies)
-- ==========================================================================
CREATE TABLE IF NOT EXISTS ap_annotation_threads (
    id CHAR(36) PRIMARY KEY COMMENT 'UUID',
    annotation_id CHAR(36) NOT NULL,
    user_id BIGINT UNSIGNED NOT NULL,
    text TEXT NOT NULL COMMENT 'Reply content',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at DATETIME NULL COMMENT 'Soft delete timestamp',

    FOREIGN KEY (annotation_id) REFERENCES ap_annotations(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES ap_users(id) ON DELETE CASCADE,

    INDEX idx_annotation_id (annotation_id),
    INDEX idx_user_id (user_id),
    INDEX idx_created_at (created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
  COMMENT='Threaded replies to annotations';

-- ==========================================================================
-- Migration applied successfully
-- ==========================================================================
