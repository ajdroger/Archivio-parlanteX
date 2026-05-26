# 🦀 Archivio Parlante - Rust Engine

RAG enterprise engine for Italian legal contract forensic analysis with zero hallucinations.

## Overview

The Rust Engine is the core processing component of Archivio Parlante, handling:
- **Document ingestion** with semantic chunking and contextual retrieval
- **Hybrid search** (dense + sparse BM25 with RRF fusion)
- **Multi-contract comparison** with parallel retrieval
- **Multi-provider LLM support** (Ollama default, 12+ cloud providers opt-in)
- **Intent-based routing** for intelligent query handling

## Features

- ✅ **Zero-cost default**: Ollama local models (qwen2.5:7b, nomic-embed-text)
- ✅ **Hybrid RAG**: Dense (768-dim cosine) + Sparse (BM25) with Reciprocal Rank Fusion
- ✅ **Contextual Retrieval**: Anthropic technique for -49% retrieval errors
- ✅ **Multi-contract analysis**: Parallel per-doc retrieval with aspect extraction
- ✅ **Security**: Internal auth token + rate limiting
- ✅ **Observability**: Prometheus metrics + JSON structured logging
- ✅ **API docs**: OpenAPI spec + Swagger UI at `/docs`

## Quick Start

### Prerequisites

- Rust 1.82+
- Running services:
  - Qdrant (host REST **6335**; internal `http://qdrant:6333` in Docker)
  - Ollama (port 11434)
  - Python AI Worker (port 8091)

### Environment Variables

Copy `.env.example` to `.env` and configure:

```bash
# Server
LISTEN_ADDR=0.0.0.0:8090
LOG_FORMAT=pretty  # or "json" for production

# Qdrant
QDRANT_URL=http://qdrant:6333

# Ollama (local LLM - default zero-cost)
OLLAMA_URL=http://ollama:11434
OLLAMA_MODEL_CHAT=qwen2.5:7b-instruct-q4_K_M
OLLAMA_MODEL_CHAT_SMALL=qwen2.5:3b-instruct-q4_K_M
OLLAMA_MODEL_EMBED=nomic-embed-text

# Python AI Worker
PYTHON_WORKER_URL=http://python-worker:8091

# Security
RUST_ENGINE_INTERNAL_TOKEN=your-secret-token-here

# Cloud LLM providers (opt-in, leave empty for zero-cost)
ANTHROPIC_API_KEY=
GOOGLE_API_KEY=
OPENAI_API_KEY=
# ... (see .env.example for all providers)
```

### Build & Run

```bash
# Development
cargo run

# Production release
cargo build --release
./target/release/archivio-parlante-rust-engine
```

## API Endpoints

### Core Endpoints

#### `POST /ingest`
Ingest a document into the knowledge base.

**Request:**
```json
{
  "doc_id": "doc_123",
  "kb_id": "kb_legal",
  "file_path": "/shared/contract.pdf",
  "source_name": "NDA_2024.pdf",
  "mime_type": "application/pdf",
  "tags": ["nda", "2024"]
}
```

**Response:**
```json
{
  "doc_id": "doc_123",
  "chunks_indexed": 45,
  "processing_ms": 3500,
  "entities_extracted": 0
}
```

#### `POST /query`
Query documents with hybrid search + reranking.

**Request:**
```json
{
  "query": "Qual è la durata del contratto?",
  "kb_id": "kb_legal",
  "top_k": 5
}
```

**Response:**
```json
{
  "results": [
    {
      "chunk_id": "uuid-...",
      "doc_id": "doc_123",
      "chunk_index": 3,
      "text": "Il contratto avrà durata di 24 mesi...",
      "score": 0.95
    }
  ],
  "processing_ms": 450,
  "candidates_count": 30
}
```

#### `POST /compare_contracts`
Compare multiple contracts side-by-side.

**Request:**
```json
{
  "kb_id": "kb_legal",
  "doc_ids": ["doc_2023", "doc_2024"],
  "question": "Confronta durata, penali e foro competente",
  "save_analysis": false
}
```

**Response:**
```json
{
  "markdown_result": "# Confronto Contratti\n\n| Aspetto | ... |",
  "structured": {
    "aspects": [...],
    "differences_summary": "...",
    "recommendations": [...]
  },
  "processing_ms": 5200
}
```

### KB Management

- `GET /kb/{kb_id}/documents` - List documents
- `DELETE /kb/{kb_id}/documents/{doc_id}` - Delete document
- `GET /kb/{kb_id}/graph?doc_ids=X,Y` - Knowledge graph
- `GET /kb/{kb_id}/stats` - KB statistics
- `POST /admin/reindex/{kb_id}` - Reindex KB (background job)

### Health & Observability

- `GET /health` - Health check
- `GET /metrics` - Prometheus metrics
- `GET /docs` - Swagger UI
- `GET /openapi.json` - OpenAPI spec

## Architecture

```
┌─────────────────────────────────────────────────┐
│          Rust Engine (Axum + Tokio)            │
│                                                 │
│  ┌──────────┐  ┌──────────┐  ┌──────────────┐ │
│  │ Ingestion│  │  Query   │  │ Multi-Contract│ │
│  │ Pipeline │  │  Engine  │  │  Comparison  │ │
│  └────┬─────┘  └────┬─────┘  └───────┬──────┘ │
│       │             │                 │         │
│  ┌────▼─────────────▼─────────────────▼──────┐ │
│  │      Hybrid Search (Dense + Sparse)       │ │
│  │      RRF Fusion + BGE Reranker           │ │
│  └────┬──────────────────────────────┬──────┘ │
│       │                               │         │
└───────┼───────────────────────────────┼─────────┘
        │                               │
   ┌────▼────┐                     ┌───▼────┐
   │ Qdrant  │                     │ Ollama │
   │(Vector  │                     │  LLM   │
   │  DB)    │                     │        │
   └─────────┘                     └────────┘
```

## Testing

```bash
# Unit tests
cargo test

# Integration tests (requires full stack up)
cargo test --test '*_e2e' -- --ignored --nocapture

# Specific test
cargo test --test query_e2e -- --ignored --nocapture
```

## Security

### Internal Authentication

All endpoints (except `/health`, `/metrics`, `/docs`) require `X-Internal-Token` header:

```bash
curl -H "X-Internal-Token: your-secret-token" \
  http://localhost:8090/query \
  -d '{"query": "...", "kb_id": "..."}'
```

### Rate Limiting

100 requests/minute per IP (placeholder, configurable).

## Observability

### Prometheus Metrics

Available at `GET /metrics`:

```
# HELP http_requests_total Total number of HTTP requests
# HELP llm_calls_total Total number of LLM API calls
# HELP qdrant_queries_total Total number of Qdrant vector searches
# HELP documents_ingested_total Total number of documents ingested
# HELP chunks_indexed_total Total number of chunks indexed
```

### Structured Logging

Set `LOG_FORMAT=json` for production JSON logs:

```bash
LOG_FORMAT=json cargo run
```

## Performance

- **Ingestion**: ~3-5 seconds for 20-page PDF (includes chunking, contextual enrichment, embedding)
- **Query**: < 500ms p95 (hybrid search + reranking)
- **Multi-contract comparison**: ~5-15 seconds for 3-5 contracts

## Configuration

### Chunking

- Default chunk size: 800 tokens
- Overlap: 15%
- Italian legal-aware (respects Art., CAPO, Sezione markers)

### Hybrid Search

- Dense: top 30 (cosine similarity, 768-dim)
- Sparse: top 30 (BM25, tantivy tokenizer)
- RRF k=60
- Reranker: BGE cross-encoder (top 5 final)

### LLM Providers

Default: Ollama (zero-cost)

Cloud providers (opt-in via API keys):
- Anthropic (Claude)
- Google (Gemini)
- OpenAI (GPT)
- DeepSeek, Qwen, Moonshot, Zhipu
- Mistral, Groq, OpenRouter, Together, Fireworks

## Development

```bash
# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings

# Security audit
cargo audit

# Build optimized release
cargo build --release --target x86_64-unknown-linux-gnu

# Binary size (should be < 50 MB)
ls -lh target/release/archivio-parlante-rust-engine
```

## License

MIT License - see LICENSE file

## Support

For issues and questions, see the main repository documentation.
