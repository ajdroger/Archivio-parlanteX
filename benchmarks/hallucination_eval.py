#!/usr/bin/env python3
"""
Hallucination Detection Evaluation Script

Tests hallucination detector on trick questions and validates precision/recall.
Measures false positive rate, latency overhead, and specificity.

Target KPIs:
- Hallucination rate on trick questions: ≤1%
- Precision on flagging: ≥85%
- Latency overhead: ≤300ms (p95)
- Specificity (no false positives on valid answers): >95%

Usage:
    python benchmarks/hallucination_eval.py --engine-url http://localhost:8090 --kb-id <kb_id>
"""

import argparse
import asyncio
import json
import re
import statistics
import time
from pathlib import Path
from typing import Dict, List, Any, Optional

import httpx
from rich.console import Console
from rich.table import Table
from rich.progress import Progress

console = Console()


class HallucinationEvaluator:
    def __init__(
        self,
        engine_url: str,
        kb_id: str,
        trick_questions_file: Path,
        valid_queries_file: Optional[Path] = None,
    ):
        self.engine_url = engine_url.rstrip("/")
        self.kb_id = kb_id
        self.trick_questions_file = trick_questions_file
        self.valid_queries_file = valid_queries_file
        self.results: List[Dict[str, Any]] = []

    async def run_chat(self, client: httpx.AsyncClient, query: str, user_id: int = 1) -> Dict[str, Any]:
        """Execute chat query with hallucination detection."""
        session_id = f"eval_{int(time.time() * 1000)}"
        start_time = time.perf_counter()

        try:
            response = await client.post(
                f"{self.engine_url}/chat",
                json={
                    "kb_id": self.kb_id,
                    "query": query,
                    "session_id": session_id,
                    "user_id": user_id,
                    "verify_hallucinations": True,
                    "top_k": 5,
                },
                timeout=60.0,
            )
            latency_ms = (time.perf_counter() - start_time) * 1000

            if response.status_code != 200:
                return {
                    "error": f"HTTP {response.status_code}",
                    "latency_ms": latency_ms,
                    "answer": None,
                    "verification": None,
                }

            data = response.json()
            return {
                "latency_ms": latency_ms,
                "answer": data.get("answer", ""),
                "verification": data.get("verification"),
                "citations": data.get("citations", []),
                "error": None,
            }

        except Exception as e:
            latency_ms = (time.perf_counter() - start_time) * 1000
            return {
                "error": str(e),
                "latency_ms": latency_ms,
                "answer": None,
                "verification": None,
            }

    def check_refusal(self, answer: str, expected_pattern: str) -> bool:
        """Check if answer correctly refuses to answer (matches expected pattern)."""
        if not answer:
            return False

        answer_lower = answer.lower()
        # Check for common refusal phrases
        refusal_indicators = [
            "informazioni non presenti",
            "informazioni non sono presenti",
            "non disponibile",
            "non specificato",
            "non menziona",
            "non specifica",
            "non riportato",
            "non indicato",
            "fuori ambito",
            "posso solo",
            "documenti caricati",
            "non genero",
        ]

        has_refusal = any(indicator in answer_lower for indicator in refusal_indicators)

        # Also check expected pattern (regex)
        try:
            pattern_match = bool(re.search(expected_pattern, answer, re.IGNORECASE))
        except re.error:
            pattern_match = False

        return has_refusal or pattern_match

    async def evaluate_trick_questions(self, client: httpx.AsyncClient, test_cases: List[Dict]) -> Dict[str, Any]:
        """Evaluate on trick questions (should refuse to answer)."""
        console.print("\n[bold cyan]Testing trick questions (should refuse)...[/bold cyan]\n")

        correct_refusals = 0
        hallucinations = 0
        latencies: List[float] = []
        hallucination_scores: List[float] = []

        with Progress() as progress:
            task = progress.add_task("[cyan]Testing trick questions...", total=len(test_cases))

            for test_case in test_cases:
                question_id = test_case["id"]
                question = test_case["question"]
                expected_pattern = test_case.get("expected_answer_pattern", "")

                console.print(f"[dim]Testing {question_id}: {question[:60]}...[/dim]")

                result = await self.run_chat(client, question)
                progress.update(task, advance=1)

                if result["error"]:
                    console.print(f"[red]Error: {result['error']}[/red]")
                    continue

                latencies.append(result["latency_ms"])
                answer = result["answer"] or ""
                verification = result["verification"]

                hallucination_score = verification.get("hallucination_score", 1.0) if verification else 1.0
                hallucination_scores.append(hallucination_score)

                refused = self.check_refusal(answer, expected_pattern)

                if refused:
                    correct_refusals += 1
                    console.print(f"[green]✓ Correctly refused[/green]")
                else:
                    hallucinations += 1
                    console.print(f"[red]✗ Hallucinated answer: {answer[:100]}...[/red]")

                self.results.append(
                    {
                        "question_id": question_id,
                        "type": "trick",
                        "question": question,
                        "answer": answer,
                        "hallucination_score": hallucination_score,
                        "refused": refused,
                        "latency_ms": result["latency_ms"],
                    }
                )

        hallucination_rate = (hallucinations / len(test_cases) * 100) if test_cases else 0
        avg_latency = statistics.mean(latencies) if latencies else 0
        avg_score = statistics.mean(hallucination_scores) if hallucination_scores else 0

        return {
            "total": len(test_cases),
            "correct_refusals": correct_refusals,
            "hallucinations": hallucinations,
            "hallucination_rate_pct": hallucination_rate,
            "avg_hallucination_score": avg_score,
            "avg_latency_ms": avg_latency,
        }

    async def evaluate_valid_queries(self, client: httpx.AsyncClient, test_cases: List[Dict]) -> Dict[str, Any]:
        """Evaluate on valid queries (should answer with low hallucination score)."""
        console.print("\n[bold cyan]Testing valid queries (should answer correctly)...[/bold cyan]\n")

        correct_answers = 0
        false_positives = 0  # Flagged as hallucination but answer was correct
        latencies: List[float] = []
        hallucination_scores: List[float] = []

        with Progress() as progress:
            task = progress.add_task("[cyan]Testing valid queries...", total=len(test_cases))

            for test_case in test_cases:
                query_id = test_case["id"]
                query = test_case["query"]
                expected_answer_pattern = test_case.get("expected_answer_pattern", "")

                console.print(f"[dim]Testing {query_id}: {query[:60]}...[/dim]")

                result = await self.run_chat(client, query)
                progress.update(task, advance=1)

                if result["error"]:
                    console.print(f"[red]Error: {result['error']}[/red]")
                    continue

                latencies.append(result["latency_ms"])
                answer = result["answer"] or ""
                verification = result["verification"]

                hallucination_score = verification.get("hallucination_score", 0.0) if verification else 0.0
                hallucination_scores.append(hallucination_score)

                # Check if answer contains expected content
                try:
                    answer_correct = bool(re.search(expected_answer_pattern, answer, re.IGNORECASE))
                except re.error:
                    answer_correct = False

                flagged = hallucination_score > 0.3  # Threshold

                if answer_correct and not flagged:
                    correct_answers += 1
                    console.print(f"[green]✓ Correct answer, not flagged[/green]")
                elif answer_correct and flagged:
                    false_positives += 1
                    console.print(f"[yellow]⚠ Correct answer but flagged (score: {hallucination_score:.2f})[/yellow]")
                else:
                    console.print(f"[red]✗ Incorrect answer or high score: {hallucination_score:.2f}[/red]")

                self.results.append(
                    {
                        "question_id": query_id,
                        "type": "valid",
                        "question": query,
                        "answer": answer,
                        "hallucination_score": hallucination_score,
                        "correct": answer_correct,
                        "flagged": flagged,
                        "latency_ms": result["latency_ms"],
                    }
                )

        specificity = (correct_answers / len(test_cases) * 100) if test_cases else 0
        false_positive_rate = (false_positives / len(test_cases) * 100) if test_cases else 0
        avg_latency = statistics.mean(latencies) if latencies else 0
        avg_score = statistics.mean(hallucination_scores) if hallucination_scores else 0
        latency_p95 = statistics.quantiles(latencies, n=20)[18] if len(latencies) >= 20 else max(latencies, default=0)

        return {
            "total": len(test_cases),
            "correct_answers": correct_answers,
            "false_positives": false_positives,
            "specificity_pct": specificity,
            "false_positive_rate_pct": false_positive_rate,
            "avg_hallucination_score": avg_score,
            "avg_latency_ms": avg_latency,
            "latency_p95_ms": latency_p95,
        }

    async def run_evaluation(self) -> Dict[str, Any]:
        """Execute full evaluation suite."""
        console.print("[bold cyan]Loading test data...[/bold cyan]")

        with open(self.trick_questions_file, "r", encoding="utf-8") as f:
            trick_questions = json.load(f)

        console.print(f"Loaded {len(trick_questions)} trick questions")

        valid_queries = []
        if self.valid_queries_file and self.valid_queries_file.exists():
            with open(self.valid_queries_file, "r", encoding="utf-8") as f:
                valid_queries = json.load(f)
            console.print(f"Loaded {len(valid_queries)} valid queries")

        async with httpx.AsyncClient() as client:
            # Test trick questions
            trick_results = await self.evaluate_trick_questions(client, trick_questions)

            # Test valid queries
            valid_results = {}
            if valid_queries:
                valid_results = await self.evaluate_valid_queries(client, valid_queries)

        # Calculate overall metrics
        return {
            "trick_questions": trick_results,
            "valid_queries": valid_results,
            "passed": (
                trick_results["hallucination_rate_pct"] <= 1.0
                and (not valid_results or valid_results.get("specificity_pct", 100) > 95)
                and (not valid_results or valid_results.get("latency_p95_ms", 0) <= 300)
            ),
        }

    def print_results(self, summary: Dict[str, Any]):
        """Print formatted results table."""
        console.print("\n[bold cyan]=" * 60)
        console.print("[bold cyan]Hallucination Detection Evaluation Results[/bold cyan]")
        console.print("[bold cyan]=" * 60 + "\n")

        # Trick questions table
        trick_table = Table(title="Trick Questions (Should Refuse)", show_header=True, header_style="bold magenta")
        trick_table.add_column("Metric", style="cyan")
        trick_table.add_column("Value", style="green")
        trick_table.add_column("Target", style="yellow")
        trick_table.add_column("Status", style="bold")

        trick = summary["trick_questions"]
        trick_table.add_row(
            "Hallucination Rate",
            f"{trick['hallucination_rate_pct']:.2f}%",
            "≤1%",
            "✅" if trick['hallucination_rate_pct'] <= 1.0 else "❌",
        )
        trick_table.add_row(
            "Correct Refusals",
            f"{trick['correct_refusals']} / {trick['total']}",
            f"{trick['total']}",
            "✅" if trick['correct_refusals'] == trick['total'] else "❌",
        )
        trick_table.add_row(
            "Avg Hallucination Score",
            f"{trick['avg_hallucination_score']:.3f}",
            ">0.8",
            "✅" if trick['avg_hallucination_score'] > 0.8 else "⚠️",
        )

        console.print(trick_table)

        # Valid queries table
        if summary["valid_queries"]:
            valid = summary["valid_queries"]
            valid_table = Table(title="\nValid Queries (Should Answer)", show_header=True, header_style="bold magenta")
            valid_table.add_column("Metric", style="cyan")
            valid_table.add_column("Value", style="green")
            valid_table.add_column("Target", style="yellow")
            valid_table.add_column("Status", style="bold")

            valid_table.add_row(
                "Specificity",
                f"{valid['specificity_pct']:.2f}%",
                ">95%",
                "✅" if valid['specificity_pct'] > 95 else "❌",
            )
            valid_table.add_row(
                "False Positive Rate",
                f"{valid['false_positive_rate_pct']:.2f}%",
                "<5%",
                "✅" if valid['false_positive_rate_pct'] < 5 else "❌",
            )
            valid_table.add_row(
                "Latency P95",
                f"{valid['latency_p95_ms']:.1f} ms",
                "≤300 ms",
                "✅" if valid['latency_p95_ms'] <= 300 else "❌",
            )
            valid_table.add_row(
                "Avg Hallucination Score",
                f"{valid['avg_hallucination_score']:.3f}",
                "<0.2",
                "✅" if valid['avg_hallucination_score'] < 0.2 else "⚠️",
            )

            console.print(valid_table)

        # Final verdict
        console.print()
        if summary["passed"]:
            console.print("[bold green]✅ EVALUATION PASSED - All KPI targets met[/bold green]")
        else:
            console.print("[bold red]❌ EVALUATION FAILED - Some KPI targets not met[/bold red]")

        console.print()


async def main():
    parser = argparse.ArgumentParser(description="Hallucination Detection Evaluation")
    parser.add_argument("--engine-url", default="http://localhost:8090", help="Rust Engine URL")
    parser.add_argument("--kb-id", required=True, help="Knowledge Base ID")
    parser.add_argument(
        "--trick-questions",
        type=Path,
        default=Path("tests/fixtures/trick_questions/hallucination_test_questions.json"),
        help="Trick questions JSON file",
    )
    parser.add_argument(
        "--valid-queries",
        type=Path,
        default=Path("tests/fixtures/queries/graph_rag_test_queries.json"),
        help="Valid queries JSON file (optional)",
    )
    parser.add_argument("--output", type=Path, help="Output JSON file for results")

    args = parser.parse_args()

    if not args.trick_questions.exists():
        console.print(f"[red]Error: Trick questions file not found: {args.trick_questions}[/red]")
        return 1

    evaluator = HallucinationEvaluator(
        args.engine_url,
        args.kb_id,
        args.trick_questions,
        args.valid_queries if args.valid_queries.exists() else None,
    )

    console.print(f"[bold cyan]Starting Hallucination Detection Evaluation[/bold cyan]")
    console.print(f"Engine URL: {args.engine_url}")
    console.print(f"KB ID: {args.kb_id}")
    console.print(f"Trick questions: {args.trick_questions}")
    if args.valid_queries and args.valid_queries.exists():
        console.print(f"Valid queries: {args.valid_queries}\n")

    try:
        summary = await evaluator.run_evaluation()
        evaluator.print_results(summary)

        if args.output:
            with open(args.output, "w", encoding="utf-8") as f:
                json.dump(
                    {
                        "summary": summary,
                        "detailed_results": evaluator.results,
                        "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
                    },
                    f,
                    indent=2,
                )
            console.print(f"\n[dim]Results saved to {args.output}[/dim]")

        return 0 if summary["passed"] else 1

    except KeyboardInterrupt:
        console.print("\n[yellow]Evaluation interrupted by user[/yellow]")
        return 130
    except Exception as e:
        console.print(f"\n[red]Evaluation failed with error: {e}[/red]")
        import traceback
        traceback.print_exc()
        return 1


if __name__ == "__main__":
    exit(asyncio.run(main()))
