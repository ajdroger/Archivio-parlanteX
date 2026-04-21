# ============================================================================
# Archivio Parlante — Python AI Worker Dockerfile
# ============================================================================

FROM python:3.11-slim

WORKDIR /app

# Install system dependencies (for PDF/OCR in Fase 2)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy requirements
COPY requirements.txt .

# Install Python dependencies
RUN pip install --no-cache-dir --upgrade pip && \
    pip install --no-cache-dir -r requirements.txt

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
