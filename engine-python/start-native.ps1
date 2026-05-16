# ============================================================================
# Archivio Parlante — Python Worker Native Startup (PowerShell)
# ============================================================================

$ErrorActionPreference = "Stop"

Write-Host "🐍 Starting Python AI Worker (Native Mode)..." -ForegroundColor Cyan

# Navigate to script directory
Set-Location $PSScriptRoot

# Check Python
try {
    $pythonVersion = python --version 2>&1
    Write-Host "✅ Python version: $pythonVersion" -ForegroundColor Green
} catch {
    Write-Host "❌ Python not found. Install it first." -ForegroundColor Red
    exit 1
}

# Create virtual environment if not exists
if (-not (Test-Path "venv")) {
    Write-Host "📦 Creating virtual environment..." -ForegroundColor Yellow
    python -m venv venv
}

# Activate virtual environment
Write-Host "🔌 Activating virtual environment..." -ForegroundColor Cyan
& "venv\Scripts\Activate.ps1"

# Install dependencies
Write-Host "📥 Installing dependencies..." -ForegroundColor Cyan
python -m pip install --upgrade pip -q
pip install -r requirements-minimal.txt -q

# Load environment variables from .env
$envFile = "..\\.env"
if (Test-Path $envFile) {
    Get-Content $envFile | Where-Object { $_ -notmatch '^#' -and $_ -match '=' } | ForEach-Object {
        $parts = $_ -split '=', 2
        [Environment]::SetEnvironmentVariable($parts[0].Trim(), $parts[1].Trim(), 'Process')
    }
    Write-Host "✅ Environment variables loaded from .env" -ForegroundColor Green
} else {
    Write-Host "⚠️  No .env file found, using defaults" -ForegroundColor Yellow
}

# Set defaults if not in .env
if (-not $env:PYTHON_LOG_LEVEL) { $env:PYTHON_LOG_LEVEL = "info" }
if (-not $env:OLLAMA_URL) { $env:OLLAMA_URL = "http://localhost:11434" }
if (-not $env:MYSQL_HOST) { $env:MYSQL_HOST = "localhost" }
if (-not $env:MYSQL_DB) { $env:MYSQL_DB = "archivio_parlante_x" }
if (-not $env:MYSQL_USER) { $env:MYSQL_USER = "root" }

Write-Host ""
Write-Host "🚀 Starting FastAPI server on http://0.0.0.0:8091" -ForegroundColor Green
Write-Host "📊 Logs: structlog JSON format" -ForegroundColor Cyan
Write-Host "🛑 Stop with Ctrl+C" -ForegroundColor Yellow
Write-Host ""

# Start server
uvicorn app.main:app --host 0.0.0.0 --port 8091 --reload
