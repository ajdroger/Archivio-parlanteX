# ============================================================================
# Archivio Parlante — Python AI Worker Dockerfile (FULL BUILD)
# ============================================================================
# Includes ALL ML dependencies (torch 2.5.1, FlagEmbedding, unstructured)

FROM python:3.10

WORKDIR /app

# Install system dependencies for ML libs
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    libgomp1 && \
    rm -rf /var/lib/apt/lists/*

# Upgrade pip and setuptools to latest versions
RUN python3 -m pip install --upgrade pip setuptools wheel

# Install ALL dependencies including ML/OCR/Reranker
COPY requirements.txt .
RUN python3 -m pip install --no-cache-dir -r requirements.txt

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
