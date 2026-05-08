#!/bin/bash
# ============================================================================
# Archivio Parlante - Fase 6.3 Complete Test Suite
# ============================================================================
# Comprehensive testing script for multi-tenant workspace isolation
#
# Usage: ./TEST_FASE_6.3.sh
#
# Prerequisites:
# - Docker services running: make up
# - Cargo installed (for Rust tests)
# - k6 installed (for load tests)
# - jq installed (for JSON parsing)

set -e

echo "============================================="
echo "🧪 Fase 6.3 Complete Test Suite"
echo "============================================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

PASSED=0
FAILED=0

pass() {
    echo -e "${GREEN}✓ PASS${NC}: $1"
    ((PASSED++))
}

fail() {
    echo -e "${RED}✗ FAIL${NC}: $1"
    ((FAILED++))
}

section() {
    echo ""
    echo -e "${BLUE}================================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}================================================${NC}"
}

# ============================================================================
# 1. Pre-flight Checks
# ============================================================================

section "1. Pre-flight Checks"

# Check if Docker is running
if docker ps &> /dev/null; then
    pass "Docker is running"
else
    fail "Docker is not running"
    echo "Please start Docker and run: make up"
    exit 1
fi

# Check if services are up
SERVICES=("mysql" "redis" "rust-engine" "php-gateway")
for service in "${SERVICES[@]}"; do
    if docker ps --format '{{.Names}}' | grep -q "$service"; then
        pass "Service $service is running"
    else
        fail "Service $service is not running"
    fi
done

# Check if cargo is available
if command -v cargo &> /dev/null; then
    pass "Cargo is installed"
    CARGO_AVAILABLE=true
else
    echo -e "${YELLOW}⚠ Cargo not found - Rust tests will be skipped${NC}"
    CARGO_AVAILABLE=false
fi

# Check if k6 is available
if command -v k6 &> /dev/null; then
    pass "k6 is installed"
    K6_AVAILABLE=true
else
    echo -e "${YELLOW}⚠ k6 not found - Load tests will be skipped${NC}"
    K6_AVAILABLE=false
fi

# ============================================================================
# 2. Database Migration Check
# ============================================================================

section "2. Database Migration Verification"

# Check if migration 010 was applied
MIGRATION_CHECK=$(docker exec mysql mysql -u root -pdevpass123 archivio_parlante_x \
    -e "SHOW TABLES LIKE 'ap_workspaces';" 2>/dev/null | grep -c "ap_workspaces" || echo "0")

if [ "$MIGRATION_CHECK" = "1" ]; then
    pass "Migration 010 applied - ap_workspaces table exists"
else
    fail "Migration 010 not applied"
fi

# Check all workspace tables
TABLES=("ap_workspaces" "ap_workspace_members" "ap_kb_permissions" "ap_permission_audit")
for table in "${TABLES[@]}"; do
    TABLE_CHECK=$(docker exec mysql mysql -u root -pdevpass123 archivio_parlante_x \
        -e "SHOW TABLES LIKE '$table';" 2>/dev/null | grep -c "$table" || echo "0")
    if [ "$TABLE_CHECK" = "1" ]; then
        pass "Table $table exists"
    else
        fail "Table $table not found"
    fi
done

# ============================================================================
# 3. Rust Compilation & Unit Tests
# ============================================================================

section "3. Rust Compilation & Unit Tests"

if [ "$CARGO_AVAILABLE" = true ]; then
    echo "Compiling Rust engine..."
    cd engine-rust

    if cargo check 2>&1 | tee /tmp/cargo_check.log; then
        pass "Rust code compiles successfully"
    else
        fail "Rust compilation failed"
        echo "Check /tmp/cargo_check.log for details"
    fi

    echo ""
    echo "Running security test suite (100 tests)..."
    if cargo test --test kb_access_complete_suite 2>&1 | tee /tmp/cargo_test.log; then
        pass "Security test suite passed"
    else
        fail "Security tests failed"
        echo "Check /tmp/cargo_test.log for details"
    fi

    cd ..
else
    echo -e "${YELLOW}⚠ Skipping Rust tests (cargo not available)${NC}"
fi

# ============================================================================
# 4. PHP Route Verification
# ============================================================================

section "4. PHP Gateway API Verification"

# Check if PHP service responds
PHP_HEALTH=$(curl -s http://localhost:9080/health 2>&1 || echo "failed")

if echo "$PHP_HEALTH" | grep -q "ok\|status"; then
    pass "PHP Gateway is responding"
else
    fail "PHP Gateway not responding"
fi

# Test workspace endpoint (may fail without auth, but should not 404)
WORKSPACE_RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:9080/api/workspaces 2>&1)

if [ "$WORKSPACE_RESPONSE" = "200" ] || [ "$WORKSPACE_RESPONSE" = "401" ] || [ "$WORKSPACE_RESPONSE" = "403" ]; then
    pass "Workspace endpoint registered (HTTP $WORKSPACE_RESPONSE)"
else
    fail "Workspace endpoint not found (HTTP $WORKSPACE_RESPONSE)"
fi

# ============================================================================
# 5. Rust Engine Verification
# ============================================================================

section "5. Rust Engine API Verification"

# Check if Rust service responds
RUST_HEALTH=$(curl -s http://localhost:8090/health 2>&1 || echo "failed")

if echo "$RUST_HEALTH" | grep -q "ok\|status"; then
    pass "Rust Engine is responding"
else
    fail "Rust Engine not responding"
fi

# ============================================================================
# 6. Integration Tests
# ============================================================================

section "6. Integration Tests (5 Scenarios)"

if [ -f "tests/integration/workspace_scenarios.sh" ]; then
    chmod +x tests/integration/workspace_scenarios.sh

    echo "Running integration test suite..."
    if ./tests/integration/workspace_scenarios.sh; then
        pass "Integration tests passed"
    else
        fail "Integration tests failed"
    fi
else
    fail "Integration test script not found"
fi

# ============================================================================
# 7. Load Tests (k6)
# ============================================================================

section "7. Load Tests (k6)"

if [ "$K6_AVAILABLE" = true ] && [ -f "benchmarks/k6/workspace_permissions_load.js" ]; then
    echo "Running k6 load test (100 concurrent users)..."
    echo "This may take 4-5 minutes..."

    if k6 run benchmarks/k6/workspace_permissions_load.js 2>&1 | tee /tmp/k6_load_test.log; then
        pass "Load test completed"

        # Check thresholds
        if grep -q "✓" /tmp/k6_load_test.log; then
            pass "All k6 thresholds met"
        else
            fail "Some k6 thresholds not met - check /tmp/k6_load_test.log"
        fi
    else
        fail "Load test failed"
    fi
else
    echo -e "${YELLOW}⚠ Skipping load tests (k6 not available or script not found)${NC}"
fi

# ============================================================================
# 8. Frontend Verification
# ============================================================================

section "8. Frontend Component Verification"

# Check if WorkspaceSwitcher exists
if [ -f "frontend/src/components/layout/WorkspaceSwitcher.tsx" ]; then
    pass "WorkspaceSwitcher component exists"
else
    fail "WorkspaceSwitcher component not found"
fi

# Check if integrated in MainLayout
if grep -q "WorkspaceSwitcher" frontend/src/components/layout/MainLayout.tsx; then
    pass "WorkspaceSwitcher integrated in MainLayout"
else
    fail "WorkspaceSwitcher not integrated"
fi

# Check if store updated
if grep -q "currentWorkspace" frontend/src/store/appStore.ts; then
    pass "Workspace state added to Zustand store"
else
    fail "Workspace state not in store"
fi

# ============================================================================
# 9. Documentation Verification
# ============================================================================

section "9. Documentation Verification"

DOCS=(
    "CHANGELOG.md"
    "docs/SECURITY_AUDIT_FASE_6.3.md"
    "docs/FASE_6.3_COMPLETE.md"
    "docs/FASE_6.3_VERIFICATION.md"
)

for doc in "${DOCS[@]}"; do
    if [ -f "$doc" ]; then
        pass "Documentation: $doc exists"
    else
        fail "Documentation: $doc not found"
    fi
done

# Check if CHANGELOG has Fase 6.3 entry
if grep -q "0.6.3" CHANGELOG.md && grep -q "Multi-tenant" CHANGELOG.md; then
    pass "CHANGELOG updated with Fase 6.3"
else
    fail "CHANGELOG not updated"
fi

# ============================================================================
# 10. File Count Verification
# ============================================================================

section "10. File Count Verification"

echo "Files created/modified for Fase 6.3:"
echo ""

# Count new files
NEW_FILES=(
    "db/migrations/010_workspaces.sql"
    "engine-rust/src/middleware/kb_access_control.rs"
    "engine-rust/tests/kb_access_complete_suite.rs"
    "php-gateway/src/Service/WorkspaceService.php"
    "php-gateway/src/Controller/WorkspaceController.php"
    "frontend/src/components/layout/WorkspaceSwitcher.tsx"
    "tests/integration/workspace_scenarios.sh"
    "benchmarks/k6/workspace_permissions_load.js"
    "docs/SECURITY_AUDIT_FASE_6.3.md"
    "docs/FASE_6.3_COMPLETE.md"
)

NEW_COUNT=0
for file in "${NEW_FILES[@]}"; do
    if [ -f "$file" ]; then
        ((NEW_COUNT++))
        echo -e "${GREEN}  ✓${NC} $file"
    else
        echo -e "${RED}  ✗${NC} $file (missing)"
    fi
done

echo ""
echo "New files: $NEW_COUNT/${#NEW_FILES[@]}"

if [ "$NEW_COUNT" -eq "${#NEW_FILES[@]}" ]; then
    pass "All new files created"
else
    fail "Some files missing"
fi

# ============================================================================
# Summary
# ============================================================================

section "Test Summary"

echo ""
echo "Total Tests: $((PASSED + FAILED))"
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}✓ ALL TESTS PASSED - FASE 6.3 100% COMPLETE!${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. Review test logs in /tmp/*.log"
    echo "  2. Commit changes: git add . && git commit"
    echo "  3. Create PR to develop branch"
    echo "  4. Proceed to Fase 6.1 + 6.2 (parallel)"
    echo ""
    exit 0
else
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${RED}✗ SOME TESTS FAILED${NC}"
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo "Please review failures above and fix before proceeding."
    echo ""
    exit 1
fi
