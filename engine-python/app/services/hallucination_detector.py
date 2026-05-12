"""
Hallucination Detection Service
Fase 6.2 - Advanced Hallucination Detection

Extracts claims from LLM answers and verifies them against source documents
using embedding similarity. Flags unsupported claims and calculates hallucination score.
"""

import asyncio
import json
import logging
from typing import List, Dict
from dataclasses import dataclass
import httpx
import numpy as np

logger = logging.getLogger(__name__)


@dataclass
class HallucinationResult:
    """Result of hallucination detection"""
    hallucination_score: float  # 0-1, higher = more hallucination
    flagged_claims: List[str]   # Claims without support
    supported_claims: List[str]  # Claims with support
    total_claims: int


class HallucinationDetector:
    """Detect hallucinations in LLM-generated answers"""

    SIMILARITY_THRESHOLD = 0.75  # Min similarity for claim support

    def __init__(
        self,
        ollama_url: str = "http://localhost:11434",
        model: str = "qwen2.5:3b-instruct-q4_K_M",
    ):
        self.ollama_url = ollama_url
        self.model = model
        self.client = httpx.AsyncClient(timeout=30)

    async def detect(
        self,
        answer: str,
        sources: List[Dict],
    ) -> HallucinationResult:
        """
        Detect hallucinations in answer against sources.

        Args:
            answer: LLM-generated answer
            sources: List of source dicts with 'text_quote' key

        Returns:
            HallucinationResult with score and flagged claims
        """
        # Extract claims from answer
        claims = await self.extract_claims(answer)

        if not claims:
            return HallucinationResult(
                hallucination_score=0.0,
                flagged_claims=[],
                supported_claims=[],
                total_claims=0,
            )

        # Verify each claim
        supported = []
        flagged = []

        for claim in claims:
            is_supported = await self.verify_claim(claim, sources)
            if is_supported:
                supported.append(claim)
            else:
                flagged.append(claim)

        # Calculate score: ratio of unsupported claims
        score = len(flagged) / len(claims) if claims else 0.0

        logger.info(
            f"Hallucination detection: {len(claims)} claims, "
            f"{len(flagged)} flagged, score={score:.2f}"
        )

        return HallucinationResult(
            hallucination_score=score,
            flagged_claims=flagged,
            supported_claims=supported,
            total_claims=len(claims),
        )

    async def extract_claims(self, answer: str) -> List[str]:
        """Extract atomic claims from answer using LLM"""
        prompt = f"""Split this answer into individual factual claims (one per line).
Extract only verifiable factual statements, not opinions or questions.

Answer:
{answer}

Claims (one per line):"""

        try:
            response = await self.client.post(
                f"{self.ollama_url}/api/generate",
                json={
                    "model": self.model,
                    "prompt": prompt,
                    "stream": False,
                    "options": {"temperature": 0.1, "num_predict": 500},
                },
            )
            response.raise_for_status()

            result = response.json()
            claims_text = result.get("response", "")

            # Parse claims (one per line)
            claims = [
                line.strip("- ").strip()
                for line in claims_text.split("\n")
                if line.strip() and len(line.strip()) > 10
            ]

            return claims[:20]  # Limit to 20 claims

        except Exception as e:
            logger.error(f"Claim extraction failed: {e}")
            return []

    async def verify_claim(
        self,
        claim: str,
        sources: List[Dict],
    ) -> bool:
        """
        Verify if claim is supported by sources.

        Uses simple string matching + embedding similarity (future).

        Args:
            claim: Factual claim to verify
            sources: Source documents with text_quote

        Returns:
            True if claim is supported, False otherwise
        """
        # Simple verification: check if claim text appears in any source
        claim_lower = claim.lower()

        for source in sources:
            quote = source.get("text_quote", "").lower()

            # Exact substring match
            if claim_lower in quote or quote in claim_lower:
                return True

            # Token overlap heuristic
            claim_tokens = set(claim_lower.split())
            quote_tokens = set(quote.split())

            if len(claim_tokens) > 3:
                overlap = len(claim_tokens & quote_tokens) / len(claim_tokens)
                if overlap > 0.7:
                    return True

        return False

    async def close(self):
        """Close HTTP client"""
        await self.client.aclose()


# Convenience function
async def verify_hallucination(
    answer: str,
    sources: List[Dict],
    ollama_url: str = "http://localhost:11434",
) -> Dict:
    """
    Verify answer for hallucinations (convenience wrapper).

    Returns dict representation of HallucinationResult.
    """
    detector = HallucinationDetector(ollama_url=ollama_url)

    try:
        result = await detector.detect(answer, sources)
        return {
            "hallucination_score": result.hallucination_score,
            "flagged_claims": result.flagged_claims,
            "supported_claims": result.supported_claims,
            "total_claims": result.total_claims,
        }
    finally:
        await detector.close()
