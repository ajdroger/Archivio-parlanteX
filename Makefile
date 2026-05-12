# ============================================================================
# Archivio Parlante — Makefile
# ============================================================================
# Comandi operativi per gestione Docker Compose stack

.PHONY: help setup up down logs ps health rebuild-rust rebuild-python ollama-pull mysql-shell backup-db restore-db test-all test-e2e test-e2e-ui lint clean bench-all audit-security observability-up observability-down observability-logs load-test stress-test spike-test

# === Default target ===
help:
	@echo "🏛️  Archivio Parlante — Available Commands"
	@echo ""
	@echo "Setup & Lifecycle:"
	@echo "  make setup          - Initial setup (build images, install deps)"
	@echo "  make up             - Start all containers (docker compose up -d)"
	@echo "  make down           - Stop all containers"
	@echo "  make logs           - Follow logs from all containers"
	@echo "  make ps             - Show running containers status"
	@echo "  make health         - Check health of all services"
	@echo ""
	@echo "Build & Deploy:"
	@echo "  make rebuild-rust   - Rebuild only Rust engine"
	@echo "  make rebuild-python - Rebuild only Python worker"
	@echo "  make rebuild-all    - Rebuild all custom images"
	@echo ""
	@echo "Ollama:"
	@echo "  make ollama-pull    - Download default Ollama models"
	@echo "  make ollama-models  - List downloaded models"
	@echo ""
	@echo "Database:"
	@echo "  make mysql-shell    - Open MySQL shell"
	@echo "  make backup-db      - Backup MySQL database"
	@echo "  make restore-db     - Restore database from backup"
	@echo ""
	@echo "Testing & Quality:"
	@echo "  make test-all       - Run all unit tests (Rust + Python + PHP + Frontend)"
	@echo "  make test-e2e       - Run E2E tests with Playwright (requires stack UP)"
	@echo "  make test-e2e-ui    - Run E2E tests in UI mode (interactive)"
	@echo "  make lint           - Run all linters (clippy, ruff, phpstan, eslint)"
	@echo "  make bench-all      - Run full benchmark suite (~30 min)"
	@echo "  make audit-security - Run security audits (cargo audit, pip-audit, etc.)"
	@echo ""
	@echo "Load Testing:"
	@echo "  make load-test      - Run k6 load test (50 VU, 5 min)"
	@echo "  make stress-test    - Run k6 stress test (100→500 VU ramp)"
	@echo "  make spike-test     - Run k6 spike test (0→200 VU immediate)"
	@echo ""
	@echo "Observability:"
	@echo "  make observability-up    - Start Prometheus + Grafana stack"
	@echo "  make observability-down  - Stop observability stack"
	@echo "  make observability-logs  - Follow observability logs"
	@echo ""
	@echo "Cleanup:"
	@echo "  make clean          - Remove containers, volumes, and build artifacts"

# === Setup & Lifecycle ===
setup:
	@echo "📦 Setting up Archivio Parlante..."
	@cp -n .env.example .env || true
	@echo "⚠️  IMPORTANT: Edit .env and set JWT_SECRET and RUST_ENGINE_INTERNAL_TOKEN"
	@echo "   Generate with: openssl rand -hex 32"
	@docker compose build
	@echo "✅ Setup complete! Run 'make up' to start services."

up:
	@echo "🚀 Starting all containers..."
	@docker compose up -d
	@echo "✅ Stack is running. Check status with 'make ps' or 'make health'"

down:
	@echo "🛑 Stopping all containers..."
	@docker compose down
	@echo "✅ All containers stopped."

logs:
	@docker compose logs -f

ps:
	@docker compose ps

# === Health Checks ===
health:
	@echo "🔍 Checking service health..."
	@echo ""
	@echo "PHP Gateway:"
	@curl -s http://localhost:8080 || echo "❌ PHP Gateway not responding"
	@echo ""
	@echo "Rust Engine:"
	@curl -s http://localhost:8090/health | jq '.' || echo "❌ Rust Engine not responding"
	@echo ""
	@echo "Python Worker:"
	@curl -s http://localhost:8091/health | jq '.' || echo "❌ Python Worker not responding"
	@echo ""
	@echo "Qdrant:"
	@curl -s http://localhost:6333 || echo "❌ Qdrant not responding"
	@echo ""
	@echo "Ollama:"
	@curl -s http://localhost:11434/api/tags || echo "❌ Ollama not responding"
	@echo ""
	@echo "MySQL:"
	@docker compose exec mysql mysqladmin ping -h localhost || echo "❌ MySQL not responding"
	@echo ""
	@echo "Redis:"
	@docker compose exec redis redis-cli ping || echo "❌ Redis not responding"

# === Observability Stack ===
observability-up:
	@echo "📊 Starting Observability stack (Prometheus + Grafana)..."
	@docker compose -f docker-compose.observability.yml up -d
	@echo "✅ Observability stack running:"
	@echo "   Prometheus: http://localhost:9090"
	@echo "   Grafana:    http://localhost:3001 (user: admin, pass: admin)"

observability-down:
	@echo "🛑 Stopping Observability stack..."
	@docker compose -f docker-compose.observability.yml down
	@echo "✅ Observability stack stopped."

observability-logs:
	@docker compose -f docker-compose.observability.yml logs -f

# === Load Testing with k6 ===
load-test:
	@echo "🚀 Running k6 load test (50 VU, 5 min)..."
	@k6 run benchmarks/k6/load_test.js

stress-test:
	@echo "💪 Running k6 stress test (100→500 VU ramp)..."
	@k6 run benchmarks/k6/stress_test.js

spike-test:
	@echo "⚡ Running k6 spike test (0→200 VU immediate)..."
	@k6 run benchmarks/k6/spike_test.js

# === Build & Deploy ===
rebuild-rust:
	@echo "🔨 Rebuilding Rust engine..."
	@docker compose build rust-engine
	@docker compose up -d rust-engine
	@echo "✅ Rust engine rebuilt and restarted."

rebuild-python:
	@echo "🔨 Rebuilding Python worker..."
	@docker compose build python-worker
	@docker compose up -d python-worker
	@echo "✅ Python worker rebuilt and restarted."

rebuild-all:
	@echo "🔨 Rebuilding all images..."
	@docker compose build
	@docker compose up -d
	@echo "✅ All images rebuilt and containers restarted."

# === Ollama ===
ollama-pull:
	@echo "📥 Downloading default Ollama models..."
	@echo "This may take 10-15 minutes depending on connection speed."
	@docker compose exec ollama ollama pull qwen2.5:7b-instruct-q4_K_M
	@docker compose exec ollama ollama pull qwen2.5:3b-instruct-q4_K_M
	@docker compose exec ollama ollama pull nomic-embed-text
	@echo "✅ Default models downloaded."

ollama-models:
	@echo "📋 Downloaded Ollama models:"
	@docker compose exec ollama ollama list

# === Database ===
mysql-shell:
	@docker compose exec mysql mysql -u root archivio_parlante_x

backup-db:
	@echo "💾 Backing up database..."
	@mkdir -p backups
	@docker compose exec mysql mysqldump -u root archivio_parlante_x | gzip > backups/db_$(shell date +%Y%m%d_%H%M%S).sql.gz
	@echo "✅ Backup saved to backups/db_*.sql.gz"

restore-db:
	@echo "⚠️  This will restore database from latest backup."
	@echo "Press Ctrl+C to cancel, or Enter to continue..."
	@read
	@gunzip < $(shell ls -t backups/*.sql.gz | head -1) | docker compose exec -T mysql mysql -u root archivio_parlante_x
	@echo "✅ Database restored."

# === Testing (Fase 5) ===
test-rust:
	@echo "🧪 Running Rust tests..."
	@cd engine-rust && cargo test --release

test-python:
	@echo "🧪 Running Python tests..."
	@cd engine-python && pytest --cov=app --cov-report=term

test-php:
	@echo "🧪 Running PHP tests..."
	@cd php-gateway && composer test

test-frontend:
	@echo "🧪 Running Frontend tests..."
	@cd frontend && npm run test:run

test-e2e:
	@echo "🧪 Running E2E tests (requires stack UP)..."
	@echo "⚠️  Make sure all services are running: make up"
	@cd frontend && npm run test:e2e

test-e2e-ui:
	@echo "🧪 Running E2E tests in UI mode..."
	@cd frontend && npm run test:e2e:ui

test-all: test-rust test-python test-php test-frontend
	@echo "✅ All unit tests completed."
	@echo "💡 Run 'make test-e2e' for end-to-end tests (requires stack UP)"

# === Linting (Fase 5) ===
lint-rust:
	@cd engine-rust && cargo fmt --check && cargo clippy --all-targets -- -D warnings

lint-python:
	@cd engine-python && ruff format --check . && ruff check . && mypy app --strict

lint-php:
	@cd php-gateway && composer run phpstan

lint-frontend:
	@cd frontend && npm run lint && npm run type-check

lint: lint-rust lint-python lint-php lint-frontend
	@echo "✅ All linters passed."

# === Cleanup ===
clean:
	@echo "🧹 Cleaning up..."
	@docker compose down -v
	@docker system prune -f
	@cd engine-rust && cargo clean
	@echo "✅ Cleanup complete."

# === Benchmarking (Fase 5) ===
bench-setup:
	@echo "🔧 Setting up benchmark environment..."
	@cd benchmarks && pip install -r requirements.txt
	@cd benchmarks/fixtures && python generate_contracts.py --count 50
	@echo "✅ Benchmark setup complete (50 PDF fixtures generated)"

bench-ingest:
	@echo "📊 Running ingest benchmark..."
	@cd benchmarks && python ingest_bench.py

bench-query:
	@echo "📊 Running query benchmark..."
	@cd benchmarks && python query_bench.py

bench-hallucination:
	@echo "📊 Running hallucination evaluation..."
	@cd benchmarks && python hallucination_eval.py

bench-concurrent:
	@echo "📊 Running concurrent benchmark..."
	@cd benchmarks && python concurrent_bench.py

bench-all: bench-setup
	@echo "🚀 Running full benchmark suite..."
	@echo "Estimated time: ~30 minutes"
	@echo ""
	@make bench-ingest
	@make bench-query
	@make bench-hallucination
	@make bench-concurrent
	@echo ""
	@echo "✅ All benchmarks complete! Reports in benchmarks/reports/"
	@echo "📄 Summary: benchmarks/reports/benchmark_summary.html"

# === Security Audits ===
audit-security:
	@echo "🔒 Running security audits..."
	@cd engine-rust && cargo audit
	@cd engine-python && pip-audit
	@cd php-gateway && composer audit
	@echo "✅ Security audit complete"

