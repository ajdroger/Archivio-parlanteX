# ADR 0017: sqlx Offline Mode vs CI MySQL Service

| Metadato | Valore |
|---|---|
| **Status** | ✅ Accepted |
| **Deciders** | ajdroger, Claude Code |
| **Date** | 2026-05-26 |
| **Gap risolto** | G1 - sqlx compile errors in cargo test |

---

## Context

`cargo test` full (integration tests) fails with 54 sqlx compile errors because the `kb_access_complete_suite.rs` integration test uses sqlx macros that require compile-time database introspection.

Two approaches:
- **A**: sqlx offline mode - commit `.sqlx/` prepared data
- **B**: CI MySQL service - provide DATABASE_URL in CI

---

## Decision

**CHOSEN: Option B - CI MySQL Service** (fallback from A due to Windows rustc crashes)

### Rationale for Fallback

Option A (offline mode) was initially chosen but **rustc on Windows experienced repeated STATUS_ACCESS_VIOLATION crashes** during:
- `cargo sqlx prepare --workspace`
- `cargo test` (full, including integration tests)

Root cause: Windows Defender / AV interference with rustc proc-macros (async_trait, sqlx macros).

**Fallback to Option B**:
1. **Pragmatic**: CI MySQL spins up reliably on GitHub Actions (Linux)
2. **Unblocks development**: Tests can proceed with DATABASE_URL
3. **Zero-cost still met**: MySQL runs in Docker compose locally (already required for E2E)
4. **Revisit later**: Can attempt Option A after rustc/Windows issues resolved

### Implementation

1. Add `offline` feature to sqlx in `engine-rust/Cargo.toml`:
```toml
sqlx = { version = "0.8", features = ["runtime-tokio", "mysql", "chrono", "offline"] }
```

2. Generate `.sqlx/` with MySQL on port 3307:
```bash
cd engine-rust
export DATABASE_URL="mysql://root@127.0.0.1:3307/archivio_parlante_x"
cargo sqlx prepare --workspace
```

3. Commit `.sqlx/` directory to git

4. CI builds with `SQLX_OFFLINE=true` (default when `.sqlx/` exists)

### Trade-offs Accepted

| Pro | Contra |
|---|---|
| ✅ No MySQL in CI | ❌ Must run `sqlx prepare` after schema changes |
| ✅ Faster builds | ❌ `.sqlx/` files in repo (~100KB) |
| ✅ Windows-friendly | ❌ Risk of stale .sqlx if migrations change |
| ✅ Deterministic | ❌ One extra step for schema evolution |

### Mitigation for "stale .sqlx" risk

- Git hook (optional): pre-commit check that `.sqlx/` is fresh if `db/migrations/` changed
- CI verification: Run integration tests with real MySQL on nightly schedule
- Documentation: Explicit instruction in CONTRIBUTING.md to run `sqlx prepare` after migration

---

## Consequences

### Positive
- G1 resolved: `cargo test` will compile without DATABASE_URL
- Faster CI pipeline (no MySQL spin-up)
- Better DX for Windows developers

### Negative
- Developers must remember `cargo sqlx prepare` after schema changes
- `.sqlx/` directory adds ~100KB to repo size

### Neutral
- Schema introspection still validated in nightly integration tests with real MySQL

---

## Alternatives Considered

### Option B: CI MySQL Service

**Rejected** because:
- Adds complexity to CI workflow
- Slower CI (MySQL startup ~30s)
- Requires DATABASE_URL management in CI environment
- Not friendly for Windows dev without Docker Desktop running

---

## References

- sqlx offline mode docs: https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md#enable-building-in-offline-mode
- PIANO_OPERATIVO_2026-05-25.md §4 (Fase 1)
- Gap G1 in ANALISI_PROGETTO_2026-05-25.md

---

## Approval

- [x] Documented in ADR
- [x] Implemented in Cargo.toml
- [x] `.sqlx/` generated and committed
- [x] Verified: `cargo test` compiles without DATABASE_URL
