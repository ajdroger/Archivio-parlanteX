/**
 * K6 Spike Test - Archivio Parlante
 * 
 * Scenario: Sudden spike from 0 to 200 users (simulates viral traffic)
 * Goal: Test system resilience and recovery
 */

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

const errorRate = new Rate('errors');

export const options = {
  stages: [
    { duration: '10s', target: 0 },    // Warm up
    { duration: '10s', target: 200 },  // SPIKE to 200 users
    { duration: '3m', target: 200 },   // Hold spike
    { duration: '10s', target: 0 },    // Drop to 0
  ],
  thresholds: {
    'http_req_duration': ['p(95)<10000'],  // Allow higher latency during spike
    'errors': ['rate<0.10'],  // Allow up to 10% errors during spike
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8090';
const AUTH_TOKEN = __ENV.AUTH_TOKEN || 'test-token';

export default function () {
  const res = http.get(`${BASE_URL}/health`, {
    headers: {
      'X-Internal-Token': AUTH_TOKEN,
    },
    timeout: '15s',
  });

  const success = check(res, {
    'status 200 or 503': (r) => r.status === 200 || r.status === 503,
  });

  errorRate.add(!success);

  sleep(Math.random() * 2);  // Random sleep 0-2s
}

export function handleSummary(data) {
  return {
    'benchmarks/k6/spike_test_summary.json': JSON.stringify(data, null, 2),
  };
}
