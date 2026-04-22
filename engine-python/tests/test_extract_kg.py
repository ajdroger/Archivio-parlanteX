"""
Tests for /extract_kg endpoint and KG extraction service
"""

import pytest
from fastapi.testclient import TestClient
from unittest.mock import MagicMock, patch


SAMPLE_CONTRACT = """
CONTRATTO DI LOCAZIONE COMMERCIALE

Tra Alpha S.r.l. (Locatore) e Beta S.r.l. (Conduttore).

Art. 1 - Durata
Il contratto ha durata di 24 mesi.

Art. 2 - Canone
Il canone annuo è di Euro 24.000,00 oltre IVA.

Art. 3 - Foro competente
Per ogni controversia è competente il Tribunale di Milano.

Art. 4 - Penali
In caso di inadempimento, penale di Euro 5.000,00.
"""


def test_extract_kg_endpoint_success(client: TestClient):
    """Test successful KG extraction"""

    # Mock the KG extractor
    with patch("app.routers.extract_kg.get_kg_extractor") as mock_get_extractor:
        mock_extractor = MagicMock()
        mock_extractor.extract.return_value = (
            [
                {
                    "id": "doc_1_PARTIES_0",
                    "entity_type": "PARTIES",
                    "name": "Alpha S.r.l.",
                    "properties": {"start_char": 30, "confidence": 1.0},
                },
                {
                    "id": "doc_1_AMOUNTS_0",
                    "entity_type": "AMOUNTS",
                    "name": "Euro 24.000,00",
                    "properties": {"start_char": 150, "confidence": 0.95},
                },
            ],
            [
                {
                    "source_id": "doc_1_PARTIES_0",
                    "target_id": "doc_1_AMOUNTS_0",
                    "relation_type": "PAGA",
                    "properties": {"confidence": 0.7},
                }
            ],
        )
        mock_get_extractor.return_value = mock_extractor

        request_data = {
            "doc_id": "doc_1",
            "text": SAMPLE_CONTRACT,
        }

        response = client.post("/extract_kg", json=request_data)

        assert response.status_code == 200
        data = response.json()

        assert "nodes" in data
        assert "edges" in data
        assert "processing_ms" in data
        assert data["doc_id"] == "doc_1"

        # Verify nodes structure
        assert len(data["nodes"]) == 2
        node1 = data["nodes"][0]
        assert node1["id"] == "doc_1_PARTIES_0"
        assert node1["entity_type"] == "PARTIES"
        assert node1["name"] == "Alpha S.r.l."
        assert "properties" in node1

        # Verify edges structure
        assert len(data["edges"]) == 1
        edge1 = data["edges"][0]
        assert edge1["source_id"] == "doc_1_PARTIES_0"
        assert edge1["target_id"] == "doc_1_AMOUNTS_0"
        assert edge1["relation_type"] == "PAGA"


def test_extract_kg_endpoint_validation_empty_doc_id(client: TestClient):
    """Test validation error for empty doc_id"""

    request_data = {
        "doc_id": "",
        "text": "Sample text",
    }

    response = client.post("/extract_kg", json=request_data)
    assert response.status_code == 422


def test_extract_kg_endpoint_validation_empty_text(client: TestClient):
    """Test validation error for empty text"""

    request_data = {
        "doc_id": "doc_1",
        "text": "",
    }

    response = client.post("/extract_kg", json=request_data)
    assert response.status_code == 422


def test_extract_kg_endpoint_text_too_long(client: TestClient):
    """Test validation for text exceeding max length"""

    # Create text longer than max (50000)
    long_text = "x" * 50001

    request_data = {
        "doc_id": "doc_1",
        "text": long_text,
    }

    response = client.post("/extract_kg", json=request_data)
    assert response.status_code == 400
    data = response.json()
    assert "too long" in data["detail"].lower()


def test_extract_kg_endpoint_error_handling(client: TestClient):
    """Test error handling when extraction fails"""

    with patch("app.routers.extract_kg.get_kg_extractor") as mock_get_extractor:
        mock_extractor = MagicMock()
        mock_extractor.extract.side_effect = Exception("spaCy failed")
        mock_get_extractor.return_value = mock_extractor

        request_data = {
            "doc_id": "doc_1",
            "text": "Sample text",
        }

        response = client.post("/extract_kg", json=request_data)

        assert response.status_code == 500
        data = response.json()
        assert "detail" in data


def test_kg_service_entity_extraction():
    """Test KGExtractor entity extraction"""

    from app.services.knowledge_graph import KGExtractor

    # Mock spaCy
    with patch("app.services.knowledge_graph.spacy.load") as mock_load:
        mock_nlp = MagicMock()

        # Mock entities
        mock_ent1 = MagicMock()
        mock_ent1.label_ = "ORG"
        mock_ent1.text = "Alpha S.r.l."
        mock_ent1.start_char = 30
        mock_ent1.end_char = 42

        mock_ent2 = MagicMock()
        mock_ent2.label_ = "DATE"
        mock_ent2.text = "24 mesi"
        mock_ent2.start_char = 100
        mock_ent2.end_char = 107

        mock_doc = MagicMock()
        mock_doc.ents = [mock_ent1, mock_ent2]
        mock_doc.text = SAMPLE_CONTRACT
        mock_doc.sents = []  # No sentences for this test

        mock_nlp.return_value = mock_doc
        mock_nlp.pipe_names = []
        mock_load.return_value = mock_nlp

        extractor = KGExtractor()
        extractor.initialize()

        nodes, edges = extractor.extract(SAMPLE_CONTRACT, "doc_1")

        # Should extract at least the 2 entities
        assert len(nodes) >= 2

        # Check entity types
        entity_types = {n["entity_type"] for n in nodes}
        assert "PARTIES" in entity_types or "DATES" in entity_types


def test_kg_service_amount_extraction():
    """Test monetary amount extraction with regex"""

    from app.services.knowledge_graph import KGExtractor

    text = "Il canone è di Euro 24.000,00 oltre IVA. Deposito di € 5.000."

    with patch("app.services.knowledge_graph.spacy.load") as mock_load:
        mock_nlp = MagicMock()
        mock_doc = MagicMock()
        mock_doc.ents = []
        mock_doc.text = text
        mock_doc.sents = []

        mock_nlp.return_value = mock_doc
        mock_nlp.pipe_names = []
        mock_load.return_value = mock_nlp

        extractor = KGExtractor()
        extractor.initialize()

        nodes, _ = extractor.extract(text, "doc_1")

        # Should find at least one amount
        amount_nodes = [n for n in nodes if n["entity_type"] == "AMOUNTS"]
        assert len(amount_nodes) >= 1


def test_kg_service_duration_extraction():
    """Test duration extraction"""

    from app.services.knowledge_graph import KGExtractor

    text = "Il contratto ha durata di 24 mesi e si rinnova per 6 anni."

    with patch("app.services.knowledge_graph.spacy.load") as mock_load:
        mock_nlp = MagicMock()
        mock_doc = MagicMock()
        mock_doc.ents = []
        mock_doc.text = text
        mock_doc.sents = []

        mock_nlp.return_value = mock_doc
        mock_nlp.pipe_names = []
        mock_load.return_value = mock_nlp

        extractor = KGExtractor()
        extractor.initialize()

        nodes, _ = extractor.extract(text, "doc_1")

        # Should find durations
        duration_nodes = [
            n for n in nodes
            if n["entity_type"] == "DATES" and n["properties"].get("duration")
        ]
        assert len(duration_nodes) >= 1


def test_kg_service_singleton():
    """Test get_kg_extractor returns singleton"""

    from app.services.knowledge_graph import get_kg_extractor

    extractor1 = get_kg_extractor()
    extractor2 = get_kg_extractor()

    assert extractor1 is extractor2  # Same instance
