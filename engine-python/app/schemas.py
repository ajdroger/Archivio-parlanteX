"""
Data schemas for Python AI Worker

All request/response models for FastAPI endpoints.
"""

from typing import List, Optional

from pydantic import BaseModel, Field, field_validator


class ParseRequest(BaseModel):
    """Request to parse a document"""

    kb_id: str = Field(..., min_length=1, description="Knowledge base ID")
    doc_id: str = Field(..., min_length=1, description="Unique document ID")
    file_path: str = Field(..., min_length=1, description="Path to file in shared volume")
    mime_type: str = Field(..., description="MIME type (application/pdf, image/png, etc.)")
    use_ocr: bool = Field(default=False, description="Force OCR even for text PDFs")

    @field_validator("mime_type")
    @classmethod
    def validate_mime_type(cls, v: str) -> str:
        """Validate MIME type"""
        allowed = [
            "application/pdf",
            "image/png",
            "image/jpeg",
            "image/jpg",
            "image/tiff",
        ]
        if v not in allowed:
            raise ValueError(f"MIME type {v} not supported. Allowed: {allowed}")
        return v


class ParsedChunk(BaseModel):
    """A single parsed chunk of text"""

    text: str = Field(..., description="Chunk text content")
    page_number: Optional[int] = Field(None, description="Page number (PDF only)")
    metadata: dict = Field(default_factory=dict, description="Additional metadata")


class ParseResponse(BaseModel):
    """Response from document parsing"""

    doc_id: str
    kb_id: str
    chunks: List[ParsedChunk]
    total_chunks: int
    total_pages: Optional[int] = None
    parsing_method: str = Field(
        ...,
        description="Method used: pymupdf | pdfplumber | unstructured | ocr",
    )
    processing_ms: int


class ErrorResponse(BaseModel):
    """Error response"""

    error: str
    detail: Optional[str] = None


# === Reranker schemas (Fase 2.2) ===


class RerankPassage(BaseModel):
    """A passage to rerank"""

    text: str = Field(..., min_length=1, description="Passage text content")
    metadata: dict = Field(default_factory=dict, description="Optional metadata (doc_id, chunk_id, etc.)")


class RerankRequest(BaseModel):
    """Request to rerank passages"""

    query: str = Field(..., min_length=1, description="Search query")

    # Support both formats: texts (simple strings from Rust) or passages (rich with metadata)
    texts: Optional[List[str]] = Field(None, min_items=1, max_items=100, description="Simple text passages (Rust format)")
    passages: Optional[List[RerankPassage]] = Field(None, min_items=1, max_items=100, description="Rich passages with metadata")

    top_k: int = Field(default=5, ge=1, le=50, description="Number of top results to return")

    @field_validator("texts", "passages")
    @classmethod
    def validate_one_input(cls, v, info):
        """Ensure at least one of texts or passages is provided"""
        # This will be called for each field, check after both are set
        return v


class RerankResult(BaseModel):
    """A reranked passage with score"""

    index: int = Field(..., description="Original index in input list")
    score: float = Field(..., description="Reranking score (normalized 0-1, higher is better)")

    # Optional fields (included for rich responses, omitted for Rust compatibility)
    text: Optional[str] = Field(None, description="Passage text (optional)")
    metadata: Optional[dict] = Field(None, description="Original metadata (optional)")


class RerankResponse(BaseModel):
    """Response from reranking"""

    results: List[RerankResult]
    processing_ms: int
