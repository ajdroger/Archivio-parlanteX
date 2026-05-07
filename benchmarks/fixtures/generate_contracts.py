"""
Generate Fake PDF Contracts for Benchmarking

Genera 50 PDF fittizi di contratti italiani per testing ingestion pipeline.
Usa reportlab per creare PDF realistici con testo, tabelle, clausole legali.
"""

import argparse
from pathlib import Path
from datetime import datetime, timedelta
import random

from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib.units import cm
from reportlab.platypus import SimpleDocTemplate, Paragraph, Spacer, Table, TableStyle, PageBreak
from reportlab.lib import colors


# Template contratti italiani
CONTRACT_TEMPLATES = {
    "NDA": {
        "title": "ACCORDO DI NON DIVULGAZIONE",
        "clauses": [
            "1. Definizioni: Per 'Informazioni Riservate' si intendono...",
            "2. Obblighi di Riservatezza: Il Ricevente si impegna a...",
            "3. Durata: Il presente accordo avrà validità di 5 anni...",
            "4. Penali: In caso di violazione, penale pari a €50.000...",
            "5. Foro Competente: Milano, Italia.",
        ]
    },
    "APPALTO": {
        "title": "CONTRATTO DI APPALTO SERVIZI IT",
        "clauses": [
            "1. Oggetto: Sviluppo piattaforma web enterprise...",
            "2. Corrispettivo: €250.000 + IVA, pagabili in 4 milestone...",
            "3. Tempistiche: Consegna entro 12 mesi dalla firma...",
            "4. Penali Ritardo: €500/giorno oltre scadenza milestone...",
            "5. Garanzie: 24 mesi post-collaudo su difetti software...",
        ]
    },
    "FORNITURA": {
        "title": "CONTRATTO DI FORNITURA HARDWARE",
        "clauses": [
            "1. Beni: N.50 workstation HP EliteDesk con monitor 27\"...",
            "2. Prezzo: €75.000 + IVA, pagamento 60 giorni data fattura...",
            "3. Consegna: Franco magazzino acquirente, termine 30 giorni...",
            "4. Garanzia: 36 mesi del produttore, sostituzione entro 48h...",
            "5. Clausola Risolutiva: Ritardo >15 giorni risoluzione diritto...",
        ]
    },
}


def generate_contract_pdf(output_path: Path, template_key: str, doc_id: int):
    """Genera singolo PDF contratto"""
    doc = SimpleDocTemplate(str(output_path), pagesize=A4,
                            rightMargin=2*cm, leftMargin=2*cm,
                            topMargin=2*cm, bottomMargin=2*cm)
    
    styles = getSampleStyleSheet()
    title_style = ParagraphStyle(
        'CustomTitle',
        parent=styles['Heading1'],
        fontSize=16,
        textColor=colors.HexColor('#1a1a1a'),
        spaceAfter=30,
        alignment=1  # center
    )
    
    story = []
    template = CONTRACT_TEMPLATES[template_key]
    
    # Titolo
    story.append(Paragraph(template["title"], title_style))
    story.append(Spacer(1, 0.5*cm))
    
    # Intestazione
    story.append(Paragraph(f"<b>Contratto N.</b> {doc_id:04d}/2024", styles['Normal']))
    story.append(Paragraph(f"<b>Data:</b> {datetime.now().strftime('%d/%m/%Y')}", styles['Normal']))
    story.append(Spacer(1, 0.5*cm))
    
    # Parti
    parties_data = [
        ['COMMITTENTE', 'FORNITORE'],
        ['Acme Corporation S.p.A.', f'Tech Solutions {doc_id} S.r.l.'],
        ['P.IVA: 12345678901', f'P.IVA: {random.randint(10000000000, 99999999999)}'],
        ['Via Roma 1, Milano', f'Via Verdi {doc_id}, Roma'],
    ]
    parties_table = Table(parties_data, colWidths=[8*cm, 8*cm])
    parties_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), colors.grey),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.whitesmoke),
        ('ALIGN', (0, 0), (-1, -1), 'CENTER'),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, 0), 12),
        ('BOTTOMPADDING', (0, 0), (-1, 0), 12),
        ('GRID', (0, 0), (-1, -1), 1, colors.black)
    ]))
    story.append(parties_table)
    story.append(Spacer(1, 1*cm))
    
    # Clausole
    story.append(Paragraph("<b>CLAUSOLE</b>", styles['Heading2']))
    for clause in template["clauses"]:
        story.append(Paragraph(clause, styles['Normal']))
        story.append(Spacer(1, 0.3*cm))
    
    # Aggiungi padding per simulare contratti più lunghi
    for page in range(random.randint(3, 8)):
        story.append(Spacer(1, 2*cm))
        story.append(Paragraph(
            f"Articolo Aggiuntivo {page+6}: Lorem ipsum dolor sit amet, consectetur adipiscing elit. "
            "Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. "
            "Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris.",
            styles['Normal']
        ))
        if page % 2 == 0:
            story.append(PageBreak())
    
    # Firma
    story.append(Spacer(1, 2*cm))
    story.append(Paragraph("<b>Firme:</b>", styles['Heading3']))
    story.append(Spacer(1, 1*cm))
    sig_data = [
        ['_____________________', '_____________________'],
        ['Committente', 'Fornitore'],
    ]
    sig_table = Table(sig_data, colWidths=[8*cm, 8*cm])
    story.append(sig_table)
    
    doc.build(story)


def main():
    parser = argparse.ArgumentParser(description='Generate fake contract PDFs')
    parser.add_argument('--count', type=int, default=50, help='Number of PDFs to generate')
    parser.add_argument('--output', type=str, default='contracts', help='Output directory')
    args = parser.parse_args()
    
    output_dir = Path(__file__).parent / args.output
    output_dir.mkdir(exist_ok=True)
    
    templates = list(CONTRACT_TEMPLATES.keys())
    
    print(f"Generating {args.count} fake contract PDFs...")
    for i in range(args.count):
        template = templates[i % len(templates)]
        filename = f"contract_{i+1:03d}_{template.lower()}.pdf"
        output_path = output_dir / filename
        
        generate_contract_pdf(output_path, template, i+1)
        
        if (i+1) % 10 == 0:
            print(f"  Generated {i+1}/{args.count}")
    
    print(f"✓ Generated {args.count} PDFs in {output_dir}")
    print(f"  Total size: {sum(f.stat().st_size for f in output_dir.glob('*.pdf')) / 1024 / 1024:.1f} MB")


if __name__ == "__main__":
    main()
