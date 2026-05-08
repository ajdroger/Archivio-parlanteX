#!/bin/bash
# ============================================================================
# Archivio Parlante - Workspace Integration Test Suite
# ============================================================================
# Fase 6.3 - 5 end-to-end scenarios for multi-tenant workspace isolation
#
# Prerequisites:
# - All services running: make up
# - Test users exist in database
# - jq installed for JSON parsing
#
# Usage: ./workspace_scenarios.sh

set -e

BASE_URL="http://localhost:9080"
RUST_URL="http://localhost:8090"

echo "========================================="
echo "Workspace Integration Test Suite"
echo "========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
PASSED=0
FAILED=0

# Helper functions
pass() {
    echo -e "${GREEN}✓ PASS${NC}: $1"
    ((PASSED++))
}

fail() {
    echo -e "${RED}✗ FAIL${NC}: $1"
    ((FAILED++))
}

info() {
    echo -e "${YELLOW}→${NC} $1"
}

# Mock JWT tokens (in real scenario, would login via /api/auth/login)
# For testing, we assume auth middleware allows these test tokens
ALICE_TOKEN="test-token-alice-100"
BOB_TOKEN="test-token-bob-101"
CHARLIE_TOKEN="test-token-charlie-102"
DAVID_TOKEN="test-token-david-103"

echo "========================================="
echo "Scenario 1: Create Workspace & Add Members"
echo "========================================="
echo ""

info "Step 1.1: Alice creates workspace 'Legal Team'"
WORKSPACE_RESPONSE=$(curl -s -X POST "$BASE_URL/api/workspaces" \
  -H "Authorization: Bearer $ALICE_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"Legal Team"}')

WORKSPACE_ID=$(echo "$WORKSPACE_RESPONSE" | jq -r '.id // empty')

if [ -n "$WORKSPACE_ID" ]; then
    pass "Workspace created: $WORKSPACE_ID"
else
    fail "Workspace creation failed"
    echo "Response: $WORKSPACE_RESPONSE"
fi

info "Step 1.2: Alice adds Bob as member"
ADD_BOB=$(curl -s -X POST "$BASE_URL/api/workspaces/$WORKSPACE_ID/members" \
  -H "Authorization: Bearer $ALICE_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"user_id":101,"role":"member"}')

if echo "$ADD_BOB" | jq -e '.success' > /dev/null 2>&1; then
    pass "Bob added as member"
else
    fail "Failed to add Bob"
fi

info "Step 1.3: Alice adds Charlie as viewer"
ADD_CHARLIE=$(curl -s -X POST "$BASE_URL/api/workspaces/$WORKSPACE_ID/members" \
  -H "Authorization: Bearer $ALICE_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"user_id":102,"role":"viewer"}')

if echo "$ADD_CHARLIE" | jq -e '.success' > /dev/null 2>&1; then
    pass "Charlie added as viewer"
else
    fail "Failed to add Charlie"
fi

info "Step 1.4: Bob verifies workspace in list"
BOB_WORKSPACES=$(curl -s "$BASE_URL/api/workspaces" \
  -H "Authorization: Bearer $BOB_TOKEN")

if echo "$BOB_WORKSPACES" | jq -e ".workspaces[] | select(.id == \"$WORKSPACE_ID\" and .user_role == \"member\")" > /dev/null 2>&1; then
    pass "Bob sees workspace with role 'member'"
else
    fail "Bob doesn't see workspace correctly"
fi

info "Step 1.5: Charlie verifies workspace with viewer role"
CHARLIE_WORKSPACES=$(curl -s "$BASE_URL/api/workspaces" \
  -H "Authorization: Bearer $CHARLIE_TOKEN")

if echo "$CHARLIE_WORKSPACES" | jq -e ".workspaces[] | select(.id == \"$WORKSPACE_ID\" and .user_role == \"viewer\")" > /dev/null 2>&1; then
    pass "Charlie sees workspace with role 'viewer'"
else
    fail "Charlie doesn't see workspace correctly"
fi

echo ""
echo "========================================="
echo "Scenario 2: KB Sharing Within Workspace"
echo "========================================="
echo ""

info "Step 2.1: Alice creates KB in workspace"
KB_RESPONSE=$(curl -s -X POST "$BASE_URL/api/kb" \
  -H "Authorization: Bearer $ALICE_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"Contracts 2024\",\"workspace_id\":\"$WORKSPACE_ID\"}")

KB_ID=$(echo "$KB_RESPONSE" | jq -r '.id // empty')

if [ -n "$KB_ID" ]; then
    pass "KB created: $KB_ID"
else
    fail "KB creation failed"
fi

info "Step 2.2: Alice shares KB with workspace (READ permission)"
SHARE_KB=$(curl -s -X POST "$BASE_URL/api/kb/$KB_ID/permissions" \
  -H "Authorization: Bearer $ALICE_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"workspace_id\":\"$WORKSPACE_ID\",\"permission\":\"read\"}")

if echo "$SHARE_KB" | jq -e '.success' > /dev/null 2>&1; then
    pass "KB shared with workspace"
else
    fail "Failed to share KB"
fi

info "Step 2.3: Bob (member) queries KB"
BOB_QUERY=$(curl -s -w "\n%{http_code}" -X POST "$RUST_URL/query" \
  -H "Authorization: Bearer $BOB_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"kb_id\":\"$KB_ID\",\"query\":\"test query\"}")

HTTP_CODE=$(echo "$BOB_QUERY" | tail -n1)
if [ "$HTTP_CODE" = "200" ]; then
    pass "Bob can query KB (200 OK)"
else
    fail "Bob cannot query KB (HTTP $HTTP_CODE)"
fi

info "Step 2.4: Bob tries to upload document"
BOB_UPLOAD=$(curl -s -w "\n%{http_code}" -X POST "$RUST_URL/kb/$KB_ID/documents" \
  -H "Authorization: Bearer $BOB_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"document":"test"}')

HTTP_CODE=$(echo "$BOB_UPLOAD" | tail -n1)
if [ "$HTTP_CODE" = "200" ] || [ "$HTTP_CODE" = "201" ]; then
    pass "Bob can upload (member has write)"
else
    fail "Bob cannot upload (HTTP $HTTP_CODE)"
fi

info "Step 2.5: Charlie (viewer) queries KB"
CHARLIE_QUERY=$(curl -s -w "\n%{http_code}" -X POST "$RUST_URL/query" \
  -H "Authorization: Bearer $CHARLIE_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"kb_id\":\"$KB_ID\",\"query\":\"test query\"}")

HTTP_CODE=$(echo "$CHARLIE_QUERY" | tail -n1)
if [ "$HTTP_CODE" = "200" ]; then
    pass "Charlie can query KB (200 OK)"
else
    fail "Charlie cannot query KB (HTTP $HTTP_CODE)"
fi

info "Step 2.6: Charlie tries to upload document"
CHARLIE_UPLOAD=$(curl -s -w "\n%{http_code}" -X POST "$RUST_URL/kb/$KB_ID/documents" \
  -H "Authorization: Bearer $CHARLIE_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"document":"test"}')

HTTP_CODE=$(echo "$CHARLIE_UPLOAD" | tail -n1)
if [ "$HTTP_CODE" = "403" ]; then
    pass "Charlie denied upload (403 Forbidden - viewer cannot write)"
else
    fail "Charlie upload not properly blocked (HTTP $HTTP_CODE)"
fi

echo ""
echo "========================================="
echo "Scenario 3: Permission Revocation"
echo "========================================="
echo ""

info "Step 3.1: Alice removes Bob from workspace"
REMOVE_BOB=$(curl -s -X DELETE "$BASE_URL/api/workspaces/$WORKSPACE_ID/members/101" \
  -H "Authorization: Bearer $ALICE_TOKEN")

if echo "$REMOVE_BOB" | jq -e '.success' > /dev/null 2>&1; then
    pass "Bob removed from workspace"
else
    fail "Failed to remove Bob"
fi

info "Step 3.2: Bob tries to query KB (should be denied)"
sleep 2  # Allow cache to expire (if TTL < 2s)
BOB_QUERY_AFTER=$(curl -s -w "\n%{http_code}" -X POST "$RUST_URL/query" \
  -H "Authorization: Bearer $BOB_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"kb_id\":\"$KB_ID\",\"query\":\"test query\"}")

HTTP_CODE=$(echo "$BOB_QUERY_AFTER" | tail -n1)
if [ "$HTTP_CODE" = "403" ]; then
    pass "Bob access revoked (403 Forbidden)"
else
    fail "Bob still has access after removal (HTTP $HTTP_CODE)"
fi

echo ""
echo "========================================="
echo "Scenario 4: Cross-Workspace Isolation"
echo "========================================="
echo ""

info "Step 4.1: Alice creates workspace 'Legal Team'"
# Already created above

info "Step 4.2: Bob creates workspace 'Finance Team'"
FINANCE_WS=$(curl -s -X POST "$BASE_URL/api/workspaces" \
  -H "Authorization: Bearer $BOB_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"Finance Team"}')

FINANCE_WS_ID=$(echo "$FINANCE_WS" | jq -r '.id // empty')

if [ -n "$FINANCE_WS_ID" ]; then
    pass "Finance workspace created: $FINANCE_WS_ID"
else
    fail "Finance workspace creation failed"
fi

info "Step 4.3: Bob creates KB in Finance workspace"
FINANCE_KB=$(curl -s -X POST "$BASE_URL/api/kb" \
  -H "Authorization: Bearer $BOB_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"Finance Reports\",\"workspace_id\":\"$FINANCE_WS_ID\"}")

FINANCE_KB_ID=$(echo "$FINANCE_KB" | jq -r '.id // empty')

if [ -n "$FINANCE_KB_ID" ]; then
    pass "Finance KB created: $FINANCE_KB_ID"
else
    fail "Finance KB creation failed"
fi

info "Step 4.4: Alice tries to query Bob's Finance KB"
ALICE_CROSS_QUERY=$(curl -s -w "\n%{http_code}" -X POST "$RUST_URL/query" \
  -H "Authorization: Bearer $ALICE_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"kb_id\":\"$FINANCE_KB_ID\",\"query\":\"test\"}")

HTTP_CODE=$(echo "$ALICE_CROSS_QUERY" | tail -n1)
if [ "$HTTP_CODE" = "403" ]; then
    pass "Cross-workspace access denied (403 Forbidden)"
else
    fail "Cross-workspace isolation broken (HTTP $HTTP_CODE)"
fi

info "Step 4.5: Bob tries to query Alice's Legal KB"
BOB_CROSS_QUERY=$(curl -s -w "\n%{http_code}" -X POST "$RUST_URL/query" \
  -H "Authorization: Bearer $BOB_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"kb_id\":\"$KB_ID\",\"query\":\"test\"}")

HTTP_CODE=$(echo "$BOB_CROSS_QUERY" | tail -n1)
if [ "$HTTP_CODE" = "403" ]; then
    pass "Cross-workspace access denied (403 Forbidden)"
else
    fail "Cross-workspace isolation broken (HTTP $HTTP_CODE)"
fi

echo ""
echo "========================================="
echo "Scenario 5: Workspace Admin Privileges"
echo "========================================="
echo ""

info "Step 5.1: Bob creates private KB in Legal workspace (not shared)"
PRIVATE_KB=$(curl -s -X POST "$BASE_URL/api/kb" \
  -H "Authorization: Bearer $BOB_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"Bob Private\",\"workspace_id\":\"$WORKSPACE_ID\"}")

PRIVATE_KB_ID=$(echo "$PRIVATE_KB" | jq -r '.id // empty')

if [ -n "$PRIVATE_KB_ID" ]; then
    pass "Private KB created: $PRIVATE_KB_ID"
else
    fail "Private KB creation failed"
fi

info "Step 5.2: Alice (workspace admin) queries Bob's private KB"
ALICE_ADMIN_QUERY=$(curl -s -w "\n%{http_code}" -X POST "$RUST_URL/query" \
  -H "Authorization: Bearer $ALICE_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"kb_id\":\"$PRIVATE_KB_ID\",\"query\":\"test\"}")

HTTP_CODE=$(echo "$ALICE_ADMIN_QUERY" | tail -n1)
if [ "$HTTP_CODE" = "200" ]; then
    pass "Workspace admin can access all workspace KBs (200 OK)"
else
    fail "Workspace admin privilege not working (HTTP $HTTP_CODE)"
fi

info "Step 5.3: Charlie (member) queries Bob's private KB"
CHARLIE_PRIVATE_QUERY=$(curl -s -w "\n%{http_code}" -X POST "$RUST_URL/query" \
  -H "Authorization: Bearer $CHARLIE_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"kb_id\":\"$PRIVATE_KB_ID\",\"query\":\"test\"}")

HTTP_CODE=$(echo "$CHARLIE_PRIVATE_QUERY" | tail -n1)
if [ "$HTTP_CODE" = "403" ]; then
    pass "Non-admin member denied private KB access (403 Forbidden)"
else
    fail "Private KB security breach (HTTP $HTTP_CODE)"
fi

echo ""
echo "========================================="
echo "Test Summary"
echo "========================================="
echo -e "Total: $((PASSED + FAILED))"
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All integration tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi
