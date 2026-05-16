# Integration Test Scripts

Helper scripts for Fase 6 integration testing and benchmarking.

## Available Scripts

### 1. `run_integration_tests.sh`

**Purpose**: Orchestrates full integration test suite for Fase 6.

**Usage**:
```bash
# Run all tests (including WebSocket load test)
./scripts/run_integration_tests.sh

# Skip WebSocket test (if k6 not installed)
./scripts/run_integration_tests.sh --skip-websocket
```

**What it does**:
1. ✅ Health checks (Rust Engine, PHP Gateway, Python Worker)
2. ✅ Creates test KB and ingests sample contracts
3. ✅ Runs Graph RAG benchmark (recall improvement, latency)
4. ✅ Runs Hallucination detection evaluation (trick questions, valid queries)
5. ✅ Runs WebSocket load test with k6 (optional)
6. ✅ Generates summary report

**Output**:
- JSON results in `benchmarks/results/integration_YYYYMMDD_HHMMSS/`
- Markdown summary: `SUMMARY.md`
- Exit code 0 if all tests pass, 1 if any fail

**Requirements**:
- All services running: `docker compose up -d`
- Python 3.11+ with `httpx`, `rich` installed
- k6 installed (optional, for WebSocket test): https://k6.io/docs/getting-started/installation/
- Python worker running (for hallucination tests): `cd engine-python && uvicorn app.main:app --port 8091`

**Environment Variables**:
```bash
ENGINE_URL=http://localhost:8090        # Rust Engine
PHP_GATEWAY_URL=http://localhost:9080   # PHP Gateway
PYTHON_WORKER_URL=http://localhost:8091 # Python Worker
```

---

### 2. `ingest_test_fixtures.py`

**Purpose**: Creates test KB and ingests sample contracts for testing.

**Usage**:
```bash
# Default (KB ID: fase6_test_kb)
python scripts/ingest_test_fixtures.py

# Custom KB ID
python scripts/ingest_test_fixtures.py --kb-id my_test_kb

# Custom engine URL
python scripts/ingest_test_fixtures.py --engine-url http://staging:8090
```

**Arguments**:
- `--engine-url`: Rust Engine URL (default: `http://localhost:8090`)
- `--kb-id`: Knowledge Base ID to create (default: `fase6_test_kb`)
- `--contracts-dir`: Directory with contract files (default: `tests/fixtures/contracts`)

**What it does**:
1. Checks engine health
2. Reads all `.txt` files from `tests/fixtures/contracts/`
3. Ingests each contract via `POST /ingest` endpoint
4. Reports success/failure for each file

**Output**:
```
Ingest Test Fixtures
Engine URL: http://localhost:8090
KB ID: fase6_test_kb
Contracts: 2 files

✓ Engine is healthy

Ingesting sample_contract_acme.txt...
✓ sample_contract_acme.txt ingested successfully
Ingesting sample_contract_gamma.txt...
✓ sample_contract_gamma.txt ingested successfully

✅ All 2 contracts ingested successfully!

KB ID 'fase6_test_kb' is ready for testing
```

**Exit codes**:
- `0`: All contracts ingested successfully
- `1`: One or more contracts failed to ingest

---

## Quick Start Guide

### Full Integration Test (Recommended)

```bash
# 1. Start all services
docker compose up -d

# 2. Start Python worker manually (if not containerized)
cd engine-python
uvicorn app.main:app --host 0.0.0.0 --port 8091 &
cd ..

# 3. Wait for services to be ready
sleep 30

# 4. Run full test suite
./scripts/run_integration_tests.sh

# 5. Check results
cat benchmarks/results/integration_*/SUMMARY.md
```

### Manual Testing (Individual Benchmarks)

```bash
# 1. Start services
docker compose up -d

# 2. Ingest test fixtures
python scripts/ingest_test_fixtures.py --kb-id fase6_test_kb

# 3. Run specific benchmark
cd benchmarks

# Graph RAG
python graph_rag_bench.py --kb-id fase6_test_kb --output results/graph_rag.json

# Hallucination detection
python hallucination_eval.py --kb-id fase6_test_kb --output results/hallucination.json

# WebSocket load (requires k6)
k6 run k6/websocket_load.js
```

---

## Test Fixtures

All test data is in `tests/fixtures/`:

- **Contracts**: `tests/fixtures/contracts/`
  - `sample_contract_acme.txt` - Acme/Beta software supply (€50K, €10K penalty)
  - `sample_contract_gamma.txt` - Gamma/Delta partnership (€120K min, multiple penalties)

- **Test Queries**: `tests/fixtures/queries/graph_rag_test_queries.json`
  - 6 queries testing multi-hop graph traversal
  - Expected entities, relations, answers defined

- **Trick Questions**: `tests/fixtures/trick_questions/hallucination_test_questions.json`
  - 30 questions testing hallucination detection
  - Categories: out-of-domain, edge cases, negatives, ambiguous, stress tests, precision tests

See `tests/fixtures/README.md` for detailed documentation.

---

## Target KPIs

### Graph RAG Benchmark
- ✅ Recall@10 improvement: **≥5%** vs pure hybrid
- ✅ Latency P95: **<200ms**
- ✅ Failure rate: **0%**

### Hallucination Detection
- ✅ Hallucination rate on trick questions: **≤1%**
- ✅ Precision on flagging: **≥85%**
- ✅ Specificity (no false positives): **>95%**
- ✅ Latency P95: **≤300ms**

### WebSocket Collaboration
- ✅ Concurrent connections: **100 stable**
- ✅ Message latency P95: **<500ms**
- ✅ Message loss: **0%**

---

## Troubleshooting

### Python Worker Not Responding

**Problem**: Hallucination tests fail with "Python Worker not responding"

**Solution**:
```bash
cd engine-python
pip install -r requirements.txt
uvicorn app.main:app --host 0.0.0.0 --port 8091
```

Check: `curl http://localhost:8091/health`

### Graph RAG Returns No Results

**Problem**: Graph retrieval doesn't improve recall

**Cause**: Graph not built yet (no entities extracted during ingestion)

**Solution**:
1. Check Python worker is running (needed for entity extraction)
2. Re-ingest contracts: `python scripts/ingest_test_fixtures.py --kb-id new_kb_id`
3. Verify entities in MySQL: `SELECT COUNT(*) FROM ap_graph_nodes WHERE kb_id = 'your_kb_id';`

### k6 Command Not Found

**Problem**: WebSocket load test fails with "k6: command not found"

**Solution**:
```bash
# macOS
brew install k6

# Ubuntu/Debian
sudo gpg -k
sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
sudo apt-get update
sudo apt-get install k6

# Windows
choco install k6
```

Or skip WebSocket test: `./scripts/run_integration_tests.sh --skip-websocket`

### Services Not Healthy

**Problem**: Health check fails for one or more services

**Solution**:
```bash
# Check service logs
docker compose logs rust-engine
docker compose logs php-gateway
docker compose logs mysql

# Restart services
docker compose restart

# Full rebuild if needed
docker compose down -v
docker compose build
docker compose up -d
```

### Permission Denied on Scripts

**Problem**: `./scripts/run_integration_tests.sh: Permission denied`

**Solution**:
```bash
chmod +x scripts/run_integration_tests.sh
chmod +x scripts/ingest_test_fixtures.py
```

---

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Fase 6 Integration Tests

on:
  push:
    branches: [develop, main]
  pull_request:
    branches: [develop]

jobs:
  integration:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Start services
        run: docker compose up -d

      - name: Wait for services
        run: sleep 30

      - name: Start Python worker
        run: |
          cd engine-python
          pip install -r requirements.txt
          uvicorn app.main:app --port 8091 &

      - name: Run integration tests
        run: ./scripts/run_integration_tests.sh --skip-websocket

      - name: Upload results
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: integration-test-results
          path: benchmarks/results/integration_*/
```

---

## License

MIT - Part of Archivio Parlante project

**Generated**: 2026-05-08  
**Fase**: 6 Integration Testing  
**Author**: Claude Sonnet 4.5
