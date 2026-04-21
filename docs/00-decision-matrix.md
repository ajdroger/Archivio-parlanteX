# Decision Matrix — Approccio di Costruzione

**Data decisione**: 2026-04-21  
**Context**: Fase -1, valutazione approcci architetturali per Archivio Parlante

---

## 🎯 Candidati Valutati

| Opzione | Descrizione | Framework Base |
|---|---|---|
| **A. Clone + Adattamento** | Clonare Onyx/Danswer e adattare per requisiti specifici | Onyx (27.8k⭐, MIT) |
| **B. Ibrido** | Riusare componenti best-of-breed + core Rust custom | RAGFlow parser + Open WebUI UI + LlamaIndex chunker + Haystack retriever |
| **C. From-Scratch** | Costruire tutto da zero secondo piano originale | Rust + Python + PHP + React (greenfield puro) |

---

## 📐 Dimensioni di Valutazione

| Dimensione | Peso | Descrizione |
|---|---|---|
| **Time-to-MVP** | 25% | Tempo per rilasciare MVP funzionante (giorni di sviluppo) |
| **Aderenza Zero-Hallucination** | 20% | Capacità di implementare 4 tecniche anti-allucinazione (Hybrid Search, Reranker, Contextual Retrieval, Self-RAG, KG) |
| **Performance 50+ contratti** | 15% | Scalabilità e performance su query multi-contratto parallele |
| **Manutenibilità lungo termine** | 15% | Facilità di aggiornamento, debug, estensione |
| **Fit Hardware 8 GB VRAM** | 10% | Allineamento con vincolo RTX 4070 8 GB VRAM |
| **Coerenza Stack Rust** | 10% | Aderenza a piano originale (Rust core) |
| **Licenza / Compliance** | 5% | Compatibilità licenze, assenza vendor lock-in |
| **TOTALE** | 100% | |

---

## 📊 Punteggio Dimensioni (Scala 1-5)

| Dimensione | Peso | Clone Onyx (A) | Ibrido (B) | From-Scratch (C) | Note |
|---|---|---|---|---|---|
| **Time-to-MVP** | 25% | **5** | **4** | **2** | Onyx 80% fatto, Ibrido assembly ~15-23gg, From-scratch ~40-60gg |
| **Zero-Hallucination** | 20% | **3** | **5** | **5** | Onyx ha base hybrid search ma no Self-RAG built-in; Ibrido e From-scratch full control |
| **Performance 50+ contratti** | 15% | **4** | **5** | **5** | Onyx buono ma Python bottleneck; Ibrido e From-scratch con Rust core = ottimale |
| **Manutenibilità** | 15% | **3** | **3** | **4** | Onyx lock-in architetturale; Ibrido multi-codebase complesso; From-scratch controllo totale |
| **Fit 8 GB VRAM** | 10% | **4** | **5** | **5** | Tutti possono usare Ollama locale; Ibrido e From-scratch più controllo |
| **Coerenza Stack Rust** | 10% | **2** | **4** | **5** | Onyx è Python+TS; Ibrido ha Rust core; From-scratch Rust nativo |
| **Licenza** | 5% | **4** | **5** | **5** | Onyx MIT OK; Ibrido mix MIT/Apache OK; From-scratch controllo totale |

---

## 🧮 Punteggio Totale Ponderato

| Opzione | Calcolo | **Punteggio Finale** | Rank |
|---|---|---|---|
| **A. Clone Onyx** | (5×0.25) + (3×0.20) + (4×0.15) + (3×0.15) + (4×0.10) + (2×0.10) + (4×0.05) | **3.65** | 🥉 Terzo |
| **B. Ibrido** | (4×0.25) + (5×0.20) + (5×0.15) + (3×0.15) + (5×0.10) + (4×0.10) + (5×0.05) | **4.30** | 🥇 **Primo** |
| **C. From-Scratch** | (2×0.25) + (5×0.20) + (5×0.15) + (4×0.15) + (5×0.10) + (5×0.10) + (5×0.05) | **4.00** | 🥈 Secondo |

---

## 🏆 Raccomandazione

**Approccio raccomandato**: **Opzione B — Ibrido**

**Punteggio**: 4.30/5.00

**Motivazione**:

1. **Best-of-breed components**: Riusa solo "ciò che funziona bene" da framework maturi (RAGFlow parser PDF/OCR, Open WebUI UI Svelte, LlamaIndex chunker semantico, kotaemon citation logic) senza lock-in architetturale.

2. **Rust core performante**: Mantiene l'architettura Rust+Python+PHP del piano originale, garantendo performance target (p95 < 500ms, 50+ contratti paralleli < 2s).

3. **Controllo totale su anti-allucinazione**: Implementazione custom delle 4 tecniche (Hybrid Search con Tantivy BM25, Self-RAG evaluator Rust, Contextual Retrieval, Knowledge Graph legale) senza dipendere da logica opinionated di framework terzi.

4. **Time-to-MVP bilanciato**: ~15-23 giorni (60% tempo vs From-scratch), ma senza compromessi su quality/performance come Clone Onyx.

5. **Zero vendor lock-in**: Ogni componente riusato è sostituibile. Se RAGFlow parser diventa problema → swap con Unstructured o custom. Se Open WebUI UI non scala → swap con React custom.

6. **Fit con principi del progetto**:
   - ✅ Zero-Cost: tutti componenti OSS con licenze compatibili (MIT, Apache 2.0)
   - ✅ Open Source First: riusa 4-5 framework esistenti invece di reinventare
   - ✅ Performance: Rust core garantisce target KPI
   - ✅ Coerenza stack: mantiene Rust+Python+PHP come da piano

---

## ⚠️ Rischi Opzione B (Ibrido)

| Rischio | Probabilità | Impatto | Mitigazione |
|---|---|---|---|
| **Integration complexity** | Media | Medio | Test integration rigorosi, isolamento chiaro worker Python / core Rust |
| **Multi-codebase maintenance** | Media | Basso | Skill già definiti per Rust/Python/PHP, CI automation per tutti layer |
| **Dipendenza upstream** | Bassa | Basso | Fork locale dei componenti critici (RAGFlow parser), pin version |
| **Performance overhead** | Bassa | Medio | Profiling continuo (flamegraph), benchmark suite in Fase 5 |

**Tutti i rischi sono accettabili** e mitigabili con pratiche engineering standard (test, monitoring, profiling).

---

## 📉 Perché NON Opzione A (Clone Onyx)?

| Problema | Severità |
|---|---|
| Python core (non Rust) | ⚠️ Alta — performance target non garantite (Rust 50-100× più veloce) |
| Lock-in architetturale Onyx | ⚠️ Media — difficile deviare da architettura Celery+Elasticsearch |
| No Self-RAG built-in | ⚠️ Alta — requisito critico per zero-hallucination |
| Setup complesso (Celery, Elasticsearch, multi-container) | ⚠️ Bassa — gestibile ma overhead |

**Verdetto**: Clone Onyx è **buon fallback** se Ibrido fallisce, ma non prima scelta.

---

## 📈 Perché NON Opzione C (From-Scratch)?

| Considerazione | Analisi |
|---|---|
| Time-to-MVP ~40-60 giorni | 🔴 Doppio rispetto a Ibrido (15-23gg) |
| Reinventare parsing PDF/OCR | 🔴 RAGFlow DeepDoc è production-ready e top-tier (OCR + TSR + DLR), perché riscriverlo? |
| Reinventare UI componenti | 🔴 Open WebUI ha UI eccellente e Svelte ben fatto, perché rifare da zero? |
| Learning value alto | 🟢 Pro, ma non giustifica 2× tempo in contesto enterprise |

**Verdetto**: From-scratch è **perfetto per learning project**, ma non ottimale per enterprise time-to-market.

---

## 🎯 Escalation all'Utente (Task #7)

Le domande da porre via AskUserQuestion:

1. **Confermi Opzione B (Ibrido)?** Trade-off: assembly multi-framework (15-23gg) ma Rust core performante + zero lock-in.

2. **Alternative accettabili**:
   - **Opzione A (Clone Onyx)**: più veloce (8-12gg) ma Python core + lock-in architetturale.
   - **Opzione C (From-scratch)**: controllo totale ma doppio tempo (40-60gg).

3. **Vuoi rivedere la decisione?** Se hai dubbi su componenti specifici (es. preferisci scrivere parser PDF custom invece di riusare RAGFlow), possiamo ridiscutere.

---

## 📝 Prossimo Step

**Se utente conferma Opzione B (Ibrido)**:
- Creare ADR `0001-path-build-vs-clone.md` documentando la decisione
- Procedere a Fase 0 con setup Docker Compose secondo piano ibrido
- In Fase 1.1, integrare componenti riusati (RAGFlow parser, LlamaIndex chunker) come Python workers

**Se utente sceglie Opzione A o C**:
- Aggiornare ADR di conseguenza
- Adattare Fase 0 al percorso scelto

---

**Ultimo aggiornamento**: 2026-04-21 — Fase -1, Step 5
