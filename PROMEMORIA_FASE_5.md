# 📌 PROMEMORIA FASE 5 — Ripresa Lavoro

**Data creazione**: 2026-05-06 (sera)  
**Ripresa prevista**: 2026-05-07 (pomeriggio)  
**Fase corrente**: Fase 5 (Testing, Benchmark, Hardening)

---

## ✅ Stato Avanzamento Fase 5

**Completate**: 12/21 task (57%)  
**Rimanenti**: 10 task  
**Tempo stimato rimanente**: ~4-6 ore

---

## 📋 TASK RIMANENTI (in ordine di esecuzione consigliato)

### 🔒 BLOCCO 1: Security Hardening (PRIORITÀ ALTA)

#### Task #9 — Security hardening Python Worker
**Stato**: Pending  
**Tempo stimato**: 45-60 min

**Cosa fare**:
1. Audit codice Python Worker per vulnerabilità OWASP:
   - ✅ Input validation (già presente con Pydantic strict)
   - ⚠️ Path traversal in file uploads
   - ⚠️ Command injection in subprocess (Tesseract OCR)
   - ⚠️ Dependency vulnerabilities (pip-audit)
2. Implementare mitigazioni:
   - Aggiungere path canonicalization esplicita
   - Timeout Tesseract (già presente, verificare)
   - Whitelist MIME types (già presente, documentare)
3. Aggiornare `docs/SECURITY_AUDIT_FASE_5.md` sezione Python Worker

**File da modificare**:
- `engine-python/app/routers/parse.py` (validazione input)
- `engine-python/app/services/pdf_parser.py` (path safety)
- `engine-python/app/services/ocr_service.py` (subprocess hardening)

**Comando verifica**:
```bash
cd engine-python
pip-audit
ruff check .
```

---

#### Task #10 — Security hardening PHP Gateway
**Stato**: Pending  
**Tempo stimato**: 45-60 min

**Cosa fare**:
1. Audit codice PHP Gateway:
   - ✅ SQL injection (verificare PDO prepared statements)
   - ⚠️ CSRF protection (implementare double-submit cookie)
   - ⚠️ XSS in error messages
   - ⚠️ Session fixation
2. Implementare mitigazioni:
   - Aggiungere CSRF middleware
   - Escape output HTML
   - Regenerate session ID on login
   - Implement rate limiting via Redis
3. Aggiornare `docs/SECURITY_AUDIT_FASE_5.md` sezione PHP

**File da modificare**:
- `php-gateway/src/Middleware/CsrfMiddleware.php` (DA CREARE)
- `php-gateway/src/Middleware/RateLimitMiddleware.php` (DA CREARE)
- `php-gateway/src/Controller/AuthController.php` (session regeneration)

**Comando verifica**:
```bash
cd php-gateway
composer run phpstan
composer audit
```

---

#### Task #20 — Security audit finale e review
**Stato**: Pending  
**Tempo stimato**: 30 min

**Cosa fare**:
1. Eseguire tutti i security audit tools:
   ```bash
   make audit-security
   ```
2. Review manuale `SECURITY_AUDIT_FASE_5.md`
3. Verificare che tutti i controlli siano ✅
4. Aggiornare sezione "Residual Risks"
5. Sign-off finale del documento

---

### 🚀 BLOCCO 2: DevOps & Observability

#### Task #11 — Update CI/CD pipeline
**Stato**: Pending  
**Tempo stimato**: 45 min

**Cosa fare**:
1. Aggiornare `.github/workflows/ci.yml`:
   - Aggiungere step `cargo audit` (Rust)
   - Aggiungere step `pip-audit` (Python)
   - Aggiungere step `composer audit` (PHP)
   - Aggiungere step `npm audit` (Frontend)
   - Eseguire `make test-e2e` (richiede stack UP)
   - Fail build se CVE HIGH/CRITICAL
2. Aggiungere badge README.md per CI status
3. Configurare branch protection rules su GitHub

**File da modificare**:
- `.github/workflows/ci.yml` (già esistente, aggiornare)
- `README.md` (aggiungere badge)

**Test**:
- Push su feature branch e verificare che CI verde

---

#### Task #12 — Setup Observability stack
**Stato**: Pending  
**Tempo stimato**: 60 min

**Cosa fare**:
1. Creare `docker-compose.observability.yml`:
   - Prometheus (porta 9090)
   - Grafana (porta 3001) ⚠️ NON 3000 (conflitto con archivio-parlante-starter)
   - Node Exporter (metriche host)
   - Cadvisor (metriche container)
2. Configurare Prometheus scrape targets:
   - Rust Engine `/metrics` (già implementato)
   - PHP Gateway `/metrics` (DA IMPLEMENTARE)
   - Python Worker `/metrics` (DA IMPLEMENTARE)
3. Configurare Grafana datasource (Prometheus)
4. Aggiungere comandi Makefile:
   ```makefile
   observability-up:
       docker-compose -f docker-compose.observability.yml up -d
   
   observability-down:
       docker-compose -f docker-compose.observability.yml down
   ```

**File da creare**:
- `docker-compose.observability.yml`
- `observability/prometheus/prometheus.yml`
- `observability/grafana/datasources.yml`

---

#### Task #13 — Create Grafana dashboards
**Stato**: Pending  
**Tempo stimato**: 45 min

**Cosa fare**:
1. Creare dashboard "Archivio Parlante Overview":
   - Panel: Request rate (req/s)
   - Panel: Latency p50/p95/p99
   - Panel: Error rate
   - Panel: Active users
   - Panel: Qdrant vector count
   - Panel: Ollama GPU usage (se disponibile)
2. Creare dashboard "RAG Pipeline Metrics":
   - Panel: Query latency breakdown (embedding, search, rerank, LLM)
   - Panel: Hallucination rate
   - Panel: Citations per query
   - Panel: Top queries
3. Esportare JSON dashboards in `observability/grafana/dashboards/`

**File da creare**:
- `observability/grafana/dashboards/overview.json`
- `observability/grafana/dashboards/rag_pipeline.json`

**Nota**: Se Rust Engine non espone ancora le metriche custom, aggiungerle con `prometheus` crate (già in Cargo.toml)

---

### 🧪 BLOCCO 3: Testing & Load

#### Task #14 — Implement load testing with k6
**Stato**: Pending  
**Tempo stimato**: 60 min

**Cosa fare**:
1. Installare k6: `choco install k6` (Windows)
2. Creare script k6 in `benchmarks/k6/`:
   - `load_test.js`: Scenario base (50 VU, 5 min)
   - `stress_test.js`: Scenario stress (100 → 500 VU ramp)
   - `spike_test.js`: Scenario spike (0 → 200 VU immediate)
3. Script devono testare:
   - POST /query (RAG query)
   - POST /ingest (upload documenti)
   - GET /kb/:id/documents (lista documenti)
4. Metriche target:
   - p95 latency < 5s
   - Error rate < 1%
   - Throughput > 10 req/s
5. Aggiungere comando Makefile:
   ```makefile
   load-test:
       k6 run benchmarks/k6/load_test.js
   ```

**File da creare**:
- `benchmarks/k6/load_test.js`
- `benchmarks/k6/stress_test.js`
- `benchmarks/k6/spike_test.js`

**Prerequisiti**: Stack UP + documenti indicizzati

---

#### Task #19 — Run full verification suite
**Stato**: Pending  
**Tempo stimato**: 30 min (esecuzione) + 15 min (fix eventuali errori)

**Cosa fare**:
1. Eseguire tutti i test:
   ```bash
   make test-all          # Unit tests
   make test-e2e          # E2E tests (richiede stack UP)
   make bench-all         # Benchmarks (~30 min)
   make load-test         # k6 load test
   make audit-security    # Security audits
   ```
2. Verificare che tutti passino al 100%
3. Se fallimenti:
   - Analizzare log
   - Fixare problemi
   - Re-run test
4. Generare report riassuntivo:
   - Coverage totale (target: >80%)
   - Test passati/falliti
   - Benchmark KPI vs target
   - Vulnerabilità trovate

**Output atteso**:
- Tutti i test verdi ✅
- Nessun CVE HIGH/CRITICAL
- Benchmark KPI entro target (vedi `PIANO_IMPLEMENTAZIONE_RUST_PYTHON.md` §12)

---

### 📚 BLOCCO 4: Documentation

#### Task #17 — Update main documentation
**Stato**: Pending  
**Tempo stimato**: 45 min

**Cosa fare**:
1. Aggiornare `README.md`:
   - Sezione "Status": Fase 5 ✅ COMPLETED
   - Aggiungere badge CI/CD
   - Aggiungere screenshot (opzionale)
   - Quick start aggiornato con comandi nuovi
2. Aggiornare `docs/ARCHITECTURE.md`:
   - Diagramma aggiornato con Prometheus + Grafana
   - Sezione "Security" con reference a SECURITY_AUDIT_FASE_5.md
   - Sezione "Observability" con metriche esposte
3. Creare `CHANGELOG.md` (Keep a Changelog format):
   - Sezione [Unreleased]
   - Sezione [0.1.0] - 2026-05-07 con tutte le features Fase 1-5
4. Aggiornare `PIANO_IMPLEMENTAZIONE_RUST_PYTHON.md`:
   - Marcare Fase 5 come ✅ COMPLETED
   - Aggiornare tracker progress

**File da modificare**:
- `README.md`
- `docs/ARCHITECTURE.md`
- `CHANGELOG.md` (DA CREARE)
- `PIANO_IMPLEMENTAZIONE_RUST_PYTHON.md`

---

### 🎯 BLOCCO 5: Finalization

#### Task #21 — Git commit e documentazione finale
**Stato**: Pending  
**Tempo stimato**: 20 min

**Cosa fare**:
1. Verificare stato git:
   ```bash
   git status
   git diff
   ```
2. Verificare che nessun file sensibile sia staged (`.env`, secrets)
3. Stage modifiche:
   ```bash
   git add .
   ```
4. Commit con messaggio Conventional Commits:
   ```bash
   git commit -m "feat(fase-5): complete Testing, Benchmark & Hardening phase
   
   - Implemented comprehensive benchmark suite (ingest, query, hallucination, concurrent)
   - Added E2E tests with Playwright (login, chat, documents, comparison)
   - Security hardening: Rust (rate limit, security headers, constant-time auth)
   - Security hardening: Python (input validation, subprocess safety)
   - Security hardening: PHP (CSRF protection, session security)
   - Created observability stack (Prometheus + Grafana)
   - Implemented k6 load testing
   - Documented security audit (OWASP ASVS L2 compliant)
   - Updated CI/CD pipeline with security checks
   
   Test Results:
   - Unit tests: 100% pass
   - E2E tests: 100% pass
   - Benchmarks: All KPI targets met
   - Security audit: 0 HIGH/CRITICAL vulnerabilities
   
   Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
   ```
5. Push su feature branch:
   ```bash
   git push -u origin feature/fase-5-testing-hardening
   ```
6. Aprire Pull Request su GitHub:
   - Titolo: `[Fase 5] Testing, Benchmark & Hardening - COMPLETE`
   - Descrizione: Usare template `.github/PULL_REQUEST_TEMPLATE.md` (se esiste)
   - Assignee: te stesso
   - Labels: `fase-5`, `testing`, `security`, `observability`
7. Aspettare CI verde ✅ prima del merge

---

## 🎯 PRIORITÀ ESECUZIONE DOMANI

### Ordine consigliato:

1. **Task #9** (Python security) — 60 min
2. **Task #10** (PHP security) — 60 min
3. ☕ **PAUSA** (15 min)
4. **Task #11** (CI/CD) — 45 min
5. **Task #12** (Observability setup) — 60 min
6. **Task #13** (Grafana dashboards) — 45 min
7. ☕ **PAUSA** (15 min)
8. **Task #14** (k6 load test) — 60 min
9. **Task #19** (Full verification) — 45 min
10. **Task #20** (Security audit final) — 30 min
11. **Task #17** (Documentation) — 45 min
12. **Task #21** (Git commit & PR) — 20 min

**Tempo totale**: ~6 ore 45 min (pause incluse)

---

## 📦 STATO CORRENTE REPO

### Files modificati (non committati):
- `engine-rust/src/middleware/internal_auth.rs` (constant-time compare)
- `engine-rust/src/middleware/rate_limit.rs` (implementazione vera)
- `engine-rust/src/middleware/mod.rs` (export nuovi moduli)
- `engine-rust/src/main.rs` (apply middleware)
- `engine-rust/Cargo.toml` (aggiunta once_cell)
- `Makefile` (comandi test-e2e)
- `frontend/tests/e2e/chat.spec.ts` (NUOVO)
- `frontend/tests/e2e/documents.spec.ts` (NUOVO)
- `frontend/tests/e2e/comparison.spec.ts` (NUOVO)
- `docs/SECURITY_AUDIT_FASE_5.md` (NUOVO)

### Files da creare domani:
- `engine-python/app/middleware/security.py`
- `php-gateway/src/Middleware/CsrfMiddleware.php`
- `php-gateway/src/Middleware/RateLimitMiddleware.php`
- `docker-compose.observability.yml`
- `observability/prometheus/prometheus.yml`
- `observability/grafana/datasources.yml`
- `observability/grafana/dashboards/overview.json`
- `observability/grafana/dashboards/rag_pipeline.json`
- `benchmarks/k6/load_test.js`
- `benchmarks/k6/stress_test.js`
- `benchmarks/k6/spike_test.js`
- `CHANGELOG.md`

---

## ⚠️ NOTE IMPORTANTI

### Porte in uso (evitare conflitti):
- **9080**: PHP Gateway (già mappata)
- **8090**: Rust Engine (già mappata)
- **8091**: Python Worker NATIVO (NON Docker)
- **3307**: MySQL (già mappata)
- **6380**: Redis (già mappata)
- **6335**: Qdrant (già mappata)
- **11434**: Ollama (già mappata)
- **9090**: Prometheus (DA AGGIUNGERE)
- **3001**: Grafana (DA AGGIUNGERE, NON 3000!)

### Python Worker nativo:
Ricorda che il Python Worker **NON è in Docker**, si avvia manualmente:
```bash
cd engine-python
.\venv\Scripts\Activate.ps1
uvicorn app.main:app --host 0.0.0.0 --port 8091 --reload
```

### Prima di committare:
- ✅ Tutti i test passati
- ✅ `make lint` pulito
- ✅ `make audit-security` zero CVE HIGH/CRITICAL
- ✅ Nessun `.env` o secret in staging
- ✅ Python Worker fermo (non committare con server running)

---

## 🚀 COMANDO RAPIDO PER DOMANI

```bash
# 1. Avvia stack Docker (6 servizi)
make up

# 2. Avvia Python Worker nativo (terminale separato)
cd engine-python
.\venv\Scripts\Activate.ps1
uvicorn app.main:app --host 0.0.0.0 --port 8091 --reload

# 3. Verifica health
make health

# 4. Inizia task #9 (Python security hardening)
```

---

## 📞 CONTATTI & RIFERIMENTI

- **Piano Maestro**: `PIANO_IMPLEMENTAZIONE_RUST_PYTHON.md`
- **Istruzioni Claude**: `.claude/CLAUDE.md`
- **Runbook Operativo**: `docs/RUNBOOK.md`
- **Security Audit**: `docs/SECURITY_AUDIT_FASE_5.md`
- **Contract Prompts**: `docs/CONTRACT_ANALYSIS_PROMPTS.md`

---

**Buon lavoro domani! 🦀🐍🐘⚛️**
