# Archivio Parlante — System Architecture

**Version**: 1.0  
**Last Updated**: 2026-05-06  
**Status**: Production-Ready (6/7 services, Python worker requires native startup)

---

## Table of Contents

1. [High-Level Overview](#high-level-overview)
2. [Service Architecture](#service-architecture)
3. [Data Flow](#data-flow)
4. [LLM Provider Registry](#llm-provider-registry)
5. [Storage Architecture](#storage-architecture)
6. [Security Architecture](#security-architecture)
7. [Deployment Architecture](#deployment-architecture)

---

## High-Level Overview

Archivio Parlante is a **greenfield RAG (Retrieval-Augmented Generation) enterprise system** for forensic analysis of Italian business contracts with zero-hallucination guarantees.

### Design Principles

1. **Zero-Cost Default**: Entire stack runs offline with no required API keys
2. **Privacy-First**: Local LLM processing, no data sent to cloud by default
3. **Production-Grade**: OWASP ASVS L2 compliance, 80%+ test coverage
4. **Hybrid Architecture**: Local-first with opt-in cloud provider support
5. **Microservices**: 7 containerized services, independently scalable

### Technology Stack

| Layer | Technology | Purpose |
|---|---|---|
| **Frontend** | React 18 + Vite + TypeScript | SPA with real-time chat, document management, comparison UI |
| **API Gateway** | PHP 8.2 + Slim 4 | Authentication, rate limiting, request routing |
| **Core Engine** | Rust 1.82 + Axum + Tokio | RAG pipeline, hybrid search, multi-contract comparison |
| **AI Worker** | Python 3.11 + FastAPI | PDF parsing, OCR, BGE reranking, knowledge graph extraction |
| **Vector Database** | Qdrant 1.12 | Dense (cosine) + sparse (BM25) hybrid search |
| **LLM (Local)** | Ollama | qwen2.5:7b-instruct (default), nomic-embed-text (embeddings) |
| **RDBMS** | MySQL 8.0 | User accounts, documents metadata, audit logs |
| **Cache** | Redis 7 | Rate limiting, session storage |

---

## Service Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         User Browser                             │
│                    (React 18 SPA + Vite)                        │
└────────────────────────────┬────────────────────────────────────┘
                             │ HTTP/S (JWT tokens)
                             ↓
┌─────────────────────────────────────────────────────────────────┐
│                  PHP Gateway (Slim 4)                            │
│  - JWT authentication & refresh                                  │
│  - Rate limiting (Redis-backed)                                  │
│  - Request validation & audit logging                            │
│  - Proxies to Rust engine with X-Internal-Token                 │
└────────────────────────────┬────────────────────────────────────┘
                             │ Internal auth (X-Internal-Token)
                             ↓
┌─────────────────────────────────────────────────────────────────┐
│              Rust Engine (Axum + Tokio)                         │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  LLM Provider Registry (Multi-Provider Runtime Switching)│  │
│  │  - Ollama (local, zero-cost default)                     │  │
│  │  - Anthropic, Google, OpenAI, DeepSeek (opt-in)          │  │
│  └──────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  RAG Pipeline                                             │  │
│  │  1. Chunking (Semantic + Contextual Retrieval)          │  │
│  │  2. Hybrid Search (Dense + Sparse RRF k=60)             │  │
│  │  3. Reranking (BGE-reranker-v2-m3 via Python)           │  │
│  │  4. LLM Response Generation (Self-RAG validation)       │  │
│  └──────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  Multi-Contract Comparison                                │  │
│  │  - Parallel retrieval across N contracts                 │  │
│  │  - Aspect-based comparison table generation              │  │
│  │  - Information gap detection                             │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────┬───────────────┬──────────────┬────────────────────────────┘
      │               │              │
      ↓               ↓              ↓
┌─────────────┐ ┌─────────────┐ ┌──────────────────────┐
│   Qdrant    │ │   Ollama    │ │  Python AI Worker    │
│  (Vectors)  │ │  (LLM)      │ │  (FastAPI)           │
│             │ │             │ │  - PDF parsing       │
│  Dense:     │ │  Chat:      │ │  - OCR (Tesseract)   │
│  768-dim    │ │  qwen2.5:7b │ │  - BGE reranker      │
│  cosine     │ │             │ │  - Knowledge graph   │
│             │ │  Embed:     │ │  - Contextual chunks │
│  Sparse:    │ │  nomic-     │ └──────────────────────┘
│  BM25       │ │  embed-text │
│  (tantivy)  │ │             │
└─────────────┘ └─────────────┘
      │                               
      ↓                               
┌─────────────────────────────────────┐
│         MySQL 8.0 + Redis 7         │
│  - Users, sessions, API keys        │
│  - Documents metadata               │
│  - Audit logs (auth, query, upload) │
│  - Rate limit counters (Redis)      │
└─────────────────────────────────────┘
```

### Port Mappings

| Service | Internal Port | External Port |
|---|---|---|
| PHP Gateway | 80 | 9080 |
| Rust Engine | 8090 | 8090 |
| Python Worker | 8091 | 8091 |
| Qdrant REST | 6333 | 6335 |
| Qdrant gRPC | 6334 | 6336 |
| Ollama | 11434 | 11434 |
| MySQL | 3306 | 3307 |
| Redis | 6379 | 6380 |

---

## Data Flow

### 1. Document Ingestion

```
User uploads PDF → PHP Gateway (auth) → Rust Engine
                                            ↓
                                    Python Worker (parse PDF)
                                            ↓
                                    Extract text + metadata
                                            ↓
                                    Rust Engine (chunk + contextualize)
                                            ↓
                                    Ollama (embed chunks → 768-dim vectors)
                                            ↓
                                    Qdrant (store vectors + BM25 index)
                                            ↓
                                    MySQL (store doc metadata)
```

**Key Steps**:
1. **Validation**: MIME type whitelist, size limit (200MB), virus scan
2. **Parsing**: PyMuPDF + pdfplumber + Unstructured (fallback OCR)
3. **Chunking**: Semantic chunker (800 tokens, 15% overlap)
4. **Contextualization**: Anthropic technique — prefix each chunk with document context
5. **Embedding**: Ollama nomic-embed-text (768-dim)
6. **Indexing**: Qdrant stores both dense vectors + sparse BM25 tokens

### 2. RAG Query

```
User query → PHP Gateway (auth, rate limit) → Rust Engine
                                                  ↓
                                          Ollama (embed query)
                                                  ↓
                                          Qdrant Hybrid Search
                                          - Dense: cosine similarity (top 30)
                                          - Sparse: BM25 (top 30)
                                          - Fusion: RRF k=60 → top 30
                                                  ↓
                                          Python Worker (BGE rerank)
                                          - Cross-encoder reranking → top 5
                                                  ↓
                                          Rust Engine (build prompt)
                                          - System prompt + query + top 5 chunks
                                                  ↓
                                          Ollama LLM (generate answer)
                                          - qwen2.5:7b-instruct (default)
                                                  ↓
                                          Self-RAG Validation
                                          - Verify all claims have citations
                                          - Flag information gaps
                                                  ↓
                                          Return to user with sources
```

**Anti-Hallucination Stack**:
1. **Hybrid Search**: Dense + Sparse prevents recall gaps
2. **Reranker**: Cross-encoder ensures semantic relevance
3. **Contextual Retrieval**: Chunks have document context
4. **Self-RAG**: LLM validates own answers against retrieved chunks
5. **Citation Enforcement**: All claims must reference `text_quote` verbatim

### 3. Multi-Contract Comparison

```
User selects contracts [A, B, C] + aspects ["penalties", "termination"]
                                ↓
                     Parallel RAG queries (one per contract)
                                ↓
                     Aggregate results into comparison table
                     | Aspect      | Contract A | Contract B | Contract C |
                     |-------------|------------|------------|------------|
                     | Penalties   | 10% fee    | None found | 5% + €1K   |
                     | Termination | 30 days    | 60 days    | No clause  |
                                ↓
                     Identify key differences + information gaps
                                ↓
                     Return structured comparison (Markdown table)
```

---

## LLM Provider Registry

**File**: `engine-rust/src/providers/registry.rs`

### Architecture

The system uses a **runtime-switchable multi-provider architecture** that allows:
- Default local processing (Ollama, zero-cost)
- Opt-in cloud providers (Anthropic, Google, OpenAI, DeepSeek, etc.)
- Per-request provider selection via API parameter

### Provider Interface

All providers implement the `LlmProvider` trait:

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn available_models(&self) -> Result<Vec<ModelInfo>>;
    async fn is_available(&self) -> bool;
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;
    async fn chat_stream(&self, request: ChatRequest) -> Result<StreamResponse>;
    async fn embed(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>>;
}
```

### Registered Providers

| Provider | Status | Cost | API Key Required | Models |
|---|---|---|---|---|
| **Ollama** | ✅ Always enabled | Free | ❌ No | qwen2.5:7b, qwen2.5:3b, nomic-embed-text |
| **Anthropic** | ⏸️ Opt-in | Paid | ✅ Yes (`ANTHROPIC_API_KEY`) | Claude Opus 4.7, Sonnet 4.6, Haiku 4.5 |
| **Google** | ⏸️ Opt-in | Paid | ✅ Yes (`GOOGLE_API_KEY`) | Gemini 2.5 Pro, 2.5 Flash |
| **OpenAI** | ⏸️ Opt-in | Paid | ✅ Yes (`OPENAI_API_KEY`) | GPT-5, o3 |
| **DeepSeek** | ⏸️ Opt-in | Paid | ✅ Yes (`DEEPSEEK_API_KEY`) | DeepSeek V3, R1 |

### Budget Guard

- `DAILY_COST_BUDGET_EUR=0.00` by default → blocks all cloud providers
- Admin must explicitly raise budget via environment variable
- Cost tracking per request (logged to MySQL `llm_usage` table)

### Provider Selection

```typescript
// Frontend request example
POST /api/query
{
  "kb_id": "contracts_2024",
  "query": "Quali sono le penali?",
  "provider": "ollama",  // or "anthropic", "google", etc.
  "model": "qwen2.5:7b-instruct-q4_K_M"  // provider-specific model
}
```

```rust
// Rust engine provider resolution
let provider = registry
    .get_or_default(request.provider.as_deref()) // Defaults to "ollama"
    .await?;

let response = provider.chat(chat_request).await?;
```

### Embedding Model Configuration

**Phase 4 Improvement**: Embedding model is now configurable via environment variable instead of hardcoded.

```rust
// Before (hardcoded):
model: "nomic-embed-text".to_string(), // hardcoded for now, config later

// After (from config):
model: self.embed_model.clone(),  // Read from OLLAMA_MODEL_EMBED env var
```

**Environment Configuration**:
```bash
OLLAMA_MODEL_EMBED=nomic-embed-text  # 768-dim, default
# Or for multi-lingual:
# OLLAMA_MODEL_EMBED=paraphrase-multilingual
```

---

## Storage Architecture

### Vector Database (Qdrant)

**Purpose**: Dense + sparse hybrid vector search

**Schema** (per knowledge base):
- **Collection name**: `kb_{kb_id}` (e.g., `kb_contracts_2024`)
- **Vector config**:
  - **Dense**: 768-dim float32, cosine distance
  - **Sparse**: BM25 full-text index (via tantivy)
- **Payload**:
  ```json
  {
    "doc_id": "contract_001",
    "chunk_id": "abc123",
    "text": "Clausola 4.2: Penali per inadempimento...",
    "context": "Contratto di fornitura 2024, Sezione Penali",
    "chunk_index": 42,
    "is_contextualized": true,
    "page": 7,
    "section": "Clausole Penali"
  }
  ```

**Operations**:
- **Index**: POST `/collections/{kb_id}/points` (batch upsert)
- **Search**: POST `/collections/{kb_id}/points/search` (dense + sparse)
- **Delete**: DELETE `/collections/{kb_id}/points/{point_id}`

### Relational Database (MySQL)

**Database**: `archivio_parlante_x`

**Tables** (with `ap_` prefix):
- `ap_users`: User accounts (id, email, password_hash, role, created_at)
- `ap_knowledge_bases`: KB metadata (id, name, owner_id, created_at)
- `ap_documents`: Document metadata (id, kb_id, filename, mime_type, status, indexed_at)
- `ap_chat_messages`: Chat history (id, user_id, kb_id, query, answer, sources, verified, created_at)
- `ap_llm_providers`: Enabled providers + API keys (encrypted)
- `ap_llm_usage`: Cost tracking (provider, model, tokens, cost_eur, timestamp)
- `ap_audit_log`: All operations (user_id, action, resource, ip, timestamp)

**Soft Delete**: Documents and messages use `deleted_at NULL` column instead of hard DELETE

### Cache (Redis)

**Use Cases**:
- Rate limiting counters: `rate_limit:{user_id}` (TTL 60s)
- Session storage: `session:{token}` (TTL 24h)
- API key validation cache: `api_key:{hash}` (TTL 5min)

---

## Security Architecture

### Authentication Flow

```
User login → PHP Gateway
               ↓
         Verify credentials (MySQL)
               ↓
         Generate JWT pair (access + refresh)
         - Access token: 15 min TTL, HS256, {user_id, role}
         - Refresh token: 7 days TTL, stored in Redis
               ↓
         Return tokens to client (httpOnly cookie + localStorage)

User request → PHP Gateway
                 ↓
           Validate JWT (check signature + expiry)
                 ↓
           Add X-Internal-Token header (PHP → Rust auth)
                 ↓
           Rust Engine validates X-Internal-Token
                 ↓
           Process request
```

### CORS Configuration (Phase 2 Fix)

**Before**: Permissive allow-all (`Access-Control-Allow-Origin: *`)  
**After**: Explicit origin whitelist

```rust
// Production mode validation (config.rs)
if app_env == "production" && cors_origins.iter().any(|o| o == "*") {
    anyhow::bail!("CORS allow-all (*) forbidden in production mode");
}

// Runtime CORS layer (main.rs)
let cors_layer = if config.cors_origins.iter().any(|o| o == "*") {
    CorsLayer::permissive()  // Dev mode only
} else {
    CorsLayer::new()
        .allow_origin(explicit_origins)
        .allow_credentials(true)
        .allow_headers([CONTENT_TYPE, AUTHORIZATION, X_INTERNAL_TOKEN])
};
```

**Environment Configuration**:
```bash
# Dev mode
CORS_ORIGINS=http://localhost:3000,http://localhost:5173

# Production mode
CORS_ORIGINS=https://archivioparlante.com
```

### Secrets Management

| Secret | Environment Variable | Required | Validation |
|---|---|---|---|
| Internal auth token | `RUST_ENGINE_INTERNAL_TOKEN` | ✅ (production) | 64+ hex chars |
| JWT signing key | `JWT_SECRET` | ✅ (production) | 32+ chars |
| MySQL password | `MYSQL_PASSWORD` | ✅ | Non-empty |
| Anthropic API key | `ANTHROPIC_API_KEY` | ❌ | Valid sk-ant-* format |
| Google API key | `GOOGLE_API_KEY` | ❌ | Valid AIza* format |

**Production Validation** (config.rs):
```rust
if app_env == "production" {
    if rust_engine_internal_token.is_empty() {
        anyhow::bail!("RUST_ENGINE_INTERNAL_TOKEN required in production mode");
    }
    if jwt_secret.is_empty() {
        anyhow::bail!("JWT_SECRET required in production mode");
    }
}
```

---

## Deployment Architecture

### Docker Compose Orchestration

```yaml
services:
  php-gateway:
    build: ./php-gateway
    ports: ["9080:80"]
    depends_on: [mysql, redis]
    environment:
      - APP_ENV=production
      - JWT_SECRET=${JWT_SECRET}
      - RUST_ENGINE_URL=http://rust-engine:8090

  rust-engine:
    build: ./engine-rust
    ports: ["8090:8090"]
    depends_on: [qdrant, ollama, mysql, python-worker]
    environment:
      - APP_ENV=production
      - RUST_ENGINE_INTERNAL_TOKEN=${RUST_ENGINE_INTERNAL_TOKEN}
      - CORS_ORIGINS=${CORS_ORIGINS}
      - OLLAMA_URL=http://ollama:11434
      - OLLAMA_MODEL_EMBED=${OLLAMA_MODEL_EMBED}
      - QDRANT_URL=http://qdrant:6333

  python-worker:
    build: ./engine-python
    ports: ["8091:8091"]
    volumes:
      - ./shared/uploads:/shared/uploads

  qdrant:
    image: qdrant/qdrant:v1.12.0
    ports: ["6335:6333", "6336:6334"]
    volumes:
      - qdrant_data:/qdrant/storage

  ollama:
    image: ollama/ollama:latest
    ports: ["11434:11434"]
    volumes:
      - ollama_models:/root/.ollama

  mysql:
    image: mysql:8.0
    ports: ["3307:3306"]
    environment:
      - MYSQL_ROOT_PASSWORD=${MYSQL_PASSWORD}
      - MYSQL_DATABASE=archivio_parlante_x

  redis:
    image: redis:7-alpine
    ports: ["6380:6379"]
```

### Health Checks

All services expose `/health` endpoints:

```bash
# PHP Gateway
curl http://localhost:9080/health
# → {"status":"ok","service":"php-gateway","version":"1.0.0"}

# Rust Engine
curl http://localhost:8090/health
# → {"status":"ok","service":"rust-engine","version":"0.1.0","providers":["ollama"],"cloud_enabled":false}

# Python Worker
curl http://localhost:8091/health
# → {"status":"ok","service":"python-worker","version":"0.1.0"}

# Qdrant
curl http://localhost:6335/health
# → 200 OK

# Ollama
curl http://localhost:11434/api/tags
# → {"models":[...]}
```

### Monitoring

- **Logs**: All services use structured JSON logging (tracing, structlog)
- **Metrics**: Prometheus metrics exposed at `/metrics` (Rust engine)
  - `http_requests_total`
  - `http_errors_total`
  - `llm_calls_total`
  - `qdrant_queries_total`
  - `documents_ingested_total`
- **Tracing**: OpenTelemetry-compatible (future: Jaeger integration)

---

## Performance Characteristics

### Target KPIs

| Metric | Target | Measured |
|---|---|---|
| Ingestion throughput | >100 pages/min | ⏳ TBD |
| RAG query latency (p95) | <500ms (local LLM) | ⏳ TBD |
| Recall@10 | >95% | ⏳ TBD |
| Precision@5 | >90% | ⏳ TBD |
| Hallucination rate | <1% | ⏳ TBD |
| Multi-contract comparison | 50+ contracts in <2s | ⏳ TBD |

### Scalability

- **Concurrent embeddings**: Max 16 (configurable via `MAX_CONCURRENT_EMBEDDINGS`)
- **Concurrent LLM calls**: Max 8 (configurable via `MAX_CONCURRENT_LLM_CALLS`)
- **Qdrant sharding**: Horizontal scaling via collection sharding (future)
- **Rust engine**: Stateless, horizontally scalable behind load balancer

---

## Future Architecture Enhancements

### Phase 5: Advanced RAG Features
- Knowledge graph entity extraction (spaCy NER)
- Graph-based retrieval (Neo4j integration)
- Multi-hop reasoning chains

### Phase 6: Observability
- Distributed tracing (Jaeger)
- Real-time metrics dashboards (Grafana)
- Cost breakdown per query (LLM provider + compute)

### Phase 7: Scale
- Kubernetes deployment manifests
- Read replicas for MySQL
- Qdrant cluster sharding
- CDN for frontend assets

---

## References

- **Implementation Plan**: `implementation_plan.md`
- **Security Audit**: `docs/SECURITY_AUDIT_PHASE_2.md`
- **CLAUDE.md**: `.claude/CLAUDE.md` (project-level instructions)
- **ADRs**: `docs/ADR/*.md` (architecture decision records)

---

**Document Version**: 1.0  
**Maintained By**: Development Team  
**Last Review**: 2026-05-06
