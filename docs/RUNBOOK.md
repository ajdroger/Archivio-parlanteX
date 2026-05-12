# 🔧 Archivio Parlante — Operational Runbook

**Version**: 1.1  
**Last Updated**: 2026-05-08  
**Target Audience**: DevOps, SRE, System Administrators

---

## Table of Contents

1. [System Architecture Overview](#system-architecture-overview)
2. [Starting & Stopping Services](#starting--stopping-services)
3. [Health Checks](#health-checks)
4. [Scaling](#scaling)
5. [Backup & Restore](#backup--restore)
6. [Model Management](#model-management)
7. [Secret Management](#secret-management)
8. [Troubleshooting](#troubleshooting)
9. [Monitoring](#monitoring)
10. [Disaster Recovery](#disaster-recovery)

---

## System Architecture Overview

Archivio Parlante consists of **7 microservices** (6 Docker + 1 native Python):

| Service | Type | Port | Critical | Dependencies |
|---|---|---|---|---|
| **php-gateway** | Docker | 9080 | ✅ Yes | mysql, redis, rust-engine |
| **rust-engine** | Docker | 8090 | ✅ Yes | qdrant, ollama |
| **python-worker** | **Native** | 8091 | ✅ Yes | ollama (optional) |
| **qdrant** | Docker | 6335 | ✅ Yes | - |
| **ollama** | Docker | 11434 | ⚠️ Optional | - |
| **mysql** | Docker | 3307 | ✅ Yes | - |
| **redis** | Docker | 6380 | ✅ Yes | - |

**Note**: Python Worker runs **natively on Windows** (not in Docker) due to ML dependency issues.

---

## Starting & Stopping Services

### Start All Services

```bash
# Start Docker services (6/7)
make up

# Start Python Worker (native, in separate terminal)
cd engine-python
.\venv\Scripts\Activate.ps1
uvicorn app.main:app --host 0.0.0.0 --port 8091 --reload
```

### Stop All Services

```bash
# Stop Docker services
make down

# Stop Python Worker
# Press Ctrl+C in the Python worker terminal
```

### Graceful Shutdown

```bash
# 1. Stop accepting new requests (update health check to fail)
curl -X POST http://localhost:8090/admin/drain

# 2. Wait for in-flight requests to complete (30s grace period)
sleep 30

# 3. Stop services
make down
```

### Restart Single Service

```bash
# Rebuild and restart Rust Engine
make rebuild-rust

# Rebuild and restart Python Worker
# (Native: just restart the uvicorn process)

# Restart PHP Gateway
docker-compose restart php-gateway
```

---

## Health Checks

### Verify All Services

```bash
make health
```

Expected output:
```
✓ PHP Gateway: http://localhost:9080 (Apache)
✓ Rust Engine: {"status":"ok","service":"rust-engine"}
✓ Python Worker: {"status":"ok","service":"python-worker"}
✓ Qdrant: {"title":"qdrant","version":"1.12.0"}
✓ Ollama: Ollama is running
✓ MySQL: Connected
✓ Redis: PONG
```

### Individual Health Checks

```bash
# PHP Gateway (Apache)
curl http://localhost:9080

# Rust Engine
curl http://localhost:8090/health

# Python Worker
curl http://localhost:8091/health

# Qdrant
curl http://localhost:6335

# Ollama
curl http://localhost:11434/api/tags

# MySQL
docker exec archivio-mysql mysql -uroot -pdevpass123 -e "SELECT 1"

# Redis
docker exec archivio-redis redis-cli ping
```

---

## Scaling

### Horizontal Scaling (Multiple Replicas)

#### Scale Rust Engine (stateless)

```yaml
# docker-compose.override.yml
services:
  rust-engine:
    deploy:
      replicas: 3
```

Add load balancer (nginx) in front:
```nginx
upstream rust_backend {
    server localhost:8090;
    server localhost:8091;
    server localhost:8092;
}
```

#### Scale Python Worker (native)

Run multiple instances on different ports:
```bash
# Terminal 1
uvicorn app.main:app --port 8091

# Terminal 2  
uvicorn app.main:app --port 8092

# Terminal 3
uvicorn app.main:app --port 8093
```

Update Rust Engine to round-robin between workers.

### Vertical Scaling

#### Increase Ollama VRAM

```yaml
# docker-compose.yml
ollama:
  deploy:
    resources:
      reservations:
        devices:
          - driver: nvidia
            count: 1
            capabilities: [gpu]
      limits:
        memory: 16G  # Increase if needed
```

#### Increase Qdrant Memory

```yaml
qdrant:
  environment:
    - QDRANT__STORAGE__PERFORMANCE__MAX_SEARCH_THREADS=8
  deploy:
    resources:
      limits:
        memory: 8G
```

---

## Backup & Restore

### MySQL Backup

```bash
# Full backup
make backup-db
# Output: backups/db_YYYYMMDD_HHMM.sql.gz

# Manual backup
docker exec archivio-mysql mysqldump -uroot -pdevpass123 \
  archivio_parlante_x | gzip > backup.sql.gz
```

### MySQL Restore

```bash
# From compressed backup
gunzip < backups/db_20260506_1200.sql.gz | \
  docker exec -i archivio-mysql mysql -uroot -pdevpass123 archivio_parlante_x

# Or use make command
make restore-db FILE=backups/db_20260506_1200.sql.gz
```

### Qdrant Snapshot

```bash
# Create snapshot
curl -X POST "http://localhost:6335/collections/contracts_2024/snapshots"

# List snapshots
curl "http://localhost:6335/collections/contracts_2024/snapshots"

# Download snapshot
curl "http://localhost:6335/collections/contracts_2024/snapshots/snapshot-2024-05-06.snapshot" \
  -o qdrant_backup.snapshot

# Restore snapshot
curl -X PUT "http://localhost:6335/collections/contracts_2024/snapshots/upload" \
  --data-binary @qdrant_backup.snapshot
```

### Backup Schedule

**Recommended**:
- MySQL: Daily at 2 AM (automated via cron)
- Qdrant: Weekly full snapshot
- Shared uploads: Sync to S3/backup storage daily

---

## Model Management

### Ollama Model Update

```bash
# 1. Pull new model version
docker exec archivio-ollama ollama pull qwen2.5:7b-instruct-q4_K_M

# 2. Test new model
curl http://localhost:11434/api/generate -d '{
  "model": "qwen2.5:7b-instruct-q4_K_M",
  "prompt": "Test query"
}'

# 3. Update .env
OLLAMA_MODEL_CHAT=qwen2.5:7b-instruct-q4_K_M

# 4. Restart Rust Engine
docker-compose restart rust-engine

# 5. (Optional) Reindex if embeddings changed
# Trigger reindex via admin UI or API
```

### List Available Models

```bash
docker exec archivio-ollama ollama list
```

### Remove Old Models (free space)

```bash
docker exec archivio-ollama ollama rm qwen2.5:3b-instruct-q4_K_M
```

---

## Secret Management

### Generate Secrets

```bash
# JWT Secret (32 bytes hex)
openssl rand -hex 32

# Rust Engine Internal Token (64 bytes hex)
openssl rand -hex 64
```

### Update Secrets

1. **Never commit secrets to git!**
2. Update `.env` file:
   ```env
   JWT_SECRET=<new_32_byte_hex>
   RUST_ENGINE_INTERNAL_TOKEN=<new_64_byte_hex>
   ```
3. Restart services:
   ```bash
   make down && make up
   ```

### Production Secret Management

Use **external secret manager**:
- AWS Secrets Manager
- Azure Key Vault
- HashiCorp Vault
- Kubernetes Secrets

**Never** store production secrets in `.env` files checked into git.

---

## Troubleshooting

### Common Issues

#### 1. Ollama Out of Memory (OOM)

**Symptoms**: Container crashes, `docker logs archivio-ollama` shows OOM killer.

**Solutions**:
- Use smaller model: `qwen2.5:3b` instead of `7b`
- Increase Docker memory limit (Settings → Resources)
- Enable CPU offload for large models:
  ```env
  OLLAMA_MODEL_CHAT_HEAVY=qwen2.5:14b-instruct-q4_K_M  # Uses CPU fallback
  ```

#### 2. Qdrant Disk Full

**Symptoms**: `curl http://localhost:6335` returns 507 Insufficient Storage.

**Solutions**:
```bash
# Check disk usage
docker exec archivio-qdrant du -sh /qdrant/storage

# Delete old collections
curl -X DELETE "http://localhost:6335/collections/old_kb_2023"

# Optimize collection (reclaim deleted space)
curl -X POST "http://localhost:6335/collections/contracts_2024/optimize"
```

#### 3. Rust Engine Deadlock

**Symptoms**: Requests hang, no response from `:8090/health`.

**Diagnosis**:
```bash
# Check logs for panics or deadlocks
docker logs archivio-rust-engine --tail 100

# Check CPU usage (should be near 100% if busy, 0% if deadlocked)
docker stats archivio-rust-engine
```

**Solutions**:
- Restart container: `docker-compose restart rust-engine`
- If persists, check for infinite loops in code

#### 4. Python Worker Not Reachable from Docker

**Symptoms**: Rust Engine logs show "Connection refused http://host.docker.internal:8091".

**Solutions**:
- Verify Python Worker is running: `curl http://localhost:8091/health`
- Check Windows Firewall allows port 8091
- Verify `host.docker.internal` resolves:
  ```bash
  docker exec archivio-rust-engine ping host.docker.internal
  ```

#### 5. MySQL Connection Refused

**Symptoms**: PHP/Rust logs show "Connection refused mysql:3306".

**Solutions**:
```bash
# Check MySQL is UP
docker ps | grep mysql

# Check logs
docker logs archivio-mysql

# Verify credentials
docker exec archivio-mysql mysql -uroot -pdevpass123 -e "SELECT 1"

# Reset if corrupted
docker-compose down
docker volume rm archivio-parlantex_mysql_data
docker-compose up -d mysql
make migrate
```

#### 6. Graph RAG Retrieval Returns No Results (Fase 6.1)

**Symptoms**: Query with `"retrieval_mode": "graph"` returns empty or same results as hybrid.

**Diagnosis**:
```bash
# Check if knowledge graph exists
docker exec archivio-mysql mysql -uroot -pdevpass123 archivio_parlante_x \
  -e "SELECT COUNT(*) FROM ap_graph_nodes WHERE kb_id='contracts_2024';"

# Check if graph edges exist
docker exec archivio-mysql mysql -uroot -pdevpass123 archivio_parlante_x \
  -e "SELECT COUNT(*) FROM ap_graph_edges WHERE kb_id='contracts_2024';"
```

**Solutions**:
- Graph not built yet: Wait for document ingestion to complete (check Python Worker logs)
- LLM relation extraction failed: Check Ollama `qwen2.5:3b` model loaded
  ```bash
  docker exec archivio-ollama ollama list | grep qwen2.5:3b
  # If missing:
  docker exec archivio-ollama ollama pull qwen2.5:3b-instruct-q4_K_M
  ```
- Re-ingest document to trigger graph extraction:
  ```bash
  curl -X POST http://localhost:8090/ingest \
    -H "X-Internal-Token: $RUST_ENGINE_INTERNAL_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"kb_id":"contracts_2024","content":"...","filename":"contract.pdf"}'
  ```

#### 7. Hallucination Detection Not Working (Fase 6.2)

**Symptoms**: `hallucination_score` always 0.0 or missing in `/chat` response.

**Diagnosis**:
```bash
# Test Python Worker hallucination endpoint directly
curl -X POST http://localhost:8091/verify_hallucination \
  -H "Content-Type: application/json" \
  -d '{
    "answer": "Test answer",
    "sources": [{"text_quote": "Test quote", "doc_id": "123"}]
  }'

# Check if detector is loaded (Python Worker logs)
# Should see: "HallucinationDetector preloaded and ready"
```

**Solutions**:
- Python Worker not running: Start native process (see Starting & Stopping Services)
- Redis cache not available: Check `docker ps | grep redis`
- Verify `verify_hallucinations` parameter in request:
  ```json
  POST /chat
  {
    "kb_id": "contracts_2024",
    "messages": [...],
    "verify_hallucinations": true  // Must be explicit
  }
  ```
- Check Rust Engine can reach Python Worker:
  ```bash
  docker exec archivio-rust-engine curl http://host.docker.internal:8091/health
  ```

#### 8. WebSocket Connection Drops Immediately (Fase 6.4)

**Symptoms**: Frontend shows "Disconnected" immediately after connect.

**Diagnosis**:
```bash
# Test WebSocket endpoint with wscat
npm install -g wscat
wscat -c "ws://localhost:8090/ws/collaborate/test_kb/test_doc?jwt=YOUR_JWT_TOKEN"

# Check Rust Engine logs for WebSocket errors
docker logs archivio-rust-engine --tail 50 | grep -i websocket
```

**Solutions**:
- Invalid JWT token: Verify token not expired
  ```bash
  # Decode JWT to check expiry (use jwt.io or jwt-cli)
  jwt decode $JWT_TOKEN
  ```
- Redis pub/sub not working:
  ```bash
  # Test Redis pub/sub manually
  docker exec -it archivio-redis redis-cli
  > SUBSCRIBE ws:collab:test_kb:test_doc
  # In another terminal:
  docker exec -it archivio-redis redis-cli
  > PUBLISH ws:collab:test_kb:test_doc "test message"
  ```
- Firewall blocking WebSocket upgrade: Check CORS configuration in `.env`:
  ```bash
  CORS_ORIGINS=http://localhost:5173,http://localhost:8080
  ```

#### 9. Permission Denied on KB Access (Fase 6.3)

**Symptoms**: `403 Forbidden` on `/query` or `/chat` even though user should have access.

**Diagnosis**:
```bash
# Check user's workspace membership
docker exec archivio-mysql mysql -uroot -pdevpass123 archivio_parlante_x \
  -e "SELECT * FROM ap_workspace_members WHERE user_id=1;"

# Check KB permissions
docker exec archivio-mysql mysql -uroot -pdevpass123 archivio_parlante_x \
  -e "SELECT * FROM ap_kb_permissions WHERE kb_id='contracts_2024';"

# Check if KB is in a workspace
docker exec archivio-mysql mysql -uroot -pdevpass123 archivio_parlante_x \
  -e "SELECT * FROM ap_knowledge_bases WHERE id='contracts_2024';"
```

**Solutions**:
- Permission cache stale: Clear Redis cache
  ```bash
  docker exec archivio-redis redis-cli KEYS "perm:*" | xargs docker exec archivio-redis redis-cli DEL
  ```
- User not added to workspace: Add via PHP Gateway API
  ```bash
  curl -X POST http://localhost:9080/api/workspaces/{workspace_id}/members \
    -H "Authorization: Bearer $JWT_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"user_id": 1, "role": "member"}'
  ```
- KB not shared with workspace: Grant permission
  ```bash
  # Direct SQL insert (admin only)
  docker exec archivio-mysql mysql -uroot -pdevpass123 archivio_parlante_x \
    -e "INSERT INTO ap_kb_permissions (kb_id, workspace_id, permission, granted_by) 
        VALUES ('contracts_2024', 'legal_team', 'read', 1);"
  ```

#### 10. Rust Compiler SIGSEGV During Build (Fase 6 Known Issue)

**Symptoms**: `docker compose build rust-engine` fails with:
```
error: rustc interrupted by SIGSEGV, printing backtrace
help: you can increase rustc's stack size by setting RUST_MIN_STACK=33554432
```

**Root Cause**: Insufficient stack size during `serde_derive` macro expansion in Docker build.

**Solutions**:
- **Option A** (Recommended): Increase Docker memory allocation
  1. Docker Desktop → Settings → Resources
  2. Memory: 8GB+ (from default 4GB)
  3. Rebuild: `docker compose build rust-engine`

- **Option B**: Increase Rust stack size in Dockerfile
  ```dockerfile
  # In engine-rust/Dockerfile, add before RUN cargo build:
  ENV RUST_MIN_STACK=33554432
  ```

- **Option C**: Use native build instead of Docker (development only)
  ```bash
  cd engine-rust
  cargo build --release
  # Stop Docker Rust container
  docker compose stop rust-engine
  # Run native binary
  RUST_ENGINE_INTERNAL_TOKEN=$TOKEN ./target/release/archivio-parlante-rust-engine
  ```

**Status**: Known blocker documented in `docs/FASE_6_INTEGRATION_TEST_BLOCKERS.md`

---

## Monitoring

### Key Metrics to Monitor

| Metric | Tool | Alert Threshold |
|---|---|---|
| Rust p95 latency | Prometheus | >5s |
| Error rate | Logs | >1% |
| Ollama availability | Health check | Down >1min |
| Qdrant disk usage | `df -h` | >80% |
| MySQL connections | `SHOW STATUS` | >100 |
| Memory usage (Docker) | `docker stats` | >90% |

### Log Access

```bash
# All services
make logs

# Single service
docker logs -f archivio-rust-engine

# Python Worker (native)
# Check the terminal where uvicorn is running
```

### Prometheus Metrics (if enabled)

```
http://localhost:9090/graph
```

Key queries:
- `rate(http_requests_total[5m])` - Request rate
- `histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))` - p95 latency
- `qdrant_collection_vectors_count` - Vector count per collection

---

## Disaster Recovery

### Scenario 1: Complete Data Loss

**Recovery Steps**:
1. Restore MySQL from latest backup:
   ```bash
   make restore-db FILE=backups/db_latest.sql.gz
   ```
2. Restore Qdrant snapshots (per collection)
3. Reindex missing documents:
   ```bash
   # Trigger reindex via admin API
   curl -X POST http://localhost:8090/admin/reindex \
     -H "X-Internal-Token: $RUST_ENGINE_INTERNAL_TOKEN"
   ```

**RTO**: 2-4 hours  
**RPO**: Last backup (daily = 24h max data loss)

### Scenario 2: Corrupted Qdrant Index

**Recovery**:
1. Delete corrupted collection
2. Recreate from MySQL metadata + reindex:
   ```bash
   # Via admin UI or API
   POST /admin/collections/contracts_2024/rebuild
   ```

### Scenario 3: Ollama Model Corruption

**Recovery**:
```bash
docker exec archivio-ollama ollama rm qwen2.5:7b-instruct-q4_K_M
docker exec archivio-ollama ollama pull qwen2.5:7b-instruct-q4_K_M
docker-compose restart rust-engine
```

---

## Contact & Escalation

**On-Call**: [Your on-call rotation]  
**Incident Channel**: [Slack/Teams channel]  
**Escalation**: [Email/phone for critical issues]

---

**Last Reviewed**: 2026-05-06  
**Next Review**: 2026-08-06  
