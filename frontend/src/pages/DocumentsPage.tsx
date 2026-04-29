import { useState, useEffect } from 'react';
import { useAppStore } from '../store/appStore';
import { FileText, Trash2, Loader2, AlertCircle } from 'lucide-react';
import DocumentUpload from '../components/documents/DocumentUpload';
import api from '../lib/api';
import type { Document } from '../types';

/**
 * Documents page for managing documents in a knowledge base.
 * Allows users to upload, view, and delete documents.
 */
export default function DocumentsPage() {
  const { currentKb } = useAppStore();
  const [documents, setDocuments] = useState<Document[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [deletingId, setDeletingId] = useState<string | null>(null);

  // Fetch documents when KB changes
  useEffect(() => {
    if (currentKb) {
      fetchDocuments();
    }
  }, [currentKb]);

  const fetchDocuments = async () => {
    if (!currentKb) return;

    setLoading(true);
    setError(null);
    try {
      const docs = await api.listDocuments(currentKb.id);
      setDocuments(docs);
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Errore durante il caricamento';
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async (docId: string) => {
    if (!currentKb) return;

    const confirmed = window.confirm(
      'Sei sicuro di voler eliminare questo documento? Questa azione è irreversibile.'
    );
    if (!confirmed) return;

    setDeletingId(docId);
    try {
      await api.deleteDocument(currentKb.id, docId);
      setDocuments((prev) => prev.filter((doc) => doc.id !== docId));
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Errore durante la cancellazione';
      alert(errorMessage);
    } finally {
      setDeletingId(null);
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
            Seleziona o crea una KB per gestire i documenti
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-screen">
      {/* Header */}
      <header className="border-b border-dark-50 bg-dark-100 px-6 py-4">
        <h2 className="text-2xl font-semibold text-gray-100">
          📄 Documenti - {currentKb.name}
        </h2>
        <p className="text-sm text-gray-400 mt-1">
          Carica e gestisci i documenti della tua Knowledge Base
        </p>
      </header>

      {/* Content */}
      <div className="flex-1 overflow-auto p-6">
        <div className="max-w-5xl mx-auto space-y-6">
          {/* Upload section */}
          <section>
            <h3 className="text-lg font-semibold text-gray-200 mb-4">
              Carica Documenti
            </h3>
            <DocumentUpload kbId={currentKb.id} onUploadComplete={fetchDocuments} />
          </section>

          {/* Documents list */}
          <section>
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-semibold text-gray-200">
                Documenti ({documents.length})
              </h3>
              {documents.length > 0 && (
                <button
                  onClick={fetchDocuments}
                  disabled={loading}
                  className="text-sm text-primary hover:text-primary-400 disabled:opacity-50"
                >
                  Aggiorna
                </button>
              )}
            </div>

            {/* Loading state */}
            {loading && (
              <div className="flex items-center justify-center py-12">
                <Loader2 className="w-6 h-6 text-primary animate-spin" />
              </div>
            )}

            {/* Error state */}
            {error && (
              <div className="flex items-start gap-3 p-4 bg-red-900/30 border border-red-700 rounded-lg">
                <AlertCircle className="w-5 h-5 text-red-300 flex-shrink-0 mt-0.5" />
                <div>
                  <p className="text-sm font-medium text-red-300">Errore</p>
                  <p className="text-sm text-red-200">{error}</p>
                </div>
              </div>
            )}

            {/* Empty state */}
            {!loading && !error && documents.length === 0 && (
              <div className="text-center py-12 text-gray-500">
                <FileText className="w-12 h-12 mx-auto mb-4 opacity-50" />
                <p className="text-sm">Nessun documento caricato</p>
                <p className="text-xs mt-1">
                  Carica il tuo primo documento per iniziare
                </p>
              </div>
            )}

            {/* Documents grid */}
            {!loading && documents.length > 0 && (
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {documents.map((doc) => (
                  <div
                    key={doc.id}
                    className="bg-dark-100 border border-dark-50 rounded-lg p-4 hover:border-primary/50 transition-colors"
                  >
                    {/* Document header */}
                    <div className="flex items-start gap-3 mb-3">
                      <FileText className="w-5 h-5 text-primary flex-shrink-0 mt-0.5" />
                      <div className="flex-1 min-w-0">
                        <h4 className="text-sm font-medium text-gray-200 truncate">
                          {doc.source_name}
                        </h4>
                        <p className="text-xs text-gray-500 mt-1">
                          {doc.mime_type}
                        </p>
                      </div>
                    </div>

                    {/* Document metadata */}
                    <div className="space-y-2 mb-3">
                      <div className="flex items-center justify-between text-xs">
                        <span className="text-gray-500">Status</span>
                        <span
                          className={`px-2 py-0.5 rounded ${
                            doc.status === 'indexed'
                              ? 'bg-green-900/30 text-green-400'
                              : doc.status === 'processing'
                                ? 'bg-yellow-900/30 text-yellow-400'
                                : 'bg-red-900/30 text-red-400'
                          }`}
                        >
                          {doc.status}
                        </span>
                      </div>
                      <div className="flex items-center justify-between text-xs">
                        <span className="text-gray-500">Creato</span>
                        <span className="text-gray-400">
                          {new Date(doc.created_at).toLocaleDateString('it-IT')}
                        </span>
                      </div>
                      {doc.indexed_at && (
                        <div className="flex items-center justify-between text-xs">
                          <span className="text-gray-500">Indicizzato</span>
                          <span className="text-gray-400">
                            {new Date(doc.indexed_at).toLocaleDateString('it-IT')}
                          </span>
                        </div>
                      )}
                    </div>

                    {/* Tags */}
                    {doc.tags && doc.tags.length > 0 && (
                      <div className="flex flex-wrap gap-1 mb-3">
                        {doc.tags.map((tag, idx) => (
                          <span
                            key={idx}
                            className="text-xs px-2 py-0.5 bg-dark-200 text-gray-400 rounded"
                          >
                            {tag}
                          </span>
                        ))}
                      </div>
                    )}

                    {/* Actions */}
                    <div className="pt-3 border-t border-dark-50">
                      <button
                        onClick={() => handleDelete(doc.id)}
                        disabled={deletingId === doc.id}
                        className="flex items-center gap-2 text-sm text-red-400 hover:text-red-300 disabled:opacity-50 transition-colors"
                      >
                        {deletingId === doc.id ? (
                          <>
                            <Loader2 className="w-4 h-4 animate-spin" />
                            <span>Eliminazione...</span>
                          </>
                        ) : (
                          <>
                            <Trash2 className="w-4 h-4" />
                            <span>Elimina</span>
                          </>
                        )}
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </section>
        </div>
      </div>
    </div>
  );
}

