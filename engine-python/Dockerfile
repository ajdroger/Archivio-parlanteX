# ============================================================================
# Archivio Parlante — Python AI Worker Dockerfile (MINIMAL BUILD)
# ============================================================================
# NOTE: Heavy ML dependencies (torch, FlagEmbedding, unstructured) are SKIPPED
#       due to Docker/WSL2 segfault issues during pip install.
#       TODO: Install at runtime or use pre-built base image.

FROM python:3.10

WORKDIR /app

# Install minimal system dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Upgrade pip to avoid TypeError in older pip versions
RUN python3 -m pip install --upgrade pip

# Install ONLY core dependencies (no ML/OCR for now)
COPY requirements-minimal.txt .
RUN python3 -m pip install --no-cache-dir -r requirements-minimal.txt

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
