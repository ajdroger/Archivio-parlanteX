# 🔧 Manuale Tecnico Operativo — Archivio Parlante

> **Audience**: DevOps Engineers, System Administrators, Technical Operations  
> **Versione**: v0.7.0  
> **Ultimo aggiornamento**: 2026-05-12

---

## 📋 Indice

1. [Architettura del Sistema](#architettura-del-sistema)
2. [Requisiti Hardware e Software](#requisiti-hardware-e-software)
3. [Installazione e Deployment](#installazione-e-deployment)
4. [Configurazione](#configurazione)
5. [Gestione Servizi](#gestione-servizi)
6. [Monitoring e Observability](#monitoring-e-observability)
7. [Backup e Recovery](#backup-e-recovery)
8. [Troubleshooting](#troubleshooting)
9. [Procedure di Manutenzione](#procedure-di-manutenzione)
10. [Security Hardening](#security-hardening)
11. [Performance Tuning](#performance-tuning)
12. [Disaster Recovery](#disaster-recovery)

---

## 1. Architettura del Sistema

### 1.1 Stack Tecnologico

Archivio Parlante è un sistema RAG (Retrieval-Augmented Generation) enterprise composto da **7 microservizi containerizzati**:

```
┌─────────────────────────────────────────────────────────────┐
│                    CLIENT (Browser)                         │
└─────────────────────────────────────────────────────────────┘
                            │ HTTPS
                            ↓
┌─────────────────────────────────────────────────────────────┐
│              PHP 8.2 Gateway (Slim 4)                       │
│  • Autenticazione JWT                                       │
│  • Rate Limiting (Redis)                                    │
│  • Audit Logging                                            │
│  • Proxy verso Rust Engine                                  │
└─────────────────────────────────────────────────────────────┘
                            │ HTTP (interno)
                            ↓
┌─────────────────────────────────────────────────────────────┐
│           🦀 Rust Core Engine (Axum + Tokio)                │
│  • Chunking semantico                                       │
│  • Hybrid Search (dense + sparse)                           │
│  • Graph RAG (knowledge graph)                              │
│  • Multi-contract comparison                                │
│  • WebSocket collaboration                                  │
└─────────────────────────────────────────────────────────────┘
         │              │              │
         ↓              ↓              ↓
    ┌────────┐    ┌────────┐    ┌────────────────┐
    │ Qdrant │    │ Ollama │    │ 🐍 Python Worker│
    │Vector  │    │  LLM   │    │  • PDF Parsing  │
    │Database│    │ Locale │    │  • OCR          │
    └────────┘    └────────┘    │  • BGE Reranker │
                                 │  • Hallucination│
                                 │    Detection    │
                                 └────────────────┘
         ↓              ↓              ↓
    ┌────────┐    ┌────────┐    ┌────────┐
    │ MySQL  │    │ Redis  │    │ (LLMs) │
    │  8.0   │    │   7    │    │        │
    └────────┘    └────────┘    └────────┘
```

### 1.2 Porte e Networking

| Servizio | Porta Host | Porta Container | Protocollo | Note |
|---|---|---|---|---|
| **php-gateway** | 9080 | 80 | HTTP | API Gateway pubblico |
| **rust-engine** | 8090 | 8090 | HTTP | Core engine (interno) |
| **python-worker** | 8091 | 8091 | HTTP | AI Worker (interno) |
| **mysql** | 3307 | 3306 | MySQL | Database |
| **redis** | 6380 | 6379 | Redis | Cache e pub/sub |
| **qdrant** | 6335 | 6333 | HTTP | Vector DB REST |
| **qdrant-grpc** | 6336 | 6334 | gRPC | Vector DB gRPC |
| **ollama** | 11434 | 11434 | HTTP | LLM locale |
| **grafana** | 3001 | 3000 | HTTP | Monitoring UI (opt) |
| **prometheus** | 9090 | 9090 | HTTP | Metrics (opt) |

**Nota**: Le porte host possono essere modificate in `docker-compose.yml` in caso di conflitti.

### 1.3 Volumi Persistenti

| Volume | Path Container | Descrizione | Backup |
|---|---|---|---|
| `archivio_mysql_data` | `/var/lib/mysql` | Database MySQL | ✅ Critico |
| `archivio_qdrant_storage` | `/qdrant/storage` | Vector embeddings | ✅ Critico |
| `archivio_ollama_models` | `/root/.ollama` | Modelli LLM | ⚠️ Raccomandato |
| `archivio_redis_data` | `/data` | Cache Redis | ❌ Non necessario |
| `./shared/uploads` | `/shared/uploads` | File upload | ✅ Critico |

---

## 2. Requisiti Hardware e Software

### 2.1 Requisiti Minimi (Dev)

- **CPU**: 4 core / 8 thread
- **RAM**: 16 GB
- **Storage**: 50 GB SSD
- **GPU**: Nessuna (solo CPU, performance limitate)
- **OS**: Windows 10/11, macOS 12+, Linux (Ubuntu 22.04+)

### 2.2 Requisiti Raccomandati (Staging/Production)

- **CPU**: Intel i7/i9 o AMD Ryzen 7/9 (8+ core)
- **RAM**: 32 GB DDR4/DDR5
- **Storage**: 200 GB NVMe SSD (IOPS > 10k)
- **GPU**: NVIDIA RTX 3060+ con 8+ GB VRAM (per modelli LLM locali veloci)
- **Network**: 1 Gbps (lan) o 100 Mbps (wan)
- **OS**: Ubuntu Server 22.04 LTS o RHEL 9+

### 2.3 Requisiti Software

| Software | Versione Minima | Comando Verifica |
|---|---|---|
| **Docker Engine** | 24.0+ | `docker --version` |
| **Docker Compose** | 2.20+ | `docker compose version` |
| **Git** | 2.30+ | `git --version` |
| **Make** (opzionale) | 4.0+ | `make --version` |

**Windows**: Docker Desktop 4.25+ (include Docker Compose)  
**Linux**: Installare Docker Engine + plugin Compose separatamente

---

## 3. Installazione e Deployment

### 3.1 Clone Repository

```bash
git clone https://github.com/ajdroger/Archivio-parlanteX.git
cd Archivio-parlanteX
git checkout v0.7.0  # O tag desiderato
```

### 3.2 Configurazione Iniziale

```bash
# 1. Copia file env
cp .env.example .env

# 2. Genera segreti (OBBLIGATORIO)
# Linux/macOS:
echo "JWT_SECRET=$(openssl rand -hex 32)" >> .env
echo "RUST_ENGINE_INTERNAL_TOKEN=$(openssl rand -hex 64)" >> .env

# Windows (PowerShell):
# Add-Content .env "JWT_SECRET=$([System.Convert]::ToHexString([System.Security.Cryptography.RandomNumberGenerator]::GetBytes(32)))"
# Add-Content .env "RUST_ENGINE_INTERNAL_TOKEN=$([System.Convert]::ToHexString([System.Security.Cryptography.RandomNumberGenerator]::GetBytes(64)))"

# 3. Configura password MySQL (PRODUCTION)
# Edita .env manualmente:
# MYSQL_PASSWORD=<password-sicura>
# MYSQL_ROOT_PASSWORD=<password-sicura>
```

### 3.3 Build e Avvio Stack

```bash
# Build tutte le immagini
make setup
# Oppure: docker compose build

# Avvia stack completo
make up
# Oppure: docker compose up -d

# Verifica health
make health
```

**Tempo stimato primo build**: 10-15 minuti (download immagini + compile Rust/Python)

### 3.4 Download Modelli LLM Locali

```bash
# Scarica modelli default (~ 7 GB totale)
make ollama-pull

# Oppure manualmente:
docker exec archivio-ollama ollama pull qwen2.5:7b-instruct-q4_K_M
docker exec archivio-ollama ollama pull qwen2.5:3b-instruct-q4_K_M
docker exec archivio-ollama ollama pull nomic-embed-text
```

**Tempo stimato**: 10-20 minuti (dipende da connessione internet)

### 3.5 Verifica Installazione

```bash
# Health check di tutti i servizi
curl http://localhost:9080/health  # PHP Gateway
curl http://localhost:8090/health  # Rust Engine
curl http://localhost:8091/health  # Python Worker
curl http://localhost:6335/collections  # Qdrant

# Verifica log senza errori
docker compose logs --tail=50

# Swagger UI (documentazione API)
open http://localhost:8090/docs
```

✅ **Installazione completata** se tutti gli health check ritornano `{"status":"ok"}`

---

## 4. Configurazione

### 4.1 Variabili d'Ambiente Critiche

#### Sicurezza (OBBLIGATORIE in produzione)

```env
# JWT per autenticazione utenti
JWT_SECRET=<64-char-hex-string>  # openssl rand -hex 32

# Token interno PHP→Rust
RUST_ENGINE_INTERNAL_TOKEN=<128-char-hex-string>  # openssl rand -hex 64

# Password database
MYSQL_PASSWORD=<secure-password-min-16-chars>
MYSQL_ROOT_PASSWORD=<different-secure-password>
```

⚠️ **MAI committare** questi valori in git. Usare secret manager in produzione (AWS Secrets Manager, HashiCorp Vault, etc.)

#### Database

```env
MYSQL_HOST=mysql              # Nome servizio Docker (non localhost)
MYSQL_PORT=3306               # Porta interna container
MYSQL_DB=archivio_parlante_x  # Nome database (FISSO, non modificare)
MYSQL_USER=root               # User (produzione: creare user dedicato)
```

#### LLM Provider (Zero-cost default)

```env
# Ollama locale (default, GRATIS)
OLLAMA_URL=http://ollama:11434
OLLAMA_MODEL_CHAT=qwen2.5:7b-instruct-q4_K_M
OLLAMA_MODEL_EMBED=nomic-embed-text

# Cloud providers (OPT-IN, a pagamento)
ANTHROPIC_API_KEY=          # Lasciare vuoto per disabilitare
OPENAI_API_KEY=             # Lasciare vuoto per disabilitare
GOOGLE_API_KEY=             # Lasciare vuoto per disabilitare
# ... (altri provider)

# Budget guard (protezione costi)
DAILY_COST_BUDGET_EUR=0.00    # Default: ZERO (solo Ollama)
MONTHLY_COST_BUDGET_EUR=0.00  # Alzare da admin UI per abilitare cloud
```

#### Performance Tuning

```env
# Chunking
CHUNK_SIZE_TOKENS=800          # Dimensione chunk (600-1200 ottimale)
CHUNK_OVERLAP_PERCENT=15       # Overlap tra chunk (10-20%)

# RAG Retrieval
TOP_K_DENSE=30                 # Top risultati dense search
TOP_K_SPARSE=30                # Top risultati sparse search
TOP_K_RERANK=5                 # Risultati finali dopo rerank
HYBRID_FUSION_K=60             # Parametro RRF fusion

# Concorrenza
MAX_CONCURRENT_EMBEDDINGS=16   # Embed paralleli (CPU: 4-8, GPU: 16-32)
MAX_CONCURRENT_LLM_CALLS=8     # LLM calls parallele (Ollama: 4-8)
MAX_CONCURRENT_UPLOADS=3       # Upload simultanei
```

### 4.2 Configurazione Docker

#### docker-compose.yml - Limiti Risorse

```yaml
services:
  rust-engine:
    deploy:
      resources:
        limits:
          cpus: '4.0'
          memory: 4G
        reservations:
          cpus: '2.0'
          memory: 2G
```

**Produzione**: Aumentare limiti in base a hardware disponibile:
- **CPU**: 4-8 core per rust-engine
- **Memory**: 4-8 GB per rust-engine, 8-16 GB per ollama (con GPU)

#### Ollama GPU Support

```yaml
ollama:
  image: ollama/ollama:latest
  deploy:
    resources:
      reservations:
        devices:
          - driver: nvidia
            count: 1
            capabilities: [gpu]
```

**Prerequisiti**: NVIDIA Container Toolkit installato sul host.

---

## 5. Gestione Servizi

### 5.1 Comandi Base

```bash
# Avvio
make up                    # Start tutti i servizi
docker compose up -d       # Equivalente senza Makefile

# Stop
make down                  # Stop tutti i servizi
docker compose down        # Equivalente

# Restart singolo servizio
docker compose restart rust-engine

# Stop + rimozione volumi (⚠️ DATI PERSI)
docker compose down -v
```

### 5.2 Log Management

```bash
# Log di tutti i servizi
make logs
docker compose logs -f

# Log singolo servizio
docker compose logs -f rust-engine

# Ultime N righe
docker compose logs --tail=100 rust-engine

# Log con timestamp
docker compose logs -f --timestamps rust-engine

# Esporta log in file
docker compose logs rust-engine > rust-engine-$(date +%Y%m%d).log
```

### 5.3 Rebuild e Update

```bash
# Rebuild singolo servizio
make rebuild-rust          # Solo Rust engine
make rebuild-python        # Solo Python worker

# Rebuild tutto
make rebuild-all
docker compose build --no-cache  # Force rebuild

# Pull ultima versione immagini
docker compose pull

# Update immagini + rebuild custom
docker compose pull && docker compose build && docker compose up -d
```

### 5.4 Gestione Modelli Ollama

```bash
# Lista modelli installati
docker exec archivio-ollama ollama list

# Scarica nuovo modello
docker exec archivio-ollama ollama pull llama3:8b

# Rimuovi modello non usato
docker exec archivio-ollama ollama rm qwen2.5:14b

# Verifica spazio disco
docker exec archivio-ollama du -sh /root/.ollama
```

---

## 6. Monitoring e Observability

### 6.1 Health Checks

#### Endpoint Health

```bash
# Script automatico
make health

# Manuale
curl http://localhost:9080/health | jq
curl http://localhost:8090/health | jq
curl http://localhost:8091/health | jq
curl http://localhost:6335/collections | jq
```

#### Response Attese

**PHP Gateway** (`:9080/health`):
```json
{
  "status": "ok",
  "service": "php-gateway",
  "version": "0.1.0",
  "timestamp": 1778605000,
  "rust_engine": "connected"
}
```

**Rust Engine** (`:8090/health`):
```json
{
  "status": "ok",
  "service": "rust-engine",
  "version": "0.1.0",
  "cloud_enabled": false,
  "providers": ["ollama"]
}
```

### 6.2 Prometheus + Grafana Stack

```bash
# Avvia observability stack
make observability-up
docker compose -f docker-compose.observability.yml up -d

# Accesso
open http://localhost:3001  # Grafana (admin/admin)
open http://localhost:9090  # Prometheus
```

#### Metriche Esposte

**Rust Engine** (`:8090/metrics`):
- `http_requests_total` — Totale richieste
- `http_request_duration_seconds` — Latenza richieste
- `rag_query_duration_seconds` — Tempo query RAG
- `rag_chunks_retrieved` — Chunks recuperati
- `llm_tokens_used` — Token LLM consumati

**Python Worker** (`:8091/metrics`) *(da implementare)*:
- `pdf_parse_duration_seconds`
- `ocr_pages_processed`
- `reranker_score_distribution`

### 6.3 Alert Rules (Prometheus)

```yaml
# observability/prometheus/alerts.yml
groups:
  - name: archivio_parlante
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.05
        for: 5m
        annotations:
          summary: "Tasso errori HTTP 5xx > 5%"
          
      - alert: SlowRAGQueries
        expr: histogram_quantile(0.95, rag_query_duration_seconds) > 10
        for: 5m
        annotations:
          summary: "Query RAG p95 > 10 secondi"
```

---

## 7. Backup e Recovery

### 7.1 Backup Automatico MySQL

```bash
# Backup manuale
make backup-db

# Oppure:
docker exec archivio-mysql mysqldump \
  -u root \
  -pdevpass123 \  # Usa password da .env in produzione
  archivio_parlante_x \
  | gzip > backups/db_$(date +%Y%m%d_%H%M%S).sql.gz

# Backup con cron (Linux)
0 2 * * * cd /opt/archivio-parlante && make backup-db
```

### 7.2 Restore Database

```bash
# Restore da backup più recente
make restore-db

# Restore da file specifico
gunzip < backups/db_20260512_140000.sql.gz \
  | docker exec -i archivio-mysql mysql \
    -u root \
    -pdevpass123 \
    archivio_parlante_x
```

### 7.3 Backup Qdrant Vectors

```bash
# Backup volume Qdrant
docker run --rm \
  -v archivio_qdrant_storage:/source \
  -v $(pwd)/backups:/backup \
  alpine \
  tar czf /backup/qdrant_$(date +%Y%m%d).tar.gz -C /source .

# Restore
docker run --rm \
  -v archivio_qdrant_storage:/target \
  -v $(pwd)/backups:/backup \
  alpine \
  tar xzf /backup/qdrant_20260512.tar.gz -C /target
```

### 7.4 Backup Completo Stack

```bash
#!/bin/bash
# backup-all.sh

BACKUP_DIR="./backups/$(date +%Y%m%d_%H%M%S)"
mkdir -p $BACKUP_DIR

# MySQL
docker exec archivio-mysql mysqldump -u root -p$MYSQL_PASSWORD archivio_parlante_x \
  | gzip > $BACKUP_DIR/mysql.sql.gz

# Qdrant
docker run --rm -v archivio_qdrant_storage:/source -v $(pwd)/$BACKUP_DIR:/backup alpine \
  tar czf /backup/qdrant.tar.gz -C /source .

# Ollama models (opzionale, grande)
# docker run --rm -v archivio_ollama_models:/source -v $(pwd)/$BACKUP_DIR:/backup alpine \
#   tar czf /backup/ollama.tar.gz -C /source .

# File upload utenti
tar czf $BACKUP_DIR/uploads.tar.gz shared/uploads

# .env (escluso da git)
cp .env $BACKUP_DIR/.env.backup

echo "✅ Backup completo in: $BACKUP_DIR"
```

**Retention Policy Raccomandato**:
- Daily: 7 giorni
- Weekly: 4 settimane
- Monthly: 12 mesi

---

## 8. Troubleshooting

### 8.1 Problemi Comuni

#### Errore: "Port already in use"

```bash
# Trova processo che usa porta 9080
lsof -i :9080
# O su Windows:
netstat -ano | findstr :9080

# Opzione A: Kill processo
kill -9 <PID>

# Opzione B: Cambia porta in docker-compose.yml
services:
  php-gateway:
    ports:
      - "9081:80"  # Cambia 9080 → 9081
```

#### Servizio non si avvia

```bash
# Check log dettagliato
docker compose logs rust-engine --tail=200

# Check exit code
docker compose ps rust-engine

# Restart con fresh state
docker compose down rust-engine
docker compose up -d rust-engine
```

#### Qdrant "http2 protocol error"

**Diagnosi**: Version mismatch tra client Rust e server Qdrant.

```bash
# Check versioni
docker exec archivio-qdrant cat /qdrant/VERSION  # Server
grep qdrant-client engine-rust/Cargo.toml        # Client

# Fix: Update client in Cargo.toml
qdrant-client = "1.12"  # Match server version

# Rebuild Rust engine
docker compose build rust-engine
docker compose up -d rust-engine
```

#### Ollama model non si carica

```bash
# Check log Ollama
docker logs archivio-ollama --tail=50

# Errore: "resource limitations"
# Fix: Aumenta Docker memory (Settings → Resources → 8GB+)

# Re-download model corrotto
docker exec archivio-ollama ollama rm nomic-embed-text
docker exec archivio-ollama ollama pull nomic-embed-text
```

#### MySQL "connection refused"

```bash
# Verifica MySQL sia UP
docker compose ps mysql

# Check password corretta
docker exec -it archivio-mysql mysql -u root -p  # Inserisci password da .env

# Reset password (se dimenticata)
docker compose down mysql
docker volume rm archivio_mysql_data  # ⚠️ DATI PERSI
docker compose up -d mysql
```

### 8.2 Diagnostic Commands

```bash
# Container resource usage
docker stats

# Disk usage
docker system df

# Network inspect
docker network inspect archivio-parlantex_default

# Exec shell nel container
docker exec -it archivio-rust-engine sh
docker exec -it archivio-mysql bash

# Check logs errori critici
docker compose logs | grep -i "error\|fatal\|panic"
```

---

## 9. Procedure di Manutenzione

### 9.1 Update Sistema

```bash
# 1. Backup pre-update
./backup-all.sh

# 2. Pull nuova versione
git fetch --tags
git checkout v0.8.0  # O versione desiderata

# 3. Check changelog breaking changes
cat CHANGELOG.md | grep "BREAKING"

# 4. Update environment se necessario
diff .env.example .env

# 5. Rebuild immagini
docker compose build

# 6. Run migrations
make migrate

# 7. Restart stack
docker compose down
docker compose up -d

# 8. Verify
make health
```

### 9.2 Pulizia Spazio Disco

```bash
# Rimuovi immagini non usate
docker image prune -a

# Rimuovi container stopped
docker container prune

# Rimuovi volumi orfani
docker volume prune

# Pulizia completa (⚠️ DATI PERSI se volumi in uso)
docker system prune -a --volumes

# Compatta log Docker (Linux)
sudo truncate -s 0 /var/lib/docker/containers/*/*-json.log
```

### 9.3 Rotazione Log

```bash
# Configura logrotate (Linux)
# /etc/docker/daemon.json
{
  "log-driver": "json-file",
  "log-opts": {
    "max-size": "10m",
    "max-file": "3"
  }
}

# Restart Docker daemon
sudo systemctl restart docker
```

---

## 10. Security Hardening

### 10.1 Checklist Produzione

- [ ] **Secrets rotati**: JWT_SECRET e RUST_ENGINE_INTERNAL_TOKEN univoci
- [ ] **MySQL password forte**: Min 16 caratteri, alfanumerici + simboli
- [ ] **User MySQL dedicato**: Non usare root in produzione
  ```sql
  CREATE USER 'archivio_app'@'%' IDENTIFIED BY '<strong-password>';
  GRANT ALL ON archivio_parlante_x.* TO 'archivio_app'@'%';
  ```
- [ ] **Firewall configurato**: Solo porte necessarie esposte
- [ ] **HTTPS obbligatorio**: Reverse proxy (Nginx/Caddy) con Let's Encrypt
- [ ] **Rate limiting attivo**: Redis + middleware PHP
- [ ] **Audit logging abilitato**: Tutte le azioni admin loggata
- [ ] **Backup automatici**: Cron daily con retention policy
- [ ] **Monitoring attivo**: Prometheus + alerts configurati
- [ ] **Docker rootless** (se possibile): Run Docker in modalità non-root

### 10.2 Reverse Proxy Nginx (HTTPS)

```nginx
# /etc/nginx/sites-available/archivio-parlante
server {
    listen 443 ssl http2;
    server_name archivio.example.com;

    ssl_certificate /etc/letsencrypt/live/archivio.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/archivio.example.com/privkey.pem;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    # PHP Gateway
    location / {
        proxy_pass http://localhost:9080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # WebSocket (collaboration)
    location /ws/ {
        proxy_pass http://localhost:8090;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
    }
}
```

### 10.3 Security Scanning

```bash
# Scan vulnerabilità immagini Docker
docker scan archivio-parlantex-rust-engine

# Oppure con Trivy
trivy image archivio-parlantex-rust-engine

# Scan dipendenze Rust
cd engine-rust && cargo audit

# Scan dipendenze Python
cd engine-python && pip-audit

# Scan dipendenze PHP
cd php-gateway && composer audit
```

---

## 11. Performance Tuning

### 11.1 Database Optimization

```sql
-- MySQL tuning (my.cnf o my.ini)
[mysqld]
innodb_buffer_pool_size = 2G        # 50-70% RAM disponibile
innodb_log_file_size = 256M
innodb_flush_log_at_trx_commit = 2
query_cache_size = 0                # Disabled in MySQL 8
max_connections = 200

-- Index optimization
ANALYZE TABLE ap_documents;
ANALYZE TABLE ap_chunks;
OPTIMIZE TABLE ap_chat_messages;
```

### 11.2 Qdrant Tuning

```yaml
# In Qdrant config (se necessario custom config)
storage:
  optimizers:
    memmap_threshold: 20000  # Soglia per mmap
  on_disk_payload: true      # Payload su disco (risparmia RAM)
```

### 11.3 Ollama GPU Utilization

```bash
# Monitor GPU usage
nvidia-smi -l 1

# Ollama GPU layers (in .env o runtime)
# Più layer = più veloce, più VRAM
OLLAMA_NUM_GPU_LAYERS=35  # 0 = solo CPU, -1 = tutte su GPU
```

### 11.4 Concurrent Load Testing

```bash
# Install k6
# https://k6.io/docs/get-started/installation/

# Run load test
cd benchmarks/k6
k6 run load_test.js

# Target KPI:
# - p95 latency < 5s
# - Error rate < 1%
# - Throughput > 10 req/s (50 VU)
```

---

## 12. Disaster Recovery

### 12.1 RTO/RPO

**Recovery Time Objective (RTO)**: < 1 ora  
**Recovery Point Objective (RPO)**: < 24 ore (backup daily)

### 12.2 DR Procedure

```bash
# 1. Provision nuovo server con stessi requisiti
# 2. Install Docker + Docker Compose
# 3. Clone repository
git clone https://github.com/ajdroger/Archivio-parlanteX.git
cd Archivio-parlanteX

# 4. Restore .env da backup
cp /path/to/backup/.env.backup .env

# 5. Restore database
gunzip < /path/to/backup/mysql.sql.gz | \
  docker exec -i archivio-mysql mysql -u root -p<password> archivio_parlante_x

# 6. Restore Qdrant vectors
docker run --rm -v archivio_qdrant_storage:/target -v /path/to/backup:/backup alpine \
  tar xzf /backup/qdrant.tar.gz -C /target

# 7. Restore uploads
tar xzf /path/to/backup/uploads.tar.gz -C shared/

# 8. Start stack
docker compose up -d

# 9. Verify
make health

# 10. Test query RAG
curl -X POST http://localhost:9080/api/query -H "Authorization: Bearer <token>" \
  -d '{"kb_id":"<id>","query":"test recovery"}'
```

### 12.3 Failover Strategy

**Primary Site Down**:
1. Point DNS A record to DR site IP
2. Wait DNS propagation (TTL ~ 300s)
3. Monitor RustEngine logs for incoming requests
4. Notify users via status page

**Rollback**:
1. Fix issue on primary site
2. Sync data from DR → primary (if writes occurred)
3. Point DNS back to primary
4. Monitor for 24h

---

## 📞 Supporto e Escalation

### Livelli di Severità

| Severity | Descrizione | SLA Response | Esempio |
|---|---|---|---|
| **P1 - Critical** | Sistema down, dati a rischio | 15 min | MySQL non si avvia, disk full |
| **P2 - High** | Funzionalità critiche degradate | 1 ora | Ollama slow, Qdrant errors |
| **P3 - Medium** | Funzionalità secondarie non funzionanti | 4 ore | WebSocket intermittent |
| **P4 - Low** | Issue minori, richieste enhancement | 24 ore | Log verbosity, UI tweak |

### Contatti

- **GitHub Issues**: https://github.com/ajdroger/Archivio-parlanteX/issues
- **Email**: support@archivio-parlante.example.com *(da configurare)*
- **On-call**: *(definire rotation e pagerduty)*

---

**Versione Documento**: v1.0  
**Ultimo aggiornamento**: 2026-05-12  
**Prossima revisione**: 2026-06-12  
**Maintainer**: DevOps Team

---

*Questo manuale è in continuo aggiornamento. Per contribuire: invia PR a `docs/MANUALE_TECNICO_OPERATIVO.md`*
