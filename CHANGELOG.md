# Changelog

Tutte le modifiche significative al progetto saranno documentate in questo file.

Il formato è basato su [Keep a Changelog](https://keepachangelog.com/it/1.0.0/),
e questo progetto aderisce al [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fase 0 — Setup Infrastruttura Docker Compose (2026-04-21)
- Docker Compose orchestration con 7 servizi (PHP + Rust + Python + Qdrant + Ollama + MySQL + Redis)
- Scaffolding Rust engine (Axum) con GET /health endpoint
- Scaffolding Python worker (FastAPI) con GET /health endpoint
- Migration MySQL schema iniziale (ap_users, ap_knowledge_bases, ap_documents, ap_chat_messages, ap_graph_nodes/edges, ap_llm_providers)
- Makefile con comandi operativi (up, down, logs, health, rebuild, ollama-pull, mysql-shell, backup-db)
- `.env.example` con configurazione completa (zero-cost default, provider cloud opt-in)
- Rete Docker `archivio_net` e volumi persistenti (mysql_data, qdrant_data, ollama_models)

### Fase -1 — Bootstrap Repository & Ricerca OSS (2026-04-21)
- Setup iniziale repository con Git Flow (main + develop)
- Struttura directory vuota per tutti i microservizi
- Ricerca framework RAG OSS (10 candidati analizzati: Verba, Quivr, kotaemon, Danswer, AnythingLLM, Open WebUI, Cheshire Cat, Haystack, RAGFlow, LlamaIndex)
- Ricerca MCP servers e plugin disponibili (Qdrant, Ollama, Docker, MySQL, Rust/Python/PHP LSP)
- Decision Matrix (Ibrido 4.30/5 vs Clone 3.65 vs From-scratch 4.00) — **Opzione B Ibrido confermata**
- ADR 0001 path-build-vs-clone (Ibrido: RAGFlow parser + Open WebUI UI + LlamaIndex chunker + Rust core)
- Skills Claude Code per coding standards e testing checklist

---

<!-- Template per future release:

## [X.Y.Z] - YYYY-MM-DD

### Added
- Nuove funzionalità

### Changed
- Modifiche a funzionalità esistenti

### Deprecated
- Funzionalità deprecate (da rimuovere nelle prossime release)

### Removed
- Funzionalità rimosse

### Fixed
- Bug fix

### Security
- Patch di sicurezza

-->
