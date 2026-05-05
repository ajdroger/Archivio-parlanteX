# 🏛️ Archivio Parlante

**Sistema RAG enterprise per analisi forense di contratti aziendali italiani con zero allucinazioni**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Stack: Rust + Python + PHP](https://img.shields.io/badge/Stack-Rust%20%2B%20Python%20%2B%20PHP-blue)](https://github.com)
[![Status: Fase 4 ✅](https://img.shields.io/badge/Status-Fase%204%20%E2%9C%85-green)](./CHANGELOG.md)

> **📍 Status Progetto**: Fase 4 (Frontend Multi-Contract UI) completata — React 18 + Vite + TypeScript frontend con chat RAG, confronto multi-contratto, gestione documenti, selezione LLM provider. Bundle 146KB gzipped. Prossimo: Fase 5 (Advanced Features - Graph RAG, Hallucination Detection).

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

## ⚛️ Frontend Development

Il frontend è una Single Page Application (SPA) React 18 con Vite, TypeScript e TailwindCSS v4.

### Setup Locale

```bash
cd frontend

# Installa dipendenze
npm install

# Avvia dev server (con HMR)
npm run dev
# UI disponibile su http://localhost:5173

# Build per produzione
npm run build
# Output in dist/ (146KB gzipped)

# Preview build di produzione
npm run preview

# Lint e format
npm run lint
npm run format

# Test TypeScript compilation
npx tsc --noEmit
```

### Variabili d'Ambiente Frontend

Crea `frontend/.env.local` (escluso da git):

```env
# Base URL del backend (default: /api)
VITE_API_BASE_URL=http://localhost:8080/api

# Se backend su porta diversa o dominio diverso
# VITE_API_BASE_URL=https://api.archivioparlante.com/api
```

### Struttura Frontend

```
frontend/
├── src/
│   ├── components/
│   │   ├── auth/           # ProtectedRoute
│   │   ├── chat/           # ChatMessage, ContextViewer
│   │   ├── comparison/     # ContractComparison
│   │   ├── documents/      # DocumentSelector, DocumentUpload
│   │   ├── layout/         # MainLayout
│   │   └── settings/       # ModelSelector
│   ├── pages/              # Dashboard, Documents, Compare, Analytics, Admin, Login
│   ├── store/              # Zustand stores (authStore, appStore)
│   ├── lib/                # API client (Axios)
│   ├── types/              # TypeScript interfaces
│   └── App.tsx             # React Router setup
├── public/                 # Static assets
├── dist/                   # Build output (gitignored)
└── package.json
```

### Stack Frontend

| Libreria | Versione | Uso |
|---|---|---|
| React | 19.2.5 | UI framework |
| Vite | 8.0.10 | Build tool |
| TypeScript | 6.0.2 | Type safety |
| TailwindCSS | 4.2.4 | Styling |
| React Router | 7.14.2 | Routing |
| Zustand | 5.0.12 | State management |
| Axios | 1.15.2 | HTTP client |
| react-markdown | 10.1.0 | Markdown rendering |
| lucide-react | 1.11.0 | Icons |
| @tanstack/react-query | 5.100.5 | Data fetching (future) |

### Comandi npm

| Comando | Descrizione |
|---|---|
| `npm run dev` | Dev server con HMR (porta 5173) |
| `npm run build` | Build produzione (output in dist/) |
| `npm run preview` | Preview build locale |
| `npm run lint` | ESLint check |
| `npm run format` | Prettier format |
| `npm run test` | Unit tests (Vitest) *(TODO)* |
| `npm run test:e2e` | E2E tests (Playwright) *(TODO)* |

### Flusso di Sviluppo

1. **Backend running**: Assicurati che `make up` sia attivo (backend su porta 8080)
2. **Frontend dev**: `cd frontend && npm run dev` (porta 5173)
3. **Proxy dev**: Vite proxya richieste `/api/*` a `http://localhost:8080` (vedi `vite.config.ts`)
4. **Hot Reload**: Modifiche ai componenti si riflettono immediatamente
5. **Build + Test**: `npm run build && npm run preview` per testare build di produzione

### Note Importanti

- **CORS**: Backend deve accettare richieste da `http://localhost:5173` in dev
- **JWT Tokens**: Salvati in `localStorage` (chiavi: `access_token`, `refresh_token`)
- **Theme**: Dark mode di default con palette neon (#00ff9f primary, #0a0f1a background)
- **Responsive**: Breakpoints Tailwind (sm, md, lg, xl) già configurati
- **Accessibility**: Tutti i componenti con aria-labels e keyboard navigation

---

## 🔌 API Endpoints

Tutti gli endpoint richiedono autenticazione JWT (header `Authorization: Bearer <token>`).

### POST /api/query
Query RAG con hybrid search + reranking + LLM response generation.

**Request:**
```json
{
  "kb_id": "contracts_2024",
  "query": "Quali sono le penali previste per inadempimento?",
  "top_k": 10,
  "rerank_top_n": 5
}
```

**Response:**
```json
{
  "answer": "Le penali previste sono...",
  "sources": [
    {
      "doc_id": "contract_001",
      "chunk_id": "abc123",
      "text_quote": "...",
      "confidence": 0.92
    }
  ],
  "verified": true
}
```

### POST /api/ingest
Ingest documento in knowledge base (parsing + chunking + embedding + Qdrant storage).

**Request:**
```json
{
  "doc_id": "contract_001",
  "kb_id": "contracts_2024",
  "file_path": "/shared/uploads/contract_001.pdf",
  "mime_type": "application/pdf"
}
```

**Response:**
```json
{
  "doc_id": "contract_001",
  "chunk_count": 42,
  "processing_time_ms": 3450,
  "status": "indexed"
}
```

### POST /api/compare
Confronto multi-contratto con analisi comparativa.

**Request:**
```json
{
  "kb_id": "contracts_2024",
  "doc_ids": ["contract_001", "contract_002", "contract_003"],
  "comparison_aspects": ["penalties", "payment_terms", "termination_clauses"]
}
```

**Response:**
```json
{
  "comparison_table": "| Aspetto | Contract 001 | Contract 002 | ...",
  "key_differences": ["..."],
  "information_gaps": [],
  "verified": true
}
```

**Protezioni:**
- Rate limiting: 100 req/min per utente
- Audit logging: tutti gli eventi (success/failed) loggati
- Validazione: MIME type whitelist, length limits, business rules

---

## 📚 Documentazione

- **[Piano di Implementazione](./implementation_plan.md)** — documento maestro con tutte le fasi
- **[CLAUDE.md](./.claude/CLAUDE.md)** — istruzioni project-level per Claude Code
- **[Architettura](./docs/ARCHITECTURE.md)** — diagrammi e decisioni tecniche *(da creare)*
- **[Runbook](./docs/RUNBOOK.md)** — troubleshooting operativo *(da creare)*
- **[ADR](./docs/ADR/)** — Architecture Decision Records

---

## 🧪 Testing

### Unit Tests

```bash
# Rust
cd engine-rust && cargo test --release --all-features

# Python
cd engine-python && pytest --cov=app --cov-report=term

# PHP
cd php-gateway && composer test

# Frontend
cd frontend && npm run test
```

### Coverage Reports

```bash
# Rust (requires cargo-tarpaulin)
cargo install cargo-tarpaulin
cd engine-rust && cargo tarpaulin --out Html --output-dir coverage
# View coverage/index.html in browser

# Python
cd engine-python && pytest --cov=app --cov-report=html
# View htmlcov/index.html

# PHP
cd php-gateway && composer test -- --coverage-html coverage/
# View php-gateway/coverage/index.html
```

### E2E Tests (Requires Full Stack)

```bash
# Start all services
docker compose up -d

# Wait for health checks
for i in {1..30}; do 
  curl -f http://localhost:8090/health && break
  sleep 2
done

# Run E2E tests
cd engine-rust && cargo test --test '*_e2e' -- --ignored --nocapture
```

### Quality Gates (per CLAUDE.md §14)

- ✅ **Rust**: 80% coverage minimum
- ✅ **Python**: 80% coverage minimum
- ✅ **PHP**: 80% coverage minimum
- ✅ **Frontend**: 70% coverage minimum
- ✅ **All tests pass** before commit
- ✅ **E2E tests pass** in CI before merge

### CI Pipeline

The GitHub Actions CI pipeline automatically runs:
- Unit tests for all layers (Rust, Python, PHP, Frontend)
- Code coverage measurement (enforces 80% threshold for backend)
- Linting and formatting checks (`cargo clippy`, `ruff`, `phpstan`, `eslint`)
- Security audits (`cargo audit`, `pip-audit`, `composer audit`, `npm audit`)
- E2E tests with full Docker stack
- Coverage reports uploaded to artifacts

**All checks must pass before PR merge.**

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

[MIT License](./LICENSE) — Copyright (c) 2026 Archivio Parlante Team

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
