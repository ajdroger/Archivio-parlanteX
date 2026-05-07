#!/bin/bash
# Archivio Parlante - Fase 5 Verification Script

set -e

echo "🔍 Fase 5 Verification Suite"
echo "=============================="
echo ""

# Check if stack is running
echo "📋 Prerequisites check..."
if ! docker ps | grep -q "archivio-rust-engine"; then
    echo "⚠️  WARNING: Docker stack not running. Some tests will be skipped."
    echo "   Run 'make up' first to enable E2E and load tests."
    STACK_RUNNING=false
else
    echo "✅ Docker stack detected"
    STACK_RUNNING=true
fi
echo ""

# 1. Security Audits (fast, no Docker needed)
echo "🔒 Step 1/5: Security Audits"
echo "----------------------------"
make audit-security || { echo "❌ Security audit failed"; exit 1; }
echo "✅ Security audits passed"
echo ""

# 2. Lint (fast, no Docker needed)
echo "🧹 Step 2/5: Linting"
echo "----------------------------"
make lint || { echo "⚠️  Linting found issues"; }
echo "✅ Linting completed"
echo ""

# 3. Unit Tests (requires dependencies)
echo "🧪 Step 3/5: Unit Tests"
echo "----------------------------"
if [ "$STACK_RUNNING" = true ]; then
    make test-all || { echo "❌ Unit tests failed"; exit 1; }
    echo "✅ Unit tests passed"
else
    echo "⏭️  SKIPPED (requires running stack)"
fi
echo ""

# 4. E2E Tests (requires stack UP)
echo "🎭 Step 4/5: E2E Tests"
echo "----------------------------"
if [ "$STACK_RUNNING" = true ]; then
    make test-e2e || { echo "❌ E2E tests failed"; exit 1; }
    echo "✅ E2E tests passed"
else
    echo "⏭️  SKIPPED (requires running stack)"
fi
echo ""

# 5. Benchmarks (long running)
echo "⚡ Step 5/5: Benchmarks"
echo "----------------------------"
if [ "$STACK_RUNNING" = true ]; then
    echo "⏳ Running benchmarks (this may take 30+ minutes)..."
    make bench-all || { echo "❌ Benchmarks failed"; exit 1; }
    echo "✅ Benchmarks passed"
else
    echo "⏭️  SKIPPED (requires running stack)"
fi
echo ""

# Summary
echo "================================"
echo "✅ Fase 5 Verification Complete!"
echo "================================"
echo ""
echo "To run skipped tests:"
echo "  1. make up"
echo "  2. ./verify-phase5.sh"
echo ""
echo "Optional load testing:"
echo "  make load-test    # 50 VU, 5 min"
echo "  make stress-test  # 100→500 VU ramp"
echo "  make spike-test   # 0→200 VU spike"
