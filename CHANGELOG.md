# Changelog

All notable changes to Archivio Parlante will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- Knowledge Graph-based RAG for entity extraction and relationship mapping
- Advanced hallucination detection with confidence scoring
- Multi-tenant architecture with workspace isolation
- Real-time collaborative document annotation

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
