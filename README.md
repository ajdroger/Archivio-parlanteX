# 🏛️ Archivio Parlante

**Sistema RAG enterprise per analisi forense di contratti aziendali italiani con zero allucinazioni**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Stack: Rust + Python + PHP](https://img.shields.io/badge/Stack-Rust%20%2B%20Python%20%2B%20PHP-blue)](https://github.com)
[![Status: Fase 1.3 ✅](https://img.shields.io/badge/Status-Fase%201.3%20%E2%9C%85-green)](./CHANGELOG.md)

> **📍 Status Progetto**: Fase 1.3 (Ingestion Pipeline End-to-End) completata — pipeline completo parse→chunk→contextualize→embed→store Qdrant, validazione MIME, batched embeddings, per-KB collections. Prossimo: Fase 1.4 (Hybrid Search + Reranker).

---

## 🎯 Obiettivo

Archivio Parlante è una piattaforma greenfield per l'analisi intelligente di contratti aziendali complessi, destinata a **enti istituzionali ad alto rischio reputazionale**. Il sistema combina:

- **Zero allucinazioni** tramite Hybrid Search + Reranker + Contextual Retrieval + Self-RAG + Knowledge Graph legale
- **Confronto multi-contratto** in parallelo per analisi comparative forensi
- **Multi-provider LLM** runtime-switchable: Ollama locale (privacy totale) + cloud premium opt-in (Claude, Gemini, DeepSeek, GPT, ecc.)
- **Zero-cost di default**: stack completamente open-source, nessuna API key obbligatoria

---

## 🏗️ Architettura

```
React 18 SPA (Vite + TS + TailwindCSS)
         ↓
PHP 8.2 Gateway (Slim 4) — Auth, Rate Limiting, Proxy
         ↓
🦀 Rust Core Engine (Axum + Tokio) — Chunking, Hybrid Search, RAG, Multi-Contract
    ↓     ↓      ↓
Qdrant  Ollama  🐍 Python AI Worker (FastAPI) — PDF Parsing, OCR, BGE Reranker, Knowledge Graph
         ↓
MySQL 8 + Redis 7
```

**7 microservizi orchestrati via Docker Compose:**
- `php-gateway`: API Gateway e autenticazione (porta 8080)
- `rust-engine`: Core RAG engine (porta 8090)
- `python-worker`: AI Worker per parsing e ML (porta 8091)
- `qdrant`: Vector database (porta 6333)
- `ollama`: LLM locale (porta 11434)
- `mysql`: Database relazionale (`archivio_parlante_x`)
- `redis`: Cache e rate limiting

---

## 🚀 Quick Start

### Prerequisiti

- **Docker Desktop** (Windows/Mac) o Docker Engine + Docker Compose (Linux)
- **Git** 2.30+
- **Hardware minimo**: 16 GB RAM, 50 GB storage libero
- **Hardware raccomandato**: 32 GB RAM, GPU NVIDIA 8+ GB VRAM (per modelli locali)

### Setup in 5 minuti

```bash
# 1. Clone repository
git clone https://github.com/<your-org>/archivio-parlante.git
cd archivio-parlante

# 2. Copia e configura variabili d'ambiente
cp .env.example .env
# Edita .env per impostare segreti (JWT_SECRET, RUST_ENGINE_INTERNAL_TOKEN)

# 3. Avvia stack completo
make setup    # build immagini + install dipendenze
make up       # docker compose up -d

# 4. Scarica modelli LLM locali (opzionale, ~5 GB)
make ollama-pull

# 5. Esegui migration database
make migrate

# 6. Verifica salute sistema
make health   # curl ai 4 /health endpoint
```

Accedi all'UI: **http://localhost:8080**

---

## 🛠️ Comandi utili

| Comando | Descrizione |
|---|---|
| `make up` | Avvia tutti i container |
| `make down` | Ferma tutti i container |
| `make logs` | Visualizza log aggregati |
| `make rebuild-rust` | Rebuild solo Rust engine |
| `make rebuild-python` | Rebuild solo Python worker |
| `make test-all` | Esegue suite completa di test |
| `make bench` | Esegue benchmark (ingest, query, hallucination, concurrent) |
| `make mysql-shell` | Connessione shell MySQL |
| `make backup-db` | Backup database |

---

## 📚 Documentazione

- **[Piano di Implementazione](./implementation_plan.md)** — documento maestro con tutte le fasi
- **[CLAUDE.md](./.claude/CLAUDE.md)** — istruzioni project-level per Claude Code
- **[Architettura](./docs/ARCHITECTURE.md)** — diagrammi e decisioni tecniche *(da creare)*
- **[Runbook](./docs/RUNBOOK.md)** — troubleshooting operativo *(da creare)*
- **[ADR](./docs/ADR/)** — Architecture Decision Records

---

## 🧪 Testing

```bash
# Test unitari Rust
make test-rust        # cargo test --release

# Test unitari Python
make test-python      # pytest --cov

# Test unitari PHP
make test-php         # composer test

# Test frontend
make test-frontend    # vitest run + playwright test

# Test end-to-end
make test-e2e         # playwright test (richiede stack up)

# Suite completa
make test-all
```

**Coverage minima richiesta**: 80% per Rust/Python/PHP, 70% per frontend.

---

## 🔒 Sicurezza

- **OWASP ASVS Level 2** compliance
- Input validation su tutti gli endpoint
- JWT con rotazione chiavi, rate limiting Redis
- Prepared statements, no SQL injection
- Container security: Trivy scan, non-root user
- Dependency audit: `cargo audit`, `pip-audit`, `composer audit`, `npm audit`
- TLS obbligatorio in produzione

Report audit di sicurezza per ogni fase in `docs/SECURITY_AUDIT_*.md`.

---

## 🧠 Modelli LLM

### Locale (Ollama) — Default Zero-Cost

- **Chat principale**: `qwen2.5:7b-instruct-q4_K_M` (~4.7 GB VRAM)
- **Lightweight**: `qwen2.5:3b-instruct-q4_K_M` (per task massivi)
- **Embedding**: `nomic-embed-text` (768 dim)

### Cloud Premium — Opt-In

Disabilitati di default. Attivabili da admin UI inserendo API key:
- Anthropic Claude (Opus 4.7, Sonnet 4.6, Haiku 4.5)
- Google Gemini (2.5 Pro, 2.5 Flash)
- OpenAI (GPT-5, o3)
- DeepSeek (V3, R1)
- Altri: Qwen, Moonshot, Zhipu, Mistral, Groq, OpenRouter, Together, Fireworks

**Budget guard**: `DAILY_COST_BUDGET_EUR=0.00` di default → richiede incremento esplicito.

---

## 📊 KPI e Benchmark

Target di performance (vedi `benchmarks/`):
- **Ingestion**: > 100 pagine PDF/minuto
- **Query RAG**: < 500 ms (p95) con modello locale
- **Accuracy**: Recall@10 > 95%, Precision@5 > 90%
- **Hallucination rate**: < 1% (verificato via Self-RAG evaluator)
- **Multi-contract**: 50+ contratti paralleli in < 2 secondi

---

## 🤝 Contributi

Progetto interno. Per modifiche:
1. Crea feature branch da `develop`: `git checkout -b feature/fase-N-descrizione`
2. Segui il [ciclo 8-step](./implementation_plan.md#08--ciclo-di-lavoro-obbligatorio-per-ogni-fase--8-step-senior-workflow)
3. Apri PR su `develop` con checklist completa
4. Richiedi review, attendi CI verde
5. Merge solo dopo approvazione

**Conventional Commits**: `feat|fix|refactor|perf|docs|test|chore|ci|build|security`

---

## 📄 Licenza

[MIT License](./LICENSE) — Copyright (c) 2025 Archivio Parlante Team

---

## 🙏 Credits

Costruito con ❤️ per enti istituzionali che richiedono massima precisione e zero allucinazioni nell'analisi contrattuale.

**Stack tecnologico**:
- [Rust](https://www.rust-lang.org/) 🦀
- [Python](https://www.python.org/) 🐍
- [PHP](https://www.php.net/) 🐘
- [React](https://react.dev/) ⚛️
- [Qdrant](https://qdrant.tech/)
- [Ollama](https://ollama.com/)
- [Axum](https://github.com/tokio-rs/axum)
- [FastAPI](https://fastapi.tiangolo.com/)
- [Slim Framework](https://www.slimframework.com/)
