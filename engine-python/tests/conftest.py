"""
Pytest configuration and fixtures
"""

import os
import tempfile
from pathlib import Path
from typing import Generator

import pytest
from fastapi.testclient import TestClient
from PIL import Image

from app.main import app


@pytest.fixture
def client() -> TestClient:
    """FastAPI test client"""
    return TestClient(app)


@pytest.fixture
def temp_dir() -> Generator[Path, None, None]:
    """Temporary directory for test files"""
    with tempfile.TemporaryDirectory() as tmpdir:
        yield Path(tmpdir)


@pytest.fixture
def sample_image(temp_dir: Path) -> Path:
    """Create a sample image with text for OCR testing"""
    img_path = temp_dir / "sample.png"

    # Create a simple white image
    img = Image.new("RGB", (400, 100), color="white")
    img.save(img_path)

    return img_path


@pytest.fixture
def sample_pdf(temp_dir: Path) -> Path:
    """Create a minimal PDF for testing"""
    pdf_path = temp_dir / "sample.pdf"

    # Minimal PDF with text
    # This is a simplified PDF structure for testing
    pdf_content = b"""%PDF-1.4
1 0 obj
<< /Type /Catalog /Pages 2 0 R >>
endobj
2 0 obj
<< /Type /Pages /Kids [3 0 R] /Count 1 >>
endobj
3 0 obj
<< /Type /Page /Parent 2 0 R /Resources 4 0 R /MediaBox [0 0 612 792] /Contents 5 0 R >>
endobj
4 0 obj
<< /Font << /F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> >> >>
endobj
5 0 obj
<< /Length 44 >>
stream
BT
/F1 12 Tf
100 700 Td
(Test PDF content) Tj
ET
endstream
endobj
xref
0 6
0000000000 65535 f
0000000009 00000 n
0000000058 00000 n
0000000115 00000 n
0000000214 00000 n
0000000299 00000 n
trailer
<< /Size 6 /Root 1 0 R >>
startxref
392
%%EOF
"""

    with open(pdf_path, "wb") as f:
        f.write(pdf_content)

    return pdf_path
