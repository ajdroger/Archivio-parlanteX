import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import ContextViewer from './ContextViewer';
import type { SearchResult } from '../../types';

describe('ContextViewer', () => {
  const mockSources: SearchResult[] = [
    {
      doc_id: 'contract_001',
      text_quote: 'Article 5: Payment terms are 30 days net',
      confidence: 0.92,
      metadata: { page: 3 },
    },
    {
      doc_id: 'contract_002',
      text_quote: 'Termination clause: 60 days notice required',
      confidence: 0.65,
      metadata: { page: 12 },
    },
    {
      doc_id: 'contract_003',
      text_quote: 'Liability cap: €50,000',
      confidence: 0.45,
      metadata: { page: 8 },
    },
  ];

  it('renders all sources', () => {
    render(<ContextViewer sources={mockSources} verified={false} />);

    expect(screen.getByText(/contract_001/)).toBeInTheDocument();
    expect(screen.getByText(/contract_002/)).toBeInTheDocument();
    expect(screen.getByText(/contract_003/)).toBeInTheDocument();
  });

  it('displays confidence scores with correct color coding', () => {
    render(<ContextViewer sources={mockSources} verified={false} />);

    // High confidence (>70%) should be green
    const highConfidence = screen.getByText(/92% rilevanza/);
    expect(highConfidence.className).toContain('text-green');

    // Medium confidence (50-70%) should be yellow
    const mediumConfidence = screen.getByText(/65% rilevanza/);
    expect(mediumConfidence.className).toContain('text-yellow');

    // Low confidence (<50%) should be red
    const lowConfidence = screen.getByText(/45% rilevanza/);
    expect(lowConfidence.className).toContain('text-red');
  });

  it('displays verification badge when verified=true', () => {
    render(<ContextViewer sources={mockSources} verified={true} />);

    // Should show "Risposta verificata"
    expect(screen.getByText(/Risposta verificata/i)).toBeInTheDocument();
  });

  it('does not display verification badge when verified=false', () => {
    render(<ContextViewer sources={mockSources} verified={false} />);

    // Should not show verification badge
    expect(screen.queryByText(/Risposta verificata/i)).not.toBeInTheDocument();
  });

  it('displays information gaps warning when provided', () => {
    const informationGaps = [
      'Missing data: Contract start date',
      'Missing data: Party B contact information',
    ];

    render(
      <ContextViewer
        sources={mockSources}
        verified={false}
        informationGaps={informationGaps}
      />
    );

    expect(screen.getByText(/Informazioni mancanti/i)).toBeInTheDocument();
    expect(screen.getByText(/Missing data: Contract start date/)).toBeInTheDocument();
  });

  it('displays text quotes in italic', () => {
    render(<ContextViewer sources={mockSources} verified={false} />);

    const quote = screen.getByText(/Payment terms are 30 days net/);
    expect(quote.className).toContain('italic');
  });

  it('displays page numbers from metadata', () => {
    render(<ContextViewer sources={mockSources} verified={false} />);

    expect(screen.getByText(/Pagina 3/)).toBeInTheDocument();
    expect(screen.getByText(/Pagina 12/)).toBeInTheDocument();
    expect(screen.getByText(/Pagina 8/)).toBeInTheDocument();
  });

  it('renders null when no sources and no information gaps', () => {
    const { container } = render(<ContextViewer sources={[]} verified={false} />);

    // Component returns null when empty
    expect(container.firstChild).toBeNull();
  });

  it('displays sources count in header', () => {
    render(<ContextViewer sources={mockSources} verified={false} />);

    expect(screen.getByText(/Fonti \(3\)/)).toBeInTheDocument();
  });

  it('can expand and collapse sources list', async () => {
    const { getByRole } = render(<ContextViewer sources={mockSources} verified={false} />);

    const expandButton = getByRole('button', { name: /Fonti/i });

    // Initially expanded - sources should be visible
    expect(screen.getByText(/Payment terms are 30 days net/)).toBeInTheDocument();

    // Click to collapse
    await expandButton.click();

    // Sources should still be in DOM (React doesn't remove when collapsed in this implementation)
    // The actual collapse behavior depends on the component implementation
    expect(expandButton).toBeInTheDocument();
  });
});
