# K6 Load Testing Suite

## Prerequisites

Install k6:
- **Windows**: `choco install k6`
- **macOS**: `brew install k6`
- **Linux**: `sudo snap install k6`

## Running Tests

### Load Test (Normal Traffic)
```bash
k6 run benchmarks/k6/load_test.js
```
50 VUs, 5 minutes, normal load simulation

### Stress Test (Ramp Up)
```bash
k6 run benchmarks/k6/stress_test.js
```
100 → 500 VUs, 10 minutes, find breaking point

### Spike Test (Sudden Traffic)
```bash
k6 run benchmarks/k6/spike_test.js
```
0 → 200 VUs immediate spike, test resilience

## Custom Configuration

Override base URL and auth token:
```bash
k6 run -e BASE_URL=http://production:8090 -e AUTH_TOKEN=prod-token benchmarks/k6/load_test.js
```

## Expected Results

| Test | p95 Latency | Error Rate | Throughput |
|------|------------|------------|------------|
| Load | < 5s | < 1% | > 10 req/s |
| Stress | < 10s | < 5% | Variable |
| Spike | < 10s | < 10% | Variable |

## Output

Results saved to:
- `benchmarks/k6/load_test_summary.json`
- `benchmarks/k6/stress_test_summary.json`
- `benchmarks/k6/spike_test_summary.json`
