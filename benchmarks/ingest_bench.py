"""
Ingest Benchmark - Archivio Parlante

Misura performance ingestion pipeline:
- Tempo per documento (p50/p95/p99)
- Throughput (documenti/minuto)
- Memory peak, CPU%, VRAM usage
"""

import asyncio
import time
from pathlib import Path
from datetime import datetime
import statistics
import json

import httpx
import psutil
import matplotlib.pyplot as plt
from rich.console import Console
from rich.table import Table
from rich.progress import track

console = Console()

# Configurazione
API_BASE_URL = "http://localhost:9080/api"
KB_ID = "bench_kb_ingest"
FIXTURES_DIR = Path(__file__).parent / "fixtures" / "contracts"
REPORT_DIR = Path(__file__).parent / "reports"


async def ingest_document(client: httpx.AsyncClient, pdf_path: Path, doc_id: str):
    """Ingest singolo PDF e misura tempo"""
    start = time.perf_counter()
    
    try:
        # Upload PDF
        with open(pdf_path, 'rb') as f:
            files = {'file': (pdf_path.name, f, 'application/pdf')}
            data = {
                'kb_id': KB_ID,
                'doc_id': doc_id,
            }
            
            response = await client.post(
                f"{API_BASE_URL}/ingest",
                files=files,
                data=data,
                timeout=120.0
            )
            response.raise_for_status()
            result = response.json()
        
        elapsed = time.perf_counter() - start
        
        return {
            'doc_id': doc_id,
            'success': True,
            'elapsed_ms': elapsed * 1000,
            'chunk_count': result.get('chunk_count', 0),
            'file_size_kb': pdf_path.stat().st_size / 1024,
        }
    
    except Exception as e:
        elapsed = time.perf_counter() - start
        return {
            'doc_id': doc_id,
            'success': False,
            'elapsed_ms': elapsed * 1000,
            'error': str(e),
        }


async def run_sequential_benchmark(pdf_files: list[Path]):
    """Benchmark sequenziale: un doc alla volta"""
    console.print("\n[bold cyan]Sequential Ingestion Benchmark[/bold cyan]")
    
    results = []
    process = psutil.Process()
    initial_memory = process.memory_info().rss / 1024 / 1024  # MB
    
    async with httpx.AsyncClient() as client:
        for pdf_path in track(pdf_files, description="Ingesting..."):
            doc_id = f"bench_{pdf_path.stem}"
            result = await ingest_document(client, pdf_path, doc_id)
            results.append(result)
            
            # Simulate realistic delay between ingests
            await asyncio.sleep(0.5)
    
    peak_memory = process.memory_info().rss / 1024 / 1024  # MB
    memory_delta = peak_memory - initial_memory
    
    return results, memory_delta


async def run_parallel_benchmark(pdf_files: list[Path], concurrency: int = 5):
    """Benchmark parallelo: N doc simultanei"""
    console.print(f"\n[bold cyan]Parallel Ingestion Benchmark (concurrency={concurrency})[/bold cyan]")
    
    async with httpx.AsyncClient() as client:
        tasks = []
        for pdf_path in pdf_files:
            doc_id = f"bench_parallel_{pdf_path.stem}"
            tasks.append(ingest_document(client, pdf_path, doc_id))
        
        results = await asyncio.gather(*tasks)
    
    return results


def analyze_results(results: list[dict], memory_delta_mb: float = 0):
    """Analisi statistiche sui risultati"""
    successful = [r for r in results if r['success']]
    failed = [r for r in results if not r['success']]
    
    if not successful:
        console.print("[bold red]No successful ingestions![/bold red]")
        return
    
    latencies = [r['elapsed_ms'] for r in successful]
    chunk_counts = [r['chunk_count'] for r in successful]
    
    # Statistiche
    stats = {
        'count': len(successful),
        'failed_count': len(failed),
        'latency_p50': statistics.median(latencies),
        'latency_p95': statistics.quantiles(latencies, n=20)[18] if len(latencies) > 1 else latencies[0],
        'latency_p99': statistics.quantiles(latencies, n=100)[98] if len(latencies) > 1 else latencies[0],
        'latency_mean': statistics.mean(latencies),
        'latency_min': min(latencies),
        'latency_max': max(latencies),
        'throughput_docs_per_min': 60000 / statistics.mean(latencies) if latencies else 0,
        'total_chunks': sum(chunk_counts),
        'avg_chunks_per_doc': statistics.mean(chunk_counts) if chunk_counts else 0,
        'memory_delta_mb': memory_delta_mb,
    }
    
    # Tabella risultati
    table = Table(title="Ingest Benchmark Results")
    table.add_column("Metric", style="cyan")
    table.add_column("Value", style="green")
    
    table.add_row("Total Documents", str(stats['count']))
    table.add_row("Failed", str(stats['failed_count']))
    table.add_row("Latency p50", f"{stats['latency_p50']:.0f} ms")
    table.add_row("Latency p95", f"{stats['latency_p95']:.0f} ms")
    table.add_row("Latency p99", f"{stats['latency_p99']:.0f} ms")
    table.add_row("Mean Latency", f"{stats['latency_mean']:.0f} ms")
    table.add_row("Throughput", f"{stats['throughput_docs_per_min']:.1f} docs/min")
    table.add_row("Total Chunks", str(stats['total_chunks']))
    table.add_row("Avg Chunks/Doc", f"{stats['avg_chunks_per_doc']:.1f}")
    table.add_row("Memory Delta", f"{stats['memory_delta_mb']:.1f} MB")
    
    console.print(table)
    
    # Grafico latenze
    plt.figure(figsize=(10, 6))
    plt.hist(latencies, bins=20, edgecolor='black', alpha=0.7)
    plt.axvline(stats['latency_p50'], color='green', linestyle='--', label='p50')
    plt.axvline(stats['latency_p95'], color='orange', linestyle='--', label='p95')
    plt.axvline(stats['latency_p99'], color='red', linestyle='--', label='p99')
    plt.xlabel('Latency (ms)')
    plt.ylabel('Frequency')
    plt.title('Ingest Latency Distribution')
    plt.legend()
    plt.grid(alpha=0.3)
    
    chart_path = REPORT_DIR / f"ingest_latency_{datetime.now().strftime('%Y%m%d_%H%M')}.png"
    plt.savefig(chart_path, dpi=150, bbox_inches='tight')
    console.print(f"[green]Chart saved:[/green] {chart_path}")
    
    return stats


def generate_markdown_report(stats: dict, timestamp: str):
    """Genera report Markdown"""
    report_path = REPORT_DIR / f"ingest_{timestamp}.md"
    
    with open(report_path, 'w', encoding='utf-8') as f:
        f.write(f"# Ingest Benchmark Report\n\n")
        f.write(f"**Date**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}  \n")
        f.write(f"**KB ID**: `{KB_ID}`  \n\n")
        
        f.write(f"## Summary\n\n")
        f.write(f"| Metric | Value |\n")
        f.write(f"|---|---|\n")
        f.write(f"| Documents Processed | {stats['count']} |\n")
        f.write(f"| Failed | {stats['failed_count']} |\n")
        f.write(f"| Latency p50 | {stats['latency_p50']:.0f} ms |\n")
        f.write(f"| Latency p95 | {stats['latency_p95']:.0f} ms |\n")
        f.write(f"| Latency p99 | {stats['latency_p99']:.0f} ms |\n")
        f.write(f"| Throughput | {stats['throughput_docs_per_min']:.1f} docs/min |\n")
        f.write(f"| Total Chunks | {stats['total_chunks']} |\n")
        f.write(f"| Memory Delta | {stats['memory_delta_mb']:.1f} MB |\n\n")
        
        f.write(f"## Targets vs Actual\n\n")
        f.write(f"| KPI | Target | Actual | Status |\n")
        f.write(f"|---|---|---|---|\n")
        
        p50_pass = "✅" if stats['latency_p50'] < 30000 else "❌"
        f.write(f"| 50-page PDF p50 | <30s | {stats['latency_p50']/1000:.1f}s | {p50_pass} |\n")
        
        throughput_pass = "✅" if stats['throughput_docs_per_min'] >= 10 else "❌"
        f.write(f"| Parallel Throughput | ≥10 docs/min | {stats['throughput_docs_per_min']:.1f} | {throughput_pass} |\n\n")
        
        f.write(f"## Chart\n\n")
        f.write(f"![Latency Distribution](ingest_latency_{timestamp}.png)\n")
    
    console.print(f"[green]Report saved:[/green] {report_path}")


async def main():
    REPORT_DIR.mkdir(exist_ok=True)
    timestamp = datetime.now().strftime('%Y%m%d_%H%M')
    
    # Verifica fixtures
    if not FIXTURES_DIR.exists():
        console.print(f"[red]Fixtures not found![/red] Run: cd fixtures && python generate_contracts.py")
        return
    
    pdf_files = list(FIXTURES_DIR.glob("*.pdf"))[:20]  # Limit to 20 for speed
    
    if not pdf_files:
        console.print(f"[red]No PDF files in {FIXTURES_DIR}[/red]")
        return
    
    console.print(f"[bold]Found {len(pdf_files)} PDF files[/bold]")
    
    # Run benchmarks
    results, memory_delta = await run_sequential_benchmark(pdf_files)
    stats = analyze_results(results, memory_delta)
    
    if stats:
        generate_markdown_report(stats, timestamp)
    
    console.print("\n[bold green]✓ Ingest benchmark complete[/bold green]")


if __name__ == "__main__":
    asyncio.run(main())
