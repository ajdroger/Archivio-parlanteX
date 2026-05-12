"""
PDF parsing service with multi-strategy fallback

Strategies (in order):
1. PyMuPDF (fast, good for text-based PDFs)
2. pdfplumber (better table extraction)
3. unstructured (robust, handles complex layouts)
4. OCR via pytesseract (fallback for scanned PDFs)
"""

import time
from pathlib import Path
from typing import List, Optional, Tuple, TYPE_CHECKING, Any

import structlog
from tenacity import retry, stop_after_attempt, wait_exponential

# Lazy import PDF libraries (not in minimal install - cause Docker/WSL2 segfault)
if TYPE_CHECKING:
    import fitz  # PyMuPDF
    import pdfplumber
    from PIL import Image
    from unstructured.partition.pdf import partition_pdf

from app.config import settings
from app.schemas import ParsedChunk
from app.services.ocr_service import OCRService

logger = structlog.get_logger()


def _import_pdf_libs():
    """Lazy import PDF libraries with helpful error messages"""
    try:
        import fitz
        import pdfplumber
        from PIL import Image
        return fitz, pdfplumber, Image
    except ImportError as e:
        logger.error("pdf_dependencies_not_installed", error=str(e))
        raise RuntimeError(
            "PDF dependencies not installed. Run: pip install PyMuPDF pdfplumber Pillow"
        ) from e


class PDFParser:
    """PDF parser with multi-strategy fallback"""

    def __init__(self):
        self.ocr_service = OCRService()

    @retry(
        stop=stop_after_attempt(3),
        wait=wait_exponential(multiplier=1, min=2, max=10),
    )
    async def parse(
        self, file_path: str, doc_id: str, use_ocr: bool = False
    ) -> Tuple[List[ParsedChunk], str, Optional[int]]:
        """
        Parse PDF with fallback strategies

        Returns:
            (chunks, parsing_method, total_pages)
        """
        path = Path(file_path)

        if not path.exists():
            raise FileNotFoundError(f"File not found: {file_path}")

        if path.stat().st_size > settings.max_upload_size_mb * 1024 * 1024:
            raise ValueError(f"File exceeds max size {settings.max_upload_size_mb}MB")

        logger.info(
            "parsing_pdf",
            doc_id=doc_id,
            file_path=file_path,
            use_ocr=use_ocr,
        )

        # Force OCR if requested
        if use_ocr:
            chunks, total_pages = await self._parse_with_ocr(file_path, doc_id)
            return chunks, "ocr", total_pages

        # Strategy 1: PyMuPDF (fastest)
        try:
            chunks, total_pages = self._parse_with_pymupdf(file_path, doc_id)
            if self._is_valid_extraction(chunks):
                logger.info("parsing_success", method="pymupdf", chunks=len(chunks))
                return chunks, "pymupdf", total_pages
        except Exception as e:
            logger.warning("pymupdf_failed", error=str(e))

        # Strategy 2: pdfplumber (better tables)
        try:
            chunks, total_pages = self._parse_with_pdfplumber(file_path, doc_id)
            if self._is_valid_extraction(chunks):
                logger.info("parsing_success", method="pdfplumber", chunks=len(chunks))
                return chunks, "pdfplumber", total_pages
        except Exception as e:
            logger.warning("pdfplumber_failed", error=str(e))

        # Strategy 3: unstructured (robust)
        try:
            chunks, total_pages = self._parse_with_unstructured(file_path, doc_id)
            if self._is_valid_extraction(chunks):
                logger.info("parsing_success", method="unstructured", chunks=len(chunks))
                return chunks, "unstructured", total_pages
        except Exception as e:
            logger.warning("unstructured_failed", error=str(e))

        # Strategy 4: OCR fallback
        logger.info("falling_back_to_ocr", doc_id=doc_id)
        chunks, total_pages = await self._parse_with_ocr(file_path, doc_id)
        return chunks, "ocr", total_pages

    def _parse_with_pymupdf(
        self, file_path: str, doc_id: str
    ) -> Tuple[List[ParsedChunk], int]:
        """Parse with PyMuPDF (requires PyMuPDF package)"""
        fitz, _, _ = _import_pdf_libs()
        chunks = []
        doc = fitz.open(file_path)

        for page_num, page in enumerate(doc, start=1):
            if page_num > settings.pdf_max_pages:
                logger.warning("max_pages_reached", limit=settings.pdf_max_pages)
                break

            text = page.get_text()
            if text.strip():
                chunks.append(
                    ParsedChunk(
                        text=text.strip(),
                        page_number=page_num,
                        metadata={"method": "pymupdf"},
                    )
                )

        doc.close()
        return chunks, len(doc)

    def _parse_with_pdfplumber(
        self, file_path: str, doc_id: str
    ) -> Tuple[List[ParsedChunk], int]:
        """Parse with pdfplumber (requires pdfplumber package)"""
        _, pdfplumber, _ = _import_pdf_libs()
        chunks = []

        with pdfplumber.open(file_path) as pdf:
            total_pages = len(pdf.pages)

            for page_num, page in enumerate(pdf.pages, start=1):
                if page_num > settings.pdf_max_pages:
                    logger.warning("max_pages_reached", limit=settings.pdf_max_pages)
                    break

                text = page.extract_text()
                if text and text.strip():
                    chunks.append(
                        ParsedChunk(
                            text=text.strip(),
                            page_number=page_num,
                            metadata={"method": "pdfplumber"},
                        )
                    )

        return chunks, total_pages

    def _parse_with_unstructured(
        self, file_path: str, doc_id: str
    ) -> Tuple[List[ParsedChunk], int]:
        """Parse with unstructured library (requires unstructured package)"""
        # Lazy import unstructured
        try:
            from unstructured.partition.pdf import partition_pdf
        except ImportError as e:
            logger.error("unstructured_not_installed", error=str(e))
            raise RuntimeError(
                "unstructured not installed. Run: pip install unstructured"
            ) from e

        elements = partition_pdf(
            filename=file_path,
            strategy="auto",
            languages=["ita", "eng"],
        )

        chunks = []
        total_pages = 0

        for elem in elements:
            text = elem.text.strip()
            if text:
                page_num = getattr(elem.metadata, "page_number", None)
                if page_num:
                    total_pages = max(total_pages, page_num)

                chunks.append(
                    ParsedChunk(
                        text=text,
                        page_number=page_num,
                        metadata={
                            "method": "unstructured",
                            "category": elem.category,
                        },
                    )
                )

        return chunks, total_pages if total_pages > 0 else None

    async def _parse_with_ocr(
        self, file_path: str, doc_id: str
    ) -> Tuple[List[ParsedChunk], int]:
        """Parse with OCR (requires PyMuPDF and OCR dependencies)"""
        fitz, _, Image = _import_pdf_libs()
        chunks = []
        doc = fitz.open(file_path)

        # Sanitize doc_id to prevent path traversal in temp files
        from app.middleware.security import sanitize_filename
        safe_doc_id = sanitize_filename(doc_id)

        for page_num, page in enumerate(doc, start=1):
            if page_num > settings.pdf_max_pages:
                logger.warning("max_pages_reached", limit=settings.pdf_max_pages)
                break

            # Render page to image
            pix = page.get_pixmap(dpi=settings.ocr_dpi)
            img_data = pix.tobytes("png")

            # Save temporarily (safe_doc_id prevents directory traversal)
            temp_img_path = f"/tmp/{safe_doc_id}_page_{page_num}.png"
            with open(temp_img_path, "wb") as f:
                f.write(img_data)

            # OCR
            text = await self.ocr_service.extract_text(temp_img_path)

            if text.strip():
                chunks.append(
                    ParsedChunk(
                        text=text.strip(),
                        page_number=page_num,
                        metadata={"method": "ocr"},
                    )
                )

            # Cleanup
            Path(temp_img_path).unlink(missing_ok=True)

        total_pages = len(doc)
        doc.close()

        return chunks, total_pages

    def _is_valid_extraction(self, chunks: List[ParsedChunk]) -> bool:
        """
        Check if extraction is valid

        Valid if:
        - At least 1 chunk
        - Average chunk length > 20 chars (avoid gibberish)
        """
        if not chunks:
            return False

        total_chars = sum(len(c.text) for c in chunks)
        avg_len = total_chars / len(chunks)

        return avg_len > 20
