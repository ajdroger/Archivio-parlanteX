# Archivio Parlante — Development Skill

**Tipo**: Coding Standards & Best Practices  
**Applicabile a**: Tutti i layer (Rust, Python, PHP, TypeScript/React)

---

## 🎯 Scopo

Questo skill definisce le regole di coding, naming, gestione dipendenze, e workflow git per il progetto **Archivio Parlante**. Claude Code DEVE seguire queste regole per mantenere coerenza e qualità del codice.

---

## 🦀 Rust — Engine Core

### Coding Standards

- **Edition**: 2021, MSRV 1.82
- **Formattazione**: `cargo fmt` obbligatorio prima di ogni commit
- **Linting**: `cargo clippy --all-targets -- -D warnings` DEVE passare senza warning
- **Error handling**:
  - ❌ **VIETATO** `.unwrap()` o `.expect()` in codice di produzione
  - ✅ Usare `?` operator + `anyhow::Context` per errori informativi
  - ✅ `thiserror` enum per errori applicativi con `#[from]` conversions
- **Async**: `tokio` multi-thread runtime, preferire `tokio::join!` / `futures::stream::buffer_unordered` per parallelismo
- **Logging**: `tracing::info!/warn!/error!` ovunque, ❌ mai `println!` o `eprintln!` in produzione
- **Documentazione**: struct/enum pubbliche → doc-comment con `///`
- **Testing**:
  - Test unitari `#[cfg(test)]` nello stesso file del modulo
  - Integration test in `tests/`
  - Naming: `test_nome_funzione_scenario_atteso`

### Dipendenze Ammesse

| Crate | Versione | Note |
|---|---|---|
| `tokio` | 1.40+ | Async runtime |
| `axum` | 0.7+ | Web framework |
| `reqwest` | 0.12+ | HTTP client |
| `qdrant-client` | 1.12+ | Vector DB ufficiale |
| `serde` + `serde_json` | 1.x | Serializzazione |
| `anyhow` + `thiserror` | 1.x | Error handling |
| `tracing` + `tracing-subscriber` | 0.3+ | Logging |
| `tantivy` | 0.22+ | BM25 sparse search |

**Licenze ammesse senza chiedere**: MIT, Apache 2.0, BSD-2/3, MPL-2.0  
**Licenze che richiedono AskUserQuestion**: GPL, AGPL, LGPL, commerciali

---

## 🐍 Python — AI Worker

### Coding Standards

- **Versione**: Python 3.11+
- **Type hints**: obbligatori, `mypy --strict` DEVE passare
- **Formattazione**: `ruff format` obbligatorio
- **Linting**: `ruff check --fix` senza errori
- **Logging**: `structlog` per log strutturati, ❌ mai `print()` in produzione
- **Async**: `asyncio` + `httpx.AsyncClient` per I/O concorrente
- **Validazione**: Pydantic v2 per modelli I/O
- **Sicurezza**:
  - ❌ Mai `shell=True` in `subprocess`
  - ✅ Whitelist comandi eseguibili
- **Testing**:
  - `pytest` + `pytest-asyncio` + `pytest-cov`
  - Fixture in `conftest.py`
  - Coverage minima: 80%

### Dipendenze Ammesse

| Libreria | Versione | Note |
|---|---|---|
| `fastapi` | 0.115+ | Web framework |
| `uvicorn[standard]` | 0.32+ | ASGI server |
| `pydantic` | 2.9+ | Validation |
| `httpx` | 0.28+ | HTTP async client |
| `torch` | 2.5+ | PyTorch runtime |
| `transformers` | 4.46+ | HuggingFace models |
| `unstructured[pdf,docx]` | 0.16+ | Document parsing |

---

## 🐘 PHP — Gateway

### Coding Standards

- **Versione**: PHP 8.2+
- **Strict types**: `declare(strict_types=1);` in OGNI file
- **Coding style**: PSR-12
- **Static analysis**: PHPStan level 8 DEVE passare
- **Logging**: PSR-3 logger (`monolog/monolog`), ❌ mai `echo` o `var_dump()` in produzione
- **Dependency Injection**: `php-di/php-di`
- **Database**: PDO prepared statements, ❌ mai concat string per SQL
- **Testing**: PHPUnit, coverage > 80%

### Dipendenze Ammesse

| Package | Versione | Note |
|---|---|---|
| `slim/slim` | 4.x | Micro-framework |
| `monolog/monolog` | 3.x | Logger PSR-3 |
| `php-di/php-di` | 7.x | DI container |
| `guzzlehttp/guzzle` | 7.x | HTTP client |

---

## ⚛️ TypeScript/React — Frontend

### Coding Standards

- **TypeScript**: strict mode (`strict: true`, `noImplicitAny`, `strictNullChecks`)
- **React**: 18+ functional components + hooks, ❌ mai class components
- **Linting**: ESLint + Prettier configurati
- **Type check**: `tsc --noEmit` DEVE passare
- **State**: Zustand per global state, react-query per data fetching
- **Accessibilità**: aria-labels, focus management, contrasto AAA
- **Sicurezza**:
  - ❌ Mai `localStorage`/`sessionStorage` per dati sensibili (usa httpOnly cookie)
  - ❌ Mai `dangerouslySetInnerHTML` con input utente
- **Testing**: Vitest + React Testing Library + Playwright E2E, coverage > 70%

---

## 📂 Database — MySQL

### Naming Conventions

- **Database**: `archivio_parlante_x` (vincolante, già creato via phpMyAdmin)
- **Prefix tabelle**: `ap_` (es. `ap_users`, `ap_documents`, `ap_chat_messages`)
- **Charset**: `utf8mb4` con collation `utf8mb4_unicode_ci`
- **Engine**: InnoDB con foreign key dichiarate
- **Migrations**: in `db/migrations/`, ordinate `001_*`, `002_*`, eseguite automaticamente
- **Naming colonne**: snake_case, singolare per FK (`user_id` non `users_id`)
- **Timestamp**: ogni tabella ha `created_at` + `updated_at` (DATETIME, default CURRENT_TIMESTAMP)
- **Soft delete**: `deleted_at NULL` invece di DELETE fisica per `ap_documents`, `ap_chat_messages`, `ap_users`

---

## 🌐 Lingua

| Contesto | Lingua |
|---|---|
| **Codice**: variabili, funzioni, classi, moduli | **Inglese** |
| **Commit messages** | **Inglese** (Conventional Commits) |
| **Log tecnici** | **Inglese** |
| **Commenti su algoritmi/pattern** | **Inglese** |
| **Commenti logica dominio legale/contrattuale** | **Italiano** |
| **Documentazione utente finale** | **Italiano** |
| **UI labels e messaggi** | **Italiano** |
| **Prompt LLM per analisi contratti** | **Italiano** |

---

## 🔀 Git Workflow

### Branch Strategy

- **`main`**: solo release stabili, protetto, no commit diretti
- **`develop`**: integrazione continua, protetto, merge solo via PR
- **`feature/fase-<N>-<slug>`**: lavoro di sviluppo (es. `feature/fase-1-1-rust-scaffolding`)
- **`hotfix/<slug>`**: fix urgenti su `main`, poi back-merge in `develop`

### Conventional Commits

**Tipi ammessi**: `feat`, `fix`, `refactor`, `perf`, `docs`, `test`, `chore`, `ci`, `build`, `security`

**Formato**:
```
<tipo>(<scope opzionale>): <descrizione breve>

<corpo opzionale con dettagli>
<footer opzionale>
```

**Esempi**:
```
feat(rust): implement hybrid search with RRF fusion
fix(php): prevent SQL injection in document query
refactor(python): extract BGE reranker to separate service
perf(rust): parallelize embedding generation with semaphore
docs(readme): update quick start with ollama-pull command
test(rust): add integration test for multi-contract comparison
security(deps): upgrade axum to 0.7.5 (CVE-2024-XXXXX)
```

### Divieti Assoluti

- ❌ `git push --force` su `main` o `develop`
- ❌ `git commit --no-verify` (se hook fallisce, fixa la causa)
- ❌ Committare `.env`, credenziali, chiavi private, dati sensibili
- ❌ Amend di commit già pushati
- ❌ Reset hard su branch condivisi

---

## 🔒 Sicurezza

### Regole Non Negoziabili

1. **Input validation**: SEMPRE su ogni endpoint (lunghezza, tipo, whitelist, sanitizzazione)
2. **SQL Injection**: SOLO prepared statements, mai string concat
3. **XSS**: React auto-escape, CSP header, SameSite=Strict cookies
4. **Path traversal**: normalizzazione path, reject `../`, canonicalize
5. **Command injection**: mai `shell=True`, mai `exec()`, mai input utente in comandi
6. **Secrets**: mai hardcoded, mai loggati, `.env` in `.gitignore`
7. **Dependencies**: `cargo audit`, `pip-audit`, `composer audit`, `npm audit` — zero High/Critical

---

## 📦 Dipendenze — Processo di Aggiunta

Prima di aggiungere qualsiasi dipendenza:

1. **Verifica licenza**:
   - `cargo metadata` (Rust)
   - `pip show <package>` (Python)
   - `composer show <package>` (PHP)
   - `npm view <package>` (Node)

2. **Licenze ammesse senza chiedere**: MIT, Apache 2.0, BSD-2/3, MPL-2.0, ISC

3. **Licenze che richiedono AskUserQuestion**:
   - LGPL (solo dynamic linking)
   - GPL / AGPL (effetto copyleft)
   - Commerciali / Proprietary
   - BUSL / SSPL
   - Sconosciuta / nessuna LICENSE

4. **Vietate**: dipendenze con costo ricorrente, SaaS obbligatorio

---

## 🧪 Testing — Coverage Minima

| Layer | Coverage Minima | Tool |
|---|---|---|
| Rust | 80% | `cargo test --release` + `tarpaulin` |
| Python | 80% | `pytest --cov` |
| PHP | 80% | PHPUnit |
| Frontend | 70% | Vitest + Playwright |

**Regola d'oro**: se `make test-all` non passa al 100%, la fase non è chiusa, non si committa, non si apre PR.

---

## ✅ Checklist Prima di Ogni Commit

- [ ] Formattazione eseguita (`cargo fmt`, `ruff format`, `php-cs-fixer fix`, `prettier`)
- [ ] Linting pulito (`clippy -D warnings`, `ruff check`, `phpstan L8`, `eslint`)
- [ ] Test passano al 100%
- [ ] Nessun `.unwrap()` / `print()` / `echo` / `console.log` in produzione
- [ ] Nessun secret committato
- [ ] Doc-comment aggiornati
- [ ] CHANGELOG.md aggiornato se applicabile

---

**Ultimo aggiornamento**: 2025-04-21 — Fase -1
