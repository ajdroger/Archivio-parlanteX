"""
Archivio Parlante — Python AI Worker
FastAPI server for PDF parsing, reranking, and knowledge graph extraction
"""

import logging
from contextlib import asynccontextmanager

import structlog
from fastapi import FastAPI, Request
from fastapi.exceptions import RequestValidationError
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
import traceback

from app.routers import contextualize, extract_kg, parse, rerank, verify_hallucination
from app.config import settings

# Configure structured logging
structlog.configure(
    processors=[
        structlog.processors.TimeStamper(fmt="iso"),
        structlog.processors.add_log_level,
        structlog.dev.ConsoleRenderer(),
    ]
)

logger = structlog.get_logger()


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Lifespan context manager for startup/shutdown"""
    logger.info("🐍 Python AI Worker starting...")

    # Initialize BGE reranker (lazy loading, but warm up on startup)
    try:
        from app.services.reranker import get_reranker
        reranker = get_reranker()
        reranker.initialize()
        logger.info("✅ BGE reranker initialized")
    except Exception as e:
        logger.warning("reranker_init_skipped", error=str(e))

    # Initialize hallucination detector (Fase 6.2)
    try:
        from app.services.hallucination_detector import HallucinationDetector
        detector = HallucinationDetector()
        logger.info("✅ Hallucination detector ready")
    except Exception as e:
        logger.warning("hallucination_detector_init_skipped", error=str(e))

    # TODO Fase 2.3+: Initialize spaCy NER, contextual retrieval, etc.
    yield
    logger.info("Python AI Worker shutting down...")


# Create FastAPI app
app = FastAPI(
    title="Archivio Parlante Python AI Worker",
    description="PDF parsing, OCR, reranking, and knowledge graph extraction",
    version="0.1.0",
    lifespan=lifespan,
)

# CORS middleware (configured from environment)
app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.cors_origins,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Include routers
app.include_router(parse.router)
app.include_router(rerank.router)
app.include_router(contextualize.router)
app.include_router(extract_kg.router)
app.include_router(verify_hallucination.router)


# Exception handlers
@app.exception_handler(RequestValidationError)
async def validation_exception_handler(request: Request, exc: RequestValidationError):
    """Handle validation errors"""
    # Safely convert errors to string to avoid JSON serialization issues with embedded exception objects
    safe_errors = []
    for error in exc.errors():
        safe_error = dict(error)
        if "ctx" in safe_error and "error" in safe_error["ctx"]:
            safe_error["ctx"] = {k: str(v) if k == "error" else v for k, v in safe_error["ctx"].items()}
        safe_errors.append(safe_error)
        
    logger.warning("validation_error", errors=safe_errors, body=exc.body)
    return JSONResponse(
        status_code=400,
        content={
            "error": "Validation error",
            "detail": safe_errors,
        },
    )


@app.exception_handler(Exception)
async def global_exception_handler(request: Request, exc: Exception):
    """Handle all other exceptions"""
    traceback.print_exc()
    logger.error("unhandled_exception", error=str(exc), path=request.url.path)
    return JSONResponse(
        status_code=500,
        content={
            "error": "Internal server error",
            "detail": repr(exc),
        },
    )


@app.get("/health")
async def health_check():
    """Health check endpoint"""
    logger.debug("Health check requested")

    return JSONResponse(
        status_code=200,
        content={
            "status": "ok",
            "service": "python-worker",
            "version": "0.1.0",
        },
    )


@app.get("/")
async def root():
    """Root endpoint"""
    return {
        "service": "Archivio Parlante Python AI Worker",
        "endpoints": [
            "GET /health",
            "POST /parse",
            "POST /rerank",
            "POST /contextualize",
            "POST /extract_kg",
            "POST /verify_hallucination",
        ],
    }


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(
        "main:app",
        host="0.0.0.0",
        port=8091,
        log_level="info",
    )
