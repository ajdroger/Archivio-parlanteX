"""
Knowledge Graph extraction service

Extracts entities and relations from Italian legal contracts using spaCy NER.
Optimized for contract forensic analysis.

Entity types:
- PARTIES: Contracting parties (companies, individuals)
- DATES: Contract dates, deadlines, durations
- AMOUNTS: Monetary amounts (canone, penali, depositi)
- CLAUSES: Contract clauses (Art. 1, Art. 2, etc.)
- JURISDICTIONS: Foro competente, tribunali
- PENALTIES: Penali, sanzioni
"""

import re
from typing import Dict, List, Tuple

import spacy
import structlog
from spacy.language import Language

from app.config import settings

logger = structlog.get_logger()


class KGExtractor:
    """Knowledge graph extractor for Italian legal contracts"""

    def __init__(self):
        self.nlp: Language | None = None
        self._initialized = False

    def initialize(self) -> None:
        """Initialize spaCy model"""
        if self._initialized:
            return

        logger.info("initializing_spacy", model=settings.kg_spacy_model)

        try:
            # Load Italian model
            self.nlp = spacy.load(settings.kg_spacy_model)

            # Add custom entity ruler for legal-specific entities
            if "entity_ruler" not in self.nlp.pipe_names:
                ruler = self.nlp.add_pipe("entity_ruler", before="ner")
                self._add_legal_patterns(ruler)

            self._initialized = True

            logger.info("spacy_initialized", model=settings.kg_spacy_model)

        except OSError as e:
            logger.error(
                "spacy_model_not_found",
                model=settings.kg_spacy_model,
                error=str(e),
                hint="Run: python -m spacy download it_core_news_lg",
            )
            raise

    def _add_legal_patterns(self, ruler) -> None:
        """Add legal-specific entity patterns"""
        patterns = [
            # Clauses: "Art. 1", "Articolo 2", "Art.1"
            {"label": "CLAUSE", "pattern": [{"TEXT": {"REGEX": r"Art\.?\s*\d+"}}]},
            {"label": "CLAUSE", "pattern": [{"LOWER": "articolo"}, {"IS_DIGIT": True}]},
            # Jurisdictions
            {
                "label": "JURISDICTION",
                "pattern": [
                    {"LOWER": {"IN": ["tribunale", "foro"]}},
                    {"LOWER": "di"},
                    {"IS_TITLE": True},
                ],
            },
            # Penalties keywords
            {"label": "PENALTY", "pattern": [{"LOWER": {"IN": ["penale", "penali", "sanzione"]}}]},
        ]

        ruler.add_patterns(patterns)

    def extract(self, text: str, doc_id: str) -> Tuple[List[Dict], List[Dict]]:
        """
        Extract knowledge graph from text

        Args:
            text: Contract text
            doc_id: Document ID

        Returns:
            (nodes, edges) where:
            - nodes: [{"id": str, "entity_type": str, "name": str, "properties": dict}]
            - edges: [{"source_id": str, "target_id": str, "relation_type": str, "properties": dict}]
        """
        if not self._initialized:
            self.initialize()

        logger.info("extracting_kg", doc_id=doc_id, text_length=len(text))

        # Process with spaCy
        doc = self.nlp(text)

        # Extract entities
        nodes = self._extract_entities(doc, doc_id)

        # Extract relations (simple pattern-based for now)
        edges = self._extract_relations(doc, nodes, doc_id)

        logger.info(
            "kg_extracted",
            doc_id=doc_id,
            nodes_count=len(nodes),
            edges_count=len(edges),
        )

        return nodes, edges

    def _extract_entities(self, doc, doc_id: str) -> List[Dict]:
        """Extract entities from spaCy doc"""
        nodes = []
        seen_entities = set()

        for ent in doc.ents:
            # Map spaCy labels to our taxonomy
            entity_type = self._map_entity_type(ent.label_)

            if not entity_type:
                continue

            # Create unique ID
            entity_id = f"{doc_id}_{entity_type}_{len(nodes)}"

            # Avoid duplicates
            entity_key = (entity_type, ent.text.strip())
            if entity_key in seen_entities:
                continue

            seen_entities.add(entity_key)

            nodes.append(
                {
                    "id": entity_id,
                    "entity_type": entity_type,
                    "name": ent.text.strip(),
                    "properties": {
                        "start_char": ent.start_char,
                        "end_char": ent.end_char,
                        "confidence": 1.0,  # spaCy doesn't provide scores
                    },
                }
            )

        # Additional extraction: monetary amounts with regex
        self._extract_amounts(doc.text, nodes, doc_id, seen_entities)

        # Extract durations (e.g., "24 mesi", "6 anni")
        self._extract_durations(doc.text, nodes, doc_id, seen_entities)

        return nodes

    def _map_entity_type(self, spacy_label: str) -> str | None:
        """Map spaCy entity label to our taxonomy"""
        mapping = {
            "ORG": "PARTIES",
            "PERSON": "PARTIES",
            "DATE": "DATES",
            "MONEY": "AMOUNTS",
            "CLAUSE": "CLAUSES",
            "JURISDICTION": "JURISDICTIONS",
            "PENALTY": "PENALTIES",
        }

        return mapping.get(spacy_label)

    def _extract_amounts(self, text: str, nodes: List[Dict], doc_id: str, seen: set) -> None:
        """Extract monetary amounts with regex"""
        # Pattern: "Euro 50.000,00" or "€ 50.000"
        pattern = r"(?:Euro|EUR|€)\s*([0-9]{1,3}(?:\.[0-9]{3})*(?:,[0-9]{2})?)"

        for match in re.finditer(pattern, text, re.IGNORECASE):
            amount_text = match.group(0)

            entity_key = ("AMOUNTS", amount_text)
            if entity_key in seen:
                continue

            seen.add(entity_key)

            entity_id = f"{doc_id}_AMOUNTS_{len(nodes)}"

            nodes.append(
                {
                    "id": entity_id,
                    "entity_type": "AMOUNTS",
                    "name": amount_text,
                    "properties": {
                        "start_char": match.start(),
                        "end_char": match.end(),
                        "confidence": 0.95,
                    },
                }
            )

    def _extract_durations(self, text: str, nodes: List[Dict], doc_id: str, seen: set) -> None:
        """Extract contract durations"""
        # Pattern: "24 mesi", "6 anni", "12 months"
        pattern = r"(\d+)\s+(mes[ei]|ann[oi]|giorni|settimane)"

        for match in re.finditer(pattern, text, re.IGNORECASE):
            duration_text = match.group(0)

            entity_key = ("DATES", duration_text)
            if entity_key in seen:
                continue

            seen.add(entity_key)

            entity_id = f"{doc_id}_DATES_{len(nodes)}"

            nodes.append(
                {
                    "id": entity_id,
                    "entity_type": "DATES",
                    "name": duration_text,
                    "properties": {
                        "start_char": match.start(),
                        "end_char": match.end(),
                        "confidence": 0.9,
                        "duration": True,
                    },
                }
            )

    def _extract_relations(
        self, doc, nodes: List[Dict], doc_id: str
    ) -> List[Dict]:
        """
        Extract relations between entities

        Simple pattern-based for now. Future: use dependency parsing.
        """
        edges = []

        # Simple heuristic: entities mentioned in same sentence are related
        # Group entities by sentence
        sentence_entities = {}

        for sent in doc.sents:
            sent_start = sent.start_char
            sent_end = sent.end_char

            entities_in_sent = [
                node
                for node in nodes
                if node["properties"]["start_char"] >= sent_start
                and node["properties"]["end_char"] <= sent_end
            ]

            if len(entities_in_sent) >= 2:
                sentence_entities[sent.text] = entities_in_sent

        # Create relations between co-occurring entities
        for sent_text, entities in sentence_entities.items():
            # Pair entities (limited to avoid explosion)
            for i, entity1 in enumerate(entities[:5]):
                for entity2 in entities[i + 1 : i + 3]:
                    relation_type = self._infer_relation_type(
                        entity1["entity_type"], entity2["entity_type"]
                    )

                    if relation_type:
                        edges.append(
                            {
                                "source_id": entity1["id"],
                                "target_id": entity2["id"],
                                "relation_type": relation_type,
                                "properties": {
                                    "context": sent_text[:200],
                                    "confidence": 0.7,
                                },
                            }
                        )

        return edges

    def _infer_relation_type(self, type1: str, type2: str) -> str | None:
        """Infer relation type between entity types"""
        relations = {
            ("PARTIES", "PARTIES"): "CONTRATTO_CON",
            ("PARTIES", "AMOUNTS"): "PAGA",
            ("PARTIES", "DATES"): "ENTRO_DATA",
            ("PARTIES", "JURISDICTIONS"): "SOTTO_GIURISDIZIONE",
            ("CLAUSES", "AMOUNTS"): "DEFINISCE_IMPORTO",
            ("CLAUSES", "DATES"): "DEFINISCE_DURATA",
            ("CLAUSES", "PENALTIES"): "PREVEDE_PENALE",
        }

        key = (type1, type2)
        reverse_key = (type2, type1)

        return relations.get(key) or relations.get(reverse_key)


# Global singleton
_kg_extractor_instance: KGExtractor | None = None


def get_kg_extractor() -> KGExtractor:
    """Get global KG extractor instance"""
    global _kg_extractor_instance

    if _kg_extractor_instance is None:
        _kg_extractor_instance = KGExtractor()

    return _kg_extractor_instance
