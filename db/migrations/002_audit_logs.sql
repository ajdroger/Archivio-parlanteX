-- Migration: 002_audit_logs.sql
-- Purpose: Create ap_audit_logs table for structured security event tracking
-- Phase: Fase 3.3 - Audit Logging & Configurable Rate Limiting
-- Date: 2026-04-22

CREATE TABLE IF NOT EXISTS ap_audit_logs (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    user_id BIGINT UNSIGNED NULL COMMENT 'NULL for failed login or anonymous events',
    event_type ENUM(
        'register_success',
        'login_success',
        'login_failed',
        'logout',
        'token_refresh',
        'unauthorized_access',
        'rate_limit_exceeded',
        'password_change'
    ) NOT NULL COMMENT 'Type of security event',
    ip_address VARCHAR(45) NOT NULL COMMENT 'IPv4 (15 chars) or IPv6 (45 chars)',
    user_agent VARCHAR(512) NULL COMMENT 'Browser User-Agent string',
    request_path VARCHAR(255) NOT NULL COMMENT 'API endpoint path',
    request_method VARCHAR(10) NOT NULL COMMENT 'HTTP method (GET, POST, etc.)',
    status_code SMALLINT UNSIGNED NOT NULL COMMENT 'HTTP status code (200, 401, 429, etc.)',
    event_data JSON NULL COMMENT 'Additional context: error reason, attempt count, etc.',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Event timestamp',

    -- Foreign key to ap_users (ON DELETE SET NULL to preserve audit trail even if user deleted)
    CONSTRAINT fk_audit_log_user
        FOREIGN KEY (user_id)
        REFERENCES ap_users(id)
        ON DELETE SET NULL,

    -- Indexes for efficient querying
    INDEX idx_user_id (user_id),
    INDEX idx_event_type (event_type),
    INDEX idx_ip_address (ip_address),
    INDEX idx_created_at (created_at),
    INDEX idx_composite (user_id, event_type, created_at)

) ENGINE=InnoDB
  DEFAULT CHARSET=utf8mb4
  COLLATE=utf8mb4_unicode_ci
  COMMENT='Security audit log for authentication and authorization events';
