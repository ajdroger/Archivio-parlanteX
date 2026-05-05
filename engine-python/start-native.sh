#!/bin/bash
# ============================================================================
# Archivio Parlante — Python Worker Native Startup (Bash/WSL)
# ============================================================================

set -e

echo "🐍 Starting Python AI Worker (Native Mode)..."

# Navigate to script directory
cd "$(dirname "$0")"

# Check Python
if ! command -v python3 &> /dev/null; then
    echo "❌ Python 3 not found. Install it first."
    exit 1
fi

echo "✅ Python version: $(python3 --version)"

# Create virtual environment if not exists
if [ ! -d "venv" ]; then
    echo "📦 Creating virtual environment..."
    python3 -m venv venv
fi

# Activate virtual environment
echo "🔌 Activating virtual environment..."
source venv/bin/activate

# Install dependencies
echo "📥 Installing dependencies..."
pip install --upgrade pip -q
pip install -r requirements-minimal.txt -q

# Load environment variables
if [ -f "../.env" ]; then
    export $(cat ../.env | grep -v '^#' | xargs)
    echo "✅ Environment variables loaded from .env"
else
    echo "⚠️  No .env file found, using defaults"
fi

# Set defaults if not in .env
export PYTHON_LOG_LEVEL=${PYTHON_LOG_LEVEL:-info}
export OLLAMA_URL=${OLLAMA_URL:-http://localhost:11434}
export MYSQL_HOST=${MYSQL_HOST:-localhost}
export MYSQL_DB=${MYSQL_DB:-archivio_parlante_x}
export MYSQL_USER=${MYSQL_USER:-root}

echo ""
echo "🚀 Starting FastAPI server on http://0.0.0.0:8091"
echo "📊 Logs: structlog JSON format"
echo "🛑 Stop with Ctrl+C"
echo ""

# Start server
uvicorn app.main:app --host 0.0.0.0 --port 8091 --reload
