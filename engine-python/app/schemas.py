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
