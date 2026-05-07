# 📊 Archivio Parlante — Benchmark Suite

Suite completa di benchmark per validare performance, accuracy e zero-hallucination del sistema RAG.

## Requisiti

```bash
pip install -r requirements.txt
```

**Note**: Richiede stack completo attivo (`make up`).

## Benchmark Disponibili

### 1. Ingest Benchmark
```bash
python ingest_bench.py
```
**Misura**: throughput ingestion PDF, latency p50/p95/p99, memoria/CPU/VRAM.  
**Target**: <30s per PDF 50 pagine, throughput ≥10 PDF/min paralleli.  
**Output**: `reports/ingest_YYYYMMDD.md`

### 2. Query Benchmark
```bash
python query_bench.py
```
**Misura**: latency query RAG, recall@5, keyword coverage su 100 query gold-set.  
**Target**: p95 <3s con qwen2.5:7b locale.  
**Output**: `reports/query_YYYYMMDD.md`

### 3. Hallucination Evaluation
```bash
python hallucination_eval.py
```
**Misura**: tasso allucinazioni su 30 trick questions + 30 query valide.  
**Target**: <1% hallucination rate, precision/recall >90%.  
**Output**: `reports/hallucination_YYYYMMDD.md`

### 4. Concurrent Benchmark
```bash
python concurrent_bench.py
```
**Misura**: throughput e tail latency con 50 query simultanee.  
**Target**: throughput >5 req/s, p99 <5s.  
**Output**: `reports/concurrent_YYYYMMDD.md`

### 5. Load Test (k6)
```bash
cd k6 && k6 run load_test.js
```
**Misura**: ramping 10→100 VU, stability test.  
**Target**: p95 <3s, error rate <0.1%.

## Orchestrazione

Esegui tutti i benchmark in sequenza:
```bash
make bench-all
```

**Tempo stimato**: <30 minuti  
**Output**: Report HTML consolidato in `reports/benchmark_summary.html`

## Fixtures

### Generate Fake Contracts
```bash
cd fixtures && python generate_contracts.py --count 50
```
Genera 50 PDF fittizi di contratti con testo realistico in italiano (NDA, appalti, forniture).

### Query Gold-Set
File `fixtures/queries.jsonl` contiene 100 query con:
- `question`: domanda in italiano
- `expected_doc_ids`: documenti che dovrebbero essere citati
- `expected_keywords`: parole chiave attese nella risposta

## KPI Target (Fase 5)

| Metrica | Target | Attuale |
|---|---|---|
| Ingest 50-page PDF | <30s | TBD |
| Query p95 (locale) | <3s | TBD |
| Hallucination rate | <1% | TBD |
| Recall@5 | >95% | TBD |
| Concurrent throughput | >5 req/s | TBD |

## Troubleshooting

**Errore: Connection refused**
→ Verifica stack: `make health`

**Benchmark lenti**
→ Chiudi altre applicazioni, libera VRAM GPU

**Fixture generation fails**
→ Installa `reportlab`: `pip install reportlab`
