/**
 * K6 Stress Test - Archivio Parlante
 * 
 * Scenario: Ramp up from 100 to 500 users over 10 minutes
 * Goal: Find breaking point and observe degradation
 */

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

const errorRate = new Rate('errors');

export const options = {
  stages: [
    { duration: '2m', target: 100 },  // Ramp up to 100 users
    { duration: '3m', target: 300 },  // Ramp to 300 users
    { duration: '3m', target: 500 },  // Ramp to 500 users (stress)
    { duration: '2m', target: 0 },    // Ramp down to 0 users
  ],
  thresholds: {
    'errors': ['rate<0.05'],  // Allow up to 5% errors under stress
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8090';
const AUTH_TOKEN = __ENV.AUTH_TOKEN || 'test-token';

export default function () {
  const payload = JSON.stringify({
    query: 'Test query under stress',
    kb_id: 1,
    provider: 'ollama',
    model: 'qwen2.5:3b',  // Use smaller model for stress test
  });

  const res = http.post(`${BASE_URL}/api/query`, payload, {
    headers: {
      'Content-Type': 'application/json',
      'X-Internal-Token': AUTH_TOKEN,
    },
    timeout: '30s',
  });

  const success = check(res, {
    'status 200 or 429': (r) => r.status === 200 || r.status === 429,
  });

  errorRate.add(!success);

  sleep(1);
}

export function handleSummary(data) {
  return {
    'benchmarks/k6/stress_test_summary.json': JSON.stringify(data, null, 2),
  };
}
