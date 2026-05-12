#!/usr/bin/env python3
"""
Direct Ingestion Script - Bypasses Rust /ingest endpoint

Calls Python Worker directly and stores in Qdrant + MySQL.
Workaround for Rust Handler trait compilation issue.
"""

import asyncio
import hashlib
import json
import os
import sys
import uuid
from pathlib import Path
from typing import List

import httpx
from qdrant_client import QdrantClient
from qdrant_client.models import Distance, PointStruct, VectorParams, SparseVector, SparseVectorParams
from rich.console import Console
from rich.progress import Progress
import pymysql

console = Console()

# Configuration
ENGINE_URL = os.getenv("RUST_ENGINE_URL", "http://localhost:8090")
PYTHON_WORKER_URL = os.getenv("PYTHON_WORKER_URL", "http://localhost:8091")
QDRANT_URL = os.getenv("QDRANT_URL", "http://localhost:6335")
OLLAMA_URL = os.getenv("OLLAMA_URL", "http://localhost:11434")
OLLAMA_MODEL_EMBED = os.getenv("OLLAMA_MODEL_EMBED", "nomic-embed-text")

MYSQL_HOST = os.getenv("MYSQL_HOST", "localhost")
MYSQL_PORT = int(os.getenv("MYSQL_PORT", "3307"))
MYSQL_USER = os.getenv("MYSQL_USER", "root")
MYSQL_PASSWORD = os.getenv("MYSQL_PASSWORD", "devpass123")
MYSQL_DB = os.getenv("MYSQL_DB", "archivio_parlante_x")


async def parse_document(client: httpx.AsyncClient, file_path: str, doc_id: str):
    """Call Python Worker to parse document."""
    console.print(f"[cyan]Parsing document: {file_path}[/cyan]")

    response = await client.post(
        f"{PYTHON_WORKER_URL}/parse",
        json={"file_path": file_path, "doc_id": doc_id},
        timeout=120.0,
    )
    response.raise_for_status()
    return response.json()


async def generate_embeddings(client: httpx.AsyncClient, texts: List[str]):
    """Generate embeddings via Ollama."""
    console.print(f"[cyan]Generating embeddings for {len(texts)} chunks...[/cyan]")

    all_embeddings = []

    for text in texts:
        response = await client.post(
            f"{OLLAMA_URL}/api/embed",
            json={"model": OLLAMA_MODEL_EMBED, "input": text},
            timeout=60.0,
        )
        response.raise_for_status()

        result = response.json()
        embedding = result["embeddings"][0] if "embeddings" in result else result["embedding"]
        all_embeddings.append(embedding)

    console.print(f"[green]✓ Generated {len(all_embeddings)} embeddings[/green]")
    return all_embeddings


def simple_chunk_text(text: str, chunk_size: int = 800, overlap: int = 120) -> List[str]:
    """Simple character-based chunking."""
    chunks = []
    start = 0

    while start < len(text):
        end = start + chunk_size
        chunk = text[start:end]

        if chunk.strip():
            chunks.append(chunk)

        start += (chunk_size - overlap)

    return chunks


async def store_in_qdrant(qdrant: QdrantClient, kb_id: str, doc_id: str, chunks: List[str], embeddings: List[List[float]]):
    """Store chunks and embeddings in Qdrant."""
    console.print(f"[cyan]Storing {len(chunks)} chunks in Qdrant collection: kb_{kb_id}[/cyan]")

    collection_name = f"kb_{kb_id}"

    # Create collection if not exists
    try:
        qdrant.get_collection(collection_name)
    except:
        qdrant.create_collection(
            collection_name=collection_name,
            vectors_config=VectorParams(size=768, distance=Distance.COSINE),
            sparse_vectors_config={"text": SparseVectorParams()},
        )
        console.print(f"[green]✓ Created collection {collection_name}[/green]")

    # Prepare points
    points = []
    for idx, (chunk, embedding) in enumerate(zip(chunks, embeddings)):
        # Generate deterministic UUID from doc_id and chunk index
        point_id = str(uuid.uuid5(uuid.NAMESPACE_DNS, f"{doc_id}_{idx}"))

        # Simple BM25-like sparse vector (word frequency)
        words = chunk.lower().split()
        word_freq = {}
        for word in words:
            word_freq[word] = word_freq.get(word, 0) + 1

        sparse_indices = list(range(len(word_freq)))
        sparse_values = list(word_freq.values())

        points.append(PointStruct(
            id=point_id,
            vector={"": embedding, "text": SparseVector(indices=sparse_indices, values=sparse_values)},
            payload={
                "doc_id": doc_id,
                "chunk_id": f"chunk_{idx}",
                "text": chunk,
                "chunk_index": idx,
            }
        ))

    # Upsert points
    qdrant.upsert(collection_name=collection_name, points=points)
    console.print(f"[green]✓ Stored {len(points)} points in Qdrant[/green]")


def store_in_mysql(kb_id: str, doc_id: str, filename: str, file_path: str, file_size: int, num_chunks: int):
    """Store document metadata in MySQL."""
    console.print(f"[cyan]Storing metadata in MySQL...[/cyan]")

    conn = pymysql.connect(
        host=MYSQL_HOST,
        port=MYSQL_PORT,
        user=MYSQL_USER,
        password=MYSQL_PASSWORD,
        database=MYSQL_DB,
    )

    try:
        with conn.cursor() as cursor:
            # Insert or update document
            sql = """
            INSERT INTO ap_documents (id, kb_id, filename, file_path, file_size_bytes, mime_type, status, indexed_at, chunks_count)
            VALUES (%s, %s, %s, %s, %s, %s, %s, NOW(), %s)
            ON DUPLICATE KEY UPDATE
                indexed_at = NOW(),
                chunks_count = %s,
                status = %s
            """
            cursor.execute(sql, (doc_id, kb_id, filename, file_path, file_size, "text/plain", "indexed", num_chunks, num_chunks, "indexed"))

        conn.commit()
        console.print(f"[green]✓ Stored metadata in MySQL[/green]")

    finally:
        conn.close()


async def ingest_document(file_path: str, kb_id: str, doc_id: str):
    """Main ingestion pipeline."""
    console.print(f"\n[bold cyan]{'='*60}[/bold cyan]")
    console.print(f"[bold]Ingesting Document[/bold]")
    console.print(f"File: {file_path}")
    console.print(f"KB ID: {kb_id}")
    console.print(f"Doc ID: {doc_id}")
    console.print(f"[bold cyan]{'='*60}[/bold cyan]\n")

    async with httpx.AsyncClient() as client:
        # Step 1: Parse document (fallback to direct read for .txt files)
        if file_path.endswith('.txt'):
            console.print(f"[cyan]Reading text file directly (no parsing needed)[/cyan]")
            with open(file_path, 'r', encoding='utf-8') as f:
                text = f.read()
            parsed = {"pages": [{"text": text}]}
        else:
            try:
                parsed = await parse_document(client, file_path, doc_id)
            except Exception as e:
                console.print(f"[red]✗ Python Worker parse failed: {e}[/red]")
                console.print(f"[yellow]Falling back to simple text extraction...[/yellow]")

                # Fallback: read file directly if it's text
                with open(file_path, 'r', encoding='utf-8') as f:
                    text = f.read()
                parsed = {"pages": [{"text": text}]}

        # Combine text from all pages
        full_text = "\n\n".join(page["text"] for page in parsed["pages"])
        console.print(f"[green]✓ Extracted {len(full_text)} characters[/green]")

        # Step 2: Chunk text
        chunks = simple_chunk_text(full_text, chunk_size=800, overlap=120)
        console.print(f"[green]✓ Created {len(chunks)} chunks[/green]")

        # Step 3: Generate embeddings
        embeddings = await generate_embeddings(client, chunks)

        # Step 4: Store in Qdrant
        qdrant = QdrantClient(url=QDRANT_URL)
        await store_in_qdrant(qdrant, kb_id, doc_id, chunks, embeddings)

        # Step 5: Store metadata in MySQL
        filename = Path(file_path).name
        file_size = Path(file_path).stat().st_size
        store_in_mysql(kb_id, doc_id, filename, file_path, file_size, len(chunks))

    console.print(f"\n[bold green]✅ Ingestion complete![/bold green]\n")


async def main():
    import argparse

    parser = argparse.ArgumentParser(description="Direct document ingestion bypassing Rust endpoint")
    parser.add_argument("file_path", help="Path to document file")
    parser.add_argument("--kb-id", required=True, help="Knowledge base ID")
    parser.add_argument("--doc-id", help="Document ID (auto-generated if not provided)")

    args = parser.parse_args()

    if not args.doc_id:
        args.doc_id = f"doc_{hashlib.sha256(args.file_path.encode()).hexdigest()[:16]}"

    if not Path(args.file_path).exists():
        console.print(f"[red]Error: File not found: {args.file_path}[/red]")
        sys.exit(1)

    await ingest_document(args.file_path, args.kb_id, args.doc_id)


if __name__ == "__main__":
    asyncio.run(main())
