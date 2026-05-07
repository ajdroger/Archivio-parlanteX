/**
 * K6 Load Test - Archivio Parlante
 * 
 * Scenario: Normal load (50 concurrent users, 5 minutes)
 * Target: p95 latency < 5s, error rate < 1%, throughput > 10 req/s
 */

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const queryLatency = new Trend('query_latency');

// Test configuration
export const options = {
  vus: 50,  // 50 virtual users
  duration: '5m',  // Run for 5 minutes
  thresholds: {
    'http_req_duration': ['p(95)<5000'],  // 95% of requests < 5s
    'errors': ['rate<0.01'],  // Error rate < 1%
    'http_reqs': ['rate>10'],  // Throughput > 10 req/s
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8090';
const AUTH_TOKEN = __ENV.AUTH_TOKEN || 'test-token';

export default function () {
  // Test 1: Query RAG endpoint
  const queryPayload = JSON.stringify({
    query: 'Quali sono le clausole di rescissione del contratto?',
    kb_id: 1,
    provider: 'ollama',
    model: 'qwen2.5:7b',
  });

  const queryRes = http.post(`${BASE_URL}/api/query`, queryPayload, {
    headers: {
      'Content-Type': 'application/json',
      'X-Internal-Token': AUTH_TOKEN,
    },
  });

  const querySuccess = check(queryRes, {
    'query status 200': (r) => r.status === 200,
    'query has answer': (r) => r.json('answer') !== undefined,
  });

  errorRate.add(!querySuccess);
  queryLatency.add(queryRes.timings.duration);

  sleep(1);

  // Test 2: List documents
  const listRes = http.get(`${BASE_URL}/api/kb/1/documents`, {
    headers: {
      'X-Internal-Token': AUTH_TOKEN,
    },
  });

  check(listRes, {
    'list status 200': (r) => r.status === 200,
  });

  sleep(2);
}

export function handleSummary(data) {
  return {
    'benchmarks/k6/load_test_summary.json': JSON.stringify(data, null, 2),
  };
}
