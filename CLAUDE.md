# 🏛️ Archivio Parlante — CLAUDE.md

> **File project-level instructions per Claude Code.** Leggere SEMPRE all'inizio di una nuova sessione, prima di qualsiasi operazione di file/codice.

---

## 1. Identità del progetto

| Campo | Valore |
|---|---|
| **Nome** | Archivio Parlante |
| **Slug** | `archivio-parlante` |
| **Tipo** | 🟢 Greenfield / from-scratch (NON refactor) |
| **Dominio** | RAG enterprise per analisi forense di contratti aziendali italiani |
| **Stakeholder primari** | Enti istituzionali ad alto rischio reputazionale |
| **Obiettivo qualità** | Zero allucinazioni, massima precisione, confronto multi-contratto in parallelo |
| **Documento maestro** | `PIANO_IMPLEMENTAZIONE_RUST_PYTHON.md` (root) |

**Stato attuale**: directory vuota, da costruire integralmente. Nessun codice preesistente da riutilizzare. Iniziare da Fase -1 del piano.

---

## 2. Stack tecnologico

| Layer | Tecnologia | Note |
|---|---|---|
| **API Gateway** | PHP 8.2+ Slim 4 | Sottile: auth, utenti, sessioni, rate limit, audit, proxy verso Rust |
| **Core Engine** | 🦀 Rust 1.82+ con `axum` + `tokio` | Chunking, hybrid search, RAG, multi-contract, multi-provider LLM |
| **AI Worker** | 🐍 Python 3.11+ con `FastAPI` | Parsing PDF, OCR, BGE reranker, contextual retrieval, knowledge graph |
| **Vector DB** | Qdrant 1.12+ | Dense (cosine) + sparse (BM25) hybrid |
| **LLM locale** | Ollama | Modelli ≤14B per RTX 4070 Laptop 8 GB |
| **LLM cloud (opt-in)** | Anthropic, Google, OpenAI, DeepSeek, Qwen, Moonshot, Zhipu, Mistral, Groq, OpenRouter, Together, Fireworks | Disabilitati di default, attivabili con API key |
| **RDBMS** | MySQL 8.0 | Database: `archivio_parlante_x` |
| **Cache / Rate Limit** | Redis 7 | |
| **Frontend** | React 18 + Vite + TypeScript + TailwindCSS + shadcn/ui + Zustand + react-query | Greenfield from-scratch |
| **Orchestrazione** | Docker Compose | Tutti i 7 servizi (PHP + Rust + Python + Qdrant + Ollama + MySQL + Redis) |

---

## 3. Hardware target di sviluppo

```
MSI Raider GE78HX 13VG
- CPU: Intel i9-13950HX (24 core / 32 thread)
- RAM: 32 GB DDR5
- GPU: NVIDIA RTX 4070 Laptop 8 GB VRAM    ← VINCOLO PRIMARIO
- SSD: NVMe 2 TB (~966 GB liberi)
- OS:  Windows 11 Pro + Docker Desktop + WSL2
```

**Conseguenze sul design**:
- Modelli locali: massimo 14B in Q4 (`qwen2.5:7b` di default, ~4.7 GB).
- Modelli > 14B vanno via API cloud, MAI proporre offload CPU come default.
- `OLLAMA_MODEL_CHAT=qwen2.5:7b-instruct-q4_K_M` come default in `.env`.

---

## 4. Principi operativi non-negoziabili

Questi principi prevalgono in caso di conflitto con qualsiasi prompt successivo.

### 4.1 💰 Zero-Cost / Zero-Budget (vincolante)

- Default del sistema = solo stack gratuito (Ollama + Qdrant + MySQL + Redis OSS).
- API a pagamento = OPT-IN espliciti dell'admin via UI (mai abilitate di default).
- `DAILY_COST_BUDGET_EUR=0.00` di default → provider cloud bloccati finché admin non alza il budget.
- Nessuna dipendenza con licenza commerciale obbligatoria.
- Nessun servizio SaaS obbligatorio.

### 4.2 🔓 Open Source First

Prima di scrivere qualsiasi componente non banale: ricerca su GitHub / Papers With Code / Hugging Face per trovare un progetto OSS che risolva (anche parzialmente) il problema, con licenza compatibile e mantenuto (< 6 mesi dall'ultimo commit, > 500 stars). Se trovato → propone clone/adattamento invece di scrittura from-scratch, producendo matrice di confronto.

### 4.3 🧠 Ruolo Senior Solutions Architect & R&D Lead

Ogni output deve avere:
- Trade-off espliciti (tabella Opzione A/B/C con costo, complessità, rischio, manutenibilità).
- KPI misurabili (latenza p95, throughput, accuracy, recall@10, cost/query).
- Documentazione inline (doc-comment, OpenAPI, commenti SQL).

### 4.4 ❓ Ask-First — gate obbligatori via AskUserQuestion

Fermarsi e chiedere conferma PRIMA di:
- 🚦 Scelta del framework RAG base (Fase -1).
- 🚦 Abilitazione di un provider LLM a pagamento.
- 🚦 Cambio schema DB che richiede migrazione dati.
- 🚦 Introduzione di una dipendenza non MIT/Apache/BSD/MPL-2.0.
- 🚦 Qualsiasi scelta con costo ricorrente > €0/mese.
- 🚦 Spostamento di file > 500 LoC scritti in sessioni precedenti.
- 🚦 Force push, reset hard, modifiche destructive su branch condivisi.

### 4.5 🛠️ Cowork / Claude Code-native

Sfruttare al massimo le funzionalità interne anziché scrivere infrastruttura custom:
- **Skills**: `archivio-parlante-dev`, `archivio-parlante-test` in `.claude/skills/`.
- **MCP**: cerca su `mcp__mcp-registry__search_mcp_registry` prima di scrivere client HTTP custom (Qdrant, Ollama, MySQL, Docker).
- **Plugin**: cerca su `mcp__plugins__search_plugins` per linter, formatter, hook.
- **Subagents (Task tool)**: `rust-engineer`, `python-ml-engineer`, `php-reviewer`, `frontend-react-specialist` per isolamento context.
- **TodoWrite**: obbligatorio per ogni fase con > 1 step.
- **Scheduled Tasks**: per re-indexing, cleanup, backup.

---

## 5. Ciclo di lavoro 8-step OBBLIGATORIO per ogni fase

Una fase è "CLOSED" SOLO quando tutti gli 8 step sono al 100%.

| # | Ruolo | Output |
|---|---|---|
| 1 | Ricerca preliminare (OSS + MCP + plugin) | Note in `docs/` |
| 2 | Senior Tech Lead & PM | TodoWrite con sotto-task della fase |
| 3 | Senior SWE + QA Automation | Codice + test, 100% pass obbligatorio |
| 4 | Senior Performance Engineer | Profilo + ottimizzazioni + re-test 100% |
| 5 | Senior Clean Code Reviewer | Lint/format + dead code rimosso + re-test 100% |
| 6 | Senior Cybersecurity Engineer | Audit OWASP ASVS L2 + report `docs/SECURITY_AUDIT_<fase>.md` |
| 7 | Senior Technical Writer | README/ARCHITECTURE/CHANGELOG/ADR aggiornati |
| 8 | Senior DevOps/Release | Feature branch + commit Conventional + PR + CI verde + merge |

**Gestione interruzioni**: se l'esecuzione si interrompe per qualsiasi motivo (errore, context limit, riavvio), riprendere **ESATTAMENTE** dal punto di interruzione leggendo lo stato git + TodoList. **MAI** ricominciare dall'inizio. **MAI** saltare fasi o task.

---

## 6. Git workflow (vincolante)

### 6.1 Branch strategy

- `main` → solo release stabili. Protetto. Nessun commit diretto.
- `develop` → integrazione continua. Protetto. Merge solo via PR.
- `feature/fase-<N>-<slug>` → lavoro di sviluppo (es. `feature/fase-1-1-rust-scaffolding`).
- `hotfix/<slug>` → fix urgenti su `main`, poi back-merge in `develop`.
- `release/<x.y.z>` → preparazione release.

### 6.2 Sequenza di commit end-of-phase

Eseguire SOLO dopo che gli step 1-7 di §5 sono tutti verdi:

```bash
git status
git add .                              # mai -A senza verifica preventiva di .env e secrets
git commit -m "[feat] fase-<N>: <descrizione>

- Task completati: <lista>
- Test passati: <count>
- Security audit: OK (vedi docs/SECURITY_AUDIT_<fase>.md)
- Documentazione aggiornata: <lista>
"
git pull --rebase origin develop
git push -u origin feature/fase-<N>-<slug>
# Apri PR con il template .github/PULL_REQUEST_TEMPLATE.md
```

### 6.3 Conventional Commits

Tipi ammessi: `feat`, `fix`, `refactor`, `perf`, `docs`, `test`, `chore`, `ci`, `build`, `security`.

### 6.4 Divieti git

- ❌ `git push --force` su `main` o `develop`.
- ❌ `git commit --no-verify` (se hook fallisce, fixa la causa).
- ❌ Commit di `.env`, credenziali, chiavi private, dati sensibili reali.
- ❌ Amend di commit già pushati.
- ❌ Reset hard su branch condivisi.

---

## 7. Coding standards per linguaggio

### 7.1 Rust

- Edition 2021, MSRV 1.82.
- `cargo fmt` + `cargo clippy --all-targets -- -D warnings` puliti.
- Mai `.unwrap()` o `.expect()` in codice di produzione → usa `?` + `anyhow::Context`.
- Async via `tokio` (multi-thread runtime); preferire `tokio::join!` / `futures::stream::buffer_unordered` per parallelismo.
- Errori applicativi via `thiserror` enum; conversione automatica con `#[from]`.
- `tracing::info!/warn!/error!` ovunque, mai `println!` o `eprintln!`.
- Struct pubbliche → doc-comment con `///`.
- Test unitari `#[cfg(test)]` nello stesso file; integration in `tests/`.

### 7.2 Python

- Python 3.11+, type hints obbligatori (`mypy --strict` pulito).
- `ruff format` + `ruff check --fix`.
- Logging via `structlog` (mai `print` in produzione).
- Async via `asyncio` + `httpx.AsyncClient`.
- Pydantic v2 per modelli I/O.
- Mai `shell=True` in `subprocess`; whitelist dei comandi.
- Test con `pytest` + `pytest-asyncio`; fixture in `conftest.py`.

### 7.3 PHP

- PHP 8.2+, strict types (`declare(strict_types=1);` in ogni file).
- PSR-12 coding style, PHPStan level 8 pulito.
- Logger PSR-3 (`monolog/monolog`); mai `echo` di debug, mai `var_dump()`.
- `php-di` per dependency injection.
- Test con PHPUnit; coverage > 80%.
- Mai concat string per query SQL → solo prepared statements via PDO.

### 7.4 TypeScript / React

- TS strict (`strict: true`, `noImplicitAny`, `strictNullChecks`).
- React 18 functional components + hooks; mai class components.
- ESLint + Prettier configurati; `tsc --noEmit` pulito.
- State globale con Zustand; data fetching con react-query.
- Componenti accessibili: aria-labels, focus management, contrasto AAA.
- Mai `localStorage`/`sessionStorage` in produzione (usa httpOnly cookie + Zustand in memory).
- Mai `dangerouslySetInnerHTML` con input utente.

### 7.5 Lingua nei commenti

- **Inglese**: nomi di file, variabili, funzioni, classi, log tecnici, commit messages, commenti su algoritmi/pattern.
- **Italiano**: commenti che descrivono logica di dominio legale/contrattuale, UI, documentazione utente, prompt LLM in italiano forensic, i18n keys italiane.

---

## 8. Convenzioni database

- **Nome database**: `archivio_parlante_x` (vincolante, creato via phpMyAdmin in produzione).
- **Prefix tabelle**: `ap_` (es. `ap_users`, `ap_documents`, `ap_chat_messages`, `ap_llm_providers`).
- **Charset**: `utf8mb4` con collation `utf8mb4_unicode_ci`.
- **Engine**: InnoDB con foreign key dichiarate.
- **Migrations**: in `db/migrations/`, ordinate `001_*`, `002_*`, ... → eseguite automaticamente da MySQL container all'avvio.
- **Naming**: snake_case per colonne, singolare per FK (`user_id`, non `users_id`).
- **Timestamp**: ogni tabella ha `created_at` + `updated_at` (DATETIME, default CURRENT_TIMESTAMP).
- **Soft delete**: `deleted_at NULL` invece di DELETE fisica per `ap_documents`, `ap_chat_messages`, `ap_users`.
- **PII**: in produzione cifrate at-rest via colonne AES-256 (`AES_ENCRYPT` con key in KMS esterno).

---

## 9. Licenze ammesse

| Licenza | Status | Note |
|---|---|---|
| MIT | ✅ Ammessa senza chiedere | Default per il nostro progetto |
| Apache 2.0 | ✅ Ammessa senza chiedere | |
| BSD-2 / BSD-3 | ✅ Ammessa senza chiedere | |
| MPL-2.0 | ✅ Ammessa senza chiedere | |
| ISC | ✅ Ammessa senza chiedere | |
| LGPL-2.1 / LGPL-3 | ⚠️ Ask-First | Solo dynamic linking, mai static |
| GPL-2 / GPL-3 | 🚦 Ask-First obbligatorio | Effetto copyleft sull'intero progetto |
| AGPL | 🚦 Ask-First obbligatorio | Copyleft anche su uso SaaS |
| Commerciale / Proprietary | ❌ Vietata | Conflitto con principio Zero-Cost |
| BUSL / SSPL | ❌ Vietata | Restrizioni d'uso commerciale |
| Sconosciuta / nessuna LICENSE | ❌ Vietata | Non clonare repo senza licenza esplicita |

Prima di aggiungere una dipendenza Claude Code DEVE verificare la licenza (`cargo metadata`, `pip show`, `composer show`, `npm view`).

---

## 10. Comandi essenziali

### 10.1 Bootstrap (una volta)

```bash
git clone <repo> archivio-parlante && cd archivio-parlante
cp .env.example .env             # poi editare con segreti locali
make setup                       # build immagini Docker, install deps PHP/Node
make ollama-pull                 # scarica modelli locali (qwen2.5:7b, qwen2.5:3b, nomic-embed-text)
make migrate                     # esegue migration MySQL
```

### 10.2 Lavoro quotidiano

```bash
make up                          # docker compose up -d
make logs                        # docker compose logs -f
make down                        # docker compose down
make health                      # curl ai 4 /health (PHP + Rust + Python + Qdrant)
make rebuild-rust                # rebuild solo container Rust
make rebuild-python              # rebuild solo container Python
```

### 10.3 Test

```bash
make test-all                    # orchestra tutti i layer
make test-rust                   # cargo test --release
make test-python                 # pytest
make test-php                    # composer test
make test-frontend               # vitest run + playwright test
make test-e2e                    # playwright test (richiede stack up)
```

### 10.4 Quality gates (step 4-6 del ciclo §5)

```bash
make lint                        # cargo fmt+clippy + ruff + phpstan + eslint
make audit-security              # cargo audit + pip-audit + composer audit + npm audit + trivy
make bench                       # benchmark suite (ingest, query, hallucination, concurrent)
```

### 10.5 Database

```bash
make mysql-shell                 # mysql -u root -h localhost archivio_parlante_x
make backup-db                   # dump in backups/db_YYYYMMDD_HHMM.sql.gz
make restore-db FILE=backups/...
```

---

## 11. Struttura directory

```
archivio-parlante/
├── PIANO_IMPLEMENTAZIONE_RUST_PYTHON.md   ← documento maestro (LEGGI SEMPRE)
├── README.md                               ← quick start
├── CHANGELOG.md                            ← Keep a Changelog format
├── LICENSE                                 ← MIT
├── docker-compose.yml
├── Makefile
├── .env.example
├── .gitignore
├── .editorconfig
├── .github/
│   ├── workflows/ci.yml
│   └── PULL_REQUEST_TEMPLATE.md            ← checklist 8-step §5
├── .claude/
│   ├── CLAUDE.md                           ← QUESTO FILE
│   └── skills/
│       ├── archivio-parlante-dev/SKILL.md
│       └── archivio-parlante-test/SKILL.md
├── docs/
│   ├── 00-decision-matrix.md
│   ├── 01-architecture-vision.md
│   ├── 02-oss-research-report.md
│   ├── 03-mcp-plugin-inventory.md
│   ├── ARCHITECTURE.md
│   ├── RUNBOOK.md
│   ├── OBSERVABILITY.md
│   ├── CONTRACT_ANALYSIS_PROMPTS.md
│   ├── SECURITY_AUDIT_<fase>.md            ← uno per fase, prodotto in step 6 §5
│   └── ADR/
│       ├── 0001-path-build-vs-clone.md
│       ├── 0002-multi-provider-llm.md
│       └── ...
├── engine-rust/                            ← 🦀 Core Engine
│   ├── Cargo.toml
│   ├── Dockerfile
│   ├── src/
│   │   ├── main.rs
│   │   ├── config.rs
│   │   ├── errors.rs
│   │   ├── routes/
│   │   ├── chunker/
│   │   ├── rag/
│   │   ├── providers/                      ← LlmProvider trait + impl multi-provider
│   │   ├── clients/
│   │   ├── models/
│   │   └── utils/
│   └── tests/
├── engine-python/                          ← 🐍 AI Worker
│   ├── requirements.txt
│   ├── Dockerfile
│   ├── app/
│   │   ├── main.py
│   │   ├── routers/
│   │   ├── services/
│   │   └── models/
│   └── tests/
├── php-gateway/                            ← 🐘 PHP Slim 4 (gateway sottile)
│   ├── composer.json
│   ├── public/index.php
│   ├── src/
│   │   ├── Controller/
│   │   ├── Service/
│   │   └── Middleware/
│   ├── config/
│   └── tests/Unit/
├── frontend/                               ← ⚛️ React 18 + Vite + TS
│   ├── package.json
│   ├── vite.config.ts
│   ├── src/
│   │   ├── components/
│   │   ├── pages/
│   │   ├── store/
│   │   ├── lib/
│   │   └── App.tsx
│   └── tests/
├── db/migrations/                          ← SQL ordinate 001_, 002_, ...
├── benchmarks/                             ← suite KPI (ingest, query, hallucination, concurrent, k6)
├── shared/                                 ← volume condiviso fra container
│   └── uploads/                            ← staging file caricati (escluso da git)
└── observability/
    ├── grafana/dashboards/
    └── prometheus/prometheus.yml
```

---

## 12. Variabili d'ambiente (`.env`)

```env
# === App ===
APP_ENV=dev                                  # dev | staging | production
APP_DEBUG=true                               # mai true in production

# === Sicurezza ===
JWT_SECRET=                                  # openssl rand -hex 32
RUST_ENGINE_INTERNAL_TOKEN=                  # openssl rand -hex 64

# === MySQL ===
MYSQL_HOST=mysql
MYSQL_DB=archivio_parlante_x
MYSQL_USER=root
MYSQL_PASSWORD=                              # vuoto solo in dev locale

# === Qdrant ===
QDRANT_URL=http://qdrant:6333

# === Ollama (LLM locale, default zero-cost) ===
OLLAMA_URL=http://ollama:11434
OLLAMA_MODEL_CHAT=qwen2.5:7b-instruct-q4_K_M       # default 8 GB VRAM compatibile
OLLAMA_MODEL_CHAT_SMALL=qwen2.5:3b-instruct-q4_K_M # task massivi low-latency
OLLAMA_MODEL_CHAT_HEAVY=qwen2.5:14b-instruct-q4_K_M # opzionale, offload CPU
OLLAMA_MODEL_EMBED=nomic-embed-text                # 768 dim

# === Servizi interni ===
RUST_ENGINE_URL=http://rust-engine:8090
PYTHON_WORKER_URL=http://python-worker:8091
REDIS_URL=redis://redis:6379

# === Provider cloud OPT-IN (lasciare vuoti per default zero-cost) ===
ANTHROPIC_API_KEY=
GOOGLE_API_KEY=
OPENAI_API_KEY=
DEEPSEEK_API_KEY=
QWEN_API_KEY=
MOONSHOT_API_KEY=
ZHIPU_API_KEY=
MISTRAL_API_KEY=
GROQ_API_KEY=
OPENROUTER_API_KEY=
TOGETHER_API_KEY=
FIREWORKS_API_KEY=

# === Budget guard ===
DAILY_COST_BUDGET_EUR=0.00                   # default zero, alzare da admin UI
MONTHLY_COST_BUDGET_EUR=0.00

# === Storage ===
SHARED_UPLOADS_PATH=/shared/uploads
MAX_UPLOAD_SIZE_MB=200
```

---

## 13. Anti-allucinazione (sempre attivo, non opzionale)

Le 4 tecniche combinate (vedi piano §1.1):

1. **Hybrid Search**: dense (cosine 768-dim) + sparse (BM25 via tantivy) con Reciprocal Rank Fusion (k=60).
2. **Reranker**: BGE-reranker-v2-m3 cross-encoder via Python worker (top 30 → top 5).
3. **Contextual Retrieval** (Anthropic technique): ogni chunk viene arricchito con contesto del documento prima dell'embedding.
4. **Self-RAG + Citation Enforcement**: ogni risposta DEVE contenere `text_quote` verbatim del chunk citato; un secondo pass LLM valida che ogni claim sia ancorato a una citazione. JSON schema obbligatorio.
5. **Knowledge Graph legale**: extract entities (PARTIES, DATES, AMOUNTS, CLAUSES, JURISDICTIONS, PENALTIES) per cross-reference.

Se la query non trova citazioni con confidence > 0.7 → risposta standardizzata: `"Le informazioni richieste non sono presenti nei documenti caricati."` (mai inventare).

---

## 14. Test obbligatori per ogni fase

| Tipo | Tool | Coverage minima |
|---|---|---|
| Unit Rust | `cargo test` | 80% |
| Unit Python | `pytest --cov` | 80% |
| Unit PHP | `PHPUnit` | 80% |
| Unit Frontend | `vitest` | 70% |
| Integration | `cargo test --test integration_*` + `pytest tests/integration/` | flusso completo |
| E2E | `playwright test` | scenari critici (login, upload, chat, compare) |
| Benchmark | `make bench` | KPI piano §12 |
| Security | `cargo audit`, `pip-audit`, `composer audit`, `npm audit`, `trivy` | zero CVE ≥ High |

**Regola d'oro**: se `make test-all` non passa al 100%, la fase non è chiusa, non si committa, non si apre PR.

---

## 15. Documenti di riferimento

| File | Quando leggerlo |
|---|---|
| `PIANO_IMPLEMENTAZIONE_RUST_PYTHON.md` | Sempre, all'inizio di ogni sessione |
| `docs/ADR/0001-path-build-vs-clone.md` | Per capire la decisione architetturale di Fase -1 |
| `docs/ARCHITECTURE.md` | Per capire l'architettura corrente |
| `docs/RUNBOOK.md` | Per troubleshooting operativo |
| `docs/CONTRACT_ANALYSIS_PROMPTS.md` | Per i prompt italiani di dominio legale |
| `docs/SECURITY_AUDIT_<fase>.md` | Per verificare lo stato sicurezza per fase |

---

## 16. Cosa NON fare (checklist anti-pattern)

- ❌ Iniziare a scrivere codice senza aver completato Fase -1 (ricerca OSS + decision matrix + ADR + AskUserQuestion).
- ❌ Abilitare un provider LLM cloud senza chiedere conferma all'utente.
- ❌ Committare `.env`, secrets, dati di clienti reali.
- ❌ Suggerire modelli > 14B come default locale (non entrano in 8 GB VRAM).
- ❌ Usare `unwrap`/`expect` in Rust di produzione, `print` in Python, `echo` in PHP, `console.log` in React.
- ❌ Saltare lo step Security (§5 step 6) "tanto è solo dev".
- ❌ Mergere su `develop`/`main` senza PR + CI verde + checklist 8-step.
- ❌ Aggiungere dipendenze GPL/AGPL/commerciali senza AskUserQuestion.
- ❌ Inventare risposte LLM senza citazioni verbatim → sempre Self-RAG validation.
- ❌ Ricominciare dall'inizio dopo un'interruzione → riprendere ESATTAMENTE dal punto.

---

## 17. Prima azione in ogni nuova sessione

1. Leggi questo file (`.claude/CLAUDE.md`) per intero.
2. Leggi `PIANO_IMPLEMENTAZIONE_RUST_PYTHON.md` sezioni `§0` e la fase corrente.
3. Esegui `git status` + `git log --oneline -20` per capire lo stato.
4. Esegui `cat docs/ADR/*.md` (se esistono) per allinearti alle decisioni prese.
5. Apri/crea TodoList della fase corrente con `TodoWrite`.
6. Procedi step-by-step.

**Buon lavoro.** 🦀🐍🐘⚛️