# Pull Request — Fase X.Y: [Titolo Descrittivo]

## 📋 Descrizione

<!-- Breve descrizione delle modifiche apportate -->

**Fase completata**: Fase X.Y — [Nome Fase]

**Branch**: `feature/fase-X-Y-<slug>`

---

## ✅ Checklist 8-Step Obbligatoria (§0.8 del Piano)

Questa PR **NON può essere approvata** finché tutti gli step non sono verificati:

### Step 1 — Ricerca Preliminare
- [ ] Ricerca OSS/MCP/plugin completata e documentata (se applicabile)
- [ ] Risultati ricerca in `docs/` o note inline nel codice

### Step 2 — Pianificazione & Esecuzione
- [ ] TodoList della fase creata e tutti i task completati al 100%
- [ ] Nessun task rimasto in sospeso o saltato

### Step 3 — Sviluppo + Testing
- [ ] Suite test della fase passa al **100%**:
  - [ ] `cargo test --release` (se Rust)
  - [ ] `pytest --cov` (se Python)
  - [ ] `composer test` (se PHP)
  - [ ] `vitest run` + `playwright test` (se Frontend)
- [ ] Coverage > 80% (Rust/Python/PHP) o > 70% (Frontend)
- [ ] Screenshot/log di test passati allegato (opzionale ma consigliato)

### Step 4 — Ottimizzazione Performance
- [ ] Profiling eseguito (flamegraph Rust / py-spy Python / k6 endpoint)
- [ ] Colli di bottiglia identificati e risolti
- [ ] **Test ripassati al 100%** dopo ottimizzazioni

### Step 5 — Clean Code & Linting
- [ ] `cargo fmt` + `cargo clippy --all-targets -- -D warnings` (Rust)
- [ ] `ruff format` + `ruff check --fix` + `mypy --strict` (Python)
- [ ] `php-cs-fixer fix` + `phpstan analyse --level=8` (PHP)
- [ ] `eslint --fix` + `prettier --write` + `tsc --noEmit` (Frontend)
- [ ] Dead code rimosso, commenti obsoleti eliminati
- [ ] **Test ripassati al 100%** dopo pulizia

### Step 6 — Security Audit
- [ ] Report audit creato: `docs/SECURITY_AUDIT_<fase>.md`
- [ ] Checklist OWASP ASVS L2 verificata (input validation, auth, SQLi, XSS, path traversal, secrets, deps)
- [ ] `cargo audit` (Rust) — zero vulnerabilità High/Critical
- [ ] `pip-audit` / `safety check` (Python) — zero vulnerabilità High/Critical
- [ ] `composer audit` (PHP) — zero vulnerabilità High/Critical
- [ ] `npm audit` (Frontend) — zero vulnerabilità High/Critical
- [ ] `trivy image` su Dockerfile — zero vulnerabilità High/Critical
- [ ] Tutte le vulnerabilità ≥ High risolte

### Step 7 — Aggiornamento Documentazione
- [ ] `README.md` aggiornato (se cambiano comandi/setup)
- [ ] `CHANGELOG.md` aggiornato con entry della fase
- [ ] `docs/ARCHITECTURE.md` aggiornato (se architettura cambia)
- [ ] `docs/RUNBOOK.md` aggiornato (se troubleshooting operativo cambia)
- [ ] ADR creato se decisione architetturale rilevante (`docs/ADR/XXXX-*.md`)
- [ ] `.claude/CLAUDE.md` aggiornato se cambia workflow dev

### Step 8 — Git Workflow
- [ ] Branch naming corretto: `feature/fase-<N>-<slug>`
- [ ] Conventional Commits usati: `feat|fix|refactor|perf|docs|test|chore|ci|build|security`
- [ ] **Nessun secret committato** (.env, credenziali, chiavi private verificati)
- [ ] Rebased su `develop` (nessun conflitto)
- [ ] CI verde (tutti i workflow GitHub Actions passati)

---

## 🔍 Modifiche Principali

<!-- Lista dei file/moduli principali modificati con breve spiegazione -->

- `engine-rust/src/...` — [descrizione]
- `engine-python/app/...` — [descrizione]
- `docs/...` — [descrizione]

---

## 📊 Risultati Test

```bash
# Rust
cargo test --release
# [incolla output o conferma 100% pass]

# Python
pytest --cov
# [incolla output o conferma 100% pass]

# Etc.
```

---

## 🛡️ Security Audit

- **Vulnerabilità trovate**: [numero, se zero scrivi "0"]
- **Vulnerabilità risolte**: [numero]
- **Report completo**: `docs/SECURITY_AUDIT_<fase>.md`

---

## 📈 Performance

<!-- Se applicabile, risultati benchmark prima/dopo ottimizzazioni -->

- Latency p95: [prima] → [dopo]
- Throughput: [prima] → [dopo]
- Memory usage: [prima] → [dopo]

---

## 🤔 Note per Reviewer

<!-- Eventuali punti critici da rivedere, trade-off fatti, decisioni discusse -->

---

## 📎 Link Utili

- Issue correlata: #[numero] (se esiste)
- ADR: `docs/ADR/XXXX-*.md` (se creato)
- Branch: `feature/fase-<N>-<slug>`

---

**Assegna a**: @reviewer-name
**Milestone**: Fase X

---

<!-- Firma GPG (se richiesto dal cliente): Commit firmato con chiave [fingerprint] -->
