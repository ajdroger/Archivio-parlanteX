"""
Knowledge graph extraction endpoints.
"""

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel, Field
from typing import List, Dict, Optional
import structlog

from app.services.graph_extractor import get_extractor

logger = structlog.get_logger()

router = APIRouter(prefix="/graph", tags=["graph"])


class GraphExtractionRequest(BaseModel):
    """Request for graph extraction."""

    text: str = Field(..., description="Contract text to analyze")
    doc_id: str = Field(..., description="Document ID for reference")


class EntityNode(BaseModel):
    """Graph node (entity)."""

    id: str
    entity_type: str
    name: str
    properties: Dict
    doc_id: str


class RelationshipEdge(BaseModel):
    """Graph edge (relationship)."""

    id: str
    source_id: str
    target_id: str
    relationship_type: str
    properties: Dict


class GraphExtractionResponse(BaseModel):
    """Response with extracted graph."""

    doc_id: str
    nodes: List[EntityNode]
    edges: List[RelationshipEdge]
    stats: Dict


@router.post("/extract", response_model=GraphExtractionResponse)
async def extract_graph(request: GraphExtractionRequest):
    """
    Extract knowledge graph from contract text.

    Identifies entities (PARTY, DATE, AMOUNT, CLAUSE, JURISDICTION, PENALTY)
    and relationships between them.

    Args:
        request: Text and doc_id

    Returns:
        Graph with nodes and edges
    """
    try:
        logger.info("graph_extraction_started", doc_id=request.doc_id, text_length=len(request.text))

        extractor = get_extractor()
        graph = extractor.extract_graph(request.text, request.doc_id)

        logger.info(
            "graph_extraction_completed",
            doc_id=request.doc_id,
            nodes=len(graph["nodes"]),
            edges=len(graph["edges"]),
            processing_time_s=graph["stats"]["processing_time_s"],
        )

        return GraphExtractionResponse(**graph)

    except Exception as e:
        logger.error("graph_extraction_failed", doc_id=request.doc_id, error=str(e), exc_info=True)
        raise HTTPException(status_code=500, detail=f"Graph extraction failed: {str(e)}")


@router.get("/health")
async def health_check():
    """Health check for graph extraction service."""
    return {"status": "ok", "service": "graph_extraction"}
