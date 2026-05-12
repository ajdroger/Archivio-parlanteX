"""
BGE Reranker v2-m3 service

Cross-encoder reranker for improving hybrid search results.
Model: BAAI/bge-reranker-v2-m3 (~560MB, multilingual)
"""

import asyncio
from typing import List, Tuple, TYPE_CHECKING

import structlog

# Lazy import ML dependencies (not in minimal install)
if TYPE_CHECKING:
    import torch
    from FlagEmbedding import FlagReranker

from app.config import settings

logger = structlog.get_logger()


class BGEReranker:
    """BGE reranker v2-m3 cross-encoder"""

    def __init__(self):
        self.model = None
        self.device = None
        self._initialized = False

    def initialize(self) -> None:
        """
        Initialize reranker model (requires torch and FlagEmbedding)

        Lazy loading: called on first use or at startup.
        Auto-detects CUDA/CPU.
        """
        if self._initialized:
            return

        logger.info("initializing_bge_reranker", model=settings.reranker_model)

        # Lazy import ML dependencies
        try:
            import torch
            from FlagEmbedding import FlagReranker
        except ImportError as e:
            logger.error("ml_dependencies_not_installed", error=str(e))
            raise RuntimeError(
                "ML dependencies not installed. Run: pip install torch FlagEmbedding"
            ) from e

        # Store for use in class
        self._torch = torch
        self._FlagReranker = FlagReranker

        # Auto-detect device
        if settings.reranker_device == "auto":
            if torch.cuda.is_available():
                self.device = "cuda"
                logger.info(
                    "cuda_detected",
                    device_name=torch.cuda.get_device_name(0),
                    vram_gb=torch.cuda.get_device_properties(0).total_memory / 1e9,
                )
            else:
                self.device = "cpu"
                logger.warning("cuda_not_available", fallback="cpu")
        else:
            self.device = settings.reranker_device

        # Load model
        try:
            self.model = self._FlagReranker(
                settings.reranker_model,
                use_fp16=self.device == "cuda",  # FP16 only on CUDA
                device=self.device,
            )

            self._initialized = True

            logger.info(
                "reranker_initialized",
                model=settings.reranker_model,
                device=self.device,
                fp16=self.device == "cuda",
            )

        except Exception as e:
            logger.error("reranker_init_failed", error=str(e))
            raise

    async def rerank(
        self,
        query: str,
        passages: List[str],
        top_k: int = 5,
    ) -> List[Tuple[int, float]]:
        """
        Rerank passages by relevance to query

        Args:
            query: Search query
            passages: List of text passages to rerank
            top_k: Return top K results

        Returns:
            List of (original_index, score) tuples, sorted by score descending
        """
        if not self._initialized:
            self.initialize()

        if not passages:
            return []

        logger.debug(
            "reranking",
            query_len=len(query),
            passages_count=len(passages),
            top_k=top_k,
        )

        # Prepare pairs
        pairs = [[query, passage] for passage in passages]

        # Run reranking in thread pool (blocking operation)
        loop = asyncio.get_event_loop()
        scores = await loop.run_in_executor(
            None,
            self._compute_scores_sync,
            pairs,
        )

        # Create (index, score) pairs and sort by score descending
        indexed_scores = list(enumerate(scores))
        indexed_scores.sort(key=lambda x: x[1], reverse=True)

        # Take top K
        top_results = indexed_scores[:top_k]

        logger.info(
            "reranking_complete",
            input_count=len(passages),
            output_count=len(top_results),
            top_score=top_results[0][1] if top_results else None,
        )

        return top_results

    def _compute_scores_sync(self, pairs: List[List[str]]) -> List[float]:
        """
        Synchronous score computation

        Called in thread pool executor.
        """
        try:
            scores = self.model.compute_score(pairs, normalize=True)

            # Ensure list format (single pair returns float)
            if isinstance(scores, float):
                scores = [scores]

            return scores

        except Exception as e:
            logger.error("reranking_failed", error=str(e))
            raise


# Global singleton instance
_reranker_instance: BGEReranker | None = None


def get_reranker() -> BGEReranker:
    """Get global reranker instance (singleton)"""
    global _reranker_instance

    if _reranker_instance is None:
        _reranker_instance = BGEReranker()

    return _reranker_instance
