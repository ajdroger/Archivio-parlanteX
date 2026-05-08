# Changelog

All notable changes to Archivio Parlante will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned (Fase 6 - Remaining)
- Knowledge Graph-based RAG for entity extraction and relationship mapping (Fase 6.1)
- Advanced hallucination detection with confidence scoring (Fase 6.2)
- Real-time collaborative document annotation (Fase 6.4)

---

## [0.6.3] - 2026-05-08

### Added - Fase 6.3: Multi-tenant Workspace Isolation

- **Multi-Tenant Architecture**:
  - Workspace-level isolation on top of KB-level isolation
  - Three-tier access model: Workspace → Knowledge Base → User
  - Role-based access control (admin, member, viewer)
  - Permission hierarchy: Admin > Write > Read

- **Database Schema** (Migration 010):
  - `ap_workspaces`: Workspace management (id, name, owner_user_id)
  - `ap_workspace_members`: User-workspace associations with roles
  - `ap_kb_permissions`: Fine-grained KB access control (user-level + workspace-level)
  - `ap_permission_audit`: Audit trail for permission changes
  - Added `workspace_id` foreign key to `ap_knowledge_bases`
  - Performance indexes: `idx_kb_workspace_user`, `idx_permission_kb_lookup`

- **Rust Engine Enhancements**:
  - MySQL connection pool integration (sqlx 0.8)
  - `KbAccessMiddleware`: Async permission checks with Redis cache (5-min TTL)
  - 4-tier permission resolution:
    1. Direct user permission
    2. Workspace permission (via membership)
    3. KB owner (implicit admin)
    4. Workspace admin (implicit admin on all workspace KBs)
  - Permission check latency target: <50ms p95 (cached)

- **PHP Gateway API** (9 new endpoints):
  - `GET /api/workspaces`: List user workspaces
  - `POST /api/workspaces`: Create workspace
  - `GET /api/workspaces/{id}`: Get workspace details
  - `DELETE /api/workspaces/{id}`: Delete workspace (admin only)
  - `GET /api/workspaces/{id}/members`: List workspace members
  - `POST /api/workspaces/{id}/members`: Add member (admin only)
  - `DELETE /api/workspaces/{id}/members/{userId}`: Remove member
  - `PATCH /api/workspaces/{id}/members/{userId}`: Update member role
  - `WorkspaceService`: CRUD operations for workspaces
  - `WorkspaceController`: Route handlers with role validation

- **Frontend Components**:
  - `WorkspaceSwitcher`: Dropdown component for workspace selection
  - Integrated in MainLayout sidebar
  - Displays member count, KB count, admin badge
  - Zustand state management for `currentWorkspace`

- **Security & Testing**:
  - 100 security test scenarios for permission matrix
  - Integration test suite: 5 end-to-end scenarios
  - k6 load test: 100 concurrent users
  - No permission bypass vulnerabilities detected

### Changed
- `AppState`: Added `db_pool` field for MySQL access
- Main router: KB routes now enforce permission checks via middleware
- Frontend layout: Added workspace selector above KB selector

### Security
- Workspace isolation prevents cross-tenant data access
- Permission checks cached in Redis to mitigate DoS attacks
- SQL injection prevented via parameterized queries
- Cascade DELETE on workspace removal automatically revokes permissions

### Performance
- Permission check: <50ms p95 (cached)
- Redis cache hit rate: >90% target
- MySQL connection pool: 20 max connections
- Load test: 100 concurrent users, <1% error rate

---

## [0.5.0] - 2026-05-07

### Added - Fase 5: Testing, Benchmark & Hardening
- **Security Audit**: Comprehensive OWASP ASVS Level 2 audit (95% compliance)
  - Rust Engine: Constant-time auth, rate limiting, security headers, request validation
  - Python Worker: Path traversal protection, input sanitization, subprocess hardening
  - PHP Gateway: SQL injection prevention (100% prepared statements), CSRF protection, session security
- **Comprehensive Test Suite**:
  - Unit tests: Rust (cargo test), Python (pytest), PHP (PHPUnit), Frontend (vitest)
  - E2E tests: Playwright tests for chat, documents, comparison flows
  - Benchmark suite: Ingest, query, hallucination, concurrent load tests
- **Load Testing**: k6 scripts for load (50 VU), stress (100→500 VU), spike (0→200 VU) testing
- **Observability Stack**:
  - Prometheus metrics collection (port 9090)
  - Grafana dashboards (port 3001): Overview + RAG Pipeline metrics
  - Node Exporter for host metrics
  - cAdvisor for container metrics
- **CI/CD Enhancements**:
  - Security gates: `cargo audit`, `pip-audit`, `composer audit`, `npm audit`
  - Build fails on HIGH/CRITICAL CVEs
  - CI status badge in README
- **Documentation**:
  - Security audit report: `docs/SECURITY_AUDIT_FASE_5.md`
  - Runbook: `docs/RUNBOOK.md`
  - This CHANGELOG

### Changed
- Updated README badge status: Fase 4 → Fase 5 ✅
- Enhanced Makefile with observability and load testing commands

### Security
- **CRITICAL**: Fixed path traversal vulnerability in Python Worker OCR temp file creation
- Upgraded `pip` from 25.2 → 26.1.1 (fixed 4 CVEs: CVE-2025-8869, CVE-2026-1703, CVE-2026-3219, CVE-2026-6357)
- All security audits pass with 0 HIGH/CRITICAL vulnerabilities

---

## [0.4.0] - 2026-05-06

### Added - Fase 4: Frontend Multi-Contract UI
- React 18 + Vite + TypeScript frontend (port 5173)
- TailwindCSS + shadcn/ui component library
- Chat interface with RAG query + streaming responses
- Multi-contract comparison view (side-by-side diff)
- Document management (upload, list, delete)
- LLM provider selector (Ollama + 12 cloud providers)
- Zustand state management
- react-query for data fetching
- Bundle size: 146KB gzipped

### Changed
- Updated docker-compose to expose frontend on port 5173

---

## [0.3.0] - 2026-05-05

### Added - Fase 3: Multi-Provider LLM & Quality Gates
- Multi-provider LLM support: Ollama (local) + 12 cloud providers (Claude, Gemini, GPT, DeepSeek, Qwen, Moonshot, Zhipu, Mistral, Groq, OpenRouter, Together, Fireworks)
- Runtime-switchable providers via API
- Cost budget guard (daily/monthly limits)
- Zero-cost default: all cloud providers opt-in only
- PHP Gateway security hardening (JWT, CSRF, rate limiting)

---

## [0.2.0] - 2026-05-04

### Added - Fase 2: Python AI Worker & Enhanced RAG
- Python FastAPI worker (port 8091)
- Multi-strategy PDF parsing: PyMuPDF, pdfplumber, unstructured, OCR (Tesseract)
- BGE-reranker-v2-m3 cross-encoder
- Contextual retrieval (Anthropic technique)
- Knowledge graph extraction (spaCy + NetworkX)
- Self-RAG with citation enforcement

---

## [0.1.0] - 2026-05-03

### Added - Fase 1: Rust Core Engine & Basic RAG
- Rust Axum + Tokio core engine (port 8090)
- Hybrid search: Dense (cosine) + Sparse (BM25) with Reciprocal Rank Fusion
- Qdrant vector database integration
- Ollama LLM integration (qwen2.5:7b default)
- MySQL 8 database (`archivio_parlante_x`)
- Redis cache and rate limiting
- Docker Compose orchestration (7 services)
- Basic authentication and authorization
- Health check endpoints

---

## [0.0.1] - 2026-05-01

### Added - Fase 0: Project Scaffolding
- Project structure and CLAUDE.md guidelines
- Repository initialization
- .gitignore and .editorconfig
- MIT License
- Initial README

---

[Unreleased]: https://github.com/ajdroger/Archivio-parlanteX/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/ajdroger/Archivio-parlanteX/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/ajdroger/Archivio-parlanteX/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/ajdroger/Archivio-parlanteX/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/ajdroger/Archivio-parlanteX/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/ajdroger/Archivio-parlanteX/compare/v0.0.1...v0.1.0
[0.0.1]: https://github.com/ajdroger/Archivio-parlanteX/releases/tag/v0.0.1
