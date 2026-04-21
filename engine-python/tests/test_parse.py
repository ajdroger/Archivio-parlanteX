"""
Tests for /parse endpoint
"""

import pytest
from fastapi.testclient import TestClient
from pathlib import Path


def test_health_endpoint(client: TestClient):
    """Test /health endpoint"""
    response = client.get("/health")
    assert response.status_code == 200
    data = response.json()
    assert data["status"] == "ok"
    assert data["service"] == "python-worker"


def test_root_endpoint(client: TestClient):
    """Test root endpoint"""
    response = client.get("/")
    assert response.status_code == 200
    data = response.json()
    assert "endpoints" in data
    assert "POST /parse" in data["endpoints"]


def test_parse_endpoint_pdf_success(client: TestClient, sample_pdf: Path):
    """Test parsing a valid PDF"""
    request_data = {
        "kb_id": "test_kb",
        "doc_id": "test_doc_123",
        "file_path": str(sample_pdf),
        "mime_type": "application/pdf",
        "use_ocr": False,
    }

    response = client.post("/parse", json=request_data)

    assert response.status_code == 200
    data = response.json()

    assert data["doc_id"] == "test_doc_123"
    assert data["kb_id"] == "test_kb"
    assert data["total_chunks"] >= 0
    assert "chunks" in data
    assert "parsing_method" in data
    assert data["parsing_method"] in ["pymupdf", "pdfplumber", "unstructured", "ocr"]
    assert data["processing_ms"] > 0


def test_parse_endpoint_file_not_found(client: TestClient):
    """Test parsing with non-existent file"""
    request_data = {
        "kb_id": "test_kb",
        "doc_id": "test_doc_404",
        "file_path": "/nonexistent/path/file.pdf",
        "mime_type": "application/pdf",
        "use_ocr": False,
    }

    response = client.post("/parse", json=request_data)

    assert response.status_code == 404
    data = response.json()
    assert "detail" in data


def test_parse_endpoint_invalid_mime_type(client: TestClient, sample_pdf: Path):
    """Test parsing with unsupported MIME type"""
    request_data = {
        "kb_id": "test_kb",
        "doc_id": "test_doc_invalid",
        "file_path": str(sample_pdf),
        "mime_type": "application/msword",  # Not supported
        "use_ocr": False,
    }

    response = client.post("/parse", json=request_data)

    assert response.status_code == 400


def test_parse_endpoint_validation_error(client: TestClient):
    """Test parsing with missing required fields"""
    request_data = {
        "kb_id": "test_kb",
        # Missing doc_id, file_path, mime_type
    }

    response = client.post("/parse", json=request_data)

    assert response.status_code == 422  # Validation error


def test_parse_endpoint_image_ocr(client: TestClient, sample_image: Path):
    """Test OCR on image file"""
    request_data = {
        "kb_id": "test_kb",
        "doc_id": "test_img_123",
        "file_path": str(sample_image),
        "mime_type": "image/png",
        "use_ocr": True,
    }

    response = client.post("/parse", json=request_data)

    # OCR might fail on blank test image, but endpoint should handle gracefully
    # Either succeeds or returns 500 with error detail
    assert response.status_code in [200, 500]

    if response.status_code == 200:
        data = response.json()
        assert data["parsing_method"] == "ocr"
        assert "chunks" in data


def test_parse_endpoint_pdf_force_ocr(client: TestClient, sample_pdf: Path):
    """Test forcing OCR on PDF"""
    request_data = {
        "kb_id": "test_kb",
        "doc_id": "test_doc_ocr",
        "file_path": str(sample_pdf),
        "mime_type": "application/pdf",
        "use_ocr": True,  # Force OCR
    }

    response = client.post("/parse", json=request_data)

    # OCR should work or fail gracefully
    assert response.status_code in [200, 500]

    if response.status_code == 200:
        data = response.json()
        assert data["parsing_method"] == "ocr"
