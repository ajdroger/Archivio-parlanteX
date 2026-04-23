"""
Contextual Retrieval service (Anthropic technique)

Enhances chunks by prepending document context before embedding.
Improves semantic search by providing more context to each chunk.

Reference: https://www.anthropic.com/news/contextual-retrieval
"""

import asyncio
from typing import List

import httpx
import structlog

from app.config import settings

logger = structlog.get_logger()


class ContextualRetrieval:
    """Contextual retrieval using LLM to generate chunk context"""

    def __init__(self):
        self.http_client = httpx.AsyncClient(timeout=30.0)

    async def contextualize_chunks(
        self,
        document_text: str,
        chunks: List[str],
    ) -> List[str]:
        """
        Add document context to each chunk using LLM

        Args:
            document_text: Full document text (or substantial excerpt)
            chunks: List of chunk texts to contextualize

        Returns:
            List of contextualized chunks (context prefix + original chunk)
        """
        if not chunks:
            return []

        logger.info(
            "contextualizing_chunks",
            doc_length=len(document_text),
            chunks_count=len(chunks),
        )

        # Process chunks in batches to avoid overwhelming LLM
        batch_size = settings.contextualization_batch_size
        contextualized = []

        for i in range(0, len(chunks), batch_size):
            batch = chunks[i : i + batch_size]

            # Process batch in parallel
            tasks = [
                self._generate_context(document_text, chunk)
                for chunk in batch
            ]

            batch_results = await asyncio.gather(*tasks)
            contextualized.extend(batch_results)

            logger.debug(
                "batch_contextualized",
                batch_num=i // batch_size + 1,
                batch_size=len(batch),
            )

        logger.info(
            "contextualization_complete",
            chunks_count=len(contextualized),
        )

        return contextualized

    async def _generate_context(
        self,
        document_text: str,
        chunk: str,
    ) -> str:
        """
        Generate context for a single chunk using LLM

        Prompt based on Anthropic's contextual retrieval technique.
        """
        # Truncate document if too long
        max_doc_len = settings.contextualization_max_doc_length
        if len(document_text) > max_doc_len:
            document_excerpt = document_text[:max_doc_len] + "..."
        else:
            document_excerpt = document_text

        # Anthropic-style prompt
        prompt = f"""Here is the document:
<document>
{document_excerpt}
</document>

Here is the chunk we want to situate within the whole document:
<chunk>
{chunk}
</chunk>

Please give a short succinct context to situate this chunk within the overall document for the purposes of improving search retrieval of the chunk. Answer only with the succinct context and nothing else."""

        try:
            # Call Ollama (or configured LLM endpoint)
            context = await self._call_llm(prompt)

            # Prepend context to original chunk
            contextualized = f"{context}\n\n{chunk}"

            return contextualized

        except Exception as e:
            logger.warning(
                "context_generation_failed",
                error=str(e),
                chunk_preview=chunk[:50],
            )

            # Fallback: return original chunk without context
            return chunk

    async def _call_llm(self, prompt: str) -> str:
        """
        Call LLM to generate context

        Uses Ollama by default (local, zero-cost).
        """
        url = f"{settings.ollama_url}/api/generate"

        payload = {
            "model": settings.contextualization_model,
            "prompt": prompt,
            "stream": False,
            "options": {
                "temperature": 0.3,  # Low temperature for factual context
                "num_predict": 100,  # Short context only
            },
        }

        try:
            response = await self.http_client.post(url, json=payload)
            response.raise_for_status()

            result = response.json()
            context = result.get("response", "").strip()

            return context

        except Exception as e:
            logger.error("llm_call_failed", error=str(e))
            raise

    async def close(self):
        """Close HTTP client"""
        await self.http_client.aclose()


# Global singleton
_contextual_instance: ContextualRetrieval | None = None


def get_contextual_retrieval() -> ContextualRetrieval:
    """Get global contextual retrieval instance"""
    global _contextual_instance

    if _contextual_instance is None:
        _contextual_instance = ContextualRetrieval()

    return _contextual_instance
