-- Migration: 003_proxy_audit_events.sql
-- Purpose: Add proxy endpoint event types to audit logs
-- Phase: Fase 3.4 - Rust Engine Proxy Routes
-- Date: 2026-04-23

-- Alter ap_audit_logs.event_type ENUM to include proxy events
ALTER TABLE ap_audit_logs
MODIFY COLUMN event_type ENUM(
    'register_success',
    'login_success',
    'login_failed',
    'logout',
    'token_refresh',
    'unauthorized_access',
    'rate_limit_exceeded',
    'password_change',
    -- Fase 3.4: Proxy endpoint events
    'query_success',
    'query_failed',
    'ingest_success',
    'ingest_failed',
    'compare_success',
    'compare_failed'
) NOT NULL COMMENT 'Type of security or operational event';
