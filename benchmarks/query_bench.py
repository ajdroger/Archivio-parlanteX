"""
Query Benchmark - Archivio Parlante

Misura performance query RAG su 100 query gold-set:
- Latency p50/p95/p99
- Recall@5 (doc_ids attesi in top 5 citations)
- Keyword coverage nelle risposte
"""

import asyncio
import time
import json
from pathlib import Path
from datetime import datetime
import statistics

import httpx
import matplotlib.pyplot as plt
from rich.console import Console
from rich.progress import track

console = Console()

API_BASE_URL = "http://localhost:9080/api"
KB_ID = "bench_kb_query"
QUERIES_FILE = Path(__file__).parent / "fixtures" / "queries.jsonl"
REPORT_DIR = Path(__file__).parent / "reports"


async def execute_query(client: httpx.AsyncClient, query: dict):
    """Esegue singola query e misura performance"""
    start = time.perf_counter()
    
    try:
        response = await client.post(
            f"{API_BASE_URL}/query",
            json={
                "kb_id": KB_ID,
                "query": query["question"],
                "top_k": 10,
                "rerank_top_n": 5,
            },
            timeout=30.0
        )
        response.raise_for_status()
        result = response.json()
        
        elapsed = time.perf_counter() - start
        
        # Calculate metrics
        returned_doc_ids = [s.get('doc_id') for s in result.get('sources', [])]
        recall_at_5 = len(set(query['expected_doc_ids']) & set(returned_doc_ids[:5])) / len(query['expected_doc_ids']) if query['expected_doc_ids'] else 0
        
        answer_text = result.get('answer', '').lower()
        keywords_found = sum(1 for kw in query['expected_keywords'] if kw.lower() in answer_text)
        keyword_coverage = keywords_found / len(query['expected_keywords']) if query['expected_keywords'] else 0
        
        return {
            'question': query['question'],
            'success': True,
            'latency_ms': elapsed * 1000,
            'recall_at_5': recall_at_5,
            'keyword_coverage': keyword_coverage,
            'num_sources': len(result.get('sources', [])),
            'verified': result.get('verified', False),
        }
    
    except Exception as e:
        elapsed = time.perf_counter() - start
        return {
            'question': query['question'],
            'success': False,
            'latency_ms': elapsed * 1000,
            'error': str(e),
        }


async def run_benchmark():
    """Esegue benchmark su tutte le query"""
    console.print("[bold cyan]Query Benchmark[/bold cyan]\n")
    
    # Load queries
    queries = []
    with open(QUERIES_FILE, 'r', encoding='utf-8') as f:
        for line in f:
            if line.strip():
                queries.append(json.loads(line))
    
    console.print(f"Loaded {len(queries)} queries\n")
    
    results = []
    async with httpx.AsyncClient() as client:
        for query in track(queries, description="Executing queries..."):
            result = await execute_query(client, query)
            results.append(result)
            await asyncio.sleep(0.2)  # Rate limiting
    
    return results


def analyze_results(results: list[dict]):
    """Analizza risultati e genera report"""
    successful = [r for r in results if r['success']]
    failed = [r for r in results if not r['success']]
    
    if not successful:
        console.print("[bold red]No successful queries![/bold red]")
        return None
    
    latencies = [r['latency_ms'] for r in successful]
    recalls = [r['recall_at_5'] for r in successful]
    coverages = [r['keyword_coverage'] for r in successful]
    
    stats = {
        'count': len(successful),
        'failed_count': len(failed),
        'latency_p50': statistics.median(latencies),
        'latency_p95': statistics.quantiles(latencies, n=20)[18] if len(latencies) > 1 else latencies[0],
        'latency_p99': statistics.quantiles(latencies, n=100)[98] if len(latencies) > 1 else latencies[0],
        'latency_mean': statistics.mean(latencies),
        'recall_at_5_mean': statistics.mean(recalls) * 100,
        'keyword_coverage_mean': statistics.mean(coverages) * 100,
        'verified_count': sum(1 for r in successful if r.get('verified', False)),
    }
    
    # Console output
    console.print(f"\n[bold green]Results:[/bold green]")
    console.print(f"  Queries: {stats['count']} success, {stats['failed_count']} failed")
    console.print(f"  Latency p50: {stats['latency_p50']:.0f} ms")
    console.print(f"  Latency p95: {stats['latency_p95']:.0f} ms")
    console.print(f"  Latency p99: {stats['latency_p99']:.0f} ms")
    console.print(f"  Recall@5: {stats['recall_at_5_mean']:.1f}%")
    console.print(f"  Keyword Coverage: {stats['keyword_coverage_mean']:.1f}%")
    console.print(f"  Verified: {stats['verified_count']}/{stats['count']}")
    
    # Generate chart
    fig, axes = plt.subplots(1, 2, figsize=(14, 5))
    
    # Latency histogram
    axes[0].hist(latencies, bins=20, edgecolor='black', alpha=0.7, color='steelblue')
    axes[0].axvline(stats['latency_p50'], color='green', linestyle='--', label='p50', linewidth=2)
    axes[0].axvline(stats['latency_p95'], color='orange', linestyle='--', label='p95', linewidth=2)
    axes[0].axvline(3000, color='red', linestyle=':', label='Target (3s)', linewidth=2)
    axes[0].set_xlabel('Latency (ms)', fontsize=11)
    axes[0].set_ylabel('Frequency', fontsize=11)
    axes[0].set_title('Query Latency Distribution', fontsize=12, fontweight='bold')
    axes[0].legend()
    axes[0].grid(alpha=0.3)
    
    # Recall scatter
    axes[1].scatter(range(len(recalls)), [r * 100 for r in recalls], alpha=0.6, color='coral')
    axes[1].axhline(95, color='green', linestyle='--', label='Target (95%)', linewidth=2)
    axes[1].axhline(stats['recall_at_5_mean'], color='blue', linestyle='-', label=f'Mean ({stats["recall_at_5_mean"]:.1f}%)', linewidth=2)
    axes[1].set_xlabel('Query Index', fontsize=11)
    axes[1].set_ylabel('Recall@5 (%)', fontsize=11)
    axes[1].set_title('Recall@5 per Query', fontsize=12, fontweight='bold')
    axes[1].legend()
    axes[1].grid(alpha=0.3)
    
    plt.tight_layout()
    
    timestamp = datetime.now().strftime('%Y%m%d_%H%M')
    chart_path = REPORT_DIR / f"query_bench_{timestamp}.png"
    plt.savefig(chart_path, dpi=150, bbox_inches='tight')
    console.print(f"\n[green]Chart saved:[/green] {chart_path}")
    
    # Markdown report
    report_path = REPORT_DIR / f"query_{timestamp}.md"
    with open(report_path, 'w', encoding='utf-8') as f:
        f.write(f"# Query Benchmark Report\n\n")
        f.write(f"**Date**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}  \n\n")
        
        f.write(f"## Summary\n\n")
        f.write(f"| Metric | Value |\n|---|---|\n")
        f.write(f"| Queries Executed | {stats['count']} |\n")
        f.write(f"| Failed | {stats['failed_count']} |\n")
        f.write(f"| Latency p50 | {stats['latency_p50']:.0f} ms |\n")
        f.write(f"| Latency p95 | {stats['latency_p95']:.0f} ms |\n")
        f.write(f"| Latency p99 | {stats['latency_p99']:.0f} ms |\n")
        f.write(f"| Recall@5 | {stats['recall_at_5_mean']:.1f}% |\n")
        f.write(f"| Keyword Coverage | {stats['keyword_coverage_mean']:.1f}% |\n\n")
        
        f.write(f"## Targets\n\n")
        f.write(f"| KPI | Target | Actual | Status |\n|---|---|---|---|\n")
        p95_pass = "✅" if stats['latency_p95'] < 3000 else "❌"
        f.write(f"| p95 Latency | <3s | {stats['latency_p95']/1000:.2f}s | {p95_pass} |\n")
        recall_pass = "✅" if stats['recall_at_5_mean'] >= 95 else "❌"
        f.write(f"| Recall@5 | ≥95% | {stats['recall_at_5_mean']:.1f}% | {recall_pass} |\n\n")
        
        f.write(f"![Chart](query_bench_{timestamp}.png)\n")
    
    console.print(f"[green]Report saved:[/green] {report_path}")
    return stats


async def main():
    REPORT_DIR.mkdir(exist_ok=True)
    
    if not QUERIES_FILE.exists():
        console.print(f"[red]Queries file not found:[/red] {QUERIES_FILE}")
        return
    
    results = await run_benchmark()
    analyze_results(results)
    
    console.print("\n[bold green]✓ Query benchmark complete[/bold green]")


if __name__ == "__main__":
    asyncio.run(main())
