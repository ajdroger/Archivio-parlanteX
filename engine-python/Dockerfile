# ============================================================================
# Archivio Parlante — Python AI Worker Dockerfile (MINIMAL BUILD)
# ============================================================================
# NOTE: Heavy ML dependencies (torch, FlagEmbedding, unstructured) are SKIPPED
#       due to Docker/WSL2 segfault issues during pip install.
#       TODO: Install at runtime or use pre-built base image.

FROM python:3.9-slim

WORKDIR /app

# Install minimal system dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Install ONLY core dependencies (no ML/OCR for now)
RUN pip install --no-cache-dir \
    fastapi==0.115.0 \
    uvicorn==0.32.0 \
    pydantic==2.9.2 \
    pydantic-settings==2.5.0 \
    httpx==0.28.0 \
    python-multipart==0.0.12 \
    structlog==24.4.0 \
    python-dotenv==1.0.1

# Copy application code
COPY app /app/app

# Expose port
EXPOSE 8091

# Run as non-root user
RUN useradd -m -u 1000 pyuser && \
    chown -R pyuser:pyuser /app
USER pyuser

# Start FastAPI server
CMD ["uvicorn", "app.main:app", "--host", "0.0.0.0", "--port", "8091"]
