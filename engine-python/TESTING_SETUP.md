# Python Worker - Testing Setup

## Status (2026-05-26)

**Current**: Tests not executable locally due to missing venv setup in WSL2/container.

**Test files** (5):
- `tests/test_contextualize.py` - Contextual retrieval
- `tests/test_extract_kg.py` - Knowledge graph extraction
- `tests/test_parse.py` - HTTP parsing / FastAPI routes
- `tests/test_pdf_parser.py` - PDF parsing + OCR
- `tests/test_rerank.py` - BGE reranker (ML model)

**Previous run (pre-Fase 3)**: 21 pass / 23 fail (per ANALISI_PROGETTO_2026-05-25.md)

---

## Setup Required (WSL2)

### 1. Create venv in WSL2

```bash
wsl -e bash -c "cd /mnt/c/Users/aj_93/OneDrive/Documenti/GitHub/Archivio-parlanteX/engine-python && python3 -m venv venv"
```

### 2. Install dependencies

```bash
wsl -e bash -c "cd /mnt/c/Users/aj_93/OneDrive/Documenti/GitHub/Archivio-parlanteX/engine-python && source venv/bin/activate && pip install --upgrade pip && pip install -r requirements.txt && pip install pytest pytest-cov pytest-asyncio"
```

### 3. Run tests with markers

```bash
wsl -e bash -c "cd /mnt/c/Users/aj_93/OneDrive/Documenti/GitHub/Archivio-parlanteX/engine-python && source venv/bin/activate && pytest tests/ -m 'not integration' -v"
```

---

## Integration vs Unit Markers

### Classification (needs implementation)

| Test File | Type Probable | Marker to Add | Reason |
|---|---|---|---|
| `test_rerank.py` | **Integration ML** | `@pytest.mark.integration` | Requires BGE model download (~500MB) |
| `test_pdf_parser.py` | **Integration file/GPU** | `@pytest.mark.integration` | Requires PDF libs, possibly OCR (Tesseract) |
| `test_parse.py` | **HTTP / App** | Unit (mock FastAPI `TestClient`) | Can mock without real server |
| `test_contextualize.py` | **LLM** | Integration if calls Ollama, Unit if mocked | Check if mocked |
| `test_extract_kg.py` | **LLM / Neo4j** | Integration if calls services | Check dependencies |

### Add markers to test files

Example for `test_rerank.py`:

```python
import pytest

@pytest.mark.integration
def test_reranker_loads_model():
    # Requires model download
    ...

@pytest.mark.integration  
def test_rerank_top_results():
    # Requires model inference
    ...
```

### Update `pytest.ini` or `conftest.py`

```ini
[pytest]
markers =
    integration: marks tests as integration (slow, require services/models)
```

---

## DoD Fase 3 (Deferred)

- [ ] D3.1: `pytest -m "not integration"` → 100% pass (requires venv + marker implementation)
- [ ] D3.2: `requirements.txt` installabile su Python 3.10+ ✅ (confermato)
- [ ] D3.3: Worker risponde su **8091** ✅ (container up, verificato)
- [ ] D3.4: Marker integration documentati ✅ (questo file)

**Estimated effort**: 2-3 hours (venv setup + marker implementation + test fixes)

**Defer to**: CI Linux (Fase 7) or dedicated Python testing session.

---

## CI Strategy

In `.github/workflows/ci.yml`:

```yaml
jobs:
  python-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: '3.11'
      - name: Install dependencies
        run: |
          cd engine-python
          pip install -r requirements.txt
          pip install pytest pytest-cov pytest-asyncio
      - name: Run unit tests only
        run: |
          cd engine-python
          pytest tests/ -m "not integration" --cov=app --cov-report=term
```

Integration tests: nightly job with model caching.

---

## Alternative: Docker exec with mounted tests

If tests were copied to container:

```bash
docker cp engine-python/tests archivio-python-worker:/app/tests
docker exec archivio-python-worker pytest /app/tests/ -v
```

But requires container rebuild or manual copy each run.
