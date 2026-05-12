# 👤 Manuale Amministratore — Archivio Parlante

> **Audience**: System Administrators, Application Administrators  
> **Versione**: v0.7.0  
> **Ultimo aggiornamento**: 2026-05-12

---

## 📋 Indice

1. [Introduzione](#introduzione)
2. [Primo Accesso](#primo-accesso)
3. [Gestione Utenti](#gestione-utenti)
4. [Gestione Workspace](#gestione-workspace)
5. [Gestione Knowledge Base](#gestione-knowledge-base)
6. [Configurazione LLM Provider](#configurazione-llm-provider)
7. [Gestione Budget e Costi](#gestione-budget-e-costi)
8. [Audit e Log](#audit-e-log)
9. [Sicurezza e Permessi](#sicurezza-e-permessi)
10. [Backup e Restore](#backup-e-restore)
11. [Monitoring e Reportistica](#monitoring-e-reportistica)
12. [Troubleshooting Utente](#troubleshooting-utente)

---

## 1. Introduzione

### 1.1 Ruolo dell'Amministratore

Come amministratore di Archivio Parlante, sei responsabile di:

✅ **Gestione accessi**: Creazione utenti, assegnazione ruoli, gestione permessi  
✅ **Governance workspace**: Creazione workspace, gestione membri, isolamento dati  
✅ **Configurazione sistema**: Provider LLM, budget, rate limiting  
✅ **Monitoraggio utilizzo**: Usage analytics, costi, performance  
✅ **Sicurezza**: Audit log, compliance, data retention  
✅ **Supporto utenti**: Troubleshooting, training, best practices

### 1.2 Panoramica Sistema

**Archivio Parlante** è un sistema RAG (Retrieval-Augmented Generation) enterprise per analisi contrattuale multi-tenant.

**Caratteristiche principali**:
- 🏢 **Multi-tenant**: Isolamento completo tra workspace (GDPR-compliant)
- 🔒 **RBAC granulare**: 4 ruoli (Owner, Admin, Member, Viewer) con permessi fine-grained
- 📊 **Multi-provider LLM**: Ollama locale (gratis) + 12 cloud provider opt-in (Claude, GPT, Gemini, etc.)
- 💰 **Budget guard**: Protezione costi con limiti giornalieri/mensili configurabili
- 🔍 **Zero allucinazioni**: Hybrid search + knowledge graph + hallucination detection
- 📝 **Collaborative**: Real-time annotation con WebSocket sync

### 1.3 Gerarchia Organizzativa

```
┌─────────────────────────────────┐
│  Super Admin (globale)          │  ← Tu sei qui
│  • Gestisce tutti i workspace   │
│  • Config sistema                │
│  • Monitoring globale            │
└─────────────────────────────────┘
              │
    ┌─────────┴─────────┐
    │                   │
┌───▼────────┐   ┌──────▼──────┐
│ Workspace A│   │ Workspace B │
│ • Owner    │   │ • Owner     │
│ • Admin    │   │ • Admin     │
│ • Member   │   │ • Member    │
│ • Viewer   │   │ • Viewer    │
└────────────┘   └─────────────┘
    │                   │
┌───▼────────────┐ ┌───▼────────────┐
│ Knowledge Base │ │ Knowledge Base │
│ • Documenti    │ │ • Documenti    │
│ • Analisi      │ │ • Analisi      │
└────────────────┘ └────────────────┘
```

---

## 2. Primo Accesso

### 2.1 URL di Accesso

**Development**: http://localhost:9080  
**Staging**: https://staging.archivio-parlante.example.com  
**Production**: https://archivio-parlante.example.com

### 2.2 Credenziali Default

⚠️ **IMPORTANTE**: Cambiare immediatamente al primo accesso!

```
Username: admin@archivio-parlante.local
Password: Admin123!
```

**Procedura cambio password**:
1. Login con credenziali default
2. Click su avatar (alto a destra) → "Profilo"
3. Tab "Sicurezza" → "Cambia Password"
4. Inserisci password forte (min 12 caratteri, maiuscole, minuscole, numeri, simboli)
5. Conferma

### 2.3 Setup Iniziale Wizard

Al primo accesso, il sistema presenta un wizard di configurazione:

#### Step 1: Profilo Amministratore
- Nome completo
- Email aziendale
- Ruolo (pre-selezionato: Super Admin)

#### Step 2: Workspace Default
- Nome workspace (es. "Ente XYZ")
- Descrizione
- Logo (opzionale)

#### Step 3: Provider LLM
- Scegli provider default:
  - **Ollama (locale)** — Gratis, privacy totale ← Raccomandato per iniziare
  - **Anthropic Claude** — Alta qualità, richiede API key
  - **OpenAI GPT** — Versatile, richiede API key
  - Altri 10 provider disponibili

#### Step 4: Budget Guard
- **Budget giornaliero**: €0.00 (default = solo Ollama gratuito)
- **Budget mensile**: €0.00
- ℹ️ Puoi alzare dopo per abilitare provider cloud

#### Step 5: Conferma
- Review configurazione
- Accetta Terms of Service
- Click "Completa Setup"

✅ **Setup completato!** Verrai reindirizzato alla dashboard.

---

## 3. Gestione Utenti

### 3.1 Creare Nuovo Utente

**Percorso**: Dashboard → Utenti → "+ Nuovo Utente"

**Form Creazione**:

| Campo | Descrizione | Esempio |
|---|---|---|
| **Nome completo** | Nome visualizzato | Mario Rossi |
| **Email** | Login + notifiche | mario.rossi@ente.it |
| **Ruolo globale** | Admin o User | User (default) |
| **Workspace** | Assegna a workspace | Seleziona da dropdown |
| **Ruolo workspace** | Owner/Admin/Member/Viewer | Member (default) |
| **Password temporanea** | Auto-generata o custom | (auto) |
| **Invia email benvenuto** | Checkbox | ✅ Consigliato |

**Ruoli Globali**:
- **Super Admin** ← Il tuo ruolo, accesso totale a sistema
- **Admin** → Può gestire utenti e workspace assegnati
- **User** → Utente standard, accesso solo a workspace dove è membro

**Click "Crea Utente"** → L'utente riceve email con link di attivazione e password temporanea.

### 3.2 Gestione Ruoli Workspace

Ogni utente ha un ruolo **per workspace**. Matrice permessi:

| Azione | Owner | Admin | Member | Viewer |
|---|---|---|---|---|
| **Visualizzare documenti** | ✅ | ✅ | ✅ | ✅ |
| **Caricare documenti** | ✅ | ✅ | ✅ | ❌ |
| **Eliminare documenti** | ✅ | ✅ | ✅ (solo propri) | ❌ |
| **Eseguire query RAG** | ✅ | ✅ | ✅ | ✅ (read-only) |
| **Creare knowledge base** | ✅ | ✅ | ✅ | ❌ |
| **Invitare membri** | ✅ | ✅ | ❌ | ❌ |
| **Cambiare ruoli** | ✅ | ✅ (no Owner) | ❌ | ❌ |
| **Gestire settings workspace** | ✅ | ✅ | ❌ | ❌ |
| **Eliminare workspace** | ✅ | ❌ | ❌ | ❌ |
| **Accesso audit log** | ✅ | ✅ | ❌ | ❌ |

**Assegnare Ruolo**:
1. Dashboard → Workspace → [Nome Workspace] → Tab "Membri"
2. Click utente → "Cambia Ruolo"
3. Seleziona nuovo ruolo
4. Conferma

### 3.3 Disabilitare Utente

**Non eliminare utenti** (perderesti audit trail). Invece, **disabilita**:

1. Dashboard → Utenti → [Nome Utente]
2. Click "⋮" (menu) → "Disabilita Account"
3. Conferma
4. ✅ L'utente non può più accedere, ma i suoi dati restano per audit

**Riabilitare**: Stesso menu → "Riabilita Account"

### 3.4 Reset Password

Se un utente dimentica la password:

**Opzione A: Self-service** (se email configurata)
1. Utente click "Password dimenticata?" su login page
2. Inserisce email
3. Riceve link reset (valido 1 ora)
4. Imposta nuova password

**Opzione B: Admin reset**
1. Dashboard → Utenti → [Nome Utente]
2. Click "Reset Password"
3. Sistema genera password temporanea
4. Invia email a utente O copia password e comunica manualmente
5. Utente deve cambiarla al primo login

### 3.5 Bulk Import Utenti (CSV)

Per onboarding massivo:

**Percorso**: Dashboard → Utenti → "Import da CSV"

**Format CSV**:
```csv
nome,cognome,email,workspace_id,ruolo_workspace
Mario,Rossi,mario.rossi@ente.it,ws_123abc,member
Laura,Bianchi,laura.bianchi@ente.it,ws_123abc,admin
```

**Campo workspace_id**: Copia dall'URL quando sei nel workspace (es. `/workspace/ws_123abc`)

**Upload** → Sistema valida → **Conferma** → Utenti creati e email inviate.

---

## 4. Gestione Workspace

### 4.1 Creare Nuovo Workspace

**Percorso**: Dashboard → Workspace → "+ Nuovo Workspace"

**Form Creazione**:

| Campo | Descrizione | Esempio |
|---|---|---|
| **Nome** | Identificativo visibile | Ente Pubblico XYZ |
| **Slug** | URL-friendly (auto da nome) | ente-pubblico-xyz |
| **Descrizione** | Scopo workspace | Analisi contratti gare d'appalto |
| **Logo** | Immagine (opzionale) | [Upload PNG/JPG, max 2 MB] |
| **Owner iniziale** | Chi gestisce (default: tu) | admin@ente.it |
| **Privacy** | Private/Internal/Public | Private (default) |
| **Lingua default** | Per UI e prompts | Italiano |
| **Timezone** | Per timestamp | Europe/Rome |

**Privacy Levels**:
- **Private** — Solo membri espliciti (default, GDPR-safe)
- **Internal** — Tutti gli utenti autenticati possono richiedere accesso
- **Public** — Solo se deployment pubblico (⚠️ dati visibili a tutti)

**Click "Crea Workspace"** → Workspace creato, sei automaticamente Owner.

### 4.2 Configurazione Workspace

**Percorso**: Dashboard → Workspace → [Nome] → Tab "Impostazioni"

#### 4.2.1 Generali
- Nome, descrizione, logo
- Lingua e timezone
- Livello privacy

#### 4.2.2 Knowledge Base Default
- **Modello embedding**: `nomic-embed-text` (768 dim, default) o `text-embedding-3-small` (OpenAI, 1536 dim)
- **Chunk size**: 800 token (default, range 600-1200)
- **Chunk overlap**: 15% (default, range 10-20%)
- **Top-K retrieval**: 5 (default, range 3-10)

#### 4.2.3 LLM Provider Workspace
Ogni workspace può **override** il provider globale:

**Use Case**: Workspace "Analisi Contratti Critici" → Claude Opus (alta qualità), Workspace "Test" → Ollama (gratis)

**Config**:
- **Provider**: [Dropdown: Ollama, Claude, GPT, etc.]
- **Model**: [Dropdown basato su provider]
  - Ollama → `qwen2.5:7b-instruct`
  - Claude → `claude-opus-4-7`, `claude-sonnet-4-6`
  - GPT → `gpt-4`, `gpt-3.5-turbo`
- **Temperature**: 0.1 (default, range 0-1)
- **Max tokens**: 2048 (default)

#### 4.2.4 Rate Limiting
Protegge da abuse:
- **Query al minuto**: 60 (default)
- **Upload al giorno**: 100 documenti (default)
- **Token LLM al giorno**: 1M (default)

#### 4.2.5 Retention Policy
**GDPR compliance**:
- **Document retention**: 365 giorni (default, poi auto-delete)
- **Chat history retention**: 90 giorni
- **Audit log retention**: 730 giorni (min legale: 365)

**Soft delete**: Documenti "deleted" restano 30 giorni in "Cestino" prima di purge definitivo.

### 4.3 Invitare Membri a Workspace

**Percorso**: Workspace → Tab "Membri" → "+ Invita Membro"

**Opzione A: Utente Esistente**
1. Cerca per email/nome
2. Seleziona utente
3. Scegli ruolo (Owner/Admin/Member/Viewer)
4. Click "Invita"
5. ✅ Utente vede workspace nella sua dashboard

**Opzione B: Nuovo Utente (invite esterno)**
1. Inserisci email nuovo utente
2. Scegli ruolo
3. Click "Invia Invito"
4. Email inviata con link registrazione
5. Utente completa signup → automaticamente membro workspace

### 4.4 Eliminare Workspace

⚠️ **ATTENZIONE**: Operazione irreversibile!

**Prerequisiti**:
- Solo Owner può eliminare
- Tutti i membri devono essere rimossi (eccetto Owner)
- Tutte le knowledge base devono essere eliminate o migrate

**Procedura**:
1. Dashboard → Workspace → [Nome]
2. Tab "Impostazioni" → Sezione "Danger Zone" (rosso)
3. Click "Elimina Workspace"
4. Conferma digitando nome workspace esatto
5. Click "Elimina Definitivamente"
6. ✅ Workspace e tutti i dati (documenti, chat, KB) eliminati in 24h

**Backup automatico**: Sistema crea backup pre-eliminazione, conservato 30 giorni per recovery emergenze.

---

## 5. Gestione Knowledge Base

### 5.1 Creare Knowledge Base

**Percorso**: Workspace → Tab "Knowledge Base" → "+ Nuova KB"

**Form**:
| Campo | Descrizione | Esempio |
|---|---|---|
| **Nome** | Identificativo | Contratti Appalti 2024 |
| **Descrizione** | Contenuto e scopo | Gare d'appalto primo semestre |
| **Embedding model** | Da config workspace (override) | nomic-embed-text |
| **Visibilità** | Private/Workspace/Public | Workspace (default) |
| **Owner** | Chi gestisce (default: creatore) | admin@ente.it |

**Visibilità**:
- **Private** — Solo owner e utenti esplicitamente condivisi
- **Workspace** — Tutti i membri workspace (default)
- **Public** — Tutti gli utenti sistema (⚠️ solo se appropriato)

**Click "Crea"** → KB vuota creata, ready per upload documenti.

### 5.2 Upload Documenti

**Percorso**: KB → Tab "Documenti" → "Upload Documenti"

**Metodi**:

#### Opzione A: Upload Web UI (drag & drop)
1. Drag files su area upload O click "Sfoglia"
2. Seleziona 1+ file (max 10 simultanei, 200 MB totali)
3. Sistema valida formato
4. Click "Avvia Upload"
5. Progress bar per ogni file
6. ✅ Al termine: "N documenti caricati e in elaborazione"

#### Opzione B: Bulk Upload (FTP/CLI)
Per grandi volumi (100+ documenti):
```bash
# Copia file su server
scp *.pdf user@server:/opt/archivio-parlante/shared/uploads/bulk/

# Trigger ingest batch
curl -X POST http://localhost:9080/api/admin/ingest-bulk \
  -H "Authorization: Bearer <admin-token>" \
  -d '{"kb_id":"kb_123abc","path":"/shared/uploads/bulk"}'
```

**Formati Supportati**:
- ✅ **PDF** (anche scansionati con OCR)
- ✅ **DOCX** (Word)
- ✅ **TXT** (plain text)
- ✅ **MD** (Markdown)

**Metadata Estratti Automaticamente**:
- Titolo (da filename o PDF metadata)
- Data creazione/modifica
- Autore (se presente in PDF)
- Numero pagine
- Hash SHA-256 (per deduplication)

### 5.3 Pipeline di Elaborazione

Dopo upload, ogni documento passa attraverso:

```
📄 Upload → 🔍 Parse → ✂️ Chunking → 🧠 Embedding → 💾 Qdrant → ✅ Indexed
   (0s)       (2-10s)     (1-5s)       (5-30s)      (1s)        (Ready)
```

**Step dettagliati**:

1. **Parse** (Python Worker)
   - Estrazione testo da PDF/DOCX
   - OCR se necessario (Tesseract)
   - Cleaning: rimozione header/footer ripetuti

2. **Chunking** (Rust Engine)
   - Semantic chunking (non fixed-size!)
   - 800 token/chunk (configurabile)
   - 15% overlap tra chunk consecutivi
   - Preserva paragrafi e frasi intere

3. **Embedding** (Ollama o cloud)
   - Vector denso 768-dim (nomic-embed-text)
   - + Sparse vector BM25 (keyword-based)
   - **Contextual retrieval**: Ogni chunk arricchito con contesto documento

4. **Qdrant Storage**
   - Upsert vector + metadata
   - Hybrid index (dense + sparse)

5. **Knowledge Graph Extraction** (opzionale, se abilitato)
   - Estrazione entità legali: PARTIES, DATES, AMOUNTS, CLAUSES
   - Estrazione relazioni: SIGNS, OBLIGATED_TO, PAYS, etc.
   - Storage in MySQL `ap_graph_nodes` e `ap_graph_edges`

**Tempo totale medio**: 30-60 secondi per documento di 10 pagine.

### 5.4 Monitoring Elaborazione

**Percorso**: KB → Tab "Documenti" → Colonna "Status"

**Stati possibili**:
- 🟡 **Pending** — In coda
- 🔵 **Processing** — In elaborazione
- 🟢 **Indexed** — Pronto per query ✅
- 🔴 **Failed** — Errore (click per dettagli)
- ⚪ **Deleted** — Soft deleted (cestino)

**Retry Failed**:
1. Click su documento Failed
2. Tab "Errori" mostra log dettagliato
3. Fix problema (es. PDF corrotto → re-upload)
4. Click "Retry Ingestion"

### 5.5 Gestione Duplicati

Sistema rileva automaticamente duplicati via **SHA-256 hash**:

**Comportamento default**:
- Upload documento identico → **Warning**: "Documento già presente in KB"
- Opzioni:
  - **Skip** (scartare nuovo upload)
  - **Replace** (sostituire vecchio con nuovo)
  - **Keep Both** (se metadata diverso, es. versioni diverse)

**Config**: KB → Impostazioni → "Deduplication Policy"
- **Strict** (default) — Hash esatto
- **Fuzzy** — Similarity 95%+ (title + content)
- **Off** — Permetti duplicati (non raccomandato)

---

## 6. Configurazione LLM Provider

### 6.1 Provider Disponibili

| Provider | Modelli | Costo | Qualità | Velocità | Privacy |
|---|---|---|---|---|---|
| **Ollama** | qwen2.5:7b, llama3, mistral | Gratis | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Anthropic Claude** | opus-4-7, sonnet-4-6 | €€€ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| **OpenAI GPT** | gpt-4, gpt-3.5-turbo | €€ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ |
| **Google Gemini** | gemini-1.5-pro, flash | €€ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| **DeepSeek** | deepseek-chat | € | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ |
| **Mistral** | mistral-large, medium | €€ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| ... | (altri 7 provider) | | | | |

### 6.2 Abilitare Provider Cloud

**Percorso**: Dashboard → Impostazioni Globali → Tab "LLM Providers"

**Esempio: Abilitare Anthropic Claude**

1. Click su card "Anthropic Claude"
2. Inserisci **API Key**:
   - Ottieni da: https://console.anthropic.com/
   - Formato: `sk-ant-api03-...` (128+ char)
3. **Testa connessione**: Click "Test API Key"
   - Sistema esegue query test
   - ✅ "Connessione riuscita" → salva
   - ❌ Errore → verifica API key e billing attivo
4. Seleziona **modelli da abilitare**:
   - ☑️ claude-opus-4-7 (€30/1M token input, €150/1M output)
   - ☑️ claude-sonnet-4-6 (€3/1M input, €15/1M output)
   - ☐ claude-haiku-4-5 (€0.25/1M input, €1.25/1M output)
5. **Budget allocation** (opzionale):
   - Max €/giorno per questo provider: €10.00
   - Max €/mese: €200.00
6. Click "Salva Configurazione"

**⚠️ Budget Guard**: Se non imposti budget allocation, usa il budget globale.

### 6.3 Configurazione Ollama (Locale)

**Vantaggi**:
- ✅ Gratis (costo zero)
- ✅ Privacy totale (nessun dato esce dal server)
- ✅ Nessuna dipendenza internet
- ⚠️ Qualità inferiore a GPT-4/Claude (ma ottima per 80% use case)

**Modelli Raccomandati**:

| Modello | Parametri | VRAM | Uso |
|---|---|---|---|
| **qwen2.5:7b-instruct** | 7B | 4.7 GB | Chat, analisi generale (default) |
| **qwen2.5:3b-instruct** | 3B | 2.0 GB | Task massivi, bassa latenza |
| **llama3:8b** | 8B | 5.5 GB | Alternative a qwen |
| **nomic-embed-text** | 137M | 0.3 GB | Embedding (OBBLIGATORIO) |

**Gestione Modelli**:

**Percorso**: Dashboard → Impostazioni → Ollama → "Gestione Modelli"

**Download nuovo modello**:
```bash
# Via CLI (più veloce)
docker exec archivio-ollama ollama pull llama3:8b

# Via UI
Dashboard → Ollama → "+ Scarica Modello" → Seleziona da lista → "Download"
```

**Tempo download**: 5-20 min per modello 7B (dipende da connessione)

**Rimuovi modello**:
```bash
docker exec archivio-ollama ollama rm qwen2.5:14b
```

**Check spazio disco**:
```bash
docker exec archivio-ollama du -sh /root/.ollama
# Output: 15G  /root/.ollama  (es. 3 modelli 7B)
```

### 6.4 Provider Fallback Chain

**Scenario**: Claude API down → sistema fallback automatico a GPT → poi a Ollama.

**Config**: Dashboard → LLM Providers → "Fallback Chain"

**Drag & drop provider** in ordine priorità:
1. 🥇 Claude Sonnet (primario)
2. 🥈 GPT-4 Turbo (fallback 1)
3. 🥉 Ollama qwen2.5:7b (fallback 2, sempre disponibile)

**Comportamento**:
- Request arriva → prova Claude
- Claude error 5xx O rate limit → prova GPT
- GPT error → prova Ollama
- Ollama error → ritorna errore 503 a utente

**Alert**: Ricevi notifica email se fallback attivato 3+ volte in 1 ora.

---

## 7. Gestione Budget e Costi

### 7.1 Budget Guard (Protezione Costi)

**Percorso**: Dashboard → Budget & Costi

**Livelli di Budget**:

1. **Budget Globale** (tutto il sistema)
   - Daily limit: €0.00 (default = solo Ollama)
   - Monthly limit: €0.00

2. **Budget per Provider** (es. solo Claude)
   - Daily: €10.00
   - Monthly: €200.00

3. **Budget per Workspace** (es. workspace "Produzione")
   - Daily: €5.00
   - Monthly: €100.00

**Enforcement**:
- Request LLM verifica budget disponibile **prima** di chiamare API
- Se budget esaurito → blocca chiamata, ritorna errore 402 "Budget exceeded"
- Utente vede: "Budget giornaliero esaurito. Riprova domani o contatta admin."

**Auto-reset**:
- Daily budget: Reset a mezzanotte (timezone configurato)
- Monthly budget: Reset il 1° del mese

### 7.2 Monitoring Costi Real-time

**Dashboard**: Budget & Costi → Tab "Utilizzo"

**Metriche visualizzate**:
- 💰 **Spesa oggi**: €X.XX / €Y.YY budget (progress bar)
- 📊 **Spesa questo mese**: €X.XX / €Y.YY budget
- 📈 **Trend**: Graf 30 giorni (costo per giorno)
- 🏆 **Top spender**: Workspace/utenti che consumano di più
- 🤖 **Per provider**: Breakdown Claude €X, GPT €Y, etc.

**Export Report**:
- Click "Export CSV" → scarica dettaglio costi per fatturazione
- Format: `date,workspace,user,provider,model,input_tokens,output_tokens,cost_eur`

### 7.3 Alert Budget

**Config**: Budget & Costi → "Alert"

**Alert Types**:
- 🟡 **Warning 80%**: Email quando budget raggiunge 80%
- 🔴 **Critical 95%**: Email + Slack quando budget raggiunge 95%
- 🚨 **Exceeded**: Notifica immediata se budget superato (non dovrebbe mai accadere, ma...)

**Destinatari**:
- Admin email (tuo indirizzo)
- Webhook Slack (opzionale)
- PagerDuty (solo critical, opzionale)

**Esempio Email**:
```
Subject: ⚠️ Budget Alert: 85% Used Today

Ciao Admin,

Il budget giornaliero ha raggiunto 85%:
- Speso: €8.50
- Budget: €10.00
- Rimanente: €1.50

Top consumer: Workspace "Contratti Gare" (€6.20)

Azioni suggerite:
- Aumenta budget se necessario
- Verifica query ripetitive anomale
- Considera switch a Ollama per task non critici

Dashboard: https://archivio.../budget
```

### 7.4 Cost Optimization Tips

**Best Practices**:

✅ **Use Ollama per default**: Gratis, ottima qualità per 80% task  
✅ **Claude/GPT solo per task critici**: Analisi complesse, domande sensibili  
✅ **Cache risultati**: Evita query duplicate (Redis cache attivo by default)  
✅ **Batch queries**: Se devi analizzare 100 contratti, fallo in batch notturno con rate limit ridotto  
✅ **Monitor outliers**: Identifica utenti/workspace che consumano anomalo → indaga abuse o bug  

**Esempio configurazione multi-tier**:
- Workspace "Test/Dev" → Solo Ollama (€0)
- Workspace "Analisi Standard" → Ollama + GPT-3.5 fallback (€0.10/giorno)
- Workspace "Contratti Critici" → Claude Opus primary (€10/giorno)

---

## 8. Audit e Log

### 8.1 Audit Log

**Percorso**: Dashboard → Audit Log

**Cosa viene loggato**:
- ✅ Login/logout (successo e fail)
- ✅ Creazione/modifica/eliminazione utenti
- ✅ Creazione/modifica/eliminazione workspace
- ✅ Upload/eliminazione documenti
- ✅ Query RAG (chi, quando, cosa, risultati)
- ✅ Cambio permessi/ruoli
- ✅ Modifiche configurazione (LLM provider, budget, etc.)
- ✅ Access denied (tentativi accesso non autorizzato)

**Format Log Entry**:
```json
{
  "timestamp": "2026-05-12T14:23:45Z",
  "event_type": "document.upload",
  "user_id": "usr_abc123",
  "user_email": "mario.rossi@ente.it",
  "workspace_id": "ws_xyz789",
  "resource_id": "doc_qwe456",
  "action": "create",
  "details": {
    "filename": "contratto_appalto_2024.pdf",
    "kb_id": "kb_asd321",
    "file_size_bytes": 2457600,
    "ip_address": "192.168.1.50"
  },
  "result": "success"
}
```

**Filtri Disponibili**:
- **Date range**: Oggi, Ultima settimana, Ultimo mese, Custom
- **Event type**: Dropdown (login, document.*, user.*, etc.)
- **User**: Search/select
- **Workspace**: Search/select
- **Result**: Success, Failed, Denied

**Export**: Click "Export CSV" → scarica log filtrati per compliance audit.

### 8.2 GDPR Compliance

**Right to Access**: Utente richiede tutti i suoi dati.

**Procedura**:
1. Dashboard → Utenti → [Email Utente] → Menu → "Export Dati GDPR"
2. Sistema genera ZIP con:
   - Profilo utente (JSON)
   - Documenti caricati (PDF originali)
   - Chat history (JSON)
   - Audit log filtrato per user (CSV)
3. ZIP disponibile per download (link email + dashboard)
4. Scadenza download: 7 giorni

**Right to Erasure ("Right to be Forgotten")**:

**Procedura**:
1. Dashboard → Utenti → [Email Utente] → Menu → "Elimina Dati GDPR"
2. ⚠️ **Warning**: Operazione irreversibile!
3. Sistema elimina:
   - Profilo utente (soft delete con timestamp)
   - Documenti personali (solo se workspace proprietario)
   - Chat history
   - Annotation create da utente
4. **Mantiene** (anonimizzato):
   - Audit log (legal requirement 2 anni) → user_id sostituito con `<deleted_user>`
   - Documenti condivisi (ownership trasferito a workspace admin)

**Retention Policy Default**:
- Documenti: 365 giorni dalla creazione
- Chat history: 90 giorni
- Audit log: 730 giorni (min legale)

**Config Custom**: Workspace → Impostazioni → "Data Retention"

### 8.3 Activity Monitoring Real-time

**Dashboard**: Home → Widget "Activity Stream"

Mostra ultimi 20 eventi in tempo reale:
```
🟢 14:23 mario.rossi@ente.it uploaded contratto_2024.pdf to KB "Appalti Q1"
🔵 14:22 laura.bianchi@ente.it executed query "clausole penali" in Workspace "Legal"
🟠 14:20 admin@ente.it changed role of giuseppe.verdi@ente.it to Admin in Workspace "Test"
🔴 14:15 unknown@hacker.com failed login attempt (IP: 203.0.113.42)
```

**Click su evento** → espande dettagli JSON completo.

---

## 9. Sicurezza e Permessi

### 9.1 Matrice Permessi Completa

**Azioni vs Ruoli**:

| Azione | Super Admin | Workspace Owner | Workspace Admin | Member | Viewer |
|---|---|---|---|---|---|
| **Gestione Utenti Globali** | ✅ | ❌ | ❌ | ❌ | ❌ |
| **Crea Workspace** | ✅ | ❌ | ❌ | ❌ | ❌ |
| **Elimina Workspace** | ✅ | ✅ (proprio) | ❌ | ❌ | ❌ |
| **Config LLM Provider Globale** | ✅ | ❌ | ❌ | ❌ | ❌ |
| **Invita utenti a Workspace** | ✅ | ✅ | ✅ | ❌ | ❌ |
| **Cambia ruolo utente** | ✅ | ✅ | ✅ (no Owner) | ❌ | ❌ |
| **Crea KB** | ✅ | ✅ | ✅ | ✅ | ❌ |
| **Elimina KB** | ✅ | ✅ | ✅ (propria) | ✅ (propria) | ❌ |
| **Upload documenti** | ✅ | ✅ | ✅ | ✅ | ❌ |
| **Elimina documenti** | ✅ | ✅ | ✅ | ✅ (propri) | ❌ |
| **Esegui query RAG** | ✅ | ✅ | ✅ | ✅ | ✅ (read-only) |
| **View chat history** | ✅ | ✅ | ✅ | ✅ (propria) | ✅ (read-only) |
| **Create annotation** | ✅ | ✅ | ✅ | ✅ | ❌ |
| **Delete annotation** | ✅ | ✅ | ✅ (propria) | ✅ (propria) | ❌ |
| **View audit log** | ✅ (tutti) | ✅ (workspace) | ✅ (workspace) | ❌ | ❌ |
| **Export audit log** | ✅ | ✅ | ✅ | ❌ | ❌ |
| **Config budget** | ✅ | ❌ | ❌ | ❌ | ❌ |
| **View analytics** | ✅ (tutti) | ✅ (workspace) | ✅ (workspace) | ❌ | ❌ |

### 9.2 Rate Limiting

**Protezione abuse e DoS**:

**Livelli**:
1. **Globale** (tutto il sistema):
   - 1000 req/min totali
   - 10 login fail/IP/5min → IP ban temporaneo (1h)

2. **Per Utente**:
   - 60 req/min per utente
   - 10 upload/ora per utente

3. **Per Workspace**:
   - 500 req/min per workspace
   - 100 upload/giorno per workspace

**Quando rate limit hit**:
- Response HTTP 429 "Too Many Requests"
- Header `Retry-After: 60` (secondi)
- Utente vede: "Troppo richieste. Attendi 1 minuto."

**Config Custom**: Dashboard → Workspace → Impostazioni → "Rate Limits"

**Whitelist IP**: Per integrazioni CI/CD o script legittimi:
- Dashboard → Impostazioni Globali → "Rate Limit Whitelist"
- Aggiungi IP o CIDR range (es. `192.168.1.0/24`)

### 9.3 Autenticazione a Due Fattori (2FA)

**Abilitazione**:

**Per Utente** (self-service):
1. Profilo → Sicurezza → "Abilita 2FA"
2. Scansiona QR code con app (Google Authenticator, Authy, 1Password)
3. Inserisci codice 6-digit per conferma
4. **Backup codes**: Sistema genera 10 codici recovery → **SALVALI!**
5. ✅ 2FA attivo → prossimo login richiederà OTP

**Forzare 2FA** (admin):
- Dashboard → Impostazioni Globali → "Sicurezza"
- ☑️ "Richiedi 2FA per tutti gli utenti"
- Grace period: 7 giorni (default, utenti devono abilitare entro 7gg)
- Dopo grace period: Login bloccato finché 2FA non abilitato

**Recovery**:
- Utente perde device con 2FA → usa backup code
- Backup code perso → Admin reset: Dashboard → Utenti → [User] → "Reset 2FA"

### 9.4 SSO/SAML (Opzionale)

Per integrazione con Active Directory / Okta / Google Workspace:

**Config**: Dashboard → Impostazioni → "Single Sign-On"

**Provider Supportati**:
- **SAML 2.0** (generico)
- **OAuth 2.0** (Google, Microsoft, GitHub)
- **LDAP/AD** (on-premise)

**Setup SAML** (esempio Okta):
1. In Okta: Crea app "Archivio Parlante"
2. Ottieni metadata XML o Entity ID + SSO URL + Certificate
3. In Archivio Parlante:
   - Upload metadata XML O inserisci campi manualmente
   - ACS URL: `https://archivio.example.com/auth/saml/acs`
   - Entity ID: `https://archivio.example.com/saml/metadata`
4. Testa connessione
5. Abilita: "Usa SSO come metodo login primario"

**Comportamento**:
- User click "Login" → redirect a IdP (Okta)
- User autentica su Okta → SAML assertion a Archivio Parlante
- Sistema crea utente automaticamente se non esiste (JIT provisioning)
- Mapping SAML attributes → Archivio Parlante fields:
  - `email` → email
  - `firstName` + `lastName` → nome
  - `groups` (custom) → workspace assignment

---

## 10. Backup e Restore

### 10.1 Backup Automatico

**Config**: Dashboard → Impostazioni → "Backup"

**Schedule**:
- **Daily**: 02:00 AM (timezone config)
- **Weekly**: Domenica 03:00 AM (full backup)
- **Monthly**: 1° del mese 04:00 AM (archived backup)

**Cosa viene backuppato**:
- ✅ Database MySQL (schema + dati)
- ✅ Qdrant vectors (collezioni complete)
- ✅ File upload utenti (`shared/uploads`)
- ✅ Configurazione sistema (provider, budget, etc.)
- ⏸️ Ollama models (opzionale, pesante)

**Storage Destination**:
- **Locale**: `/opt/archivio-parlante/backups` (default)
- **S3**: Bucket AWS S3 (raccomandato produzione)
- **Azure Blob**: Azure Storage
- **Google Cloud Storage**: GCS bucket

**Retention**:
- Daily: 7 giorni
- Weekly: 4 settimane
- Monthly: 12 mesi
- Archived: Indefinito (manuale delete)

**Config S3**:
```env
BACKUP_STORAGE=s3
BACKUP_S3_BUCKET=archivio-backups-prod
BACKUP_S3_REGION=eu-west-1
AWS_ACCESS_KEY_ID=<key>
AWS_SECRET_ACCESS_KEY=<secret>
```

### 10.2 Backup Manuale

**Trigger backup immediato**:

**Via UI**:
1. Dashboard → Backup → "Backup Ora"
2. Seleziona componenti (MySQL, Qdrant, Files, Config)
3. Descrizione (es. "Pre-upgrade v0.8")
4. Click "Avvia Backup"
5. Progress bar → notifica al completamento

**Via CLI**:
```bash
# Full backup
make backup-all

# Solo database
make backup-db

# Solo Qdrant
make backup-qdrant
```

**Tempo stimato**:
- Database 1 GB: ~30 secondi
- Qdrant 10 GB: ~2 minuti
- Files 50 GB: ~5 minuti
- **Totale (esempio 60 GB)**: ~10 minuti

### 10.3 Restore da Backup

⚠️ **ATTENZIONE**: Restore sovrascrive dati correnti!

**Procedura**:

1. **Stop sistema**:
   ```bash
   docker compose down
   ```

2. **Lista backup disponibili**:
   ```bash
   ls -lh backups/
   # Output:
   # backup_20260512_020000_full.tar.gz  (5.2 GB)
   # backup_20260511_020000_full.tar.gz  (5.1 GB)
   ```

3. **Estrai backup**:
   ```bash
   tar xzf backups/backup_20260512_020000_full.tar.gz -C /tmp/restore
   ```

4. **Restore MySQL**:
   ```bash
   docker compose up -d mysql  # Start solo MySQL
   sleep 10  # Attendi avvio
   
   gunzip < /tmp/restore/mysql.sql.gz | \
     docker exec -i archivio-mysql mysql -u root -p$MYSQL_PASSWORD archivio_parlante_x
   ```

5. **Restore Qdrant**:
   ```bash
   docker run --rm \
     -v archivio_qdrant_storage:/target \
     -v /tmp/restore:/backup \
     alpine tar xzf /backup/qdrant.tar.gz -C /target
   ```

6. **Restore Files**:
   ```bash
   tar xzf /tmp/restore/uploads.tar.gz -C shared/
   ```

7. **Start completo**:
   ```bash
   docker compose up -d
   ```

8. **Verify**:
   ```bash
   make health
   ```

**Tempo restore**: ~15-20 minuti per backup 60 GB.

### 10.4 Disaster Recovery Test

**Raccomandazione**: Testa DR ogni trimestre.

**Procedura Test**:
1. Provision server DR (clone config prod)
2. Copia ultimo backup da S3/locale a server DR
3. Esegui restore completo
4. Verify: Login, query RAG, upload test document
5. Misura RTO (Recovery Time Objective): Tempo da disaster a sistema UP
   - **Target**: < 1 ora
6. Documenta risultato in `docs/DR_TEST_YYYY-MM-DD.md`

---

## 11. Monitoring e Reportistica

### 11.1 Dashboard Admin

**URL**: `/admin/dashboard`

**Widget Principali**:

#### Panoramica Sistema
- 🟢 **Status Servizi**: 7/7 UP (verde) o dettaglio errori (rosso)
- 📊 **Utilizzo Risorse**: CPU 45%, RAM 12 GB/32 GB, Disk 120 GB/500 GB
- 👥 **Utenti Attivi**: 42 online, 150 totali
- 📁 **Documenti**: 1,234 indicizzati, 12 in elaborazione

#### Attività 24h
- 📈 **Query RAG**: 567 query (trend +12% vs ieri)
- 📤 **Upload**: 23 documenti (145 MB)
- 💬 **Chat Messages**: 1,234 messaggi
- ❌ **Errori**: 3 errori (0.5% error rate, soglia OK < 1%)

#### Budget
- 💰 **Speso Oggi**: €12.45 / €50.00 (25%)
- 📊 **Speso Mese**: €245.67 / €1,000.00 (24%)
- 🔝 **Top Provider**: Claude €8.20, GPT €3.15, Ollama €0.00

#### Alert Attivi
- 🟡 **Warning**: Disk usage 80% → Pianifica cleanup
- 🟢 **Info**: Backup completato 02:15 AM (5.2 GB)

### 11.2 Report Utilizzo

**Percorso**: Dashboard → Report → "Utilizzo Sistema"

**Report Disponibili**:

#### Report Utenti
- **Top utenti per query**: Chi usa di più il sistema
- **Utenti inattivi**: Non loggati da 30+ giorni (candidati disabilitazione)
- **Login trends**: Orari picco, giorni settimana, trend mensile

#### Report Workspace
- **Top workspace per storage**: Chi consuma più spazio
- **Top workspace per costi LLM**: Chi spende di più
- **Workspace inattivi**: Nessuna attività 90+ giorni

#### Report Performance
- **Latenza query p50/p95/p99**: Median 1.2s, p95 3.4s, p99 8.1s
- **Error rate trend**: Grafico 30 giorni
- **Throughput**: Query/ora per ora del giorno

#### Report Qualità RAG
- **Recall@5 medio**: 82% (target > 80%)
- **Hallucination rate**: 0.8% (target < 1%)
- **Citation coverage**: 95% risposte con citazioni

**Export**: Ogni report esportabile in CSV, PDF (con grafici), o JSON.

**Schedule Report Email**:
- Dashboard → Report → [Nome Report] → "Schedule"
- Frequenza: Daily, Weekly, Monthly
- Destinatari: Admin email + altri
- Formato: PDF attachment

### 11.3 Analytics Avanzati (Grafana)

**Se observability stack attivo**:

**Accesso**: http://localhost:3001 (default creds: `admin/admin`)

**Dashboard Pre-configurati**:

#### 1. System Overview
- Service health (uptime %)
- CPU/RAM/Disk per servizio
- Network traffic in/out
- Docker container status

#### 2. RAG Pipeline Metrics
- Query latency breakdown (embed, search, rerank, LLM)
- Chunks retrieved distribution
- LLM token usage (input/output)
- Cache hit rate (Redis)

#### 3. Business Metrics
- Active users (real-time)
- Documents uploaded (trend)
- Query volume (per workspace)
- Cost per query (€)

**Custom Dashboard**: Grafana consente creare dashboard custom con metriche Prometheus.

---

## 12. Troubleshooting Utente

### 12.1 Problemi Comuni

#### "Non riesco a fare login"

**Diagnosi**:
1. Check Audit Log → filtro user email → cerca "login.failed"
2. Possibili cause:
   - ❌ Password errata (più comune)
   - ❌ Account disabilitato
   - ❌ 2FA code invalido
   - ❌ IP banned (troppe fail)

**Fix**:
- Password errata → Reset password (§3.4)
- Account disabilitato → Riabilita (§3.3)
- 2FA issue → Reset 2FA (§9.3)
- IP ban → Whitelist IP temporaneamente (§9.2)

#### "Upload documento fallisce"

**Diagnosi**:
1. Check error message utente (screenshot se possibile)
2. Check log Python Worker: `docker logs archivio-python-worker --tail=50`
3. Possibili cause:
   - ❌ File troppo grande (> 200 MB default limit)
   - ❌ Formato non supportato (es. .zip)
   - ❌ PDF corrotto
   - ❌ Storage pieno

**Fix**:
- File troppo grande → Aumenta `MAX_UPLOAD_SIZE_MB` in `.env`
- Formato non supportato → Chiedi utente convertire (es. ZIP → singoli PDF)
- PDF corrotto → Testa aprire con Adobe Reader, se fail → file irrecuperabile
- Storage pieno → Cleanup vecchi documenti o aumenta volume disk

#### "Query RAG molto lenta (> 30s)"

**Diagnosi**:
1. Check Grafana dashboard "RAG Pipeline" → identifica bottleneck
2. Possibili cause:
   - ❌ Ollama overload (CPU-only, troppe richieste concorrenti)
   - ❌ Qdrant slow (troppi documenti, nessun index optimization)
   - ❌ Database slow (MySQL query non ottimizzata)

**Fix**:
- Ollama overload → Considera GPU o switch a cloud provider per picchi
- Qdrant slow → Comando Qdrant: `OPTIMIZE COLLECTION kb_xxx` (ricostruisce index)
- Database slow → `ANALYZE TABLE ap_chunks; OPTIMIZE TABLE ap_chunks;`

#### "Risposta RAG contiene informazioni sbagliate"

**Possibili cause**:
- ❌ **Hallucination** (LLM inventa)
- ❌ **Retrieval sbagliato** (chunk non pertinenti)
- ❌ **Document OCR error** (testo estratto male da PDF scansionato)

**Diagnosi**:
1. Check chat message → Tab "Fonti" → verifica chunk citati
2. Se chunk corretti ma risposta sbagliata → **Hallucination**
3. Se chunk sbagliati → **Retrieval issue**

**Fix Hallucination**:
- Abilita "Hallucination Detection" (Fase 6.2) → auto-flag risposte sospette
- Switch a LLM migliore (es. Claude Opus invece di qwen2.5:7b)
- Aumenta "Citation enforcement" (ogni claim DEVE avere citazione)

**Fix Retrieval**:
- Verifica chunk size ottimale (prova 600, 800, 1000 token)
- Abilita Graph RAG (Fase 6.1) per query multi-hop
- Re-index documento con parametri migliori

---

## 📞 Supporto

### Livelli di Supporto

| Livello | Responsabilità | Contatto |
|---|---|---|
| **L1 - Utente Finale** | Troubleshooting base, FAQ | help@archivio.example.com |
| **L2 - Admin (Tu)** | Config sistema, gestione utenti | admin@archivio.example.com |
| **L3 - DevOps** | Infrastruttura, deployment | ops@archivio.example.com |
| **L4 - Vendor (Anthropic/OpenAI)** | Issue provider cloud | support@anthropic.com |

### Escalation

**Quando escalare a L3 (DevOps)**:
- ❌ Servizio Docker non si avvia dopo restart
- ❌ Performance degradation inspiegabile
- ❌ Security incident (accesso non autorizzato)
- ❌ Data loss

**SLA**:
- **P1 (Critical)**: 15 minuti
- **P2 (High)**: 1 ora
- **P3 (Medium)**: 4 ore
- **P4 (Low)**: 24 ore

---

## 📚 Risorse Aggiuntive

- **Manuale Tecnico Operativo**: `docs/MANUALE_TECNICO_OPERATIVO.md`
- **Manuale Utente**: `docs/MANUALE_UTENTE.md`
- **Guida Rapida**: `docs/GUIDA_RAPIDA.md`
- **Architecture**: `docs/ARCHITECTURE.md`
- **Runbook**: `docs/RUNBOOK.md`
- **API Docs**: http://localhost:8090/docs (Swagger UI)

---

**Versione Documento**: v1.0  
**Ultimo aggiornamento**: 2026-05-12  
**Prossima revisione**: 2026-06-12  
**Maintainer**: Admin Team

---

*Per segnalare errori o suggerire miglioramenti: invia PR a `docs/MANUALE_AMMINISTRATORE.md`*
