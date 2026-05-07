"""Hallucination Evaluation - Zero-hallucination verification"""
import asyncio
import json
from pathlib import Path
from datetime import datetime
import httpx
from rich.console import Console

console = Console()
API_BASE_URL = "http://localhost:9080/api"
KB_ID = "bench_kb_hallucination"
REPORT_DIR = Path(__file__).parent / "reports"

# 30 trick questions (info NOT in docs)
TRICK_QUESTIONS = [
    "Qual è il budget previsto per il marketing digitale?",
    "Chi è il CEO dell'azienda fornitrice?",
    "Quanti dipendenti ha l'azienda committente?",
    "Qual è la strategia di espansione internazionale?",
    # ... add 26 more
]

# 30 valid questions with gold answers
VALID_QUESTIONS = [
    {"q": "Quali sono le penali?", "gold": "€50.000 per NDA"},
    {"q": "Termine consegna?", "gold": "12 mesi dalla firma"},
    # ... add 28 more
]

async def test_trick_question(client, question):
    try:
        resp = await client.post(f"{API_BASE_URL}/query", 
                                 json={"kb_id": KB_ID, "query": question},
                                 timeout=20.0)
        resp.raise_for_status()
        answer = resp.json().get('answer', '')
        # Should contain "non sono presenti" or similar
        is_refused = any(phrase in answer.lower() for phrase in 
                        ['non sono presenti', 'non disponibile', 'non trovato'])
        return {'question': question, 'refused': is_refused, 'answer': answer}
    except Exception as e:
        return {'question': question, 'error': str(e)}

async def main():
    REPORT_DIR.mkdir(exist_ok=True)
    console.print("[cyan]Hallucination Evaluation[/cyan]\n")
    
    async with httpx.AsyncClient() as client:
        # Test trick questions
        console.print("Testing trick questions...")
        trick_results = await asyncio.gather(*[test_trick_question(client, q) for q in TRICK_QUESTIONS[:5]])
        
        refused_count = sum(1 for r in trick_results if r.get('refused', False))
        hallucination_rate = (1 - refused_count / len(trick_results)) * 100
        
        console.print(f"\nRefused: {refused_count}/{len(trick_results)}")
        console.print(f"Hallucination Rate: {hallucination_rate:.1f}%")
        
        # Save report
        timestamp = datetime.now().strftime('%Y%m%d_%H%M')
        report_path = REPORT_DIR / f"hallucination_{timestamp}.md"
        with open(report_path, 'w') as f:
            f.write(f"# Hallucination Evaluation\n\n")
            f.write(f"**Hallucination Rate**: {hallucination_rate:.1f}%  \n")
            f.write(f"**Target**: <1%  \n")
            status = "✅" if hallucination_rate < 1 else "❌"
            f.write(f"**Status**: {status}\n")
        
        console.print(f"[green]Report saved:[/green] {report_path}")

if __name__ == "__main__":
    asyncio.run(main())
