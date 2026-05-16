import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { User, Bot } from 'lucide-react';
import type { SearchResult } from '../../types';

interface ChatMessageProps {
  message: string;
  role: 'user' | 'assistant';
  sources?: SearchResult[];
}

/**
 * Chat message component with markdown rendering and role-based styling.
 * User messages appear on the right, assistant messages on the left.
 */
export default function ChatMessage({
  message,
  role,
}: ChatMessageProps) {
  const isUser = role === 'user';

  return (
    <div className={`flex gap-3 ${isUser ? 'flex-row-reverse' : 'flex-row'}`}>
      {/* Avatar */}
      <div
        className={`flex-shrink-0 w-8 h-8 rounded-full flex items-center justify-center ${
          isUser
            ? 'bg-primary/20 text-primary'
            : 'bg-dark-100 border border-dark-50 text-gray-400'
        }`}
      >
        {isUser ? (
          <User className="w-5 h-5" />
        ) : (
          <Bot className="w-5 h-5" />
        )}
      </div>

      {/* Message content */}
      <div
        className={`flex-1 max-w-3xl rounded-lg px-4 py-3 ${
          isUser
            ? 'bg-primary/10 border border-primary/30 text-gray-100'
            : 'bg-dark-100 border border-dark-50 text-gray-200'
        }`}
      >
        <div className="prose prose-invert prose-sm max-w-none">
          <ReactMarkdown
            remarkPlugins={[remarkGfm]}
            components={{
              // Custom link styling for citations
              a: ({ node, ...props }) => (
                <a
                  {...props}
                  className="text-primary hover:text-primary-400 underline cursor-pointer"
                  target={props.href?.startsWith('#') ? undefined : '_blank'}
                  rel={
                    props.href?.startsWith('#') ? undefined : 'noopener noreferrer'
                  }
                />
              ),
              // Code blocks with dark theme
              code: ({ node, ...props }) => {
                const isInline = !props.className?.includes('language-');
                return isInline ? (
                  <code
                    {...props}
                    className="bg-dark-200 px-1.5 py-0.5 rounded text-primary font-mono text-sm"
                  />
                ) : (
                  <code
                    {...props}
                    className="block bg-dark-200 p-3 rounded-lg overflow-x-auto font-mono text-sm"
                  />
                );
              },
            }}
          >
            {message}
          </ReactMarkdown>
        </div>
      </div>
    </div>
  );
}
