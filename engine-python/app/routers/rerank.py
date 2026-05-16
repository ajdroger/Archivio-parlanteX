"""
Reranking router

Endpoint for BGE reranker v2-m3 cross-encoder.
"""

import time

import structlog
from fastapi import APIRouter, HTTPException

from app.schemas import ErrorResponse, RerankRequest, RerankResponse, RerankResult
from app.services.reranker import get_reranker

logger = structlog.get_logger()
router = APIRouter(prefix="/rerank", tags=["reranking"])


@router.post(
    "",
    response_model=RerankResponse,
    responses={
        400: {"model": ErrorResponse, "description": "Invalid request"},
        500: {"model": ErrorResponse, "description": "Reranking failed"},
    },
)
async def rerank_passages(request: RerankRequest) -> RerankResponse:
    """
    Rerank passages using BGE reranker v2-m3 cross-encoder

    **Purpose**: Improve hybrid search results by reranking candidates
    with a more sophisticated cross-encoder model.

    **Workflow**:
    1. Rust engine performs hybrid search (dense + sparse) → top 30 candidates
    2. Candidates sent to this endpoint
    3. BGE reranker scores each candidate against query
    4. Return top K (default 5) reranked results

    **Model**: BAAI/bge-reranker-v2-m3
    - Multilingual (supports Italian)
    - Cross-encoder (more accurate than bi-encoders)
    - ~560MB model size
    - Normalized scores 0-1 (higher = more relevant)

    **Performance**:
    - CUDA: ~50ms for 30 passages
    - CPU: ~200ms for 30 passages
    """
    start_time = time.time()

    # Validate: must have either texts or passages
    if not request.texts and not request.passages:
        raise HTTPException(
            status_code=400,
            detail="Either 'texts' or 'passages' must be provided",
        )

    # Extract texts based on input format
    if request.texts:
        # Simple format from Rust (texts only)
        passage_texts = request.texts
        passage_metadata = [{} for _ in request.texts]
    else:
        # Rich format (passages with metadata)
        passage_texts = [p.text for p in request.passages]
        passage_metadata = [p.metadata for p in request.passages]

    logger.info(
        "rerank_request",
        query_len=len(request.query),
        passages_count=len(passage_texts),
        top_k=request.top_k,
        format="texts" if request.texts else "passages",
    )

    try:
        # Get reranker instance (lazy init)
        reranker = get_reranker()

        # Rerank
        top_indices_scores = await reranker.rerank(
            query=request.query,
            passages=passage_texts,
            top_k=request.top_k,
        )

        # Build results
        results = [
            RerankResult(
                index=idx,
                text=passage_texts[idx],
                score=score,
                metadata=passage_metadata[idx],
            )
            for idx, score in top_indices_scores
        ]

        processing_ms = int((time.time() - start_time) * 1000)

        logger.info(
            "reranking_complete",
            results_count=len(results),
            processing_ms=processing_ms,
            top_score=results[0].score if results else None,
        )

        return RerankResponse(
            results=results,
            processing_ms=processing_ms,
        )

    except Exception as e:
        logger.error("reranking_failed", error=str(e))
        raise HTTPException(
            status_code=500,
            detail=f"Reranking failed: {str(e)}",
        )
