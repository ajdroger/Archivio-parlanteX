"""
Tests for /rerank endpoint and BGE reranker service
"""

import pytest
from fastapi.testclient import TestClient
from unittest.mock import AsyncMock, MagicMock, patch


def test_rerank_endpoint_success(client: TestClient):
    """Test successful reranking"""

    # Mock the reranker
    with patch("app.routers.rerank.get_reranker") as mock_get_reranker:
        mock_reranker = MagicMock()
        mock_reranker.rerank = AsyncMock(return_value=[
            (1, 0.95),  # Index 1 has highest score
            (0, 0.87),  # Index 0 second
            (2, 0.72),  # Index 2 third
        ])
        mock_get_reranker.return_value = mock_reranker

        request_data = {
            "query": "Qual è la durata del contratto?",
            "passages": [
                {"text": "Il contratto ha validità di 12 mesi.", "metadata": {"doc_id": "doc_1"}},
                {"text": "La durata è di 24 mesi con rinnovo automatico.", "metadata": {"doc_id": "doc_2"}},
                {"text": "Il foro competente è Milano.", "metadata": {"doc_id": "doc_3"}},
            ],
            "top_k": 3,
        }

        response = client.post("/rerank", json=request_data)

        assert response.status_code == 200
        data = response.json()

        assert "results" in data
        assert "processing_ms" in data
        assert len(data["results"]) == 3

        # Verify results are sorted by score descending
        assert data["results"][0]["index"] == 1
        assert data["results"][0]["score"] == 0.95
        assert data["results"][0]["text"] == "La durata è di 24 mesi con rinnovo automatico."
        assert data["results"][0]["metadata"]["doc_id"] == "doc_2"

        assert data["results"][1]["index"] == 0
        assert data["results"][1]["score"] == 0.87

        assert data["results"][2]["index"] == 2
        assert data["results"][2]["score"] == 0.72

        # Verify processing time is reasonable
        assert data["processing_ms"] > 0


def test_rerank_endpoint_top_k_filter(client: TestClient):
    """Test top_k filtering"""

    with patch("app.routers.rerank.get_reranker") as mock_get_reranker:
        mock_reranker = MagicMock()
        mock_reranker.rerank = AsyncMock(return_value=[
            (4, 0.95),
            (2, 0.89),
        ])
        mock_get_reranker.return_value = mock_reranker

        request_data = {
            "query": "test",
            "passages": [
                {"text": f"Passage {i}"} for i in range(10)
            ],
            "top_k": 2,  # Only want top 2
        }

        response = client.post("/rerank", json=request_data)

        assert response.status_code == 200
        data = response.json()

        assert len(data["results"]) == 2


def test_rerank_endpoint_validation_empty_query(client: TestClient):
    """Test validation error for empty query"""

    request_data = {
        "query": "   ",  # Empty/whitespace
        "passages": [{"text": "Test passage"}],
        "top_k": 5,
    }

    response = client.post("/rerank", json=request_data)

    assert response.status_code == 422  # Validation error


def test_rerank_endpoint_validation_no_passages(client: TestClient):
    """Test validation error for empty passages list"""

    request_data = {
        "query": "test query",
        "passages": [],  # Empty list
        "top_k": 5,
    }

    response = client.post("/rerank", json=request_data)

    assert response.status_code == 422  # Validation error


def test_rerank_endpoint_validation_too_many_passages(client: TestClient):
    """Test validation error for too many passages"""

    request_data = {
        "query": "test query",
        "passages": [{"text": f"Passage {i}"} for i in range(101)],  # Max is 100
        "top_k": 5,
    }

    response = client.post("/rerank", json=request_data)

    assert response.status_code == 422  # Validation error


def test_rerank_endpoint_validation_invalid_top_k(client: TestClient):
    """Test validation error for invalid top_k"""

    # top_k = 0 (minimum is 1)
    request_data = {
        "query": "test query",
        "passages": [{"text": "Test passage"}],
        "top_k": 0,
    }

    response = client.post("/rerank", json=request_data)
    assert response.status_code == 422

    # top_k = 100 (maximum is 50)
    request_data["top_k"] = 100
    response = client.post("/rerank", json=request_data)
    assert response.status_code == 422


def test_rerank_endpoint_error_handling(client: TestClient):
    """Test error handling when reranking fails"""

    with patch("app.routers.rerank.get_reranker") as mock_get_reranker:
        mock_reranker = MagicMock()
        mock_reranker.rerank = AsyncMock(side_effect=Exception("Model failed"))
        mock_get_reranker.return_value = mock_reranker

        request_data = {
            "query": "test query",
            "passages": [{"text": "Test passage"}],
            "top_k": 5,
        }

        response = client.post("/rerank", json=request_data)

        assert response.status_code == 500
        data = response.json()
        assert "detail" in data


@pytest.mark.asyncio
async def test_reranker_service_basic():
    """Test BGEReranker service basic functionality"""

    from app.services.reranker import BGEReranker

    # Mock the FlagReranker model
    with patch("app.services.reranker.FlagReranker") as mock_flag_reranker:
        mock_model = MagicMock()
        mock_model.compute_score = MagicMock(return_value=[0.9, 0.7, 0.5])
        mock_flag_reranker.return_value = mock_model

        reranker = BGEReranker()
        reranker.initialize()

        assert reranker._initialized
        assert reranker.model is not None

        # Test reranking
        results = await reranker.rerank(
            query="test query",
            passages=["passage 1", "passage 2", "passage 3"],
            top_k=2,
        )

        assert len(results) == 2
        assert results[0][0] == 0  # Index of highest score (0.9)
        assert results[0][1] == 0.9
        assert results[1][0] == 1  # Index of second highest (0.7)
        assert results[1][1] == 0.7


@pytest.mark.asyncio
async def test_reranker_service_empty_passages():
    """Test reranker with empty passages list"""

    from app.services.reranker import BGEReranker

    reranker = BGEReranker()
    results = await reranker.rerank(
        query="test",
        passages=[],
        top_k=5,
    )

    assert results == []


@pytest.mark.asyncio
async def test_reranker_singleton():
    """Test get_reranker returns singleton"""

    from app.services.reranker import get_reranker

    reranker1 = get_reranker()
    reranker2 = get_reranker()

    assert reranker1 is reranker2  # Same instance
