# 📋 Prompt Forensi per Analisi Contrattuale

**Libreria di prompt pre-configurati per analisi legale italiana**  
**Version**: 1.0  
**Last Updated**: 2026-05-06

---

## Indice

1. [NDA (Non-Disclosure Agreement)](#nda-non-disclosure-agreement)
2. [Appalti Pubblici](#appalti-pubblici)
3. [Contratti di Fornitura](#contratti-di-fornitura)
4. [Licenze Software](#licenze-software)
5. [M&A (Merger & Acquisition)](#ma-merger--acquisition)
6. [Confronto Multi-Contratto](#confronto-multi-contratto)

---

## NDA (Non-Disclosure Agreement)

### 1. Analisi Obblighi di Riservatezza

**Prompt**:
```
Analizza gli obblighi di riservatezza previsti nell'accordo di non divulgazione.
Estrai e riporta testualmente:
1. Definizione di "Informazioni Riservate"
2. Obblighi del Ricevente
3. Eccezioni alla riservatezza
4. Durata degli obblighi
5. Penali in caso di violazione

Per ogni punto, cita la clausola esatta con numero articolo e testo verbatim.
```

**Output atteso**:
- Elenco strutturato con citazioni testuali
- Clausole critiche evidenziate
- Verificazione anti-allucinazione (solo info presenti)

### 2. Verifica Elementi Mancanti

**Prompt**:
```
Verifica se l'NDA contiene le seguenti clausole obbligatorie:
- Definizione di Informazioni Riservate
- Durata dell'accordo
- Foro competente
- Clausola penale
- Restituzione materiali riservati
- Clausola di non sollecitazione

Per ogni clausola, indica:
- ✅ PRESENTE: [cita testo]
- ❌ ASSENTE: indica lacuna potenziale
```

---

## Appalti Pubblici

### 3. Analisi Milestone e Penali

**Prompt**:
```
Estrai tutte le milestone del progetto e le relative penali previste.
Struttura output come tabella:

| Milestone | Termine | Deliverable | Penale Ritardo | Note |
|---|---|---|---|---|

Cita per ogni riga il numero articolo del contratto da cui è estratta l'informazione.
```

### 4. Verifica Conformità Codice Appalti (D.Lgs. 50/2016)

**Prompt**:
```
Analizza il contratto di appalto pubblico e verifica la presenza delle seguenti clausole obbligatorie:
1. Tracciabilità flussi finanziari (art. 3 L. 136/2010)
2. Subappalto (art. 105 D.Lgs. 50/2016)
3. Clausola sociale (art. 50 D.Lgs. 50/2016)
4. Garanzie (art. 103 D.Lgs. 50/2016)
5. Penali (art. 113-bis D.Lgs. 50/2016)
6. Collaudo (art. 102 D.Lgs. 50/2016)

Per ogni punto, indica se presente e cita la clausola. Se assente, segnala come lacuna critica.
```

---

## Contratti di Fornitura

### 5. Analisi Condizioni di Pagamento

**Prompt**:
```
Estrai tutte le condizioni economiche e di pagamento:
1. Prezzo totale (IVA esclusa/inclusa)
2. Modalità di pagamento (bonifico, ri.ba., ecc.)
3. Scadenze pagamento (giorni dalla fattura)
4. Ritenuta d'acconto (se prevista)
5. Garanzie bancarie o fideiussioni richieste
6. Penali per ritardato pagamento

Cita testualmente ogni clausola con riferimento ad articolo/paragrafo.
```

### 6. Garanzie e Assistenza Post-Vendita

**Prompt**:
```
Analizza le garanzie offerte sui beni/servizi:
1. Durata garanzia (mesi/anni)
2. Copertura (difetti di conformità, vizi occulti, ecc.)
3. Modalità di intervento (tempi, costi, procedure)
4. Esclusioni dalla garanzia
5. Assistenza tecnica inclusa
6. SLA (Service Level Agreement) eventuale

Struttura output in formato elenco puntato con citazioni testuali.
```

---

## Licenze Software

### 7. Analisi Diritti d'Uso e Limitazioni

**Prompt**:
```
Estrai dal contratto di licenza software:
1. Tipologia licenza (perpetua, subscription, concurrent users, ecc.)
2. Numero utenti/dispositivi autorizzati
3. Ambito geografico d'uso
4. Diritti di modifica/personalizzazione
5. Diritti di sublicenza
6. Restrizioni (reverse engineering, decompilazione, ecc.)
7. Durata licenza e rinnovo

Per ogni punto, cita la clausola esatta. Segnala eventuali clausole ambigue o mancanti.
```

### 8. Analisi Clausole di Indennizzo e Limitazione Responsabilità

**Prompt**:
```
Analizza le clausole di responsabilità e indennizzo:
1. Limitazione di responsabilità del licenziante (importo massimo, esclusioni)
2. Esclusioni di garanzia (es. "as-is", "no warranty")
3. Obblighi di indennizzo per violazione IP
4. Copertura assicurativa eventuale
5. Clausole di forza maggiore

Valuta il livello di rischio per il licenziatario e indica eventuali clausole potenzialmente vessatorie.
```

---

## M&A (Merger & Acquisition)

### 9. Analisi Rappresentazioni e Garanzie

**Prompt**:
```
Estrai tutte le rappresentazioni e garanzie (reps & warranties) del venditore relative a:
1. Titolarità delle quote/azioni
2. Assenza di gravami
3. Contenzioso pendente o potenziale
4. Conformità fiscale e contributiva
5. Assenza di passività occulte
6. Contratti in essere e loro durata residua
7. Dipendenti e rapporti di lavoro
8. Proprietà intellettuale

Cita per ogni rappresentazione la clausola esatta e evidenzia eventuali limitazioni o eccezioni.
```

### 10. Analisi Earn-out e Aggiustamento Prezzo

**Prompt**:
```
Analizza le clausole di aggiustamento prezzo:
1. Prezzo base iniziale
2. Meccanismo di aggiustamento (working capital, debt-like items, ecc.)
3. Earn-out (condizioni, target, formula di calcolo, durata)
4. Scadenze per la determinazione del prezzo finale
5. Procedura di contestazione/arbitrato

Evidenzia formule di calcolo e condizioni sospensive.
```

---

## Confronto Multi-Contratto

### 11. Confronto Penali tra N Contratti

**Prompt per Multi-Contract API**:
```
Confronta le clausole penali presenti nei contratti selezionati.
Genera tabella comparativa:

| Contratto | Tipo Inadempimento | Penale Prevista | Limite Massimo | Note |
|---|---|---|---|---|

Evidenzia:
- Contratto con penali più severe
- Contratto con penali più favorevoli
- Eventuali difformità rilevanti
```

### 12. Gap Analysis tra Contratti

**Prompt**:
```
Confronta i seguenti aspetti tra i contratti selezionati:
1. Durata contrattuale
2. Termini di pagamento
3. Garanzie offerte
4. Foro competente
5. Legge applicabile
6. Clausole risolutive

Genera tabella comparativa e indica eventuali "information gap" 
(informazioni presenti in alcuni contratti ma assenti in altri).
```

---

## Integrazione con Frontend

### Uso da Interfaccia Web

I prompt possono essere:
1. **Pre-caricati** nel dropdown della chat UI
2. **Personalizzati** dall'utente via textarea
3. **Combinati** con selezione multi-contratto per comparazione

### API Call Example

```json
POST /api/query
{
  "kb_id": "contracts_2024",
  "query": "<prompt da questa libreria>",
  "top_k": 10,
  "rerank_top_n": 5,
  "contract_ids": ["contract_001", "contract_002"]  // per multi-contract
}
```

### Best Practices

1. **Citazioni obbligatorie**: Ogni affermazione deve avere `text_quote` dal documento
2. **Verifica anti-allucinazione**: Self-RAG validator attivo
3. **Lingua italiana**: Prompt e risposte sempre in italiano
4. **Formato strutturato**: Tabelle, elenchi puntati per leggibilità
5. **Clausole critiche**: Evidenziare penali, scadenze, obblighi vincolanti

---

## Disclaimer Legale

⚠️ **IMPORTANTE**: Questi prompt sono strumenti di **supporto all'analisi**, non sostituiscono la consulenza legale professionale. Le risposte generate devono essere:
- Verificate da un avvocato qualificato
- Contestualizzate al caso specifico
- Interpretate alla luce della normativa vigente

Il sistema fornisce **citazioni testuali** per facilitare la verifica, ma la responsabilità dell'interpretazione resta del professionista legale.

---

**Contributi**: Per aggiungere nuovi prompt o categorie contrattuali, aprire PR su GitHub o contattare il team di sviluppo.
