import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import ChatMessage from './ChatMessage';

describe('ChatMessage', () => {
  it('renders user message with correct styling', () => {
    const { container } = render(
      <ChatMessage
        message="Hello, this is a test"
        role="user"
      />
    );

    const messageElement = screen.getByText('Hello, this is a test');
    expect(messageElement).toBeInTheDocument();

    // User messages should have flex-row-reverse on the outer container
    const outerDiv = container.querySelector('.flex-row-reverse');
    expect(outerDiv).toBeInTheDocument();
  });

  it('renders assistant message with correct styling', () => {
    const { container } = render(
      <ChatMessage
        message="I am Claude, how can I help?"
        role="assistant"
      />
    );

    const messageElement = screen.getByText('I am Claude, how can I help?');
    expect(messageElement).toBeInTheDocument();

    // Assistant messages should have flex-row, not flex-row-reverse
    const outerDiv = container.querySelector('.flex-row');
    expect(outerDiv).toBeInTheDocument();
    expect(outerDiv?.className).not.toContain('flex-row-reverse');
  });

  it('renders markdown content correctly', () => {
    render(
      <ChatMessage
        message="This is **bold** and *italic*"
        role="assistant"
      />
    );

    // Check that markdown is rendered as HTML
    const boldElement = screen.getByText('bold');
    expect(boldElement.tagName).toBe('STRONG');

    const italicElement = screen.getByText('italic');
    expect(italicElement.tagName).toBe('EM');
  });

  it('renders code blocks correctly', () => {
    const codeMessage = '```javascript\nconst x = 42;\n```';
    render(
      <ChatMessage
        message={codeMessage}
        role="assistant"
      />
    );

    const codeElement = screen.getByText(/const x = 42/);
    expect(codeElement).toBeInTheDocument();
    expect(codeElement.tagName).toBe('CODE');
  });

  it('displays user avatar icon', () => {
    const { container } = render(
      <ChatMessage
        message="Test"
        role="user"
      />
    );

    // Should have User icon (lucide-user class)
    const userIcon = container.querySelector('.lucide-user');
    expect(userIcon).toBeInTheDocument();
  });

  it('displays assistant avatar icon', () => {
    const { container } = render(
      <ChatMessage
        message="Test"
        role="assistant"
      />
    );

    // Should have Bot icon (lucide-bot class)
    const botIcon = container.querySelector('.lucide-bot');
    expect(botIcon).toBeInTheDocument();
  });
});
