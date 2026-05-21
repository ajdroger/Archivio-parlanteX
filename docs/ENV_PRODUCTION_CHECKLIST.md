# 🔐 Production Environment Checklist

> **CRITICAL**: Verifica OBBLIGATORIA prima del deploy in produzione

## ✅ Checklist `.env` Production

### 1. App Environment

```env
# ❌ DEV (da cambiare)
APP_ENV=dev
APP_DEBUG=true

# ✅ PRODUCTION (obbligatorio)
APP_ENV=production
APP_DEBUG=false              # ⚠️ NEVER true in production
```

### 2. Security Secrets (⚠️ CRITICAL)

```bash
# Generate strong secrets:
openssl rand -hex 32  # For JWT_SECRET
openssl rand -hex 64  # For RUST_ENGINE_INTERNAL_TOKEN
```

```env
# ❌ DEV (placeholder - da cambiare)
JWT_SECRET=CHANGE_ME_32_HEX_CHARS_MIN
RUST_ENGINE_INTERNAL_TOKEN=CHANGE_ME_64_HEX_CHARS_FOR_PHP_TO_RUST_AUTH

# ✅ PRODUCTION (esempio - generate nuovo)
JWT_SECRET=a3f8c9d2e1b7f4a6c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1
RUST_ENGINE_INTERNAL_TOKEN=1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1
```

### 3. MySQL Database

```env
# ❌ DEV (password vuota)
MYSQL_PASSWORD=
MYSQL_ROOT_PASSWORD=

# ✅ PRODUCTION (strong password obbligatoria)
MYSQL_PASSWORD=<strong-password-20-chars-min>
MYSQL_ROOT_PASSWORD=<different-strong-password>
```

**Generazione password sicura**:
```bash
openssl rand -base64 32
```

### 4. Logging

```env
# ❌ DEV (troppo verboso)
LOG_LEVEL=debug
RUST_LOG=debug

# ✅ PRODUCTION (bilanciato)
LOG_LEVEL=info                          # warn per ridurre overhead
RUST_LOG=info,rust_engine=info          # info per troubleshooting, warn per minimal
PYTHON_LOG_LEVEL=INFO                   # WARNING per minimal
```

### 5. CORS (⚠️ CRITICAL SECURITY)

```env
# ❌ DEV (aperto a localhost)
ENABLE_CORS=true
CORS_ORIGINS=http://localhost:3000,http://localhost:5173

# ✅ PRODUCTION (solo dominio reale)
ENABLE_CORS=false                       # Se frontend è servito dallo stesso dominio
# Oppure se frontend è su dominio separato:
ENABLE_CORS=true
CORS_ORIGINS=https://archivio-parlante.example.com,https://app.example.com
```

**NEVER** usare `*` in produzione.

### 6. Budget Guard

```env
# ✅ OK (default zero-cost mantenerlo così)
DAILY_COST_BUDGET_EUR=0.00
MONTHLY_COST_BUDGET_EUR=0.00
```

Admin deve alzare da UI se vuole abilitare provider cloud.

### 7. Storage Paths

```env
# ⚠️ Verificare che il path esista e abbia permessi corretti
SHARED_UPLOADS_PATH=/shared/uploads     # Inside Docker container
MAX_UPLOAD_SIZE_MB=200                  # Alzare se necessario (es. 500)
```

In produzione con Docker:
- Mappare a volume persistente (non ephemeral)
- Backup automatico

### 8. LLM Provider Keys (OPT-IN)

```env
# Default: tutti vuoti (zero-cost)
ANTHROPIC_API_KEY=
GOOGLE_API_KEY=
OPENAI_API_KEY=
...

# ✅ Compilare SOLO se admin vuole abilitare cloud providers
# E alzare DAILY_COST_BUDGET_EUR > 0.00
```

### 9. Database Name (⚠️ VINCOLANTE)

```env
# ✅ NON CAMBIARE (già creato via phpMyAdmin)
MYSQL_DB=archivio_parlante_x
```

### 10. Service URLs (Docker Compose)

```env
# ✅ OK for Docker Compose (lasciare così)
RUST_ENGINE_URL=http://rust-engine:8090
PYTHON_WORKER_URL=http://python-worker:8091
REDIS_URL=redis://redis:6379
QDRANT_URL=http://qdrant:6333
OLLAMA_URL=http://ollama:11434
```

Se deploy su Kubernetes/cloud, sostituire con IP/DNS reali.

---

## 🚀 Final Production `.env` Template

```env
# === PRODUCTION CONFIG ===
APP_ENV=production
APP_DEBUG=false
APP_NAME=Archivio Parlante

# === SECRETS (⚠️ GENERATE NEW) ===
JWT_SECRET=<openssl rand -hex 32>
RUST_ENGINE_INTERNAL_TOKEN=<openssl rand -hex 64>

# === MySQL ===
MYSQL_HOST=mysql
MYSQL_PORT=3306
MYSQL_DB=archivio_parlante_x
MYSQL_USER=root
MYSQL_PASSWORD=<openssl rand -base64 32>
MYSQL_ROOT_PASSWORD=<openssl rand -base64 32>

# === Qdrant ===
QDRANT_URL=http://qdrant:6333
QDRANT_GRPC_PORT=6334

# === Ollama (Zero-Cost Default) ===
OLLAMA_URL=http://ollama:11434
OLLAMA_MODEL_CHAT=qwen2.5:7b-instruct-q4_K_M
OLLAMA_MODEL_CHAT_SMALL=qwen2.5:3b-instruct-q4_K_M
OLLAMA_MODEL_EMBED=nomic-embed-text

# === Internal Services ===
RUST_ENGINE_URL=http://rust-engine:8090
PYTHON_WORKER_URL=http://python-worker:8091
REDIS_URL=redis://redis:6379

# === Cloud Providers (OPT-IN) ===
ANTHROPIC_API_KEY=
GOOGLE_API_KEY=
OPENAI_API_KEY=
DEEPSEEK_API_KEY=
QWEN_API_KEY=
MOONSHOT_API_KEY=
ZHIPU_API_KEY=
MISTRAL_API_KEY=
GROQ_API_KEY=
OPENROUTER_API_KEY=
TOGETHER_API_KEY=
FIREWORKS_API_KEY=

# === Budget Guard ===
DAILY_COST_BUDGET_EUR=0.00
MONTHLY_COST_BUDGET_EUR=0.00

# === Storage ===
SHARED_UPLOADS_PATH=/shared/uploads
MAX_UPLOAD_SIZE_MB=200
ALLOWED_MIME_TYPES=application/pdf,application/vnd.openxmlformats-officedocument.wordprocessingml.document,text/plain,text/markdown

# === RAG Tuning ===
CHUNK_SIZE_TOKENS=800
CHUNK_OVERLAP_PERCENT=15
TOP_K_DENSE=30
TOP_K_SPARSE=30
TOP_K_RERANK=5
HYBRID_FUSION_K=60

# === Concurrency ===
MAX_CONCURRENT_EMBEDDINGS=16
MAX_CONCURRENT_LLM_CALLS=8
MAX_CONCURRENT_UPLOADS=3

# === Logging ===
LOG_LEVEL=info
RUST_LOG=info,rust_engine=info
PYTHON_LOG_LEVEL=INFO

# === CORS (adjust for your domain) ===
ENABLE_CORS=false                       # true se frontend su dominio separato
CORS_ORIGINS=https://your-domain.com    # Sostituire con dominio reale
```

---

## ⚠️ Security Audit Checklist

- [ ] **JWT_SECRET**: generato con `openssl rand -hex 32`, minimo 32 caratteri hex
- [ ] **RUST_ENGINE_INTERNAL_TOKEN**: generato con `openssl rand -hex 64`, minimo 64 caratteri hex
- [ ] **MYSQL_PASSWORD**: generato con `openssl rand -base64 32`, minimo 20 caratteri
- [ ] **MYSQL_ROOT_PASSWORD**: diverso da MYSQL_PASSWORD, stesso livello di sicurezza
- [ ] **APP_DEBUG**: `false`
- [ ] **APP_ENV**: `production`
- [ ] **CORS_ORIGINS**: solo domini reali HTTPS, mai `*` o `http://localhost`
- [ ] **LOG_LEVEL**: `info` o `warn` (non `debug` in produzione)
- [ ] **.env file permissions**: `chmod 600 .env` (solo owner read/write)
- [ ] **.env** nel `.gitignore`: verificare che NON sia committato
- [ ] **Backup `.env`**: salvato in vault sicuro (KMS, 1Password, Vault)

---

## 📋 Deployment Steps

1. **Copia template**:
   ```bash
   cp .env.example .env
   ```

2. **Genera secrets**:
   ```bash
   echo "JWT_SECRET=$(openssl rand -hex 32)"
   echo "RUST_ENGINE_INTERNAL_TOKEN=$(openssl rand -hex 64)"
   echo "MYSQL_PASSWORD=$(openssl rand -base64 32)"
   echo "MYSQL_ROOT_PASSWORD=$(openssl rand -base64 32)"
   ```

3. **Modifica `.env`**:
   - Incolla i secrets generati
   - Sostituisci `CORS_ORIGINS` con dominio reale
   - Verifica tutte le sezioni di questa checklist

4. **Proteggi il file**:
   ```bash
   chmod 600 .env
   # Backup in vault sicuro
   ```

5. **Verifica**:
   ```bash
   # Check che .env non sia in git
   git status | grep .env  # Deve essere IGNORED

   # Check secrets non vuoti
   grep "CHANGE_ME" .env   # Nessun risultato se tutto OK
   ```

6. **Deploy**:
   ```bash
   docker compose --env-file .env up -d
   ```

---

## 🆘 Troubleshooting

### Se hai committato `.env` per errore:

```bash
# ⚠️ EMERGENCY: Remove .env from git history
git rm --cached .env
git commit -m "security: remove .env from git"

# ⚠️ Se già pushato, ROTATE TUTTI I SECRETS immediatamente
# - JWT_SECRET
# - RUST_ENGINE_INTERNAL_TOKEN
# - MYSQL_PASSWORD
# - Tutte le API keys
```

### Se .env è leggibile da altri utenti:

```bash
chmod 600 .env              # Solo owner read/write
chown root:root .env        # Se run as root (Docker)
```

### Se secrets sono deboli:

Rigenera con:
```bash
openssl rand -hex 64 | wc -c   # Deve essere >= 128 chars (64 hex = 128)
```

---

**Status**: ✅ Ready for production deployment dopo aver completato questa checklist.
