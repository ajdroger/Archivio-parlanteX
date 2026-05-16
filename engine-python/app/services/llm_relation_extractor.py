"""
LLM-Based Relation Extraction for Knowledge Graphs
Fase 6.1 - Enhanced Knowledge Graph RAG

Uses Ollama LLM to extract typed relations from legal text with higher precision
than heuristic methods. Extracts relations like SIGNS, OBLIGATED_TO, PAYS, etc.
"""

import asyncio
import json
import logging
from typing import List, Dict, Optional, Tuple
from dataclasses import dataclass
import httpx

logger = logging.getLogger(__name__)


@dataclass
class Entity:
    """Entity extracted from text"""
    text: str
    type: str  # PERSON, ORG, DATE, MONEY, CLAUSE, etc.
    start: int
    end: int


@dataclass
class Relation:
    """Typed relation between entities"""
    source: str
    relation_type: str
    target: str
    confidence: float
    text_evidence: str


class LLMRelationExtractor:
    """
    Extract typed relations from legal text using LLM.

    Uses Ollama qwen2.5:3b for efficient relation extraction with
    retry logic and timeout handling.
    """

    # Legal relation types we want to extract
    RELATION_TYPES = [
        "SIGNS",           # Party signs document
        "OBLIGATED_TO",    # Party obligated to perform action
        "PAYS",            # Payment obligation
        "RECEIVES",        # Receives payment/goods
        "GOVERNED_BY",     # Subject to law/jurisdiction
        "EXPIRES_ON",      # Expiration date
        "REFERS_TO",       # References another document/clause
        "AMENDS",          # Amends previous agreement
        "TERMINATES",      # Termination condition
        "CONTAINS_CLAUSE", # Document contains clause
    ]

    def __init__(
        self,
        ollama_url: str = "http://localhost:11434",
        model: str = "qwen2.5:3b-instruct-q4_K_M",
        timeout: int = 30,
        max_retries: int = 3,
    ):
        """
        Initialize LLM relation extractor.

        Args:
            ollama_url: Ollama API base URL
            model: Model to use (lightweight for speed)
            timeout: Request timeout in seconds
            max_retries: Maximum retry attempts
        """
        self.ollama_url = ollama_url
        self.model = model
        self.timeout = timeout
        self.max_retries = max_retries
        self.client = httpx.AsyncClient(timeout=timeout)

    async def extract_relations(
        self,
        text: str,
        entities: List[Entity],
    ) -> List[Relation]:
        """
        Extract typed relations from text given pre-identified entities.

        Args:
            text: Legal text to analyze
            entities: Pre-extracted entities from spaCy NER

        Returns:
            List of typed relations with confidence scores
        """
        if len(entities) < 2:
            logger.debug("Less than 2 entities, skipping relation extraction")
            return []

        # Build prompt with entity context
        prompt = self._build_extraction_prompt(text, entities)

        # Call LLM with retry logic
        for attempt in range(self.max_retries):
            try:
                relations = await self._call_ollama(prompt)
                logger.info(f"Extracted {len(relations)} relations from text")
                return relations
            except Exception as e:
                logger.warning(
                    f"Relation extraction attempt {attempt + 1} failed: {e}"
                )
                if attempt == self.max_retries - 1:
                    logger.error("Max retries reached, returning empty relations")
                    return []
                await asyncio.sleep(2 ** attempt)  # Exponential backoff

        return []

    def _build_extraction_prompt(
        self,
        text: str,
        entities: List[Entity],
    ) -> str:
        """Build prompt for LLM relation extraction."""
        entity_list = "\n".join(
            f"- {e.text} ({e.type})" for e in entities[:20]  # Limit to avoid token overflow
        )

        relation_types = ", ".join(self.RELATION_TYPES)

        prompt = f"""Analyze the following legal text and extract structured relations between entities.

Text:
{text[:2000]}

Entities identified:
{entity_list}

Task: Extract relations between these entities. For each relation, specify:
- source entity
- relation type (one of: {relation_types})
- target entity
- confidence (0.0-1.0)
- text evidence (quote from document)

Return ONLY valid JSON array, no other text:
[
  {{
    "source": "Acme Corp",
    "relation_type": "SIGNS",
    "target": "Contract 2024-001",
    "confidence": 0.9,
    "text_evidence": "Acme Corp hereby signs this agreement..."
  }}
]

JSON output:"""

        return prompt

    async def _call_ollama(self, prompt: str) -> List[Relation]:
        """
        Call Ollama API for relation extraction.

        Args:
            prompt: Extraction prompt

        Returns:
            List of extracted relations
        """
        payload = {
            "model": self.model,
            "prompt": prompt,
            "stream": False,
            "options": {
                "temperature": 0.1,  # Low temperature for consistency
                "top_p": 0.9,
                "num_predict": 1000,  # Limit tokens
            }
        }

        response = await self.client.post(
            f"{self.ollama_url}/api/generate",
            json=payload,
        )
        response.raise_for_status()

        result = response.json()
        response_text = result.get("response", "")

        # Parse JSON from response
        relations = self._parse_json_response(response_text)
        return relations

    def _parse_json_response(self, response_text: str) -> List[Relation]:
        """
        Parse JSON response from LLM, handling common issues.

        Args:
            response_text: Raw LLM response

        Returns:
            List of Relation objects
        """
        # Try to find JSON array in response
        start_idx = response_text.find("[")
        end_idx = response_text.rfind("]")

        if start_idx == -1 or end_idx == -1:
            logger.warning("No JSON array found in LLM response")
            return []

        json_str = response_text[start_idx:end_idx + 1]

        try:
            relations_data = json.loads(json_str)

            if not isinstance(relations_data, list):
                logger.warning("LLM response is not a list")
                return []

            relations = []
            for item in relations_data:
                try:
                    relation = Relation(
                        source=item["source"],
                        relation_type=item["relation_type"],
                        target=item["target"],
                        confidence=float(item.get("confidence", 0.5)),
                        text_evidence=item.get("text_evidence", ""),
                    )

                    # Validate relation type
                    if relation.relation_type in self.RELATION_TYPES:
                        relations.append(relation)
                    else:
                        logger.debug(
                            f"Skipping relation with invalid type: {relation.relation_type}"
                        )

                except (KeyError, ValueError) as e:
                    logger.warning(f"Skipping malformed relation: {e}")
                    continue

            return relations

        except json.JSONDecodeError as e:
            logger.error(f"Failed to parse JSON from LLM response: {e}")
            return []

    async def close(self):
        """Close HTTP client."""
        await self.client.aclose()


# Convenience function for use in knowledge graph service
async def extract_relations_with_llm(
    text: str,
    entities: List[Dict],
    ollama_url: str = "http://localhost:11434",
) -> List[Dict]:
    """
    Extract relations using LLM (convenience wrapper).

    Args:
        text: Text to analyze
        entities: List of entity dicts with 'text' and 'type' keys
        ollama_url: Ollama API URL

    Returns:
        List of relation dicts
    """
    # Convert entity dicts to Entity objects
    entity_objects = [
        Entity(
            text=e.get("text", ""),
            type=e.get("type", "UNKNOWN"),
            start=e.get("start", 0),
            end=e.get("end", 0),
        )
        for e in entities
    ]

    extractor = LLMRelationExtractor(ollama_url=ollama_url)

    try:
        relations = await extractor.extract_relations(text, entity_objects)

        # Convert back to dicts
        return [
            {
                "source": r.source,
                "relation_type": r.relation_type,
                "target": r.target,
                "confidence": r.confidence,
                "text_evidence": r.text_evidence,
            }
            for r in relations
        ]
    finally:
        await extractor.close()
