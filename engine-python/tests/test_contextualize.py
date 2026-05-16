"""
Tests for /contextualize endpoint and contextual retrieval service
"""

import pytest
from fastapi.testclient import TestClient
from unittest.mock import AsyncMock, MagicMock, patch


def test_contextualize_endpoint_success(client: TestClient):
    """Test successful contextualization"""

    # Mock the contextual service
    with patch("app.routers.contextualize.get_contextual_retrieval") as mock_get_contextual:
        mock_service = MagicMock()
        mock_service.contextualize_chunks = AsyncMock(return_value=[
            "Questo è un contratto di locazione commerciale.\n\nArticolo 1: Durata del contratto è di 24 mesi.",
            "Il contratto riguarda locazione tra Alpha e Beta.\n\nIl canone annuo è di Euro 24.000,00.",
        ])
        mock_get_contextual.return_value = mock_service

        request_data = {
            "doc_id": "doc_123",
            "document_text": "CONTRATTO DI LOCAZIONE\n\nTra Alpha S.r.l. e Beta S.r.l.\n\nArticolo 1: Durata del contratto è di 24 mesi.\nArticolo 2: Il canone annuo è di Euro 24.000,00.",
            "chunks": [
                {"index": 0, "text": "Articolo 1: Durata del contratto è di 24 mesi."},
                {"index": 1, "text": "Il canone annuo è di Euro 24.000,00."},
            ],
        }

        response = client.post("/contextualize", json=request_data)

        assert response.status_code == 200
        data = response.json()

        assert "chunks" in data
        assert "processing_ms" in data
        assert data["doc_id"] == "doc_123"
        assert len(data["chunks"]) == 2

        # Verify first chunk structure
        chunk1 = data["chunks"][0]
        assert chunk1["index"] == 0
        assert chunk1["original_text"] == "Articolo 1: Durata del contratto è di 24 mesi."
        assert "context_prefix" in chunk1
        assert "full_text" in chunk1
        assert chunk1["context_prefix"] in chunk1["full_text"]
        assert chunk1["original_text"] in chunk1["full_text"]


def test_contextualize_endpoint_validation_empty_doc_id(client: TestClient):
    """Test validation error for empty doc_id"""

    request_data = {
        "doc_id": "",
        "document_text": "Sample document",
        "chunks": [{"index": 0, "text": "Sample chunk"}],
    }

    response = client.post("/contextualize", json=request_data)
    assert response.status_code == 422  # Validation error


def test_contextualize_endpoint_validation_empty_chunks(client: TestClient):
    """Test validation error for empty chunks list"""

    request_data = {
        "doc_id": "doc_123",
        "document_text": "Sample document",
        "chunks": [],
    }

    response = client.post("/contextualize", json=request_data)
    assert response.status_code == 422


def test_contextualize_endpoint_validation_too_many_chunks(client: TestClient):
    """Test validation error for too many chunks"""

    request_data = {
        "doc_id": "doc_123",
        "document_text": "Sample document",
        "chunks": [{"index": i, "text": f"Chunk {i}"} for i in range(501)],  # Max is 500
    }

    response = client.post("/contextualize", json=request_data)
    assert response.status_code == 422


def test_contextualize_endpoint_error_handling(client: TestClient):
    """Test error handling when contextualization fails"""

    with patch("app.routers.contextualize.get_contextual_retrieval") as mock_get_contextual:
        mock_service = MagicMock()
        mock_service.contextualize_chunks = AsyncMock(side_effect=Exception("LLM failed"))
        mock_get_contextual.return_value = mock_service

        request_data = {
            "doc_id": "doc_123",
            "document_text": "Sample document",
            "chunks": [{"index": 0, "text": "Sample chunk"}],
        }

        response = client.post("/contextualize", json=request_data)

        assert response.status_code == 500
        data = response.json()
        assert "detail" in data


@pytest.mark.asyncio
async def test_contextual_service_basic():
    """Test ContextualRetrieval service basic functionality"""

    from app.services.contextual import ContextualRetrieval

    # Mock httpx AsyncClient
    with patch("app.services.contextual.httpx.AsyncClient") as mock_client_class:
        mock_client = MagicMock()
        mock_response = MagicMock()
        mock_response.json.return_value = {
            "response": "Questo è un contratto di locazione commerciale tra due società."
        }
        mock_response.raise_for_status = MagicMock()

        mock_client.post = AsyncMock(return_value=mock_response)
        mock_client_class.return_value = mock_client

        service = ContextualRetrieval()

        document_text = "CONTRATTO DI LOCAZIONE\n\nTra Alpha e Beta.\n\nArticolo 1: Durata 24 mesi."
        chunks = [
            "Articolo 1: Durata 24 mesi.",
        ]

        results = await service.contextualize_chunks(document_text, chunks)

        assert len(results) == 1
        assert "Questo è un contratto" in results[0]
        assert "Articolo 1: Durata 24 mesi." in results[0]


@pytest.mark.asyncio
async def test_contextual_service_empty_chunks():
    """Test service with empty chunks list"""

    from app.services.contextual import ContextualRetrieval

    service = ContextualRetrieval()
    results = await service.contextualize_chunks("Sample doc", [])

    assert results == []


@pytest.mark.asyncio
async def test_contextual_service_batch_processing():
    """Test batch processing of chunks"""

    from app.services.contextual import ContextualRetrieval

    with patch("app.services.contextual.httpx.AsyncClient") as mock_client_class:
        mock_client = MagicMock()
        mock_response = MagicMock()
        mock_response.json.return_value = {"response": "Context"}
        mock_response.raise_for_status = MagicMock()

        mock_client.post = AsyncMock(return_value=mock_response)
        mock_client_class.return_value = mock_client

        service = ContextualRetrieval()

        # 12 chunks with batch size 5 → 3 batches
        chunks = [f"Chunk {i}" for i in range(12)]

        results = await service.contextualize_chunks("Doc", chunks)

        assert len(results) == 12

        # Verify all chunks were processed
        for i, result in enumerate(results):
            assert f"Chunk {i}" in result


@pytest.mark.asyncio
async def test_contextual_service_llm_fallback():
    """Test fallback when LLM fails"""

    from app.services.contextual import ContextualRetrieval

    with patch("app.services.contextual.httpx.AsyncClient") as mock_client_class:
        mock_client = MagicMock()
        mock_client.post = AsyncMock(side_effect=Exception("LLM unavailable"))
        mock_client_class.return_value = mock_client

        service = ContextualRetrieval()

        chunks = ["Original chunk text"]
        results = await service.contextualize_chunks("Doc", chunks)

        # Should return original chunk as fallback
        assert results[0] == "Original chunk text"


@pytest.mark.asyncio
async def test_contextual_singleton():
    """Test get_contextual_retrieval returns singleton"""

    from app.services.contextual import get_contextual_retrieval

    service1 = get_contextual_retrieval()
    service2 = get_contextual_retrieval()

    assert service1 is service2  # Same instance
