# ⚡ Guida Rapida — Archivio Parlante

**Tempo di lettura**: 5 minuti  
**Obiettivo**: Dalla prima query al risultato RAG in 10 minuti  
**Versione**: 0.7.0

---

## 📋 Prerequisiti (2 minuti)

Prima di iniziare, assicurati di avere:

- ✅ **Account attivo** (email + password dall'admin)
- ✅ **Browser aggiornato** (Chrome 100+, Firefox 100+, Edge 100+)
- ✅ **Contratto PDF** da analizzare (max 200 MB)

**Non hai un account?** → Chiedi all'amministratore del sistema.

---

## 🚀 Quick Start (8 minuti)

### Step 1: Login (30 secondi)

1. Vai a: `https://archivio-parlante.tuaazienda.it` (o URL fornito)
2. Inserisci **email** e **password**
3. Clicca **"Accedi"**
4. (Primo accesso) Cambia password → almeno 12 caratteri

**Problemi?** → [Troubleshooting Login](#-problemi-comuni)

---

### Step 2: Carica il Tuo Primo Documento (2 minuti)

1. Sidebar → **📄 Documenti**
2. **Trascina** il PDF nell'area "Trascina qui" (oppure clicca **"+ Carica"**)
3. Compila:
   - **Titolo**: `Contratto Acme 2025`
   - **Categoria**: Fornitura (opzionale)
   - **Data Contratto**: (opzionale)
4. Clicca **"Salva Documento"**
5. **Attendi elaborazione** (1-3 minuti per 50 pagine)
   - Barra progresso: Estrazione testo → Chunking → Embedding → Knowledge Graph → Indicizzazione

**✅ Completato** quando stato = `Indicizzato`

---

### Step 3: Crea Knowledge Base (1 minuto)

1. Sidebar → **📚 Knowledge Bases**
2. Clicca **"+ Nuova Knowledge Base"**
3. Compila:
   - **Nome**: `Test KB`
   - **Descrizione**: (opzionale)
   - **Provider LLM**: Ollama (default, gratis)
4. Sezione **"Documenti Iniziali"**:
   - Spunta il documento caricato prima (`Contratto Acme 2025`)
5. Clicca **"Crea Knowledge Base"**
6. Attendi indicizzazione (30 secondi)

**✅ KB pronta** quando badge = `Attiva`

---

### Step 4: Fai la Tua Prima Domanda RAG (1 minuto)

1. Clicca sulla KB appena creata (`Test KB`)
2. Campo **"Fai una domanda…"**
3. Scrivi: `Quali sono le penali previste per ritardi?`
4. Clicca **"Cerca"** (o premi `Enter`)
5. **Attendi risposta** (3-8 secondi)

**Risultato atteso**:
```
Risposta:
Nel contratto "Contratto Acme 2025.pdf" (pagina X), le penali per
ritardi sono specificate come:

"[testo citazione verbatim dal PDF]"

Pertanto, la penale è del [sintesi].

Fonti:
📄 Contratto Acme 2025.pdf | Pagina X | Rilevanza: 94%
"[testo esatto citato]"
```

**✅ Successo!** Hai completato il primo ciclo RAG completo.

---

### Step 5 (Opzionale): Esplora Knowledge Graph (1 minuto)

1. Nel visualizzatore documento (clicca sul nome PDF)
2. Tab **"Knowledge Graph"**
3. Vedi grafo interattivo:
   - **Nodi**: PARTY, AMOUNT, DATE, CLAUSE, PENALTY, JURISDICTION
   - **Archi**: HAS_PENALTY, SIGNED_ON, GOVERNED_BY, ecc.
4. Clicca su un nodo per vedere documenti correlati

---

### Step 6 (Opzionale): Confronta 2 Contratti (2 minuti)

1. Carica un secondo PDF (ripeti Step 2)
2. Aggiungi alla stessa KB (KB → Tab Documenti → + Aggiungi)
3. Sidebar → **🔀 Confronto**
4. Seleziona i 2 PDF
5. Clicca **"Avvia Confronto"**
6. Attiva **"Evidenzia Differenze"**
7. Fai domanda comparativa: `Quale contratto ha la penale più alta?`

---

## 🔥 Workflow Comuni

### Workflow 1: Analisi Singolo Contratto

```
1. Carica PDF → 2. Crea KB → 3. Fai domande → 4. Esporta conversazione
```

**Tempo totale**: 10 minuti (primo contratto), 5 minuti (successivi)

---

### Workflow 2: Confronto Multi-Contratto

```
1. Carica 3-5 PDF → 2. Crea KB con tutti → 3. Confronto → 4. Query comparative
```

**Tempo totale**: 15 minuti (setup), 2 minuti per domanda

---

### Workflow 3: Collaborazione con Annotazioni

```
1. Apri PDF → 2. Seleziona testo → 3. Annota → 4. Menziona collega (@nome) → 5. Real-time sync
```

**Tempo totale**: 30 secondi per annotazione

---

### Workflow 4: Audit Trail (Solo Admin/Owner)

```
1. Sidebar → Audit Log → 2. Filtra per utente/azione → 3. Esporta CSV
```

**Tempo totale**: 2 minuti

---

## 🎯 Best Practices (da memorizzare)

### ✅ Fai Domande Specifiche

| ❌ Vago | ✅ Specifico |
|---|---|
| "Dimmi tutto" | "Qual è l'importo della penale?" |
| "Cosa dice il contratto?" | "Quali clausole riguardano la riservatezza?" |
| "È un buon contratto?" | "Confronta le penali tra questi 3 contratti" |

---

### ✅ Organizza le KB per Scopo

| Scopo | Nome KB | Documenti |
|---|---|---|
| Analisi annuale | "Contratti 2025" | Tutti i contratti dell'anno |
| Analisi fornitore | "Acme SpA" | Solo contratti con Acme |
| Analisi categoria | "Appalti Pubblici" | Solo appalti |

---

### ✅ Verifica Sempre le Citazioni

- **Non fidarti** della sintesi testuale del LLM
- **Clicca** sulle citazioni per vedere il PDF originale
- **Controlla** il badge **Hallucination Risk**:
  - 🟢 Verde = Sicuro
  - 🔴 Rosso = Verifica manualmente

---

### ✅ Usa Ollama per Privacy, Cloud per Velocità

| Provider | Quando usarlo | Latenza | Costo |
|---|---|---|---|
| **Ollama** (default) | Documenti riservati, uso quotidiano | 3-8s | Gratis |
| **Claude 3.5** | Analisi critiche, massima accuratezza | 1-3s | ~€0.01/query |
| **Qwen (DeepSeek)** | Compromesso velocità/costo | 2-4s | ~€0.002/query |

**Cambiare provider**: KB → Impostazioni → Modello LLM

---

## ❓ Problemi Comuni

### 🔴 "Non riesco a fare login"

**Cause**:
1. Account non verificato → Controlla email
2. Password sbagliata → Usa "Password dimenticata?"
3. Account disabilitato → Contatta admin

**Fix rapido**: Svuota cache browser (`Ctrl+Shift+Del`)

---

### 🔴 "Query ritorna sempre 'Nessun risultato'"

**Cause**:
1. KB vuota o documento non indicizzato
2. Domanda troppo specifica
3. Threshold rilevanza troppo alto

**Fix rapido**:
1. Verifica stato documento = `Indicizzato`
2. Riformula domanda con parole più comuni
3. Query → Parametri Avanzati → Threshold: `0.5`

---

### 🔴 "Upload fallisce al 99%"

**Cause**:
1. File > 200 MB
2. PDF protetto da password
3. Connessione interrotta

**Fix rapido**:
1. Comprimi PDF (riduci qualità immagini)
2. Rimuovi protezione PDF
3. Riprova con connessione stabile

---

### 🔴 "Latenza >30 secondi"

**Cause**:
1. Provider cloud lento
2. KB troppo grande (>5.000 documenti)
3. Espansione grafo eccessiva (3 hop)

**Fix rapido**:
1. Passa a Ollama (KB → Impostazioni)
2. Dividi KB in sotto-KB
3. Query → Parametri Avanzati → Espansione Grafo: `1 hop`

---

### 🔴 "WebSocket disconnesso"

**Sintomo**: Alert rosso "Connessione persa"

**Fix rapido**:
1. Ricarica pagina (`F5`)
2. Controlla connessione internet
3. Se persiste, contatta admin (verifica Redis)

---

## 📞 Supporto

### In caso di problemi non risolti:

1. **Documentazione completa**: [MANUALE_UTENTE.md](./MANUALE_UTENTE.md) (950+ pagine)
2. **Manuale Admin**: [MANUALE_AMMINISTRATORE.md](./MANUALE_AMMINISTRATORE.md) (se sei admin)
3. **Manuale Tecnico**: [MANUALE_TECNICO_OPERATIVO.md](./MANUALE_TECNICO_OPERATIVO.md) (per DevOps)
4. **Email supporto**: archivio-parlante-support@tuaazienda.it
5. **Admin workspace**: Vedi sidebar → "Contatti"

**Quando segnali un problema, includi**:
- Browser + versione (es: Chrome 125)
- Timestamp errore
- Screenshot
- Passi per riprodurre

---

## 🎓 Prossimi Passi

### Dopo aver completato questa guida:

✅ **Livello Base** (completato con questa guida):
- Login, upload, KB, query RAG, citazioni

📚 **Livello Intermedio** (20 minuti):
- Annotazioni collaborative
- Confronto multi-contratto
- Esportazione conversazioni
- Filtri avanzati

🚀 **Livello Avanzato** (1 ora):
- Knowledge Graph profondo
- Chat conversazionale con context
- Configurazione provider LLM cloud
- Audit trail e compliance GDPR

**Leggi**: [MANUALE_UTENTE.md](./MANUALE_UTENTE.md) per dettagli completi.

---

## 📊 Metriche di Successo

**Dopo 1 settimana di uso**:
- ✅ 10+ documenti caricati
- ✅ 3+ KB create (per categoria/anno/fornitore)
- ✅ 50+ query RAG eseguite
- ✅ 5+ conversazioni salvate
- ✅ 20+ annotazioni collaborative

**Dopo 1 mese**:
- ✅ 100+ documenti
- ✅ 10+ KB
- ✅ Confidenza con confronto multi-contratto
- ✅ Uso quotidiano per analisi forense

---

## 🔐 Promemoria Privacy

- ✅ Default **Ollama locale** = nessun dato in cloud
- ✅ Provider cloud = **opt-in** dall'admin
- ✅ Multi-tenant = non vedi documenti di altri workspace
- ✅ Audit trail = ogni azione tracciata (GDPR compliant)

**Se carichi dati sensibili**: Usa SOLO Ollama (default).

---

## ⌨️ Shortcut Essenziali

| Shortcut | Funzione |
|---|---|
| `Ctrl+K` | Ricerca globale |
| `Ctrl+Q` | Nuova query RAG |
| `Ctrl+Enter` | Invia messaggio chat |
| `Esc` | Chiudi popup |
| `Alt+K` | Vai a Knowledge Bases |
| `Alt+D` | Vai a Documenti |

**Shortcut completi**: [MANUALE_UTENTE.md - Appendice A](./MANUALE_UTENTE.md#appendice-a-shortcut-tastiera)

---

## 📖 Risorse Utili

| Documento | Quando leggerlo |
|---|---|
| **GUIDA_RAPIDA.md** (questo file) | ✅ Primo accesso (5 min) |
| **MANUALE_UTENTE.md** | 📚 Uso completo (50 min) |
| **MANUALE_AMMINISTRATORE.md** | 🔧 Se sei admin (1 ora) |
| **MANUALE_TECNICO_OPERATIVO.md** | ⚙️ Se sei DevOps (1 ora) |
| **ARCHITECTURE.md** | 🏗️ Per capire l'architettura |
| **RUNBOOK.md** | 🚨 Troubleshooting avanzato |

---

## ✅ Checklist Completamento

Prima di considerarti "operativo", verifica:

- [ ] Ho fatto login con successo
- [ ] Ho caricato almeno 1 PDF
- [ ] Ho creato almeno 1 KB
- [ ] Ho fatto almeno 1 query RAG
- [ ] Ho visto le citazioni nel PDF originale
- [ ] Ho capito la differenza tra Ollama e provider cloud
- [ ] So dove trovare supporto se ho problemi
- [ ] Ho letto le best practices (domande specifiche)

**Se tutti ✅ → Sei pronto per l'uso quotidiano!**

---

**Versione**: 1.0.0  
**Ultima revisione**: 2026-05-12  
**Tempo medio completamento**: 10 minuti (setup) + 5 minuti/documento  
**Autore**: Archivio Parlante Team

---

## 🎉 Congratulazioni!

Hai completato la Guida Rapida. Ora sei in grado di:
- ✅ Caricare contratti
- ✅ Creare Knowledge Bases
- ✅ Fare domande RAG con citazioni verificabili
- ✅ Interpretare i risultati
- ✅ Risolvere problemi comuni

**Prossimo step**: Esplora funzionalità avanzate nel [MANUALE_UTENTE.md](./MANUALE_UTENTE.md)

**Buon lavoro con Archivio Parlante! 🏛️**
