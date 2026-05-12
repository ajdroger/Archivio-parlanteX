"""
Hallucination Verification Router
Fase 6.2 - Advanced Hallucination Detection

Exposes POST /verify_hallucination endpoint for LLM answer validation.
"""

from typing import List, Optional
from pydantic import BaseModel, Field
import structlog

from fastapi import APIRouter, HTTPException

from app.services.hallucination_detector import verify_hallucination

logger = structlog.get_logger()

router = APIRouter(tags=["hallucination-detection"])


class SourceDocument(BaseModel):
    """Source document with text quote"""

    text_quote: str = Field(..., description="Verbatim text quote from source")
    doc_id: Optional[str] = Field(None, description="Source document ID")


class VerifyHallucinationRequest(BaseModel):
    """Request to verify LLM answer for hallucinations"""

    answer: str = Field(..., description="LLM-generated answer to verify")
    sources: List[SourceDocument] = Field(
        ..., description="Source documents used for RAG"
    )


class VerifyHallucinationResponse(BaseModel):
    """Hallucination verification result"""

    hallucination_score: float = Field(
        ...,
        description="Hallucination score (0-1, higher = more hallucination)",
        ge=0.0,
        le=1.0,
    )
    flagged_claims: List[str] = Field(..., description="Claims without support")
    supported_claims: List[str] = Field(..., description="Claims with support")
    total_claims: int = Field(..., description="Total claims extracted")


@router.post("/verify_hallucination", response_model=VerifyHallucinationResponse)
async def verify_hallucination_endpoint(request: VerifyHallucinationRequest):
    """
    Verify LLM-generated answer for hallucinations

    Pipeline:
    1. Extract atomic claims from answer using Ollama
    2. Verify each claim against source documents
    3. Calculate hallucination score (ratio of unsupported claims)
    4. Return flagged and supported claims

    Returns:
        VerifyHallucinationResponse with score and claim categorization
    """
    logger.info(
        "hallucination_verification_requested",
        answer_length=len(request.answer),
        sources_count=len(request.sources),
    )

    if not request.answer.strip():
        raise HTTPException(status_code=400, detail="Answer cannot be empty")

    if not request.sources:
        raise HTTPException(
            status_code=400, detail="At least one source document is required"
        )

    try:
        # Convert Pydantic models to dicts for service layer
        sources_dicts = [
            {"text_quote": s.text_quote, "doc_id": s.doc_id} for s in request.sources
        ]

        # Call hallucination detector service
        result = await verify_hallucination(answer=request.answer, sources=sources_dicts)

        logger.info(
            "hallucination_verification_completed",
            score=result["hallucination_score"],
            flagged_count=len(result["flagged_claims"]),
            total_claims=result["total_claims"],
        )

        return VerifyHallucinationResponse(
            hallucination_score=result["hallucination_score"],
            flagged_claims=result["flagged_claims"],
            supported_claims=result["supported_claims"],
            total_claims=result["total_claims"],
        )

    except Exception as e:
        logger.error("hallucination_verification_failed", error=str(e))
        raise HTTPException(
            status_code=500, detail=f"Hallucination verification failed: {str(e)}"
        )
