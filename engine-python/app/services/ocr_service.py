"""
OCR service using Tesseract

Handles image-based text extraction for scanned PDFs and images.
"""

import asyncio
from pathlib import Path

import pytesseract
import structlog
from PIL import Image

from app.config import settings

logger = structlog.get_logger()


class OCRService:
    """OCR service with pytesseract"""

    async def extract_text(self, image_path: str) -> str:
        """
        Extract text from image using OCR

        Args:
            image_path: Path to image file

        Returns:
            Extracted text
        """
        path = Path(image_path)

        if not path.exists():
            raise FileNotFoundError(f"Image not found: {image_path}")

        logger.debug("ocr_extracting", image_path=image_path)

        # Run OCR in thread pool (blocking operation)
        loop = asyncio.get_event_loop()
        text = await loop.run_in_executor(
            None,
            self._extract_sync,
            image_path,
        )

        logger.info(
            "ocr_extracted",
            image_path=image_path,
            text_length=len(text),
        )

        return text

    def _extract_sync(self, image_path: str) -> str:
        """
        Synchronous OCR extraction

        Called in thread pool executor.
        """
        try:
            # Open and preprocess image
            img = Image.open(image_path)

            # Convert to RGB if needed
            if img.mode != "RGB":
                img = img.convert("RGB")

            # Run Tesseract
            text = pytesseract.image_to_string(
                img,
                lang=settings.tesseract_lang,
                config="--psm 1",  # Automatic page segmentation with OSD
            )

            return text.strip()

        except Exception as e:
            logger.error("ocr_failed", error=str(e), image_path=image_path)
            raise

    async def extract_text_from_multiple(
        self, image_paths: list[str]
    ) -> list[str]:
        """
        Extract text from multiple images in parallel

        Args:
            image_paths: List of image file paths

        Returns:
            List of extracted texts (same order as input)
        """
        tasks = [self.extract_text(path) for path in image_paths]
        return await asyncio.gather(*tasks)
