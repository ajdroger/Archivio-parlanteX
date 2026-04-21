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

from app.routers import parse

# Configure structured logging
structlog.configure(
    processors=[
        structlog.processors.TimeStamper(fmt="iso"),
        structlog.processors.add_log_level,
        structlog.processors.JSONRenderer(),
    ]
)

logger = structlog.get_logger()


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Lifespan context manager for startup/shutdown"""
    logger.info("🐍 Python AI Worker starting...")
    # TODO Fase 2: Initialize ML models (BGE reranker, spaCy NER, etc.)
    yield
    logger.info("Python AI Worker shutting down...")


# Create FastAPI app
app = FastAPI(
    title="Archivio Parlante Python AI Worker",
    description="PDF parsing, OCR, reranking, and knowledge graph extraction",
    version="0.1.0",
    lifespan=lifespan,
)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # TODO: restrict in production
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Include routers
app.include_router(parse.router)


# Exception handlers
@app.exception_handler(RequestValidationError)
async def validation_exception_handler(request: Request, exc: RequestValidationError):
    """Handle validation errors"""
    logger.warning("validation_error", errors=exc.errors(), body=exc.body)
    return JSONResponse(
        status_code=400,
        content={
            "error": "Validation error",
            "detail": exc.errors(),
        },
    )


@app.exception_handler(Exception)
async def global_exception_handler(request: Request, exc: Exception):
    """Handle all other exceptions"""
    logger.error("unhandled_exception", error=str(exc), path=request.url.path)
    return JSONResponse(
        status_code=500,
        content={
            "error": "Internal server error",
            "detail": str(exc),
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
            "POST /rerank (TODO: Fase 2.2)",
            "POST /contextualize (TODO: Fase 2.3)",
            "POST /extract_kg (TODO: Fase 2.4)",
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
