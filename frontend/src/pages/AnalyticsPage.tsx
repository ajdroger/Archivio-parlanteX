export default function AnalyticsPage() {
  return (
    <div className="flex flex-col h-screen">
      <header className="border-b border-dark-50 bg-dark-100 px-6 py-4">
        <h2 className="text-2xl font-semibold text-gray-100">
          📊 Analytics
        </h2>
        <p className="text-sm text-gray-400 mt-1">
          Statistiche di utilizzo e performance del sistema
        </p>
      </header>

      <div className="flex-1 overflow-auto p-6">
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          {/* Placeholder metric cards */}
          <div className="bg-dark-100 border border-dark-50 rounded-lg p-4">
            <p className="text-sm text-gray-400">Query totali</p>
            <p className="text-3xl font-bold text-primary mt-2">-</p>
          </div>
          <div className="bg-dark-100 border border-dark-50 rounded-lg p-4">
            <p className="text-sm text-gray-400">Latenza media</p>
            <p className="text-3xl font-bold text-primary mt-2">- ms</p>
          </div>
          <div className="bg-dark-100 border border-dark-50 rounded-lg p-4">
            <p className="text-sm text-gray-400">Documenti indicizzati</p>
            <p className="text-3xl font-bold text-primary mt-2">-</p>
          </div>
          <div className="bg-dark-100 border border-dark-50 rounded-lg p-4">
            <p className="text-sm text-gray-400">Costo giornaliero</p>
            <p className="text-3xl font-bold text-green-400 mt-2">€0.00</p>
          </div>
        </div>

        <div className="mt-6 text-center text-gray-500">
          <p className="text-sm">Dashboard analytics in sviluppo</p>
        </div>
      </div>
    </div>
  );
}
