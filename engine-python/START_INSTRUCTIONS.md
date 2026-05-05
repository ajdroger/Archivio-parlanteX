# 🐍 Python Worker - Avvio Nativo (Manuale)

## Prerequisiti
- Python 3.9+ installato
- pip installato

## Istruzioni Passo-Passo

### 1️⃣ Apri terminale nella cartella `engine-python`

```bash
cd C:\Users\aj_93\OneDrive\Documenti\GitHub\Archivio-parlanteX\engine-python
```

### 2️⃣ Crea virtual environment

**PowerShell/CMD:**
```powershell
python -m venv venv
```

**Bash/WSL:**
```bash
python3 -m venv venv
```

### 3️⃣ Attiva virtual environment

**PowerShell:**
```powershell
.\venv\Scripts\Activate.ps1
```

**CMD:**
```cmd
venv\Scripts\activate.bat
```

**Bash/WSL:**
```bash
source venv/bin/activate
```

### 4️⃣ Installa dipendenze minime

```bash
pip install --upgrade pip
pip install -r requirements-minimal.txt
```

**Output atteso:**
```
Successfully installed fastapi-0.115.0 uvicorn-0.32.0 pydantic-2.9.2 ...
```

### 5️⃣ Configura variabili d'ambiente

**Opzione A: Crea file `.env.local`** (raccomandato)
```bash
# engine-python/.env.local
PYTHON_LOG_LEVEL=info
OLLAMA_URL=http://localhost:11434
MYSQL_HOST=localhost
MYSQL_DB=archivio_parlante_x
MYSQL_USER=root
MYSQL_PASSWORD=
```

**Opzione B: Export manuale** (temporaneo)

**PowerShell:**
```powershell
$env:PYTHON_LOG_LEVEL="info"
$env:OLLAMA_URL="http://localhost:11434"
$env:MYSQL_HOST="localhost"
```

**Bash:**
```bash
export PYTHON_LOG_LEVEL=info
export OLLAMA_URL=http://localhost:11434
export MYSQL_HOST=localhost
```

### 6️⃣ Avvia server

```bash
uvicorn app.main:app --host 0.0.0.0 --port 8091 --reload
```

**Output atteso:**
```
INFO:     Uvicorn running on http://0.0.0.0:8091 (Press CTRL+C to quit)
INFO:     Started reloader process [12345] using WatchFiles
INFO:     Started server process [12346]
INFO:     Waiting for application startup.
INFO:     Application startup complete.
```

### 7️⃣ Verifica funzionamento

**Apri nuovo terminale e testa:**
```bash
curl http://localhost:8091/health
```

**Risposta attesa:**
```json
{"status":"ok","service":"python-worker","version":"1.0.0"}
```

---

## 🎯 Quick Start (One-Liner)

**PowerShell:**
```powershell
cd engine-python; python -m venv venv; .\venv\Scripts\Activate.ps1; pip install -r requirements-minimal.txt; uvicorn app.main:app --host 0.0.0.0 --port 8091
```

**Bash:**
```bash
cd engine-python && python3 -m venv venv && source venv/bin/activate && pip install -r requirements-minimal.txt && uvicorn app.main:app --host 0.0.0.0 --port 8091
```

---

## 🔧 Troubleshooting

### ❌ `python: command not found`
**Soluzione:** Installa Python da https://www.python.org/downloads/

### ❌ `pip: command not found`
**Soluzione:** 
```bash
python -m ensurepip --upgrade
```

### ❌ `ModuleNotFoundError: No module named 'fastapi'`
**Soluzione:** Virtual environment non attivato
```bash
# Riattiva venv
source venv/bin/activate  # bash
.\venv\Scripts\Activate.ps1  # powershell
```

### ❌ `Address already in use` (porta 8091 occupata)
**Soluzione:** Cambia porta
```bash
uvicorn app.main:app --host 0.0.0.0 --port 8092
```

### ❌ Dipendenze ML mancanti (FlagEmbedding, torch, etc.)
**Soluzione:** Funzionalità ML temporaneamente disabilitate
- Per ora il worker funziona solo per endpoint base
- Per funzionalità complete: installare `requirements.txt` completo (lungo)

---

## 📦 Installazione Completa (con ML/OCR)

**SOLO se necessario e hai tempo (20-30 minuti):**

```bash
pip install -r requirements.txt
```

**Nota:** Su Windows potrebbe richiedere Visual Studio Build Tools

---

## 🐳 Tornare a Docker

**Quando il bug Docker/WSL2 sarà risolto:**

```bash
# Stop processo nativo (Ctrl+C)
deactivate  # disattiva venv
docker compose up -d python-worker  # riprova Docker
```

---

## 📊 Monitoraggio

**Logs in tempo reale:**
```bash
# Il server stampa automaticamente su stdout
```

**Metrics:**
```bash
curl http://localhost:8091/metrics
```

**Health check continuo:**
```bash
# PowerShell
while ($true) { curl http://localhost:8091/health; Start-Sleep 5 }

# Bash
watch -n 5 curl http://localhost:8091/health
```

---

## ✅ Stack Completo

**Dopo l'avvio, verifica tutti i servizi:**

```bash
curl http://localhost:8080/api/health  # PHP Gateway
curl http://localhost:8090/health      # Rust Engine  
curl http://localhost:8091/health      # Python Worker (NATIVO)
curl http://localhost:6333/collections # Qdrant
curl http://localhost:11434/api/tags   # Ollama
```

**Tutti devono rispondere 200 OK** ✅
