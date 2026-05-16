# 🧪 Integration Testing Complete System - Checklist

**Data**: 2026-05-17  
**Stato**: Ready for execution  
**Branch**: `develop`  
**Commit**: `148e4eb`

---

## 📋 Pre-requisiti

### 1. Environment Setup
```bash
# Verificare che tutti i servizi siano disponibili
docker-compose ps

# Servizi richiesti:
# - mysql (porta 3306)
# - redis (porta 6379)
# - qdrant (porta 6333)
# - ollama (porta 11434)
```

### 2. Dependencies Installation
```bash
# Rust
cd engine-rust
cargo build --release

# Python
cd engine-python
pip install -r requirements.txt
python -m spacy download it_core_news_lg

# PHP
cd php-gateway
composer install

# Frontend
cd frontend
npm install
```

### 3. Database Migrations
```bash
# MySQL migrations devono essere eseguite
# Verificare che tutte le tabelle esistano:
# - ap_users
# - ap_workspaces
# - ap_workspace_members
# - ap_knowledge_bases
# - ap_kb_permissions
# - ap_documents
# - ap_chat_messages
# - ap_graph_nodes
# - ap_graph_edges
# - ap_annotations
```

---

## 🧪 Test Suite Execution

### 1. Unit Tests

#### Rust
```bash
cd engine-rust
cargo test --lib
```

**Expected**: Tutti i test devono passare (0 failures)

**Coverage target**: ≥ 80%

#### Python
```bash
cd engine-python
pytest --cov=app --cov-report=term-missing
```

**Expected**: Tutti i test devono passare

**Coverage target**: ≥ 80%

#### PHP
```bash
cd php-gateway
composer test
```

**Expected**: Tutti i test PHPUnit devono passare

**Coverage target**: ≥ 80%

#### Frontend
```bash
cd frontend
npm run test
```

**Expected**: Tutti i test Vitest devono passare

**Coverage target**: ≥ 70%

### 2. Integration Tests

#### Test KB Access Control (74 tests)
```bash
cd engine-rust
cargo test test_kb_access_control_complete -- --test-threads=1
```

**Expected**: 74/74 test passati

**Categories**:
- ✅ Direct Permission Tests (13)
- ✅ Workspace Permission Tests (24)
- ✅ Ownership Tests (11)
- ✅ Hierarchy Tests (11)
- ✅ Edge Cases (15)

#### Test Knowledge Graph Extraction
```bash
cd engine-python
pytest tests/test_graph_extractor.py -v
```

**Expected**: Tutti i test KG passati

**Verify**:
- ✅ spaCy NER extraction
- ✅ Legal entity recognition (PARTIES, DATES, AMOUNTS, CLAUSES, etc.)
- ✅ Relationship extraction
- ✅ MySQL storage (ap_graph_nodes, ap_graph_edges)

#### Test Sparse Vectors
```bash
cd engine-rust
cargo test sparse_vectors
```

**Expected**: Test sparse vector generation e hybrid search passati

**Verify**:
- ✅ Named vectors in Qdrant (dense + sparse)
- ✅ BM25/SPLADE generation
- ✅ Hybrid search query

### 3. End-to-End Tests

#### Playwright E2E
```bash
cd frontend
npm run test:e2e
```

**Expected**: Tutti gli scenari E2E passati

**Critical flows**:
- ✅ User login/logout
- ✅ Workspace creation
- ✅ Workspace settings (add/remove members, change roles)
- ✅ KB creation
- ✅ Document upload and ingestion
- ✅ Chat with RAG
- ✅ Multi-contract comparison
- ✅ Real-time annotations (WebSocket)

### 4. Performance Benchmarks

#### Ingest Performance
```bash
cd benchmarks
./run_ingest_bench.sh
```

**Target KPIs**:
- PDF parsing: < 2s per page
- Chunking: < 500ms per document
- Embedding: < 100ms per chunk (batch)
- Qdrant insert: < 50ms per chunk
- Total ingest: < 10s per 5-page document

#### Query Performance
```bash
cd benchmarks
./run_query_bench.sh
```

**Target KPIs**:
- Hybrid search: < 200ms (p95)
- Reranking: < 300ms (p95)
- LLM generation: < 2s (p95)
- Total query latency: < 3s (p95)

#### Concurrent Load
```bash
cd benchmarks
k6 run load_test.js
```

**Target**:
- 100 concurrent users
- 1000 req/min sustained
- Error rate < 0.1%
- p95 latency < 5s

### 5. Security Audit

#### OWASP ASVS L2 Compliance
```bash
# Dependency vulnerabilities
cd engine-rust && cargo audit
cd engine-python && pip-audit
cd php-gateway && composer audit
cd frontend && npm audit

# Container vulnerabilities
trivy image archivio-parlante/rust-engine:latest
trivy image archivio-parlante/python-worker:latest
trivy image archivio-parlante/php-gateway:latest
```

**Expected**: Zero HIGH or CRITICAL vulnerabilities

#### Manual Security Checks
- ✅ SQL Injection protection (prepared statements)
- ✅ XSS protection (CSP headers, input sanitization)
- ✅ CSRF protection (tokens)
- ✅ Authentication (JWT, httpOnly cookies)
- ✅ Authorization (KB access control)
- ✅ Rate limiting (Redis-based)
- ✅ Input validation (zod, pydantic, etc.)

---

## 🔄 Integration Points Testing

### 1. Rust ↔ Python Communication
**Test**: Call `/extract_kg` endpoint from Rust ingest

```bash
# Start Python worker
cd engine-python && uvicorn app.main:app --port 8091

# Run ingest test
cd engine-rust && cargo test test_ingest_with_kg_extraction
```

**Verify**:
- ✅ HTTP request succeeds
- ✅ JSON parsing works
- ✅ Nodes/edges stored in MySQL

### 2. PHP ↔ Rust Communication
**Test**: Proxy request from PHP to Rust

```bash
# Start Rust engine
cd engine-rust && cargo run --release

# Test PHP proxy
curl -X POST http://localhost:8080/api/query \
  -H "Authorization: Bearer $JWT" \
  -d '{"kb_id": "test", "query": "test"}'
```

**Verify**:
- ✅ JWT validation
- ✅ Request forwarding
- ✅ Response handling
- ✅ Error propagation

### 3. Frontend ↔ Backend Communication
**Test**: Full user flow through UI

1. Login → JWT cookie set
2. Create workspace → API call succeeds
3. Upload document → Ingest triggered
4. Query KB → RAG response received
5. Real-time annotations → WebSocket connected

**Verify**:
- ✅ API responses correct
- ✅ Loading states work
- ✅ Error handling graceful
- ✅ Optimistic updates

### 4. Redis Rate Limiting
**Test**: Exceed rate limit

```bash
# Send 100 requests rapidly
for i in {1..100}; do
  curl http://localhost:8090/health &
done
wait
```

**Verify**:
- ✅ Rate limit triggered after threshold
- ✅ 429 status code returned
- ✅ Retry-After header present

### 5. Qdrant Hybrid Search
**Test**: Query with both dense and sparse vectors

```bash
cd engine-rust
cargo test test_hybrid_search
```

**Verify**:
- ✅ Named vectors used (`dense`, `sparse`)
- ✅ Reciprocal Rank Fusion applied
- ✅ Recall improvement vs dense-only

### 6. WebSocket Real-time Sync
**Test**: Two users annotating same document

```bash
# User 1: Connect WebSocket
wscat -c "ws://localhost:8090/ws/annotations?kb_id=test&doc_id=test&user_id=1&user_name=User1"

# User 2: Connect WebSocket
wscat -c "ws://localhost:8090/ws/annotations?kb_id=test&doc_id=test&user_id=2&user_name=User2"

# User 1: Create annotation
{"type": "annotation.create", "chunk_id": "chunk1", "text": "Test", "position": {"start": 0, "end": 10}}

# Verify User 2 receives annotation.created event
```

**Verify**:
- ✅ KB access control enforced
- ✅ Presence tracking works
- ✅ Annotations broadcast to all users
- ✅ Concurrent edits handled

---

## 📊 Success Criteria

### Must Pass (Blockers)
- [ ] All unit tests pass (Rust, Python, PHP, Frontend)
- [ ] All 74 KB access control tests pass
- [ ] E2E critical flows pass (login, workspace, KB, query)
- [ ] Zero HIGH/CRITICAL security vulnerabilities
- [ ] No SQL injection, XSS, or CSRF vulnerabilities

### Should Pass (Non-blockers)
- [ ] Performance benchmarks meet targets
- [ ] Code coverage ≥ 80% (Rust, Python, PHP), ≥ 70% (Frontend)
- [ ] Load test sustains 100 concurrent users
- [ ] WebSocket real-time sync works

### Nice to Have
- [ ] Advanced hallucination detection tests
- [ ] Multi-language support tests
- [ ] Backup/restore procedures tested

---

## 🚀 Deployment Checklist

After all tests pass:

1. **Merge to main**
```bash
git checkout main
git merge develop
git push origin main
```

2. **Tag release**
```bash
git tag -a v1.0.0 -m "Release v1.0.0 - Complete production system"
git push origin v1.0.0
```

3. **Deploy to production**
```bash
# Build production images
docker-compose -f docker-compose.prod.yml build

# Deploy
docker-compose -f docker-compose.prod.yml up -d

# Verify health
curl http://production-domain/health
```

4. **Monitor**
- Check Grafana dashboards
- Monitor error rates
- Verify logs are clean
- Test critical flows in production

---

## 📝 Notes

- Cargo non è installato nel PATH corrente → **Installare Rust toolchain**
- Database test richiede `DATABASE_URL` env var
- Ollama richiede modelli scaricati: `qwen2.5:7b`, `nomic-embed-text`
- Python worker richiede spaCy model: `it_core_news_lg`

---

## ✅ Completion Status

**Date completed**: _________________  
**Completed by**: _________________  
**Test results**: _________________  
**Notes**: _________________
