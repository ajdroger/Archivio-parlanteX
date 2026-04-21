# Changelog

Tutte le modifiche significative al progetto saranno documentate in questo file.

Il formato è basato su [Keep a Changelog](https://keepachangelog.com/it/1.0.0/),
e questo progetto aderisce al [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fase 1.3 — Ingestion Pipeline End-to-End (2026-04-21)
- Document models (`src/models/document.rs`)
  - Document, IngestRequest, IngestResponse structs
  - Timestamp auto-generation with chrono::Utc
- Ingest route handler (`src/routes/ingest.rs`)
  - Complete pipeline: validate → parse → chunk → contextualize → embed → store
  - AppState with config + llm_registry + python_worker
  - Validation: doc_id/kb_id non-empty, MIME type whitelist (PDF/TXT/DOC/DOCX)
  - Python worker integration for document parsing
  - Semantic chunking with configurable params
  - Contextual enrichment with parallel LLM calls
  - Embedding generation in batches (max_concurrent_embeddings)
  - Qdrant storage with per-KB collections (ap_kb_{kb_id})
  - Error handling graceful, structured logging
  - Processing time tracking
- Qdrant integration
  - Per-KB collection creation (dense 768 cosine + sparse BM25 ready)
  - ChunkInsert with embeddings and metadata
  - Batch upsert operation
- Main.rs updates
  - AppState from routes::ingest module
  - POST /ingest → handle_ingest (501 placeholder removed)
  - Health endpoint enhanced with provider list
- E2E ingestion tests (`tests/ingestion_e2e.rs`)
  - Full flow test with sample contract
  - Qdrant verification (chunks exist, correct doc_id)
  - Validation error tests (empty doc_id, invalid MIME)
  - Marked #[ignore], requires stack up
- Documentation: FASE_1_3_VERIFICATION.md with pipeline diagram

### Fase 1.2 — Semantic Chunker + Contextual Retrieval (2026-04-21)
- Chunk model (`src/models/chunk.rs`)
  - Struct Chunk con UUID, metadata JSON, offsets, token count
  - Support per contextual_text enrichment
  - Helper methods: `embedding_text()`, `is_contextualized()`
- Tokenizer wrapper (`src/utils/tokenizer.rs`)
  - tiktoken-rs integration con cl100k_base encoding
  - `count_tokens(text)` per conteggio accurato
  - `split_by_token_limit(text, limit)` con preservazione word boundaries
- Semantic chunker (`src/chunker/semantic.rs`)
  - Split per Markdown headers (# ## ###)
  - Split per clausole legali italiane (Art., Articolo, CAPO, Sezione)
  - Split per frasi con regex Unicode-aware
  - Overlap configurabile tra chunk consecutivi (default 15%)
  - Metadata arricchita: section_header, clause_marker, is_clause_start
  - Preservazione ordinamento logico (articoli numerati)
  - Gestione casi edge: testo vuoto, paragrafo enorme, solo headers
- Contextual Retrieval enricher (`src/chunker/contextual.rs`)
  - Implementazione tecnica Anthropic per riduzione errori (-49%)
  - Generazione summary documento se > 8000 tokens
  - Arricchimento parallelo chunks con LLM (max 16 concurrent)
  - Caching con DashMap per evitare ricalcoli
  - Fallback graceful se LLM fallisce (chunk passa senza context)
- Test completi (`tests/chunker_test.rs`)
  - Test contratto italiano (~3000 parole) con articoli numerati
  - Verifica chunk count, token limits, overlap, ordinamento logico
  - Test edge cases: testo corto, vuoto, solo headers
  - Test contextual enrichment (#[ignore], richiede Ollama)
- Chunk demo CLI (`examples/chunk_demo.rs`)
  - Tool interattivo per verifica manuale chunking
  - Statistiche: count, avg/min/max tokens, clause markers
  - Preview chunks con metadata
- Dipendenze aggiunte: tiktoken-rs, regex, dashmap, chrono

### Fase 1.1 — Rust Engine Scaffolding (2026-04-21)
- Configuration system con caricamento da environment (`src/config.rs`)
  - Supporto per Ollama (locale, zero-cost default)
  - Cloud provider API keys opt-in (Anthropic, Google, OpenAI, DeepSeek)
  - Budget guard (`daily_cost_budget_eur`)
  - Parametri chunking e retrieval configurabili
- Sistema errori strutturato (`src/errors.rs`)
  - Enum `AppError` con mapping HTTP status codes
  - Integrazione Axum `IntoResponse`
  - JSON error responses con structured logging
- Multi-provider LLM architecture (`src/providers/`)
  - Trait `LlmProvider` con `async_trait`
  - `OllamaProvider` implementation con rate limiting (Semaphore)
  - `LlmRegistry` per runtime provider switching
  - Tipi condivisi: `ChatRequest`, `ChatResponse`, `Message`, `Usage`
- Client Qdrant wrapper (`src/clients/qdrant.rs`)
  - Hybrid search ready (dense cosine + sparse BM25)
  - Collection management (ensure_collection, upsert_chunks)
  - Search methods: `search_dense`, `search_sparse`
  - Delete by doc_id
- Client Python AI Worker (`src/clients/python_worker.rs`)
  - Document parsing endpoint
  - Contextual retrieval endpoint
  - BGE reranker endpoint
  - Knowledge graph extraction endpoint
- Router Axum con AppState (`src/main.rs`)
  - GET /health (operational con provider list)
  - POST /ingest (501 placeholder, Fase 1.2)
  - POST /query (501 placeholder, Fase 1.3)
  - POST /compare_contracts (501 placeholder, Fase 1.5)
- Integration test Ollama (`tests/ollama_smoke.rs`)
  - Connectivity check
  - Chat completion test
  - Embeddings test
- Library exports (`src/lib.rs`) per integration tests
- Documentazione verifica (`docs/FASE_1_1_VERIFICATION.md`)
- Security audit OWASP ASVS L2 (`docs/SECURITY_AUDIT_FASE_1_1.md`)
  - 1 issue medio identificato (Config Debug redaction)
  - Zero vulnerabilità critiche/alte

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
