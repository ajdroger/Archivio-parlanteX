"""
Contextual Retrieval router

Endpoint for adding document context to chunks (Anthropic technique).
"""

import time

import structlog
from fastapi import APIRouter, HTTPException

from app.schemas import (
    ContextualizeRequest,
    ContextualizeResponse,
    ContextualizedChunk,
    ErrorResponse,
)
from app.services.contextual import get_contextual_retrieval

logger = structlog.get_logger()
router = APIRouter(prefix="/contextualize", tags=["contextual-retrieval"])


@router.post(
    "",
    response_model=ContextualizeResponse,
    responses={
        400: {"model": ErrorResponse, "description": "Invalid request"},
        500: {"model": ErrorResponse, "description": "Contextualization failed"},
    },
)
async def contextualize_chunks(request: ContextualizeRequest) -> ContextualizeResponse:
    """
    Add document context to chunks using LLM (Anthropic technique)

    **Purpose**: Improve semantic search by prepending situating context
    to each chunk before embedding.

    **Technique** (from Anthropic):
    1. For each chunk, ask LLM: "Given the full document, provide a short
       context to situate this chunk for search retrieval purposes."
    2. Prepend generated context to original chunk
    3. Embed the contextualized chunk (done by Rust engine)
    4. Results in better retrieval accuracy (reduces ambiguity)

    **Example**:
    - Original chunk: "Il contratto ha durata di 24 mesi."
    - Context: "Questo è un contratto di locazione commerciale tra Alpha S.r.l. e Beta S.r.l."
    - Full text: "{context}\\n\\n{original}"

    **Performance**:
    - LLM: qwen2.5:3b (fast, local, zero-cost)
    - Parallel batch processing (5 chunks at a time)
    - ~500ms for 10 chunks (CUDA)
    - ~2s for 10 chunks (CPU)

    **Called by**: Rust engine during document ingestion
    """
    start_time = time.time()

    # Reconstruct document text from chunks if not provided (for Rust compatibility)
    if request.document_text:
        document_text = request.document_text
    else:
        # Concatenate chunks to approximate full document
        document_text = "\n\n".join(c.text for c in request.chunks)
        logger.debug("document_text_reconstructed", chunks_count=len(request.chunks))

    logger.info(
        "contextualize_request",
        doc_id=request.doc_id,
        chunks_count=len(request.chunks),
        doc_length=len(document_text),
        doc_provided=request.document_text is not None,
    )

    try:
        # Get contextual retrieval service
        contextual = get_contextual_retrieval()

        # Extract chunk texts
        chunk_texts = [c.text for c in request.chunks]

        # Contextualize all chunks
        contextualized_texts = await contextual.contextualize_chunks(
            document_text=document_text,
            chunks=chunk_texts,
        )

        # Build response chunks
        chunks = []
        for i, (original_chunk, contextualized_text) in enumerate(
            zip(request.chunks, contextualized_texts)
        ):
            # Extract context prefix (everything before the "\n\n" separator)
            if "\n\n" in contextualized_text:
                context_prefix, _ = contextualized_text.split("\n\n", 1)
            else:
                # No context generated (fallback case)
                context_prefix = ""

            chunks.append(
                ContextualizedChunk(
                    index=original_chunk.index,
                    original_text=original_chunk.text,
                    context_prefix=context_prefix,
                    full_text=contextualized_text,
                )
            )

        processing_ms = int((time.time() - start_time) * 1000)

        logger.info(
            "contextualization_complete",
            doc_id=request.doc_id,
            chunks_count=len(chunks),
            processing_ms=processing_ms,
            avg_context_len=sum(len(c.context_prefix) for c in chunks) // len(chunks)
            if chunks
            else 0,
        )

        return ContextualizeResponse(
            doc_id=request.doc_id,
            chunks=chunks,
            processing_ms=processing_ms,
        )

    except Exception as e:
        logger.error("contextualization_failed", error=str(e), doc_id=request.doc_id)
        raise HTTPException(
            status_code=500,
            detail=f"Contextualization failed: {str(e)}",
        )
