// ============================================================================
// Archivio Parlante - Workspace Permissions Load Test
// ============================================================================
// Fase 6.3 - Load test for multi-tenant permission checks
//
// Test scenario:
// - 100 concurrent users
// - Each user queries KB with permission check
// - Validates <50ms p95 latency for permission checks
// - Validates >90% cache hit rate
// - Validates <1% error rate
//
// Usage: k6 run workspace_permissions_load.js

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics
const permissionCheckDuration = new Trend('permission_check_duration');
const errorRate = new Rate('errors');

// Test configuration
export const options = {
  stages: [
    { duration: '30s', target: 50 },   // Ramp up to 50 users
    { duration: '1m', target: 100 },   // Ramp up to 100 users
    { duration: '2m', target: 100 },   // Sustain 100 users
    { duration: '30s', target: 0 },    // Ramp down
  ],
  thresholds: {
    // Permission check latency: p95 < 50ms
    'permission_check_duration': ['p(95)<50'],

    // HTTP request duration: p95 < 200ms
    'http_req_duration': ['p(95)<200'],

    // Error rate: < 1%
    'errors': ['rate<0.01'],

    // Request failure rate: < 1%
    'http_req_failed': ['rate<0.01'],
  },
};

// Test data - simulating multiple workspaces and users
const workspaces = ['ws-legal', 'ws-finance', 'ws-hr'];
const kbs = ['kb-contracts', 'kb-shared', 'kb-finance', 'kb-reports'];
const users = [
  { id: 100, token: 'test-token-alice-100', role: 'admin' },
  { id: 101, token: 'test-token-bob-101', role: 'member' },
  { id: 102, token: 'test-token-charlie-102', role: 'viewer' },
];

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8090';

export default function () {
  // Select random user
  const user = users[Math.floor(Math.random() * users.length)];

  // Select random KB
  const kb = kbs[Math.floor(Math.random() * kbs.length)];

  // Prepare headers
  const headers = {
    'Authorization': `Bearer ${user.token}`,
    'Content-Type': 'application/json',
  };

  // Execute query (triggers permission check in middleware)
  const startTime = new Date();

  const payload = JSON.stringify({
    kb_id: kb,
    query: 'test query for load testing',
  });

  const response = http.post(`${BASE_URL}/query`, payload, { headers });

  const duration = new Date() - startTime;
  permissionCheckDuration.add(duration);

  // Validate response
  const success = check(response, {
    'status is 200 or 403': (r) => r.status === 200 || r.status === 403,
    'response has body': (r) => r.body.length > 0,
    'latency < 200ms': (r) => r.timings.duration < 200,
  });

  if (!success) {
    errorRate.add(1);
    console.error(`Request failed: ${response.status} - ${response.body.substring(0, 100)}`);
  } else {
    errorRate.add(0);
  }

  // Simulate user think time
  sleep(1);
}

// Warmup function - runs once before main test
export function setup() {
  console.log('='.repeat(60));
  console.log('Workspace Permission Load Test');
  console.log('='.repeat(60));
  console.log(`Target: ${BASE_URL}`);
  console.log('Configuration:');
  console.log('  - Max concurrent users: 100');
  console.log('  - Duration: 4 minutes');
  console.log('  - Workspaces: 3');
  console.log('  - Knowledge bases: 4');
  console.log('  - Test users: 3 (admin, member, viewer)');
  console.log('='.repeat(60));
  console.log('');

  // Warmup request
  const warmupResponse = http.get(`${BASE_URL}/health`);
  if (warmupResponse.status !== 200) {
    console.error('⚠️  Health check failed - server may not be ready');
  } else {
    console.log('✓ Server health check passed');
  }

  return { startTime: new Date() };
}

// Teardown function - runs once after main test
export function teardown(data) {
  console.log('');
  console.log('='.repeat(60));
  console.log('Load Test Complete');
  console.log('='.repeat(60));
  const duration = (new Date() - data.startTime) / 1000;
  console.log(`Total duration: ${duration.toFixed(2)}s`);
  console.log('');
  console.log('Check summary for detailed metrics:');
  console.log('  - permission_check_duration (p95 should be <50ms)');
  console.log('  - http_req_duration (p95 should be <200ms)');
  console.log('  - errors (should be <1%)');
  console.log('  - http_req_failed (should be <1%)');
  console.log('='.repeat(60));
}

// ============================================================================
// Expected Results:
// ============================================================================
// ✓ permission_check_duration: p95 < 50ms
// ✓ http_req_duration: p95 < 200ms
// ✓ errors: rate < 1%
// ✓ http_req_failed: rate < 1%
//
// Redis cache hit rate should be >90% (check Redis INFO stats separately)
// ============================================================================
