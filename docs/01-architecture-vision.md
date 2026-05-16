# Architecture Vision — Archivio Parlante

**Data**: 2026-04-21  
**Versione**: 1.0 (Fase -1)

---

## 🎯 Problem Statement

### Il Problema

**Enti istituzionali ad alto rischio reputazionale** (agenzie governative, autorità di regolamentazione, grandi aziende) gestiscono centinaia o migliaia di **contratti aziendali complessi** in italiano (NDA, accordi commerciali, contratti di servizio, appalti pubblici). L'analisi manuale di questi contratti è:

- ⏱️ **Lenta**: giorni/settimane per comparare 10+ contratti
- 💰 **Costosa**: team legali pagati a ore per analisi ripetitive
- 🚫 **Limitata**: difficile trovare pattern nascosti o discrepanze tra contratti
- ❌ **Error-prone**: rischio di oversight su clausole critiche (penali, giurisdizioni, termini di recesso)

### Soluzioni Esistenti (Inadeguate)

| Soluzione | Problema |
|---|---|
| **Ricerca full-text (Ctrl+F)** | Trova solo match esatti, ignora sinonimi/varianti |
| **ChatGPT / Claude web** | ⚠️ **Allucinazioni** (inventa clausole inesistenti), no privacy (dati su server OpenAI/Anthropic), no citation enforcement |
| **RAG tools generici** (Verba, Quivr, AnythingLLM) | Non enterprise-grade, no knowledge graph legale, no confronto multi-contratto parallelo, no Self-RAG validation |
| **Legal AI SaaS** (LexisNexis, Kira Systems) | 💰 €€€€ costosi, vendor lock-in, no self-hosting (dati sensibili su cloud terzi) |

### Cosa Serve

Un sistema **self-hosted, zero-allucinazioni, enterprise-grade** che:

1. **Analizza contratti italiani** con comprensione semantica (sinonimi, varianti, contesto)
2. **Confronta 50+ contratti in parallelo** (es. "Quali contratti hanno penali > €10k?")
3. **Zero allucinazioni** tramite citation enforcement + Self-RAG validation
4. **Privacy totale** (LLM locale Ollama di default, dati mai escono dal perimetro)
5. **Zero-cost** (stack OSS, API cloud opt-in solo se admin lo abilita)
6. **Knowledge Graph legale** (entities: PARTI, DATE, IMPORTI, CLAUSOLE, GIURISDIZIONI)

---

## 👥 Stakeholder Primari

### 1. Legal Teams (Utenti Finali)

**Chi sono**: avvocati, compliance officers, contract managers in enti istituzionali

**Bisogni**:
- Risposta rapida a domande precise: *"Qual è la penale in questo NDA?"*, *"Quali contratti hanno giurisdizione Milano?"*
- Citation verbatim (quote esatti) per verificare fonti
- Confronto multi-contratto: *"Confronta le clausole di recesso nei 15 NDA del 2024"*
- Zero allucinazioni (requisito vincolante — una risposta inventata può costare milioni)

**KPI per loro**:
- Time-to-answer: < 30 secondi per query semplice, < 2 minuti per confronto 50 contratti
- Accuracy: Recall@10 > 95%, Precision@5 > 90%
- Hallucination rate: < 1%

---

### 2. IT/Security Teams (Admin/Deployer)

**Chi sono**: SysAdmin, CISO, DevOps in enti istituzionali

**Bisogni**:
- Self-hosting on-premise (no cloud SaaS, dati sensibili restano in perimetro)
- Stack OSS (no vendor lock-in, audit security possibile)
- RBAC (Role-Based Access Control) — solo utenti autorizzati accedono a specifici knowledge base
- Audit log completo (chi ha fatto quale query, quando, su quali documenti)
- Zero-cost di default (modelli locali), API cloud opzionali e budgettate

**KPI per loro**:
- Uptime: > 99.9%
- Security: zero CVE High/Critical, OWASP ASVS L2 compliance
- Privacy: PII cifrate at-rest, log PII redatti

---

### 3. Management / Decision Makers

**Chi sono**: C-level, procurement, finance in enti

**Bisogni**:
- ROI misurabile: riduzione tempo analisi contratti di 80%+ (giorni → ore)
- Budget prevedibile: zero-cost default, costi cloud opt-in e controllati con budget guard
- Reporting: dashboard con metriche utilizzo (query/giorno, documenti indicizzati, cost/query)

**KPI per loro**:
- Cost/query: €0.00 con LLM locali, <€0.10 con cloud premium
- Time saved: 80%+ riduzione vs analisi manuale
- User adoption: > 70% legal team usa il sistema regolarmente

---

## 🖥️ Constraint Hardware

### Ambiente di Sviluppo

**Hardware disponibile**:

```
MSI Raider GE78HX 13VG
- CPU: Intel i9-13950HX (24 core / 32 thread)
- RAM: 32 GB DDR5
- GPU: NVIDIA RTX 4070 Laptop 8 GB VRAM    ← VINCOLO PRIMARIO
- SSD: NVMe 2 TB (~966 GB liberi)
- OS: Windows 11 Pro + Docker Desktop + WSL2
```

**Implicazioni Architetturali**:

1. **Modelli LLM locali: MAX 14B parametri in Q4**
   - `qwen2.5:7b` (4.7 GB VRAM) → default chat
   - `qwen2.5:3b` (2.0 GB VRAM) → task massivi (contextual retrieval)
   - `nomic-embed-text` (0.3 GB VRAM) → embedding
   - **Totale VRAM**: ~7 GB (lascia 1 GB per OS/buffer)

2. **Modelli > 14B → API cloud**
   - `qwen2.5:32b`, Llama 3.3 70B, Mixtral 8×22B → **NON entrano in 8 GB VRAM**
   - Per task critici: Claude Opus 4.7, Gemini 2.5 Pro, DeepSeek V3 → via API cloud
   - **Budget guard**: `DAILY_COST_BUDGET_EUR=0.00` default (admin deve alzare)

3. **Embedding GPU-accelerated**
   - `nomic-embed-text` usa GPU (10-20× più veloce vs CPU)
   - Batch embedding: 500+ chunk/secondo con RTX 4070

4. **Reranker BGE-v2-m3**
   - Cross-encoder PyTorch su GPU
   - ~1.2 GB VRAM (condiviso con embedding)
   - Alternativa CPU-only se VRAM shortage

### Ambiente Produzione (Target Futuro)

**Raccomandato per ente**:

```
Server on-premise
- CPU: AMD EPYC 7763 (64 core) o Intel Xeon Platinum
- RAM: 128 GB DDR4 ECC
- GPU: NVIDIA A100 80 GB o H100 80 GB    ← per modelli 70B+ locali
- SSD: NVMe RAID 10, 4 TB
- OS: Ubuntu 22.04 LTS Server
```

**Oppure (budget-friendly)**:

```
Workstation
- CPU: AMD Ryzen Threadripper PRO 5975WX (32 core)
- RAM: 256 GB DDR4 ECC
- GPU: NVIDIA RTX 6000 Ada 48 GB VRAM    ← modelli fino a 70B in Q4
- SSD: NVMe 8 TB
```

**Scalabilità**: Multi-GPU con vLLM per throughput 10× vs Ollama (produzione alta domanda).

---

## 🎯 SLO (Service Level Objectives)

### Performance

| Metrica | Target | Misurazione |
|---|---|---|
| **Query RAG semplice** (p95 latency) | < 500 ms | k6 load test, 100 query concorrenti |
| **Multi-contract comparison** (50 contratti) | < 2 secondi | Benchmark suite Fase 5 |
| **Ingestion throughput** | > 100 pagine PDF/minuto | Benchmark ingest con PDF reali |
| **Concurrent users** | > 50 utenti simultanei | k6 stress test |

### Accuracy

| Metrica | Target | Misurazione |
|---|---|---|
| **Recall@10** (retrieval) | > 95% | Eval dataset 50 Q&A con ground truth |
| **Precision@5** (top results) | > 90% | Eval dataset 50 Q&A |
| **Hallucination rate** | < 1% | Self-RAG evaluator su 200 query |
| **Citation accuracy** | 100% | Validation: ogni quote LLM deve esistere verbatim in chunk |

### Availability & Security

| Metrica | Target | Misurazione |
|---|---|---|
| **Uptime** | > 99.9% (8.76 ore downtime/anno max) | Monitoring Prometheus + Grafana |
| **Data loss** | Zero (backup giornaliero) | Automated backup MySQL + Qdrant + audit log |
| **Vulnerabilità CVE** | Zero High/Critical | `cargo audit`, `pip-audit`, `trivy image` in CI |
| **RBAC enforcement** | 100% (no unauthorized access) | Penetration test + audit log review |

### Cost

| Metrica | Target | Misurazione |
|---|---|---|
| **Cost/query** (LLM locale) | €0.00 | Ollama locale gratuito |
| **Cost/query** (cloud premium) | < €0.10 | CostTracker in Rust core (API metering) |
| **Daily budget overage** | 0 occorrenze | Budget guard soft-limit alert |

---

## 🏗️ Architettura High-Level (Post Fase -1)

### Approccio Scelto: **Ibrido** (Decision Matrix §00)

```
┌─────────────────────────────────────────────────────────────────┐
│  React 18 SPA (Vite + TS + TailwindCSS + shadcn/ui)             │
│  UI base: Open WebUI layout Svelte (adattato)                    │
└────────────────┬────────────────────────────────────────────────┘
                 │ HTTPS + JWT
┌────────────────▼────────────────────────────────────────────────┐
│  PHP 8.2 Gateway (Slim 4) — sottile                              │
│  - Auth JWT, RBAC, Rate limiting Redis                           │
│  - Audit log, proxy verso Rust Engine                            │
└────────────────┬────────────────────────────────────────────────┘
                 │ REST/JSON interno
┌────────────────▼────────────────────────────────────────────────┐
│  🦀 Rust Core Engine (Axum + Tokio) — custom                     │
│  ───────────────────────────────────────────────────────────────│
│  • Chunker semantico (LlamaIndex logic + custom regex IT)        │
│  • Hybrid Search (dense Qdrant + sparse BM25 Tantivy)            │
│  • Multi-provider LLM switching (Ollama + 13 cloud)              │
│  • Self-RAG Evaluator (citation validation)                      │
│  • Multi-contract comparison engine (async parallel)             │
│  • Streaming SSE verso PHP                                       │
└──┬─────────────┬──────────────┬─────────────┬────────────────────┘
   │             │              │             │
   │    ┌────────▼─────────┐    │    ┌────────▼─────────┐
   │    │  🐍 Python Worker │    │    │  Qdrant 1.12+    │
   │    │  FastAPI (8091)   │    │    │  (porta 6333)    │
   │    ├──────────────────┤    │    ├─────────────────┤
   │    │ RAGFlow DeepDoc  │    │    │ Dense (768 dim)  │
   │    │ PDF/OCR/TSR/DLR  │    │    │ Sparse (BM25)    │
   │    │ BGE Reranker v2  │    │    │ Collection/KB    │
   │    │ KG Extractor     │    │    │                  │
   │    │ (spaCy NER IT)   │    │    │                  │
   │    └──────────────────┘    │    └──────────────────┘
   │                            │
   │    ┌────────▼──────────────▼─────────────────────────┐
   │    │  Ollama (porta 11434)                            │
   │    │  - qwen2.5:7b (chat)                             │
   │    │  - qwen2.5:3b (contextual retrieval)             │
   │    │  - nomic-embed-text (embedding)                  │
   │    └──────────────────────────────────────────────────┘
   │
   └─────────────────────────────────────────────────────────┐
                 │                                            │
   ┌─────────────▼──────────────┐   ┌──────────────▼────────┐
   │  MySQL 8 (porta 3306)       │   │  Redis 7 (porta 6379) │
   │  archivio_parlante_x        │   │  Rate limit + cache   │
   │  - ap_users, ap_documents   │   │                       │
   │  - ap_graph_nodes/edges     │   │                       │
   │  - ap_chat_messages         │   │                       │
   └─────────────────────────────┘   └───────────────────────┘
```

**7 microservizi orchestrati**: Docker Compose con rete interna `archivio_net`

---

## 📐 Design Principles

### 1. Zero Allucinazioni (Non Negoziabile)

**4 tecniche combinate**:

1. **Hybrid Search**: Dense (cosine 768-dim) + Sparse (BM25 Tantivy) → RRF (k=60)
2. **Reranker**: BGE-reranker-v2-m3 cross-encoder (top 30 → top 5)
3. **Contextual Retrieval**: chunk arricchito con summary del documento intero
4. **Self-RAG + Citation**: JSON schema con `text_quote` verbatim obbligatorio, LLM validator verifica grounding

**Fallback**: Se confidence < 0.7 → risposta standard *"Le informazioni richieste non sono presenti nei documenti caricati."* (mai inventare).

---

### 2. Privacy & Zero-Cost Default

- **LLM locale Ollama** di default (dati mai escono dal server)
- **API cloud** disabilitati finché admin non inserisce chiave + alza budget (`DAILY_COST_BUDGET_EUR`)
- **MySQL + Qdrant + Redis** self-hosted (no SaaS obbligatorio)
- **Audit log** completo (chi, cosa, quando) per compliance

---

### 3. Performance via Rust

- **Chunking semantico**: 50-100× più veloce vs Python (tiktoken-rs + regex)
- **Embedding concorrente**: 500+ req/s paralleli verso Ollama (`tokio::join!`)
- **Hybrid search**: BM25 Tantivy nativo Rust (no Elasticsearch overhead)
- **Multi-contract**: 50+ chiamate LLM parallele in streaming (`futures::stream::buffer_unordered`)

---

### 4. Modularità & Best-of-Breed

- **Parser PDF**: riusa RAGFlow DeepDoc (production-ready OCR/TSR/DLR)
- **UI**: adatta layout Open WebUI Svelte (eccellente UX)
- **Chunker**: logica LlamaIndex + custom regex legale italiano
- **Citation**: logica kotaemon (preview + validation)
- **Core orchestrator**: Rust custom (zero lock-in architetturale)

---

## 🛤️ Roadmap Fasi

| Fase | Obiettivo | Effort | Deliverable |
|---|---|---|---|
| **-1** ✅ | Bootstrap repo + ricerca OSS + decision matrix | 1-2 gg | README, LICENSE, docs, ADR, conferma utente |
| **0** | Setup Docker Compose (7 servizi) | 2-3 gg | `make up` → stack completo funzionante |
| **1** | Rust Engine (chunker, hybrid search, multi-provider LLM, Self-RAG) | 8-12 gg | Ingest + query RAG con Ollama locale |
| **2** | Python AI Worker (parser PDF, reranker, KG extraction) | 5-7 gg | Integration Rust ↔ Python |
| **3** | PHP Gateway (auth, RBAC, rate limit, audit, proxy) | 3-5 gg | Login + JWT + RBAC |
| **4** | Frontend React (UI document-centric, chat, compare) | 6-8 gg | UI completa + SSE streaming |
| **5** | Testing, benchmark, hardening, security audit | 5-7 gg | CI verde, coverage >80%, CVE=0, KPI verificati |

**Totale stima**: **30-44 giorni** (vs 40-60 from-scratch, vs 8-12 clone Onyx ma con compromessi quality).

---

## 📊 Success Criteria

### MVP (Fine Fase 4)

- [ ] Utente può uploadare contratto PDF italiano
- [ ] Sistema indicizza con OCR + chunking semantico + contextual retrieval
- [ ] Utente può fare query RAG con risposta + citation verbatim
- [ ] Utente può confrontare 10+ contratti in parallelo
- [ ] LLM locale Ollama funziona (zero-cost)
- [ ] Self-RAG valida risposta (hallucination check)

### Production-Ready (Fine Fase 5)

- [ ] Tutti i KPI SLO verificati (latency, accuracy, hallucination, cost)
- [ ] Coverage test > 80% (Rust/Python/PHP), > 70% (Frontend)
- [ ] Zero CVE High/Critical
- [ ] RBAC funzionante (multi-utente con permessi)
- [ ] Audit log completo
- [ ] Documentazione completa (README, ARCHITECTURE, RUNBOOK, ADR)
- [ ] CI/CD pipeline funzionante

---

**Ultimo aggiornamento**: 2026-04-21 — Fase -1, Step 5
