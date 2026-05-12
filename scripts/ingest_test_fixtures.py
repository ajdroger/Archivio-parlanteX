#!/usr/bin/env python3
"""
Ingest Test Fixtures Script

Creates a test knowledge base and ingests sample contracts for integration testing.

Usage:
    python scripts/ingest_test_fixtures.py --kb-id fase6_test_kb --engine-url http://localhost:8090
"""

import argparse
import asyncio
import os
import shutil
import uuid
from pathlib import Path
import httpx
from rich.console import Console
from rich.progress import Progress

console = Console()


async def ingest_contract(
    client: httpx.AsyncClient,
    engine_url: str,
    kb_id: str,
    contract_file: Path,
    internal_token: str = None,
) -> bool:
    """Ingest single contract file."""
    console.print(f"[cyan]Ingesting {contract_file.name}...[/cyan]")

    # Ensure shared/uploads exists
    shared_uploads_dir = Path("shared/uploads")
    shared_uploads_dir.mkdir(parents=True, exist_ok=True)

    # Copy file to shared volume
    dest_path = shared_uploads_dir / contract_file.name
    shutil.copy2(contract_file, dest_path)

    doc_id = str(uuid.uuid4())
    file_path = f"/shared/uploads/{contract_file.name}"

    headers = {}
    if internal_token:
        headers["X-Internal-Token"] = internal_token

    try:
        response = await client.post(
            f"{engine_url}/ingest",
            json={
                "doc_id": doc_id,
                "kb_id": kb_id,
                "file_path": file_path,
                "source_name": contract_file.name,
                "mime_type": "text/plain",
                "tags": ["test_fixture", "sample"],
            },
            headers=headers,
            timeout=120.0,  # Long timeout for processing
        )

        if response.status_code == 200:
            console.print(f"[green]✓ {contract_file.name} ingested successfully[/green]")
            return True
        else:
            console.print(f"[red]✗ Failed to ingest {contract_file.name}: HTTP {response.status_code}[/red]")
            console.print(f"[dim]{response.text}[/dim]")
            return False

    except Exception as e:
        console.print(f"[red]✗ Error ingesting {contract_file.name}: {e}[/red]")
        return False


async def main():
    parser = argparse.ArgumentParser(description="Ingest test fixtures for integration testing")
    parser.add_argument("--engine-url", default="http://localhost:8090", help="Rust Engine URL")
    parser.add_argument("--kb-id", default="fase6_test_kb", help="Knowledge Base ID to create")
    parser.add_argument(
        "--contracts-dir",
        type=Path,
        default=Path("tests/fixtures/contracts"),
        help="Directory containing contract fixtures",
    )

    args = parser.parse_args()

    if not args.contracts_dir.exists():
        console.print(f"[red]Error: Contracts directory not found: {args.contracts_dir}[/red]")
        return 1

    # Find all contract files
    contract_files = list(args.contracts_dir.glob("*.txt"))

    if not contract_files:
        console.print(f"[red]Error: No .txt files found in {args.contracts_dir}[/red]")
        return 1

    console.print(f"\n[bold cyan]Ingest Test Fixtures[/bold cyan]")
    console.print(f"Engine URL: {args.engine_url}")
    console.print(f"KB ID: {args.kb_id}")
    console.print(f"Contracts: {len(contract_files)} files\n")

    # Get internal token from environment
    internal_token = os.getenv("RUST_ENGINE_INTERNAL_TOKEN")
    if not internal_token:
        console.print("[yellow]Warning: RUST_ENGINE_INTERNAL_TOKEN not set, ingest may fail with 401[/yellow]")

    async with httpx.AsyncClient() as client:
        # Check engine health
        try:
            health_response = await client.get(f"{args.engine_url}/health", timeout=5.0)
            if health_response.status_code != 200:
                console.print("[red]Error: Engine health check failed[/red]")
                return 1
            console.print("[green]✓ Engine is healthy[/green]\n")
        except Exception as e:
            console.print(f"[red]Error: Cannot connect to engine: {e}[/red]")
            return 1

        # Ingest all contracts
        with Progress() as progress:
            task = progress.add_task("[cyan]Ingesting contracts...", total=len(contract_files))

            results = []
            for contract_file in contract_files:
                success = await ingest_contract(client, args.engine_url, args.kb_id, contract_file, internal_token)
                results.append(success)
                progress.update(task, advance=1)

        # Summary
        successful = sum(results)
        failed = len(results) - successful

        console.print()
        if failed == 0:
            console.print(f"[bold green]✅ All {successful} contracts ingested successfully![/bold green]")
            console.print(f"\n[dim]KB ID '{args.kb_id}' is ready for testing[/dim]")
            return 0
        else:
            console.print(f"[bold yellow]⚠ {successful} successful, {failed} failed[/bold yellow]")
            return 1


if __name__ == "__main__":
    exit(asyncio.run(main()))
