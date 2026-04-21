-- ============================================================================
-- Archivio Parlante — Initial Database Schema
-- ============================================================================
-- Database: archivio_parlante_x (già creato via phpMyAdmin)
-- Charset: utf8mb4_unicode_ci
-- Engine: InnoDB con foreign keys

USE archivio_parlante_x;

-- ==========================================================================
-- Users & Authentication
-- ==========================================================================
CREATE TABLE IF NOT EXISTS ap_users (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    full_name VARCHAR(255) NOT NULL,
    role ENUM('admin', 'user', 'viewer') NOT NULL DEFAULT 'user',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    last_login_at DATETIME NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_at DATETIME NULL,
    INDEX idx_email (email),
    INDEX idx_role (role),
    INDEX idx_deleted_at (deleted_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
  COMMENT='Utenti sistema con autenticazione JWT';

-- ==========================================================================
-- Knowledge Bases (Workspace per gruppi di documenti)
-- ==========================================================================
CREATE TABLE IF NOT EXISTS ap_knowledge_bases (
    id CHAR(36) PRIMARY KEY,  -- UUID
    name VARCHAR(255) NOT NULL,
    description TEXT NULL,
    owner_user_id BIGINT UNSIGNED NOT NULL,
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    settings JSON NULL COMMENT 'Chunking params, retrieval config, etc.',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_at DATETIME NULL,
    FOREIGN KEY (owner_user_id) REFERENCES ap_users(id) ON DELETE CASCADE,
    INDEX idx_owner (owner_user_id),
    INDEX idx_deleted_at (deleted_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
  COMMENT='Knowledge bases (workspace per documenti)';

-- ==========================================================================
-- Documents (Contratti caricati)
-- ==========================================================================
CREATE TABLE IF NOT EXISTS ap_documents (
    id CHAR(36) PRIMARY KEY,  -- UUID
    kb_id CHAR(36) NOT NULL,
    filename VARCHAR(512) NOT NULL,
    file_path VARCHAR(1024) NOT NULL,
    file_size_bytes BIGINT UNSIGNED NOT NULL,
    mime_type VARCHAR(127) NOT NULL,
    status ENUM('uploading', 'parsing', 'chunking', 'embedding', 'indexed', 'failed') NOT NULL DEFAULT 'uploading',
    page_count INT UNSIGNED NULL,
    chunks_count INT UNSIGNED NULL DEFAULT 0,
    metadata JSON NULL COMMENT 'Parsed metadata: title, author, contract_type, parties, dates, etc.',
    error_message TEXT NULL,
    indexed_at DATETIME NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_at DATETIME NULL,
    FOREIGN KEY (kb_id) REFERENCES ap_knowledge_bases(id) ON DELETE CASCADE,
    INDEX idx_kb_id (kb_id),
    INDEX idx_status (status),
    INDEX idx_indexed_at (indexed_at),
    INDEX idx_deleted_at (deleted_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
  COMMENT='Documenti caricati (PDF, DOCX, etc.)';

-- ==========================================================================
-- Ingest Jobs (Tracking ingestion pipeline)
-- ==========================================================================
CREATE TABLE IF NOT EXISTS ap_ingest_jobs (
    id CHAR(36) PRIMARY KEY,  -- UUID
    doc_id CHAR(36) NOT NULL,
    kb_id CHAR(36) NOT NULL,
    status ENUM('queued', 'parsing', 'chunking', 'embedding', 'kg_extraction', 'done', 'failed') NOT NULL DEFAULT 'queued',
    progress TINYINT UNSIGNED NOT NULL DEFAULT 0 COMMENT 'Percentage 0-100',
    current_step VARCHAR(127) NULL,
    error_msg TEXT NULL,
    processing_ms INT UNSIGNED NULL COMMENT 'Total processing time in milliseconds',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (doc_id) REFERENCES ap_documents(id) ON DELETE CASCADE,
    FOREIGN KEY (kb_id) REFERENCES ap_knowledge_bases(id) ON DELETE CASCADE,
    INDEX idx_doc_id (doc_id),
    INDEX idx_status (status)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
  COMMENT='Job tracking per ingestion pipeline';

-- ==========================================================================
-- Chat Messages (Conversazioni RAG)
-- ==========================================================================
CREATE TABLE IF NOT EXISTS ap_chat_messages (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    session_id CHAR(36) NOT NULL,  -- UUID sessione chat
    kb_id CHAR(36) NOT NULL,
    user_id BIGINT UNSIGNED NOT NULL,
    role ENUM('user', 'assistant', 'system') NOT NULL,
    content TEXT NOT NULL,
    citations JSON NULL COMMENT 'Array di {doc_id, chunk_idx, text_quote, score}',
    llm_provider VARCHAR(63) NULL COMMENT 'ollama, anthropic, openai, etc.',
    llm_model VARCHAR(127) NULL COMMENT 'qwen2.5:7b, claude-opus-4-7, etc.',
    tokens_used INT UNSIGNED NULL,
    cost_eur DECIMAL(10, 6) NULL COMMENT 'Costo query in EUR',
    latency_ms INT UNSIGNED NULL COMMENT 'Latenza query in milliseconds',
    confidence DECIMAL(3, 2) NULL COMMENT 'Self-RAG confidence score 0.00-1.00',
    verified BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Self-RAG validation passed',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at DATETIME NULL,
    FOREIGN KEY (kb_id) REFERENCES ap_knowledge_bases(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES ap_users(id) ON DELETE CASCADE,
    INDEX idx_session_id (session_id),
    INDEX idx_kb_id (kb_id),
    INDEX idx_user_id (user_id),
    INDEX idx_created_at (created_at),
    INDEX idx_deleted_at (deleted_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
  COMMENT='Messaggi chat RAG con citations e metrics';

-- ==========================================================================
-- Knowledge Graph Nodes (Entità Legali)
-- ==========================================================================
CREATE TABLE IF NOT EXISTS ap_graph_nodes (
    id CHAR(36) PRIMARY KEY,  -- UUID
    kb_id CHAR(36) NOT NULL,
    doc_id CHAR(36) NOT NULL,
    entity_type ENUM('PARTY', 'DATE', 'AMOUNT', 'CLAUSE', 'JURISDICTION', 'PENALTY', 'OTHER') NOT NULL,
    label VARCHAR(512) NOT NULL COMMENT 'Nome entità estratta',
    properties JSON NULL COMMENT 'Metadata aggiuntivi',
    chunk_idx INT UNSIGNED NULL COMMENT 'Indice chunk da cui estratta',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (kb_id) REFERENCES ap_knowledge_bases(id) ON DELETE CASCADE,
    FOREIGN KEY (doc_id) REFERENCES ap_documents(id) ON DELETE CASCADE,
    INDEX idx_kb_id (kb_id),
    INDEX idx_doc_id (doc_id),
    INDEX idx_entity_type (entity_type),
    INDEX idx_label (label(255))
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
  COMMENT='Knowledge graph nodes (entità legali estratte)';

-- ==========================================================================
-- Knowledge Graph Edges (Relazioni tra Entità)
-- ==========================================================================
CREATE TABLE IF NOT EXISTS ap_graph_edges (
    id CHAR(36) PRIMARY KEY,  -- UUID
    kb_id CHAR(36) NOT NULL,
    source_node_id CHAR(36) NOT NULL,
    target_node_id CHAR(36) NOT NULL,
    relation_type VARCHAR(127) NOT NULL COMMENT 'has_penalty, governed_by, signed_by, etc.',
    properties JSON NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (kb_id) REFERENCES ap_knowledge_bases(id) ON DELETE CASCADE,
    FOREIGN KEY (source_node_id) REFERENCES ap_graph_nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (target_node_id) REFERENCES ap_graph_nodes(id) ON DELETE CASCADE,
    INDEX idx_kb_id (kb_id),
    INDEX idx_source (source_node_id),
    INDEX idx_target (target_node_id),
    INDEX idx_relation (relation_type)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
  COMMENT='Knowledge graph edges (relazioni entità)';

-- ==========================================================================
-- LLM Providers (Config multi-provider)
-- ==========================================================================
CREATE TABLE IF NOT EXISTS ap_llm_providers (
    id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    provider_name VARCHAR(63) NOT NULL UNIQUE COMMENT 'ollama, anthropic, openai, deepseek, etc.',
    display_name VARCHAR(127) NOT NULL,
    is_enabled BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Admin abilita provider',
    api_key_encrypted VARCHAR(512) NULL COMMENT 'AES-256 encrypted API key',
    base_url VARCHAR(512) NULL COMMENT 'Custom API endpoint se necessario',
    models_available JSON NULL COMMENT 'Lista modelli disponibili',
    daily_cost_eur DECIMAL(10, 6) NOT NULL DEFAULT 0.000000 COMMENT 'Costo accumulato oggi',
    monthly_cost_eur DECIMAL(10, 6) NOT NULL DEFAULT 0.000000 COMMENT 'Costo accumulato mese',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
  COMMENT='LLM providers configuration e cost tracking';

-- ==========================================================================
-- Contract Analyses (Salvate analisi multi-contratto)
-- ==========================================================================
CREATE TABLE IF NOT EXISTS ap_contract_analyses (
    id CHAR(36) PRIMARY KEY,  -- UUID
    kb_id CHAR(36) NOT NULL,
    user_id BIGINT UNSIGNED NOT NULL,
    doc_ids JSON NOT NULL COMMENT 'Array di UUID documenti analizzati',
    question TEXT NOT NULL COMMENT 'Domanda originale',
    answer_json JSON NOT NULL COMMENT 'Risposta strutturata con comparazione',
    llm_provider VARCHAR(63) NULL,
    llm_model VARCHAR(127) NULL,
    processing_ms INT UNSIGNED NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (kb_id) REFERENCES ap_knowledge_bases(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES ap_users(id) ON DELETE CASCADE,
    INDEX idx_kb_id (kb_id),
    INDEX idx_user_id (user_id),
    INDEX idx_created_at (created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
  COMMENT='Analisi multi-contratto salvate';

-- ==========================================================================
-- Initial Data
-- ==========================================================================
-- Admin user di default (password: admin123 — CHANGE IN PRODUCTION!)
-- Hash generato con password_hash('admin123', PASSWORD_BCRYPT)
INSERT INTO ap_users (email, password_hash, full_name, role, is_active)
VALUES ('admin@archivioParlante.local', '$2y$10$92IXUNpkjO0rOQ5byMi.Ye4oKoEa3Ro9llC/.og/at2.uheWG/igi', 'Admin Default', 'admin', TRUE)
ON DUPLICATE KEY UPDATE email=email;

-- Provider Ollama (default abilitato, zero-cost)
INSERT INTO ap_llm_providers (provider_name, display_name, is_enabled, base_url)
VALUES ('ollama', 'Ollama (Locale)', TRUE, 'http://ollama:11434')
ON DUPLICATE KEY UPDATE provider_name=provider_name;

-- Provider cloud (disabilitati di default)
INSERT INTO ap_llm_providers (provider_name, display_name, is_enabled)
VALUES
    ('anthropic', 'Anthropic Claude', FALSE),
    ('google', 'Google Gemini', FALSE),
    ('openai', 'OpenAI', FALSE),
    ('deepseek', 'DeepSeek', FALSE),
    ('qwen', 'Qwen (Alibaba)', FALSE),
    ('moonshot', 'Moonshot Kimi', FALSE),
    ('zhipu', 'Zhipu GLM', FALSE),
    ('mistral', 'Mistral AI', FALSE),
    ('groq', 'Groq', FALSE),
    ('openrouter', 'OpenRouter', FALSE),
    ('together', 'Together.ai', FALSE),
    ('fireworks', 'Fireworks.ai', FALSE)
ON DUPLICATE KEY UPDATE provider_name=provider_name;

-- ============================================================================
-- Finito schema iniziale
-- ============================================================================
