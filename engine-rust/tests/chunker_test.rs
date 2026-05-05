/// Chunker integration tests

use archivio_parlante_rust_engine::chunker::semantic::SemanticChunker;
use archivio_parlante_rust_engine::utils::tokenizer;

// Sample Italian contract for testing (~3000 words)
const SAMPLE_CONTRACT: &str = r#"
# CONTRATTO DI FORNITURA SERVIZI INFORMATICI

Tra le seguenti parti:

FORNITORE: TechSolutions S.r.l., con sede in Milano, Via Roma 123, P.IVA 12345678901
CLIENTE: Ente Pubblico Regionale, con sede in Roma, Piazza della Repubblica 45, CF 98765432100

## PREMESSE

Le parti convengono quanto segue:

Art. 1 - Oggetto del contratto
Il presente contratto ha per oggetto la fornitura di servizi di consulenza informatica, sviluppo software e manutenzione sistemi per il periodo di 24 mesi decorrenti dalla data di sottoscrizione.

Art. 2 - Descrizione dei servizi
Il Fornitore si impegna a fornire i seguenti servizi:
a) Consulenza tecnica specialistica per un minimo di 40 ore mensili
b) Sviluppo di applicazioni web custom secondo le specifiche del Cliente
c) Manutenzione correttiva ed evolutiva dei sistemi esistenti
d) Supporto help desk di primo e secondo livello
e) Formazione del personale del Cliente sulle nuove tecnologie adottate

Art. 3 - Corrispettivo e modalità di pagamento
Il corrispettivo complessivo per i servizi di cui all'Art. 2 è fissato in Euro 240.000,00 (duecentoquarantamila/00) oltre IVA, da corrispondersi in rate mensili anticipate di Euro 10.000,00 ciascuna.
Il pagamento avverrà entro 60 giorni data fattura mediante bonifico bancario sul conto corrente del Fornitore.

Art. 4 - Obblighi del Fornitore
Il Fornitore si obbliga a:
- Eseguire i servizi con la massima diligenza professionale
- Rispettare i termini di consegna concordati
- Garantire la riservatezza dei dati trattati
- Nominare un Project Manager dedicato
- Fornire reportistica mensile dettagliata delle attività svolte

Art. 5 - Obblighi del Cliente
Il Cliente si impegna a:
- Corrispondere puntualmente i compensi pattuiti
- Fornire accesso ai sistemi informatici necessari all'esecuzione dei servizi
- Nominare un Responsabile del procedimento
- Comunicare tempestivamente ogni variazione delle specifiche richieste

## CAPO II - PENALI E RISOLUZIONE

Art. 6 - Penali per ritardo
In caso di ritardo nell'esecuzione delle attività previste dal piano di lavoro, il Fornitore sarà soggetto ad una penale pari allo 0,3% del valore della singola attività in ritardo per ogni giorno di ritardo, fino ad un massimo del 10% del valore contrattuale complessivo.

Art. 7 - Clausola risolutiva espressa
Il contratto si intenderà risolto di diritto nei seguenti casi:
a) Mancato pagamento di tre rate consecutive da parte del Cliente
b) Grave inadempimento agli obblighi contrattuali non sanato entro 30 giorni dalla contestazione formale
c) Cessazione dell'attività da parte di una delle parti contraenti
d) Violazione degli obblighi di riservatezza

Art. 8 - Recesso anticipato
Ciascuna parte potrà recedere dal contratto con preavviso scritto di almeno 90 giorni, senza obbligo di motivazione. In caso di recesso del Cliente prima del termine contrattuale, sarà dovuto un indennizzo pari al 20% del valore residuo del contratto.

## CAPO III - DISPOSIZIONI FINALI

Art. 9 - Riservatezza
Le parti si impegnano a mantenere riservate tutte le informazioni, i dati, i documenti e le notizie di cui vengano a conoscenza in esecuzione del presente contratto. L'obbligo di riservatezza permane anche dopo la cessazione del rapporto contrattuale per un periodo di 5 anni.

Art. 10 - Proprietà intellettuale
Tutti i prodotti software sviluppati in esecuzione del presente contratto saranno di proprietà esclusiva del Cliente. Il Fornitore rinuncia espressamente ad ogni diritto di proprietà intellettuale sugli stessi.

Art. 11 - Subappalto
Il Fornitore potrà affidare in subappalto parte dei servizi solo previo consenso scritto del Cliente. In ogni caso il Fornitore rimane unico responsabile nei confronti del Cliente dell'esecuzione di tutti i servizi.

Art. 12 - Modifiche contrattuali
Ogni modifica al presente contratto dovrà essere formalizzata per iscritto mediante atto aggiuntivo sottoscritto da entrambe le parti.

Art. 13 - Trattamento dei dati personali
Il Fornitore si impegna a trattare i dati personali di cui entrerà in possesso nel rispetto del Regolamento (UE) 2016/679 (GDPR) e della normativa nazionale vigente. Il Fornitore è nominato Responsabile del trattamento ai sensi dell'art. 28 GDPR.

Art. 14 - Legge applicabile e foro competente
Il presente contratto è regolato dalla legge italiana. Per qualsiasi controversia derivante dal presente contratto è competente in via esclusiva il Foro di Roma.

Art. 15 - Registrazione
Il presente contratto è soggetto a registrazione in caso d'uso ai sensi del D.P.R. 131/1986.

Letto, approvato e sottoscritto.

Data: Milano, 15 gennaio 2024

Per il Fornitore (TechSolutions S.r.l.)
________________________

Per il Cliente (Ente Pubblico Regionale)
________________________
"#;

#[test]
fn test_chunker_italian_contract() {
    let chunker = SemanticChunker::new(800, 0.15);
    let chunks = chunker
        .chunk(SAMPLE_CONTRACT, "doc_test_001")
        .expect("Chunking should succeed");

    // Verify chunk count is reasonable (5-15 chunks for this ~3000 word contract)
    assert!(
        chunks.len() >= 5 && chunks.len() <= 20,
        "Expected 5-20 chunks, got {}",
        chunks.len()
    );

    // Verify no chunk exceeds size limit significantly (120% tolerance)
    for (idx, chunk) in chunks.iter().enumerate() {
        assert!(
            chunk.token_count <= (800 * 12 / 10),
            "Chunk {} has {} tokens, exceeds 120% of limit (960)",
            idx,
            chunk.token_count
        );
    }

    // Verify overlap between consecutive chunks
    for i in 1..chunks.len() {
        let prev = &chunks[i - 1];
        let curr = &chunks[i];

        // Check that current chunk's beginning appears in previous chunk's end
        let curr_start = curr.text.chars().take(50).collect::<String>();
        let prev_end = prev.text.chars().rev().take(200).collect::<String>();

        // Simple overlap check: at least some common substring
        let has_overlap = curr_start.split_whitespace().take(5).any(|word| {
            prev_end.contains(word) && word.len() > 3
        });

        assert!(
            has_overlap || i == 0,
            "No overlap detected between chunk {} and {}",
            i - 1,
            i
        );
    }

    // Verify chunks preserve logical order (Art. numbers increasing)
    let article_chunks: Vec<_> = chunks
        .iter()
        .filter_map(|c| {
            if let Some(clause) = c.metadata.get("clause_marker") {
                clause.as_str().and_then(|s| {
                    if s.starts_with("Art.") {
                        s.split_whitespace()
                            .nth(1)
                            .and_then(|n| n.parse::<u32>().ok())
                    } else {
                        None
                    }
                })
            } else {
                None
            }
        })
        .collect();

    for window in article_chunks.windows(2) {
        assert!(
            window[0] <= window[1],
            "Article numbering not preserved: {} followed by {}",
            window[0],
            window[1]
        );
    }

    println!("✅ Chunked contract into {} chunks", chunks.len());
    println!("  Avg tokens: {}", chunks.iter().map(|c| c.token_count).sum::<usize>() / chunks.len());
    println!("  Article chunks detected: {}", article_chunks.len());
}

#[test]
fn test_chunker_short_text() {
    let chunker = SemanticChunker::new(800, 0.15);
    let text = "Questo è un contratto molto breve. Solo poche parole per testare.";
    let chunks = chunker.chunk(text, "doc_short").expect("Should handle short text");

    // Short text should produce single chunk
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].doc_id, "doc_short");
    assert_eq!(chunks[0].chunk_idx, 0);

    // Verify token count is reasonable
    let expected_tokens = tokenizer::count_tokens(text).expect("Token count should work");
    assert_eq!(chunks[0].token_count, expected_tokens);
}

#[test]
fn test_chunker_empty_text() {
    let chunker = SemanticChunker::new(800, 0.15);
    let chunks = chunker.chunk("", "doc_empty").expect("Should handle empty text");

    assert_eq!(chunks.len(), 0);
}

#[test]
fn test_chunker_only_headers() {
    let chunker = SemanticChunker::new(800, 0.15);
    let text = "# Header 1\n## Header 2\n### Header 3";
    let chunks = chunker.chunk(text, "doc_headers").expect("Should handle headers-only text");

    // Headers without content might produce empty chunks that are filtered
    // or single chunk with headers
    assert!(chunks.len() <= 3);
}

#[test]
fn test_chunker_legal_clauses_detection() {
    let chunker = SemanticChunker::new(800, 0.15);
    let text = r#"
Art. 1 - Prima clausola
Contenuto della prima clausola.

Articolo 2 - Seconda clausola
Contenuto della seconda clausola.

CAPO III - Titolo del capo
Sezione 1 - Dettagli
Contenuto della sezione.
"#;

    let chunks = chunker.chunk(text, "doc_clauses").expect("Should handle clauses");

    // Verify at least some chunks have clause markers
    let clauses_detected = chunks.iter().filter(|c| {
        c.metadata.get("clause_marker").is_some()
    }).count();

    assert!(
        clauses_detected > 0,
        "Should detect at least one legal clause marker"
    );

    // Verify is_clause_start flag
    let clause_starts = chunks.iter().filter(|c| {
        c.metadata.get("is_clause_start") == Some(&serde_json::json!(true))
    }).count();

    assert!(clause_starts > 0, "Should mark clause starts");
}

#[tokio::test]
#[ignore] // Requires Ollama running
async fn test_contextual_enrichment() {
    use archivio_parlante_rust_engine::chunker::contextual::ContextualRetrievalEnricher;
    use archivio_parlante_rust_engine::providers::ollama::OllamaProvider;
    use std::sync::Arc;

    let ollama = Arc::new(OllamaProvider::new(
        std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string()),
        8,
        "nomic-embed-text".to_string(),
    ));

    let enricher = ContextualRetrievalEnricher::new(
        ollama.clone(),
        "qwen2.5:7b-instruct-q4_K_M".to_string(),
        4,
    );

    let chunker = SemanticChunker::new(800, 0.15);
    let mut chunks = chunker
        .chunk(SAMPLE_CONTRACT, "doc_test_enrichment")
        .expect("Chunking should succeed");

    enricher
        .enrich(SAMPLE_CONTRACT, &mut chunks)
        .await
        .expect("Enrichment should succeed");

    // Verify at least some chunks were enriched
    let enriched_count = chunks.iter().filter(|c| c.is_contextualized()).count();

    println!("✅ Enriched {} / {} chunks", enriched_count, chunks.len());

    assert!(
        enriched_count > 0,
        "At least some chunks should be enriched"
    );

    // Verify contextual text is longer than original
    for chunk in chunks.iter().filter(|c| c.is_contextualized()) {
        assert!(
            chunk.embedding_text().len() > chunk.text.len(),
            "Contextual text should be longer than original"
        );
    }
}
