import { useState } from 'react';
import { useAppStore } from '../store/appStore';
import api from '../lib/api';
import type { QueryResponse } from '../types';
import { Send, Loader2 } from 'lucide-react';
import ChatMessage from '../components/chat/ChatMessage';
import ContextViewer from '../components/chat/ContextViewer';
import ModelSelector from '../components/settings/ModelSelector';

export default function DashboardPage() {
  const { currentKb } = useAppStore();
  const [query, setQuery] = useState('');
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<QueryResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!query.trim() || !currentKb) return;

    setLoading(true);
    setError(null);

    try {
      const response = await api.query({
        kb_id: currentKb.id,
        query: query.trim(),
        top_k: 5,
        rerank_top_n: 3,
      });
      setResult(response);
    } catch (err: any) {
      setError(err.response?.data?.message || 'Errore durante la query');
    } finally {
      setLoading(false);
    }
  };

  if (!currentKb) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <p className="text-gray-400 text-lg">
            Nessuna Knowledge Base selezionata
          </p>
          <p className="text-gray-500 text-sm mt-2">
            Seleziona o crea una KB per iniziare
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-screen">
      {/* Header */}
      <header className="border-b border-dark-50 bg-dark-100 px-6 py-4">
        <div className="flex items-start justify-between gap-4">
          <div className="flex-1">
            <h2 className="text-2xl font-semibold text-gray-100">
              💼 Chat RAG - {currentKb.name}
            </h2>
            <p className="text-sm text-gray-400 mt-1">
              Interroga i tuoi contratti con precisione forensica
            </p>
          </div>
          <div className="flex-shrink-0">
            <ModelSelector />
          </div>
        </div>
      </header>

      {/* Messages Area */}
      <div className="flex-1 overflow-auto p-6 space-y-6">
        {query && result && (
          <div className="space-y-6">
            {/* User query */}
            <ChatMessage message={query} role="user" />

            {/* Assistant answer */}
            <ChatMessage
              message={result.answer}
              role="assistant"
              sources={result.sources}
            />

            {/* Context (sources + verification) */}
            <ContextViewer
              sources={result.sources}
              verified={result.verified}
              informationGaps={result.information_gaps}
            />

            {/* Processing time */}
            <p className="text-xs text-gray-500 text-center">
              Elaborato in {result.processing_time_ms}ms
            </p>
          </div>
        )}

        {error && (
          <div className="bg-red-900/30 border border-red-700 rounded-lg p-4 text-red-300">
            {error}
          </div>
        )}
      </div>

      {/* Input Form */}
      <div className="border-t border-dark-50 bg-dark-100 p-4">
        <form onSubmit={handleSubmit} className="flex gap-2">
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Scrivi la tua domanda sui contratti..."
            disabled={loading}
            className="flex-1 px-4 py-3 bg-dark-200 border border-dark-50 rounded-lg text-gray-100 placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-colors disabled:opacity-50"
          />
          <button
            type="submit"
            disabled={loading || !query.trim()}
            className="px-6 py-3 bg-primary text-dark-300 font-medium rounded-lg hover:bg-primary-400 transition-colors focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2 focus:ring-offset-dark disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
          >
            {loading ? (
              <>
                <Loader2 className="w-5 h-5 animate-spin" />
                <span>Elaborazione...</span>
              </>
            ) : (
              <>
                <Send className="w-5 h-5" />
                <span>Invia</span>
              </>
            )}
          </button>
        </form>
      </div>
    </div>
  );
}
