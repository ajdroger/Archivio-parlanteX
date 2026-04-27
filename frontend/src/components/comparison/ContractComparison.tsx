import { useState } from 'react';
import { GitCompare, Loader2, AlertCircle, Plus, X } from 'lucide-react';
import { useAppStore } from '../../store/appStore';
import api from '../../lib/api';

/**
 * Contract comparison component for multi-document analysis.
 * Allows users to compare multiple contracts across specified aspects.
 */
export default function ContractComparison() {
  const {
    currentKb,
    selectedDocIds,
    comparisonResult,
    comparisonLoading,
    comparisonError,
    setComparisonResult,
    setComparisonLoading,
    setComparisonError,
  } = useAppStore();

  const [aspects, setAspects] = useState<string[]>(['Clausole di recesso', 'Penali', 'Durata contratto']);
  const [newAspect, setNewAspect] = useState('');

  const handleCompare = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!currentKb || selectedDocIds.length < 2 || aspects.length === 0) return;

    setComparisonLoading(true);
    setComparisonError(null);

    try {
      const result = await api.compareContracts({
        kb_id: currentKb.id,
        doc_ids: selectedDocIds,
        comparison_aspects: aspects,
      });
      setComparisonResult(result);
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Errore durante il confronto';
      setComparisonError(errorMessage);
    } finally {
      setComparisonLoading(false);
    }
  };

  const addAspect = () => {
    if (newAspect.trim() && !aspects.includes(newAspect.trim())) {
      setAspects([...aspects, newAspect.trim()]);
      setNewAspect('');
    }
  };

  const removeAspect = (aspect: string) => {
    setAspects(aspects.filter((a) => a !== aspect));
  };

  const canCompare = currentKb && selectedDocIds.length >= 2 && aspects.length > 0;

  return (
    <div className="flex flex-col h-full">
      {/* Results area */}
      <div className="flex-1 overflow-auto p-6 space-y-6">
        {/* Instructions */}
        {!comparisonResult && !comparisonLoading && !comparisonError && (
          <div className="flex items-center justify-center h-full">
            <div className="text-center max-w-md">
              <GitCompare className="w-12 h-12 text-primary mx-auto mb-4" />
              <h3 className="text-lg font-semibold text-gray-200 mb-2">
                Confronto Multi-Contratto
              </h3>
              <p className="text-sm text-gray-400 mb-4">
                Seleziona almeno 2 documenti dalla sidebar e formula una domanda per
                confrontare i contratti in parallelo
              </p>
              {selectedDocIds.length > 0 && (
                <p className="text-xs text-primary">
                  {selectedDocIds.length} documento{selectedDocIds.length > 1 ? 'i' : ''}{' '}
                  selezionat{selectedDocIds.length > 1 ? 'i' : 'o'}
                  {selectedDocIds.length < 2 && ' (minimo 2 richiesti)'}
                </p>
              )}
            </div>
          </div>
        )}

        {/* Error state */}
        {comparisonError && (
          <div className="flex items-start gap-3 p-4 bg-red-900/30 border border-red-700 rounded-lg">
            <AlertCircle className="w-5 h-5 text-red-300 flex-shrink-0 mt-0.5" />
            <div className="flex-1">
              <p className="text-sm font-medium text-red-300 mb-1">
                Errore durante il confronto
              </p>
              <p className="text-sm text-red-200">{comparisonError}</p>
            </div>
          </div>
        )}

        {/* Results */}
        {comparisonResult && (
          <div className="space-y-6">
            {/* Comparison table */}
            <div className="bg-dark-100 border border-dark-50 rounded-lg overflow-hidden">
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead className="bg-dark-200 border-b border-dark-50">
                    <tr>
                      <th className="px-4 py-3 text-left text-sm font-medium text-gray-300">
                        Aspetto
                      </th>
                      {selectedDocIds.map((docId) => (
                        <th key={docId} className="px-4 py-3 text-left text-sm font-medium text-gray-300">
                          {docId.substring(0, 8)}...
                        </th>
                      ))}
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-dark-50">
                    {comparisonResult.aspects.map((aspect, idx) => (
                      <tr key={idx} className="hover:bg-dark-50">
                        <td className="px-4 py-3 text-sm font-medium text-primary">
                          {aspect.aspect_name}
                        </td>
                        {aspect.cells.map((cell, cellIdx) => (
                          <td key={cellIdx} className="px-4 py-3 text-sm">
                            {cell.present ? (
                              <div>
                                <p className="text-gray-300 italic mb-1">
                                  {cell.text_quote ? `"${cell.text_quote.substring(0, 100)}..."` : 'Presente'}
                                </p>
                                {cell.confidence !== undefined && (
                                  <span className={`text-xs ${
                                    cell.confidence > 0.7 ? 'text-green-400' :
                                    cell.confidence > 0.5 ? 'text-yellow-400' : 'text-red-400'
                                  }`}>
                                    {(cell.confidence * 100).toFixed(0)}%
                                  </span>
                                )}
                              </div>
                            ) : (
                              <span className="text-gray-500 text-xs">Non trovato</span>
                            )}
                          </td>
                        ))}
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>

            {/* Key differences section */}
            {comparisonResult.key_differences &&
              comparisonResult.key_differences.length > 0 && (
                <div className="bg-dark-100 border border-primary/30 rounded-lg p-4">
                  <h4 className="text-sm font-semibold text-primary mb-3">
                    Differenze Chiave
                  </h4>
                  <div className="space-y-2">
                    {comparisonResult.key_differences.map((diff, idx) => (
                      <div
                        key={idx}
                        className="flex items-start gap-2 text-sm text-gray-300 bg-dark-200 px-3 py-2 rounded"
                      >
                        <span className="text-primary font-mono mt-0.5">•</span>
                        <span>{diff}</span>
                      </div>
                    ))}
                  </div>
                </div>
              )}

            {/* Information gaps */}
            {comparisonResult.information_gaps && comparisonResult.information_gaps.length > 0 && (
              <div className="px-3 py-2 bg-yellow-900/20 border border-yellow-700 rounded-lg">
                <p className="text-sm text-yellow-300 font-medium mb-1">
                  ⚠️ Informazioni mancanti:
                </p>
                <ul className="text-sm text-yellow-200 list-disc list-inside">
                  {comparisonResult.information_gaps.map((gap, idx) => (
                    <li key={idx}>{gap}</li>
                  ))}
                </ul>
              </div>
            )}

            {/* Processing time */}
            <p className="text-xs text-gray-500 text-center">
              Elaborato in {comparisonResult.processing_time_ms}ms •{' '}
              {comparisonResult.verified ? '✅ Verificato' : ''}
            </p>
          </div>
        )}
      </div>

      {/* Input form */}
      <div className="border-t border-dark-50 bg-dark-100 p-4">
        <form onSubmit={handleCompare} className="space-y-3">
          {/* Aspects to compare */}
          <div>
            <label className="text-xs text-gray-400 mb-2 block">
              Aspetti da confrontare
            </label>
            <div className="flex flex-wrap gap-2 mb-2">
              {aspects.map((aspect) => (
                <span
                  key={aspect}
                  className="inline-flex items-center gap-1 px-2 py-1 bg-primary/20 border border-primary/30 rounded text-sm text-primary"
                >
                  {aspect}
                  <button
                    type="button"
                    onClick={() => removeAspect(aspect)}
                    className="hover:text-primary-400"
                  >
                    <X className="w-3 h-3" />
                  </button>
                </span>
              ))}
            </div>
            <div className="flex gap-2">
              <input
                type="text"
                value={newAspect}
                onChange={(e) => setNewAspect(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') {
                    e.preventDefault();
                    addAspect();
                  }
                }}
                placeholder="Aggiungi aspetto (es. Clausole penali)"
                disabled={comparisonLoading}
                className="flex-1 px-3 py-2 text-sm bg-dark-200 border border-dark-50 rounded-lg text-gray-100 placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent disabled:opacity-50"
              />
              <button
                type="button"
                onClick={addAspect}
                disabled={!newAspect.trim() || comparisonLoading}
                className="px-3 py-2 bg-dark-200 border border-dark-50 rounded-lg text-gray-400 hover:text-primary hover:border-primary/50 transition-colors disabled:opacity-50"
              >
                <Plus className="w-4 h-4" />
              </button>
            </div>
          </div>

          <div className="flex items-center justify-between pt-2">
            <p className="text-xs text-gray-500">
              {selectedDocIds.length} documento{selectedDocIds.length !== 1 ? 'i' : ''}{' '}
              selezionat{selectedDocIds.length !== 1 ? 'i' : 'o'} •{' '}
              {aspects.length} aspetto{aspects.length !== 1 ? 'i' : ''}
            </p>
            <button
              type="submit"
              disabled={!canCompare || comparisonLoading}
              className="px-6 py-2 bg-primary text-dark-300 font-medium rounded-lg hover:bg-primary-400 transition-colors focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2 focus:ring-offset-dark disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
            >
              {comparisonLoading ? (
                <>
                  <Loader2 className="w-4 h-4 animate-spin" />
                  <span>Confronto in corso...</span>
                </>
              ) : (
                <>
                  <GitCompare className="w-4 h-4" />
                  <span>Confronta</span>
                </>
              )}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
