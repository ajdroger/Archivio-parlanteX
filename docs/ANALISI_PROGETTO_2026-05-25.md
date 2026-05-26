# Analisi completa progetto — Archivio ParlanteX

| Metadato | Valore |
|---|---|
| **Data analisi** | 2026-05-25 |
| **Repository** | `c:\Users\aj_93\OneDrive\Documenti\GitHub\Archivio-parlanteX` |
| **Branch** | `develop` (tracking `origin/develop`) |
| **HEAD commit** | `7812364` — `fix(tests): resolve all 74 KB access control test failures (100% pass rate)` |
| **Analista** | Sessione agente Cursor (analisi + debug) |
| **Commit report** | Non eseguito (su richiesta utente) |

---

## 1. Stato repository

### Branch e cronologia recente

| Voce | Valore |
|---|---|
| Branch corrente | `develop` |
| Allineamento remote | `develop...origin/develop` |
| Ultimo commit | `7812364` |

**Ultimi commit rilevanti:**

| Hash | Messaggio |
|---|---|
| `7812364` | fix(tests): resolve all 74 KB access control test failures (100% pass rate) |
| `e05b288` | docs: session complete part 2 - KB access tests 45/74 passing |
| `229d9d4` | merge: sync develop into main |
| `fc626fe` | fix(tests): improve kb_access_control tests - 45/74 passing |
| `b13def1` | docs: session complete 2026-05-21 - all 131/131 tests passing |

### Working tree (non committato)

**26 file modificati** (+106 / −50 righe) — fix sessione precedente ancora solo in working tree:

| Area | File chiave | Modifica |
|---|---|---|
| Rust | `engine-rust/src/lib.rs` | Aggiunto `pub mod middleware;` |
| PHP | `php-gateway/src/Service/RustEngineProxy.php` | Metodi `query()` / `ingest()` / `compare()`; rimosso `final` |
| Porte | `Makefile`, `README.md`, `.env.example`, `frontend/vite.config.ts`, … | Coexistence 9080/3307/6380/6335 |
| Rust E2E | `engine-rust/tests/ingestion_e2e.rs`, `full_workflow_e2e.rs` | Default Qdrant host `6335` |
| Docs | `docs/RUNBOOK.md`, checklist, … | Allineamento porte |

### File untracked (da gestire)

| Path | Tipo | Azione consigliata |
|---|---|---|
| `docs/PORTS_COEXISTENCE.md` | Documentazione | Committare |
| `frontend/.env.example` | Config frontend | Committare |
| `.cursor/rules/ports-coexistence.mdc` | Regola Cursor | Committare (opzionale) |
| `docs/ADR/0006,0007,0008,0010,0012,0013,0015,0016*.md` | ADR | Committare |
| `docs/FASE_2_VERIFICATION.md`, `FASE_3_VERIFICATION.md`, `FASE_5_VERIFICATION.md` | Verifiche fase | Committare |
| `engine-rust/final_test_results.txt`, `test_results.txt` | Output test locali | **Non committare** — aggiungere a `.gitignore` |
| `php-gateway/.phpunit.cache/` | Cache coverage | **Non committare** — aggiungere a `.gitignore` |

### Gap critico nel repository

- **`PIANO_IMPLEMENTAZIONE_RUST_PYTHON.md`** — referenziato da `README.md`, `CLAUDE.md`, `STATUS.md` ma **assente** nel repository.

---

## 2. Salute test

Esecuzione in ambiente Windows locale (2026-05-22).

### Riepilogo per layer

| Layer | Comando | Esito | Numeri |
|---|---|---|---|
| **Rust (lib)** | `cargo test --lib` | ✅ PASS | **135 passed**, 0 failed, 0 ignored |
| **Rust (bin)** | `cargo build --bin archivio-parlante-rust-engine` | ✅ OK | 0 errori; ~45 warning |
| **Rust (full)** | `cargo test` | ❌ FAIL (compile) | Test `kb_access_complete_suite`: **54 errori sqlx** (`DATABASE_URL` o cache `.sqlx/` mancante) |
| **PHP** | `composer test` | ✅ PASS | **69 tests**, **308 assertions**, **1 skipped** |
| **PHP coverage** | (da report PHPUnit) | ⚠️ Bassa | Classes 16.67%, Methods 29.63%, **Lines 49.40%** (617/1249) |
| **Frontend** | `npm run test:run` | ✅ PASS | **53 passed** (5 file), durata ~1.5s |
| **Python** | `pytest tests/` (venv parziale) | ⚠️ PARZIALE | **21 passed**, **23 failed**, 1 warning |

### Dettaglio Rust

- **`cargo test --lib`**: include test del modulo `middleware` dopo export in `lib.rs` (da 118 a 135 test unitari).
- **`cargo test` completo**: fallisce in compilazione del crate di test `kb_access_complete_suite` per macro `sqlx::query!` senza connessione MySQL o dati offline in `.sqlx/`.
- **File test integration** in `engine-rust/tests/`: 9 file (`ingestion_e2e.rs`, `query_e2e.rs`, `comparison_e2e.rs`, `full_workflow_e2e.rs`, `kb_access_complete_suite.rs`, `test_kb_access_control_complete.rs`, `chunker_test.rs`, `ollama_smoke.rs`, `common/mod.rs`).

### Dettaglio PHP

- Tutti i test unitari passano dopo fix `RustEngineProxy` (metodi proxy + mock PHPUnit).
- **1 test skipped** (non bloccante per la suite).

### Dettaglio Frontend

- Vitest 4.1.5 — nessun fallimento sui 53 test nei 5 file.

### Dettaglio Python (23 fallimenti)

| File test | Problema probabile |
|---|---|
| `tests/test_parse.py` | Worker non in esecuzione / fixture file |
| `tests/test_pdf_parser.py` | Dipendenze ML (`unstructured`, PyMuPDF) / retry |
| `tests/test_rerank.py` | Modello BGE reranker non caricato |

Causa ambiente: `requirements.txt` non installato per intero nel venv; test di integrazione richiedono stack Docker + worker su **8091**.

---

## 3. Regressioni scan

| Controllo | Esito | Note |
|---|---|---|
| `localhost:8080` per PHP Gateway ParlanteX | ✅ **0 occorrenze** | 8080 riservata a `archivio-parlante-starter` |
| `pub mod middleware` in `engine-rust/src/lib.rs` | ✅ Presente | |
| `RustEngineProxy::query/ingest/compare` | ✅ Presente | Fix bug critico runtime |
| Vite proxy `/api` → `http://localhost:9080` | ✅ `frontend/vite.config.ts` | |
| `make health` PHP / Qdrant | ✅ | `9080/health`, `6335` |

---

## 4. Docker ports vs `.env` — coexistence con archivio-parlante-starter

### Porte ESTERNE (host Windows) — ParlanteX

| Servizio | Mapping Docker | Porta host | Riservata allo starter |
|---|---|---|---|
| PHP Gateway | `9080:80` | **9080** | **8080** |
| MySQL | `3307:3306` | **3307** | **3306** (AMPPS) |
| Redis | `6380:6379` | **6380** | **6379** |
| Qdrant REST | `6335:6333` | **6335** | **6333** (se usata) |
| Qdrant gRPC | `6336:6334` | **6336** | — |
| Rust Engine | `8090:8090` | 8090 | — |
| Python Worker | `8091:8091` | 8091 | — |
| Ollama | `11434:11434` | 11434 | Condivisa (OK) |
| Frontend Vite (dev) | — | 5173 | — |

**Mai** mappare ParlanteX su **8080, 3306, 6379, 6333** sul host.

### URL INTERNI (`.env` su rete `archivio_net`)

| Variabile | Valore corretto |
|---|---|
| `MYSQL_HOST` | `mysql` |
| `MYSQL_PORT` | `3306` (non 3307) |
| `REDIS_URL` | `redis://redis:6379` (non 6380) |
| `QDRANT_URL` | `http://qdrant:6333` (non 6335) |
| `OLLAMA_URL` | `http://ollama:11434` |
| `RUST_ENGINE_URL` | `http://rust-engine:8090` |
| `PYTHON_WORKER_URL` | `http://python-worker:8091` (o `host.docker.internal:8091` se worker nativo) |

### Conformità verificata

- `docker-compose.yml` — mapping esterni corretti (non modificati in questa analisi).
- `.env.example` — valori interni corretti + commenti coexistence + `CORS_ORIGINS` con `http://localhost:9080`.
- Documentazione: `docs/PORTS_COEXISTENCE.md` (untracked) descrive tabella completa.

### Nota `docker-compose.yml` — Python nativo

Il servizio `rust-engine` usa `PYTHON_WORKER_URL=http://host.docker.internal:8091` quando il worker gira fuori da Docker (WSL2/Windows). Coerente con setup documentato.

---

## 5. Bug trovati / fix applicati / gap aperti

### Fix applicati (working tree, non committati)

| ID | Severità | Path | Descrizione fix |
|---|---|---|---|
| **B1** | **Critica** | `php-gateway/src/Service/RustEngineProxy.php` | `ProxyController` chiamava `query()`/`ingest()`/`compare()` ma la classe esponeva solo `proxyRequest()` → **fatal runtime** su API proxy. Aggiunti wrapper + rimosso `final` per mock PHPUnit. |
| **B2** | **Alta** | `engine-rust/src/lib.rs` | Modulo `middleware` non esportato → test `kb_access_complete_suite` non compilava. Aggiunto `pub mod middleware;`. |
| **B3** | **Media** | Makefile, README, Vite, `.env.example`, docs | Allineamento porte 9080/3307/6380/6335 vs starter. |

**Verifica post-fix:** PHP **69/69** test OK; Rust lib **135/135** test OK.

### Gap / bug ancora aperti

| ID | Severità | Descrizione | Path / evidenza |
|---|---|---|---|
| **G1** | Alta | `cargo test` full non compila senza `DATABASE_URL` o sqlx offline | `engine-rust/tests/kb_access_complete_suite.rs` |
| **G2** | Alta | Fix B1/B2/B3 solo in working tree locale | `git status` — non su `origin/develop` |
| **G3** | Media | Python **23/44** test falliscono senza stack ML completo | `engine-python/tests/` |
| **G4** | Media | `STATUS.md` obsoleto (2026-05-17, 118 test) | `STATUS.md` |
| **G5** | Media | README dichiara "100% Production Ready" (v0.8.0) | `README.md` |
| **G6** | Bassa | CI PHP: `composer test \|\| true` non blocca pipeline | `.github/workflows/ci.yml` |
| **G7** | Bassa | Coverage PHP **49%** << target 80% | Report PHPUnit |

---

## 6. Gap documentazione

| Documento | Stato | Azione |
|---|---|---|
| `PIANO_IMPLEMENTAZIONE_RUST_PYTHON.md` | ❌ Mancante | Ripristinare o rimuovere tutti i riferimenti |
| `STATUS.md` | ❌ Obsoleto (2026-05-17) | Aggiornare a 135 test Rust, fase corrente, porte 9080 |
| `docs/PORTS_COEXISTENCE.md` | ✅ Creato (untracked) | Committare |
| `README.md` | ⚠️ Porte OK in WT; claim marketing eccessivo | Allineare verdict a evidenze test |
| `docs/ARCHITECTURE.md` | ✅ Coerente con stack | — |
| `docs/RUNBOOK.md` | ✅ Porte 9080/6335; link coexistence in WT | — |
| ADR 0006–0016 (8 file) | Untracked | Committare se approvati |
| `FASE_*_VERIFICATION.md` (3 file) | Untracked | Committare |

---

## 7. Raccomandazioni top 5

1. **Commit atomico** su branch feature: fix PHP proxy + `pub mod middleware` + porte + `docs/PORTS_COEXISTENCE.md` (escludere `.phpunit.cache/`, `*_test_results.txt`).

2. **Abilitare build Rust offline:** eseguire `cargo sqlx prepare` con MySQL su `localhost:3307` e committare directory `.sqlx/`, **oppure** aggiungere servizio MySQL in CI con `DATABASE_URL=mysql://root@localhost:3307/archivio_parlante_x`.

3. **Python:** `pip install -r requirements.txt` nel venv; avviare stack (`make up`) + worker su **8091**; rieseguire `pytest` fino a verde o marcare test integration come `@pytest.mark.integration`.

4. **Allineare narrativa progetto:** aggiornare `STATUS.md` e README — rimuovere "100% production ready" finché integration/E2E non sono verdi in CI.

5. **Ripristinare piano maestro** `PIANO_IMPLEMENTAZIONE_RUST_PYTHON.md` o aggiornare `CLAUDE.md` / README con nuovo percorso documento.

---

## 8. Verdict

### **Non production-ready**

Il progetto ha uno **stack multi-layer sostanziale** e i **test unitari core sono verdi**, ma non soddisfa i criteri di chiusura fase (100% test, CI, documentazione allineata, fix in git).

| Criterio | Evidenza | Esito |
|---|---|---|
| Unit test Rust lib | 135/135 pass | ✅ |
| Unit test PHP | 69/69 pass (1 skip) | ✅ |
| Unit test Frontend | 53/53 pass | ✅ |
| `cargo test` completo | Compile fail sqlx su KB suite | ❌ |
| Test Python | 21 pass / 23 fail | ❌ |
| Fix critici in git remoto | Solo working tree | ❌ |
| Stack Docker E2E verificato | Non eseguito in questa sessione | ⏳ |
| Piano maestro presente | File assente | ❌ |
| STATUS / README vs realtà | Obsoleto / iper-ottimistico | ❌ |
| CI blocca su fallimenti PHP | `\|\| true` su test | ⚠️ |

**Verdict intermedio:** adatto a **sviluppo locale** con `make up`, frontend Vite (`5173` → proxy `9080`) **dopo commit dei fix**; **non** adatto a rilascio produzione o merge su `main` senza sqlx CI, integration test verdi e allineamento documentazione.

---

## 9. Note operative

### Frontend dev vs Docker

| Modalità | URL UI | URL API |
|---|---|---|
| Vite dev (consigliato) | `http://localhost:5173` | `/api` (proxy → `http://localhost:9080`) |
| Chiamata diretta API | — | `VITE_API_BASE_URL=http://localhost:9080/api` |
| UI servita da PHP container | `http://localhost:9080` | Stesso host `/api` |

Non confondere la porta **5173** (solo dev frontend) con **9080** (gateway API).

### Python worker

- Può girare come container Docker sulla porta **8091** o **nativo** su Windows/WSL2.
- In compose, `rust-engine` punta a `host.docker.internal:8091` quando il worker è nativo.

### Ollama (11434)

Porta **condivisa** con `archivio-parlante-starter`. Accettabile se un solo demone Ollama; possibile contesa risorse se entrambi i progetti caricano modelli pesanti.

### Comandi verifica rapida

```bash
make health
curl -s http://localhost:9080/health
curl -s http://localhost:8090/health
curl -s http://localhost:6335/
```

### Riferimenti

- Coexistence porte: `docs/PORTS_COEXISTENCE.md`
- Runbook operativo: `docs/RUNBOOK.md`
- Setup locale Windows: `SETUP_LOCALE.md`

---

*Report generato automaticamente — non sostituisce security audit di fase né esecuzione E2E con stack completo.*
