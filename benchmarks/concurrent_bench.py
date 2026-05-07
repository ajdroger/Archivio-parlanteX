"""Concurrent Query Benchmark - 50 simultaneous queries"""
import asyncio
import time
import statistics
from datetime import datetime
from pathlib import Path
import httpx
from rich.console import Console

console = Console()
API_BASE_URL = "http://localhost:9080/api"
REPORT_DIR = Path(__file__).parent / "reports"

QUERIES = ["Quali sono le penali?"] * 50

async def query(client, q, idx):
    start = time.perf_counter()
    try:
        resp = await client.post(f"{API_BASE_URL}/query",
                                json={"kb_id": "bench", "query": q},
                                timeout=30.0)
        resp.raise_for_status()
        return time.perf_counter() - start
    except:
        return None

async def main():
    REPORT_DIR.mkdir(exist_ok=True)
    console.print("[cyan]Concurrent Benchmark (50 queries)[/cyan]\n")
    
    start_total = time.perf_counter()
    async with httpx.AsyncClient() as client:
        tasks = [query(client, q, i) for i, q in enumerate(QUERIES)]
        results = await asyncio.gather(*tasks)
    
    total_time = time.perf_counter() - start_total
    successful = [r for r in results if r is not None]
    
    if successful:
        throughput = len(successful) / total_time
        p99 = statistics.quantiles([r*1000 for r in successful], n=100)[98]
        
        console.print(f"Throughput: {throughput:.2f} req/s")
        console.print(f"p99 Latency: {p99:.0f} ms")
        console.print(f"Total time: {total_time:.1f}s")
        
        timestamp = datetime.now().strftime('%Y%m%d_%H%M')
        with open(REPORT_DIR / f"concurrent_{timestamp}.md", 'w') as f:
            f.write(f"# Concurrent Benchmark\n\n")
            f.write(f"**Throughput**: {throughput:.2f} req/s (target: >5)  \n")
            f.write(f"**p99 Latency**: {p99:.0f} ms (target: <5000)  \n")
            status = "✅" if throughput > 5 and p99 < 5000 else "❌"
            f.write(f"**Status**: {status}\n")

if __name__ == "__main__":
    asyncio.run(main())
