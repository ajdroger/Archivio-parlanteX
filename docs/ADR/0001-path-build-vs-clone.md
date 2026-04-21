# ADR 0001: Approccio Architetturale — Ibrido vs Clone vs From-Scratch

**Status**: ✅ **Accepted** (confermato dall'utente 2026-04-21)  
**Data**: 2026-04-21  
**Deciders**: Claude Code (Senior Solutions Architect), Utente (Product Owner)  
**Context**: Fase -1, valutazione approcci costruzione per Archivio Parlante RAG system

---

## Context

### Problema

Stiamo costruendo **Archivio Parlante**, un sistema RAG enterprise greenfield per analisi forense di contratti italiani con zero allucinazioni, destinato a enti istituzionali ad alto rischio reputazionale.

**Requisiti chiave**:
1. Zero allucinazioni (4 tecniche: Hybrid Search, Reranker, Contextual Retrieval, Self-RAG + Citation, KG)
2. Performance: query < 500ms p95, 50+ contratti paralleli < 2s
3. Privacy: LLM locale Ollama di default (zero-cost)
4. Self-hosted: MySQL + Qdrant + Redis on-premise
5. Knowledge Graph legale (PARTI, DATE, IMPORTI, CLAUSOLE, GIURISDIZIONI)
6. Multi-provider LLM (Ollama + 13 cloud opt-in)

**Constraint hardware**: RTX 4070 Laptop **8 GB VRAM** (max modelli 14B in Q4)

### Ricerca Condotta

**10 framework RAG OSS analizzati** (vedi `docs/02-oss-research-report.md`):

| Framework | Stars | Licenza | Coverage Requisiti (/15) | Note |
|---|---|---|---|---|
| Verba | 7.7k | BSD-3 | 6/15 | Accoppiato Weaviate |
| Quivr | 39.1k | Apache 2.0 | 8/15 | No KG, refactor recente |
| kotaemon | 25.3k | Apache 2.0 | 9/15 | **Citation top**, performance bassa |
| Onyx/Danswer | 27.8k | MIT | 10/15 | Enterprise, ma Python core |
| AnythingLLM | 58.7k | MIT | 6/15 | Consumer-grade, Node.js |
| Open WebUI | 133k 🏆 | Multi | 9/15 | **UI eccellente**, chat-centric |
| Cheshire Cat | 3k | GPL-3 ⚠️ | 5/15 | Licenza copyleft blocker |
| Haystack | 24.9k | Apache 2.0 | 11/15 | **Framework top**, no UI |
| RAGFlow | 78.6k 🥈 | Apache 2.0 | 10/15 | **Parser PDF/OCR top** |
| LlamaIndex | 48.7k 🥉 | MIT | 11/15 | **Ecosystem enorme**, no UI |

**Nessun framework copre 100% requisiti**. I migliori (Haystack, LlamaIndex) sono **framework** (no app end-to-end). I migliori **app** (Onyx, RAGFlow, kotaemon) mancano di Self-RAG + KG legale + Rust core.

### Opzioni Valutate

**Tre approcci**:

1. **Clone + Adattamento**: Clonare Onyx/Danswer (27.8k⭐, MIT, enterprise-ready) e adattare per requisiti specifici
   - Effort: 8-12 giorni
   - Pro: 80% lavoro fatto, architettura enterprise
   - Contro: Python core (non Rust), lock-in architetturale, no Self-RAG built-in

2. **Ibrido (Best-of-Breed)**: Riusare componenti migliori + core Rust custom
   - Parser PDF: RAGFlow DeepDoc (OCR/TSR/DLR production-ready)
   - UI: Open WebUI layout Svelte (adattato)
   - Chunker: LlamaIndex logic + regex custom italiano
   - Hybrid search: kotaemon retriever modules
   - Core: Rust (Axum + Tokio) orchestrator custom
   - Effort: 15-23 giorni
   - Pro: Flessibilità, Rust performante, zero lock-in
   - Contro: Assembly multi-framework, integration complexity

3. **From-Scratch**: Costruire tutto da zero (Rust + Python + PHP + React)
   - Effort: 40-60 giorni
   - Pro: Controllo totale, learning value alto
   - Contro: Doppio tempo vs Ibrido, reinventa parsing PDF/OCR già risolto

---

## Decision

**Approccio scelto**: **Opzione B — Ibrido (Best-of-Breed Components + Core Rust Custom)**

**Punteggio Decision Matrix**: 4.30/5.00 (vs Clone 3.65, vs From-scratch 4.00)

### Componenti Riusati

| Componente | Framework Fonte | Cosa Riusare | Integration |
|---|---|---|---|
| **Parser PDF/OCR** | RAGFlow DeepDoc | Modulo Python con OCR + TSR (Table Structure Recognition) + DLR (Document Layout Recognition) | Python worker FastAPI, chiamato da Rust via HTTP |
| **UI Layout** | Open WebUI | Componenti Svelte (layout document-centric, chat, sidebar) | Adattato in React + TailwindCSS per coerenza stack |
| **Chunker Semantico** | LlamaIndex | Logic chunking + regex custom per contratti italiani | Port in Rust (tiktoken-rs + regex) |
| **Citation Logic** | kotaemon | Citation tracking + preview + validation | Port in Rust (logic Self-RAG) |
| **Hybrid Search** | kotaemon / Haystack | Retriever modules (dense + sparse + RRF) | Implementato in Rust con qdrant-client + Tantivy BM25 |

### Core Custom in Rust

**Tutto il core orchestrator è Rust nativo**:

- **Axum web server** (porta 8090)
- **Tokio async runtime** (parallelismo I/O)
- **Multi-provider LLM switching** (Ollama + 13 cloud via trait `LlmProvider`)
- **Self-RAG Evaluator** (citation validation, grounding check, hallucination detection)
- **Knowledge Graph extraction coordinator** (chiama Python worker spaCy NER, salva in MySQL)
- **Hybrid search orchestrator** (combina dense Qdrant + sparse BM25 Tantivy, RRF fusion)
- **Multi-contract comparison engine** (50+ query parallele in streaming)

---

## Consequences

### ✅ Positive

1. **Performance target garantite**: Rust core → latency p95 < 500ms, 50+ contratti < 2s (benchmark Fase 5).

2. **Best-of-breed components**: Riusa solo "ciò che funziona bene" (RAGFlow parser = production-ready OCR/TSR, Open WebUI UI = eccellente UX) senza compromessi.

3. **Zero vendor lock-in**: Ogni componente riusato è sostituibile. Se RAGFlow parser diventa problema → swap con Unstructured o custom. Architettura modulare.

4. **Controllo totale su anti-allucinazione**: Self-RAG evaluator custom in Rust, citation validation custom, Knowledge Graph custom per entità legali italiane. Nessuna dipendenza da logica opinionated di framework terzi.

5. **Time-to-MVP bilanciato**: ~15-23 giorni (60% tempo vs From-scratch 40-60gg), ma senza compromessi quality/performance come Clone Onyx.

6. **Aderenza a principi progetto**:
   - ✅ Zero-Cost: componenti OSS con licenze compatibili (MIT, Apache 2.0)
   - ✅ Open Source First: riusa 4-5 framework invece di reinventare
   - ✅ Performance: Rust core garantisce KPI
   - ✅ Coerenza stack: mantiene Rust+Python+PHP+React come da piano originale

### ⚠️ Negative (Mitigabili)

1. **Integration complexity**: Assembly componenti da 4-5 framework diversi.
   - **Mitigazione**: Test integration rigorosi (Fase 1.x), isolamento chiaro worker Python / core Rust, interface HTTP/gRPC ben definite.

2. **Multi-codebase maintenance**: Rust + Python + PHP + React/Svelte.
   - **Mitigazione**: Skill già definiti (`.claude/skills/archivio-parlante-dev`), CI automation per tutti layer, documentation inline obbligatoria.

3. **Dipendenza upstream**: RAGFlow parser, Open WebUI UI components.
   - **Mitigazione**: Fork locale dei componenti critici (RAGFlow parser), pin version, monitoring upstream per breaking changes.

4. **Learning curve**: Team deve conoscere Rust + Python + PHP + Svelte/React.
   - **Mitigazione**: Stack già nel piano originale, skills Claude Code per onboarding, documentazione ADR + ARCHITECTURE.md.

### 🚫 Risks Rejected Options

**Se avessimo scelto Clone Onyx (A)**:
- ❌ Python core (non Rust) → performance target non garantite (Rust 50-100× più veloce su chunking)
- ❌ Lock-in architetturale Onyx (Celery, Elasticsearch) → difficile deviare
- ❌ No Self-RAG built-in → requisito critico per zero-hallucination

**Se avessimo scelto From-Scratch (C)**:
- ❌ Doppio tempo (40-60gg vs 15-23gg)
- ❌ Reinventare parsing PDF/OCR → RAGFlow DeepDoc è production-ready e top-tier (OCR + TSR + DLR), perché riscriverlo?
- ❌ Reinventare UI → Open WebUI ha UI eccellente, perché rifare da zero?

---

## Validation

### Acceptance Criteria Opzione B

- [ ] Parser RAGFlow DeepDoc integrato in Python worker (Fase 2.1)
- [ ] Layout UI Open WebUI adattato in React (Fase 4.1)
- [ ] Chunker semantico LlamaIndex portato in Rust (Fase 1.2)
- [ ] Hybrid search Rust con Qdrant + Tantivy funzionante (Fase 1.3)
- [ ] Self-RAG evaluator Rust con citation validation (Fase 1.4)
- [ ] Knowledge Graph legale custom in MySQL (Fase 2.3)
- [ ] Multi-provider LLM switching Rust (Ollama + 13 cloud) (Fase 1.1)
- [ ] Tutti i KPI SLO verificati (Fase 5): latency p95 < 500ms, accuracy recall@10 > 95%, hallucination < 1%

### Fallback Plan

**Se approccio Ibrido fallisce** (es. integration troppo complessa, performance insoddisfacenti):
- **Fallback 1**: Pivotare a Clone Onyx (Opzione A) e accettare trade-off Python core
- **Fallback 2**: Pivotare a From-Scratch (Opzione C) e accettare doppio tempo

**Trigger per fallback**: Fine Fase 1 (Rust Engine) → se latency p95 > 1000ms o integration con Python worker fallisce ripetutamente.

---

## Related Documents

- `docs/00-decision-matrix.md` — Analisi quantitativa 3 approcci con punteggi ponderati
- `docs/01-architecture-vision.md` — Problem statement, stakeholder, constraint, SLO
- `docs/02-oss-research-report.md` — Ricerca dettagliata 10 framework RAG OSS
- `implementation_plan.md` — Piano maestro Fase 0-5

---

## Notes

Questa decisione è stata presa in **Fase -1** (bootstrap repository) prima di scrivere qualsiasi codice applicativo. È **reversibile fino a fine Fase 1** (Rust Engine) — se l'approccio Ibrido si rivela inadeguato, possiamo pivotare a Clone o From-Scratch con effort incrementale accettabile.

La decisione finale richiede **conferma esplicita utente** (Task #7 — AskUserQuestion) prima di procedere a Fase 0.

---

**Decider finale**: Utente (Product Owner) — **Opzione B (Ibrido) confermata**  
**Implementation start**: Fase 0 (Setup Docker Compose)

---

**Ultimo aggiornamento**: 2026-04-21 — Fase -1, Step 6
