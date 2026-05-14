# 🏠 Setup Locale Completo - Backend + Frontend

**Testa tutto il sistema sul tuo PC Windows prima del deploy cloud**

**Tempo totale:** ~30 minuti  
**Prerequisiti:** Docker Desktop, Node.js  
**Costo:** €0.00 (tutto locale)

---

## 📋 Prerequisiti - Verifica

### 1. Docker Desktop

**🔍 Verifica installazione:**

```powershell
# Verifica se Docker è installato
docker --version
docker compose version

# Verifica se Docker Desktop sta running
docker info
```

**Output atteso:**
```
Docker version 24.x.x
Docker Compose version v2.x.x
Server:
 Containers: X
 Running: X
 ...
```

✅ **Se vedi output sopra:** Docker installato e running, vai al prossimo step!

❌ **Se comando non trovato:**
1. Download: https://www.docker.com/products/docker-desktop/
2. Installa Docker Desktop
3. Avvia Docker Desktop (icona nella system tray)
4. Aspetta che sia "Running" (verde)
5. Riprova comandi sopra

❌ **Se errore "Cannot connect to Docker daemon":**
```powershell
# Docker non sta running - aprilo manualmente
# 1. Cerca "Docker Desktop" nel menu Start
# 2. Avvialo
# 3. Aspetta icona verde nella system tray
# 4. Riprova: docker info
```

### 2. Node.js e npm

**🔍 Verifica installazione:**

```powershell
# Verifica se Node.js è installato
node --version
npm --version

# Verifica percorso installazione
where.exe node
where.exe npm
```

**Output atteso:**
```
v18.x.x o v20.x.x (o v22.x.x)
9.x.x o 10.x.x (o 11.x.x)
C:\Program Files\nodejs\node.exe
C:\Program Files\nodejs\npm.cmd
```

✅ **Se vedi versioni sopra:** Node.js installato, vai al prossimo step!

❌ **Se comando non trovato:**
1. Download: https://nodejs.org/ (versione LTS - recommended)
2. Installa con opzioni default
3. Riapri PowerShell
4. Riprova comandi sopra

### 3. Git

```powershell
git --version
```

✅ Hai già Git (stai usando il repo).

---

## 🚀 PARTE 1: Avvio Backend (Stack Completo)

### Step 1.1: Verifica File .env (2 min)

**🔍 Controlla se .env esiste già:**

```powershell
# Sei nella root del progetto
cd C:\Users\aj_93\OneDrive\Documenti\GitHub\Archivio-parlanteX

# Verifica se .env esiste
Test-Path .env

# Se TRUE, verifica contenuto
Get-Content .env | Select-String "MYSQL_PASSWORD"
Get-Content .env | Select-String "RUST_ENGINE_INTERNAL_TOKEN"
Get-Content .env | Select-String "OLLAMA_MODEL_CHAT"
```

**Output atteso:**
```
True
MYSQL_PASSWORD=devpass123
RUST_ENGINE_INTERNAL_TOKEN=1c5b997b0c11c412ca0...
OLLAMA_MODEL_CHAT=qwen2.5:7b-instruct-q4_K_M
```

✅ **Se vedi output sopra:** .env configurato correttamente, vai al prossimo step!

❌ **Se file non esiste o mancano valori:**
```powershell
# Copia da esempio
Copy-Item .env.example .env

# Verifica di nuovo
Get-Content .env | Select-String "MYSQL"
```

### Step 1.2: Download Modelli Ollama (PRIMA VOLTA - 10 min)

**🔍 Prima controlla se i modelli sono già scaricati:**

```powershell
# Verifica se Ollama container esiste già
docker ps -a | Select-String "ollama"

# Se esiste, verifica modelli già scaricati
docker exec archivio-ollama ollama list 2>$null

# Output mostra i 3 modelli? Salta al Step 1.3!
```

**Se vedi i 3 modelli già scaricati:**
```
NAME                              ID              SIZE
qwen2.5:7b-instruct-q4_K_M        abc123...       4.7 GB
qwen2.5:3b-instruct-q4_K_M        def456...       2.0 GB
nomic-embed-text:latest           ghi789...       274 MB
```

✅ **Modelli già presenti? SALTA QUESTO STEP**, vai a Step 1.3!

---

**❌ Se container non esiste o modelli mancanti:**

**⚠️ IMPORTANTE:** Fai questo PRIMA di avviare lo stack (solo la prima volta).

```powershell
# Avvia solo Ollama per scaricare i modelli
docker compose up -d ollama

# Aspetta 30 secondi che Ollama si avvii
Start-Sleep -Seconds 30

# Verifica Ollama risponde
docker exec archivio-ollama ollama --version

# Scarica modelli (questo richiede tempo!)
docker exec archivio-ollama ollama pull qwen2.5:7b-instruct-q4_K_M
docker exec archivio-ollama ollama pull qwen2.5:3b-instruct-q4_K_M
docker exec archivio-ollama ollama pull nomic-embed-text

# Verifica modelli scaricati
docker exec archivio-ollama ollama list
```

**Output atteso:**
```
NAME                              ID              SIZE      MODIFIED
qwen2.5:7b-instruct-q4_K_M        abc123...       4.7 GB    X minutes ago
qwen2.5:3b-instruct-q4_K_M        def456...       2.0 GB    X minutes ago
nomic-embed-text:latest           ghi789...       274 MB    X minutes ago
```

⏳ **Download richiede ~10 minuti** (dipende dalla connessione)

💡 **Tip:** Fai questo mentre leggi il resto della guida!

### Step 1.3: Avvia Stack Completo (3 min)

**🔍 Prima controlla se i servizi sono già running:**

```powershell
# Verifica se container sono già avviati
docker compose ps

# Verifica porte occupate
netstat -ano | findstr ":8090"  # Rust Engine
netstat -ano | findstr ":8091"  # Python Worker
netstat -ano | findstr ":9080"  # PHP Gateway
```

**Se vedi container già "Up" e porte occupate:**

✅ **Stack già running!** Vai direttamente a Step 1.4 (Health Checks)

---

**❌ Se container non esistono o sono "Exited":**

```powershell
# Avvia tutti i servizi
docker compose up -d

# Verifica stato
docker compose ps
```

**Output atteso (tutti UP):**
```
NAME                      STATUS    PORTS
archivio-mysql            Up        0.0.0.0:3307->3306/tcp
archivio-ollama           Up        0.0.0.0:11434->11434/tcp
archivio-php-gateway      Up        0.0.0.0:9080->80/tcp
archivio-python-worker    Up        0.0.0.0:8091->8091/tcp
archivio-qdrant           Up        0.0.0.0:6335->6333/tcp
archivio-redis            Up        0.0.0.0:6380->6379/tcp
archivio-rust-engine      Up        0.0.0.0:8090->8090/tcp
```

✅ Tutti "Up"? Perfetto!

❌ **Se qualcuno è "Exited" o "Restarting":**

```powershell
# Vedi log del servizio problematico
docker compose logs rust-engine --tail 50

# O tutti i log
docker compose logs --tail 100
```

### Step 1.4: Verifica Health Checks (2 min)

```powershell
# Test Rust Engine
curl.exe http://localhost:8090/health | ConvertFrom-Json

# Test Python Worker
curl.exe http://localhost:8091/health | ConvertFrom-Json

# Test Qdrant
curl.exe http://localhost:6335

# Test Ollama
curl.exe http://localhost:11434/api/tags | ConvertFrom-Json
```

**Output atteso:**

**Rust:**
```json
{
  "status": "ok",
  "service": "rust-engine",
  "version": "0.8.0",
  "providers": ["ollama"],
  "cloud_enabled": false
}
```

**Python:**
```json
{
  "status": "ok",
  "service": "python-worker",
  "version": "0.1.0"
}
```

**Qdrant:**
```
qdrant - vector search engine
```

**Ollama:**
```json
{
  "models": [
    {"name": "qwen2.5:7b-instruct-q4_K_M", ...},
    {"name": "nomic-embed-text:latest", ...}
  ]
}
```

✅ **Backend COMPLETO e FUNZIONANTE!**

---

## ⚛️ PARTE 2: Avvio Frontend React (5 min)

### Step 2.1: Installa Dipendenze (PRIMA VOLTA - 3 min)

**🔍 Prima controlla se dipendenze sono già installate:**

```powershell
# Vai nella cartella frontend
cd frontend

# Verifica se node_modules esiste
Test-Path node_modules

# Verifica se package-lock.json esiste
Test-Path package-lock.json

# Conta pacchetti installati
if (Test-Path node_modules) { 
    (Get-ChildItem node_modules -Directory).Count 
}
```

**Se vedi:**
```
True
True
XXX  (numero di pacchetti > 500)
```

✅ **Dipendenze già installate!** Vai a Step 2.2!

---

**❌ Se node_modules non esiste o è vuoto:**

```powershell
# Installa dipendenze
npm install
```

⏳ **Tempo:** ~2-3 minuti (scarica pacchetti)

**Verifica installazione completata:**
```powershell
Test-Path node_modules/@vitejs/plugin-react
# Output: True = installazione OK
```

### Step 2.2: Configura Environment Frontend (1 min)

**🔍 Prima controlla se .env.local esiste già:**

```powershell
# Verifica se file esiste (devi essere in frontend/)
Test-Path .env.local

# Se esiste, vedi contenuto
if (Test-Path .env.local) {
    Get-Content .env.local
}
```

**Se vedi output corretto:**
```
VITE_API_URL=http://localhost:9080
VITE_RUST_ENGINE_URL=http://localhost:8090
VITE_APP_NAME=Archivio Parlante
VITE_APP_ENV=development
```

✅ **.env.local già configurato!** Vai a Step 2.3!

---

**❌ Se file non esiste o è sbagliato:**

```powershell
# Crea file .env.local per frontend
@"
VITE_API_URL=http://localhost:9080
VITE_RUST_ENGINE_URL=http://localhost:8090
VITE_APP_NAME=Archivio Parlante
VITE_APP_ENV=development
"@ | Out-File -FilePath .env.local -Encoding utf8

# Verifica creazione
Get-Content .env.local
```

### Step 2.3: Avvia Dev Server (1 min)

**🔍 Prima controlla se dev server è già running:**

```powershell
# Verifica se porta 5173 è occupata
netstat -ano | findstr ":5173"

# Se vedi output, server già running!
# Apri browser: http://localhost:5173
```

**Se porta 5173 è occupata:**

✅ **Dev server già running!** Apri http://localhost:5173 nel browser!

---

**❌ Se porta libera (nessun output):**

```powershell
# Avvia Vite dev server
npm run dev
```

**Output atteso:**
```
  VITE v5.x.x  ready in XXX ms

  ➜  Local:   http://localhost:5173/
  ➜  Network: use --host to expose
  ➜  press h + enter to show help
```

✅ **Frontend RUNNING!**

**Verifica funzionamento:**
```powershell
# In un'altra finestra PowerShell
curl.exe http://localhost:5173
# Dovresti vedere HTML della app
```

---

## 🧪 PARTE 3: Test Sistema End-to-End

### Step 3.1: Accedi all'Interfaccia (1 min)

1. **Apri browser:** http://localhost:5173
2. Dovresti vedere la **homepage di Archivio Parlante**

✅ Frontend carica? Procedi!

### Step 3.2: Test Backend API (2 min)

**Apri una NUOVA finestra PowerShell** (lascia frontend running):

```powershell
cd C:\Users\aj_93\OneDrive\Documenti\GitHub\Archivio-parlanteX

# Test ingestion endpoint
$headers = @{
    "Content-Type" = "application/json"
    "X-Internal-Token" = "1c5b997b0c11c412ca0fddab6fd04ce2f45650b071924aab32c36181a0479d16091125846707ff40a2c14610b3c989cab53f4b2141ae9c2f9aec1324669e4770"
}

$body = @{
    kb_id = "kb_test_local"
    doc_id = "doc_test_001"
    file_path = "/shared/uploads/test.pdf"
} | ConvertTo-Json

# Test health (senza auth)
Invoke-RestMethod -Uri "http://localhost:8090/health" -Method Get
```

**Output:**
```json
{
  "status": "ok",
  "service": "rust-engine",
  ...
}
```

### Step 3.3: Test Completo RAG (5 min)

#### 3.3.1: Crea un KB di Test

```powershell
# Crea knowledge base via API
$createKbBody = @{
    name = "Test Locale"
    description = "Knowledge base per test in locale"
} | ConvertTo-Json

# Se hai endpoint per creare KB, usalo qui
# Altrimenti, procedi con kb_id esistente
```

#### 3.3.2: Test Ingestion con File Reale

```powershell
# Crea un PDF di test semplice
# Oppure usa uno dei PDF di esempio già nel repo

# Per ora, test con documento esistente dal setup precedente
$ingestBody = @{
    kb_id = "kb_prod"  # Usa quello già creato
    doc_id = "doc_test_locale"
    file_path = "/shared/uploads/test.pdf"
} | ConvertTo-Json

# Nota: Per ingestion completa, devi avere un PDF in /shared/uploads/
# Verifica se esiste:
docker exec archivio-rust-engine ls -la /shared/uploads/
```

#### 3.3.3: Test Query RAG

```powershell
# Test query (su kb_prod che abbiamo già testato)
$queryBody = @{
    kb_id = "kb_prod"
    query = "Quali sono le parti del contratto?"
    retrieval_mode = "hybrid"
    top_k = 3
} | ConvertTo-Json

$response = Invoke-RestMethod -Uri "http://localhost:8090/query" -Method Post -Headers $headers -Body $queryBody -ContentType "application/json"

# Mostra risultati
$response | ConvertTo-Json -Depth 10
```

**Output atteso:**
```json
{
  "results": [
    {
      "chunk_id": "...",
      "doc_id": "doc_production_test",
      "text": "CONTRATTO DI FORNITURA\n\nParti:\n- Fornitore: Acme SpA...",
      "score": 0.016393442
    },
    ...
  ],
  "processing_ms": 73,
  "candidates_count": 4
}
```

✅ **Ricevi risultati?** Sistema RAG funziona!

### Step 3.4: Test Chat (se implementato) (2 min)

```powershell
# Test chat endpoint
$chatBody = @{
    kb_id = "kb_prod"
    query = "Riassumi le clausole principali del contratto"
    conversation_id = ""  # Nuova conversazione
} | ConvertTo-Json

$chatResponse = Invoke-RestMethod -Uri "http://localhost:8090/chat" -Method Post -Headers $headers -Body $chatBody -ContentType "application/json"

$chatResponse | ConvertTo-Json -Depth 10
```

### Step 3.5: Test Frontend UI (5 min)

**Nel browser (http://localhost:5173):**

1. **Homepage:**
   - ✅ Logo e titolo visibili?
   - ✅ Menu navigazione funziona?

2. **Test Upload (se implementato):**
   - ✅ Pulsante "Upload PDF" presente?
   - ✅ Selezione file funziona?
   - ✅ Progress bar appare?

3. **Test Query Interface:**
   - ✅ Input query presente?
   - ✅ Pulsante "Cerca" funziona?
   - ✅ Risultati appaiono?

4. **Test Chat (se implementato):**
   - ✅ Interfaccia chat presente?
   - ✅ Invio messaggio funziona?
   - ✅ Risposta LLM appare?

---

## 📊 PARTE 4: Monitoring e Debug

### Step 4.1: Visualizza Logs in Tempo Reale

```powershell
# Tutti i servizi
docker compose logs -f

# Solo Rust Engine
docker compose logs -f rust-engine

# Solo Python Worker
docker compose logs -f python-worker

# Ultimi 100 log
docker compose logs --tail 100
```

**Ctrl+C per uscire**

### Step 4.2: Verifica Risorse Docker

```powershell
# Memoria/CPU usati
docker stats

# Spazio disco
docker system df
```

### Step 4.3: Database MySQL

```powershell
# Accedi a MySQL
docker exec -it archivio-mysql mysql -u root -pdevpass123 archivio_parlante_x

# Query di test
mysql> SHOW TABLES;
mysql> SELECT COUNT(*) FROM ap_users;
mysql> exit;
```

### Step 4.4: Qdrant Web UI

Apri browser: http://localhost:6335/dashboard

- ✅ Vedi le collections?
- ✅ Vedi i punti (chunks) indicizzati?

---

## 🛑 PARTE 5: Stop e Cleanup

### Stop Servizi (Mantieni Dati)

```powershell
# Stop tutti i servizi
docker compose stop

# Verifica tutto fermo
docker compose ps
```

Dati persistono nei volumi Docker. Riavvio con `docker compose up -d`.

### Stop + Rimuovi Container (Mantieni Dati)

```powershell
# Stop e rimuovi container
docker compose down

# I volumi (dati) rimangono
docker volume ls | Select-String "archivio"
```

### Reset Completo (Cancella TUTTO)

⚠️ **ATTENZIONE: Cancella anche il database!**

```powershell
# Stop e rimuovi TUTTO (container + volumi)
docker compose down -v

# Verifica volumi rimossi
docker volume ls
```

---

## 🔧 Troubleshooting

### Problema: "Port already in use"

**Errore:**
```
Error: bind: address already in use
```

**Soluzione:**

```powershell
# Trova processo su porta 8090 (esempio)
netstat -ano | findstr :8090

# Uccidi processo (sostituisci PID)
Stop-Process -Id <PID> -Force

# Oppure cambia porta in docker-compose.yml
```

### Problema: "Cannot connect to Docker daemon"

**Soluzione:**
1. Apri Docker Desktop
2. Aspetta che sia "Running" (verde)
3. Riprova comando

### Problema: Ollama modelli non scaricati

**Errore in log:**
```
Error: model 'qwen2.5:7b' not found
```

**Soluzione:**
```powershell
# Scarica modello manualmente
docker exec archivio-ollama ollama pull qwen2.5:7b-instruct-q4_K_M

# Verifica
docker exec archivio-ollama ollama list
```

### Problema: Frontend non si connette a backend

**Errore console browser:**
```
Failed to fetch: http://localhost:9080
CORS error
```

**Soluzione:**

Verifica `.env` nella root:
```
ENABLE_CORS=true
CORS_ORIGINS=http://localhost:3000,http://localhost:5173
```

Riavvia backend:
```powershell
docker compose restart php-gateway rust-engine
```

### Problema: MySQL non si avvia

**Errore:**
```
Can't connect to MySQL server
```

**Soluzione:**

```powershell
# Vedi log MySQL
docker compose logs mysql

# Reset MySQL volume
docker compose down
docker volume rm archivio_mysql_data
docker compose up -d mysql
```

### Problema: Qdrant errori HTTP/2

**Errore:**
```
http2 protocol error
```

**Soluzione:**

Già fixato in v0.8.0! Ma se persiste:

```powershell
# Reset Qdrant
docker compose down
docker volume rm archivio_qdrant_data
docker compose up -d qdrant
```

---

## ✅ Checklist Test Completo

Prima di procedere con deploy cloud, verifica:

**Backend:**
- [ ] Tutti i 7 container "Up" (`docker compose ps`)
- [ ] Rust health check OK
- [ ] Python health check OK
- [ ] Ollama modelli scaricati (3 modelli)
- [ ] Qdrant dashboard accessibile
- [ ] MySQL connessione OK

**Frontend:**
- [ ] Dev server avviato su :5173
- [ ] Homepage carica senza errori console
- [ ] API calls funzionano (no CORS errors)

**RAG Pipeline:**
- [ ] Query endpoint risponde
- [ ] Restituisce risultati rilevanti
- [ ] Processing time < 5s
- [ ] Chat endpoint funziona (se implementato)

**Performance:**
- [ ] Query latency < 3s (media)
- [ ] RAM Docker < 12GB
- [ ] CPU < 50% a idle

✅ **Tutto check?** Sei pronto per deploy cloud!

---

## 🎯 Prossimi Passi

Una volta testato tutto in locale:

1. **Commit eventuali fix:**
   ```powershell
   git add .
   git commit -m "test: verified local setup works"
   ```

2. **Procedi con cloud:**
   ```powershell
   # Apri guida cloud
   code infrastructure\QUICK_START_ITALIANO.md
   ```

3. **Deploy su Oracle Free Tier:**
   - Stesso stack
   - Stesso codice
   - **€0.00/mese** invece di locale

---

## 📝 Comandi Rapidi - Cheat Sheet

```powershell
# ===== AVVIO =====
docker compose up -d                    # Avvia tutto
npm run dev --prefix frontend           # Avvia frontend

# ===== VERIFICA =====
docker compose ps                       # Stato servizi
docker compose logs -f rust-engine      # Log Rust
curl.exe http://localhost:8090/health   # Health check

# ===== STOP =====
docker compose stop                     # Stop (mantieni dati)
docker compose down                     # Stop + rimuovi container
docker compose down -v                  # Reset completo

# ===== DEBUG =====
docker compose logs --tail 100          # Ultimi 100 log
docker stats                            # Risorse usate
docker exec -it archivio-mysql bash     # Shell MySQL

# ===== OLLAMA =====
docker exec archivio-ollama ollama list              # Lista modelli
docker exec archivio-ollama ollama pull <model>      # Scarica modello
```

---

## 🎉 Successo!

Hai un sistema RAG production-ready che gira sul tuo PC!

**Cosa puoi fare ora:**
- ✅ Testare tutte le funzionalità
- ✅ Fare development in locale
- ✅ Debuggare problemi facilmente
- ✅ Preparare demo per clienti
- ✅ Quando pronto → Deploy su cloud a costo ZERO

---

**Tempo Totale Speso:** ~30 minuti  
**Costo:** €0.00  
**Valore:** Sistema enterprise RAG completo 🚀

**Buon testing!** 🎊
