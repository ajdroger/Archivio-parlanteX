# Archivio Parlante — Testing Skill

**Tipo**: Testing Checklist & Quality Assurance  
**Applicabile a**: Tutti i layer (Rust, Python, PHP, TypeScript/React)

---

## 🎯 Scopo

Questo skill definisce i requisiti di testing, coverage, benchmark, e quality gates per il progetto **Archivio Parlante**. Ogni fase DEVE passare tutti i test al 100% prima di essere considerata chiusa.

---

## 🧪 Tipologie di Test

### 1. Unit Test

**Scopo**: Testare singole funzioni/metodi in isolamento

#### Rust (`cargo test`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunker_splits_on_headers() {
        let input = "# Title\nContent\n## Subtitle\nMore";
        let chunks = chunk_by_headers(input, 500);
        assert_eq!(chunks.len(), 2);
        assert!(chunks[0].contains("# Title"));
    }

    #[tokio::test]
    async fn test_ollama_embed_returns_768_dims() {
        let provider = OllamaProvider::new("http://localhost:11434");
        let result = provider.embed(&["test"], "nomic-embed-text").await.unwrap();
        assert_eq!(result[0].len(), 768);
    }
}
```

**Coverage minima**: 80%  
**Tool**: `cargo tarpaulin` o `cargo llvm-cov`  
**Comando**: `cargo test --release`

---

#### Python (`pytest`)

```python
import pytest
from app.services.pdf_parser import parse_pdf

def test_parse_pdf_extracts_text():
    result = parse_pdf("fixtures/sample.pdf")
    assert result.text is not None
    assert result.page_count > 0

@pytest.mark.asyncio
async def test_reranker_scores_relevant_higher():
    from app.services.reranker import rerank_chunks
    query = "clausola penale"
    candidates = [
        ("chunk1", "penale 1000 euro"),
        ("chunk2", "irrilevante testo")
    ]
    scored = await rerank_chunks(query, candidates)
    assert scored[0][1] > scored[1][1]  # score chunk1 > chunk2
```

**Coverage minima**: 80%  
**Tool**: `pytest-cov`  
**Comando**: `pytest --cov=app --cov-report=term --cov-report=html`

---

#### PHP (PHPUnit)

```php
<?php
declare(strict_types=1);

namespace Tests\Unit;

use PHPUnit\Framework\TestCase;
use App\Service\Auth\JwtService;

class JwtServiceTest extends TestCase
{
    public function testGenerateTokenReturnsValidJwt(): void
    {
        $service = new JwtService('secret-key');
        $token = $service->generate(['user_id' => 123]);
        $this->assertIsString($token);
        $this->assertMatchesRegularExpression('/^[\w-]+\.[\w-]+\.[\w-]+$/', $token);
    }

    public function testVerifyTokenReturnsPayload(): void
    {
        $service = new JwtService('secret-key');
        $token = $service->generate(['user_id' => 123]);
        $payload = $service->verify($token);
        $this->assertEquals(123, $payload['user_id']);
    }
}
```

**Coverage minima**: 80%  
**Tool**: PHPUnit con Xdebug/PCOV  
**Comando**: `composer test` (configurato per `phpunit --coverage-html coverage/`)

---

#### Frontend (Vitest + React Testing Library)

```typescript
import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { ChatMessage } from './ChatMessage'

describe('ChatMessage', () => {
  it('renders user message with citation', () => {
    const message = {
      role: 'assistant',
      content: 'La penale è di 1000 euro [doc_123:chunk_5]',
      citations: [{ doc_id: 'doc_123', chunk_idx: 5, text_quote: '...' }]
    }
    render(<ChatMessage message={message} />)
    expect(screen.getByText(/La penale è di 1000 euro/)).toBeInTheDocument()
    expect(screen.getByText(/doc_123:chunk_5/)).toBeInTheDocument()
  })
})
```

**Coverage minima**: 70%  
**Tool**: Vitest  
**Comando**: `npm run test:unit` (configurato per `vitest run --coverage`)

---

### 2. Integration Test

**Scopo**: Testare interazione tra componenti (es. Rust → Python worker, Rust → Qdrant)

#### Rust Integration Test (`tests/integration_ingest.rs`)

```rust
#[tokio::test]
async fn test_full_ingest_pipeline() {
    // Setup containers (testcontainers-rs)
    let qdrant = QdrantContainer::new().start().await;
    let ollama = OllamaContainer::new().start().await;
    
    let app_state = AppState::new(/* config con container URLs */);
    
    // Ingest documento
    let doc_path = "fixtures/contract_sample.pdf";
    let result = ingest_document(&app_state, doc_path, "kb_test").await.unwrap();
    
    assert!(result.chunks_indexed > 0);
    assert!(result.processing_ms < 60000);
    
    // Verifica su Qdrant
    let points = qdrant_client.search("ap_kb_kb_test", /* query */).await.unwrap();
    assert!(!points.is_empty());
}
```

**Comando**: `cargo test --test integration_*`

---

#### Python Integration Test

```python
@pytest.mark.integration
@pytest.mark.asyncio
async def test_parse_and_extract_kg_full_flow():
    # Parse documento
    parsed = await parse_document("fixtures/nda_sample.docx")
    assert parsed.text is not None
    
    # Extract knowledge graph
    kg = await extract_knowledge_graph(parsed.text)
    assert len(kg.nodes) > 0
    assert any(n.entity_type == "PARTY" for n in kg.nodes)
```

**Comando**: `pytest -m integration`

---

### 3. End-to-End Test

**Scopo**: Testare flusso utente completo da browser

#### Playwright (`tests/e2e/upload-and-query.spec.ts`)

```typescript
import { test, expect } from '@playwright/test'

test('upload contract and query', async ({ page }) => {
  await page.goto('http://localhost:5173')  // Vite dev; API via proxy → PHP :9080
  
  // Login
  await page.fill('[name="email"]', 'test@example.com')
  await page.fill('[name="password"]', 'password123')
  await page.click('button:has-text("Login")')
  
  // Upload contract
  await page.setInputFiles('input[type="file"]', 'fixtures/nda.pdf')
  await page.click('button:has-text("Upload")')
  await expect(page.locator('.upload-success')).toBeVisible({ timeout: 30000 })
  
  // Query
  await page.fill('[name="query"]', 'Qual è la penale?')
  await page.click('button:has-text("Ask")')
  await expect(page.locator('.answer')).toContainText(/penale/i, { timeout: 10000 })
  
  // Verify citation
  await expect(page.locator('.citation')).toBeVisible()
})
```

**Requisito**: Stack completo up (`make up`)  
**Comando**: `npm run test:e2e` (configurato per `playwright test`)

---

## 📊 Coverage Requirements

| Layer | Minimo | Ideale | Tool | Report |
|---|---|---|---|---|
| Rust | 80% | 90% | `tarpaulin` | `cargo tarpaulin --out Html --output-dir coverage/` |
| Python | 80% | 90% | `pytest-cov` | `pytest --cov --cov-report=html` |
| PHP | 80% | 90% | PHPUnit + Xdebug | `composer test -- --coverage-html coverage/` |
| Frontend | 70% | 85% | Vitest | `vitest run --coverage` |

**Regola**: Se coverage < minimo → la fase NON è chiusa, non si committa, non si apre PR.

---

## 🚀 Benchmark & Performance Test

**Scopo**: Validare KPI performance (latency, throughput, accuracy)

### Suite Benchmark (`benchmarks/`)

#### 1. Ingest Benchmark (`benchmarks/ingest_bench.sh`)

```bash
#!/bin/bash
# Test: 100 pagine PDF → misura tempo totale, pagine/minuto

time cargo run --release --bin ingest_bench -- \
  --file fixtures/contract_100_pages.pdf \
  --kb-id bench_kb

# Target: > 100 pagine/minuto
```

#### 2. Query RAG Benchmark (`benchmarks/query_bench.sh`)

```bash
# Test: 100 query su kb con 1000 chunks → misura p50, p95, p99

k6 run benchmarks/k6/query_load.js

# Target: p95 < 500ms (modello locale)
```

#### 3. Hallucination Test (`benchmarks/hallucination_eval.py`)

```python
# Test: 50 domande con ground truth → conta risposte inventate

python benchmarks/hallucination_eval.py \
  --dataset fixtures/legal_qa_50.jsonl \
  --kb-id eval_kb

# Target: hallucination rate < 1%
```

#### 4. Concurrent Multi-Contract (`benchmarks/concurrent_bench.sh`)

```bash
# Test: 50 contratti analizzati in parallelo → misura tempo totale

python benchmarks/concurrent_comparison.py \
  --contracts fixtures/contracts/*.pdf \
  --query "Confronta le penali"

# Target: < 2 secondi per 50 contratti
```

**Comando unico**: `make bench` (esegue tutti i 4 benchmark e genera report JSON)

---

## 🛡️ Security Test

**Scopo**: Verificare assenza vulnerabilità OWASP Top 10

### Dependency Audit

```bash
# Rust
cargo audit --deny warnings

# Python
pip-audit --strict

# PHP
composer audit --no-dev

# Frontend
npm audit --audit-level=high
```

**Regola**: Zero vulnerabilità High/Critical → blocco commit

---

### Container Security Scan

```bash
trivy image --severity HIGH,CRITICAL \
  archivio-parlante-rust-engine:latest

trivy image --severity HIGH,CRITICAL \
  archivio-parlante-python-worker:latest
```

**Regola**: Zero vulnerabilità High/Critical → blocco deploy

---

### Input Fuzzing (opzionale, Fase 5)

```bash
# Fuzz Rust endpoint /ingest con input malformati
cargo fuzz run fuzz_ingest_endpoint -- -max_total_time=300
```

---

## ✅ Checklist Testing per Ogni Fase

Prima di chiudere una fase (Step 3 del ciclo 8-step §0.8):

### Unit Test
- [ ] Rust: `cargo test --release` passa al 100%
- [ ] Python: `pytest` passa al 100%
- [ ] PHP: `composer test` passa al 100%
- [ ] Frontend: `vitest run` passa al 100%
- [ ] Coverage verificata: Rust/Python/PHP ≥ 80%, Frontend ≥ 70%

### Integration Test
- [ ] Test Rust → Qdrant passa
- [ ] Test Rust → Python worker passa
- [ ] Test Rust → Ollama passa
- [ ] Test PHP → Rust engine passa
- [ ] Test Frontend → PHP gateway passa

### E2E Test (se UI è implementata)
- [ ] Playwright: scenario upload + query passa
- [ ] Playwright: scenario multi-contract comparison passa
- [ ] Playwright: scenario login + admin panel passa

### Performance Test (se applicabile)
- [ ] Benchmark ingest: > 100 pag/min
- [ ] Benchmark query: p95 < 500ms
- [ ] Hallucination test: < 1%
- [ ] Concurrent: 50 contratti < 2s

### Security Test
- [ ] `cargo audit` — zero High/Critical
- [ ] `pip-audit` — zero High/Critical
- [ ] `composer audit` — zero High/Critical
- [ ] `npm audit` — zero High/Critical
- [ ] `trivy image` — zero High/Critical

---

## 🔄 Regression Test Obbligatorio

**Dopo ogni modifica (step 4, 5, 6 del ciclo)**:

1. **Step 4 — Dopo ottimizzazioni performance**: rilanciare `make test-all` → 100% pass
2. **Step 5 — Dopo linting/cleanup**: rilanciare `make test-all` → 100% pass
3. **Step 6 — Dopo security fix**: rilanciare `make test-all` → 100% pass

**Se anche un solo test fallisce dopo una modifica**: rollback o fix immediato prima di procedere.

---

## 📋 Test Fixture & Dati di Test

**Directory**: `fixtures/` (nella root di ogni layer)

### Rust
- `fixtures/contract_sample.pdf` (5 pagine, NDA standard)
- `fixtures/nda_sample.docx` (Word con tabelle)
- `fixtures/scanned_contract.pdf` (OCR test)

### Python
- `fixtures/sample.pdf`
- `fixtures/complex_table.pdf`
- `fixtures/italian_contract.pdf`

### E2E
- `tests/e2e/fixtures/nda.pdf`
- `tests/e2e/fixtures/contract_100_pages.pdf`

**Regola**: Mai usare contratti reali o dati sensibili nei test. Solo documenti sintetici o pubblici.

---

## 🎯 KPI di Qualità (Target Fase 5)

| Metrica | Target | Misurazione |
|---|---|---|
| **Test success rate** | 100% | CI pipeline |
| **Code coverage** | >80% (Rust/Python/PHP), >70% (Frontend) | Coverage report |
| **Linting warnings** | 0 | `clippy -D warnings`, `ruff`, `phpstan L8` |
| **Vulnerabilità dipendenze** | 0 High/Critical | `audit` commands |
| **Ingest throughput** | >100 pag/min | Benchmark |
| **Query latency p95** | <500ms locale | k6 |
| **Hallucination rate** | <1% | Eval dataset |
| **Concurrent capacity** | 50 contratti <2s | Load test |

---

## 🚨 Gate di Blocco

**Una fase NON può essere chiusa se**:

1. Anche un solo test fallisce
2. Coverage sotto la soglia minima
3. `clippy` / `ruff` / `phpstan` / `eslint` ha warning
4. `audit` trova vulnerabilità High/Critical
5. Benchmark sotto i target (Fase 5+)

**Azione**: Fix → Re-test → Verifica 100% pass → Poi procedi al prossimo step.

---

**Ultimo aggiornamento**: 2025-04-21 — Fase -1
