#!/usr/bin/env bash
#
# Integration Test Suite for Fase 6
#
# Orchestrates full integration testing workflow:
# 1. Health checks
# 2. Test KB creation + fixture ingestion
# 3. Graph RAG benchmark
# 4. Hallucination detection evaluation
# 5. WebSocket load test (optional)
# 6. Results verification
#
# Usage:
#   ./scripts/run_integration_tests.sh [--skip-websocket]
#
# Requirements:
#   - All services running (docker compose up -d)
#   - Python 3.11+ with httpx, rich installed
#   - k6 installed (for WebSocket test)

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
ENGINE_URL="${ENGINE_URL:-http://localhost:8090}"
PHP_GATEWAY_URL="${PHP_GATEWAY_URL:-http://localhost:9080}"
PYTHON_WORKER_URL="${PYTHON_WORKER_URL:-http://localhost:8091}"
TEST_KB_ID="fase6_test_kb_$(date +%s)"
RESULTS_DIR="benchmarks/results/integration_$(date +%Y%m%d_%H%M%S)"
SKIP_WEBSOCKET=false
SKIP_PHP_GATEWAY=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --skip-websocket)
            SKIP_WEBSOCKET=true
            shift
            ;;
        --skip-php-gateway)
            SKIP_PHP_GATEWAY=true
            shift
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Helper functions
print_header() {
    echo -e "\n${CYAN}================================================${NC}"
    echo -e "${CYAN}$1${NC}"
    echo -e "${CYAN}================================================${NC}\n"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

check_service() {
    local name=$1
    local url=$2
    echo -n "Checking $name... "

    if curl -sf "$url/health" > /dev/null 2>&1; then
        print_success "$name is healthy"
        return 0
    else
        print_error "$name is not responding at $url"
        return 1
    fi
}

# Main execution
main() {
    print_header "Fase 6 Integration Test Suite"

    echo "Configuration:"
    echo "  Engine URL: $ENGINE_URL"
    echo "  PHP Gateway: $PHP_GATEWAY_URL"
    echo "  Python Worker: $PYTHON_WORKER_URL"
    echo "  Test KB ID: $TEST_KB_ID"
    echo "  Results Directory: $RESULTS_DIR"
    echo ""

    # Create results directory
    mkdir -p "$RESULTS_DIR"

    # Step 1: Health checks
    print_header "Step 1: Service Health Checks"

    check_service "Rust Engine" "$ENGINE_URL" || exit 1

    if [[ "$SKIP_PHP_GATEWAY" == "false" ]]; then
        check_service "PHP Gateway" "$PHP_GATEWAY_URL" || exit 1
    else
        print_warning "Skipping PHP Gateway check (--skip-php-gateway)"
    fi

    check_service "Python Worker" "$PYTHON_WORKER_URL" || {
        print_warning "Python Worker not responding - hallucination detection may fail"
        print_warning "Start manually: cd engine-python && uvicorn app.main:app --port 8091"
    }

    # Step 2: Ingest test fixtures
    print_header "Step 2: Ingest Test Fixtures"

    if python3 scripts/ingest_test_fixtures.py \
        --engine-url "$ENGINE_URL" \
        --kb-id "$TEST_KB_ID" \
        --contracts-dir tests/fixtures/contracts; then
        print_success "Test fixtures ingested successfully"
    else
        print_error "Failed to ingest test fixtures"
        exit 1
    fi

    # Wait for indexing to complete
    echo ""
    echo "Waiting 10 seconds for indexing to complete..."
    sleep 10

    # Step 3: Graph RAG benchmark
    print_header "Step 3: Graph RAG Benchmark"

    if python3 benchmarks/graph_rag_bench.py \
        --engine-url "$ENGINE_URL" \
        --kb-id "$TEST_KB_ID" \
        --queries-file tests/fixtures/queries/graph_rag_test_queries.json \
        --output "$RESULTS_DIR/graph_rag.json"; then

        # Check if benchmark passed
        if jq -e '.summary.passed == true' "$RESULTS_DIR/graph_rag.json" > /dev/null 2>&1; then
            print_success "Graph RAG benchmark PASSED"

            # Extract key metrics
            recall_improvement=$(jq -r '.summary.recall_improvement_pct' "$RESULTS_DIR/graph_rag.json")
            latency_p95=$(jq -r '.summary.graph_latency_p95_ms' "$RESULTS_DIR/graph_rag.json")
            echo "  Recall improvement: ${recall_improvement}%"
            echo "  Latency P95: ${latency_p95}ms"
        else
            print_error "Graph RAG benchmark FAILED"
            jq '.summary' "$RESULTS_DIR/graph_rag.json"
            exit 1
        fi
    else
        print_error "Graph RAG benchmark encountered an error"
        exit 1
    fi

    # Step 4: Hallucination detection evaluation
    print_header "Step 4: Hallucination Detection Evaluation"

    # Check if Python worker is responding
    if curl -sf "$PYTHON_WORKER_URL/health" > /dev/null 2>&1; then
        if python3 benchmarks/hallucination_eval.py \
            --engine-url "$ENGINE_URL" \
            --kb-id "$TEST_KB_ID" \
            --trick-questions tests/fixtures/trick_questions/hallucination_test_questions.json \
            --valid-queries tests/fixtures/queries/graph_rag_test_queries.json \
            --output "$RESULTS_DIR/hallucination.json"; then

            # Check if evaluation passed
            if jq -e '.summary.passed == true' "$RESULTS_DIR/hallucination.json" > /dev/null 2>&1; then
                print_success "Hallucination evaluation PASSED"

                # Extract key metrics
                hallucination_rate=$(jq -r '.summary.trick_questions.hallucination_rate_pct' "$RESULTS_DIR/hallucination.json")
                specificity=$(jq -r '.summary.valid_queries.specificity_pct // "N/A"' "$RESULTS_DIR/hallucination.json")
                echo "  Hallucination rate: ${hallucination_rate}%"
                echo "  Specificity: ${specificity}%"
            else
                print_error "Hallucination evaluation FAILED"
                jq '.summary' "$RESULTS_DIR/hallucination.json"
                exit 1
            fi
        else
            print_error "Hallucination evaluation encountered an error"
            exit 1
        fi
    else
        print_warning "Skipping hallucination evaluation (Python worker not available)"
        print_warning "Start worker: cd engine-python && uvicorn app.main:app --port 8091"
    fi

    # Step 5: WebSocket load test (optional)
    if [ "$SKIP_WEBSOCKET" = false ]; then
        print_header "Step 5: WebSocket Load Test"

        if command -v k6 &> /dev/null; then
            if [ -f "benchmarks/k6/websocket_load.js" ]; then
                if k6 run \
                    --out json="$RESULTS_DIR/k6_websocket.json" \
                    benchmarks/k6/websocket_load.js; then
                    print_success "WebSocket load test PASSED"
                else
                    print_error "WebSocket load test FAILED"
                    exit 1
                fi
            else
                print_warning "WebSocket load test script not found: benchmarks/k6/websocket_load.js"
            fi
        else
            print_warning "k6 not installed, skipping WebSocket load test"
            print_warning "Install: https://k6.io/docs/getting-started/installation/"
        fi
    else
        print_header "Step 5: WebSocket Load Test (Skipped)"
        print_warning "WebSocket load test skipped (--skip-websocket flag)"
    fi

    # Final summary
    print_header "Integration Test Results"

    echo "Test results saved to: $RESULTS_DIR"
    echo ""

    # Generate summary report
    SUMMARY_FILE="$RESULTS_DIR/SUMMARY.md"
    cat > "$SUMMARY_FILE" <<EOF
# Fase 6 Integration Test Results

**Date**: $(date '+%Y-%m-%d %H:%M:%S')
**Test KB ID**: \`$TEST_KB_ID\`
**Engine URL**: $ENGINE_URL

---

## Test Results

### Graph RAG Benchmark

EOF

    if [ -f "$RESULTS_DIR/graph_rag.json" ]; then
        jq -r '
            "- **Recall Improvement**: " + (.summary.recall_improvement_pct | tostring) + "%",
            "- **Latency P95**: " + (.summary.graph_latency_p95_ms | tostring) + " ms",
            "- **Failures**: " + (.summary.graph_failures | tostring) + " / " + (.summary.total_queries | tostring),
            "- **Status**: " + (if .summary.passed then "✅ PASSED" else "❌ FAILED" end)
        ' "$RESULTS_DIR/graph_rag.json" >> "$SUMMARY_FILE"
    fi

    cat >> "$SUMMARY_FILE" <<EOF

### Hallucination Detection Evaluation

EOF

    if [ -f "$RESULTS_DIR/hallucination.json" ]; then
        jq -r '
            "- **Hallucination Rate**: " + (.summary.trick_questions.hallucination_rate_pct | tostring) + "%",
            "- **Correct Refusals**: " + (.summary.trick_questions.correct_refusals | tostring) + " / " + (.summary.trick_questions.total | tostring),
            (if .summary.valid_queries then
                "- **Specificity**: " + (.summary.valid_queries.specificity_pct | tostring) + "%",
                "- **False Positive Rate**: " + (.summary.valid_queries.false_positive_rate_pct | tostring) + "%",
                "- **Latency P95**: " + (.summary.valid_queries.latency_p95_ms | tostring) + " ms"
            else empty end),
            "- **Status**: " + (if .summary.passed then "✅ PASSED" else "❌ FAILED" end)
        ' "$RESULTS_DIR/hallucination.json" >> "$SUMMARY_FILE"
    fi

    if [ "$SKIP_WEBSOCKET" = false ] && [ -f "$RESULTS_DIR/k6_websocket.json" ]; then
        cat >> "$SUMMARY_FILE" <<EOF

### WebSocket Load Test

- **Status**: ✅ PASSED (see k6_websocket.json for details)

EOF
    fi

    cat >> "$SUMMARY_FILE" <<EOF

---

## Files Generated

- \`graph_rag.json\` - Graph RAG benchmark detailed results
- \`hallucination.json\` - Hallucination evaluation detailed results
- \`k6_websocket.json\` - WebSocket load test results (if run)
- \`SUMMARY.md\` - This summary report

## Next Steps

1. Review detailed results in JSON files
2. If tests failed, check logs in \`docker compose logs\`
3. Clean up test KB: (optional, will be reused if re-running tests)
EOF

    # Display summary
    cat "$SUMMARY_FILE"

    print_success "All integration tests completed!"
    print_success "Summary saved to: $SUMMARY_FILE"

    return 0
}

# Run main function
main
