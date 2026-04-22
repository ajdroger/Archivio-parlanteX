"""
Knowledge Graph extraction router

Endpoint for extracting entities and relations from Italian legal contracts.
"""

import time

import structlog
from fastapi import APIRouter, HTTPException

from app.schemas import ErrorResponse, ExtractKgRequest, ExtractKgResponse, KgEdge, KgNode
from app.services.knowledge_graph import get_kg_extractor

logger = structlog.get_logger()
router = APIRouter(prefix="/extract_kg", tags=["knowledge-graph"])


@router.post(
    "",
    response_model=ExtractKgResponse,
    responses={
        400: {"model": ErrorResponse, "description": "Invalid request"},
        500: {"model": ErrorResponse, "description": "KG extraction failed"},
    },
)
async def extract_knowledge_graph(request: ExtractKgRequest) -> ExtractKgResponse:
    """
    Extract knowledge graph from legal contract text

    **Purpose**: Extract structured entities and relations from unstructured
    contract text for forensic analysis and cross-contract comparison.

    **Entities extracted**:
    - **PARTIES**: Contracting parties (ORG, PERSON)
    - **DATES**: Contract dates, deadlines, durations
    - **AMOUNTS**: Monetary amounts (canone, penali, depositi)
    - **CLAUSES**: Article references (Art. 1, Articolo 2)
    - **JURISDICTIONS**: Foro competente, tribunali
    - **PENALTIES**: Penali, sanzioni

    **Relations extracted** (pattern-based):
    - CONTRATTO_CON: Between parties
    - PAGA: Party → Amount
    - ENTRO_DATA: Party → Date
    - SOTTO_GIURISDIZIONE: Party → Jurisdiction
    - DEFINISCE_IMPORTO: Clause → Amount
    - DEFINISCE_DURATA: Clause → Date
    - PREVEDE_PENALE: Clause → Penalty

    **Technology**:
    - spaCy NER: it_core_news_lg (Italian model)
    - Custom entity ruler for legal-specific patterns
    - Regex for amounts and durations
    - Co-occurrence-based relation extraction

    **Use cases**:
    - Cross-contract comparison (same parties, different amounts?)
    - Compliance checking (all required clauses present?)
    - Risk assessment (penalty amounts, jurisdiction)
    - Timeline extraction (dates, deadlines)

    **Storage**: Nodes and edges saved to MySQL (ap_graph_nodes, ap_graph_edges)
    for querying and visualization.

    **Performance**:
    - ~500ms for 5-page contract (CPU)
    - ~200ms with GPU acceleration
    """
    start_time = time.time()

    logger.info(
        "kg_extract_request",
        doc_id=request.doc_id,
        text_length=len(request.text),
    )

    # Validate text length
    from app.config import settings

    if len(request.text) > settings.kg_max_text_length:
        raise HTTPException(
            status_code=400,
            detail=f"Text too long ({len(request.text)} chars). Max: {settings.kg_max_text_length}",
        )

    try:
        # Get KG extractor
        extractor = get_kg_extractor()

        # Extract nodes and edges
        nodes_data, edges_data = extractor.extract(request.text, request.doc_id)

        # Convert to Pydantic models
        nodes = [KgNode(**node) for node in nodes_data]
        edges = [KgEdge(**edge) for edge in edges_data]

        processing_ms = int((time.time() - start_time) * 1000)

        logger.info(
            "kg_extraction_complete",
            doc_id=request.doc_id,
            nodes_count=len(nodes),
            edges_count=len(edges),
            processing_ms=processing_ms,
            entity_types={n.entity_type for n in nodes},
        )

        return ExtractKgResponse(
            doc_id=request.doc_id,
            nodes=nodes,
            edges=edges,
            processing_ms=processing_ms,
        )

    except Exception as e:
        logger.error("kg_extraction_failed", error=str(e), doc_id=request.doc_id)
        raise HTTPException(
            status_code=500,
            detail=f"KG extraction failed: {str(e)}",
        )
