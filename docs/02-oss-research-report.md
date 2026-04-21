# Report Ricerca OSS RAG Frameworks — Archivio Parlante

**Data ricerca**: 2026-04-21  
**Scopo**: Valutare framework RAG open-source esistenti per decidere tra Clone/Ibrido/From-scratch

---

## 📊 Executive Summary

**Framework analizzati**: 10 (Verba, Quivr, kotaemon, Onyx, AnythingLLM, Open WebUI, Cheshire Cat, Haystack, RAGFlow, LlamaIndex)

**Risultato**: Nessun framework copre al 100% i requisiti **specifici** del progetto (contratti italiani, hybrid search, multi-provider LLM, knowledge graph legale, citation enforcement). I migliori candidati sono:

1. **RAGFlow** (78.6k⭐) — parsing PDF/OCR avanzato, ma UI opinionated
2. **Haystack** (24.9k⭐) — framework modulare, ma richiede assembly manuale
3. **kotaemon** (25.3k⭐) — citation-first, ma performance limitata
4. **LlamaIndex** (48.7k⭐) — ecosistema enorme, ma framework generico (non app)

**Raccomandazione preliminare**: **Approccio ibrido** (riusa parser RAGFlow + UI Open WebUI + chunker LlamaIndex) + **core Rust custom** per hybrid search, multi-provider switching, self-RAG, e knowledge graph legale.

---

## 📋 Tabella Comparativa

| Framework | Repo | ⭐ Stars | Licenza | Ultimo Release | Ling. | Parsing Contr. IT | Hybrid Search | Multi-Provider LLM | KG Legale | Citation | **Total /15** |
|---|---|---|---|---|---|---|---|---|---|---|
| **Verba** | weaviate/Verba | 7.7k | BSD-3 | Jul 2025 | Py+TS | 1 | 2 | 2 | 0 | 1 | **6** |
| **Quivr** | QuivrHQ/quivr | 39.1k | Apache 2.0 | Feb 2025 | Py+TS | 2 | 2 | 3 | 0 | 1 | **8** |
| **kotaemon** | Cinnamon/kotaemon | 25.3k | Apache 2.0 | Mar 2026 | Py | 2 | 2 | 2 | 0 | **3** | **9** ⭐ |
| **Onyx** | onyx-dot-app/onyx | 27.8k | MIT | Apr 2026 | Py+TS | 2 | **3** | 2 | 1 | 2 | **10** ⭐⭐ |
| **AnythingLLM** | Mintplex-Labs/anything-llm | 58.7k | MIT | Apr 2026 | Node+React | 1 | 1 | 3 | 0 | 1 | **6** |
| **Open WebUI** | open-webui/open-webui | 133k 🏆 | Multi (OW License) | Apr 2026 | Py+Svelte | 2 | 2 | 3 | 0 | 2 | **9** ⭐ |
| **Cheshire Cat** | cheshire-cat-ai/core | 3k | GPL-3 ⚠️ | — | Py | 1 | 1 | 2 | 0 | 1 | **5** |
| **Haystack** | deepset-ai/haystack | 24.9k | Apache 2.0 | Apr 2026 | Py | 2 | 3 | **3** | 1 | 2 | **11** ⭐⭐⭐ |
| **RAGFlow** | infiniflow/ragflow | 78.6k 🥈 | Apache 2.0 | Apr 2026 | Py+TS | **3** | 2 | 2 | 1 | 2 | **10** ⭐⭐ |
| **LlamaIndex** | run-llama/llama_index | 48.7k 🥉 | MIT | Apr 2026 | Py | 3 | 2 | 3 | 1 | 2 | **11** ⭐⭐⭐ |

**Legenda punteggi** (0-3):
- **0**: Non supportato o minimo
- **1**: Base / richiede estensione significativa
- **2**: Buono / adattabile con sforzo medio
- **3**: Ottimo / quasi plug-and-play

---

## 🔍 Analisi Dettagliata per Framework

### 1. Verba (weaviate/Verba)

| Campo | Valore |
|---|---|
| **Repo** | [weaviate/Verba](https://github.com/weaviate/Verba) |
| **Stars** | 7.7k ⭐ |
| **Licenza** | BSD-3-Clause ✅ |
| **Ultimo release** | v2.1.3 (Jul 2025, ~9 mesi fa) |
| **Linguaggio** | Python + TypeScript (frontend) |
| **Weaviate-native** | Sì (accoppiamento stretto) |

**Coverage requisiti**:
- 🟡 **Parsing contratti IT** (1/3): SimpleReader, PathReader, PDFReader base — no OCR avanzato, no tabelle
- 🟢 **Hybrid search** (2/3): Dense + BM25, ma solo su Weaviate (noi usiamo Qdrant)
- 🟢 **Multi-provider LLM** (2/3): Anthropic, Cohere, OpenAI, Ollama — no switching runtime dinamico
- 🔴 **Knowledge graph legale** (0/3): Non supportato
- 🟡 **Citation enforcement** (1/3): Citation base, no validation forte

**Effort adattamento**: **15-20 giorni**
- Portare da Weaviate a Qdrant (riscrittura retriever)
- Aggiungere OCR + parsing tabelle
- Implementare KG extraction
- Aggiungere Self-RAG evaluator

**Blocker**:
- Accoppiamento Weaviate molto forte (modularità bassa)
- UI opinionated (non facilmente separabile da backend)

**Raccomandazione**: ❌ **Non usare**. Troppo accoppiato a Weaviate, sforzo di port non giustificato.

---

### 2. Quivr (QuivrHQ/quivr)

| Campo | Valore |
|---|---|
| **Repo** | [QuivrHQ/quivr](https://github.com/QuivrHQ/quivr) |
| **Stars** | 39.1k ⭐⭐ |
| **Licenza** | Apache 2.0 ✅ |
| **Ultimo release** | v0.0.33 (Feb 2025, ~2 mesi fa) |
| **Linguaggio** | Python (FastAPI) + TypeScript (Next.js) |
| **Megaparse** | Integrato (parsing avanzato) |

**Coverage requisiti**:
- 🟢 **Parsing contratti IT** (2/3): Megaparse integrato (PDF, TXT, Markdown, custom parsers) — buono ma non specifico legale
- 🟢 **Hybrid search** (2/3): Dense + sparse via PGVector/Faiss — adattabile a Qdrant
- 🟢 **Multi-provider LLM** (3/3): GPT4, Groq, Llama, Anthropic, Mistral, Gemma — **switching dinamico**
- 🔴 **Knowledge graph legale** (0/3): Non supportato (focus RAG classico)
- 🟡 **Citation enforcement** (1/3): Citation base tramite chunk metadata

**Effort adattamento**: **10-15 giorni**
- Portare da PGVector a Qdrant
- Aggiungere KG extraction per entità legali
- Implementare Self-RAG + citation validation
- Localizzazione UI in italiano
- Rimozione funzionalità non necessarie (multi-brain, integrazioni SaaS)

**Blocker**:
- Architettura monolitica recente (refactor in corso)
- No knowledge graph out-of-the-box

**Raccomandazione**: ⚠️ **Valutare**. Buon candidato per clone, ma richiede rework significativo per KG e citation enforcement.

---

### 3. kotaemon (Cinnamon/kotaemon)

| Campo | Valore |
|---|---|
| **Repo** | [Cinnamon/kotaemon](https://github.com/Cinnamon/kotaemon) |
| **Stars** | 25.3k ⭐⭐ |
| **Licenza** | Apache 2.0 ✅ |
| **Ultimo release** | v0.11.3 (Mar 2026, ~1 mese fa) |
| **Linguaggio** | Python |
| **Citation-first** | ✅ Sì (advanced citations con document preview) |

**Coverage requisiti**:
- 🟢 **Parsing contratti IT** (2/3): Multi-modal QA (PDF con figure + tabelle) — buono
- 🟢 **Hybrid search** (2/3): Hybrid retriever (dense + BM25) + re-ranking — ottimo
- 🟢 **Multi-provider LLM** (2/3): OpenAI, Azure, Ollama, Groq — supporto multi-provider
- 🔴 **Knowledge graph legale** (0/3): Non supportato
- 🟢 **Citation enforcement** (3/3): **★ Advanced citations con preview + validation** — il migliore per citations

**Effort adattamento**: **12-18 giorni**
- Aggiungere KG extraction
- Sostituire Gradio UI con React custom
- Localizzazione italiana
- Aggiungere Self-RAG evaluator
- Migliorare performance (attualmente non ottimizzato per volumi alti)

**Blocker**:
- Performance subottimale per > 1000 documenti (lento)
- UI Gradio non enterprise-grade

**Raccomandazione**: 🟢 **Candidato forte per approccio ibrido**. Riusare modulo **hybrid retriever + citations** (codice Python ben strutturato), scartare UI Gradio, implementare KG custom.

---

### 4. Onyx / Danswer (onyx-dot-app/onyx)

| Campo | Valore |
|---|---|
| **Repo** | [onyx-dot-app/onyx](https://github.com/onyx-dot-app/onyx) (fork di Danswer) |
| **Stars** | 27.8k ⭐⭐ |
| **Licenza** | MIT ✅ (Community Edition) |
| **Ultimo release** | v3.2.4 (Apr 2026, ~6 giorni fa) ✅ |
| **Linguaggio** | Python (FastAPI) + TypeScript (React) |
| **Enterprise-grade** | Sì (RBAC, multi-tenancy, 40+ connectors) |

**Coverage requisiti**:
- 🟢 **Parsing contratti IT** (2/3): 40+ connectors (Slack, GitHub, Confluence, Google Drive, file upload) — generico non specifico legale
- 🟢 **Hybrid search** (3/3): **★ Best-in-class hybrid search + KG** — ottimo
- 🟢 **Multi-provider LLM** (2/3): OpenAI, Anthropic, Azure, Cohere — supporto multi-provider (meno ampio di Quivr/LlamaIndex)
- 🟡 **Knowledge graph legale** (1/3): Knowledge graph integrato, ma generico (non legale-specific)
- 🟢 **Citation enforcement** (2/3): Citation + grounding validation

**Effort adattamento**: **8-12 giorni**
- Aggiungere parser PDF/OCR specifico contratti
- Estendere KG per entità legali (PARTI, DATE, IMPORTI, CLAUSOLE, GIURISDIZIONI)
- Localizzazione UI italiano
- Rimozione connectors non necessari
- Aggiungere Self-RAG evaluator

**Blocker**:
- Complessità setup (Docker multi-container, Celery workers, Elasticsearch/Qdrant)
- Enterprise features in EE (self-hosting CE OK, ma alcune features avanzate paywall)

**Raccomandazione**: 🟢 **Candidato forte per clone**. Architettura enterprise, hybrid search eccellente, MIT license. Se clone questo, abbiamo l'80% del lavoro fatto — ma complessità alta e lock-in architetturale.

---

### 5. AnythingLLM (Mintplex-Labs/anything-llm)

| Campo | Valore |
|---|---|
| **Repo** | [Mintplex-Labs/anything-llm](https://github.com/Mintplex-Labs/anything-llm) |
| **Stars** | 58.7k ⭐⭐⭐ |
| **Licenza** | MIT ✅ |
| **Ultimo release** | v1.12.0 (Apr 2026, ~19 giorni fa) |
| **Linguaggio** | Node.js + React |
| **Desktop app** | Sì (Electron) |

**Coverage requisiti**:
- 🟡 **Parsing contratti IT** (1/3): Document pipelines base — no parsing avanzato PDF legali
- 🟡 **Hybrid search** (1/3): Vector search classico (dense only, no sparse BM25 nativo)
- 🟢 **Multi-provider LLM** (3/3): **★ Ampio supporto** (closed + open-source LLM)
- 🔴 **Knowledge graph legale** (0/3): Non supportato
- 🟡 **Citation enforcement** (1/3): Citation base tramite chunk source

**Effort adattamento**: **15-20 giorni**
- Aggiungere hybrid search (BM25 + dense)
- Implementare parsing PDF/OCR avanzato
- Aggiungere KG extraction
- Aggiungere Self-RAG + citation validation
- Port da Node.js a Rust/Python (se vogliamo mantenere performance target)

**Blocker**:
- Desktop-first (Electron overhead)
- Node.js non ideale per ML/performance critical
- Architettura "consumer-grade" (non enterprise-oriented)

**Raccomandazione**: ❌ **Non usare per clone**. Ottimo per uso personale, ma non enterprise. Stack Node.js non allineato al nostro (Rust+Python). Utile solo per riferimento UI/UX.

---

### 6. Open WebUI (open-webui/open-webui)

| Campo | Valore |
|---|---|
| **Repo** | [open-webui/open-webui](https://github.com/open-webui/open-webui) |
| **Stars** | 133k 🏆 **Più popolare di tutti** |
| **Licenza** | Multi-license (Open WebUI License + historical licenses) ⚠️ |
| **Ultimo release** | v0.9.0 (Apr 2026, oggi) ✅ |
| **Linguaggio** | Python (FastAPI) + Svelte |
| **Ollama-native** | Sì (ma supporta anche OpenAI, Gemini, ecc.) |

**Coverage requisiti**:
- 🟢 **Parsing contratti IT** (2/3): RAG integrato con 9 vector DB, content extraction engines (Tika, Docling, Mistral OCR) — buono
- 🟢 **Hybrid search** (2/3): RAG con retrieval multi-engine — adattabile
- 🟢 **Multi-provider LLM** (3/3): **★ Ollama + OpenAI + Gemini + …** — ottimo
- 🔴 **Knowledge graph legale** (0/3): Non supportato
- 🟢 **Citation enforcement** (2/3): RAG con source citation

**Effort adattamento**: **10-15 giorni**
- Aggiungere KG extraction
- Implementare Self-RAG evaluator
- Localizzazione UI italiana
- Rimozione funzionalità non necessarie (image gen, DALL-E, etc.)
- Valutare licenza (Open WebUI License richiede branding preservation)

**Blocker**:
- Licenza custom (non pura OSS) — richiede review legale
- UI molto "chat-centric" (non document-centric come Verba/kotaemon)
- Feature bloat (image gen, DALL-E, ComfyUI, AUTOMATIC1111 — non necessari per contratti)

**Raccomandazione**: 🟢 **Candidato forte per riuso UI**. UI eccellente, Svelte ben fatto. **Riusare solo frontend** + adattarlo a nostro backend Rust. Evitare clone completo (troppo opinionated verso chat, non document analysis).

---

### 7. Cheshire Cat AI (cheshire-cat-ai/core)

| Campo | Valore |
|---|---|
| **Repo** | [cheshire-cat-ai/core](https://github.com/cheshire-cat-ai/core) |
| **Stars** | 3k ⭐ |
| **Licenza** | GPL-3.0 ⚠️ **Copyleft** |
| **Ultimo release** | Mar 2026 |
| **Linguaggio** | Python (FastAPI) |
| **Founder** | Piero Savastano (Italia) |

**Coverage requisiti**:
- 🟡 **Parsing contratti IT** (1/3): Plugin architecture (estensibile) ma base limitato
- 🟡 **Hybrid search** (1/3): RAG base, no hybrid search nativo
- 🟢 **Multi-provider LLM** (2/3): Multi-provider via plugin
- 🔴 **Knowledge graph legale** (0/3): Non supportato
- 🟡 **Citation enforcement** (1/3): Base citation

**Effort adattamento**: **20-25 giorni**
- Aggiungere hybrid search
- Aggiungere parser PDF/OCR
- Implementare KG extraction
- Aggiungere Self-RAG + citations
- Localizzazione (ironico, founder italiano ma UI inglese)

**Blocker**:
- **Licenza GPL-3** — copyleft, tutto il nostro codice diventa GPL-3 (incompatibile con MIT)
- Community piccola (3k stars vs 78k RAGFlow)
- Meno maturo rispetto ad altri

**Raccomandazione**: ❌ **NON usare**. Licenza GPL-3 è blocker assoluto. Anche se founder italiano, il progetto non offre vantaggi sufficienti per giustificare complessità GPL.

---

### 8. Haystack (deepset-ai/haystack)

| Campo | Valore |
|---|---|
| **Repo** | [deepset-ai/haystack](https://github.com/deepset-ai/haystack) |
| **Stars** | 24.9k ⭐⭐ |
| **Licenza** | Apache 2.0 ✅ |
| **Ultimo release** | v2.28.0 (Apr 2026, ieri) ✅ |
| **Linguaggio** | Python |
| **Tipo** | Framework (non app end-to-end) |

**Coverage requisiti**:
- 🟢 **Parsing contratti IT** (2/3): 300+ integration packages, data loaders per PDF/docs — configurabile
- 🟢 **Hybrid search** (3/3): **★ Pipeline RAG industriali mature** — ottimo, modularità massima
- 🟢 **Multi-provider LLM** (3/3): **★ Model-agnostic** (OpenAI, Mistral, Anthropic, Cohere, HF, Azure, Bedrock, local) — il migliore
- 🟡 **Knowledge graph legale** (1/3): Pipeline configurabile, ma non KG legale out-of-the-box
- 🟢 **Citation enforcement** (2/3): Pipeline citation via retrieval annotations

**Effort adattamento**: **15-25 giorni**
- Costruire app end-to-end (Haystack è solo framework)
- Implementare UI da zero (Haystack non ha UI)
- Configurare pipeline specifiche (chunker, hybrid search, reranker, self-RAG)
- Aggiungere KG extraction pipeline
- Deploy architecture (Haystack non ha deployment built-in)

**Blocker**:
- Non è un'app, è un framework — richiede assembly manuale completo
- Learning curve alta (pipeline complesse, molti concetti custom)
- No UI (bisogna scrivere tutto il frontend)

**Raccomandazione**: 🟢 **Candidato per approccio ibrido**. **Riusare pipeline modules** (retriever, ranker, document store integration) come librerie dentro il nostro Rust backend. NON clone completo (troppo sforzo per costruire app sopra framework).

---

### 9. RAGFlow (infiniflow/ragflow)

| Campo | Valore |
|---|---|
| **Repo** | [infiniflow/ragflow](https://github.com/infiniflow/ragflow) |
| **Stars** | 78.6k 🥈 **Secondo più popolare** |
| **Licenza** | Apache 2.0 ✅ |
| **Ultimo release** | v0.25.0 (Apr 2026, oggi) ✅ |
| **Linguaggio** | Python + TypeScript |
| **Parser PDF** | **★ DeepDoc: parsing avanzato con DL** (OCR, TSR, DLR) |

**Coverage requisiti**:
- 🟢 **Parsing contratti IT** (3/3): **★ DeepDoc engine top-tier** (OCR, table structure, document layout, multi-language) — il migliore per PDF complessi
- 🟢 **Hybrid search** (2/3): RAG con multi-search strategies — buono
- 🟢 **Multi-provider LLM** (2/3): Multi-provider support (non ampio come Haystack)
- 🟡 **Knowledge graph legale** (1/3): Agent capabilities con RAG, ma KG non nativo
- 🟢 **Citation enforcement** (2/3): Citation con source tracking

**Effort adattamento**: **10-15 giorni**
- Aggiungere KG extraction per entità legali
- Implementare Self-RAG evaluator
- Localizzazione UI italiana
- Sostituire vector DB (se usa uno diverso da Qdrant)
- Rimozione agent features non necessari

**Blocker**:
- UI opinionated (document-centric ma layout fisso)
- Architettura monolitica (difficile estrarre solo parser)

**Raccomandazione**: 🟢 **Candidato top per approccio ibrido**. **Riusare DeepDoc parser** (parsing PDF/OCR/TSR) come servizio Python separato. UI meno prioritaria (può essere scartata). Parser è il vero valore aggiunto.

---

### 10. LlamaIndex (run-llama/llama_index)

| Campo | Valore |
|---|---|
| **Repo** | [run-llama/llama_index](https://github.com/run-llama/llama_index) |
| **Stars** | 48.7k 🥉 **Terzo più popolare** |
| **Licenza** | MIT ✅ |
| **Ultimo release** | v0.14.21 (Apr 2026, oggi) ✅ |
| **Linguaggio** | Python |
| **Tipo** | Framework + ecosystem (300+ integration packages) |

**Coverage requisiti**:
- 🟢 **Parsing contratti IT** (3/3): **★ 300+ data loaders** (PDF, docx, SQL, APIs, ecc.) + LlamaParse (parsing avanzato) — ottimo
- 🟢 **Hybrid search** (2/3): IngestionPipeline + multiple retrieval strategies — configurabile
- 🟢 **Multi-provider LLM** (3/3): **★ Ecosystem enorme** (ogni LLM immaginabile) — il migliore
- 🟡 **Knowledge graph legale** (1/3): Knowledge graph support, ma generico (non legale-specific)
- 🟢 **Citation enforcement** (2/3): Source tracking + citation nei retrieval response

**Effort adattamento**: **15-25 giorni**
- Costruire app end-to-end (LlamaIndex è framework)
- Implementare UI da zero
- Configurare pipeline RAG specifiche (chunker custom, hybrid search, reranker, self-RAG)
- Estendere KG per entità legali
- Deploy architecture

**Blocker**:
- Non è un'app, è un framework/ecosystem
- Complessità alta (300+ packages, scelte architetturali multiple)
- No UI (bisogna costruire tutto il frontend)

**Raccomandazione**: 🟢 **Candidato per approccio ibrido**. **Riusare moduli specifici** (data loaders, chunker semantico, LlamaParse per PDF) come librerie Python chiamate dal backend Rust. NON clone completo (troppo effort per costruire app).

---

## 🎯 Riepilogo Raccomandazioni

### ✅ Framework da Riusare (Approccio Ibrido)

| Componente | Framework Fonte | Cosa Riusare | Effort |
|---|---|---|---|
| **Parser PDF/OCR** | **RAGFlow** (DeepDoc) | Modulo Python per parsing PDF avanzato con OCR/TSR/DLR | 3-5 giorni |
| **UI Frontend** | **Open WebUI** | Layout Svelte + componenti chat/document viewer | 5-7 giorni |
| **Chunker semantico** | **LlamaIndex** | IngestionPipeline + semantic chunker | 2-3 giorni |
| **Hybrid search** | **Haystack** / **kotaemon** | Pipeline retriever + re-ranker | 3-5 giorni |
| **Citation logic** | **kotaemon** | Citation tracking + document preview | 2-3 giorni |

**Effort totale approccio ibrido**: **15-23 giorni** (vs from-scratch ~40-60 giorni)

---

### ❌ Framework da NON Clonare

| Framework | Motivo |
|---|---|
| **Verba** | Accoppiamento Weaviate troppo forte |
| **AnythingLLM** | Stack Node.js, non enterprise, consumer-grade |
| **Cheshire Cat** | Licenza GPL-3 (copyleft blockerante) |

---

### 🤔 Framework Candidati per Clone Completo (se non ibrido)

| Framework | Pro | Contro | Effort Clone |
|---|---|---|---|
| **Onyx/Danswer** | Enterprise-ready, hybrid search top, MIT | Setup complesso, architettura monolitica | 8-12 giorni |
| **Quivr** | Multi-provider top, Megaparse integrato, Apache 2.0 | No KG, architettura in refactor | 10-15 giorni |
| **kotaemon** | Citation enforcement migliore, hybrid search buono | Performance bassa, Gradio UI | 12-18 giorni |

---

## 📈 Decisione Finale per Decision Matrix

Sulla base della ricerca, i **3 approcci** da valutare nella Decision Matrix (Task #5) sono:

### Opzione A: Clone + Adattamento

**Framework raccomandato**: **Onyx/Danswer** (27.8k⭐, MIT, enterprise-grade)

**Pro**:
- 80% funzionalità già implementate (hybrid search, multi-provider, RBAC, connectors)
- MIT license (compatibile)
- Architettura enterprise (scalabile, multi-tenancy)
- Effort minore rispetto a from-scratch

**Contro**:
- Setup Docker complesso (Celery, Elasticsearch/Qdrant, multi-container)
- Lock-in architetturale (difficile deviare dall'architettura Onyx)
- No KG legale out-of-the-box (da implementare)
- No Self-RAG built-in (da implementare)
- Python+TypeScript (non Rust core come da piano)

**Effort**: **8-12 giorni** (adattamento + features mancanti)

---

### Opzione B: Ibrido (Riuso Componenti)

**Componenti**:
- Parser PDF: **RAGFlow DeepDoc** (Python worker)
- UI: **Open WebUI** layout Svelte (adattato)
- Chunker: **LlamaIndex** semantic chunker
- Hybrid search: **Haystack** / **kotaemon** retriever modules
- Citation: **kotaemon** citation logic

**Core custom in Rust**:
- Orchestrator async (Tokio)
- Multi-provider LLM switching
- Self-RAG evaluator
- Knowledge graph legale extraction
- Hybrid search coordinator (combina dense Qdrant + sparse BM25 Tantivy)

**Pro**:
- Flessibilità massima (best-of-breed per ogni componente)
- Core Rust performante come da piano originale
- Riuso solo "ciò che funziona bene", scarta il resto
- Nessun lock-in architetturale

**Contro**:
- Effort assembly e integration (componenti da 4-5 framework diversi)
- Testing cross-componenti più complesso
- Manutenzione multi-codebase (Python workers + Rust core + UI Svelte)

**Effort**: **15-23 giorni** (assembly + integration + core Rust custom)

---

### Opzione C: From-Scratch (Piano Originale)

**Architettura**: Rust (Axum) + Python (FastAPI) + PHP (Slim 4) + React (Vite)

**Pro**:
- Controllo totale su architettura
- Performance ottimale (Rust core)
- Zero dipendenze da framework opinionated
- Perfetto allineamento con requisiti specifici
- Learning value alto per team

**Contro**:
- Effort massimo (tutto da scrivere)
- Time-to-MVP più lungo
- Rischio di "reinventare la ruota" per problemi già risolti

**Effort**: **40-60 giorni** (stima piano originale Fase 1-5)

---

## 📚 Sources

### Framework RAG
- [GitHub - weaviate/Verba](https://github.com/weaviate/Verba)
- [GitHub - QuivrHQ/quivr](https://github.com/QuivrHQ/quivr)
- [GitHub - Cinnamon/kotaemon](https://github.com/Cinnamon/kotaemon)
- [GitHub - onyx-dot-app/onyx](https://github.com/onyx-dot-app/onyx)
- [GitHub - Mintplex-Labs/anything-llm](https://github.com/Mintplex-Labs/anything-llm)
- [GitHub - open-webui/open-webui](https://github.com/open-webui/open-webui)
- [GitHub - cheshire-cat-ai/core](https://github.com/cheshire-cat-ai/core)
- [GitHub - deepset-ai/haystack](https://github.com/deepset-ai/haystack)
- [GitHub - infiniflow/ragflow](https://github.com/infiniflow/ragflow)
- [GitHub - run-llama/llama_index](https://github.com/run-llama/llama_index)

### Articoli e Documentazione
- [Verba: Building an Open Source, Modular RAG Application | Weaviate](https://weaviate.io/blog/verba-open-source-rag-app)
- [Meet Quivr: An Open Source RAG Framework with 38k+ Github Stars - MarkTechPost](https://www.marktechpost.com/2024/03/26/meet-quivr-an-open-source-rag-framework-with-38k-github-stars/)
- [Onyx AI | Open Source Enterprise Search & AI Assistant](https://onyx.app/)
- [AnythingLLM Review 2026: Best Free Self-Hosted AI Assistant](https://andrew.ooo/posts/anythingllm-all-in-one-ai-app/)
- [Open WebUI: Self-Hosted AI Platform](https://openwebui.com/)
- [Cheshire Cat AI docs](https://cheshire-cat-ai.github.io/docs/)
- [Haystack | Haystack](https://haystack.deepset.ai/)
- [RAGFlow: Enterprise RAG Engine with 78.3k+ Stars](https://www.decisioncrafters.com/ragflow-enterprise-grade-rag-engine-with-agentic-capabilities-and-78-3k-github-stars/)
- [Introduction to RAG | LlamaIndex OSS Documentation](https://developers.llamaindex.ai/python/framework/understanding/rag/)

---

**Ultimo aggiornamento**: 2026-04-21 — Fase -1, Step 4
