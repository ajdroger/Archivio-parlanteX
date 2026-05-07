"""
Document parsing router

Endpoints for PDF and image parsing with OCR fallback.
"""

import time
from pathlib import Path

import structlog
from fastapi import APIRouter, HTTPException

from app.schemas import ErrorResponse, ParseRequest, ParseResponse
from app.services.pdf_parser import PDFParser

logger = structlog.get_logger()
router = APIRouter(prefix="/parse", tags=["parsing"])

# Initialize parser
pdf_parser = PDFParser()


@router.post(
    "",
    response_model=ParseResponse,
    responses={
        400: {"model": ErrorResponse, "description": "Invalid request"},
        404: {"model": ErrorResponse, "description": "File not found"},
        500: {"model": ErrorResponse, "description": "Parsing failed"},
    },
)
async def parse_document(request: ParseRequest) -> ParseResponse:
    """
    Parse a document (PDF or image) into chunks

    Strategies (auto-fallback):
    1. PyMuPDF (fast text extraction)
    2. pdfplumber (better tables)
    3. unstructured (robust)
    4. OCR with Tesseract (scanned PDFs)

    **Anti-hallucination**: Only extracts text actually present in document.
    No interpolation, no generation.
    """
    start_time = time.time()

    logger.info(
        "parse_request",
        doc_id=request.doc_id,
        kb_id=request.kb_id,
        file_path=request.file_path,
        mime_type=request.mime_type,
        use_ocr=request.use_ocr,
    )

    # Security validations
    from app.middleware.security import validate_file_path, validate_mime_type, validate_file_size

    # Validate MIME type against whitelist
    validate_mime_type(request.mime_type)

    # Validate file path (prevents directory traversal)
    file_path = validate_file_path(request.file_path)

    # Validate file size
    validate_file_size(file_path)

    # Parse based on MIME type
    if request.mime_type == "application/pdf":
        try:
            chunks, parsing_method, total_pages = await pdf_parser.parse(
                file_path=request.file_path,
                doc_id=request.doc_id,
                use_ocr=request.use_ocr,
            )
        except Exception as e:
            logger.error("parsing_failed", error=str(e), doc_id=request.doc_id)
            raise HTTPException(status_code=500, detail=f"Parsing failed: {str(e)}")

    elif request.mime_type.startswith("image/"):
        # Image OCR
        try:
            from app.services.ocr_service import OCRService

            ocr = OCRService()
            text = await ocr.extract_text(request.file_path)

            from app.schemas import ParsedChunk

            chunks = [
                ParsedChunk(
                    text=text,
                    page_number=1,
                    metadata={"method": "ocr"},
                )
            ]
            parsing_method = "ocr"
            total_pages = 1

        except Exception as e:
            logger.error("ocr_failed", error=str(e), doc_id=request.doc_id)
            raise HTTPException(status_code=500, detail=f"OCR failed: {str(e)}")

    else:
        raise HTTPException(
            status_code=400,
            detail=f"Unsupported MIME type: {request.mime_type}",
        )

    processing_ms = int((time.time() - start_time) * 1000)

    logger.info(
        "parsing_complete",
        doc_id=request.doc_id,
        chunks=len(chunks),
        parsing_method=parsing_method,
        processing_ms=processing_ms,
    )

    return ParseResponse(
        doc_id=request.doc_id,
        kb_id=request.kb_id,
        chunks=chunks,
        total_chunks=len(chunks),
        total_pages=total_pages,
        parsing_method=parsing_method,
        processing_ms=processing_ms,
    )
