"""
Knowledge Graph extraction service for Italian legal contracts.

Extracts entities and relationships from contract text to build
a knowledge graph stored in MySQL (ap_graph_nodes, ap_graph_edges).
"""

import re
from typing import List, Dict, Tuple, Optional
from datetime import datetime
import structlog

logger = structlog.get_logger()

# Entity types for legal contracts
ENTITY_TYPES = {
    "PARTY": "Contraente (persona fisica o giuridica)",
    "DATE": "Data (contratto, scadenze, termini)",
    "AMOUNT": "Importo (valuta, penali, corrispettivi)",
    "CLAUSE": "Clausola contrattuale (articolo, paragrafo)",
    "JURISDICTION": "Giurisdizione (foro competente, legge applicabile)",
    "PENALTY": "Penale o indennizzo",
    "TERM": "Termine (durata contratto, scadenze)",
}

# Relationship types
RELATIONSHIP_TYPES = {
    "PARTE_DI": "Entity A è parte di Entity B",
    "RIFERISCE_A": "Entity A riferisce a Entity B",
    "MODIFICA": "Entity A modifica Entity B",
    "SOSTITUISCE": "Entity A sostituisce Entity B",
    "DIPENDE_DA": "Entity A dipende da Entity B",
}


class GraphExtractor:
    """Extract entities and relationships from Italian legal text."""

    def __init__(self):
        """Initialize graph extractor with Italian patterns."""
        self.entity_patterns = self._build_patterns()

    def _build_patterns(self) -> Dict[str, List[re.Pattern]]:
        """Build regex patterns for entity extraction."""
        return {
            "PARTY": [
                re.compile(r"(?:il|la)\s+(?:Sig\.|Dott\.|Avv\.|Ing\.)\s+[A-Z][a-zà-ù]+(?:\s+[A-Z][a-zà-ù]+)*", re.IGNORECASE),
                re.compile(r"(?:la società|l'azienda|la ditta)\s+[A-Z][A-Za-z\s&.]+(?:S\.p\.A\.|S\.r\.l\.|S\.n\.c\.|S\.a\.s\.)", re.IGNORECASE),
                re.compile(r"[A-Z][A-Za-z\s&.]+(?:S\.p\.A\.|S\.r\.l\.|S\.n\.c\.|S\.a\.s\.)"),
            ],
            "DATE": [
                re.compile(r"\d{1,2}[/-]\d{1,2}[/-]\d{2,4}"),
                re.compile(r"\d{1,2}\s+(?:gennaio|febbraio|marzo|aprile|maggio|giugno|luglio|agosto|settembre|ottobre|novembre|dicembre)\s+\d{4}", re.IGNORECASE),
                re.compile(r"entro\s+(?:il\s+)?\d{1,2}\s+giorni", re.IGNORECASE),
            ],
            "AMOUNT": [
                re.compile(r"€\s*[\d.,]+(?:\s*(?:milioni?|mila))?"),
                re.compile(r"[\d.,]+\s*euro", re.IGNORECASE),
                re.compile(r"(?:Euro|EUR)\s*[\d.,]+", re.IGNORECASE),
            ],
            "CLAUSE": [
                re.compile(r"(?:art\.|articolo)\s*\d+(?:[,\s]+comma\s*\d+)?(?:[,\s]+lett?\.\s*[a-z])?", re.IGNORECASE),
                re.compile(r"clausola\s+(?:n\.\s*)?\d+(?:\.\d+)?", re.IGNORECASE),
                re.compile(r"paragrafo\s+\d+(?:\.\d+)?", re.IGNORECASE),
            ],
            "JURISDICTION": [
                re.compile(r"Foro\s+(?:competente\s+)?di\s+[A-Z][a-zà-ù]+", re.IGNORECASE),
                re.compile(r"Tribunale\s+di\s+[A-Z][a-zà-ù]+", re.IGNORECASE),
                re.compile(r"legge\s+italiana", re.IGNORECASE),
                re.compile(r"normativa\s+(?:comunitaria|europea)", re.IGNORECASE),
            ],
            "PENALTY": [
                re.compile(r"penale\s+(?:di|pari a)\s+€?\s*[\d.,]+", re.IGNORECASE),
                re.compile(r"risarcimento\s+(?:danni|del danno)", re.IGNORECASE),
                re.compile(r"indennizzo\s+(?:di|pari a)\s+€?\s*[\d.,]+", re.IGNORECASE),
            ],
        }

    def extract_entities(self, text: str, doc_id: str) -> List[Dict]:
        """
        Extract entities from text.

        Args:
            text: Contract text
            doc_id: Document ID for reference

        Returns:
            List of entities with type, name, properties
        """
        entities = []
        entity_id = 0

        for entity_type, patterns in self.entity_patterns.items():
            for pattern in patterns:
                for match in pattern.finditer(text):
                    entity_text = match.group(0).strip()

                    # Calculate position
                    start_pos = match.start()
                    end_pos = match.end()

                    # Extract surrounding context (±50 chars)
                    context_start = max(0, start_pos - 50)
                    context_end = min(len(text), end_pos + 50)
                    context = text[context_start:context_end].strip()

                    entities.append({
                        "id": f"{doc_id}_entity_{entity_id}",
                        "entity_type": entity_type,
                        "name": entity_text,
                        "properties": {
                            "position": start_pos,
                            "context": context,
                            "confidence": 0.85,  # Rule-based confidence
                        },
                        "doc_id": doc_id,
                    })
                    entity_id += 1

        logger.info(f"Extracted {len(entities)} entities from {doc_id}")
        return entities

    def extract_relationships(
        self, text: str, entities: List[Dict]
    ) -> List[Dict]:
        """
        Extract relationships between entities.

        Args:
            text: Contract text
            entities: List of extracted entities

        Returns:
            List of relationships (source, target, type)
        """
        relationships = []
        relationship_id = 0

        # Simple heuristic: entities close together (within 100 chars) likely have relationships
        for i, entity_a in enumerate(entities):
            for entity_b in entities[i + 1:]:
                pos_a = entity_a["properties"]["position"]
                pos_b = entity_b["properties"]["position"]

                distance = abs(pos_b - pos_a)

                if distance < 200:  # Within 200 characters
                    # Infer relationship type based on entity types
                    rel_type = self._infer_relationship_type(
                        entity_a["entity_type"],
                        entity_b["entity_type"],
                        text[min(pos_a, pos_b):max(pos_a, pos_b)],
                    )

                    if rel_type:
                        relationships.append({
                            "id": f"rel_{relationship_id}",
                            "source_id": entity_a["id"],
                            "target_id": entity_b["id"],
                            "relationship_type": rel_type,
                            "properties": {
                                "distance": distance,
                                "confidence": 0.7,
                            },
                        })
                        relationship_id += 1

        logger.info(f"Extracted {len(relationships)} relationships")
        return relationships

    def _infer_relationship_type(
        self, type_a: str, type_b: str, context: str
    ) -> Optional[str]:
        """Infer relationship type based on entity types and context."""
        context_lower = context.lower()

        # PARTY → AMOUNT: payment relationship
        if type_a == "PARTY" and type_b == "AMOUNT":
            if any(word in context_lower for word in ["paga", "corrisponde", "versa"]):
                return "PAGA"

        # CLAUSE → PENALTY: clause defines penalty
        if type_a == "CLAUSE" and type_b == "PENALTY":
            return "DEFINISCE"

        # DATE → TERM: date specifies term
        if type_a == "DATE" and type_b == "TERM":
            return "SPECIFICA"

        # CLAUSE → CLAUSE: hierarchical relationship
        if type_a == "CLAUSE" and type_b == "CLAUSE":
            if "modifica" in context_lower:
                return "MODIFICA"
            if "sostituisce" in context_lower:
                return "SOSTITUISCE"
            return "RIFERISCE_A"

        # PARTY → JURISDICTION: party subject to jurisdiction
        if type_a == "PARTY" and type_b == "JURISDICTION":
            return "SOGGETTO_A"

        # Default: generic reference
        return "RIFERISCE_A"

    def extract_graph(self, text: str, doc_id: str) -> Dict:
        """
        Extract complete knowledge graph from text.

        Args:
            text: Contract text
            doc_id: Document ID

        Returns:
            Graph with nodes and edges
        """
        start_time = datetime.now()

        entities = self.extract_entities(text, doc_id)
        relationships = self.extract_relationships(text, entities)

        processing_time = (datetime.now() - start_time).total_seconds()

        return {
            "doc_id": doc_id,
            "nodes": entities,
            "edges": relationships,
            "stats": {
                "nodes_count": len(entities),
                "edges_count": len(relationships),
                "processing_time_s": processing_time,
            },
        }


# Singleton instance
_extractor: Optional[GraphExtractor] = None


def get_extractor() -> GraphExtractor:
    """Get or create graph extractor instance."""
    global _extractor
    if _extractor is None:
        _extractor = GraphExtractor()
    return _extractor
