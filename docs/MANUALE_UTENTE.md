# 📚 Manuale Utente — Archivio Parlante

**Versione**: 0.8.0  
**Data**: 2026-05-12  
**Audience**: Utenti finali (analisti, consulenti legali, revisori)  
**Prerequisiti**: Account attivo fornito dall'amministratore

---

## Indice

1. [Introduzione](#1-introduzione)
2. [Primo Accesso](#2-primo-accesso)
3. [Interfaccia Utente](#3-interfaccia-utente)
4. [Gestione Documenti](#4-gestione-documenti)
5. [Creazione Knowledge Base](#5-creazione-knowledge-base)
6. [Interrogazione RAG](#6-interrogazione-rag)
7. [Chat Conversazionale](#7-chat-conversazionale)
8. [Annotazioni Collaborative](#8-annotazioni-collaborative)
9. [Confronto Multi-Contratto](#9-confronto-multi-contratto)
10. [Interpretare i Risultati](#10-interpretare-i-risultati)
11. [Best Practices](#11-best-practices)
12. [Risoluzione Problemi](#12-risoluzione-problemi)

---

## 1. Introduzione

### 1.1 Cos'è Archivio Parlante?

Archivio Parlante è un sistema intelligente per l'analisi forense di contratti aziendali che utilizza tecnologie di **Intelligenza Artificiale** (RAG - Retrieval-Augmented Generation) per:

- ✅ **Rispondere a domande complesse** sui tuoi contratti in linguaggio naturale
- ✅ **Estrarre clausole specifiche** da migliaia di pagine in pochi secondi
- ✅ **Confrontare contratti diversi** per identificare discrepanze o pattern comuni
- ✅ **Garantire la massima precisione** con citazioni testuali verificabili (zero allucinazioni)
- ✅ **Proteggere la riservatezza** con elaborazione locale (nessun dato inviato a servizi esterni di default)

### 1.2 Caratteristiche Principali

| Funzionalità | Descrizione |
|---|---|
| **Ricerca Ibrida** | Combina ricerca semantica (significato) e keyword (parole esatte) |
| **Knowledge Graph Legale** | Mappa relazioni tra parti, date, importi, clausole, giurisdizioni |
| **Anti-Allucinazione** | Ogni risposta include citazioni testuali verificabili dai documenti originali |
| **Collaborazione Real-Time** | Annotazioni condivise, modifiche sincronizzate tra utenti |
| **Privacy-First** | Elaborazione locale di default, cloud opt-in solo se necessario |
| **Multi-Lingua** | Interfaccia italiana, supporto contratti multilingua (EN, IT, FR, DE, ES) |

### 1.3 Requisiti Browser

- **Browser supportati**: Chrome 100+, Firefox 100+, Edge 100+, Safari 15+
- **Connessione**: Stabile (500 Kbps minimo per WebSocket real-time)
- **JavaScript**: Abilitato

---

## 2. Primo Accesso

### 2.1 Ricevere le Credenziali

L'amministratore del sistema ti fornirà:

1. **URL di accesso**: `https://archivio-parlante.tuaazienda.it` (o IP/porta se installazione locale)
2. **Email** del tuo account
3. **Password temporanea** (da cambiare al primo login)
4. **Ruolo** assegnato (Owner, Admin, Member, o Viewer)

### 2.2 Login

1. Apri il browser e vai all'URL fornito
2. Inserisci **email** e **password temporanea**
3. Clicca su **"Accedi"**
4. Se è il primo accesso, ti verrà richiesto di **cambiare la password**:
   - Minimo 12 caratteri
   - Almeno 1 maiuscola, 1 minuscola, 1 numero, 1 simbolo
   - Non riutilizzare password vecchie

### 2.3 Verifica Account

Se l'amministratore ha abilitato la verifica via email:

1. Controlla la tua casella email
2. Clicca sul link di verifica (valido 24 ore)
3. Torna alla pagina di login e accedi

### 2.4 Workspace di Benvenuto

Al primo accesso:

- Se sei **Owner/Admin**, vedrai un workspace vuoto da configurare
- Se sei **Member/Viewer**, vedrai i workspace a cui sei stato aggiunto
- Riceverai un **tour guidato** (opzionale, durata 3 minuti)

---

## 3. Interfaccia Utente

### 3.1 Layout Generale

```
┌────────────────────────────────────────────────────────────────┐
│  [Logo] Archivio Parlante        🔍 Ricerca Globale   👤 Utente │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  ┌─────────────┐  ┌──────────────────────────────────────┐   │
│  │             │  │                                      │   │
│  │  Sidebar    │  │       Area Principale                │   │
│  │  (Menu)     │  │       (Contenuto)                    │   │
│  │             │  │                                      │   │
│  │ • Home      │  │                                      │   │
│  │ • KB        │  │                                      │   │
│  │ • Documenti │  │                                      │   │
│  │ • Chat      │  │                                      │   │
│  │ • Confronto │  │                                      │   │
│  │ • Audit     │  │                                      │   │
│  │             │  │                                      │   │
│  └─────────────┘  └──────────────────────────────────────┘   │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

### 3.2 Barra Superiore

| Elemento | Funzione |
|---|---|
| **Logo** | Clicca per tornare alla Home |
| **Workspace Selector** | Cambia workspace (se hai accesso a più di uno) |
| **Ricerca Globale** | Cerca tra tutti i documenti accessibili (Ctrl+K) |
| **Notifiche** | Nuove annotazioni, menzioni, upload completati |
| **Menu Utente** | Profilo, Impostazioni, Logout |

### 3.3 Sidebar (Menu Laterale)

| Voce | Descrizione | Shortcut |
|---|---|---|
| 🏠 **Home** | Dashboard con metriche e attività recenti | Alt+H |
| 📚 **Knowledge Bases** | Elenco KB disponibili | Alt+K |
| 📄 **Documenti** | Carica, gestisci, visualizza documenti | Alt+D |
| 💬 **Chat** | Conversazioni RAG salvate | Alt+C |
| 🔀 **Confronto** | Compara 2-5 contratti in parallelo | Alt+M |
| 📊 **Audit Log** | Cronologia operazioni (solo Admin/Owner) | Alt+A |

### 3.4 Temi e Accessibilità

**Cambiare Tema**:
1. Menu Utente → Impostazioni → Aspetto
2. Scegli: **Chiaro**, **Scuro**, o **Automatico** (segue sistema operativo)

**Accessibilità**:
- Navigazione completa da tastiera (Tab, Enter, Esc)
- Screen reader supportato (ARIA labels)
- Contrasto AAA WCAG 2.1 in modalità "Alto Contrasto"
- Zoom fino a 200% senza perdita di funzionalità

---

## 4. Gestione Documenti

### 4.1 Formati Supportati

| Formato | Estensioni | Note |
|---|---|---|
| **PDF** | `.pdf` | Preferito, OCR automatico se scansionato |
| **Microsoft Word** | `.docx`, `.doc` | Formattazione preservata |
| **Testo** | `.txt`, `.md` | UTF-8, ASCII |
| **HTML** | `.html`, `.htm` | Pulizia automatica tag |
| **Immagini** | `.png`, `.jpg`, `.tiff` | Solo con OCR abilitato |

**Dimensioni**: Massimo 200 MB per file (configurabile dall'admin).  
**Lingua**: Supporto automatico per IT, EN, FR, DE, ES.

### 4.2 Caricare un Documento

#### Metodo 1: Drag & Drop

1. Vai a **Documenti** dalla sidebar
2. Trascina il file PDF/DOCX nell'area "Trascina qui"
3. Attendi il caricamento (barra di progresso)
4. Compila i metadati:
   - **Titolo** (obbligatorio): Nome del contratto
   - **Categoria** (opzionale): Fornitura, NDA, Appalto, ecc.
   - **Data Contratto** (opzionale): Data firma
   - **Controparte** (opzionale): Nome dell'altra parte
   - **Note** (opzionale): Descrizione libera
5. Clicca **"Salva Documento"**

#### Metodo 2: Upload Classico

1. Clicca su **"+ Carica Documento"**
2. Seleziona file dal tuo computer
3. Compila metadati come sopra
4. Clicca **"Salva Documento"**

### 4.3 Elaborazione Automatica

Dopo il salvataggio, il sistema:

1. ✅ **Estrae il testo** (OCR se necessario, 1-3 min per 100 pagine)
2. ✅ **Chunking contestuale** (divide in paragrafi con contesto, 30 sec)
3. ✅ **Genera embeddings** (vettori semantici 768-dim, 1 min)
4. ✅ **Costruisce knowledge graph** (estrae entità legali, 2 min)
5. ✅ **Indicizza in Qdrant** (vettori dense + sparse, 10 sec)

**Stato visibile**: Barra di progresso con passaggi in tempo reale.

**Errori comuni**:
- ❌ **"File corrotto"**: PDF danneggiato, riprova con altro file
- ❌ **"OCR fallito"**: Immagine illeggibile, scansione troppo bassa qualità
- ❌ **"Dimensione eccessiva"**: File > 200 MB, contatta l'admin

### 4.4 Visualizzare un Documento

1. Nella lista **Documenti**, clicca sul nome del contratto
2. Si apre il **Visualizzatore** con:
   - **Anteprima PDF** (lato sinistro): scorribile, zoomabile (Ctrl + Scroll)
   - **Metadati** (lato destro): info, data upload, stato elaborazione
   - **Knowledge Graph** (tab): grafo interattivo delle entità estratte
   - **Annotazioni** (tab): note collaborative (vedi §8)

**Shortcut Visualizzatore**:
- `Ctrl + F`: Cerca nel documento
- `Ctrl + P`: Stampa
- `Ctrl + D`: Scarica originale
- `Esc`: Chiudi visualizzatore

### 4.5 Modificare Metadati

1. Visualizzatore → **Pulsante "Modifica"** (icona matita)
2. Cambia titolo, categoria, data, note
3. Clicca **"Salva Modifiche"**

**Permessi**:
- **Owner/Admin**: Possono modificare tutti i documenti del workspace
- **Member**: Solo documenti caricati da loro
- **Viewer**: Nessuna modifica (solo lettura)

### 4.6 Eliminare un Documento

1. Lista Documenti → **Pulsante "…"** (tre puntini) → **"Elimina"**
2. Conferma con password (per sicurezza)
3. Il documento viene **soft-deleted** (recuperabile dall'admin per 30 giorni)

**Attenzione**: Eliminare un documento lo rimuove da TUTTE le Knowledge Base che lo includevano.

---

## 5. Creazione Knowledge Base

### 5.1 Cos'è una Knowledge Base (KB)?

Una **Knowledge Base** è una **collezione di documenti** su cui puoi fare domande tramite RAG. Esempi:

- **KB "Contratti Fornitori 2025"**: Tutti i contratti con i fornitori di quest'anno
- **KB "Gare Pubbliche Regione Lombardia"**: Solo appalti pubblici lombardi
- **KB "NDA Storici"**: Tutti gli accordi di riservatezza firmati dal 2020

**Vantaggi di organizzare in KB**:
- ✅ Ricerca mirata (solo nei documenti rilevanti)
- ✅ Performance migliori (meno documenti = risposte più veloci)
- ✅ Confronti omogenei (compara solo contratti simili)

### 5.2 Creare una Nuova KB

1. Vai a **Knowledge Bases** dalla sidebar
2. Clicca su **"+ Nuova Knowledge Base"**
3. Compila il form:
   - **Nome** (obbligatorio): Es. "Contratti Fornitori 2025"
   - **Descrizione** (opzionale): Breve spiegazione dello scopo
   - **Lingua Primaria** (default: IT): IT, EN, FR, DE, ES
   - **Provider LLM** (default: Ollama locale):
     - **Ollama** (gratis, privacy, ~2-5 sec latenza)
     - **Anthropic Claude** (a pagamento, ~1 sec latenza, accuratezza massima) [solo se admin ha abilitato]
     - Altri 10 provider cloud (vedi §5.4)
4. Clicca **"Crea Knowledge Base"**

**Tempo creazione**: Istantanea (la collection Qdrant viene creata vuota).

### 5.3 Aggiungere Documenti a una KB

#### Metodo 1: Durante creazione KB

1. Nel form di creazione, sezione **"Documenti Iniziali"**
2. Seleziona uno o più documenti dalla lista (checkbox)
3. Clicca **"Crea Knowledge Base"**
4. Attendi indicizzazione (progresso visibile)

#### Metodo 2: Da KB esistente

1. Apri la KB → Tab **"Documenti"**
2. Clicca **"+ Aggiungi Documento"**
3. Seleziona dalla lista documenti disponibili
4. Clicca **"Aggiungi e Indicizza"**

**Indicizzazione**:
- 1 documento (50 pagine) = ~2 minuti
- 10 documenti (500 pagine totali) = ~15 minuti
- Puoi chiudere la pagina, continua in background

### 5.4 Configurare il Provider LLM

**Quando cambiare provider**:
- **Ollama (default)**: Per uso quotidiano, privacy massima, gratis
- **Claude 3.5 Sonnet (Anthropic)**: Per analisi critiche ad alta posta (es. contenziosi legali), massima accuratezza, ~€0.01/query
- **Qwen 2.5 (DeepSeek)**: Buon compromesso, ~€0.002/query
- **GPT-4o (OpenAI)**: Se già hai contratto enterprise con OpenAI

**Come cambiare**:
1. Apri KB → Tab **"Impostazioni"**
2. Sezione **"Modello LLM"**
3. Seleziona provider dal menu dropdown
4. Se cloud provider, verifica che l'admin abbia configurato le API key
5. Clicca **"Salva Impostazioni"**

**Budget e Costi**:
- Il sistema traccia il costo per ogni query
- Se il budget giornaliero è esaurito, il sistema passa automaticamente a Ollama (gratis)
- Vedi costi attuali in **Menu Utente → Usage & Billing**

### 5.5 Rimuovere Documenti da KB

1. Apri KB → Tab **"Documenti"**
2. Checkbox sul documento da rimuovere
3. Clicca **"Rimuovi da KB"**
4. Conferma

**Nota**: Il documento rimane nel workspace, viene solo rimosso dalla KB corrente.

---

## 6. Interrogazione RAG

### 6.1 Cos'è il RAG?

**RAG** (Retrieval-Augmented Generation) è la tecnica che permette di:

1. **Cercare** nei tuoi documenti i passaggi più rilevanti per la tua domanda
2. **Fornire** quei passaggi al modello LLM come contesto
3. **Generare** una risposta precisa e verificabile con citazioni testuali

**Differenza con ChatGPT normale**:
- ❌ ChatGPT: Risponde dalla memoria (training 2023), può inventare
- ✅ RAG: Risponde SOLO dai tuoi documenti, con citazioni verificabili

### 6.2 Fare una Domanda RAG

1. Apri una **Knowledge Base** dalla lista
2. Clicca su **"Nuova Query"** (o usa shortcut `Ctrl+Q`)
3. Scrivi la tua domanda in linguaggio naturale, es:
   - *"Quali sono le penali previste per ritardi nella consegna?"*
   - *"Qual è l'importo totale del contratto con Acme SpA firmato a marzo 2025?"*
   - *"Quali contratti prevedono clausole di riservatezza superiori a 5 anni?"*
4. (Opzionale) Configura **Parametri Avanzati**:
   - **Modalità Ricerca**: Ibrida (default), Solo Semantica, Solo Keyword
   - **Espansione Grafo**: 0-3 hop (default: 2) — quanti "salti" nel knowledge graph
   - **Top K**: 5-20 chunks (default: 10) — quanti paragrafi recuperare
   - **Threshold Confidence**: 0.5-0.9 (default: 0.7) — soglia minima rilevanza
5. Clicca **"Cerca"** (o premi `Enter`)

### 6.3 Tempi di Risposta Attesi

| Configurazione | Latenza p95 |
|---|---|
| Ollama + Hybrid + Grafo (2-hop) | 3-8 secondi |
| Claude 3.5 + Hybrid + Grafo (2-hop) | 2-4 secondi |
| Ollama + Solo Semantica + No Grafo | 1-3 secondi |
| Claude 3.5 + Solo Semantica + No Grafo | 0.8-2 secondi |

**Nota**: Primi query su KB nuova possono essere più lente (cold start Qdrant).

### 6.4 Interpretare la Risposta

La risposta RAG è composta da:

#### A. Risposta Testuale

Sintesi in linguaggio naturale della risposta alla tua domanda.

**Esempio**:
```
Nel contratto "Fornitura Server 2025.pdf" (pagina 8), la penale per 
ritardi nella consegna è specificata come:

"In caso di ritardo superiore a 7 giorni lavorativi rispetto alla data
concordata, il Fornitore corrisponderà al Cliente una penale pari al 
2% del valore della fornitura per ogni settimana di ritardo, fino ad 
un massimo del 10%."

Pertanto, la penale è del **2% settimanale, massimo 10%**.
```

#### B. Citazioni (Sources)

Sotto la risposta, vedi le **"Fonti"** (Sources) con:

- **Nome file**: Cliccabile per aprire il documento originale
- **Pagina**: Numero pagina esatta
- **Chunk ID**: Identificativo univoco del paragrafo
- **Rilevanza**: Score 0-100% (quanto è pertinente alla domanda)
- **Testo Quotato**: Il passaggio esatto citato (grassetto)

**Esempio**:
```
📄 Fornitura Server 2025.pdf | Pagina 8 | Rilevanza: 94%
──────────────────────────────────────────────────────
"In caso di ritardo superiore a 7 giorni lavorativi rispetto 
alla data concordata, il Fornitore corrisponderà al Cliente 
una penale pari al 2% del valore della fornitura per ogni 
settimana di ritardo, fino ad un massimo del 10%."
```

**Clicca sulla citazione** per:
- Aprire il PDF alla pagina esatta
- Evidenziare il passaggio citato
- Vedere contesto prima/dopo

#### C. Knowledge Graph Traversal

Se hai abilitato **Espansione Grafo**, vedrai una sezione **"Entità Correlate"**:

```
🔗 Entità trovate nel grafo legale:
  • PARTY: Fornitore → Acme SpA [doc: Fornitura Server 2025.pdf]
  • AMOUNT: 2% → €50.000 (totale contratto: €2.500.000) [doc: Fornitura Server 2025.pdf]
  • DATE: Data concordata → 15 marzo 2025 [doc: Fornitura Server 2025.pdf]
  • CLAUSE: Penale → ART. 8.3 "Ritardi e Penali" [doc: Fornitura Server 2025.pdf]
```

Clicca su un'entità per:
- Vedere tutti i documenti che la menzionano
- Esplorare relazioni (es. tutte le PENALTY legate a questa PARTY)

#### D. Hallucination Risk Score

In basso a destra, vedi un badge colorato:

- 🟢 **Rischio Basso (0-20%)**: Risposta completamente ancorata a citazioni verificabili
- 🟡 **Rischio Medio (21-50%)**: Risposta parzialmente verificabile, contiene inferenze
- 🔴 **Rischio Alto (51-100%)**: Risposta non supportata da citazioni, possibile allucinazione

**Se vedi 🔴 Rischio Alto**:
- ❌ **NON** fidarti della risposta
- ✅ Leggi manualmente le citazioni
- ✅ Prova a riformulare la domanda in modo più specifico
- ✅ Verifica che la KB contenga documenti rilevanti

### 6.5 Salvare una Query

1. Dopo aver ricevuto la risposta, clicca su **"Salva Query"** (icona stella)
2. Dai un nome alla query, es: "Penali per ritardi - Acme"
3. Aggiungi tag (opzionale): "penali", "acme", "2025"
4. Clicca **"Salva"**

**Perché salvare**:
- Riutilizzare query complesse
- Condividere con colleghi
- Creare "template" per analisi ricorrenti

**Recuperare query salvate**:
- Sidebar → **Chat** → Tab **"Query Salvate"**
- Clicca per rieseguire (con documenti aggiornati)

---

## 7. Chat Conversazionale

### 7.1 Cos'è la Chat RAG?

La **Chat** ti permette di:

- ✅ Fare **domande di follow-up** (il sistema ricorda il contesto)
- ✅ **Approfondire** passaggi specifici delle risposte precedenti
- ✅ **Confrontare** risposte tra diverse KB
- ✅ **Salvare conversazioni** per audit o training

**Esempio di conversazione**:

```
Utente: Quali contratti con Acme SpA abbiamo firmato nel 2025?
Assistente: Ho trovato 3 contratti:
  1. Fornitura Server (€2.5M, 15/03/2025)
  2. Manutenzione Software (€500K, 20/03/2025)
  3. Consulenza Cloud (€800K, 10/04/2025)

Utente: Dammi più dettagli sul primo.
Assistente: Il contratto "Fornitura Server" prevede:
  - Oggetto: 50 server rack Dell PowerEdge R750
  - Importo: €2.500.000 + IVA
  - Consegna: 30 giorni dalla firma (entro 14/04/2025)
  - Penali: 2% settimanale per ritardi (max 10%)
  - Garanzia: 3 anni on-site
  [Fonte: Fornitura Server 2025.pdf, pp. 1-3]

Utente: Questa penale è più alta o più bassa rispetto agli altri contratti Acme?
Assistente: Confrontando con gli altri 2 contratti Acme:
  - Manutenzione Software: penale 1.5% settimanale (più bassa)
  - Consulenza Cloud: penale 2.5% settimanale (più alta)
  
  La penale del 2% è nel **range medio** dei vostri contratti con Acme.
  [Fonti: Manutenzione Software 2025.pdf p.5, Consulenza Cloud 2025.pdf p.7]
```

### 7.2 Avviare una Chat

1. Sidebar → **Chat** → **"+ Nuova Conversazione"**
2. Seleziona la **Knowledge Base** (o più KB, vedi §7.4)
3. Scrivi la prima domanda
4. Clicca **"Invia"** (o `Ctrl+Enter`)

### 7.3 Comandi Speciali in Chat

Durante una conversazione, puoi usare:

| Comando | Funzione | Esempio |
|---|---|---|
| `/reset` | Cancella contesto conversazione (ricomincia da zero) | `/reset` |
| `/kb <nome>` | Cambia KB senza chiudere chat | `/kb Contratti Fornitori 2025` |
| `/export` | Esporta conversazione in PDF/Markdown | `/export pdf` |
| `/citation <N>` | Mostra citazione completa numero N | `/citation 3` |
| `/graph <entità>` | Mostra grafo per un'entità | `/graph Acme SpA` |
| `/help` | Mostra elenco comandi | `/help` |

### 7.4 Chat Multi-KB

Puoi fare domande che spaziano su **più Knowledge Base** contemporaneamente:

1. Nuova Conversazione → **"Aggiungi KB"** (checkbox multiple)
2. Seleziona 2-5 KB (es: "Contratti 2024", "Contratti 2025")
3. Fai domande comparative:
   - *"Confronta le penali per ritardi tra i contratti 2024 e 2025"*
   - *"Quali clausole sono cambiate tra le due annualità?"*

**Limitazioni**:
- Massimo 5 KB contemporaneamente (performance)
- Tempo risposta aumenta linearmente con il numero di KB

### 7.5 Salvare e Condividere Conversazioni

#### Salvare

1. Durante o a fine chat, clicca **"Salva Conversazione"** (icona floppy disk)
2. Dai un nome: es. "Analisi Penali Acme - 12 Mag 2026"
3. Aggiungi tag
4. Clicca **"Salva"**

**Le conversazioni salvate**:
- Restano nella tua lista personale (sidebar → Chat → Salvate)
- Sono cercabili per nome/tag
- Mantengono TUTTO il contesto (domande, risposte, citazioni, timestamp)

#### Condividere

1. Conversazione Salvata → Pulsante **"Condividi"** (icona link)
2. Genera **link di condivisione** (valido per il workspace)
3. Copia link e invia ai colleghi

**Permessi condivisione**:
- Solo utenti con accesso alla stessa KB possono aprire il link
- I Viewer possono leggere, non modificare
- Owner/Admin possono eliminare conversazioni condivise da altri

---

## 8. Annotazioni Collaborative

### 8.1 Cos'è un'Annotazione?

Un'**annotazione** è una **nota** che puoi aggiungere a un passaggio specifico di un documento, visibile in tempo reale a tutti gli utenti del workspace.

**Casi d'uso**:
- ✅ Evidenziare clausole problematiche
- ✅ Richiedere chiarimenti al team legale
- ✅ Marcare sezioni da revisionare
- ✅ Collegare riferimenti incrociati tra contratti

### 8.2 Creare un'Annotazione

1. Apri un **documento** nel visualizzatore
2. **Seleziona** il testo che vuoi annotare con il mouse
3. Appare un popup → Clicca **"Annota"** (icona fumetto)
4. Compila il form:
   - **Testo Nota** (obbligatorio): La tua osservazione
   - **Tipo**: Commento, Domanda, Alert, Approvazione
   - **Tag**: Keyword per filtrare (es: "penale", "revisione")
   - **Menziona**: @nomeutente per notificare un collega
5. Clicca **"Salva Annotazione"**

**Visualizzazione**:
- Il testo annotato appare **evidenziato in giallo**
- Clicca sull'evidenziazione per vedere la nota
- Le annotazioni compaiono anche nel tab **"Annotazioni"** del documento

### 8.3 Rispondere a un'Annotazione (Thread)

1. Clicca sull'annotazione esistente
2. Nella card che si apre, campo **"Rispondi…"**
3. Scrivi la tua risposta
4. (Opzionale) Menziona qualcuno con `@nome`
5. Clicca **"Invia"**

**Thread conversazionale**:
- Ogni annotazione può avere risposte illimitate
- Tutte le risposte sono timestampate e firmate
- I menzionati ricevono notifica real-time

### 8.4 Risolvere un'Annotazione

Quando la questione è chiusa:

1. Apri l'annotazione
2. Clicca **"Risolvi"** (icona check)
3. L'annotazione diventa grigia (archiviata)
4. Puoi comunque vederla attivando filtro **"Mostra Risolte"**

### 8.5 Filtrare Annotazioni

Nel tab **"Annotazioni"** del documento:

- **Per Tipo**: Commenti, Domande, Alert, Approvazioni
- **Per Autore**: Solo tue, solo di un collega
- **Per Tag**: Filtra per keyword
- **Per Stato**: Aperte, Risolte, Tutte

### 8.6 Notifiche Real-Time

Quando qualcuno:
- Aggiunge un'annotazione su un documento che stai visualizzando
- Ti menziona in un'annotazione
- Risponde a un thread che hai creato

**Ricevi notifica**:
- 🔔 Badge rosso sull'icona notifiche (header)
- 🟢 Evidenziazione nuova nell'elenco annotazioni
- 📧 Email (se abilitato in Impostazioni)

**WebSocket Real-Time**:
- Le annotazioni appaiono **istantaneamente** (entro 200ms)
- Non serve ricaricare la pagina
- Vedi anche **chi sta scrivendo** (indicatore "X sta digitando...")

---

## 9. Confronto Multi-Contratto

### 9.1 Cos'è il Confronto Multi-Contratto?

La funzione **Confronto** ti permette di:

- ✅ **Visualizzare affiancati** 2-5 contratti contemporaneamente
- ✅ **Sincronizzare lo scroll** (scorrere tutti insieme o separatamente)
- ✅ **Evidenziare differenze** nelle clausole chiave
- ✅ **Fare domande comparative** (es: "Quale contratto ha la penale più alta?")

**Caso d'uso tipico**:
Hai 3 versioni di un contratto con un fornitore (bozza, revisione, finale). Vuoi vedere esattamente cosa è cambiato tra le versioni.

### 9.2 Avviare un Confronto

1. Sidebar → **Confronto** (🔀)
2. Clicca **"+ Nuovo Confronto"**
3. Seleziona **2-5 documenti** dalla lista
   - Usa filtri per trovare contratti simili (es: stessa controparte, stessa categoria)
4. Clicca **"Avvia Confronto"**

**Layout**:
```
┌────────────┬────────────┬────────────┐
│  Contratto │  Contratto │  Contratto │
│     A      │     B      │     C      │
│  (2023)    │  (2024)    │  (2025)    │
├────────────┼────────────┼────────────┤
│  Pagina 1  │  Pagina 1  │  Pagina 1  │
│  ...       │  ...       │  ...       │
│  ...       │  ...       │  ...       │
└────────────┴────────────┴────────────┘
     [Scroll Sincronizzato: ON/OFF]
     [Evidenzia Differenze: ON/OFF]
```

### 9.3 Modalità di Confronto

#### A. Scroll Sincronizzato

- **ON** (default): Scorrere in un contratto scorre tutti gli altri
- **OFF**: Scorrimento indipendente

Shortcut: `Ctrl+Shift+S`

#### B. Evidenziazione Differenze

- **ON**: Paragrafi diversi tra i documenti appaiono evidenziati in colori
  - 🟢 Verde: Presente solo in questo documento (aggiunto)
  - 🔴 Rosso: Presente negli altri, assente qui (rimosso)
  - 🟡 Giallo: Testo modificato
- **OFF**: Nessuna evidenziazione

**Algoritmo**: Diff a livello di paragrafo (non carattere per carattere).

Shortcut: `Ctrl+Shift+D`

#### C. Vista Unificata (Merge View)

Clicca **"Vista Unificata"** per passare da layout affiancato a:

```
┌──────────────────────────────────────┐
│  Contratto A (2023)                  │
│  Paragrafo 1: "Le penali sono..."   │
├──────────────────────────────────────┤
│  Contratto B (2024)                  │
│  Paragrafo 1: "Le penali sono..."   │  ← Identico
├──────────────────────────────────────┤
│  Contratto C (2025)                  │
│  Paragrafo 1: "Le penali ammontano..."  ← Cambiato
└──────────────────────────────────────┘
```

Shortcut: `Ctrl+Shift+U`

### 9.4 Domande Comparative RAG

Durante il confronto, puoi fare domande come:

- *"Quali sono le differenze nelle clausole di penale tra questi 3 contratti?"*
- *"Quale contratto ha l'importo più alto?"*
- *"In quale versione è stata aggiunta la clausola di riservatezza?"*

Il sistema:
1. Interroga tutti i documenti del confronto
2. Aggrega le risposte
3. Evidenzia le differenze chiave
4. Fornisce citazioni per ognuna

**Esempio Risposta**:
```
Confronto delle penali:

• Contratto A (2023): 1.5% settimanale, max 10%
  [Fonte: Contratto_A_2023.pdf, p. 8]

• Contratto B (2024): 2% settimanale, max 10%
  [Fonte: Contratto_B_2024.pdf, p. 9]

• Contratto C (2025): 2% settimanale, max 15%
  [Fonte: Contratto_C_2025.pdf, p. 10]

Evoluzione: La penale settimanale è aumentata dal 1.5% al 2% tra 2023 e 2024,
e il massimo dal 10% al 15% nel 2025.
```

### 9.5 Esportare il Confronto

1. Clicca **"Esporta Confronto"**
2. Scegli formato:
   - **PDF**: Report con screenshot affiancati + tabella differenze
   - **Excel**: Tabella con clausole chiave per ogni contratto
   - **Markdown**: Per documentazione tecnica
3. Clicca **"Scarica"**

**Report include**:
- Metadati dei contratti confrontati
- Lista differenze testuali (con % similarità)
- Tabella comparativa clausole (importi, date, penali, parti)
- Timestamp e autore del confronto

---

## 10. Interpretare i Risultati

### 10.1 Score di Rilevanza

Ogni chunk restituito dal RAG ha un **score 0-100%**:

| Range | Significato |
|---|---|
| **90-100%** | Estremamente rilevante, risposta quasi certa |
| **70-89%** | Rilevante, buona confidenza |
| **50-69%** | Parzialmente rilevante, richiede verifica |
| **< 50%** | Bassa rilevanza, risultato spurio |

**Cosa fare con score bassi**:
- ❌ Non fidarti ciecamente della risposta
- ✅ Leggi i chunk manualmente
- ✅ Prova a riformulare la domanda con parole chiave diverse

### 10.2 Coverage Score

Il **Coverage** indica "quanta" della tua domanda è coperta dalle citazioni trovate.

**Esempio**:
- Domanda: *"Quali sono le penali e le garanzie previste nel contratto?"*
- Risposta trova solo citazioni sulle penali, nessuna sulle garanzie
- Coverage: **50%** (1 su 2 argomenti)

**Badge Coverage**:
- 🟢 **100%**: Tutti gli argomenti della domanda hanno citazioni
- 🟡 **50-99%**: Risposta parziale
- 🔴 **< 50%**: Risposta molto incompleta

**Azione consigliata con coverage basso**:
- Dividi la domanda in 2 query separate (una per le penali, una per le garanzie)

### 10.3 Citazioni Verbatim vs Parafrasi

Il sistema differenzia:

- **Citazione Verbatim** (🔹 blu): Testo copiato **esattamente** dal documento
- **Parafrasi** (🔸 arancione): Riformulazione del LLM (basata su citazione, ma non identica)

**Regola d'oro**: In contesti legali/forensi, **fidati SOLO delle citazioni verbatim**.

### 10.4 Grafici del Knowledge Graph

Quando apri la vista **Knowledge Graph** di un documento, vedi:

```
     [PARTY: Acme SpA]
            |
            | HAS_PENALTY
            v
    [PENALTY: 2% settimanale]
            |
            | RELATED_TO
            v
    [CLAUSE: Art. 8.3 Ritardi]
            |
            | MENTIONS_DATE
            v
     [DATE: 15 marzo 2025]
```

**Interazioni**:
- **Clicca su un nodo**: Mostra tutti i documenti che menzionano quell'entità
- **Clicca su un arco**: Mostra il tipo di relazione e il contesto testuale
- **Drag nodo**: Riorganizza il layout
- **Zoom**: Rotella mouse o pinch trackpad

**Tipi di relazioni** (10 tipi legali):
1. `HAS_PENALTY`: Parte → Penale
2. `HAS_JURISDICTION`: Contratto → Giurisdizione
3. `SIGNED_ON`: Contratto → Data
4. `PARTY_ROLE`: Parte → Ruolo (Cliente, Fornitore, Garante)
5. `AMOUNT`: Clausola → Importo
6. `REFERENCES_CLAUSE`: Clausola → Altra Clausola
7. `GOVERNED_BY`: Contratto → Normativa
8. `EXPIRES_ON`: Contratto → Data Scadenza
9. `RELATED_TO`: Generico collegamento tra entità
10. `MENTIONS_ENTITY`: Documento → Entità

### 10.5 Capire gli Errori

#### Errore: "Nessun risultato trovato"

**Cause possibili**:
1. La KB non contiene documenti con informazioni rilevanti
2. La domanda usa terminologia troppo tecnica/specifica
3. I documenti sono stati indicizzati male (OCR fallito)

**Soluzioni**:
1. Verifica che la KB contenga documenti pertinenti
2. Riformula con sinonimi o parole più comuni
3. Prova modalità "Solo Keyword" invece di "Ibrida"

#### Errore: "Query timeout (>30s)"

**Cause**:
1. KB troppo grande (>10.000 documenti)
2. Espansione grafo troppo profonda (3 hop su KB grande)
3. Qdrant sovraccarico (altre query in parallelo)

**Soluzioni**:
1. Dividi la KB in sotto-KB più piccole
2. Riduci espansione grafo a 1 hop
3. Riprova tra qualche minuto

#### Errore: "Hallucination risk HIGH"

**Cause**:
1. La domanda è troppo vaga ("Dimmi tutto sul contratto")
2. Nessuna citazione supporta la risposta generata
3. Il LLM sta "inventando" dettagli

**Soluzioni**:
1. **Non fidarti della risposta**
2. Fai domande più specifiche
3. Verifica manualmente le citazioni

---

## 11. Best Practices

### 11.1 Scrivere Domande Efficaci

✅ **Buone domande** (specifiche, verificabili):
- *"Qual è l'importo della penale per ritardi nel contratto Acme SpA del 2025?"*
- *"Elenca tutte le clausole di riservatezza con durata >3 anni"*
- *"In quale documento è menzionata la giurisdizione di Milano?"*

❌ **Domande problematiche** (vaghe, non verificabili):
- *"Dimmi tutto sui contratti"* → Troppo vaga
- *"Cosa ne pensi del contratto Acme?"* → Soggettiva, non fattuale
- *"È un buon contratto?"* → Valutazione, non RAG

### 11.2 Organizzare Knowledge Bases

**Strategia consigliata**:

1. **Per Anno**: "Contratti 2024", "Contratti 2025"
   - Pro: Facile confrontare evoluzioni annuali
   - Contro: Difficile analizzare un fornitore specifico

2. **Per Controparte**: "Acme SpA", "Beta Srl", "Gamma Inc."
   - Pro: Analisi mirata per partner commerciale
   - Contro: Molte KB se hai centinaia di fornitori

3. **Per Categoria**: "Forniture IT", "Appalti Edili", "Consulenze Legali"
   - Pro: Confronti omogenei
   - Contro: Può crescere molto grande

**Best Practice**: Usa una combinazione, es:
- KB principale per anno
- KB secondarie per categorie/fornitori critici

### 11.3 Gestire Documenti Grandi

**Contratti > 500 pagine**:

1. Considera di **spezzare** il PDF in sezioni logiche:
   - Parte Generale (pp. 1-50)
   - Allegati Tecnici (pp. 51-300)
   - Allegati Finanziari (pp. 301-500)
2. Carica ogni sezione come documento separato
3. Aggiungi metadati coerenti (stesso titolo base + suffisso)
4. Crea KB dedicata per quel contratto

**Vantaggi**:
- Query più veloci (meno chunk da processare)
- Citazioni più precise (pagina relativa alla sezione)
- Parallelismo nell'indicizzazione

### 11.4 Backup e Audit Trail

**Per utenti Owner/Admin**:

1. **Esporta KB regolarmente**:
   - KB → Impostazioni → Esporta KB (ZIP con JSON + PDF originali)
   - Frequenza: Mensile o dopo modifiche maggiori

2. **Audit Log**:
   - Sidebar → Audit Log
   - Verifica chi ha caricato/eliminato/modificato documenti
   - Esporta log in CSV per compliance GDPR

3. **Conversazioni importanti**:
   - Salva come PDF con citazioni
   - Archivia esternamente per contenzioso legale

### 11.5 Privacy e Sicurezza

✅ **Cosa il sistema fa automaticamente**:
- ✅ Cifratura TLS per tutti i dati in transito
- ✅ Embedding locali di default (Ollama, zero cloud)
- ✅ Isolamento multi-tenant (non vedi documenti di altri workspace)
- ✅ Audit trail completo di ogni azione

❌ **Cosa NON fare**:
- ❌ Condividere link di conversazioni su canali pubblici (Slack pubblico, email non cifrate)
- ❌ Caricare documenti con dati personali se GDPR non è configurato (chiedi all'admin)
- ❌ Abilitare provider cloud se i contratti contengono segreti industriali (preferisci Ollama)

---

## 12. Risoluzione Problemi

### 12.1 "Non riesco a fare login"

**Sintomo**: Email/password corretti, ma errore "Credenziali non valide"

**Soluzioni**:
1. Verifica che l'account sia stato **attivato** (controlla email di verifica)
2. Prova **"Password dimenticata?"** per resettare
3. Contatta l'admin per verificare che l'utente non sia stato **disabilitato**
4. Svuota cache browser (`Ctrl+Shift+Del` → Cancella cookie)

### 12.2 "Documento caricato ma non compare"

**Sintomo**: Upload 100%, ma documento non appare nella lista

**Soluzioni**:
1. Ricarica la pagina (`F5`)
2. Verifica che il filtro **"Stato"** non sia impostato su "Solo Elaborati" (se in elaborazione, non compare)
3. Controlla che il formato sia supportato (vedi §4.1)
4. Se persiste, contatta admin (potrebbe essere errore di elaborazione backend)

### 12.3 "Query RAG ritorna sempre 'Nessun risultato'"

**Sintomo**: Tutte le domande su una KB ritornano vuoto, anche domande ovvie

**Cause + Soluzioni**:

1. **KB vuota**: Verifica che contenga documenti indicizzati
   - KB → Tab Documenti → Verifica lista non vuota

2. **Indicizzazione fallita**: Vedi stato elaborazione
   - Se "Errore", ri-carica il documento

3. **Qdrant disconnesso**: Verifica stato servizi
   - Home → Stato Servizi → Qdrant deve essere verde
   - Se rosso, contatta admin

4. **Threshold troppo alto**: Abbassa soglia confidence
   - Query → Parametri Avanzati → Threshold: 0.5 invece di 0.7

### 12.4 "WebSocket disconnesso, annotazioni non sincronizzate"

**Sintomo**: Vedi alert "Connessione WebSocket persa" in alto

**Soluzioni**:
1. Controlla connessione internet (ping google.com)
2. Ricarica la pagina (`F5`) — riconnessione automatica
3. Se persiste, verifica firewall aziendale (porta WebSocket bloccata)
4. Contatta admin per verificare stato servizio Redis (backend WebSocket)

### 12.5 "Latenza query molto alta (>30s)"

**Sintomo**: Query semplici impiegano 30+ secondi

**Cause + Soluzioni**:

1. **Provider cloud lento**: Passa a Ollama
   - KB → Impostazioni → Modello LLM → Ollama

2. **Espansione grafo eccessiva**: Riduci hop
   - Query → Parametri Avanzati → Espansione Grafo: 1 hop

3. **KB sovra-dimensionata**: Dividi in sotto-KB
   - Se >5.000 documenti, crea KB per anno/categoria

4. **Qdrant cold start**: Prima query su KB nuova è più lenta
   - Attendi 5-10 secondi, poi riprova — dovrebbe essere normale

### 12.6 "File upload fallisce a 99%"

**Sintomo**: Barra progresso arriva a 99%, poi errore "Upload failed"

**Soluzioni**:
1. Verifica dimensione file < 200 MB
2. Controlla connessione stabile (no WiFi instabile)
3. Prova browser diverso (Chrome consigliato)
4. Se PDF, verifica non sia **protetto da password** (rimuovi protezione prima)
5. Se persiste, contatta admin (verifica storage lato server)

### 12.7 "Non vedo documenti di un collega"

**Sintomo**: Un collega dice di aver caricato documenti, ma non li vedi

**Cause**:
1. **Workspace diverso**: Verifica di essere nello stesso workspace
   - Header → Workspace Selector → Controlla nome
2. **Permessi**: Se sei Viewer, vedi solo documenti di KB a cui hai accesso
   - Chiedi all'admin di aggiungerti alla KB
3. **Documento in elaborazione**: Attendi fine elaborazione
   - Il collega vede subito, ma altri vedono dopo indicizzazione

### 12.8 "Citazioni non corrispondono al PDF"

**Sintomo**: Clicco su citazione, ma testo evidenziato non corrisponde

**Cause**:
1. **PDF scansionato con OCR**: Errori di riconoscimento testo
   - Verifica qualità scansione, eventualmente ri-scansiona a 300+ DPI
2. **Versioni diverse**: PDF modificato dopo indicizzazione
   - Re-indicizza il documento (Elimina + Ricarica)
3. **Bug mapping pagine**: Raramente, offset pagina errato
   - Segnala all'admin con screenshot

### 12.9 Contatti Supporto

**Per problemi non risolti**:

1. **Email supporto**: archivio-parlante-support@tuaazienda.it (se configurato dall'admin)
2. **Admin interno**: Vedi sidebar → "Contatti" → Nome admin del workspace
3. **Documentazione tecnica**: `docs/RUNBOOK.md` (per admin/DevOps)

**Quando segnali un problema, includi**:
- Browser e versione (es: Chrome 125)
- Timestamp esatto dell'errore
- Screenshot o video del problema
- Passi per riprodurlo

---

## Appendice A: Shortcut Tastiera

| Shortcut | Funzione |
|---|---|
| `Ctrl+K` | Ricerca globale |
| `Ctrl+Q` | Nuova query RAG |
| `Ctrl+Enter` | Invia messaggio chat |
| `Ctrl+/` | Mostra/nascondi sidebar |
| `Alt+H` | Vai a Home |
| `Alt+K` | Vai a Knowledge Bases |
| `Alt+D` | Vai a Documenti |
| `Alt+C` | Vai a Chat |
| `Alt+M` | Vai a Confronto |
| `Alt+A` | Vai a Audit Log |
| `Esc` | Chiudi popup/modal |
| `Ctrl+Shift+S` | Toggle scroll sincronizzato (confronto) |
| `Ctrl+Shift+D` | Toggle evidenziazione differenze (confronto) |
| `Ctrl+Shift+U` | Toggle vista unificata (confronto) |

---

## Appendice B: Glossario

| Termine | Definizione |
|---|---|
| **RAG** | Retrieval-Augmented Generation — tecnica AI per rispondere con citazioni da documenti |
| **Knowledge Base (KB)** | Collezione di documenti indicizzati per interrogazione RAG |
| **Chunk** | Paragrafo/sezione di documento (unità di indicizzazione) |
| **Embedding** | Vettore numerico 768-dim che rappresenta il significato semantico di un chunk |
| **Hybrid Search** | Combinazione di ricerca semantica (dense) e keyword (sparse) |
| **Knowledge Graph** | Grafo di entità legali (parti, date, importi, clausole) estratte dai documenti |
| **Hallucination** | Risposta LLM inventata, non ancorata a citazioni reali |
| **Score** | Punteggio 0-100% che indica quanto un chunk è rilevante per la query |
| **Coverage** | Percentuale degli argomenti della domanda coperti dalle citazioni |
| **Provider LLM** | Servizio che fornisce il modello linguistico (Ollama, Claude, GPT-4, ecc.) |
| **WebSocket** | Protocollo per comunicazione real-time (annotazioni sincronizzate) |
| **Workspace** | Ambiente multi-tenant isolato (documenti, utenti, KB) |
| **Owner** | Ruolo massimo: crea workspace, gestisce billing, assegna permessi |
| **Admin** | Gestisce utenti, KB, documenti all'interno di un workspace |
| **Member** | Può caricare documenti, fare query, annotare |
| **Viewer** | Solo lettura: può fare query e visualizzare documenti |

---

**Versione documento**: 1.0.0  
**Ultima revisione**: 2026-05-12  
**Autore**: Archivio Parlante Team  
**Licenza**: Internal Use Only — Do Not Distribute
