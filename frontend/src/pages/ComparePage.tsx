import { useAppStore } from '../store/appStore';
import DocumentSelector from '../components/documents/DocumentSelector';
import ContractComparison from '../components/comparison/ContractComparison';

/**
 * Compare page for multi-contract comparison.
 * Allows users to select documents and compare them side-by-side.
 */
export default function ComparePage() {
  const { currentKb, selectedDocIds, toggleDocSelection, clearDocSelection } =
    useAppStore();

  if (!currentKb) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <p className="text-gray-400 text-lg">
            Nessuna Knowledge Base selezionata
          </p>
          <p className="text-gray-500 text-sm mt-2">
            Seleziona una KB per confrontare i contratti
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-screen">
      {/* Left sidebar - Document selector */}
      <aside className="w-80 border-r border-dark-50 bg-dark-100 flex flex-col">
        <div className="border-b border-dark-50 px-4 py-3">
          <h2 className="text-lg font-semibold text-gray-100">
            🔀 Confronto Contratti
          </h2>
          <p className="text-xs text-gray-400 mt-1">
            {currentKb.name}
          </p>
        </div>

        <DocumentSelector
          kbId={currentKb.id}
          selectedDocIds={selectedDocIds}
          onSelectionChange={(ids) => {
            // Update selection by toggling
            const newIds = ids.filter((id) => !selectedDocIds.includes(id));
            const removedIds = selectedDocIds.filter((id) => !ids.includes(id));

            newIds.forEach((id) => toggleDocSelection(id));
            removedIds.forEach((id) => toggleDocSelection(id));

            // If completely cleared, use clearDocSelection
            if (ids.length === 0 && selectedDocIds.length > 0) {
              clearDocSelection();
            }
          }}
        />

        {/* Clear selection button */}
        {selectedDocIds.length > 0 && (
          <div className="p-4 border-t border-dark-50">
            <button
              onClick={clearDocSelection}
              className="w-full px-3 py-2 text-sm text-gray-400 hover:text-gray-300 border border-dark-50 rounded-lg hover:bg-dark-50 transition-colors"
            >
              Deseleziona tutto
            </button>
          </div>
        )}
      </aside>

      {/* Main content - Comparison interface */}
      <main className="flex-1 flex flex-col overflow-hidden">
        <header className="border-b border-dark-50 bg-dark-100 px-6 py-4">
          <p className="text-sm text-gray-400">
            Confronta più contratti in parallelo con precisione forense
          </p>
        </header>

        <ContractComparison />
      </main>
    </div>
  );
}
