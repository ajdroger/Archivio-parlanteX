#!/usr/bin/env python3
"""
Graph RAG Benchmark Script

Tests graph-guided retrieval vs pure hybrid search on multi-hop queries.
Measures recall improvement, latency overhead, and failure rate.

Target KPIs:
- Recall@10 improvement: ≥5% vs pure hybrid
- Latency penalty: <200ms (p95)
- Zero failures (graceful fallback)

Usage:
    python benchmarks/graph_rag_bench.py --engine-url http://localhost:8090 --kb-id <kb_id>
"""

import argparse
import asyncio
import json
import statistics
import time
from pathlib import Path
from typing import Dict, List, Any

import httpx
from rich.console import Console
from rich.table import Table
from rich.progress import Progress

console = Console()


class GraphRAGBenchmark:
    def __init__(self, engine_url: str, kb_id: str, queries_file: Path):
        self.engine_url = engine_url.rstrip("/")
        self.kb_id = kb_id
        self.queries_file = queries_file
        self.results: List[Dict[str, Any]] = []

    async def run_query(
        self, client: httpx.AsyncClient, query: str, retrieval_mode: str, graph_expand_depth: int = 2
    ) -> Dict[str, Any]:
        """Execute single query and measure performance."""
        start_time = time.perf_counter()

        try:
            response = await client.post(
                f"{self.engine_url}/query",
                json={
                    "kb_id": self.kb_id,
                    "query": query,
                    "retrieval_mode": retrieval_mode,
                    "graph_expand_depth": graph_expand_depth,
                    "top_k": 10,
                },
                timeout=30.0,
            )
            latency_ms = (time.perf_counter() - start_time) * 1000

            if response.status_code != 200:
                return {
                    "error": f"HTTP {response.status_code}",
                    "latency_ms": latency_ms,
                    "results": [],
                }

            data = response.json()
            return {
                "latency_ms": latency_ms,
                "results": data.get("results", []),
                "error": None,
            }

        except Exception as e:
            latency_ms = (time.perf_counter() - start_time) * 1000
            return {"error": str(e), "latency_ms": latency_ms, "results": []}

    def calculate_recall(self, retrieved_chunks: List[Dict], expected_entities: List[str]) -> float:
        """Calculate recall: how many expected entities were found in top-K results."""
        if not expected_entities:
            return 1.0  # No expected entities = perfect recall

        retrieved_text = " ".join(chunk.get("text", "") for chunk in retrieved_chunks)
        retrieved_text_lower = retrieved_text.lower()

        found_entities = sum(
            1 for entity in expected_entities if entity.lower() in retrieved_text_lower
        )

        return found_entities / len(expected_entities)

    async def run_benchmark(self) -> Dict[str, Any]:
        """Execute full benchmark suite."""
        console.print("\n[bold cyan]Loading test queries...[/bold cyan]")

        with open(self.queries_file, "r", encoding="utf-8") as f:
            test_queries = json.load(f)

        console.print(f"Loaded {len(test_queries)} test queries\n")

        hybrid_recalls: List[float] = []
        graph_recalls: List[float] = []
        hybrid_latencies: List[float] = []
        graph_latencies: List[float] = []
        failures = {"hybrid": 0, "graph": 0}

        async with httpx.AsyncClient() as client:
            with Progress() as progress:
                task = progress.add_task("[cyan]Running benchmark...", total=len(test_queries) * 2)

                for test_case in test_queries:
                    query_id = test_case["id"]
                    query_text = test_case["query"]
                    expected_entities = test_case.get("expected_entities", [])

                    # Run hybrid search (baseline)
                    console.print(f"\n[dim]Testing {query_id} - Hybrid[/dim]")
                    hybrid_result = await self.run_query(client, query_text, "hybrid", 0)
                    progress.update(task, advance=1)

                    if hybrid_result["error"]:
                        console.print(f"[red]Hybrid failed: {hybrid_result['error']}[/red]")
                        failures["hybrid"] += 1
                        hybrid_recalls.append(0.0)
                    else:
                        recall = self.calculate_recall(hybrid_result["results"], expected_entities)
                        hybrid_recalls.append(recall)
                        hybrid_latencies.append(hybrid_result["latency_ms"])

                    # Run graph-guided search
                    console.print(f"[dim]Testing {query_id} - Graph[/dim]")
                    graph_result = await self.run_query(
                        client, query_text, "hybrid+graph", test_case.get("graph_expand_depth", 2)
                    )
                    progress.update(task, advance=1)

                    if graph_result["error"]:
                        console.print(f"[red]Graph failed: {graph_result['error']}[/red]")
                        failures["graph"] += 1
                        graph_recalls.append(0.0)
                    else:
                        recall = self.calculate_recall(graph_result["results"], expected_entities)
                        graph_recalls.append(recall)
                        graph_latencies.append(graph_result["latency_ms"])

                    # Store detailed result
                    self.results.append(
                        {
                            "query_id": query_id,
                            "query": query_text,
                            "hybrid_recall": hybrid_recalls[-1],
                            "graph_recall": graph_recalls[-1],
                            "hybrid_latency_ms": hybrid_latencies[-1] if hybrid_latencies else None,
                            "graph_latency_ms": graph_latencies[-1] if graph_latencies else None,
                            "recall_improvement": graph_recalls[-1] - hybrid_recalls[-1],
                        }
                    )

        # Calculate statistics
        avg_hybrid_recall = statistics.mean(hybrid_recalls) if hybrid_recalls else 0.0
        avg_graph_recall = statistics.mean(graph_recalls) if graph_recalls else 0.0
        recall_improvement_pct = ((avg_graph_recall - avg_hybrid_recall) / avg_hybrid_recall * 100) if avg_hybrid_recall > 0 else 0.0

        latency_overhead_ms = statistics.mean([g - h for g, h in zip(graph_latencies, hybrid_latencies)]) if graph_latencies and hybrid_latencies else 0.0
        latency_p95_ms = statistics.quantiles(graph_latencies, n=20)[18] if len(graph_latencies) >= 20 else max(graph_latencies, default=0.0)

        return {
            "total_queries": len(test_queries),
            "hybrid_avg_recall": avg_hybrid_recall,
            "graph_avg_recall": avg_graph_recall,
            "recall_improvement_pct": recall_improvement_pct,
            "hybrid_failures": failures["hybrid"],
            "graph_failures": failures["graph"],
            "avg_latency_overhead_ms": latency_overhead_ms,
            "graph_latency_p95_ms": latency_p95_ms,
            "passed": (
                recall_improvement_pct >= 5.0
                and latency_p95_ms < 200
                and failures["graph"] == 0
            ),
        }

    def print_results(self, summary: Dict[str, Any]):
        """Print formatted results table."""
        console.print("\n[bold cyan]=" * 60)
        console.print("[bold cyan]Graph RAG Benchmark Results[/bold cyan]")
        console.print("[bold cyan]=" * 60 + "\n")

        # Summary table
        summary_table = Table(title="Performance Summary", show_header=True, header_style="bold magenta")
        summary_table.add_column("Metric", style="cyan")
        summary_table.add_column("Value", style="green")
        summary_table.add_column("Target", style="yellow")
        summary_table.add_column("Status", style="bold")

        summary_table.add_row(
            "Recall Improvement",
            f"{summary['recall_improvement_pct']:.2f}%",
            "≥5%",
            "✅" if summary['recall_improvement_pct'] >= 5.0 else "❌",
        )
        summary_table.add_row(
            "Latency P95",
            f"{summary['graph_latency_p95_ms']:.1f} ms",
            "<200 ms",
            "✅" if summary['graph_latency_p95_ms'] < 200 else "❌",
        )
        summary_table.add_row(
            "Failures",
            f"{summary['graph_failures']} / {summary['total_queries']}",
            "0",
            "✅" if summary['graph_failures'] == 0 else "❌",
        )

        console.print(summary_table)

        # Detailed per-query results
        detail_table = Table(title="\nPer-Query Results", show_header=True, header_style="bold magenta")
        detail_table.add_column("Query ID", style="cyan")
        detail_table.add_column("Hybrid Recall", style="dim")
        detail_table.add_column("Graph Recall", style="green")
        detail_table.add_column("Improvement", style="yellow")
        detail_table.add_column("Latency (ms)", style="blue")

        for result in self.results:
            improvement_str = f"+{result['recall_improvement']:.2%}" if result['recall_improvement'] >= 0 else f"{result['recall_improvement']:.2%}"
            detail_table.add_row(
                result["query_id"],
                f"{result['hybrid_recall']:.2%}",
                f"{result['graph_recall']:.2%}",
                improvement_str,
                f"{result.get('graph_latency_ms', 0):.1f}",
            )

        console.print(detail_table)

        # Final verdict
        console.print()
        if summary["passed"]:
            console.print("[bold green]✅ BENCHMARK PASSED - All KPI targets met[/bold green]")
        else:
            console.print("[bold red]❌ BENCHMARK FAILED - Some KPI targets not met[/bold red]")

        console.print()


async def main():
    parser = argparse.ArgumentParser(description="Graph RAG Benchmark")
    parser.add_argument("--engine-url", default="http://localhost:8090", help="Rust Engine URL")
    parser.add_argument("--kb-id", required=True, help="Knowledge Base ID")
    parser.add_argument(
        "--queries-file",
        type=Path,
        default=Path("tests/fixtures/queries/graph_rag_test_queries.json"),
        help="Test queries JSON file",
    )
    parser.add_argument("--output", type=Path, help="Output JSON file for results")

    args = parser.parse_args()

    if not args.queries_file.exists():
        console.print(f"[red]Error: Queries file not found: {args.queries_file}[/red]")
        return 1

    benchmark = GraphRAGBenchmark(args.engine_url, args.kb_id, args.queries_file)

    console.print(f"[bold cyan]Starting Graph RAG Benchmark[/bold cyan]")
    console.print(f"Engine URL: {args.engine_url}")
    console.print(f"KB ID: {args.kb_id}")
    console.print(f"Queries file: {args.queries_file}\n")

    try:
        summary = await benchmark.run_benchmark()
        benchmark.print_results(summary)

        if args.output:
            with open(args.output, "w", encoding="utf-8") as f:
                json.dump(
                    {
                        "summary": summary,
                        "detailed_results": benchmark.results,
                        "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
                    },
                    f,
                    indent=2,
                )
            console.print(f"\n[dim]Results saved to {args.output}[/dim]")

        return 0 if summary["passed"] else 1

    except KeyboardInterrupt:
        console.print("\n[yellow]Benchmark interrupted by user[/yellow]")
        return 130
    except Exception as e:
        console.print(f"\n[red]Benchmark failed with error: {e}[/red]")
        import traceback
        traceback.print_exc()
        return 1


if __name__ == "__main__":
    exit(asyncio.run(main()))
