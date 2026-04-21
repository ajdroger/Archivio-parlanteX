"""
Archivio Parlante — Python AI Worker
FastAPI server for PDF parsing, reranking, and knowledge graph extraction
"""

import logging
from fastapi import FastAPI
from fastapi.responses import JSONResponse
import structlog

# Configure structured logging
structlog.configure(
    processors=[
        structlog.processors.TimeStamper(fmt="iso"),
        structlog.processors.add_log_level,
        structlog.processors.JSONRenderer(),
    ]
)

logger = structlog.get_logger()

# Create FastAPI app
app = FastAPI(
    title="Archivio Parlante Python AI Worker",
    description="PDF parsing, OCR, reranking, and knowledge graph extraction",
    version="0.1.0",
)


@app.on_event("startup")
async def startup_event():
    """Initialize services on startup"""
    logger.info("🐍 Python AI Worker starting...")
    # TODO Fase 2: Initialize ML models (BGE reranker, spaCy NER, etc.)


@app.on_event("shutdown")
async def shutdown_event():
    """Cleanup on shutdown"""
    logger.info("Python AI Worker shutting down...")


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
            "POST /parse (Fase 2)",
            "POST /rerank (Fase 2)",
            "POST /contextualize (Fase 2)",
            "POST /extract_kg (Fase 2)",
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
