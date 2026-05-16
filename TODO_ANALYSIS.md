# TODO Analysis - Completamento al 100%

## ✅ ALREADY IMPLEMENTED (da rimuovere)

1. **compare.rs:22** - "Optionally save to database (TODO: Phase 1.6)"
   - ✅ GIÀ IMPLEMENTATO (righe 66-91)
   - Action: Rimuovere TODO obsoleto

## 🔴 CRITICAL (bloccano produzione)

Nessuno trovato - sistema funzionante!

## 🟡 MEDIUM (features dichiarate ma incomplete)

2. **registry.rs:34** - "Add cloud providers if API keys present"
   - Status: Commented code presente
   - Action: Implementare registrazione condizionale provider cloud

3. **kb.rs:92** - "Add KB access control check here"
   - Status: Placeholder per Fase 6.3
   - Action: Implementare check workspace membership

4. **ingest.rs:114** - "Extract knowledge graph (Phase 1.4)"
   - Status: Step 7 commentato
   - Action: Implementare chiamata graph extraction service

5. **kb.rs:181** - "Query MySQL ap_graph_nodes and ap_graph_edges"
   - Status: Placeholder response
   - Action: Implementare query reale

6. **kb.rs:271** - "Implement background job"
   - Status: Placeholder response per reindex
   - Action: Implementare job queue con tokio task

## 🟢 LOW (polish, non-bloccanti)

7. **qdrant.rs:71** - "Add sparse vector support via named vectors"
   - Status: Dense-only funziona, sparse fallback attivo
   - Action: Documentare decisione di rimandare a Fase 2

8. **docs.rs:22** - "Add path annotations to handlers"
   - Status: OpenAPI paths senza description
   - Action: Aggiungere description/summary ai path

9. **rate_limit.rs:136** - "Extract from connection peer address"
   - Status: Hardcoded IP fallback
   - Action: Extract real IP da ConnectInfo

10. **main.py:53** - "Fase 2.3+: Initialize spaCy NER"
    - Status: Placeholder per future NER
    - Action: Documentare requisiti Fase 2.3

11. **WorkspaceSwitcher.tsx:59,64** - "Fase 6.3: Workspace modals"
    - Status: Placeholder onClick handlers
    - Action: Implementare modals

## 📋 Strategia di Completamento

**Ordine di implementazione**:
1. Rimuovere TODO obsoleti (compare.rs)
2. Implementare MEDIUM priority (registry, kb access, graph, reindex)
3. Implementare LOW priority (docs, rate_limit, UI)
4. Documentare future features (sparse vectors, NER)

**Tempo stimato**: ~2-3 ore per tutto
