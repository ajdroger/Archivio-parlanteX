import { useState, useEffect } from 'react';
import { FileText, Search, Loader2 } from 'lucide-react';
import api from '../../lib/api';
import type { Document } from '../../types';

interface DocumentSelectorProps {
  kbId: string;
  selectedDocIds: string[];
  onSelectionChange: (ids: string[]) => void;
}

/**
 * Document selector component with multi-select checkboxes and filtering.
 * Allows users to select multiple documents for comparison.
 */
export default function DocumentSelector({
  kbId,
  selectedDocIds,
  onSelectionChange,
}: DocumentSelectorProps) {
  const [documents, setDocuments] = useState<Document[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [filterText, setFilterText] = useState('');

  // Fetch documents on mount or when kbId changes
  useEffect(() => {
    const fetchDocuments = async () => {
      setLoading(true);
      setError(null);
      try {
        const docs = await api.listDocuments(kbId);
        setDocuments(docs);
      } catch (err: unknown) {
        const errorMessage = err instanceof Error ? err.message : 'Errore durante il caricamento dei documenti';
        setError(errorMessage);
      } finally {
        setLoading(false);
      }
    };

    fetchDocuments();
  }, [kbId]);

  // Filter documents by name
  const filteredDocuments = documents.filter((doc) =>
    doc.source_name.toLowerCase().includes(filterText.toLowerCase())
  );

  // Toggle document selection
  const toggleDocument = (docId: string) => {
    if (selectedDocIds.includes(docId)) {
      onSelectionChange(selectedDocIds.filter((id) => id !== docId));
    } else {
      onSelectionChange([...selectedDocIds, docId]);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center p-6">
        <Loader2 className="w-6 h-6 text-primary animate-spin" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-4 bg-red-900/30 border border-red-700 rounded-lg text-red-300 text-sm">
        {error}
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header with search */}
      <div className="p-4 border-b border-dark-50 space-y-3">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-medium text-gray-300">
            Documenti ({filteredDocuments.length})
          </h3>
          <span className="text-xs text-primary">
            {selectedDocIds.length} selezionati
          </span>
        </div>

        {/* Search input */}
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500" />
          <input
            type="text"
            value={filterText}
            onChange={(e) => setFilterText(e.target.value)}
            placeholder="Cerca documenti..."
            className="w-full pl-10 pr-3 py-2 bg-dark-200 border border-dark-50 rounded-lg text-sm text-gray-100 placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent"
          />
        </div>
      </div>

      {/* Document list */}
      <div className="flex-1 overflow-auto">
        {filteredDocuments.length === 0 ? (
          <div className="p-4 text-center text-gray-500 text-sm">
            {filterText ? 'Nessun documento trovato' : 'Nessun documento disponibile'}
          </div>
        ) : (
          <div className="divide-y divide-dark-50">
            {filteredDocuments.map((doc) => {
              const isSelected = selectedDocIds.includes(doc.id);
              return (
                <label
                  key={doc.id}
                  className={`flex items-start gap-3 p-3 cursor-pointer hover:bg-dark-50 transition-colors ${
                    isSelected ? 'bg-primary/10' : ''
                  }`}
                >
                  {/* Checkbox */}
                  <input
                    type="checkbox"
                    checked={isSelected}
                    onChange={() => toggleDocument(doc.id)}
                    className="mt-1 w-4 h-4 rounded border-dark-50 bg-dark-200 text-primary focus:ring-2 focus:ring-primary focus:ring-offset-0 cursor-pointer"
                  />

                  {/* Document info */}
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-1">
                      <FileText className="w-4 h-4 text-gray-400 flex-shrink-0" />
                      <span className={`text-sm truncate ${isSelected ? 'text-primary font-medium' : 'text-gray-300'}`}>
                        {doc.source_name}
                      </span>
                    </div>
                    <div className="flex items-center gap-2 text-xs text-gray-500">
                      <span>{new Date(doc.created_at).toLocaleDateString('it-IT')}</span>
                      <span>•</span>
                      <span
                        className={
                          doc.status === 'indexed'
                            ? 'text-green-400'
                            : doc.status === 'processing'
                              ? 'text-yellow-400'
                              : 'text-red-400'
                        }
                      >
                        {doc.status}
                      </span>
                    </div>
                  </div>
                </label>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
