# 🐍 Archivio Parlante — Python AI Worker

FastAPI microservice for document parsing, OCR, reranking, and knowledge graph extraction.

## Features (Fase 2.1)

- **PDF Parsing** with multi-strategy fallback:
  1. PyMuPDF (fast text extraction)
  2. pdfplumber (better table handling)
  3. unstructured (robust layout analysis)
  4. Tesseract OCR (scanned PDFs)
- **Image OCR** with pytesseract (Italian + English)
- **Zero Hallucination**: Only extracts text actually present in documents
- **Structured logging** with structlog (JSON format)
- **FastAPI** async endpoints with Pydantic validation

## Quick Start

### Native Execution (Recommended for Windows Development)

**Note**: The Python Worker runs **natively on Windows** (not in Docker) due to build issues with ML dependencies in WSL2/Docker Desktop.

```bash
# 1. Create virtual environment (Python 3.11+ from python.org, NOT Microsoft Store)
cd engine-python
python -m venv venv

# 2. Activate virtual environment
.\venv\Scripts\Activate.ps1  # PowerShell
# OR
venv\Scripts\activate.bat    # CMD

# 3. Install core dependencies
pip install -r requirements-minimal.txt

# 4. Configure environment
cp .env.example .env
# Edit .env and set RUST_ENGINE_INTERNAL_TOKEN from root .env

# 5. Run server
uvicorn app.main:app --host 0.0.0.0 --port 8091 --reload

# 6. (Optional) Install ML dependencies when needed
pip install torch FlagEmbedding spacy networkx
python -m spacy download it_core_news_lg
```

**Tesseract OCR** (optional, for scanned PDFs):
- Download: https://github.com/UB-Mannheim/tesseract/wiki
- Add to PATH or set `TESSERACT_CMD` in `.env`

### Testing

```bash
# Run tests
pytest

# Run tests with coverage
pytest --cov=app --cov-report=term-missing
```

### Docker (Not Currently Used)

Docker execution is disabled due to persistent build failures with ML dependencies.
See commit `47f09dc` for rationale.

## API Endpoints

### `GET /health`

Health check endpoint.

**Response:**
```json
{
  "status": "ok",
  "service": "python-worker",
  "version": "0.1.0"
}
```

### `POST /parse`

Parse a PDF or image document into chunks.

**Request:**
```json
{
  "kb_id": "kb_123",
  "doc_id": "doc_456",
  "file_path": "/shared/uploads/contract.pdf",
  "mime_type": "application/pdf",
  "use_ocr": false
}
```

**Response:**
```json
{
  "doc_id": "doc_456",
  "kb_id": "kb_123",
  "chunks": [
    {
      "text": "CONTRATTO DI LOCAZIONE...",
      "page_number": 1,
      "metadata": {"method": "pymupdf"}
    }
  ],
  "total_chunks": 15,
  "total_pages": 8,
  "parsing_method": "pymupdf",
  "processing_ms": 234
}
```

**Parsing Methods:**
- `pymupdf` — Fast, text-based PDFs
- `pdfplumber` — Better table extraction
- `unstructured` — Complex layouts
- `ocr` — Scanned PDFs / images

**Supported MIME types:**
- `application/pdf`
- `image/png`
- `image/jpeg`
- `image/tiff`

## Architecture

```
engine-python/
├── app/
│   ├── main.py              # FastAPI app entry point
│   ├── config.py            # Settings (pydantic-settings)
│   ├── schemas.py           # Request/response models
│   ├── routers/
│   │   └── parse.py         # /parse endpoint
│   └── services/
│       ├── pdf_parser.py    # Multi-strategy PDF parser
│       └── ocr_service.py   # Tesseract OCR wrapper
├── tests/
│   ├── conftest.py          # Pytest fixtures
│   ├── test_parse.py        # Integration tests
│   └── test_pdf_parser.py   # Unit tests
├── requirements.txt
├── Dockerfile
└── pytest.ini
```

## Configuration

Environment variables (`.env`):

```env
# Shared storage
SHARED_UPLOADS_PATH=/shared/uploads
MAX_UPLOAD_SIZE_MB=200

# OCR
TESSERACT_LANG=ita+eng
OCR_DPI=300

# Parsing
PDF_MAX_PAGES=500
CHUNK_SIZE=512
CHUNK_OVERLAP=50

# Logging
PYTHON_LOG_LEVEL=INFO
```

## Testing

Coverage target: **80%**

```bash
# Run all tests
pytest

# Run specific test file
pytest tests/test_parse.py

# Run with coverage report
pytest --cov=app --cov-report=html
# Open htmlcov/index.html

# Run only unit tests
pytest tests/test_pdf_parser.py

# Run only integration tests
pytest tests/test_parse.py
```

## Future Phases

- **Fase 2.2**: BGE reranker integration
- **Fase 2.3**: Contextual retrieval (Anthropic technique)
- **Fase 2.4**: Knowledge graph extraction (spaCy NER)

## Dependencies

Key libraries:
- `fastapi` — Modern async web framework
- `pydantic` — Data validation
- `pymupdf` — Fast PDF parsing
- `pdfplumber` — Table extraction
- `unstructured` — Robust document parsing
- `pytesseract` — OCR wrapper
- `structlog` — Structured logging
- `pytest` — Testing framework

System packages (Dockerfile):
- `tesseract-ocr` — OCR engine
- `tesseract-ocr-ita` — Italian language data
- `poppler-utils` — PDF utilities
- `libmagic1` — File type detection

## Troubleshooting

### OCR not working

Ensure Tesseract is installed with Italian language pack:
```bash
apt-get install tesseract-ocr tesseract-ocr-ita
```

### Import errors

Rebuild Docker image after adding dependencies:
```bash
docker-compose build python-worker
```

### Test failures

Check that `/shared/uploads` directory exists and is writable.

## License

MIT — See [LICENSE](../LICENSE)
