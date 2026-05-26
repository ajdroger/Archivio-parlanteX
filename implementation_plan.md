# 🏛️ Archivio Parlante — Piano di Costruzione Greenfield a Motore Ibrido Rust + Python

> **Tipo progetto**: 🟢 **Greenfield / from-scratch** — non è un refactor, **Claude Code costruisce tutto da zero** in una directory vuota. Nessun codice preesistente viene riutilizzato.
>
> **Obiettivo**: Costruire da zero un sistema enterprise-grade per l'**analisi forense di contratti aziendali complessi**, con **zero allucinazioni**, **massima precisione** e **confronto multi-contratto in parallelo**, destinato a enti istituzionali ad alto rischio reputazionale.
>
> **Stack finale**: PHP (Gateway & UI) + Rust (Core Engine) + Python (AI Worker) + Qdrant (Vector DB) + **Multi-Provider LLM** (Ollama locale + Anthropic Claude + Google Gemini + modelli cinesi DeepSeek/Qwen + OpenAI + aggregatori OpenRouter/Groq) — tutto in Docker Compose.
>
> **Filosofia operativa** (vedi §0):
> - 💰 **Zero-Cost / Zero-Budget**: tutto open-source, nessuna licenza commerciale obbligatoria. API a pagamento sono **opt-in**, disabilitate di default.
> - 🔓 **Open Source First**: prima di scrivere una riga di codice si valutano framework RAG OSS esistenti (Verba, Quivr, kotaemon, Danswer, AnythingLLM, Open WebUI, Cheshire Cat, Haystack) come potenziali base da clonare/adattare.
> - 🧠 **Senior Solutions Architect mindset**: ogni decisione documentata, trade-off espliciti, KPI misurabili.
> - ❓ **Ask-First**: Claude Code chiede sempre conferma prima di scelte irreversibili (scelta framework base, abilitazione provider a pagamento, schema DB).
> - 🛠️ **Cowork/Claude Code-native**: si sfruttano al massimo skills, MCPs, plugin, subagent invece di scrivere infrastruttura custom.
>
> **Hardware target di sviluppo**: MSI Raider GE78HX 13VG — Intel i9-13950HX (32 thread), **32 GB RAM**, **NVIDIA RTX 4070 Laptop 8 GB VRAM**, Windows 11 Pro, SSD NVMe 2 TB libero ~966 GB. Le raccomandazioni modello in §1.3 sono tarate per **8 GB VRAM**; per modelli più grandi si usano API cloud (vedi §13).

---

## 📑 Indice

0. [🧭 Principi Operativi & Ricerca OSS Preliminare](#0--principi-operativi--ricerca-oss-preliminare)
   - 0.8 [🔄 Ciclo 8-step Obbligatorio per Ogni Fase](#08--ciclo-di-lavoro-obbligatorio-per-ogni-fase--8-step-senior-workflow)
   - 0.9 [Template "Fase CLOSED" — checklist unica](#09-template-fase-closed--checklist-unica)
1. [Executive Summary](#1-executive-summary)
2. [Architettura Target](#2-architettura-target)
3. [Stack Tecnologico Dettagliato](#3-stack-tecnologico-dettagliato)
4. [Struttura del Progetto](#4-struttura-del-progetto)
5. [Fase -1 — Bootstrap Repository & Ricerca OSS](#fase-1--bootstrap-repository--ricerca-oss)
6. [Fase 0 — Setup Infrastruttura Docker Compose](#fase-0--setup-infrastruttura-docker-compose)
7. [Fase 1 — Engine Rust (Core Processing)](#fase-1--engine-rust-core-processing)
8. [Fase 2 — Python AI Worker](#fase-2--python-ai-worker)
9. [Fase 3 — PHP Gateway (greenfield, Slim 4)](#fase-3--php-gateway-greenfield-slim-4)
10. [Fase 4 — Frontend Multi-Contract UI](#fase-4--frontend-multi-contract-ui)
11. [Fase 5 — Testing, Benchmark, Hardening](#fase-5--testing-benchmark-hardening)
12. [Prompt Master per Claude Code](#11-prompt-master-per-claude-code)
13. [KPI, Benchmark e Risk Assessment](#12-kpi-benchmark-e-risk-assessment)
14. [🌐 Architettura Multi-Provider LLM (Switching Runtime)](#13--architettura-multi-provider-llm-switching-runtime)

---

## 0. 🧭 Principi Operativi & Ricerca OSS Preliminare

Questa sezione codifica le **regole di ingaggio** che Claude Code deve seguire per tutto il progetto. Sono vincolanti e prevalgono in caso di conflitto con le fasi successive.

### 0.1 Ruolo richiesto a Claude Code

**Senior Solutions Architect & R&D Lead.** Ogni output deve rispettare tre criteri:

1. **Trade-off espliciti**: per ogni scelta tecnica non banale, Claude Code produce una tabella `Opzione A vs B vs C` con colonne *costo*, *complessità*, *rischio*, *manutenibilità*, *coerenza con stack*, poi motiva la raccomandazione.
2. **KPI misurabili**: ogni componente ha almeno un KPI quantitativo (latenza p95, throughput, accuracy top-k, recall@10, cost/query). Vedi §12.
3. **Documentazione inline**: ogni modulo Rust/Python/PHP ha doc-comment, ogni endpoint ha OpenAPI, ogni tabella DB ha commento SQL.

### 0.2 Zero-Cost / Zero-Budget (vincolante)

| Regola | Implementazione pratica |
|---|---|
| Default del sistema = **solo stack gratuito** | Ollama locale + Qdrant community + MySQL community + Redis OSS + FastAPI + Slim 4 + React. Nessuna chiave API cloud richiesta per far girare il sistema. |
| API a pagamento = **opt-in espliciti** | Provider cloud (Claude/Gemini/OpenAI/DeepSeek/etc.) sono disabilitati di default. L'admin li abilita inserendo la API key dalla UI; senza chiave il provider semplicemente non appare. |
| Budget guard attivo di default | `CostTracker` (§13.7) con soft-limit €0/giorno di default — qualsiasi chiamata a provider a pagamento richiede che l'admin abbia alzato esplicitamente il budget. |
| Nessuna dipendenza con licenza commerciale obbligatoria | Tutte le librerie devono essere MIT, Apache 2.0, BSD, MPL 2.0, LGPL, o compatibili. Claude Code deve verificare la licenza prima di aggiungere una dipendenza e segnalare GPL/AGPL/commerciali. |
| Nessun servizio SaaS obbligatorio | Se una funzionalità richiede un SaaS (es. Pinecone, OpenAI), deve esistere una variante self-hosted gratuita equivalente (es. Qdrant, Ollama). |

### 0.3 Open Source First — ricerca obbligatoria prima di costruire

**Prima di scrivere qualsiasi componente non banale**, Claude Code esegue una ricerca su GitHub / Papers With Code / Hugging Face per trovare un progetto open-source che:
1. Risolve (anche parzialmente) il problema.
2. Ha licenza compatibile.
3. È mantenuto (ultimo commit < 6 mesi, > 500 stars, issue responsiveness).

Se trovato, Claude Code **propone clone + adattamento** invece di scrittura from-scratch, producendo una **matrice di confronto** (vedi §0.5).

### 0.4 Ask-First — conferme obbligatorie

Claude Code **deve fermarsi e chiedere conferma via AskUserQuestion** (o equivalente) nei seguenti casi:

- 🚦 Scelta del framework RAG base (§5 Fase -1): clone di Verba vs Quivr vs kotaemon vs custom from-scratch.
- 🚦 Abilitazione di un provider a pagamento (prima richiesta API Key).
- 🚦 Cambio schema DB che richiede migrazione dati.
- 🚦 Introduzione di una dipendenza non MIT/Apache/BSD.
- 🚦 Qualsiasi scelta con costo ricorrente > €0/mese.
- 🚦 Spostamento di file > 500 LoC scritti in sessioni precedenti.

### 0.5 Framework RAG OSS candidati (shortlist da validare in Fase -1)

Questa è la shortlist iniziale su cui Claude Code deve fare ricerca approfondita in Fase -1 prima di scegliere l'approccio:

| Framework | Repo | Linguaggio | Punti forti | Punti deboli | Licenza |
|---|---|---|---|---|---|
| **Verba** | weaviate/Verba | Python + TS | RAG polished, Weaviate-native, UI pulita | Accoppiato a Weaviate (noi usiamo Qdrant) | BSD-3 |
| **Quivr** | QuivrHQ/quivr | Python (FastAPI) + TS | Multi-brain, multi-LLM, maturo | Opinionated, refactor recente | Apache 2.0 |
| **kotaemon** | Cinnamon/kotaemon | Python | Citation-first, multi-modal, GraphRAG | Meno performante per volume alto | Apache 2.0 |
| **Danswer / Onyx** | onyx-dot-app/onyx | Python (FastAPI) + TS | Enterprise, connectors ricchi, ottimo hybrid search | Complessità setup | MIT |
| **AnythingLLM** | Mintplex-Labs/anything-llm | Node.js + React | Multi-workspace, multi-LLM, desktop app | Non enterprise-grade, sicurezza leggera | MIT |
| **Open WebUI** | open-webui/open-webui | Python + Svelte | UI eccellente, Ollama-native | Più chat UI che RAG specialistico | MIT |
| **Cheshire Cat** | cheshire-cat-ai/core | Python (FastAPI) | Plugin architecture, Italian-friendly | Community piccola | GPL-3 (⚠️ da rivedere) |
| **Haystack** | deepset-ai/haystack | Python | Pipeline RAG industriali, mature | Non è un'app end-to-end, è un framework | Apache 2.0 |
| **RAGFlow** | infiniflow/ragflow | Python + TS | Parsing PDF avanzato, OCR built-in | Backend proprietary in parti | Apache 2.0 |
| **LlamaIndex** | run-llama/llama_index | Python | Ecosistema enorme, ingestion ricca | Framework, non app | MIT |

**Nota sul nostro caso**: nessuno dei framework sopra è scritto in **Rust** e nessuno ha nativamente **contratti legali italiani + knowledge graph + multi-provider switching integrato**. Probabile esito: **clonare parzialmente** componenti specifici (es. parser PDF di RAGFlow, UI di Open WebUI, chunker di LlamaIndex) e **scrivere in Rust solo il core performance-critical** (hybrid search orchestrator, concurrent LLM caller, semantic chunker con regex legali italiane). La decisione finale è in Fase -1 con AskUserQuestion.

### 0.6 Cowork / Claude Code native — leva su funzionalità interne

Invece di scrivere infrastruttura custom, si sfruttano al massimo le funzionalità di Cowork/Claude Code:

| Funzionalità interna | Uso nel progetto |
|---|---|
| **Skills** | Creare skill `archivio-parlante-dev` con istruzioni di coding style, dipendenze vietate, regole DB naming. Skill `archivio-parlante-test` con checklist di testing. |
| **MCP Servers** | MCP per Qdrant admin, MCP per Ollama model management, MCP per log viewer. Ricerca preliminare su MCP registry (§0.7). |
| **Subagents (Task tool)** | Agent `rust-engineer` dedicato a Rust, `python-ml-engineer` a Python worker, `php-reviewer` per gateway, `frontend-react-specialist` per UI. Isolamento context window. |
| **Plugins** | Plugin Cowork per linting Rust (clippy), Python (ruff+mypy), PHP (PHPStan level 8), pre-commit. |
| **Scheduled Tasks** | Task schedulato giornaliero per re-indexing incrementale, cleanup Qdrant orphans, backup MySQL. |
| **TodoWrite** | Obbligatorio per ogni fase >1 step. Ogni fase ha una to-do list verificabile con un task finale di verification. |

### 0.7 Ricerca MCP / Plugin prima di scrivere codice infrastrutturale

Prima di scrivere qualsiasi codice che interagisce con servizi esterni (Qdrant, Ollama, MySQL, Redis, Docker, provider LLM), Claude Code **deve chiamare `mcp__mcp-registry__search_mcp_registry`** con keyword pertinenti e `mcp__plugins__search_plugins`. Se esiste un MCP ufficiale, si usa quello invece di scrivere un client HTTP custom.

Esempi di ricerche obbligatorie in Fase -1:
- `search_mcp_registry(keywords: ["qdrant", "vector database"])`
- `search_mcp_registry(keywords: ["ollama", "local llm"])`
- `search_mcp_registry(keywords: ["docker compose", "container"])`
- `search_mcp_registry(keywords: ["mysql", "database admin"])`
- `search_plugins(query: "rust clippy formatter")`
- `search_plugins(query: "python ruff mypy")`

### 0.8 🔄 Ciclo di Lavoro Obbligatorio per Ogni Fase — 8-step senior workflow

Questo è il **ciclo vincolante** che Claude Code DEVE applicare alla fine di **ogni singola fase** (Fase 0, 1.1, 1.2, …, 1.6, 2.1, 2.2, 3, 4, 5) — non solo al termine del progetto. Ogni fase è considerata chiusa **solo quando tutti gli 8 step sono completati con successo**.

#### Step 1 — Ricerca preliminare (già coperta da §0.3, §0.5, §0.7)

Prima di scrivere codice: ricerca OSS, ricerca MCP/plugin, analisi librerie esistenti, studio di soluzioni già risolte.

#### Step 2 — Pianificazione & Esecuzione Step-by-Step

- **Ruolo**: *Senior Tech Lead & Project Manager*.
- Sulla base della ricerca, Claude Code crea **preventivamente** un sotto-piano della fase corrente, diviso in task granulari (idealmente gestiti via `TodoWrite`).
- Esegue rigorosamente step-by-step fino al completamento del 100% dei task della fase.
- **Gestione interruzioni**: se l'esecuzione si interrompe per qualsiasi motivo (errore, context limit, riavvio), Claude Code **DEVE SEMPRE riprendere esattamente dal punto di interruzione**. Divieti assoluti: mai ricominciare dall'inizio, mai saltare fasi o task.
- Output atteso: una lista di task chiusi in ordine con commit intermedi.

#### Step 3 — Sviluppo + Testing (integrati, non sequenziali)

- **Ruolo**: *Senior Software Engineer + Senior QA Automation Engineer* (Claude Code tiene entrambi i cappelli).
- Dopo ogni modulo/feature scritto, **obbligo di test inerenti**:
  - Rust: `cargo test` unitario + integration test in `tests/`.
  - Python: `pytest` con fixture + snapshot test dove applicabile.
  - PHP: `PHPUnit` via `composer test`.
  - Frontend: `Vitest` + React Testing Library + Playwright E2E.
- **Requisito di passaggio**: tutti i test devono passare al **100%**. Se anche un solo test fallisce → NON si passa alla fase successiva. Si diagnostica, si corregge, si rilancia la suite.

#### Step 4 — Ottimizzazione Performance & Scalabilità

- **Ruolo**: *Senior Performance Engineer*.
- Analisi sistematica:
  - Profili CPU (flamegraph per Rust: `cargo flamegraph`; Python: `py-spy`, `scalene`).
  - Analisi memoria (Rust: `heaptrack`, Python: `tracemalloc`, `memray`).
  - Query DB (`EXPLAIN ANALYZE` MySQL, Qdrant `collection_info` per metriche vector).
  - Tail latency con k6 (`benchmarks/k6/`) per endpoint chiave.
- Identificazione colli di bottiglia: N+1 query, lock contention, serializzazione seriale quando si poteva parallelizzare, allocazioni Rust inutili, memory leak Python.
- Ottimizzazioni: batching, concorrenza async, cache Redis, indici MySQL mancanti, prepared statements, quantizzazione modelli, streaming invece di buffer totale.
- **Verifica obbligatoria**: dopo ogni ottimizzazione, **rilanciare la suite di test** (step 3). Tutti i test devono ancora passare al 100%. Se un test fallisce dopo un'ottimizzazione → rollback o fix immediato.

#### Step 5 — Clean Code, Revisione Commenti & Linting

- **Ruolo**: *Senior Clean Code Expert & Code Reviewer*.
- **Manutenzione e pulizia**:
  - Aggiorna commenti inline perché riflettano il comportamento attuale.
  - Elimina senza pietà commenti obsoleti, fuorvianti, o blocchi di codice commentato (dead code).
  - Elimina variabili inutilizzate, import morti, funzioni non più chiamate.
- **Standardizzazione**:
  - Rust: `cargo fmt`, `cargo clippy --all-targets -- -D warnings`.
  - Python: `ruff format`, `ruff check --fix`, `mypy --strict`.
  - PHP: `php-cs-fixer fix`, `phpstan analyse --level=8`.
  - Frontend: `eslint --fix`, `prettier --write`, `tsc --noEmit`.
- **Regression Test Assoluto**: dopo pulizia/formattazione, **rieseguire l'intera suite di test** inerente alla fase. Tutti i test al 100%. Se qualcosa è rotto dalla pulizia → rollback selettivo.

#### Step 6 — Cybersecurity & Sicurezza Infrastrutturale (priorità massima)

- **Ruolo**: *Senior Cybersecurity Engineer*.
- **Audit completo** del lavoro svolto nella fase corrente:
  - **Input validation**: ogni endpoint (Rust axum, Python FastAPI, PHP Slim) valida lunghezza/tipo/whitelist/sanitizzazione.
  - **Authentication**: JWT firmato HS256/RS256 con secret lungo, rotazione chiavi documentata, token TTL breve + refresh token.
  - **Authorization**: ogni risorsa (kb_id, doc_id) controllata contro l'utente corrente; no IDOR.
  - **SQLi/NoSQLi**: SEMPRE prepared statements con `sqlx` (Rust) / `PDO` (PHP) / SQLAlchemy parameterized (Python). Mai string concat.
  - **XSS/CSRF**: React auto-escape, CSP header stretta, SameSite=Strict cookies, token CSRF per mutazioni critiche.
  - **Path traversal**: normalizzazione path upload, reject `../`, canonicalize + check confine `/shared/uploads`.
  - **Command injection**: mai `shell=True` in Python, mai `exec()` PHP, mai `Command::new()` Rust con input utente concatenato.
  - **DoS**: rate limiting Redis per IP + per utente, timeout su ogni I/O, max body size, max concurrent uploads.
  - **Secrets**: mai hardcoded, mai loggati, `.env` in `.gitignore`, secret manager in produzione.
  - **Dependency audit**: `cargo audit`, `pip-audit` / `safety check`, `composer audit`, `npm audit` — bloccanti su CVE High/Critical.
  - **Container security**: `trivy image` su ogni Dockerfile; immagine base aggiornata; USER non-root; readonly root filesystem dove possibile.
  - **TLS**: HTTPS obbligatorio in produzione; certs via Let's Encrypt; HSTS; no TLS < 1.2.
  - **Privacy**: PII contrattuali cifrate at-rest (opzionale: colonne MySQL AES-256 via `AES_ENCRYPT` con key in KMS esterno); log PII redatti.
  - **LLM security**: prompt injection resistance (system prompt con delimitatori, istruzioni user isolate), jailbreak detection via Self-RAG, output sanitization.
- **Standard**: obiettivo = zero vulnerabilità CVSS ≥ 7.0 sulla piattaforma. Ogni vulnerabilità trovata → correggi, rilancia test, ripeti audit.
- Report audit in `docs/SECURITY_AUDIT_<fase>.md` con checklist OWASP ASVS Level 2 verificata.

#### Step 7 — Aggiornamento Documentazione

- **Ruolo**: *Senior Technical Writer & Knowledge Manager*.
- **Analisi radicale**: scansione di tutti i documenti di progetto:
  - `README.md` (root + sotto-cartelle).
  - `docs/ARCHITECTURE.md`, `docs/RUNBOOK.md`, `docs/OBSERVABILITY.md`, `docs/CONTRACT_ANALYSIS_PROMPTS.md`.
  - ADR in `docs/ADR/` — se la fase ha introdotto una decisione architetturale, creare un nuovo ADR (`0002-*`, `0003-*`, …).
  - `CHANGELOG.md` — aggiungere voce con data + tipo + descrizione.
  - `.claude/CLAUDE.md` — aggiornare se cambiano comandi di build/test/run.
  - OpenAPI / Rust `utoipa` / FastAPI auto-docs — rigenerare.
- **Aggiornamento obbligatorio**: se il lavoro introduce nuove logiche o rende obsoleta documentazione, aggiornare per riflettere lo stato attuale. Documentazione sincronizzata al 100% prima di chiudere la fase.

#### Step 8 — Git Workflow (obbligatorio)

- **Ruolo**: *Senior DevOps & Release Engineer*.
- **Strategia branch**: sempre feature branch partendo da `develop`. Mai commit diretti su `main` o `develop` in produzione.
- **Branch naming**: `feature/fase-<N>-<slug-descrittivo>` (es. `feature/fase-1-1-rust-scaffolding`, `feature/fase-3-php-gateway`).
- **Sequenza di commit end-of-phase** (SOLO dopo step 1-7 validati al 100%):

  ```bash
  git status                         # verifica modifiche a codice + docs
  git add .                          # stage tutto (dopo aver verificato nessun secret)
  git commit -m "[feat] fase-<N>: <descrizione sintetica>

  - Task completati: <lista>
  - Test passati: <count>
  - Security audit: OK (vedi docs/SECURITY_AUDIT_<fase>.md)
  - Documentazione aggiornata: <lista file>
  "
  git pull --rebase origin develop   # allinea e previene conflitti
  git push -u origin feature/fase-<N>-<slug>
  ```

- **Pull Request**: aperta su GitHub/GitLab con template `.github/PULL_REQUEST_TEMPLATE.md` che richiede checklist di tutti gli 8 step.
- **Conventional Commits**: tipo = `feat` | `fix` | `refactor` | `perf` | `docs` | `test` | `chore` | `ci` | `build` | `security`.
- **Commit atomici**: preferire più commit piccoli e tematici piuttosto che uno monolitico.
- **Firma**: se il cliente lo richiede, commit firmati GPG (configurare una volta nel bootstrap).
- **Divieti**: mai `git push --force` su `develop`/`main`; mai `git commit --no-verify` (se ci sono hook, risolvi prima); mai committare `.env`, credenziali, chiavi, dati sensibili.

### 0.9 Template "Fase CLOSED" — checklist unica

Ogni fase è considerata **"CLOSED"** solo quando questa checklist è al 100%:

- [ ] Step 1 — Ricerca completata, MCP/plugin/OSS valutati, risultati documentati.
- [ ] Step 2 — Piano della fase redatto, tutti i task completati in ordine.
- [ ] Step 3 — Suite test della fase passa al 100% (`cargo test` + `pytest` + `composer test` + `vitest` + E2E).
- [ ] Step 4 — Profilo performance eseguito, colli di bottiglia rimossi, test ripassati al 100%.
- [ ] Step 5 — Linter/formatter senza warning, dead code rimosso, test ripassati al 100%.
- [ ] Step 6 — Security audit completato, vulnerabilità ≥ High risolte, report in `docs/SECURITY_AUDIT_<fase>.md`.
- [ ] Step 7 — Documentazione aggiornata: README, ARCHITECTURE, RUNBOOK, CHANGELOG, ADR (se applicabile).
- [ ] Step 8 — Commit su feature branch, PR aperta, CI verde, merge su `develop` approvato.

Finché questi 8 step non sono tutti verdi, la fase non è chiusa e non si procede a quella successiva.

---

## 1. Executive Summary

### 1.1 Scelte architetturali confermate dall'utente

| Decisione | Scelta | Motivazione |
|---|---|---|
| **LLM primario locale** | Ollama (modelli 7B–14B) | Privacy totale su contratti riservati, zero costi API, sovranità dei dati, gira su 8 GB VRAM |
| **LLM cloud premium** | Multi-provider runtime-switchable: Anthropic Claude (Opus 4.7 / Sonnet 4.6 / Haiku 4.5), Google Gemini 2.5 Pro/Flash, OpenAI GPT-5/o3, DeepSeek V3/R1, Qwen Max, Moonshot Kimi, Zhipu GLM-4.5, Mistral Large, OpenRouter, Groq, Together.ai, Fireworks.ai | Massima precisione su task critici, switching per costo/qualità/latenza, fallback in caso di down |
| **Embedding model** | `nomic-embed-text` via Ollama (768 dim) — opzionale upgrade a `bge-m3` (1024 dim, multilingue) | Allineato all'infrastruttura locale; cloud opzionale (Voyage, Cohere) per benchmark |
| **Deployment** | Docker Compose | Stack unificato PHP + Rust + Python + Qdrant + Ollama + Redis + MySQL |
| **Anti-allucinazioni** | **Tutte e 4 le tecniche**: Hybrid Search + Reranker, Contextual Retrieval, Self-RAG + Citation, Knowledge Graph legale | Ridondanza necessaria per enti importanti e contratti ad alto rischio |
| **Hardware sviluppo** | RTX 4070 Laptop **8 GB VRAM**, 32 GB RAM, i9-13950HX | Vincolo principale: nessun modello locale > 14B in q4 (~9 GB). I modelli grandi (32B, 70B, 405B) si usano via API cloud. |

### 1.2 Perché Rust + Python (e non solo PHP)

| Compito | Linguaggio ottimale | Motivazione |
|---|---|---|
| Parsing PDF complessi con tabelle/layout | **Python** (`unstructured`, `PyMuPDF`, `layoutparser`) | Ecosistema ML insuperabile per vision parsing |
| Chunking semantico su documenti enormi | **Rust** (`tiktoken-rs`, `regex`) | 50–100× più veloce del PHP, memoria sicura |
| Generazione embedding concorrente | **Rust** (`tokio`, `reqwest`) | 500+ req/s paralleli verso Ollama, zero GIL |
| Chiamate LLM simultanee multi-contratto | **Rust** (`tokio::join!`, streaming SSE) | 100+ richieste parallele in attesa I/O |
| Ricerca vettoriale + BM25 | **Rust** (`qdrant-client`, `tantivy`) | Hybrid search nativo in memoria locale |
| Reranker cross-encoder | **Python** (`FlagEmbedding`, BGE-reranker-v2) | Modello PyTorch con GPU, no port Rust maturo |
| OCR + Knowledge Graph | **Python** (`tesseract`, `spaCy`, `flair`) | Standard de facto per NER legale |
| Orchestrazione business / UI / auth | **PHP** (Slim 4) | Ecosistema maturo, deploy semplicissimo, gateway sottile: poca logica significa basso rischio di bug |

**Risultato atteso**: ingestion **20–40× più veloce**, query RAG **sotto 500 ms**, capacità di analizzare **50+ contratti in parallelo** con precisione forense.

### 1.3 Modelli Ollama consigliati per uso legale (ottimizzati per RTX 4070 Laptop 8 GB VRAM)

> ⚠️ **Importante**: il tuo hardware ha **8 GB VRAM** sulla RTX 4070 Laptop e **32 GB RAM** di sistema. I modelli >14B in quantizzazione Q4 NON entrano in VRAM e vanno offloadati su CPU/RAM (3–10× più lenti). Per i task che richiedono modelli enterprise (32B, 70B, 405B+) si usa l'**API cloud multi-provider** (vedi §13).

#### Modelli locali consigliati (entrano in VRAM 8 GB)

| Ruolo | Modello | VRAM | Note |
|---|---|---|---|
| **Chat principale legale (best in 8 GB)** | `qwen2.5:7b-instruct-q4_K_M` | ~4.7 GB | Eccellente su italiano + ragionamento, lascia spazio a embedding model concorrente |
| **Alternativa multilingua** | `llama3.1:8b-instruct-q4_K_M` | ~5.0 GB | Solido, supporto italiano discreto |
| **Alternativa Mistral** | `mistral:7b-instruct-q4_K_M` | ~4.4 GB | Veloce, buon ragionamento giuridico |
| **Alternativa Google** | `gemma2:9b-instruct-q4_K_M` | ~5.8 GB | Forte su comprensione contestuale |
| **Reasoning profondo (CoT esplicito)** | `deepseek-r1:7b` (distillato) o `deepseek-r1:8b` | ~5.0 GB | Chain-of-thought esplicito per comparazione clausole — versione distillata, **NON** la 32b |
| **LLM ultra-leggero (intent / contextualize)** | `qwen2.5:3b-instruct-q4_K_M` | ~2.0 GB | Per task massivi: contextual retrieval su migliaia di chunk |
| **Embedding principale** | `nomic-embed-text` | ~0.3 GB | 768 dim, default scelto |
| **Embedding upgrade (multilingue)** | `bge-m3` | ~1.2 GB | 1024 dim, migliore su italiano legale |

#### Modelli più grandi (>14B) — opzioni

1. **Offload CPU/RAM** (lento ma funziona, hai 32 GB RAM):
   - `qwen2.5:14b-instruct-q4_K_M` (~9 GB) — borderline VRAM, parziale offload, ~10–20 tok/s
   - `qwen2.5:32b-instruct-q4_K_M` (~20 GB) — full CPU offload, ~3–5 tok/s (utilizzabile per task batch notturni)
   - `mixtral:8x7b-instruct-q4_K_M` (~26 GB) — MoE, full CPU, lento ma di altissima qualità

2. **API cloud** (raccomandato per qualità top — vedi §13):
   - Claude Opus 4.7 / Sonnet 4.6 — top reasoning legale
   - Gemini 2.5 Pro — context window 2M token (intero contratto in un colpo)
   - DeepSeek V3 / R1 — economici e potenti
   - GPT-5 / o3 — reasoning premium

#### Strategia "modello giusto per il task giusto" (default suggerito)

| Task | Modello | Provider |
|---|---|---|
| Intent classification + query routing | `qwen2.5:3b` | Ollama locale (latenza < 100 ms) |
| Contextual retrieval (per chunk) | `qwen2.5:3b` o `qwen2.5:7b` | Ollama locale (chiamate massive, costo zero) |
| Risposta RAG standard | `qwen2.5:7b` o Claude Haiku 4.5 | Ollama locale o cloud (switchable da UI) |
| Comparazione multi-contratto critica | Claude Sonnet 4.6 o Gemini 2.5 Pro | Cloud (precisione massima) |
| Audit forense / clausole alto rischio | Claude Opus 4.7 o GPT-5 | Cloud (best-in-class) |
| Self-RAG evaluator | `qwen2.5:7b` | Ollama locale (verifica ridondante) |

#### Per produzione (futuro)

- Sostituire Ollama con **vLLM** (throughput 10× superiore) su workstation con GPU >24 GB
- Oppure mantenere Ollama locale per task quotidiani + cloud per task critici (modello attuale)

---

## 2. Architettura Target

### 2.1 Diagramma microservizi

```
┌──────────────────────────────────────────────────────────────────────┐
│                        [ Utente / Browser ]                           │
└──────────────────────────────────┬───────────────────────────────────┘
                                   │
                       ┌───────────▼───────────┐
                       │  React 18 SPA / Widget │
                       │  (Vite, Zustand, SSE)  │
                       └───────────┬───────────┘
                                   │ HTTPS/JWT
                       ┌───────────▼───────────────────────────────┐
                       │  PHP Gateway — Slim 4  (host porta 9080)   │
                       │  ✓ Auth JWT / API Key                      │
                       │  ✓ Rate limiting Redis                     │
                       │  ✓ Gestione utenti, admin, sessioni        │
                       │  ✓ Proxy verso Rust Engine                 │
                       └───────────┬───────────────────────────────┘
                                   │ REST/JSON interno
                       ┌───────────▼───────────────────────────────┐
                       │  🦀 Rust Core Engine — Axum  (porta 8090)  │
                       │  ───────────────────────────────────────   │
                       │  • Orchestratore async (Tokio)             │
                       │  • Chunker semantico + Contextual Retr.    │
                       │  • Hybrid Search (dense + BM25/Tantivy)    │
                       │  • Self-RAG Evaluator                      │
                       │  • Multi-contract comparison engine        │
                       │  • Streaming SSE verso PHP                 │
                       └──┬─────────────┬──────────────┬────────────┘
                          │             │              │
            gRPC/REST ┌───▼──┐  REST ┌──▼──────┐   REST ┌──▼──────┐
                      │      │        │         │        │         │
                 ┌────▼───┐  │   ┌────▼────┐    │   ┌────▼─────┐  │
                 │Python  │  │   │ Qdrant  │    │   │  Ollama  │  │
                 │AI Worker│ │   │ Vector  │    │   │ (LLM +   │  │
                 │FastAPI  │ │   │   DB    │    │   │  Embed)  │  │
                 │(porta   │ │   │(porta   │    │   │(porta    │  │
                 │ 8091)   │ │   │ 6333)   │    │   │ 11434)   │  │
                 ├────────┤  │   ├────────┤    │   ├──────────┤  │
                 │unstruc.│  │   │dense + │    │   │nomic-emb │  │
                 │PyMuPDF │  │   │sparse  │    │   │qwen2.5   │  │
                 │OCR     │  │   │BM25    │    │   │llama3.3  │  │
                 │BGE-    │  │   │collect.│    │   │          │  │
                 │Rerank. │  │   │per KB  │    │   │          │  │
                 │KG      │  │   │        │    │   │          │  │
                 │Extract.│  │   │        │    │   │          │  │
                 └────────┘  │   └────────┘    │   └──────────┘  │
                             │                  │                 │
                   ┌─────────▼──────────────────▼─────────────────▼┐
                   │  MySQL 8  (porta 3306)  —  archivio_parlante_x│
                   │  + Redis 7  (porta 6379)  rate limit + cache  │
                   └───────────────────────────────────────────────┘
```

### 2.2 Flusso Ingestion (Upload → Indicizzazione)

```
1. [PHP] riceve POST /api/ingest (multipart)
           │
           ▼
2. [PHP] valida MIME, salva in /storage/ai/uploads/{uuid}, crea job in DB
           │
           ▼
3. [PHP] POST http://rust-engine:8090/ingest {doc_id, file_path, kb_id}
           │
           ▼
4. [RUST] riconosce estensione → se PDF/immagine/OCR/docx_complesso
         → POST http://python-worker:8091/parse (stream file)
         → riceve JSON strutturato {text, tables, metadata, layout}
         │
           ▼  (altrimenti parsing nativo Rust per .md/.txt/.csv)
5. [RUST] CHUNKING SEMANTICO
         • Split per headers (# ## ###)
         • Fallback per frasi (punteggiatura + tiktoken)
         • Overlap 15% tra chunks consecutivi
         • Chunk size adattivo 600–1200 tokens
           │
           ▼
6. [RUST → PYTHON] CONTEXTUAL RETRIEVAL (Anthropic technique)
         Per ogni chunk, chiama Python /contextualize:
         • Prepend 50–80 token di riassunto del documento intero
         • Usa LLM leggero (qwen2.5:7b) per velocità
         Risultato: chunk arricchito → -49% errori retrieval
           │
           ▼
7. [RUST] EMBEDDING CONCORRENTE
         • tokio::spawn per ogni chunk
         • Chiamate parallele a Ollama (semaforo max 16 concurrent)
         • Retry esponenziale su errori transitori
           │
           ▼
8. [RUST] STORAGE DUAL IN QDRANT
         • Dense vector (nomic-embed-text 768 dim)
         • Sparse vector (BM25 via tantivy stemmer italiano)
         • Payload: {doc_id, chunk_idx, text, contextual, metadata}
           │
           ▼
9. [RUST → PYTHON] KNOWLEDGE GRAPH EXTRACTION (asincrono, non blocca)
         POST /extract_entities_relations → JSON {nodes, edges}
         Entità legali: PARTI, DATE, IMPORTI, CLAUSOLE, GIURISDIZIONI
         Salvate in MySQL ap_graph_nodes / ap_graph_edges
           │
           ▼
10. [RUST → PHP] risposta { job_id, chunks_indexed, entities, processing_ms }
11. [PHP] aggiorna ap_documents.status = 'indexed', notifica frontend via polling
```

### 2.3 Flusso Query RAG (Domanda → Risposta)

```
1. [Utente] invia domanda (es. "Confronta le penali di NDA-2024 e NDA-2025")
           │
           ▼
2. [PHP] POST /api/chat → forward a http://rust-engine:8090/query
           │
           ▼
3. [RUST] INTENT CLASSIFICATION (qwen2.5:7b)
         → rag_query | contract_comparison | summarize | general_chat
           │
           ├─ se contract_comparison → ramo MULTI-CONTRACT (sez. 6.7)
           └─ se rag_query → continua
           │
           ▼
4. [RUST] QUERY EXPANSION
         • Genera 3 riformulazioni della domanda (HyDE)
         • Embedding della domanda + hypothetical answer
           │
           ▼
5. [RUST] HYBRID SEARCH in Qdrant
         • Dense search (cosine, top 30)
         • Sparse BM25 (top 30)
         • Fusion RRF (Reciprocal Rank Fusion)
         → top 20 candidati
           │
           ▼
6. [RUST → PYTHON] RERANKER CROSS-ENCODER
         POST /rerank {query, candidates}
         → BGE-reranker-v2-m3 su GPU
         → top 5 chunks con score finale
           │
           ▼
7. [RUST] PROMPT CONSTRUCTION con citation enforcement
         System prompt in italiano forzante:
         • Rispondi SOLO dal contesto
         • Cita testualmente (virgolette + [doc_id:chunk_idx])
         • JSON schema output: {answer, citations[], confidence}
           │
           ▼
8. [RUST → OLLAMA] STREAMING GENERATION
         Token-by-token via SSE forward al PHP → al frontend
           │
           ▼
9. [RUST] SELF-RAG EVALUATOR (post-hoc, non blocca streaming)
         Secondo LLM verifica:
         • Grounding: ogni claim è nei chunks?
         • Completeness: risponde alla domanda?
         • Citation validity: i quote sono testuali?
         Se fail → rigenera con prompt più restrittivo (max 2 retry)
           │
           ▼
10. [RUST → PHP → FRONTEND] risposta finale con:
     { answer, citations: [{doc_id, chunk, text_quote, score}],
       confidence, verified: true, processing_ms }
```

---

## 3. Stack Tecnologico Dettagliato

### 3.1 Rust Engine (`engine-rust/`)

| Crate | Versione | Scopo |
|---|---|---|
| `tokio` | `1.40` | Async runtime multi-thread |
| `axum` | `0.7` | Web framework HTTP/JSON + SSE |
| `tower-http` | `0.6` | CORS, tracing, compression middleware |
| `reqwest` | `0.12` | HTTP client async (Ollama, Python worker, Qdrant) |
| `qdrant-client` | `1.12` | Client ufficiale Qdrant (gRPC) |
| `serde` + `serde_json` | `1` | Serializzazione JSON |
| `tantivy` | `0.22` | BM25 full-text index nativo Rust |
| `tiktoken-rs` | `0.6` | Tokenizzazione compatibile OpenAI/Llama |
| `regex` | `1.11` | Chunking pattern-based |
| `tracing` + `tracing-subscriber` | `0.3` | Structured logging |
| `anyhow` + `thiserror` | `1` | Error handling |
| `uuid` | `1.11` | Generazione ID deterministici |
| `futures` | `0.3` | Stream, join concurrent |
| `dashmap` | `6` | Cache concurrente in-memory |
| `governor` | `0.7` | Rate limiting lato client LLM |

### 3.2 Python AI Worker (`engine-python/`)

| Libreria | Versione | Scopo |
|---|---|---|
| `fastapi` | `0.115` | Web framework API |
| `uvicorn[standard]` | `0.32` | ASGI server |
| `unstructured[pdf,docx]` | `0.16` | Parsing documenti complessi |
| `pymupdf` | `1.24` | PDF parsing + estrazione immagini |
| `pdfplumber` | `0.11` | Estrazione tabelle PDF |
| `pytesseract` | `0.3.13` | OCR Tesseract wrapper |
| `Pillow` | `11` | Image processing |
| `FlagEmbedding` | `1.3` | BGE Reranker v2-m3 |
| `torch` | `2.5` | Runtime PyTorch |
| `spacy` + `it_core_news_lg` | `3.8` | NER italiano |
| `transformers` | `4.46` | Modelli HuggingFace |
| `httpx` | `0.28` | HTTP client verso Ollama |
| `pydantic` | `2.9` | Validazione schema |

### 3.3 Qdrant (Vector DB)

- Versione: **1.12+**
- Config: 1 collection per `knowledge_base_id`
- Schema vettori:
  - **dense**: `nomic-embed-text`, 768 dim, cosine
  - **sparse**: BM25 (generato da Rust/Tantivy e inserito come `sparse_vector`)
- Payload indicizzato: `doc_id`, `chunk_idx`, `source_name`, `tags`, `indexed_at`, `contract_type`

### 3.4 Ollama (LLM locale — ottimizzato 8 GB VRAM)

Modelli da scaricare al primo avvio (`docker exec ollama ollama pull`):

```powershell
# === SETUP MINIMO RACCOMANDATO (entra in 8 GB VRAM) ===

# Chat principale legale — DEFAULT
ollama pull qwen2.5:7b-instruct-q4_K_M    # ~4.7 GB VRAM, italiano eccellente

# LLM leggero per intent/contextualize (massive calls)
ollama pull qwen2.5:3b-instruct-q4_K_M    # ~2.0 GB VRAM

# Embeddings
ollama pull nomic-embed-text              # ~300 MB, 768 dim

# === ALTERNATIVE (uno a scelta sostituendo qwen2.5:7b) ===
ollama pull llama3.1:8b-instruct-q4_K_M   # ~5.0 GB VRAM, multilingua
ollama pull mistral:7b-instruct-q4_K_M    # ~4.4 GB VRAM, veloce
ollama pull gemma2:9b-instruct-q4_K_M     # ~5.8 GB VRAM, contestuale

# === REASONING AVANZATO (CoT esplicito per comparazione) ===
ollama pull deepseek-r1:7b                # ~5.0 GB VRAM, distillato
# oppure
ollama pull deepseek-r1:8b                # ~5.5 GB VRAM, distillato

# === UPGRADE EMBEDDING (opzionale) ===
ollama pull bge-m3                         # ~1.2 GB, 1024 dim, multilingue top

# === OPZIONALI BORDERLINE / CPU-OFFLOAD (32 GB RAM) ===
# ⚠️ Saranno lenti ma usabili per batch notturni, NON per real-time
ollama pull qwen2.5:14b-instruct-q4_K_M   # ~9 GB VRAM, parziale offload
ollama pull qwen2.5:32b-instruct-q4_K_M   # ~20 GB, full CPU offload, 3–5 tok/s
```

Per qualità enterprise su task critici, **NON scaricare modelli 70B+ localmente** — usa i provider cloud via API configurati in §13 (Claude Opus 4.7, Gemini 2.5 Pro, DeepSeek V3, GPT-5).

---

## 4. Struttura del Progetto

```
archivio-parlante/
│
├── docker-compose.yml              ← Orchestrazione 6 servizi
├── .env                            ← Variabili condivise (MySQL, Ollama, Qdrant URLs)
├── Makefile                        ← comandi: make up/down/logs/bench
│
├── php-gateway/                    ← 🐘 PHP Gateway (Slim 4) — gateway sottile
│   ├── public/index.php            ← entry point Slim
│   ├── composer.json
│   ├── config/
│   │   ├── routes.php
│   │   └── container/services.php
│   ├── src/
│   │   ├── Controller/
│   │   │   ├── AuthController.php
│   │   │   ├── KbController.php
│   │   │   ├── ChatController.php          ← proxy verso Rust /query e /compare
│   │   │   └── AdminProvidersController.php ← gestione provider LLM (§13.9)
│   │   ├── Service/
│   │   │   ├── Engine/RustEngineClient.php ← Guzzle client → Rust
│   │   │   ├── Auth/JwtService.php
│   │   │   └── RateLimit/RedisRateLimiter.php
│   │   └── Middleware/
│   │       ├── JwtAuth.php
│   │       ├── CorrelationId.php
│   │       └── SecurityHeaders.php
│   └── tests/Unit/
│
├── frontend/                       ← ⚛️ React 18 + Vite + TS + Tailwind (greenfield)
│   ├── src/
│   │   ├── components/             ← ContractComparison, ContextViewer, ChatMessage, ModelSelector, …
│   │   ├── pages/                  ← LoginPage, AdminPanel, KbDetail
│   │   ├── store/archivioStore.ts  ← Zustand
│   │   ├── lib/api.ts              ← axios client
│   │   └── App.tsx
│   └── vite.config.ts
│
├── engine-rust/                    ← 🦀 NUOVO MICROSERVIZIO
│   ├── Cargo.toml
│   ├── Dockerfile
│   ├── .env.example
│   └── src/
│       ├── main.rs                 ← Entry point axum
│       ├── config.rs               ← Caricamento env
│       ├── errors.rs               ← AppError + IntoResponse
│       ├── routes/
│       │   ├── mod.rs
│       │   ├── ingest.rs           ← POST /ingest
│       │   ├── query.rs            ← POST /query (streaming SSE)
│       │   ├── compare.rs          ← POST /compare_contracts
│       │   └── health.rs           ← GET /health
│       ├── chunker/
│       │   ├── mod.rs
│       │   ├── semantic.rs         ← splitting markdown + sentence
│       │   └── contextual.rs       ← contextual retrieval caller
│       ├── rag/
│       │   ├── mod.rs
│       │   ├── hybrid_search.rs    ← dense+sparse+RRF
│       │   ├── reranker.rs         ← bridge a Python BGE
│       │   ├── prompt_builder.rs   ← system prompt italiano forensic
│       │   ├── self_rag.rs         ← grounding validator
│       │   └── multi_contract.rs   ← comparison engine
│       ├── clients/
│       │   ├── mod.rs
│       │   ├── ollama.rs           ← embed + generate + streaming
│       │   ├── qdrant.rs           ← wrapper tipizzato
│       │   └── python_worker.rs    ← parse/rerank/kg/contextualize
│       ├── models/
│       │   ├── mod.rs
│       │   ├── document.rs
│       │   ├── chunk.rs
│       │   └── query.rs
│       └── utils/
│           ├── mod.rs
│           ├── rate_limiter.rs
│           └── tokenizer.rs
│   └── tests/
│       ├── integration_ingest.rs
│       ├── integration_query.rs
│       └── fixtures/
│           ├── contract_sample.pdf
│           └── nda_sample.docx
│
├── engine-python/                  ← 🐍 NUOVO MICROSERVIZIO
│   ├── requirements.txt
│   ├── Dockerfile
│   ├── .env.example
│   └── app/
│       ├── main.py                 ← FastAPI entry
│       ├── config.py
│       ├── routers/
│       │   ├── __init__.py
│       │   ├── parse.py            ← /parse (PDF/docx/img)
│       │   ├── rerank.py           ← /rerank (BGE cross-encoder)
│       │   ├── contextualize.py    ← /contextualize (chunk + doc summary)
│       │   └── knowledge_graph.py  ← /extract_entities_relations
│       ├── services/
│       │   ├── __init__.py
│       │   ├── pdf_parser.py       ← unstructured + PyMuPDF
│       │   ├── ocr_service.py      ← Tesseract con ita
│       │   ├── reranker.py         ← FlagEmbedding BGE-v2-m3
│       │   ├── kg_extractor.py     ← spaCy NER + LLM relations
│       │   └── ollama_client.py    ← async calls to LLM
│       └── models/
│           └── schemas.py          ← Pydantic
│   └── tests/
│       ├── test_parse.py
│       ├── test_rerank.py
│       └── fixtures/
│
├── db/
│   └── migrations/
│       ├── 001_create_users.sql
│       ├── 002_create_kb.sql
│       ├── 003_create_documents.sql
│       ├── 004_create_chat.sql
│       ├── 005_create_analyses.sql
│       ├── 006_create_audit.sql
│       ├── 007_create_providers.sql        ← provider LLM + budget (§13.10)
│       └── 008_graph_and_jobs.sql          ← grafo legale + jobs background
│
├── docs/
│   ├── 00-decision-matrix.md
│   ├── 01-architecture-vision.md
│   ├── 02-oss-research-report.md
│   ├── 03-mcp-plugin-inventory.md
│   ├── ARCHITECTURE.md
│   ├── RUNBOOK.md
│   ├── OBSERVABILITY.md
│   ├── CONTRACT_ANALYSIS_PROMPTS.md
│   └── ADR/
│       ├── 0001-path-build-vs-clone.md
│       ├── 0002-multi-provider-llm.md
│       └── 0003-hybrid-search-strategy.md
│
├── .claude/
│   ├── CLAUDE.md                       ← project-level instructions
│   └── skills/
│       ├── archivio-parlante-dev/
│       │   └── SKILL.md
│       └── archivio-parlante-test/
│           └── SKILL.md
│
├── benchmarks/
│   ├── ingest_bench.py              ← 50 contratti in parallelo
│   ├── query_bench.py               ← latency p50/p95/p99
│   ├── hallucination_eval.py        ← gold-set domande
│   └── reports/
│
└── docs/
    ├── ARCHITECTURE.md
    ├── RUNBOOK.md
    └── CONTRACT_ANALYSIS_PROMPTS.md   ← prompt italiani forensic
```

---

## Fase -1 — Bootstrap Repository & Ricerca OSS

> ⚠️ **Questa fase precede tutte le altre ed è obbligatoria.** Non si scrive nemmeno una riga di Rust/Python/PHP finché Fase -1 non è chiusa con conferma esplicita dell'utente.

### Obiettivo

1. Creare il repository da zero (git init, licenze, README, struttura directory vuota).
2. Condurre ricerca OSS approfondita sui framework RAG candidati (§0.5) e sui MCP disponibili (§0.7).
3. Produrre una **Decision Matrix** che raccomandi uno dei tre percorsi: (A) clone + adattamento di un framework esistente, (B) ibrido (riusa componenti specifici di più OSS), (C) from-scratch puro.
4. Chiedere conferma utente via AskUserQuestion sul percorso scelto.

### Deliverable Fase -1

| File | Contenuto |
|---|---|
| `README.md` | Descrizione progetto, licenza, come avviare (placeholder finché Fase 0 non è fatta). |
| `LICENSE` | MIT (default). Da rivedere se il cliente (ente) richiede altro. |
| `.gitignore` | Rust + Python + Node + PHP + IDE + `.env` + `*.db` + `uploads/` |
| `.editorconfig` | Standard LF, 4 spaces, charset UTF-8. |
| `docs/00-decision-matrix.md` | Matrice OSS confronto (vedi template sotto). |
| `docs/01-architecture-vision.md` | Vision document: problem statement, stakeholder, constraint hardware, SLO. |
| `docs/02-oss-research-report.md` | Report ricerca OSS: per ogni framework in §0.5 → stars, ultimo commit, licenza, coverage del problema, effort di adattamento stimato. |
| `docs/03-mcp-plugin-inventory.md` | Risultato di `search_mcp_registry` e `search_plugins` → elenco MCP/plugin adottati. |
| `docs/ADR/0001-path-build-vs-clone.md` | Architecture Decision Record sulla scelta A/B/C. |
| `.claude/skills/archivio-parlante-dev/SKILL.md` | Skill Cowork con regole di coding, naming, licenze ammesse. |
| `.claude/CLAUDE.md` | Project-level instructions per Claude Code (lingue preferite, path, DB name = `archivio_parlante_x`, stile commit). |

### Template `docs/00-decision-matrix.md`

```markdown
# Decision Matrix — Approccio di Costruzione

## Candidati valutati
1. Clone + adattamento di <framework scelto>
2. Ibrido (componenti da N framework)
3. From-scratch con Rust core + Python worker + PHP gateway

## Dimensioni di valutazione (peso)
- Time-to-MVP (25%)
- Aderenza a zero-hallucination (20%)
- Performance su 50+ contratti paralleli (15%)
- Manutenibilità lungo termine (15%)
- Fit con hardware 8 GB VRAM (10%)
- Coerenza con stack Rust (10%)
- Licenza / compliance (5%)

## Punteggio (1-5 per dimensione)
| Dimensione | Peso | Clone | Ibrido | From-scratch |
|---|---|---|---|---|
| ... | ... | ... | ... | ... |
| **Totale ponderato** | 100% | X.X | X.X | X.X |

## Raccomandazione
<Claude Code scrive qui la sua raccomandazione motivata>

## Rischi della raccomandazione
<elenco rischi>

## Escalation all'utente
<domande che Claude Code pone via AskUserQuestion prima di procedere>
```

### 🧠 Prompt per Claude Code — FASE -1

````
Sei Senior Solutions Architect e R&D Lead su un nuovo progetto greenfield chiamato "Archivio Parlante" — un sistema RAG enterprise per analisi forense di contratti aziendali italiani con zero allucinazioni.

REGOLE NON NEGOZIABILI (leggi §0 del PIANO_IMPLEMENTAZIONE_RUST_PYTHON.md):
- Zero-cost / zero-budget: solo OSS, API a pagamento opt-in
- Open Source First: prima di scrivere codice, cerca progetti OSS da clonare/adattare
- Ask-First: chiedi conferma prima di scelte irreversibili
- Cowork-native: usa skills, MCP, plugin, subagents invece di scrivere infrastruttura custom

IL TUO COMPITO IN FASE -1:

1. BOOTSTRAP REPO (nessun codice applicativo ancora):
   - `git init` nella root del progetto
   - **Setup Git Flow** (come da §0.8 step 8):
     - Crea branch `main` (primo commit iniziale con README + LICENSE).
     - Crea branch `develop` partendo da `main`: `git checkout -b develop`.
     - Proteggi `main` e `develop` (se GitHub/GitLab: branch protection rules con require PR + CI green + 1 review).
     - Tutto il lavoro successivo avviene su feature branch `feature/fase-<N>-<slug>` partendo da `develop`.
   - Crea README.md placeholder, LICENSE (MIT), CHANGELOG.md (vuoto, con header "Keep a Changelog" format), .gitignore comprensivo, .editorconfig.
   - Crea **`.github/PULL_REQUEST_TEMPLATE.md`** con checklist degli 8 step di §0.8 e §0.9 (la PR non si approva senza tutte le voci spuntate).
   - Crea **`.github/workflows/ci.yml`** minimale (verrà espanso in Fase 5): jobs `rust-test`, `python-test`, `php-test`, `frontend-test`, `security-scan` (cargo audit, pip-audit, composer audit, npm audit, trivy image).
   - Crea struttura directory VUOTA: engine-rust/, engine-python/, php-gateway/, frontend/, docs/, docs/ADR/, .claude/skills/, .github/workflows/.
   - Crea `.claude/CLAUDE.md` con:
     - Lingua: italiano per docs/UI/commenti di dominio, inglese per codice/commit/log tecnici.
     - DB: `archivio_parlante_x`.
     - Stile commit: Conventional Commits (`feat|fix|refactor|perf|docs|test|chore|ci|build|security`).
     - Licenze ammesse: MIT/Apache-2.0/BSD/MPL-2.0. Vietate senza AskUserQuestion: GPL/AGPL/commerciali.
     - Branch strategy: feature branch da `develop`, PR obbligatoria, merge solo dopo CI verde + checklist 8-step completa.
     - Comandi di test: `make test-all` (orchestrare tutti i test layer).
   - Crea skill `.claude/skills/archivio-parlante-dev/SKILL.md` con coding standards (Rust clippy strict `-D warnings`, Python ruff+mypy strict, PHP PHPStan level 8, React+TS strict, no `unwrap/print/echo/console.log` in produzione).
   - Crea skill `.claude/skills/archivio-parlante-test/SKILL.md` con checklist testing (unit + integration + E2E + coverage > 80%).

2. RICERCA MCP (obbligatorio prima di scrivere qualsiasi client):
   Chiama in sequenza:
   - mcp__mcp-registry__search_mcp_registry(keywords: ["qdrant", "vector database"])
   - mcp__mcp-registry__search_mcp_registry(keywords: ["ollama", "local llm"])
   - mcp__mcp-registry__search_mcp_registry(keywords: ["docker compose"])
   - mcp__mcp-registry__search_mcp_registry(keywords: ["mysql", "database"])
   - mcp__plugins__search_plugins(query: "rust clippy")
   - mcp__plugins__search_plugins(query: "python ruff mypy")
   - mcp__plugins__search_plugins(query: "php phpstan")
   Documenta i risultati in docs/03-mcp-plugin-inventory.md indicando quali adotti.

3. RICERCA OSS RAG (obbligatorio prima di decidere l'architettura):
   Per OGNI framework in §0.5 del PIANO (Verba, Quivr, kotaemon, Danswer/Onyx, AnythingLLM, Open WebUI, Cheshire Cat, Haystack, RAGFlow, LlamaIndex), produci UNA riga in docs/02-oss-research-report.md con:
   - Repo URL, stars attuali, data ultimo commit, licenza
   - Coverage del nostro problema (parsing contratti italiani / hybrid search / multi-provider LLM / knowledge graph legale / citation enforcement): voti 0-3 per ciascuno
   - Effort stimato di adattamento in giorni-uomo per farlo funzionare sul nostro caso
   - Blocker noti (es. "Verba è legato a Weaviate, noi vogliamo Qdrant")

4. DECISION MATRIX:
   Scrivi docs/00-decision-matrix.md seguendo il template nel PIANO. Tre candidati:
   A. Clone + adattamento del MIGLIORE framework dal report
   B. Ibrido: riusa componenti specifici (es. parser RAGFlow + UI Open WebUI + chunker LlamaIndex) + Rust core custom + PHP gateway custom
   C. From-scratch come da §2 e §4 del PIANO (Rust + Python + PHP + React)
   Assegna punteggi ponderati, scrivi raccomandazione motivata, documenta rischi.

5. ADR:
   Scrivi docs/ADR/0001-path-build-vs-clone.md formato ADR standard (context, decision, consequences).

6. DOMANDA FINALE ALL'UTENTE:
   Al termine usa AskUserQuestion per chiedere:
   "Confermi di procedere con il Percorso <A|B|C>? Qui i trade-off principali: <riassunto 3 righe>"
   Opzioni: [Procedi con A] [Procedi con B] [Procedi con C] [Rivedi report e chiedi di più]

NON PROCEDERE a Fase 0 finché l'utente non conferma esplicitamente il percorso.

Rispondi in italiano per documenti e UI, inglese per codice/commit/log.
````

### Acceptance criteria Fase -1

- [ ] `docs/02-oss-research-report.md` esiste e copre tutti e 10 i framework con voti.
- [ ] `docs/03-mcp-plugin-inventory.md` elenca almeno i MCP ricercati per Qdrant/Ollama/Docker/MySQL.
- [ ] `docs/00-decision-matrix.md` ha punteggi ponderati e raccomandazione.
- [ ] `docs/ADR/0001-path-build-vs-clone.md` esiste in formato ADR.
- [ ] `.claude/CLAUDE.md` e skill `archivio-parlante-dev` esistono.
- [ ] L'utente ha risposto all'AskUserQuestion finale con una scelta esplicita.

---

## Fase 0 — Setup Infrastruttura Docker Compose

### Obiettivo
Creare la base infrastrutturale con tutti i 6 servizi orchestrati. Verificare che ogni container parta, comunichi con gli altri via rete interna Docker e abbia persistenza.

### Deliverable
- `docker-compose.yml` funzionante
- Cartelle `engine-rust/` e `engine-python/` scaffolate con Dockerfile minimali "hello world"
- `Makefile` con comandi quotidiani
- Rete Docker `archivio_net` che collega tutti i servizi
- Volumi persistenti per MySQL, Qdrant, Ollama models

### 🧠 Prompt per Claude Code — FASE 0

````
Stai costruendo da zero (greenfield) il progetto "Archivio Parlante" — un sistema RAG enterprise per analisi forense di contratti italiani, hardware target RTX 4070 Laptop 8 GB VRAM + 32 GB RAM. La Fase -1 (bootstrap repo + ricerca OSS + decision matrix) è già completata e l'utente ha confermato il percorso architetturale.

Leggi prima `.claude/CLAUDE.md` e `docs/ADR/0001-path-build-vs-clone.md` per allinearti alla decisione architetturale presa in Fase -1.

Il tuo compito in questa fase è SOLO creare l'infrastruttura Docker Compose — niente codice applicativo dentro Rust o Python ancora. Devi:

1. Creare nella root del progetto un file `docker-compose.yml` con questi 6 servizi collegati a una rete interna chiamata `archivio_net`:
   - `php-gateway`: immagine `php:8.2-apache`, espone host **9080:80**. Dipende da mysql e rust-engine.
   - `rust-engine`: build da `./engine-rust/Dockerfile`, espone porta 8090. Dipende da qdrant, ollama, python-worker.
   - `python-worker`: build da `./engine-python/Dockerfile`, espone porta 8091. Supporta opzionalmente GPU via `deploy.resources.reservations.devices` (commentalo se non disponibile).
   - `qdrant`: immagine `qdrant/qdrant:v1.12.4`, espone porta 6333 (REST) e 6334 (gRPC), volume persistente `qdrant_data:/qdrant/storage`.
   - `ollama`: immagine `ollama/ollama:latest`, espone porta 11434, volume `ollama_models:/root/.ollama`. Supporto GPU commentato.
   - `mysql`: immagine `mysql:8.0`, database `archivio_parlante_x` (come da `.claude/CLAUDE.md`, creato da phpMyAdmin in produzione), utente `root` senza password in dev, volume `mysql_data:/var/lib/mysql`. Monta `./db/migrations:/docker-entrypoint-initdb.d:ro`.
   - `redis`: immagine `redis:7-alpine`, porta 6379. Per rate limiting.

2. Creare `.env` nella root con variabili condivise:
   - `MYSQL_HOST=mysql`, `MYSQL_DB=archivio_parlante_x`, `MYSQL_USER=root`, `MYSQL_PASSWORD=`
   - `OLLAMA_URL=http://ollama:11434`
   - `OLLAMA_MODEL_CHAT=qwen2.5:7b-instruct-q4_K_M`           # default 8 GB VRAM compatibile
   - `OLLAMA_MODEL_CHAT_SMALL=qwen2.5:3b-instruct-q4_K_M`     # per task massivi low-latency
   - `OLLAMA_MODEL_CHAT_HEAVY=qwen2.5:14b-instruct-q4_K_M`    # opzionale, offload CPU se abilitato
   - `OLLAMA_MODEL_EMBED=nomic-embed-text`
   - `# Provider cloud OPT-IN (lascia vuoti per default zero-cost)`
   - `ANTHROPIC_API_KEY=`
   - `GOOGLE_API_KEY=`
   - `OPENAI_API_KEY=`
   - `DEEPSEEK_API_KEY=`
   - `OPENROUTER_API_KEY=`
   - `GROQ_API_KEY=`
   - `DAILY_COST_BUDGET_EUR=0.00`   # default zero: provider a pagamento bloccati finché admin non alza il budget
   - `QDRANT_URL=http://qdrant:6333`
   - `RUST_ENGINE_URL=http://rust-engine:8090`
   - `PYTHON_WORKER_URL=http://python-worker:8091`
   - `REDIS_URL=redis://redis:6379`

3. Creare scaffolding minimale (solo hello-world "pronti a essere riempiti"):
   - `engine-rust/Cargo.toml` con dipendenze `tokio`, `axum`, `tower-http`, `reqwest`, `serde`, `serde_json`, `tracing`, `tracing-subscriber`, `anyhow`, `thiserror`, `qdrant-client`.
   - `engine-rust/src/main.rs` che avvia axum su `0.0.0.0:8090` con una singola rotta `GET /health` → `{"status":"ok","service":"rust-engine"}`.
   - `engine-rust/Dockerfile` multi-stage (build con `rust:1.82`, runtime `debian:bookworm-slim` con ca-certificates).
   - `engine-python/requirements.txt` con `fastapi`, `uvicorn[standard]`, `pydantic`, `httpx`. Altro lo aggiungiamo in Fase 2.
   - `engine-python/app/main.py` FastAPI minimale con `GET /health` → `{"status":"ok","service":"python-worker"}`.
   - `engine-python/Dockerfile` da `python:3.11-slim`, `pip install -r requirements.txt`, avvio uvicorn su 8091.

4. Creare `Makefile` con target:
   - `up`, `down`, `logs`, `ps`, `rebuild-rust`, `rebuild-python`, `ollama-pull` (lancia `docker exec ollama ollama pull` per i 3 modelli), `mysql-shell`, `health` (curl ai 3 /health).

5. Creare `db/migrations/008_graph_and_jobs.sql` con:
   - `ap_graph_nodes (id, kb_id, doc_id, entity_type, label, properties JSON, created_at)`
   - `ap_graph_edges (id, kb_id, source_node_id, target_node_id, relation_type, properties JSON, created_at)`
   - `ap_ingest_jobs (id, doc_id, kb_id, status ENUM('queued','parsing','chunking','embedding','done','failed'), progress TINYINT, error_msg TEXT, created_at, updated_at)`
   - `ap_contract_analyses (id, kb_id, doc_ids JSON, question TEXT, answer_json JSON, created_at)`  (per salvare le analisi multi-contratto)

Requisiti finali:
- Dopo `make up`, tutti e 6 i container devono essere UP.
- `make health` deve restituire 200 da php-gateway, rust-engine, python-worker, e HTTP 200 da qdrant e ollama.
- NON scrivere ancora logica applicativa di parsing, chunking o embedding: le fasi successive lo faranno.

Verifica alla fine facendo girare `docker compose config` per validare la sintassi e commenta ogni servizio nel YAML in italiano spiegando il suo ruolo.
````

---

## Fase 1 — Engine Rust (Core Processing)

Questa è la fase più lunga: 7 step. Ogni step ha un prompt auto-contenuto.

### 1.1 — Scaffolding, config, error handling, Ollama client

#### 🧠 Prompt per Claude Code — FASE 1.1

````
Lavori dentro `engine-rust/` del progetto Archivio Parlante. Leggi prima il file `PIANO_IMPLEMENTAZIONE_RUST_PYTHON.md` alla radice per il contesto architetturale completo.

Espandi lo scaffolding esistente creando:

1. `engine-rust/src/config.rs`:
   - Struct `Config` caricata da variabili d'ambiente (tramite `std::env` o crate `envy`).
   - Campi: `listen_addr` (default `0.0.0.0:8090`), `ollama_url`, `ollama_model_chat`, `ollama_model_chat_small`, `ollama_model_embed`, `qdrant_url`, `python_worker_url`, `mysql_url`, `chunk_size` (default 800 tokens), `chunk_overlap` (default 15%), `top_k_dense` (30), `top_k_sparse` (30), `top_k_rerank` (5), `max_concurrent_embeddings` (16), `max_concurrent_llm` (8).
   - Funzione `Config::from_env() -> anyhow::Result<Self>`.

2. `engine-rust/src/errors.rs`:
   - Enum `AppError` con varianti: `Parse`, `Qdrant`, `Ollama`, `PythonWorker`, `NotFound`, `BadRequest(String)`, `Internal(anyhow::Error)`.
   - Implementa `IntoResponse` di axum: mappa errori a codici HTTP corretti con body JSON `{"error":"...","code":"..."}`.
   - `thiserror::Error` per variants con `#[from]` dove utile.

3. `engine-rust/src/providers/` — **architettura multi-provider** (vedi §13.3 per il trait completo):
   - `src/providers/mod.rs`: definisce il trait `LlmProvider` (async_trait) con metodi `name()`, `available_models()`, `is_available()`, `chat()`, `chat_stream()`, `embed()`.
   - `src/providers/types.rs`: struct condivise `ChatRequest`, `ChatResponse`, `StreamEvent`, `ModelInfo`, `Usage`, `Cost`.
   - `src/providers/ollama.rs`: **prima implementazione** concreta del trait (default zero-cost):
     - Struct `OllamaProvider` con `reqwest::Client` interno e `tokio::sync::Semaphore` per limitare richieste concorrenti.
     - Implementa `LlmProvider`:
       - `embed(&self, texts, model)` → POST `/api/embeddings` per ogni testo, parallelizzato con `futures::stream::iter().buffer_unordered(16)`.
       - `chat(&self, req)` → POST `/api/chat` stream=false.
       - `chat_stream(&self, req)` → POST con `stream: true`, parsing NDJSON in `BoxStream<StreamEvent>`.
       - `is_available()` → GET `/api/tags` con timeout 1s.
     - Retry automatico con backoff esponenziale (max 3 tentativi) per errori transitori.
   - `src/providers/registry.rs`: `LlmRegistry` che mantiene `HashMap<String, Arc<dyn LlmProvider>>`. In Fase 1.1 contiene solo Ollama; gli altri 13 provider (Anthropic, Google, OpenAI, DeepSeek, …) sono aggiunti in un sub-step dedicato di §13 dopo che il core RAG è stabile.
   - **Perché trait generico già in Fase 1.1**: evita un refactor doloroso dopo. Tutti i consumatori downstream (HybridSearcher, SelfRagEvaluator, MultiContractComparator, IntentClassifier) usano `Arc<dyn LlmProvider>` anziché `Arc<OllamaClient>`.

4. `engine-rust/src/clients/qdrant.rs`:
   - Struct `QdrantWrapper` basata su `qdrant_client::Qdrant`.
   - Metodi:
     - `ensure_collection(&self, kb_id: &str) -> Result<()>` → crea collection se non esiste, con vettori `dense` (768 cosine) e `sparse` (on-disk). Nome collection: `ap_kb_{kb_id}`.
     - `upsert_chunks(&self, kb_id: &str, points: Vec<PointStruct>) -> Result<()>`.
     - `search_dense(&self, kb_id: &str, vector: Vec<f32>, top_k: u64) -> Result<Vec<ScoredPoint>>`.
     - `search_sparse(&self, kb_id: &str, indices: Vec<u32>, values: Vec<f32>, top_k: u64) -> Result<Vec<ScoredPoint>>`.
     - `delete_by_doc_id(&self, kb_id: &str, doc_id: &str) -> Result<()>`.

5. `engine-rust/src/clients/python_worker.rs`:
   - Struct `PythonWorkerClient` con `reqwest::Client`.
   - Metodi (solo firma + chiamate HTTP, il Python verrà in Fase 2):
     - `parse_document(&self, file_path: &str, mime_type: &str) -> Result<ParsedDocument>`
     - `contextualize_chunks(&self, doc_summary: &str, chunks: Vec<String>) -> Result<Vec<String>>`
     - `rerank(&self, query: &str, candidates: Vec<(String, String)>) -> Result<Vec<(usize, f32)>>` (ritorna (index, score))
     - `extract_knowledge_graph(&self, text: &str) -> Result<KnowledgeGraph>`
   - Struct `ParsedDocument { text, tables: Vec<Table>, metadata, page_count }`, `KnowledgeGraph { nodes, edges }` — serde.

6. Aggiorna `main.rs`:
   - Inizializza `tracing_subscriber` con livello da env `RUST_LOG`.
   - Carica `Config` all'avvio, panica se fallisce.
   - Costruisce uno `AppState` (Arc<...>) che contiene `Config`, `LlmRegistry` (inizialmente solo Ollama), `QdrantWrapper`, `PythonWorkerClient`.
   - Registra middleware: CORS permissivo (per dev), `tower_http::trace::TraceLayer`, timeout 120s.
   - Monta router `/health` + placeholder `/ingest` `/query` `/compare_contracts` che ritornano `501 Not Implemented` con messaggio chiaro.

7. Scrivi un test unitario in `engine-rust/tests/ollama_smoke.rs` che verifica `OllamaProvider::is_available()` su URL da env. Marca come `#[ignore]` se non c'è Ollama disponibile. Aggiungi anche un test che verifica che il trait object `Arc<dyn LlmProvider>` sia utilizzabile correttamente (compile-check).

Requisiti:
- Tutto deve compilare con `cargo build --release` senza warning.
- `cargo clippy -- -D warnings` deve passare.
- Usa `tracing::info!`, `tracing::warn!`, `tracing::error!` ovunque — no `println!`.
- Ogni errore uscito dai client deve contenere contesto utile (`.context("...")` di anyhow).

Verifica finale: rebuild del container Rust + `curl http://localhost:8090/health` deve rispondere 200.
````

---

### 1.2 — Chunker semantico con Contextual Retrieval

#### 🧠 Prompt per Claude Code — FASE 1.2

````
Continua lavorando in `engine-rust/`. Riferisci al piano globale in `PIANO_IMPLEMENTAZIONE_RUST_PYTHON.md` sezione 2.2 (flusso ingestion).

Implementa il sistema di chunking semantico + contextual retrieval. File da creare:

1. `engine-rust/src/models/chunk.rs`:
   - Struct `Chunk { id: Uuid, doc_id: String, chunk_idx: usize, text: String, contextual_text: Option<String>, token_count: usize, start_offset: usize, end_offset: usize, metadata: serde_json::Value }`.
   - `Chunk::new(...)` costruttore con auto-generazione UUID.

2. `engine-rust/src/utils/tokenizer.rs`:
   - Wrapper su `tiktoken-rs` usando `cl100k_base` (compatibile con la maggior parte dei modelli LLM).
   - `count_tokens(text: &str) -> usize`
   - `split_by_token_limit(text: &str, limit: usize) -> Vec<String>` — split preciso per limite token.

3. `engine-rust/src/chunker/semantic.rs`:
   - Struct `SemanticChunker { chunk_size: usize, overlap_pct: f32 }`.
   - Metodo principale: `fn chunk(&self, text: &str, doc_id: &str) -> Vec<Chunk>`.
   - Strategia:
     1. **Step A — Split per headers Markdown**: regex `^(#{1,6})\s+.*$` → sezioni. Ogni sezione mantiene il proprio header come prefisso.
     2. **Step B — Split per clausole numerate legali**: regex `^(\d+\.\d*|\bArt\.?\s*\d+|\bArticolo\s+\d+|\bCAPO\s+[IVX]+|\bSezione\s+\d+)` su start-of-line. Specifico per contratti italiani.
     3. **Step C — Se sezione > chunk_size**: split per frasi usando regex `[.!?]\s+(?=[A-ZÀ-Ü])` (Unicode-aware italiano).
     4. **Step D — Overlap**: ogni chunk (tranne il primo) prepend gli ultimi `chunk_size * overlap_pct` caratteri del chunk precedente.
     5. **Step E — Metadata arricchita**: `chunk_idx`, `section_header` (se trovato), `is_clause_start` (se match Art/Sezione), `offsets`.
   - Gestione casi edge: testo vuoto, singolo paragrafo enorme (force-split per token), testo senza headers (fallback per paragrafi \n\n).

4. `engine-rust/src/chunker/contextual.rs`:
   - Struct `ContextualRetrievalEnricher { llm: Arc<dyn LlmProvider>, model: String, semaphore: Arc<Semaphore> }`.
   - Metodo:
     ```
     async fn enrich(&self, full_document_text: &str, chunks: &mut [Chunk]) -> Result<()>
     ```
   - Logica (basata su tecnica Anthropic Contextual Retrieval):
     1. Se il documento è < 2000 tokens: skip (troppo corto per valere).
     2. Se > 8000 tokens: genera un **summary del documento** (chiamata LLM a `qwen2.5:7b-instruct`) da ~500 tokens, da riutilizzare per tutti i chunks.
     3. Per ogni chunk, in parallelo (max 16 concurrent con semaphore), chiama LLM con prompt:
        ```
        <document_summary>{summary}</document_summary>
        <chunk>{chunk.text}</chunk>
        
        Fornisci un breve contesto (1-2 frasi, max 80 tokens) che situi questo chunk nel documento intero per migliorare il retrieval. Rispondi SOLO con il contesto, senza preamboli.
        ```
     4. Prepend il contesto generato al chunk: `chunk.contextual_text = Some(format!("{}\n\n{}", context, chunk.text))`.
   - Caching: usa `dashmap::DashMap` per non ricalcolare se lo stesso chunk viene riprocessato.

5. Test in `engine-rust/tests/chunker_test.rs`:
   - Test A: chunking di un contratto italiano di esempio (crealo inline, ~3000 parole con articoli numerati) verifica che chunks siano 5–15, nessuno > chunk_size*1.2 tokens, overlap presente tra consecutivi.
   - Test B: testo sotto 500 token → chunk singolo, no overlap.
   - Test C: testo con solo headers senza contenuto → no panic, chunks vuoti filtrati.
   - Test contextual enrichment marcato `#[ignore]` (richiede Ollama live).

Requisiti:
- Il chunker DEVE preservare l'ordinamento logico: se un chunk contiene "Art. 5" e il successivo "Art. 6", non devono mai essere invertiti.
- Il contextual retrieval è la singola ottimizzazione più importante secondo Anthropic (-49% errori): implementala con cura, con fallback graceful se LLM fallisce (il chunk passa senza contextual_text).
- Usa `tracing::debug!` per loggare lunghezze chunk, `tracing::warn!` per fallback.

Verifica: `cargo test --release` deve passare tutti i test non-ignored. Verifica manuale con cargo run di un binario di demo in `engine-rust/examples/chunk_demo.rs` che stampa i chunks di un file passato.
````

---

### 1.3 — Ingestion pipeline end-to-end

#### 🧠 Prompt per Claude Code — FASE 1.3

````
Continua in `engine-rust/`. Implementa la pipeline completa di ingestion che orchestra parsing → chunking → contextual → embedding → storage Qdrant.

File da creare:

1. `engine-rust/src/models/document.rs`:
   - Struct `Document { doc_id: String, kb_id: String, source_name: String, mime_type: String, file_path: String, tags: Vec<String>, metadata: serde_json::Value, created_at: DateTime<Utc> }`.
   - Struct `IngestRequest { doc_id, kb_id, file_path, source_name, mime_type, tags }`, `IngestResponse { doc_id, chunks_indexed, processing_ms, entities_extracted }`.

2. `engine-rust/src/routes/ingest.rs`:
   - Handler `async fn ingest(State(state), Json(req): Json<IngestRequest>) -> Result<Json<IngestResponse>, AppError>`.
   - Algoritmo:
     1. **Validate**: `file_path` esiste? `kb_id` non vuoto? mime_type supportato?
     2. **Ensure Qdrant collection**: `state.qdrant.ensure_collection(&req.kb_id).await?`.
     3. **Parse**:
        - Se mime_type è PDF / docx complesso / immagine / scan → chiama `state.python_worker.parse_document(&file_path, &mime_type).await?`.
        - Se è TXT / MD / CSV / JSON → leggi con `tokio::fs::read_to_string` direttamente in Rust (no round-trip Python).
        - Ritorna `String full_text`.
     4. **Chunk**: `SemanticChunker::new(cfg).chunk(&full_text, &doc_id)`.
     5. **Contextual enrich**: `ContextualRetrievalEnricher::enrich(&full_text, &mut chunks).await?` — non-fatal se fallisce (log warn, continua senza contextual).
     6. **Embed batch**: per ogni chunk, testo = `chunk.contextual_text.unwrap_or(chunk.text)`. Chiama `state.ollama.embed_batch(&texts).await?`.
     7. **Sparse vectors (BM25)**: in parallelo, genera vettori sparsi via `tantivy` (vedi step 3 sotto) — non blocca l'embed.
     8. **Upsert Qdrant**: costruisci `Vec<PointStruct>` con:
        - `id` = UUID del chunk
        - `vector::named` con `dense` + `sparse`
        - `payload` = {doc_id, chunk_idx, text (originale, NON contextual), source_name, kb_id, tags, indexed_at, section_header, is_clause_start, token_count}
     9. **Async knowledge graph**: spawn task `tokio::spawn` che chiama `state.python_worker.extract_knowledge_graph(&full_text).await` e salva in MySQL via un nuovo client `state.mysql` (crea struct `MysqlClient` con `sqlx` se non esiste).
     10. **Response**: `IngestResponse { doc_id, chunks_indexed: chunks.len(), processing_ms, entities_extracted }`.

3. `engine-rust/src/utils/sparse.rs`:
   - Helper che usa `tantivy` in-memory per generare vettori sparsi BM25 compatibili Qdrant.
   - Funzione `fn build_sparse_vector(text: &str, language: &str) -> (Vec<u32>, Vec<f32>)`.
   - Usa stemmer italiano di tantivy (`Language::Italian`).
   - Alternative: se tantivy per sparse è complesso, usa un semplice TF-IDF con vocabolario condiviso persistito in Qdrant payload o file locale. Documenta la scelta.

4. `engine-rust/src/clients/mysql.rs`:
   - Wrapper su `sqlx::MySqlPool`.
   - Metodi:
     - `save_graph(kb_id, doc_id, nodes, edges) -> Result<()>` — insert in `ap_graph_nodes` e `ap_graph_edges`.
     - `update_job_status(job_id, status, progress, error) -> Result<()>`.
     - `get_documents(kb_id) -> Result<Vec<Document>>`.

5. Aggiorna `Cargo.toml` con `sqlx = { version = "0.8", features = ["mysql", "runtime-tokio", "chrono", "json"] }` e `tantivy = "0.22"`.

6. Test integrazione `engine-rust/tests/integration_ingest.rs`:
   - Con tutti i container up (flag via env `INTEGRATION_TESTS=1`, altrimenti skip), posta un contratto .md fittizio di 2000 parole a `/ingest`, verifica risposta 200 e che Qdrant abbia almeno 3 chunks indexed (query di count).

Requisiti critici:
- **Transazione logica**: se qualcosa fallisce dopo l'upload del file, devi poter rollback. Implementa una `rollback_doc_id(kb_id, doc_id)` che cancella i chunks già inseriti in Qdrant e il record del documento.
- **Idempotenza**: se lo stesso `doc_id` è già indicizzato, cancella i chunks vecchi PRIMA di upsertare i nuovi (`delete_by_doc_id` poi `upsert`).
- **Logging dettagliato**: ogni step logga durata in ms, numero chunks, errori.
- **Nessun .unwrap()** in codice di produzione. Usa `?` e `.context("...")`.

Verifica:
- `cargo build --release` senza warning.
- Test integrazione passa con stack Docker up.
- Tempo ingestion di un PDF legale di 20 pagine: target < 8 secondi (escluso knowledge graph async).
````

---

### 1.4 — Hybrid Search + Reranker + RAG query pipeline

#### 🧠 Prompt per Claude Code — FASE 1.4

````
Continua in `engine-rust/`. Implementa la query pipeline RAG con hybrid search, reranker, prompt building e streaming SSE.

File da creare:

1. `engine-rust/src/models/query.rs`:
   - Struct `QueryRequest { question: String, kb_id: String, session_id: Option<String>, doc_ids: Option<Vec<String>>, top_k: Option<usize>, stream: Option<bool>, comparison_mode: Option<bool> }`.
   - Struct `Citation { doc_id, source_name, chunk_idx, text_quote, score, section_header }`.
   - Struct `QueryResponse { answer, session_id, citations: Vec<Citation>, confidence: f32, verified: bool, model_used, processing_ms }`.

2. `engine-rust/src/rag/hybrid_search.rs`:
   - Struct `HybridSearcher { qdrant: Arc<QdrantWrapper>, llm: Arc<dyn LlmProvider> }`.
   - Metodo `async fn search(&self, kb_id: &str, query: &str, doc_filter: Option<&[String]>, top_k_dense: u64, top_k_sparse: u64) -> Result<Vec<Candidate>>`.
   - Algoritmo:
     1. **HyDE (Hypothetical Document Embedding)**: chiama LLM piccolo con prompt "Rispondi in 2 frasi a questa domanda come se fossi un contratto: {query}". Genera risposta ipotetica.
     2. **Dual embedding**: embed(query) + embed(hypothetical_answer), poi media pesata (0.6 query + 0.4 hypothetical).
     3. **Dense search**: `qdrant.search_dense(kb_id, vector, top_k_dense)` con filtro opzionale `doc_id IN (...)`.
     4. **Sparse search**: genera sparse BM25 della query, `qdrant.search_sparse(...)`.
     5. **Reciprocal Rank Fusion (RRF)**:
        ```
        rrf_score(item) = Σ 1 / (k + rank_in_list(item))   con k=60
        ```
        Unisci dense e sparse con RRF, ritorna top 20 candidati unici.
   - Struct `Candidate { chunk_id, doc_id, chunk_idx, text, source_name, section_header, rrf_score, dense_score, sparse_score }`.

3. `engine-rust/src/rag/reranker.rs`:
   - Struct `Reranker { python_worker: Arc<PythonWorkerClient>, top_k_final: usize }`.
   - Metodo `async fn rerank(&self, query: &str, candidates: Vec<Candidate>) -> Result<Vec<Candidate>>`.
   - Invia `(query, [cand.text])` al Python BGE-reranker-v2-m3. Ordina per score cross-encoder. Ritorna top_k_final.
   - Fallback: se Python fallisce, ritorna candidates ordinati per rrf_score (log warn).

4. `engine-rust/src/rag/prompt_builder.rs`:
   - Funzione `fn build_forensic_prompt(question: &str, candidates: &[Candidate]) -> (String, String)` → (system, user).
   - System prompt (italiano) FORENSE rigoroso:
     ```
     Sei l'Archivio Parlante, un assistente AI forense specializzato nell'analisi
     di contratti aziendali. Rispondi ESCLUSIVAMENTE basandoti sui passaggi
     documentali forniti nel contesto. REGOLE INVIOLABILI:
     
     1. Non inventare MAI informazioni. Se la risposta non è interamente
        contenuta nel contesto fornito, rispondi esattamente:
        "Le informazioni richieste non sono presenti nei documenti analizzati."
     
     2. Cita SEMPRE testualmente (tra virgolette «…») i passaggi rilevanti,
        seguiti dal riferimento [SORGENTE: {nome_doc} — chunk {idx}].
     
     3. Quando confronti più contratti, struttura la risposta con intestazioni
        per ciascun contratto e una sezione finale "Confronto" con una tabella
        in Markdown se pertinente.
     
     4. Non fare inferenze non esplicitamente supportate. Distingui sempre
        tra "il contratto afferma X" e "è probabile che Y".
     
     5. Il tuo output DEVE essere un JSON valido con questo schema:
        {
          "answer": "testo markdown della risposta",
          "citations": [
            {"doc_id": "...", "chunk_idx": N, "text_quote": "...", "source_name": "..."}
          ],
          "confidence": 0.0-1.0,
          "information_gaps": ["lista di ciò che manca, se qualcosa"]
        }
     ```
   - User prompt: concatena i candidates formattati `[doc_id=X, chunk=Y, source=Z, section=W]\n{text}\n\n` + la domanda in fondo.

5. `engine-rust/src/rag/self_rag.rs`:
   - Struct `SelfRagEvaluator { llm: Arc<dyn LlmProvider>, model: String }`.
   - Metodo `async fn evaluate(&self, question: &str, answer: &str, citations: &[Citation], candidates: &[Candidate]) -> Result<GroundingReport>`.
   - Struct `GroundingReport { grounded: bool, hallucinations: Vec<String>, invalid_citations: Vec<usize>, suggestion: String, confidence: f32 }`.
   - Implementazione:
     1. Chiama LLM con prompt valutatore: data la domanda, i chunks originali e la risposta, verifica:
        - Ogni affermazione nella risposta è supportata da almeno un chunk?
        - Ogni citazione `text_quote` è effettivamente presente VERBATIM nel chunk corrispondente? (verifica programmatica in Rust tramite `contains` case-insensitive, oltre al giudizio LLM).
        - Il JSON è ben formato?
     2. Ritorna `GroundingReport`.
   - Se `grounded == false`: il chiamante può ri-generare con prompt più restrittivo (max 2 retry).

6. `engine-rust/src/routes/query.rs`:
   - Handler non-streaming `async fn query_blocking(...)`:
     1. Intent classification (opzionale, step successivo) — per ora assume rag_query.
     2. Hybrid search → candidates.
     3. Rerank → top 5.
     4. Build prompt → (system, user).
     5. LLM generate (non-stream).
     6. Parse JSON output, se parsing fallisce → riprova 1 volta con prompt "Il tuo precedente output non era JSON valido. Riprova.".
     7. Self-RAG evaluate.
     8. Se `grounded=false` e retry < 2: rigenera con prompt più stringente.
     9. Costruisci `QueryResponse`.
   - Handler streaming `async fn query_stream(...)` che ritorna `axum::response::sse::Sse`:
     1. Search + rerank uguali (non-stream).
     2. LLM `generate_stream` → forward ogni token come SSE event `data: {"token":"..."}\n\n`.
     3. Al termine, invia evento finale `data: {"done":true, "citations":[...], "verified": true/false}\n\n`.
     4. Self-RAG post-hoc (non blocca stream, ma aggiunge metadata all'evento finale).

7. Test in `engine-rust/tests/integration_query.rs`:
   - Dato un KB con 3 contratti fittizi indicizzati (usa fixture Fase 1.3), poni domanda "Quali sono le penali previste?" e verifica:
     - Risposta contiene almeno una citazione.
     - `verified = true`.
     - Latency < 3 secondi con Ollama qwen2.5:7b.
   - Test allucinazione forzata: domanda su argomento NON nei documenti, verifica che la risposta contenga esattamente "Le informazioni richieste non sono presenti...".

Requisiti:
- Il prompt forense è CRITICO per zero allucinazioni: rispetta il testo esatto sopra.
- Self-RAG deve poter essere disabilitato via env `SELF_RAG_ENABLED=false` per benchmark di baseline.
- Tutto asincrono: un'unica richiesta non deve mai bloccare worker thread.

Verifica finale:
- `cargo test --release --features integration` passa.
- `curl -N http://localhost:8090/query -X POST -d '{"question":"...", "kb_id":"test", "stream":true}'` streamma tokens in tempo reale.
````

---

### 1.5 — Multi-Contract Comparison Engine

#### 🧠 Prompt per Claude Code — FASE 1.5

````
Continua in `engine-rust/`. Implementa il motore di confronto multi-contratto, la feature più differenziante dell'app per utilizzo enterprise/legale.

Obiettivo: dato un set di doc_ids (es. NDA-2023.pdf, NDA-2024.pdf, NDA-2025.pdf), l'utente può chiedere "Confronta le clausole di riservatezza" e ottenere:
- Una tabella Markdown con righe = clausole, colonne = contratti, celle = testo citato con riferimenti.
- Una sezione "Differenze chiave" (narrata).
- Una sezione "Raccomandazioni" (neutrale, senza pareri legali).
- Zero allucinazioni: ogni cella della tabella DEVE avere la sua citazione verbatim.

File da creare:

1. `engine-rust/src/rag/multi_contract.rs`:
   - Struct `MultiContractComparator { hybrid: Arc<HybridSearcher>, reranker: Arc<Reranker>, llm: Arc<dyn LlmProvider>, self_rag: Arc<SelfRagEvaluator> }`.
   - Metodo principale:
     ```
     async fn compare(
         &self,
         kb_id: &str,
         doc_ids: Vec<String>,
         question: String,
     ) -> Result<ComparisonResult>
     ```
   - Algoritmo:
     1. **Parallel per-document retrieval**: lancia `tokio::join_all` per ogni doc_id:
        - Hybrid search filtrata `doc_id == X` con top_k_dense=15, sparse=15.
        - Rerank su top 8 candidati per documento.
     2. **Cross-document aggregation**: raccoglie 8 × N candidati. Unifica rispettando il doc_id.
     3. **Aspect extraction** (via LLM piccolo qwen2.5:7b):
        - Prompt: "Data questa domanda comparativa e questi estratti di contratto, elenca le 4-8 dimensioni/aspetti da confrontare. Rispondi in JSON: {aspects: [...]}"
     4. **Per-aspect per-contract evidence collection**:
        - Per ogni (aspect × doc_id): filtra i chunks del doc_id → invia al LLM con prompt "Estrai il testo esatto che riguarda l'aspetto '{aspect}'. Se non c'è, ritorna null."
        - Parallelo via `join_all`.
     5. **Synthesis prompt FORENSIC**:
        - System: forte istruzione di non inventare e di usare SOLO le evidenze raccolte.
        - User: tabella strutturata pre-compilata + domanda originale.
        - Output JSON schema:
          ```
          {
            "summary": "...",
            "comparison_table": [
              {"aspect": "...", "cells": [{"doc_id": "X", "source_name": "...",
                 "text_quote": "...", "chunk_idx": N, "present": true/false}]}
            ],
            "key_differences": ["...", "..."],
            "information_gaps": ["..."],
            "confidence": 0.95
          }
          ```
     6. **Self-RAG validation stringente**: ogni `text_quote` nella tabella DEVE essere verificabile via `text_quote in chunk.text`. Se anche una sola cella fallisce, rigenera quella cella specificamente (non l'intera tabella).
     7. **Markdown rendering**: converti il JSON in Markdown leggibile con tabella + sezioni.

2. `engine-rust/src/routes/compare.rs`:
   - Handler `async fn compare_contracts(State(state), Json(req): Json<CompareRequest>) -> Result<Json<CompareResponse>, AppError>`.
   - `CompareRequest { kb_id, doc_ids: Vec<String>, question: String, save_analysis: Option<bool> }`.
   - `CompareResponse { markdown_result, structured: ComparisonResult, processing_ms, analysis_id: Option<String> }`.
   - Se `save_analysis=true`: salva in MySQL `ap_contract_analyses` (vedi migration 004).
   - Supporta anche un endpoint streaming `/compare_contracts/stream` che invia eventi SSE:
     - `phase: retrieving_doc_X`
     - `phase: extracting_aspects`
     - `phase: building_comparison`
     - `phase: validating`
     - `done` con payload finale

3. `engine-rust/src/models/comparison.rs`:
   - Tutte le struct ComparisonResult, ComparisonCell, ComparisonAspect con serde + validazione custom.

4. Test `engine-rust/tests/integration_compare.rs`:
   - Fixture: 2 contratti NDA italiani (fittizi, creali inline nel test) con clausole leggermente diverse (durata, penali, foro competente).
   - Indicizzali via `/ingest`.
   - Chiama `/compare_contracts` con domanda "Confronta durata, penali e foro competente".
   - Asserzioni:
     - Tabella ha 3 aspects (durata, penali, foro).
     - Ogni cella `present=true` ha `text_quote` non vuoto e verificabile nel chunk.
     - Nessuna allucinazione: se il contratto A non ha clausola penali, la cella deve avere `present=false` e `text_quote=null`.

Requisiti critici:
- **ZERO allucinazioni nella tabella**: se il Self-RAG rileva anche una cella non grounded, rigenera fino a 3 volte. Se continua a fallire, marca la cella `present=false` e aggiungi l'errore in `information_gaps`.
- **Parallelismo aggressivo**: usa `tokio::spawn` e `join_all` ovunque possibile. Target: confronto di 5 contratti di 20 pagine in < 15 secondi.
- **Safety**: il prompt finale deve esplicitamente vietare pareri legali ("non fornire consigli legali, riporta solo il contenuto testuale dei contratti").

Verifica:
- Test integrazione passa.
- Usa il comando `time make bench-compare` per benchmarkare.
- Risposta Markdown è ben formattata e ogni citazione è rintracciabile.
````

---

### 1.6 — Intent Router + Export API REST complete

#### 🧠 Prompt per Claude Code — FASE 1.6

````
Continua in `engine-rust/`. Finalizza l'engine aggiungendo:

1. **Intent classification** (opzionale ma migliora efficienza):
   - `engine-rust/src/rag/intent.rs`.
   - Struct `IntentClassifier` con `llm: Arc<dyn LlmProvider>`.
   - Metodo `async fn classify(&self, question: &str, kb_has_docs: bool) -> Result<Intent>`.
   - Enum `Intent { RagQuery, ContractComparison, Summarize, GeneralChat }`.
   - Prompt al LLM piccolo che ritorna JSON `{"intent": "..."}`. Fallback: RagQuery se parsing fail.
   - Integra in `routes/query.rs`: se intent=ContractComparison e `req.doc_ids.len()>=2` → redirect a multi-contract. Se intent=Summarize → skip hybrid search, passa interi docs (o chunks ordinati). Se GeneralChat → skip RAG, diretto a LLM.

2. **API REST completa** — aggiungi handlers in `engine-rust/src/routes/`:
   - `GET /kb/{kb_id}/documents` → lista documenti della KB (da MySQL).
   - `DELETE /kb/{kb_id}/documents/{doc_id}` → cancella doc da Qdrant + MySQL + grafo.
   - `GET /kb/{kb_id}/graph?doc_ids=X,Y` → ritorna nodi/archi del knowledge graph (JSON {nodes, edges}), opzionalmente filtrato per doc.
   - `GET /kb/{kb_id}/stats` → conteggio docs, chunks, collection size, node count, edges.
   - `POST /admin/reindex/{kb_id}` → job in background che riembedda tutti i chunks (utile dopo cambio modello embedding).

3. **Middleware di sicurezza interna**:
   - `engine-rust/src/middleware/internal_auth.rs`: richiede header `X-Internal-Token` = valore da `.env` `RUST_ENGINE_INTERNAL_TOKEN`. Il PHP gateway passerà questo header. Proteggere tutte le rotte tranne `/health`.
   - `engine-rust/src/middleware/rate_limit.rs`: limita richieste per IP via `tower_governor`. 100 req/min per IP.

4. **Observability**:
   - Endpoint `GET /metrics` in formato Prometheus (usa `prometheus` crate): metrics di latency per endpoint, counter errori, gauge request in-flight, counter LLM calls.
   - Logs strutturati JSON con `tracing-subscriber::fmt().json()`.

5. **OpenAPI spec**:
   - Usa `utoipa` per generare spec automatica da attributi sui handlers.
   - Esponi `GET /openapi.json` e `GET /docs` (Swagger UI).

6. **Graceful shutdown**:
   - Gestisci `SIGTERM` per completare richieste in-flight prima di morire.
   - Chiudi pool Qdrant/MySQL pulitamente.

7. Test integrazione finale `engine-rust/tests/integration_full.rs`:
   - Scenario end-to-end: ingest 3 docs → list → query → compare → delete → verify delete.
   - Usa wiremock o mock Python worker per test che non richiedono container up.

Requisiti:
- README in `engine-rust/README.md` con: setup, env vars, API endpoints, esempi curl.
- `cargo clippy -- -D warnings` pulito.
- `cargo audit` nessuna vulnerability critica.
- Binario finale < 50 MB (release strip).

Verifica:
- `curl http://localhost:8090/docs` mostra Swagger.
- `curl http://localhost:8090/metrics` ritorna metriche Prometheus.
- Tutti i test passano.
````

---

## Fase 2 — Python AI Worker

### 2.1 — Setup FastAPI + Parser avanzati

#### 🧠 Prompt per Claude Code — FASE 2.1

````
Lavori ora in `engine-python/`. Leggi il piano globale `PIANO_IMPLEMENTAZIONE_RUST_PYTHON.md` sezione 2.2 e 3.2.

Implementa il servizio Python di parsing documenti (PDF complessi, OCR, docx con layout).

1. Aggiorna `engine-python/requirements.txt`:
   ```
   fastapi==0.115.0
   uvicorn[standard]==0.32.0
   pydantic==2.9.2
   httpx==0.28.0
   python-multipart==0.0.12
   unstructured[pdf,docx,image]==0.16.0
   pymupdf==1.24.12
   pdfplumber==0.11.4
   pytesseract==0.3.13
   Pillow==11.0.0
   python-magic==0.4.27
   tenacity==9.0.0
   structlog==24.4.0
   ```

2. Aggiorna `engine-python/Dockerfile` da `python:3.11-slim`:
   - Installa dipendenze di sistema: `tesseract-ocr`, `tesseract-ocr-ita`, `poppler-utils`, `libmagic1`, `libgl1`, `libglib2.0-0`.
   - Copia requirements, `pip install --no-cache-dir -r requirements.txt`.
   - Copia `app/` e avvia `uvicorn app.main:app --host 0.0.0.0 --port 8091 --workers 2`.

3. `engine-python/app/config.py`:
   - Pydantic `Settings` con env: `OLLAMA_URL`, `OLLAMA_MODEL_CHAT_SMALL`, `TESSERACT_LANGS='ita+eng'`, `MAX_FILE_SIZE_MB=200`.

4. `engine-python/app/models/schemas.py`:
   - `ParseRequest { file_path: str, mime_type: str, extract_tables: bool = true, perform_ocr: bool = true }`
   - `Table { page: int, rows: list[list[str]], header: list[str] | None }`
   - `ParseResponse { text: str, tables: list[Table], metadata: dict, page_count: int, language: str | None, extraction_method: str }`
   - `ContextualizeRequest { document_summary: str | None, chunks: list[str], max_context_tokens: int = 80 }`
   - `ContextualizeResponse { contextual_chunks: list[str] }`
   - `RerankRequest { query: str, candidates: list[str], top_k: int = 5 }`
   - `RerankResponse { scores: list[float], indices: list[int] }`
   - `KGExtractRequest { text: str, language: str = 'it' }`
   - `KGNode { id, type, label, properties }`
   - `KGEdge { source_id, target_id, type, properties }`
   - `KGExtractResponse { nodes: list[KGNode], edges: list[KGEdge] }`

5. `engine-python/app/services/pdf_parser.py`:
   - Classe `PdfParser`.
   - Metodo `parse(file_path) -> ParseResponse`:
     1. **Primo tentativo**: `unstructured.partition.pdf.partition_pdf(filename=..., strategy='hi_res', infer_table_structure=True, languages=['ita','eng'])`. Categorizza elementi (NarrativeText, Title, Table, ListItem).
     2. Ricostruisci `text` concatenando in ordine, con markdown headers per Title.
     3. Estrai tabelle con conversione a `list[list[str]]`.
     4. **Fallback se unstructured fallisce** (es. PDF scansionato): usa PyMuPDF `doc = fitz.open(file_path)`, per ogni pagina:
        - `page.get_text()` se ha testo.
        - Altrimenti `page.get_pixmap()` → Tesseract OCR.
     5. **Fallback tabelle**: `pdfplumber` per estrazione tabellare dedicata.
     6. Metadata: `page_count`, `author`, `creation_date`, `tesseract_used: bool`, `extraction_method`.

6. `engine-python/app/services/ocr_service.py`:
   - Funzione `ocr_image(image_path, langs='ita+eng') -> str` con `pytesseract.image_to_string(Image.open(path), lang=langs, config='--psm 1')`.
   - Pre-processing con PIL: converti a grayscale, aumenta contrasto se necessario.
   - Timeout 60s via `tenacity.retry(stop_after_attempt=2)`.

7. `engine-python/app/routers/parse.py`:
   - `POST /parse` → accetta `ParseRequest`, chiama `PdfParser` o routing per mime_type:
     - `application/pdf` → PdfParser
     - `application/vnd.openxmlformats-officedocument.wordprocessingml.document` (docx) → unstructured docx
     - `image/*` → OCR diretto
     - Altro mime → 415 Unsupported
   - Gestione errori: 422 se file non accessibile, 500 + log se parsing crasha.
   - Tempo max per parse: 300s (timeout handler).

8. `engine-python/app/main.py`:
   - FastAPI app con:
     - `GET /health` (già esistente)
     - Include router parse.
     - Middleware CORS permissivo (per test; production sarà limitato a rust-engine).
     - Global exception handler che logga con `structlog` e ritorna JSON consistente.
     - Lifespan handler che pre-carica modelli pesanti se serve (non ancora in questa fase).

9. Test in `engine-python/tests/test_parse.py`:
   - Fixture PDF semplice generato inline con reportlab (se installabile, altrimenti incluso in `fixtures/`).
   - Test PDF testuale → verifica text non vuoto, metadata corretta.
   - Test OCR su immagine con testo italiano → verifica parole chiave.
   - Usa `pytest-asyncio` e `httpx.AsyncClient` per test FastAPI.

Requisiti:
- Gestisci PDF di 100+ pagine senza OOM (stream elaborazione per pagina).
- Logga ogni parse con timing e metodo estrazione usato.
- Path file: il Rust deve montare lo stesso volume `/storage/ai/uploads` del Python, oppure passare bytes via multipart. **Scegli la prima opzione** (volume condiviso in docker-compose) per evitare serializzazione inutile.

Aggiorna `docker-compose.yml`: aggiungi volume condiviso `uploads_volume` montato sia su `rust-engine` che `python-worker` in `/shared/uploads`.

Verifica:
- `docker compose up --build python-worker` parte.
- `curl -X POST http://localhost:8091/parse -H 'Content-Type: application/json' -d '{"file_path":"/shared/uploads/test.pdf","mime_type":"application/pdf"}'` ritorna ParseResponse valido.
- Latency PDF 20 pagine: < 5s testuale, < 30s OCR.
````

---

### 2.2 — Reranker BGE + Contextual Retrieval + Knowledge Graph

#### 🧠 Prompt per Claude Code — FASE 2.2

````
Continua in `engine-python/`. Completa il worker con i 3 servizi ML critici per precisione: BGE Reranker, Contextual Retrieval, Knowledge Graph Extraction.

1. Aggiungi a `requirements.txt`:
   ```
   FlagEmbedding==1.3.2
   torch==2.5.0
   transformers==4.46.0
   spacy==3.8.0
   https://github.com/explosion/spacy-models/releases/download/it_core_news_lg-3.8.0/it_core_news_lg-3.8.0.tar.gz
   ```
   Se GPU non disponibile, istruisci `torch` CPU: edit Dockerfile con `pip install torch --index-url https://download.pytorch.org/whl/cpu`.

2. `engine-python/app/services/reranker.py`:
   - Classe `BGEReranker` (singleton, caricato al lifespan startup).
   - `__init__`: `self.model = FlagReranker('BAAI/bge-reranker-v2-m3', use_fp16=torch.cuda.is_available())`.
   - Metodo `rerank(query: str, candidates: list[str], top_k: int) -> tuple[list[int], list[float]]`:
     - Costruisci coppie `[[query, cand] for cand in candidates]`.
     - `scores = self.model.compute_score(pairs, normalize=True)`.
     - Ordina indici per score desc, ritorna top_k indici + score.
   - Batching: se candidates > 32, processa in chunks per non esaurire VRAM.

3. `engine-python/app/routers/rerank.py`:
   - `POST /rerank` → chiama singleton BGEReranker, ritorna `RerankResponse`.
   - Latency target: < 200ms su 20 candidati con GPU, < 1s CPU.

4. `engine-python/app/services/contextual.py`:
   - Classe `ContextualGenerator`.
   - Metodo `async contextualize(document_summary: str | None, chunks: list[str]) -> list[str]`:
     1. Se `document_summary` è None: genera un summary da ~400 tokens chiamando Ollama `qwen2.5:7b-instruct` su un sample dei chunks (primi 5 + ultimi 3 concatenati).
     2. Per ogni chunk (parallelo via `asyncio.gather` con `asyncio.Semaphore(16)`):
        - Prompt:
          ```
          <document_summary>
          {summary}
          </document_summary>
          
          <chunk>
          {chunk}
          </chunk>
          
          In 1-2 frasi brevi (max 80 token), spiega come questo chunk si situa
          nel documento intero, per aiutare un sistema di ricerca a trovarlo
          meglio. Rispondi SOLO con il contesto, senza preamboli né virgolette.
          ```
        - Chiama Ollama via `httpx.AsyncClient`.
     3. Ritorna `[f"{context}\n\n{chunk}" for context, chunk in zip(contexts, chunks)]`.
   - Gestione errori per chunk: se un chunk fallisce, ritorna `chunk` senza contesto (non blocca il batch).

5. `engine-python/app/routers/contextualize.py`:
   - `POST /contextualize` → ContextualGenerator.
   - Timeout 120s per batch di 50 chunks.

6. `engine-python/app/services/kg_extractor.py`:
   - Classe `LegalKnowledgeGraphExtractor`.
   - `__init__`: carica `nlp = spacy.load('it_core_news_lg')` (singleton).
   - Metodo `extract(text: str) -> tuple[list[KGNode], list[KGEdge]]`:
     1. **Step A — NER con spaCy**:
        - Estrai entità base: PER (persone), ORG (organizzazioni), LOC (luoghi), MISC.
        - Crea nodi con `type` mappato: PER→PERSON, ORG→ORGANIZATION, LOC→LOCATION.
     2. **Step B — Estrazione entità legali specifiche via LLM** (chiamata a Ollama qwen2.5:7b):
        - Prompt: "Dal testo fornito, estrai TUTTE le entità legali/contrattuali in JSON:
          - parties: parti contrattuali
          - dates: date rilevanti (formato ISO)
          - amounts: importi monetari
          - clauses: numeri/titoli di articoli (es. 'Art. 5 - Riservatezza')
          - jurisdictions: foro competente, legge applicabile
          - penalties: penali citate
          Output: {parties:[...], dates:[...], amounts:[...], clauses:[...], jurisdictions:[...], penalties:[...]}"
        - Merge con NER spaCy: deduplica per label+type.
     3. **Step C — Estrazione relazioni**:
        - Prompt LLM: "Data questa lista di entità {...} e il testo, estrai relazioni in JSON:
          [{source: <label1>, type: <SIGNS|OBLIGATED_TO|PAYS|GOVERNED_BY|EXPIRES_ON|REFERS_TO>, target: <label2>, evidence: '<citazione>'}]"
        - Risolve source/target ai node_id.
     4. Ritorna `(nodes, edges)`.
   - Deduplicazione: hash(type+label.lower()) per evitare duplicati cross-chunk.

7. `engine-python/app/routers/knowledge_graph.py`:
   - `POST /extract_entities_relations` → KGExtractor.
   - Timeout 180s (può essere lungo su testi > 20k tokens).

8. Aggiorna `app/main.py` lifespan:
   ```python
   @asynccontextmanager
   async def lifespan(app: FastAPI):
       # Startup: preload heavy models
       app.state.reranker = BGEReranker()
       app.state.kg_extractor = LegalKnowledgeGraphExtractor()
       app.state.contextual = ContextualGenerator(settings.OLLAMA_URL, settings.OLLAMA_MODEL_CHAT_SMALL)
       yield
       # Shutdown: cleanup se serve
   ```

9. Test:
   - `tests/test_rerank.py`: dato query e 5 candidati con uno ovviamente pertinente, verifica che sia in top 1.
   - `tests/test_contextual.py` (integration, skip se no Ollama): verifica che output abbia N elementi e ognuno contenga il chunk originale.
   - `tests/test_kg.py`: testo contratto fittizio, verifica che nodes contenga almeno PERSON e DATE.

Requisiti:
- Startup time: il loading di BGE + spaCy può richiedere 30s. Accettabile, ma documentalo nel README.
- Memory footprint: target < 4 GB RAM totali.
- Concurrent requests: FastAPI con 2 workers di uvicorn, ma BGE è GPU-bound → usa una `asyncio.Lock` o thread executor per il rerank pesante.
- Logging: ogni endpoint logga input size, output size, duration_ms, model usato.

Verifica:
- `docker compose up --build python-worker` parte in <60s.
- 4 endpoint `/parse`, `/rerank`, `/contextualize`, `/extract_entities_relations` rispondono.
- `python -m pytest tests/` passa.
````

---

## Fase 3 — PHP Gateway (greenfield, Slim 4)

> ℹ️ **Nota progetto greenfield**: il PHP Gateway **non esiste prima di questa fase** — lo si scaffolda qui da zero. Non c'è alcun "refactor" o "deprecazione" del vecchio RAG PHP perché non esiste vecchio codice. Il gateway è sottile per design: fa solo auth/utenti/sessioni/rate-limit/audit + proxy verso Rust Engine.

### 3.1 — Bootstrap PHP Gateway + Rust Engine Client

#### 🧠 Prompt per Claude Code — FASE 3.1

````
Stai creando da zero il PHP Gateway per Archivio Parlante. Il progetto è greenfield — la directory `php-gateway/` esiste ma è vuota (solo scaffold da Fase 0). Leggi `.claude/CLAUDE.md` e `docs/ADR/0001-path-build-vs-clone.md` per il contesto architetturale.

Obiettivo: il PHP è un **gateway sottile** con responsabilità limitate ad auth, utenti, sessioni, rate limit, audit log, file upload (staging su volume condiviso `/shared/uploads`), proxy verso Rust Engine. Il RAG/parsing/chunking/embedding NON si fa mai in PHP: tutto va a Rust Engine.

0. **Scaffold Slim 4 minimo** (se non già fatto in Fase 0):
   - `composer init` in `php-gateway/` con `slim/slim:^4`, `slim/psr7`, `php-di/php-di`, `vlucas/phpdotenv`, `guzzlehttp/guzzle`, `firebase/php-jwt`, `monolog/monolog`, `predis/predis`.
   - Crea struttura: `public/index.php`, `src/Controller/`, `src/Service/Engine/`, `src/Middleware/`, `config/routes.php`, `config/container/services.php`, `tests/Unit/`, `.env.example`.
   - Entry point `public/index.php` con DI container e AppFactory.

1. **Crea** `src/Service/Engine/RustEngineClient.php`:
   - Namespace: `ArchivioParlante\Service\Engine\RustEngineClient`.
   - Costruttore: base URL, internal token (da .env), logger, timeout default 300s.
   - Usa `Guzzle` (`composer require guzzlehttp/guzzle` se non presente).
   - Metodi pubblici:
     - `ingest(array $payload): array` → POST `/ingest`, ritorna response decoded.
     - `query(array $payload): array` → POST `/query` non-stream.
     - `queryStream(array $payload, callable $onToken, callable $onDone): void` → POST `/query` con `stream=true`, parsa SSE e chiama callback.
     - `compareContracts(array $payload): array` → POST `/compare_contracts`.
     - `listDocuments(string $kbId): array` → GET `/kb/{kb_id}/documents`.
     - `deleteDocument(string $kbId, string $docId): void`
     - `getGraph(string $kbId, array $docIds = []): array` → GET `/kb/{kb_id}/graph?doc_ids=...`.
     - `getStats(string $kbId): array`
     - `health(): bool`
   - Header automatici: `X-Internal-Token`, `Content-Type: application/json`.
   - Gestione errori: re-throw come `EngineClientException` con status code e messaggio utile.
   - Retry logic: 3 tentativi con backoff per errori 5xx transitori.

2. **Aggiorna DI container** `config/container/services.php`:
   - Registra `RustEngineClient` come singleton con env `RUST_ENGINE_URL`, `RUST_ENGINE_INTERNAL_TOKEN`.
   - Inietta nei controller che lo usano.

3. **Crea** i controller greenfield:
   - `src/Controller/AuthController.php` — login JWT, refresh, logout.
   - `src/Controller/KbController.php` — CRUD knowledge base + permessi per utente.
   - `src/Controller/ChatController.php`:
     - `ingest()`: valida file upload (MIME, size), salva in `/shared/uploads/`, inserisce record in `ap_documents` con `status='processing'`, chiama `rustEngine->ingest([doc_id, kb_id, file_path, source_name, mime_type, tags])`. Su successo `status='indexed'`; su fallimento `status='error'`, delete file, ritorna 500 con errore pulito.
     - `chat()`: non-stream, chiama `rustEngine->query([...])`, salva messaggio utente e risposta in `ap_chat_messages`, ritorna JSON.
     - `streamChat()`: `$rustEngine->queryStream($payload, fn($tok) => emitSseToken($tok), fn($done) => emitSseDone($done));` scrive su stdout con header SSE corretti.
     - `compareContracts()`: endpoint `POST /api/chat/compare`, accetta `{kb_id, doc_ids, question}`, chiama `rustEngine->compareContracts(...)`, salva analisi in `ap_contract_analyses`.
     - `listDocuments()`, `deleteDocument()`, `getKnowledgeGraph()` → proxy a Rust.
   - `src/Controller/AdminProvidersController.php` — CRUD provider LLM (vedi §13.9), gestione API keys, budget giornaliero/mensile.

4. **Routes** in `config/routes.php`:
   - `POST /api/auth/login`, `POST /api/auth/refresh`, `POST /api/auth/logout`
   - `GET|POST|DELETE /api/kb` (knowledge base CRUD)
   - `POST /api/kb/{kb_id}/documents` (upload → proxy a Rust `/ingest`)
   - `GET /api/kb/{kb_id}/documents`, `DELETE /api/kb/{kb_id}/documents/{doc_id}`
   - `POST /api/chat` (non-stream → proxy a Rust `/query`)
   - `POST /api/chat/stream` (SSE → proxy a Rust `/query` stream)
   - `POST /api/chat/compare` (→ proxy a Rust `/compare_contracts`)
   - `GET /api/kb/{kb_id}/graph`, `GET /api/kb/{kb_id}/stats`
   - `GET /api/admin/providers`, `PUT /api/admin/providers/{id}` (abilitazione provider LLM cloud — vedi §13.9)
   - `GET /api/health`

5. **`.env.example`** completo con:
   ```
   APP_ENV=dev
   APP_DEBUG=true
   JWT_SECRET=generate-a-strong-random-secret
   RUST_ENGINE_URL=http://rust-engine:8090
   RUST_ENGINE_INTERNAL_TOKEN=generate-a-strong-random-token-here
   MYSQL_HOST=mysql
   MYSQL_DB=archivio_parlante_x
   MYSQL_USER=root
   MYSQL_PASSWORD=
   REDIS_URL=redis://redis:6379
   SHARED_UPLOADS_PATH=/shared/uploads
   ```

6. **Migrations MySQL** in `db/migrations/` (Phinx o plain SQL):
   - `001_create_users.sql`: `ap_users`, `ap_user_sessions`, `ap_user_api_keys`.
   - `002_create_kb.sql`: `ap_knowledge_bases`, `ap_kb_users` (membership).
   - `003_create_documents.sql`: `ap_documents` (id, kb_id, source_name, status, indexed_at).
   - `004_create_chat.sql`: `ap_chat_sessions`, `ap_chat_messages`.
   - `005_create_analyses.sql`: `ap_contract_analyses` (storicizza compare multi-contratto).
   - `006_create_audit.sql`: `ap_audit_log`.
   - `007_create_providers.sql`: `ap_llm_providers`, `ap_llm_usage` (vedi §13.10).

7. **`README.md`** del gateway con istruzioni `docker compose up` + `composer install` + primi step.

8. **Test** in `tests/Unit/Service/Engine/RustEngineClientTest.php`:
   - Mocka Guzzle (`GuzzleHttp\Handler\MockHandler`).
   - Verifica che headers corretti siano inviati.
   - Verifica retry su 503.
   - Verifica parsing SSE.
   - Verifica che `/api/chat/compare` fa proxy corretto.

Requisiti:
- Se l'engine Rust è giù, restituisci 503 con messaggio italiano `Il motore di analisi non è al momento disponibile. Riprova tra qualche istante.`.
- Logga ogni chiamata proxied con correlation ID (`X-Request-ID` passato a Rust).
- PSR-12 coding style, PHPStan level 8 pulito.

Verifica:
- `composer test` passa.
- `composer analyse` (PHPStan level 8) pulito.
- `curl http://localhost:9080/health` ritorna `{"status":"ok","rust_engine":"ok"}`.
- Upload di un PDF via `POST /api/kb/{kb_id}/documents` → Rust ingest → query funziona.
````

---

## Fase 4 — Frontend Multi-Contract UI

### 🧠 Prompt per Claude Code — FASE 4

````
Stai creando da zero il frontend di Archivio Parlante nella cartella `frontend/` (già scaffoldata vuota in Fase 0). Leggi `.claude/CLAUDE.md` e `docs/ADR/0001-path-build-vs-clone.md` per il contesto architetturale deciso in Fase -1.

Stack: React 18 + Vite + TypeScript + Zustand + TailwindCSS + shadcn/ui + react-markdown + remark-gfm + react-router-dom + axios + @tanstack/react-query.

Obiettivo: costruire l'intera UI greenfield con queste sezioni principali: Login, Knowledge Base picker, Upload documenti, Chat standard, **Contract Comparison Viewer (feature killer)**, **Model Selector multi-provider** (§13.8), Knowledge Graph viewer, Admin panel (gestione provider + cost tracking).

0. **Scaffold Vite + TS**:
   - `npm create vite@latest . -- --template react-ts` dentro `frontend/`.
   - Installa dipendenze: `npm i zustand axios @tanstack/react-query react-router-dom react-markdown remark-gfm lucide-react tailwindcss postcss autoprefixer`.
   - Configura Tailwind, shadcn/ui init, ESLint + Prettier + TS strict.
   - Crea `src/lib/api.ts` con client axios configurato su `VITE_API_BASE_URL` (default `/api`).
   - Auth store Zustand con JWT (refresh automatico su 401).

1. **Nuovo componente** `frontend/src/components/ContractComparison.tsx`:
   - UI: selezione multipla di documenti dalla sidebar (checkbox), input domanda, bottone "Confronta".
   - Stato via Zustand: `selectedDocIds: []`, `comparisonResult: null`, `comparisonLoading: false`.
   - Al click "Confronta" → chiama `POST /api/chat/compare` con `{kb_id, doc_ids, question}`.
   - Streaming delle fasi (usa SSE endpoint `/compare_contracts/stream` via EventSource):
     - Mostra progress bar con le fasi: "Retrieving docs", "Extracting aspects", "Building table", "Validating citations".
   - Rendering risultato:
     - Header con domanda + confidence badge.
     - Tabella Markdown renderizzata con react-markdown + remark-gfm.
     - Per ogni cella con `text_quote`: al hover mostra tooltip con `source_name` + chunk_idx + score.
     - Click su cella → apre `ContextViewer` con il chunk originale evidenziato.
     - Sezione "Differenze chiave" (narrata).
     - Sezione "Lacune informative" in warning colore giallo.
     - Footer: tempo elaborazione + bottone "Esporta PDF".

2. **Componente** `frontend/src/components/ContextViewer.tsx`:
   - Supporto per evidenziazione di più `text_quotes` simultaneamente (non solo uno).
   - Badge "✅ Verificato" o "⚠️ Non verificato" basato su `verified` del response Rust.
   - Mostra `confidence` come barra progressiva colorata.

3. **Componente** `frontend/src/components/ChatMessage.tsx`:
   - Rendering Markdown risposta con citazioni cliccabili (aprono ContextViewer).
   - Se risposta ha `verified: false`: avviso discreto "⚠️ Risposta non completamente verificata — valida le citazioni manualmente".
   - Se `information_gaps` non vuoto: expandable con "Informazioni mancanti dai documenti".

4. **Componente** `frontend/src/components/DocumentSelector.tsx`:
   - Reusable picker multi-selezione dei documenti, con ricerca, filtro per tag/tipo, count selezionati.
   - Usato sia in ContractComparison che in "Advanced Search" view.

5. **Componente** `frontend/src/components/ModelSelector.tsx` (vedi §13.8):
   - Dropdown con raggruppamento per provider (Ollama locale / Cloud premium / Cloud low-cost).
   - Badge "🟢 Free" per Ollama, "💰 Pay-per-token" per provider cloud.
   - Provider senza API key configurata appaiono disabilitati con tooltip "Configurare API key in Admin".
   - Mostra stima costo per 1000 token input/output accanto al nome modello.

6. **Layout principale** `frontend/src/App.tsx` con `react-router-dom`:
   - Route `/login` → LoginPage
   - Route `/` (protetta) → layout con sidebar (KB picker + Documents list) + main area con tabs:
     - Tab "💼 Chat"
     - Tab "🕸️ Graph 3D" (vis-network o 3d-force-graph)
     - Tab "⚖️ Confronto Contratti"
     - Tab "📊 Analisi"
   - Route `/admin` (solo admin) → AdminPanel (gestione provider LLM, cost tracking, utenti).

7. **API client** `frontend/src/lib/api.ts`:
   - Tutti gli endpoint del gateway PHP (§ Fase 3.1 punto 4).
   - `compareContracts({kb_id, doc_ids, question, stream})` → fetch o EventSource.
   - `exportComparisonPdf(analysis_id)` → download PDF.
   - `listProviders()`, `updateProvider(id, {enabled, api_key, daily_budget})`.

8. **Store** `frontend/src/store/archivioStore.ts`:
   - `currentKb`, `selectedDocIdsForComparison`, `comparisonResult`, `comparisonPhase`, `comparisonError`.
   - `selectedProvider`, `selectedModel`, `costSoFarToday`.
   - Actions: `toggleDocSelection`, `runComparison`, `clearComparison`, `setProviderModel`.

9. **Style**:
   - Tema dark neon professionale (colori ispirati a legal tech, contrasto AAA).
   - Tabella comparazione: bordi sottili, righe alternate, header sticky.
   - Celle `present=false`: fondo gray-950 con testo "—".

10. **Export PDF**: endpoint PHP `POST /api/chat/compare/{id}/export` che genera PDF via `dompdf` o `mpdf` con tabella + citazioni.

11. **Test** (Vitest + React Testing Library):
    - `ContractComparison.test.tsx`: mock API, verifica rendering tabella da fixture JSON.
    - `DocumentSelector.test.tsx`: verifica selezione multipla, filtro.
    - `ModelSelector.test.tsx`: verifica disabilitazione provider senza API key.

Requisiti:
- Accessibilità: aria-labels su tutto, focus management dopo submit.
- Responsive: tabella scrollabile orizzontalmente su mobile.
- Performance: se 10+ contratti, usa virtualizzazione (react-window) per la lista docs.
- Localizzazione: tutto in italiano, preparato per i18n futura (crea keys in `/frontend/src/i18n/it.json` ma senza libreria ancora).

Verifica:
- `npm run build` genera bundle < 500 KB gzipped.
- Demo: indicizza 3 PDF di contratto fittizi, seleziona tutti e 3, chiedi "Confronta durata e penali", verifica tabella corretta con citazioni cliccabili.
````

---

## Fase 5 — Testing, Benchmark, Hardening

### 🧠 Prompt per Claude Code — FASE 5

````
Lavori nella root del progetto. Obiettivo: garantire qualità enterprise con test E2E, benchmark oggettivi e security review.

1. **Benchmark suite** `benchmarks/`:
   - `benchmarks/ingest_bench.py`:
     - Dataset: 50 PDF di contratti (crea fixture fittizi con `reportlab` in `benchmarks/fixtures/generate_contracts.py`).
     - Misura: tempo totale, p50/p95/p99 per documento, memory peak, CPU e VRAM utilizzati.
     - Baseline target (tratti dal mercato): ingestion < 30s per PDF 50 pagine su hardware target, throughput ≥ 10 PDF/min in parallelo.
     - Output: `benchmarks/reports/ingest_YYYYMMDD.md` con grafici (matplotlib).
   - `benchmarks/query_bench.py`:
     - 100 query gold-set (create da te in `benchmarks/fixtures/queries.jsonl` con formato `{question, expected_doc_ids, expected_keywords}`).
     - Misura: latency p50/p95/p99, recall@5 (expected_doc_id in top 5 citations), keyword coverage nella risposta.
   - `benchmarks/hallucination_eval.py`:
     - 30 domande "trick" su argomenti NON presenti nei documenti.
     - Deve ritornare "Le informazioni richieste non sono presenti..." al 100%.
     - 30 domande valide con gold-answer: verifica similarity tra risposta e gold (usa embeddings cosine).
     - Report: % allucinazioni, precision, recall.
   - `benchmarks/concurrent_bench.py`:
     - 50 query simultanee via asyncio. Misura throughput (req/s) e tail latency.

2. **Test E2E** `tests/e2e/` (nuovo, con Playwright):
   - `npm install -D @playwright/test` nel frontend.
   - Test scenari:
     - Login → upload contratto → chat domanda semplice → verifica citazione.
     - Upload 3 contratti → apri tab Confronto → seleziona tutti → domanda → verifica tabella.
     - Streaming: invia domanda, verifica token arrivano progressivamente.
     - Chat con domanda trick: verifica messaggio di "informazioni non presenti".
   - Script `make test-e2e` che orchestra: `make up` → wait health → playwright run → teardown.

3. **Security hardening**:
   - `engine-rust/`:
     - `cargo audit` in CI.
     - Input sanitization su ogni endpoint (lunghezze max, char whitelist).
     - File path validation: rifiuta path traversal, symlinks, path fuori da `/shared/uploads`.
     - Rate limit per `kb_id` (max 100 query/min).
     - CORS strict in production (solo origin di PHP gateway).
   - `engine-python/`:
     - `safety check` in CI.
     - File size limit, MIME validation doppia.
     - Timeout su tutti i subprocess (Tesseract, unstructured).
   - `PHP`:
     - `composer audit`.
     - JWT secret randomico (≥ 32 byte) generato in produzione, mai committato.
     - `X-Internal-Token` randomico lungo (≥ 64 hex char) condiviso solo via secret manager.
     - Headers di sicurezza (X-Frame-Options, CSP, Referrer-Policy, Permissions-Policy) via middleware.
   - **Secrets management**: documenta in `docs/RUNBOOK.md` come generare segreti con `openssl rand -hex 32`.

4. **CI/CD** `.github/workflows/`:
   - `ci.yml`:
     - Job `rust`: `cargo test --release`, `cargo clippy -- -D warnings`, `cargo audit`.
     - Job `python`: `pytest`, `ruff check`, `mypy`, `safety check`.
     - Job `php`: `composer test`, `composer analyse`.
     - Job `integration`: `docker compose up -d`, wait health, run smoke tests, teardown.
   - `release.yml`: build multi-arch Docker images (linux/amd64 + linux/arm64), push a registry.

5. **Observability** `docs/OBSERVABILITY.md`:
   - Setup Grafana + Prometheus (aggiungi 2 container al docker-compose opzionali).
   - Dashboard JSON in `observability/grafana/dashboards/` per:
     - Latency per endpoint Rust.
     - Throughput Ollama.
     - Qdrant index size per kb_id.
     - Hallucination rate (da Self-RAG evaluator, counter custom).
   - Alerts: p95 latency > 5s, error rate > 1%, Ollama down.

6. **Load test** con k6 (`benchmarks/k6/`):
   - Scenario ramping da 10 a 100 VU su `/api/chat` con query realistiche.
   - Report con pass/fail su p95 < 3s.

7. **Runbook** `docs/RUNBOOK.md`:
   - Come scalare: aggiungere replica Rust/Python worker.
   - Come rebootare pulitamente (grace shutdown).
   - Come fare backup Qdrant (`docker exec qdrant ...`) e MySQL.
   - Come aggiornare modello Ollama (e reindexing).
   - Diagnosi problemi comuni: Ollama OOM, Qdrant disk full, deadlock Rust.

8. **Documentazione finale**:
   - `docs/ARCHITECTURE.md`: diagrammi mermaid, descrizione ogni servizio.
   - `docs/CONTRACT_ANALYSIS_PROMPTS.md`: libreria di prompt pre-configurati in italiano per casi d'uso legali (NDA, appalti, fornitura, licenze software, M&A).
   - Aggiorna `README.md` principale del progetto con quick start.

Requisiti:
- Tutti i test/benchmark devono essere riproducibili con un singolo `make bench-all`.
- Report generati in HTML leggibili.
- Zero test flaky: usa retry solo se strettamente necessario.

Verifica finale:
- `make bench-all` gira in < 30 minuti e produce report.
- Hallucination rate < 1% su gold-set.
- p95 query latency < 3s con qwen2.5:32b.
- p95 ingest 20-page PDF < 10s.
````

---

## 11. Prompt Master per Claude Code

> Se vuoi lanciare l'intero progetto greenfield in una singola sessione (Claude Code agentico), usa questo prompt di alto livello. Richiede permission `dangerously-skip-permissions` o approvazione interattiva, ma le **conferme utente obbligatorie a fine Fase -1** sono sempre rispettate.

### 🧠 Prompt Master

````
Sei Claude Code, un agente di sviluppo autonomo con ruolo di Senior Solutions Architect
& R&D Lead. Stai costruendo DA ZERO (greenfield) il progetto "Archivio Parlante" —
una piattaforma enterprise di analisi forense di contratti aziendali italiani con
zero allucinazioni e massima precisione, destinata a enti istituzionali.

La directory di lavoro è vuota (o contiene solo questo piano). Non c'è codice preesistente.

📖 DOCUMENTAZIONE DI RIFERIMENTO (leggila PRIMA di iniziare):
- `PIANO_IMPLEMENTAZIONE_RUST_PYTHON.md` — piano definitivo (questo documento),
  in particolare §0 (Principi Operativi) e §13 (Multi-Provider LLM).

🎯 OBIETTIVO FINALE:
Stack microservizi in Docker Compose:
- PHP Slim 4 (gateway sottile: auth + utenti + sessioni + rate limit + proxy)
- 🦀 Rust axum (core engine: chunking, hybrid search, RAG, multi-contract, multi-provider LLM)
- 🐍 Python FastAPI (parsing avanzato, BGE reranker, contextual retrieval, KG legale)
- Qdrant (vector DB con dense+sparse)
- Ollama (LLM + embedding LOCALI di default — modelli: qwen2.5:7b, qwen2.5:3b, nomic-embed-text)
- Provider cloud OPT-IN (Anthropic, Google, OpenAI, DeepSeek, OpenRouter, Groq, …)
- MySQL (archivio_parlante_x) + Redis
- Frontend React 18 + Vite + TS + Tailwind + Zustand

🛡️ PRINCIPI INVIOLABILI:
1. ZERO-COST DEFAULT: tutto gira gratis con Ollama locale. API a pagamento opt-in.
2. OPEN SOURCE FIRST: in Fase -1 valuta OSS esistenti prima di scrivere da zero.
3. ASK-FIRST: fermati e chiedi conferma via AskUserQuestion prima di scelte irreversibili
   (scelta framework, abilitazione provider a pagamento, schema DB).
4. ZERO ALLUCINAZIONI: Self-RAG + Citation enforcement + JSON schema obbligatorio.
5. MASSIMA PRECISIONE: Hybrid Search (dense+BM25) + RRF + BGE cross-encoder Reranker + Contextual Retrieval Anthropic + Knowledge Graph.
6. CONFRONTO MULTI-CONTRATTO: endpoint dedicato, N contratti in parallelo, tabella con citazioni verificate.
7. PRIVACY: tutto locale di default, nessuna fuga verso cloud senza consenso esplicito.
8. TEST COVERAGE: ogni modulo ha test unitari + integrazione + E2E.
9. HARDWARE: target 8 GB VRAM + 32 GB RAM (RTX 4070 Laptop). Nessun modello locale > 14B.

🗺️ ESECUZIONE FASATA + CICLO 8-STEP (vedi §0.8 e §0.9):
Esegui le fasi IN ORDINE, una alla volta. Al termine di OGNI fase devi applicare il
ciclo 8-step obbligatorio (Tech Lead → Dev+QA → Performance → Clean Code → Security
→ Docs → Git Flow). La fase è "CLOSED" solo quando la checklist §0.9 è 100% verde.

Per ogni fase, in ordine rigoroso:
1. **Tech Lead/PM**: pianifica i task con TodoWrite, esegui step-by-step.
2. **Dev + QA**: scrivi il codice + scrivi e lancia i test. 100% pass obbligatorio.
3. **Performance Engineer**: profila, ottimizza, riesegui i test. 100% pass.
4. **Clean Code Reviewer**: pulizia commenti, dead code, lint+format, riesegui i test. 100% pass.
5. **Security Engineer**: audit completo (OWASP ASVS L2), correggi vulnerabilità, riesegui audit + test. Genera `docs/SECURITY_AUDIT_<fase>.md`.
6. **Technical Writer**: aggiorna README, ARCHITECTURE, RUNBOOK, CHANGELOG, ADR.
7. **DevOps/Release**: feature branch da `develop`, commit atomici Conventional, PR con checklist 8-step, CI verde, merge.
8. Riepilogo allo user e attesa OK prima di iniziare la fase successiva.

Gestione interruzioni: se ti interrompi (errore, context limit, riavvio sessione),
**riprendi ESATTAMENTE dal punto di interruzione** leggendo lo stato git + TodoList.
MAI ricominciare dall'inizio. MAI saltare fasi o task.

FASE -1 — Bootstrap Repo + Ricerca OSS + Decision Matrix → **ASK USER per conferma percorso**
FASE 0  — Infrastruttura Docker Compose
FASE 1  — Engine Rust (step 1.1 → 1.6)
FASE 2  — Python AI Worker (step 2.1 → 2.2)
FASE 3  — PHP Gateway (greenfield Slim 4)
FASE 4  — Frontend Multi-Contract UI (greenfield React+TS)
FASE 5  — Testing, Benchmark, Hardening

Per ogni step, leggi il prompt dedicato nella sezione corrispondente di
`PIANO_IMPLEMENTAZIONE_RUST_PYTHON.md` e segui le istruzioni alla lettera.

⚠️ REGOLE DI LAVORO:
- Progetto greenfield: costruisci tutto da zero, nessun codice legacy da preservare.
- Non scrivere .unwrap() in Rust di produzione; usa `?` e `anyhow::Context`.
- Non usare `print()` in Python; usa `structlog`.
- Non usare `echo` di debug in PHP; usa il logger PSR-3.
- Ogni file deve avere header licensing e docstring iniziale.
- Ogni funzione pubblica ha doc + tipo di ritorno esplicito.
- Commit messages in inglese, Conventional Commits (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`).
- Commenti nel codice in italiano dove si spiega logica di dominio legale/contrattuale, inglese per commenti tecnici.
- Prima di aggiungere una dipendenza, verifica la licenza: solo MIT/Apache-2.0/BSD/MPL-2.0 senza chiedere; GPL/AGPL/commerciali richiedono AskUserQuestion.

Inizia dalla **FASE -1** (bootstrap + ricerca OSS + decision matrix). **Ferma l'esecuzione** al termine di Fase -1 e chiedi all'utente conferma esplicita via AskUserQuestion sul percorso architetturale scelto prima di passare a Fase 0.
````

---

## 12. KPI, Benchmark e Risk Assessment

### 12.1 KPI target (definition of done)

| Metrica | Target | Misurazione |
|---|---|---|
| **Ingestion throughput** | ≥ 5 doc/min (PDF 20 pp) | `benchmarks/ingest_bench.py` |
| **Query latency p50** | ≤ 1.5 s | `benchmarks/query_bench.py` |
| **Query latency p95** | ≤ 3.0 s | `benchmarks/query_bench.py` |
| **Multi-contract 5 docs** | ≤ 15 s end-to-end | `benchmarks/query_bench.py --compare` |
| **Recall@5** su gold-set | ≥ 92% | `benchmarks/query_bench.py` |
| **Hallucination rate** | < 1% su trick questions | `benchmarks/hallucination_eval.py` |
| **Citation validity** (quote verbatim in chunk) | ≥ 99% | Self-RAG evaluator counter |
| **Uptime del motore Rust** | 99.9% | Prometheus `up{service="rust-engine"}` |
| **Memory footprint totale** | ≤ 12 GB (senza modello 70B) | `docker stats` |
| **Concorrenza regge** | ≥ 50 query simultanee senza errori | `benchmarks/concurrent_bench.py` |

### 12.2 Rischi e mitigazioni

| Rischio | Probabilità | Impatto | Mitigazione |
|---|---|---|---|
| Ollama OOM con modello 70B | Alta | Alto | Documenta requisiti hardware; fallback automatico a qwen2.5:32b → :14b se OOM |
| BGE reranker troppo lento su CPU | Media | Medio | Disabilitabile via env; fallback a reranking solo con RRF score |
| Sparse vectors BM25 italiano mediocri | Media | Medio | Test con stemmer `tantivy` Italian + fallback a regex keyword boost |
| Contextual retrieval costoso in tempo | Alta | Basso | Cache aggressiva (DashMap) + opzione `ENABLE_CONTEXTUAL=false` |
| LLM genera JSON malformato | Media | Medio | 2 retry automatici + parsing tollerante via `serde_json::from_str` + ricostruzione key-by-key |
| Self-RAG in loop infinito | Bassa | Alto | Hard limit 3 retry, dopo retorna risposta originale con flag `verified=false` |
| Qdrant corruzione volume | Bassa | Alto | Backup settimanale via `docker exec qdrant ...`; documentato in RUNBOOK |
| Python worker crash | Media | Medio | Uvicorn workers=2, restart policy `unless-stopped`, circuit breaker lato Rust |
| Confronto multi-contratto allucina in una cella | Media | Alto | Validazione stringente per-cella, rigenerazione isolata, cella marcata `verified=false` se fallisce |
| Sprawl dipendenze Python (torch 5GB) | Media | Basso | Dockerfile multi-stage, CPU-only build default, GPU opt-in |

### 12.3 Roadmap post-v2.0 (future)

| Item | Priorità | Note |
|---|---|---|
| Passaggio Ollama → vLLM | Alta | 10× throughput, tensor parallelism |
| Modelli custom fine-tunati su corpus legale italiano | Media | Richiede dataset etichettato |
| UI differenze con diff visuale tra versioni dello stesso contratto | Media | GUI-heavy, ma molto richiesta dai legali |
| Esportazione report in .docx tramite skill docx | Bassa | Semplice, può riusare skill esistente |
| Audit trail completo (chi ha chiesto cosa su quale contratto) | Alta | Requisito GDPR/compliance |
| Firma digitale dei report (timestamping) | Media | Per validità legale dei report |
| Integrazione con sistemi di document management (Alfresco, SharePoint) | Bassa | Post-MVP |

---

## 13. 🌐 Architettura Multi-Provider LLM (Switching Runtime)

> **Obiettivo**: consentire di switchare a runtime tra qualsiasi provider LLM (Anthropic Claude, Google Gemini, OpenAI, DeepSeek, Qwen/Alibaba, Moonshot, Zhipu, Mistral, Groq, OpenRouter, Together, Fireworks, Ollama locale, vLLM locale) per bilanciare **precisione / costo / latenza / privacy** in base al task. Ogni provider è abilitabile/disabilitabile via `.env` e selezionabile dall'utente nel frontend.

### 13.1 Perché multi-provider

| Esigenza | Soluzione |
|---|---|
| Privacy massima (contratti riservati) | Ollama locale (qwen2.5:7b) — zero data egress |
| Precisione assoluta su clausole critiche | Claude Opus 4.7, GPT-5, o3 |
| Context window enorme (intero contratto >500 pagine) | Gemini 2.5 Pro (2M tokens), Claude Sonnet 4.6 (200K) |
| Costo minimo con qualità alta | DeepSeek V3/R1, Qwen Max, Mistral Large |
| Latenza bassa (UI reattiva) | Groq (500+ tok/s), Claude Haiku 4.5, Gemini 2.5 Flash |
| Fallback se provider down | Routing automatico al successivo disponibile |
| Benchmark A/B qualità | Stessa domanda a 3 provider in parallelo, confronto risposte |

### 13.2 Tabella provider supportati

| Provider | Modelli target | Endpoint | Auth | Streaming | Uso tipico | Costo (ord.) |
|---|---|---|---|---|---|---|
| **Ollama (locale)** | qwen2.5:7b, llama3.1:8b, deepseek-r1:8b, mistral:7b, gemma2:9b | `http://ollama:11434/api/chat` | nessuna | ✅ | Privacy, dev, task massivi | Gratis |
| **vLLM (locale futuro)** | Qualsiasi HF model | `http://vllm:8000/v1/chat/completions` | API key opt. | ✅ | Produzione high-throughput | Gratis |
| **Anthropic Claude** | `claude-opus-4-6`, `claude-sonnet-4-6`, `claude-haiku-4-5-20251001` | `https://api.anthropic.com/v1/messages` | `x-api-key` | ✅ | Reasoning legale top, audit forense | $$$ Opus / $$ Sonnet / $ Haiku |
| **Google Gemini** | `gemini-2.5-pro`, `gemini-2.5-flash`, `gemini-2.5-flash-lite` | `https://generativelanguage.googleapis.com/v1beta/models/{model}:streamGenerateContent` | API key | ✅ | Context window enorme (2M), multimodale | $$ Pro / $ Flash |
| **OpenAI** | `gpt-5`, `gpt-5-mini`, `gpt-4.1`, `o3`, `o4-mini` | `https://api.openai.com/v1/chat/completions` | Bearer | ✅ | Reasoning premium, o3 per problemi complessi | $$$$ o3 / $$ gpt-5 |
| **DeepSeek** | `deepseek-chat` (V3), `deepseek-reasoner` (R1) | `https://api.deepseek.com/chat/completions` | Bearer | ✅ | Ragionamento potente, economico | $ molto economico |
| **Alibaba Qwen (DashScope)** | `qwen-max`, `qwen-plus`, `qwen-turbo`, `qwen-long` (10M ctx) | `https://dashscope-intl.aliyuncs.com/compatible-mode/v1/chat/completions` | Bearer | ✅ | Context ultra-long, multilingue CN/IT | $ economico |
| **Moonshot (Kimi)** | `kimi-k2`, `moonshot-v1-128k`, `moonshot-v1-32k` | `https://api.moonshot.cn/v1/chat/completions` | Bearer | ✅ | Long context documenti | $ |
| **Zhipu GLM** | `glm-4.5`, `glm-4-plus`, `glm-4-long` (1M ctx) | `https://open.bigmodel.cn/api/paas/v4/chat/completions` | Bearer | ✅ | Multilingue, long ctx | $ |
| **Mistral AI** | `mistral-large-latest`, `mistral-medium`, `codestral-latest` | `https://api.mistral.ai/v1/chat/completions` | Bearer | ✅ | Open weights + API hosted | $$ |
| **Groq** | `llama-3.3-70b-versatile`, `mixtral-8x7b-32768`, `deepseek-r1-distill-llama-70b` | `https://api.groq.com/openai/v1/chat/completions` | Bearer | ✅ | **Inferenza più veloce al mondo** (500+ tok/s) | $ |
| **OpenRouter** | Unified: `anthropic/claude-opus-4.6`, `google/gemini-2.5-pro`, `deepseek/deepseek-chat`, etc. | `https://openrouter.ai/api/v1/chat/completions` | Bearer | ✅ | Un'unica API key per 300+ modelli | varia |
| **Together.ai** | `meta-llama/Llama-3.3-70B-Instruct-Turbo`, `deepseek-ai/DeepSeek-V3`, Qwen, Mixtral | `https://api.together.xyz/v1/chat/completions` | Bearer | ✅ | Open models hostati, economici | $ |
| **Fireworks.ai** | `accounts/fireworks/models/llama-v3p3-70b-instruct`, DeepSeek, Mixtral | `https://api.fireworks.ai/inference/v1/chat/completions` | Bearer | ✅ | Low latency, open models | $ |

> **Nota sui nomi modello**: i nomi esatti cambiano rapidamente. L'`.env` permette override di ogni `*_MODEL_CHAT` senza toccare il codice. Un endpoint `/api/admin/models/refresh` re-interroga ogni provider per listare i modelli disponibili (ove l'API lo consenta).

### 13.3 Trait `LlmProvider` in Rust (abstrazione core)

Questa è l'astrazione chiave: ogni provider implementa lo stesso trait, il resto del sistema è agnostico.

```rust
// engine-rust/src/llm/mod.rs
use async_trait::async_trait;
use futures::stream::BoxStream;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,        // "system" | "user" | "assistant"
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub model: String,                     // nome logico, es. "claude-sonnet-4-6"
    pub temperature: f32,
    pub max_tokens: u32,
    pub json_schema: Option<serde_json::Value>, // structured output
    pub tools: Option<Vec<serde_json::Value>>,  // tool calling
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub provider: String,
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cost_usd: f64,                     // calcolato lato Rust da tariffario
    pub latency_ms: u64,
    pub finish_reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StreamEvent {
    Token(String),
    Usage { input: u32, output: u32, cost_usd: f64 },
    Done,
    Error(String),
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Identificativo univoco del provider ("anthropic", "google", "ollama", ecc.)
    fn name(&self) -> &'static str;

    /// Lista modelli abilitati (letti da .env o API discovery)
    fn available_models(&self) -> Vec<ModelInfo>;

    /// Health check con timeout 1s
    async fn is_available(&self) -> bool;

    /// Chiamata blocking
    async fn chat(&self, req: ChatRequest) -> anyhow::Result<ChatResponse>;

    /// Streaming token-by-token (per SSE)
    async fn chat_stream(&self, req: ChatRequest) -> anyhow::Result<BoxStream<'static, StreamEvent>>;

    /// Embedding (opzionale — alcuni provider non lo supportano)
    async fn embed(&self, texts: Vec<String>, model: &str) -> anyhow::Result<Vec<Vec<f32>>> {
        anyhow::bail!("Provider {} non supporta embedding", self.name())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,                    // "claude-sonnet-4-6"
    pub display_name: String,          // "Claude Sonnet 4.6"
    pub context_window: u32,
    pub supports_streaming: bool,
    pub supports_tools: bool,
    pub supports_json_mode: bool,
    pub input_cost_per_1m: f64,        // USD per 1M input tokens
    pub output_cost_per_1m: f64,       // USD per 1M output tokens
    pub capability_tier: CapabilityTier, // Premium | Standard | Fast | Local
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CapabilityTier {
    Premium,   // Claude Opus, GPT-5, o3, Gemini 2.5 Pro
    Standard,  // Claude Sonnet, GPT-5-mini, DeepSeek V3, Qwen Max
    Fast,      // Claude Haiku, Gemini Flash, Groq, Cerebras
    Local,     // Ollama, vLLM on-prem
}
```

### 13.4 Implementazioni concrete (struttura)

```
engine-rust/src/llm/
├── mod.rs                 # Trait definition (sopra)
├── registry.rs            # LlmRegistry: HashMap<String, Arc<dyn LlmProvider>>, routing policy
├── router.rs              # TaskRouter: sceglie provider in base al task
├── cost_tracker.rs        # Calcolo costo per chiamata + budget enforcement
├── providers/
│   ├── ollama.rs          # Impl per Ollama (default)
│   ├── anthropic.rs       # Impl per Claude (messages API)
│   ├── google.rs          # Impl per Gemini (streamGenerateContent)
│   ├── openai.rs          # Impl per OpenAI (chat/completions)
│   ├── deepseek.rs        # Impl per DeepSeek (OpenAI-compatible)
│   ├── qwen.rs            # Impl per Alibaba DashScope (OpenAI-compatible)
│   ├── moonshot.rs        # Impl per Moonshot Kimi
│   ├── zhipu.rs           # Impl per Zhipu GLM
│   ├── mistral.rs         # Impl per Mistral AI
│   ├── groq.rs            # Impl per Groq (OpenAI-compatible ultra-fast)
│   ├── openrouter.rs      # Impl per OpenRouter (unified gateway)
│   ├── together.rs        # Impl per Together.ai
│   └── fireworks.rs       # Impl per Fireworks.ai
```

La maggior parte dei provider espone API **compatibili OpenAI** (chat/completions), quindi condividono un modulo base `openai_compat.rs` e ogni provider specifica solo endpoint + auth header.

### 13.5 Task Router: modello giusto per il task giusto

```rust
// engine-rust/src/llm/router.rs
pub enum TaskKind {
    IntentClassification,       // veloce e minimale → locale/Haiku/Flash
    QueryExpansion,             // HyDE → Haiku/Flash o DeepSeek
    ContextualRetrieval,        // massivo (per chunk) → locale obbligato
    RagAnswer,                  // standard → Sonnet/qwen2.5:7b
    MultiContractComparison,    // critico → Opus/GPT-5/Gemini 2.5 Pro
    ForensicAudit,              // top precision → Opus/o3
    SelfRagEvaluator,           // verifica seconda opinione → Sonnet/DeepSeek-R1
    Summarization,              // economico → Haiku/Flash/Mistral medium
}

pub struct RoutingPolicy {
    pub preferred_provider: String,
    pub preferred_model: String,
    pub fallback_chain: Vec<(String, String)>, // (provider, model)
    pub max_cost_usd_per_call: f64,
    pub max_latency_ms: u64,
    pub require_local_only: bool,              // per task privacy-critical
}

impl TaskRouter {
    pub fn select(&self, task: TaskKind, user_override: Option<&str>) -> RoutingPolicy {
        // 1. Se l'utente ha forzato un modello nella UI → usa quello
        // 2. Altrimenti usa policy default per task
        // 3. Se il provider preferito è down → fallback chain
        // 4. Se cost budget superato → downgrade a tier inferiore
    }
}
```

**Policy default di sistema** (override via admin UI):

| Task | Preferred | Fallback 1 | Fallback 2 |
|---|---|---|---|
| IntentClassification | `ollama/qwen2.5:3b` | `groq/llama-3.3-70b` | `anthropic/claude-haiku-4-5` |
| QueryExpansion | `ollama/qwen2.5:7b` | `deepseek/deepseek-chat` | `google/gemini-2.5-flash` |
| ContextualRetrieval | `ollama/qwen2.5:3b` (obbligatorio locale, costo zero) | — | — |
| RagAnswer | `ollama/qwen2.5:7b` | `anthropic/claude-sonnet-4-6` | `deepseek/deepseek-chat` |
| MultiContractComparison | `anthropic/claude-sonnet-4-6` | `google/gemini-2.5-pro` | `openai/gpt-5` |
| ForensicAudit | `anthropic/claude-opus-4-6` | `openai/o3` | `google/gemini-2.5-pro` |
| SelfRagEvaluator | `ollama/qwen2.5:7b` | `deepseek/deepseek-reasoner` | `anthropic/claude-haiku-4-5` |
| Summarization | `anthropic/claude-haiku-4-5` | `google/gemini-2.5-flash` | `ollama/qwen2.5:7b` |

### 13.6 Configurazione `.env` completa

```ini
# === LLM MULTI-PROVIDER SWITCHING ===
# Provider di default (usato se nessun override)
LLM_DEFAULT_PROVIDER=ollama
LLM_DEFAULT_MODEL=qwen2.5:7b-instruct-q4_K_M

# Embedding di default
EMBEDDING_DEFAULT_PROVIDER=ollama
EMBEDDING_DEFAULT_MODEL=nomic-embed-text

# Budget limits (applicati in cost_tracker.rs)
LLM_BUDGET_DAILY_USD=50
LLM_BUDGET_MONTHLY_USD=1000
LLM_MAX_COST_PER_CALL_USD=2.00

# --- Ollama (locale, gratis) ---
OLLAMA_ENABLED=true
OLLAMA_BASE_URL=http://ollama:11434
OLLAMA_MODEL_CHAT=qwen2.5:7b-instruct-q4_K_M
OLLAMA_MODEL_CHAT_LIGHT=qwen2.5:3b-instruct-q4_K_M
OLLAMA_MODEL_CHAT_REASONING=deepseek-r1:8b
OLLAMA_MODEL_EMBED=nomic-embed-text

# --- Anthropic Claude ---
ANTHROPIC_ENABLED=false
ANTHROPIC_API_KEY=sk-ant-...
ANTHROPIC_BASE_URL=https://api.anthropic.com
ANTHROPIC_MODEL_PREMIUM=claude-opus-4-6
ANTHROPIC_MODEL_STANDARD=claude-sonnet-4-6
ANTHROPIC_MODEL_FAST=claude-haiku-4-5-20251001
ANTHROPIC_VERSION=2023-06-01

# --- Google Gemini ---
GOOGLE_ENABLED=false
GOOGLE_API_KEY=AIza...
GOOGLE_MODEL_PREMIUM=gemini-2.5-pro
GOOGLE_MODEL_FAST=gemini-2.5-flash
GOOGLE_MODEL_LITE=gemini-2.5-flash-lite

# --- OpenAI ---
OPENAI_ENABLED=false
OPENAI_API_KEY=sk-...
OPENAI_BASE_URL=https://api.openai.com/v1
OPENAI_MODEL_PREMIUM=gpt-5
OPENAI_MODEL_REASONING=o3
OPENAI_MODEL_STANDARD=gpt-4.1
OPENAI_MODEL_FAST=gpt-5-mini
OPENAI_MODEL_EMBED=text-embedding-3-large

# --- DeepSeek ---
DEEPSEEK_ENABLED=false
DEEPSEEK_API_KEY=sk-...
DEEPSEEK_BASE_URL=https://api.deepseek.com
DEEPSEEK_MODEL_CHAT=deepseek-chat
DEEPSEEK_MODEL_REASONER=deepseek-reasoner

# --- Alibaba Qwen (DashScope) ---
QWEN_ENABLED=false
QWEN_API_KEY=sk-...
QWEN_BASE_URL=https://dashscope-intl.aliyuncs.com/compatible-mode/v1
QWEN_MODEL_MAX=qwen-max
QWEN_MODEL_PLUS=qwen-plus
QWEN_MODEL_LONG=qwen-long

# --- Moonshot Kimi ---
MOONSHOT_ENABLED=false
MOONSHOT_API_KEY=sk-...
MOONSHOT_BASE_URL=https://api.moonshot.cn/v1
MOONSHOT_MODEL_CHAT=moonshot-v1-128k
MOONSHOT_MODEL_K2=kimi-k2

# --- Zhipu GLM ---
ZHIPU_ENABLED=false
ZHIPU_API_KEY=...
ZHIPU_BASE_URL=https://open.bigmodel.cn/api/paas/v4
ZHIPU_MODEL_CHAT=glm-4.5
ZHIPU_MODEL_LONG=glm-4-long

# --- Mistral ---
MISTRAL_ENABLED=false
MISTRAL_API_KEY=...
MISTRAL_BASE_URL=https://api.mistral.ai/v1
MISTRAL_MODEL_LARGE=mistral-large-latest
MISTRAL_MODEL_CODE=codestral-latest

# --- Groq (ultra-fast) ---
GROQ_ENABLED=false
GROQ_API_KEY=gsk_...
GROQ_BASE_URL=https://api.groq.com/openai/v1
GROQ_MODEL_CHAT=llama-3.3-70b-versatile
GROQ_MODEL_REASONING=deepseek-r1-distill-llama-70b

# --- OpenRouter (gateway unificato a 300+ modelli) ---
OPENROUTER_ENABLED=false
OPENROUTER_API_KEY=sk-or-...
OPENROUTER_BASE_URL=https://openrouter.ai/api/v1
OPENROUTER_DEFAULT_MODEL=anthropic/claude-sonnet-4.6

# --- Together.ai ---
TOGETHER_ENABLED=false
TOGETHER_API_KEY=...
TOGETHER_BASE_URL=https://api.together.xyz/v1
TOGETHER_MODEL_CHAT=meta-llama/Llama-3.3-70B-Instruct-Turbo

# --- Fireworks.ai ---
FIREWORKS_ENABLED=false
FIREWORKS_API_KEY=fw_...
FIREWORKS_BASE_URL=https://api.fireworks.ai/inference/v1
FIREWORKS_MODEL_CHAT=accounts/fireworks/models/llama-v3p3-70b-instruct
```

### 13.7 Cost Tracker e budget enforcement

```rust
// engine-rust/src/llm/cost_tracker.rs
pub struct CostTracker {
    // Redis-backed per persistenza cross-process
    redis: redis::Client,
    daily_limit_usd: f64,
    monthly_limit_usd: f64,
}

impl CostTracker {
    pub async fn record(&self, provider: &str, model: &str, input: u32, output: u32) -> f64 {
        let cost = compute_cost(provider, model, input, output);
        // INCRBYFLOAT su chiavi daily:{YYYY-MM-DD} e monthly:{YYYY-MM}
        cost
    }

    pub async fn check_budget(&self, estimated_cost: f64) -> Result<(), BudgetExceeded> {
        let daily = self.get_daily_usage().await?;
        if daily + estimated_cost > self.daily_limit_usd {
            return Err(BudgetExceeded::Daily);
        }
        // idem monthly
        Ok(())
    }
}
```

Dashboard admin mostra:
- Spesa giornaliera/mensile per provider
- Top 10 documenti/chat più costosi
- Alerting email/Slack se superi 80% del budget

### 13.8 Frontend: Model Selector UI

Componente React `<ModelSelector />` nel chat widget:

```jsx
<ModelSelector
  value={selectedModel}
  onChange={setSelectedModel}
  options={availableModels}
  groupBy="tier"
  showCost={true}
  showLatency={true}
/>
```

UX:
- Dropdown raggruppato per tier: **Premium** / **Standard** / **Fast** / **Locale (privacy)**
- Ogni entry mostra: nome, provider, costo stimato per 1K token, latenza media (ultimi 100 call), indicatore disponibilità (🟢/🔴)
- Tag colorati: 🔒 Locale (privacy), ⚡ Veloce, 🏆 Top qualità, 💰 Economico
- Comparatore: "confronta questa domanda su 3 modelli" (esegue in parallelo, mostra le risposte affiancate)
- Salva preferenza per-utente in `ap_users.preferred_model`

Nell'admin panel: toggle on/off per provider, API key management (cifrata in DB), override policy routing per task.

### 13.9 Nuovi endpoint API

Aggiunti al Rust engine e proxati da PHP:

| Endpoint | Metodo | Descrizione |
|---|---|---|
| `/api/llm/providers` | GET | Lista provider abilitati con health status |
| `/api/llm/models` | GET | Lista modelli disponibili per provider |
| `/api/llm/usage` | GET | Statistiche costo giornaliero/mensile |
| `/api/admin/llm/providers/{name}` | PATCH | Abilita/disabilita provider |
| `/api/admin/llm/providers/{name}/test` | POST | Test call a provider per verificare API key |
| `/api/admin/llm/routing` | GET/PUT | Get/set policy routing per task |
| `/api/admin/llm/budget` | GET/PUT | Get/set limiti di spesa |
| `/api/chat/compare` | POST | Esegue stessa query su N modelli, ritorna array |

### 13.10 Database: nuove tabelle

```sql
-- Tracking dettagliato costi
CREATE TABLE ap_llm_calls (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    user_id INT NULL,
    session_id VARCHAR(64) NULL,
    provider VARCHAR(32) NOT NULL,
    model VARCHAR(128) NOT NULL,
    task_kind VARCHAR(32) NOT NULL,
    input_tokens INT NOT NULL,
    output_tokens INT NOT NULL,
    cost_usd DECIMAL(10,6) NOT NULL,
    latency_ms INT NOT NULL,
    finish_reason VARCHAR(32),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_created (created_at),
    INDEX idx_user_date (user_id, created_at),
    INDEX idx_provider_date (provider, created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Configurazione provider dinamica (override .env)
CREATE TABLE ap_llm_providers (
    name VARCHAR(32) PRIMARY KEY,
    enabled BOOLEAN NOT NULL DEFAULT false,
    api_key_encrypted TEXT,           -- AES-256 con chiave in APP_KEY
    base_url VARCHAR(255),
    default_model VARCHAR(128),
    rate_limit_rpm INT DEFAULT 60,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Policy routing per task (override programmatico)
CREATE TABLE ap_llm_routing_policy (
    task_kind VARCHAR(32) PRIMARY KEY,
    preferred_provider VARCHAR(32) NOT NULL,
    preferred_model VARCHAR(128) NOT NULL,
    fallback_json JSON NOT NULL,      -- [{provider,model},...]
    max_cost_usd DECIMAL(6,4),
    max_latency_ms INT,
    require_local BOOLEAN DEFAULT false,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
```

### 13.11 🎯 PROMPT DEDICATO per Claude Code — Implementazione Multi-Provider

```
Fase 1.7 — Multi-Provider LLM Layer (Rust)

Implementa nel progetto engine-rust/ un sistema multi-provider LLM completo, partendo dalla struttura già esistente. L'obiettivo è poter switchare runtime tra Ollama locale, Anthropic Claude, Google Gemini, OpenAI, DeepSeek, Qwen, Moonshot, Zhipu, Mistral, Groq, OpenRouter, Together, Fireworks.

REQUISITI:

1. Crea il modulo `src/llm/` con la struttura:
   - mod.rs (trait LlmProvider, ChatRequest/Response, StreamEvent, ModelInfo, CapabilityTier)
   - registry.rs (LlmRegistry con HashMap<String, Arc<dyn LlmProvider>>, init da .env)
   - router.rs (TaskRouter con policy default per 8 TaskKind diversi)
   - cost_tracker.rs (persistenza su Redis, budget daily/monthly, tariffario hardcoded con override DB)
   - providers/ollama.rs, anthropic.rs, google.rs, openai.rs, deepseek.rs, qwen.rs, moonshot.rs, zhipu.rs, mistral.rs, groq.rs, openrouter.rs, together.rs, fireworks.rs

2. Il trait `LlmProvider` deve esporre:
   - fn name() -> &'static str
   - fn available_models() -> Vec<ModelInfo>
   - async fn is_available() -> bool  (timeout 1s, health check endpoint)
   - async fn chat(req: ChatRequest) -> Result<ChatResponse>
   - async fn chat_stream(req: ChatRequest) -> Result<BoxStream<StreamEvent>>
   - async fn embed(texts: Vec<String>, model: &str) -> Result<Vec<Vec<f32>>>  (default impl: bail!)

3. Ogni provider legge config da .env (vedi §13.6 del PIANO_IMPLEMENTAZIONE_RUST_PYTHON.md) ed è abilitato solo se `{PROVIDER}_ENABLED=true` e la API key non è vuota.

4. Fattorizza codice comune in un modulo `providers/openai_compat.rs` perché DeepSeek, Qwen, Moonshot, Zhipu, Groq, OpenRouter, Together, Fireworks usano tutti endpoint OpenAI-compatible.

5. Implementazioni specifiche (NON OpenAI-compatible):
   - anthropic.rs: usa endpoint `/v1/messages` con header `x-api-key` e `anthropic-version`, parsing SSE con eventi `content_block_delta`
   - google.rs: usa `streamGenerateContent`, payload `{contents:[{parts:[{text}]}]}`, parsing chunked JSON
   - ollama.rs: usa `/api/chat` con streaming NDJSON

6. Aggiungi endpoint axum:
   - GET /llm/providers → JSON {name, enabled, available, models:[...]}
   - GET /llm/models?provider=X → lista ModelInfo
   - GET /llm/usage → {today_usd, month_usd, by_provider: {...}}
   - POST /llm/chat → body {messages, task_kind?, model_override?, provider_override?}, risp. ChatResponse
   - POST /llm/chat/stream → SSE stream di StreamEvent
   - POST /llm/compare → body {messages, targets:[{provider,model},...]}, risp. array di ChatResponse (in parallelo con tokio::join_all)

7. Router logic:
   - Se user passa `model_override`, usalo direttamente (con check che sia enabled)
   - Altrimenti consulta `ap_llm_routing_policy` via sqlx (fallback a default hardcoded)
   - Se cost_tracker.check_budget() fallisce, downgrade al tier inferiore
   - Se is_available() ritorna false, passa al fallback successivo (max 3 tentativi)
   - Logga ogni call in ap_llm_calls

8. Cost tracking:
   - Tabella prezzi hardcoded in `cost_tracker.rs` (input/output per 1M token), allineata a §13.2
   - Dopo ogni chiamata, INSERT in ap_llm_calls e INCRBYFLOAT su Redis `budget:daily:{YYYY-MM-DD}`
   - Metric Prometheus: `llm_cost_usd_total{provider,model,task_kind}`

9. Admin CRUD (PHP layer lo espone, ma Rust mantiene l'endpoint):
   - PATCH /llm/providers/{name} body {enabled, api_key?, default_model?}
     → aggiorna ap_llm_providers, re-inizializza il provider nella LlmRegistry via `registry.reload(name)`
   - POST /llm/providers/{name}/test → fa una call minima e ritorna {success, latency_ms, error?}

10. Test:
    - Unit test per ogni provider con mock server (wiremock crate)
    - Integration test che verifichi fallback chain (primo provider fallisce → usa secondo)
    - Test che verifichi budget enforcement (budget pieno → error 429)
    - Test compare endpoint con 3 provider paralleli

11. Aggiorna `main.rs` per inizializzare `LlmRegistry` all'avvio leggendo da .env + ap_llm_providers (DB override prevale).

12. Aggiungi migration SQL `db/migrations/004_llm_multiprovider.sql` con le 3 tabelle di §13.10.

DELIVERABLE:
- ~15 file Rust (mod.rs, registry, router, cost_tracker, 13 provider)
- 1 SQL migration
- 1 test suite (tests/llm_providers.rs)
- README aggiornato in engine-rust/README.md con sezione "Multi-Provider"
- Aggiornamento .env.example con tutti i blocchi di §13.6

VINCOLI:
- Nessun provider abilitato di default tranne Ollama (privacy first)
- Tutti i segreti da .env, mai hardcoded
- Se `{PROVIDER}_API_KEY` è vuoto → provider marcato come disabilitato nella registry
- Zero panic in produzione: ogni errore di rete/parse → anyhow::Error tracciato
- Streaming real-time obbligatorio per tutti i provider che lo supportano (13 su 14)
```

### 13.12 Integrazione con le altre fasi

Questa sezione **non sostituisce** le Fasi 1–5, ma le **estende**:

- **Fase 1.1** (Rust core): il prompt originale chiedeva un `OllamaClient`. Va **generalizzato** a `LlmRegistry` con il trait di §13.3 come default. L'`OllamaClient` diventa `providers::ollama::OllamaProvider`.
- **Fase 1.4** (RAG engine): tutte le chiamate `ollama.generate(...)` diventano `registry.resolve(task_kind).chat(...)` → automaticamente routate.
- **Fase 2** (Python): nessun impatto diretto, ma il reranker può essere invocato passando il modello tramite lo stesso trait se un giorno si aggiunge un provider reranker cloud (es. Cohere Rerank).
- **Fase 3** (PHP gateway): aggiungere 7 nuovi endpoint proxy (vedi §13.9) + controller admin per gestione provider e budget.
- **Fase 4** (Frontend): aggiungere `<ModelSelector />` nel chat, `<ProviderAdmin />` nell'admin, dashboard costi, vista compare multi-modello.

### 13.13 Checklist di accettazione §13

- [ ] Tutti i 14 provider implementano `LlmProvider` trait
- [ ] `.env.example` contiene blocchi configurabili per ciascuno
- [ ] `LlmRegistry` si auto-configura leggendo `{PROVIDER}_ENABLED` e API keys
- [ ] Streaming SSE funziona su Claude, Gemini, Ollama, OpenAI, DeepSeek (testato manualmente)
- [ ] Endpoint `/llm/compare` esegue 3 modelli in parallelo in <3s
- [ ] Admin UI permette abilitare/disabilitare provider senza restart container
- [ ] Budget limit superato → errore 429 con messaggio "Budget giornaliero esaurito"
- [ ] Tabella `ap_llm_calls` popolata dopo ogni chiamata con costo corretto
- [ ] Fallback chain funziona: disabilita provider preferito → usa automaticamente il next
- [ ] Model Selector frontend mostra costo stimato, latenza media, tag privacy/velocità/qualità
- [ ] Policy routing editabile dall'admin (8 task kinds)
- [ ] Nessuna regressione su flussi RAG esistenti (test suite verde)

---

## 📌 Note conclusive per l'utente

Questo piano è **pronto per l'esecuzione greenfield**. L'approccio consigliato:

1. **Crea una directory vuota** sul tuo PC (es. `D:\progetti\archivio-parlante\`) e salva questo file come `PIANO_IMPLEMENTAZIONE_RUST_PYTHON.md` al suo interno.
2. Apri Claude Code nella directory vuota.
3. **Esegui prima Fase -1** (Bootstrap + Ricerca OSS + Decision Matrix) — è la fase più importante perché Claude Code valuterà se è meglio clonare un framework OSS esistente o costruire from-scratch. Al termine ti chiederà conferma esplicita via AskUserQuestion.
4. Solo dopo la tua conferma, procedi **Fase 0 → Fase 5 una alla volta**, copia/incolla il prompt della fase in Claude Code, attendi il completamento e la verifica, poi passa alla successiva.
5. Dopo Fase 0 e prima di Fase 1, scarica i modelli Ollama: `make ollama-pull`.
6. Il "Prompt Master" della sezione 11 è per esecuzione full-autonomous (richiede supervisione costante comunque), ma rispetta i gate AskUserQuestion di Fase -1 e prima di abilitare provider a pagamento.

**Requisiti hardware — il tuo setup (MSI Raider GE78HX 13VG) è più che sufficiente:**
- ✅ CPU: **i9-13950HX** (24 core / 32 thread) — ottimo per Rust async concurrency
- ✅ RAM: **32 GB DDR5** — sufficiente per tutto lo stack Docker + offload CPU fino a mixtral:8x7b
- ⚠️ GPU: **RTX 4070 Laptop 8 GB VRAM** — adeguata per modelli 7B/8B/9B in Q4, NON per 32B/70B locali (usa API cloud per quelli)
- ✅ Disco: **966 GB liberi** su NVMe 2 TB — abbondante per modelli Ollama (50 GB) + Qdrant + Docker images (20 GB)
- ✅ OS: Windows 11 Pro con WSL2 richiesto per Docker Desktop (già supportato)

**Cosa gira in locale sul tuo PC (senza problemi)**:
- Qwen2.5 7B/8B, Llama3.1 8B, Mistral 7B, Gemma2 9B, DeepSeek-R1 7B/8B distilled — tutti in VRAM
- Stack Docker completo: PHP + Rust + Python + Qdrant + MySQL + Redis + Ollama (~10 GB RAM in totale)
- Embedding nomic-embed-text o bge-m3 concorrente con il chat model

**Cosa delegato al cloud multi-provider (§13)**:
- Reasoning top-tier: Claude Opus 4.7, Sonnet 4.6, GPT-5, o3, Gemini 2.5 Pro
- Modelli economici ma potenti: DeepSeek V3/R1, Qwen Max (via DashScope)
- Inferenza ultraveloce: Groq, Fireworks, Together

**Tempo stimato di implementazione** seguendo il piano step-by-step con Claude Code (progetto greenfield):
- Fase -1 (bootstrap + ricerca OSS + decision matrix): 1 giornata
- Fase 0 (Docker Compose): 1 giornata
- Fase 1 (Rust engine + multi-provider §13): 6–9 giornate (la più grande)
- Fase 2 (Python worker): 2–3 giornate
- Fase 3 (PHP gateway greenfield): 2–3 giornate
- Fase 4 (Frontend greenfield React+TS): 3–4 giornate
- Fase 5 (Testing + benchmark + hardening): 3–5 giornate
- **Totale: 18–26 giornate di sviluppo attivo**

Buon greenfield. 🚀