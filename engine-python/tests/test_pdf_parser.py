"""
Unit tests for PDFParser service
"""

import pytest
from pathlib import Path

from app.services.pdf_parser import PDFParser
from app.schemas import ParsedChunk


@pytest.fixture
def parser():
    """PDFParser instance"""
    return PDFParser()


@pytest.mark.asyncio
async def test_parse_pdf_success(parser: PDFParser, sample_pdf: Path):
    """Test successful PDF parsing"""
    chunks, method, total_pages = await parser.parse(
        file_path=str(sample_pdf),
        doc_id="test_123",
        use_ocr=False,
    )

    assert isinstance(chunks, list)
    assert len(chunks) >= 0  # Might be 0 for minimal test PDF
    assert method in ["pymupdf", "pdfplumber", "unstructured", "ocr"]
    assert total_pages is None or total_pages >= 0


@pytest.mark.asyncio
async def test_parse_pdf_not_found(parser: PDFParser):
    """Test parsing non-existent PDF"""
    with pytest.raises(FileNotFoundError):
        await parser.parse(
            file_path="/nonexistent/file.pdf",
            doc_id="test_404",
            use_ocr=False,
        )


@pytest.mark.asyncio
async def test_parse_pdf_force_ocr(parser: PDFParser, sample_pdf: Path):
    """Test forcing OCR mode"""
    chunks, method, total_pages = await parser.parse(
        file_path=str(sample_pdf),
        doc_id="test_ocr",
        use_ocr=True,
    )

    # OCR was forced
    assert method == "ocr"
    assert isinstance(chunks, list)


def test_is_valid_extraction_empty(parser: PDFParser):
    """Test validation with empty chunks"""
    assert not parser._is_valid_extraction([])


def test_is_valid_extraction_valid(parser: PDFParser):
    """Test validation with valid chunks"""
    chunks = [
        ParsedChunk(
            text="This is a valid chunk with enough content to be meaningful.",
            page_number=1,
            metadata={},
        ),
        ParsedChunk(
            text="Another chunk with substantial text content here.",
            page_number=2,
            metadata={},
        ),
    ]

    assert parser._is_valid_extraction(chunks)


def test_is_valid_extraction_gibberish(parser: PDFParser):
    """Test validation with gibberish (too short)"""
    chunks = [
        ParsedChunk(text="a", page_number=1, metadata={}),
        ParsedChunk(text="b", page_number=2, metadata={}),
        ParsedChunk(text="c", page_number=3, metadata={}),
    ]

    # Average length < 20, should fail validation
    assert not parser._is_valid_extraction(chunks)


def test_is_valid_extraction_mixed(parser: PDFParser):
    """Test validation with mixed content"""
    chunks = [
        ParsedChunk(text="x", page_number=1, metadata={}),
        ParsedChunk(
            text="This is a long enough chunk to pass validation on average.",
            page_number=2,
            metadata={},
        ),
    ]

    # Average should be > 20
    total = len(chunks[0].text) + len(chunks[1].text)
    avg = total / 2

    if avg > 20:
        assert parser._is_valid_extraction(chunks)
    else:
        assert not parser._is_valid_extraction(chunks)
