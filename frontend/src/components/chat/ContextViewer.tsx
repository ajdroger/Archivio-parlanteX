import { useState } from 'react';
import { ChevronDown, ChevronUp, CheckCircle, AlertTriangle } from 'lucide-react';
import type { SearchResult } from '../../types';

interface ContextViewerProps {
  sources: SearchResult[];
  verified?: boolean;
  informationGaps?: string[];
}

/**
 * Context viewer component that displays source citations with confidence scores,
 * verification badges, and information gaps warnings.
 */
export default function ContextViewer({
  sources,
  verified = false,
  informationGaps = [],
}: ContextViewerProps) {
  const [isExpanded, setIsExpanded] = useState(true);

  if (sources.length === 0 && informationGaps.length === 0) {
    return null;
  }

  return (
    <div className="mt-4 space-y-3">
      {/* Verification badge */}
      {verified && (
        <div className="flex items-center gap-2 px-3 py-2 bg-green-900/30 border border-green-700 rounded-lg">
          <CheckCircle className="w-4 h-4 text-green-300" />
          <span className="text-sm text-green-300 font-medium">
            Risposta verificata
          </span>
        </div>
      )}

      {/* Information gaps warning */}
      {informationGaps.length > 0 && (
        <div className="px-3 py-2 bg-yellow-900/20 border border-yellow-700 rounded-lg">
          <div className="flex items-start gap-2">
            <AlertTriangle className="w-4 h-4 text-yellow-300 flex-shrink-0 mt-0.5" />
            <div className="flex-1">
              <p className="text-sm text-yellow-300 font-medium mb-1">
                Informazioni mancanti:
              </p>
              <ul className="text-sm text-yellow-200 list-disc list-inside space-y-0.5">
                {informationGaps.map((gap, idx) => (
                  <li key={idx}>{gap}</li>
                ))}
              </ul>
            </div>
          </div>
        </div>
      )}

      {/* Sources section */}
      {sources.length > 0 && (
        <div className="border border-dark-50 rounded-lg overflow-hidden bg-dark-100">
          {/* Header */}
          <button
            onClick={() => setIsExpanded(!isExpanded)}
            className="w-full px-4 py-3 flex items-center justify-between hover:bg-dark-50 transition-colors"
          >
            <span className="text-sm font-medium text-gray-400">
              Fonti ({sources.length})
            </span>
            {isExpanded ? (
              <ChevronUp className="w-4 h-4 text-gray-400" />
            ) : (
              <ChevronDown className="w-4 h-4 text-gray-400" />
            )}
          </button>

          {/* Sources list */}
          {isExpanded && (
            <div className="border-t border-dark-50 divide-y divide-dark-50">
              {sources.map((source, idx) => {
                // Color-code confidence score
                const confidencePercent = (source.confidence * 100).toFixed(0);
                const confidenceColor =
                  source.confidence > 0.7
                    ? 'text-green-400'
                    : source.confidence > 0.5
                      ? 'text-yellow-400'
                      : 'text-red-400';

                return (
                  <div
                    key={idx}
                    className="px-4 py-3 hover:bg-dark-50 transition-colors"
                  >
                    {/* Source header */}
                    <div className="flex items-center justify-between mb-2">
                      <span className="text-xs font-mono text-gray-500 bg-dark-200 px-2 py-1 rounded">
                        {source.doc_id}
                      </span>
                      <span className={`text-xs font-medium ${confidenceColor}`}>
                        {confidencePercent}% rilevanza
                      </span>
                    </div>

                    {/* Text quote */}
                    <p className="text-sm text-gray-300 italic leading-relaxed">
                      &ldquo;{source.text_quote}&rdquo;
                    </p>

                    {/* Metadata (if available) */}
                    {source.metadata && (
                      <div className="mt-2 flex gap-2 text-xs text-gray-500">
                        {source.metadata.page && (
                          <span>Pagina {source.metadata.page}</span>
                        )}
                        {source.metadata.section && (
                          <span>• {source.metadata.section}</span>
                        )}
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
