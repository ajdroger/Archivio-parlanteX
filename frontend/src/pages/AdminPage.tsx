import { useAuthStore } from '../store/authStore';
import { Navigate } from 'react-router-dom';

export default function AdminPage() {
  const { user } = useAuthStore();

  // Only admins can access this page
  if (user?.role !== 'admin') {
    return <Navigate to="/" replace />;
  }

  return (
    <div className="flex flex-col h-screen">
      <header className="border-b border-dark-50 bg-dark-100 px-6 py-4">
        <h2 className="text-2xl font-semibold text-gray-100">
          ⚙️ Amministrazione
        </h2>
        <p className="text-sm text-gray-400 mt-1">
          Gestione utenti, Knowledge Bases e configurazione sistema
        </p>
      </header>

      <div className="flex-1 overflow-auto p-6">
        <div className="space-y-6">
          {/* KB Management Section */}
          <div className="bg-dark-100 border border-dark-50 rounded-lg p-4">
            <h3 className="text-lg font-semibold text-gray-200 mb-2">
              Knowledge Bases
            </h3>
            <p className="text-sm text-gray-400">
              Gestione KB in arrivo...
            </p>
          </div>

          {/* User Management Section */}
          <div className="bg-dark-100 border border-dark-50 rounded-lg p-4">
            <h3 className="text-lg font-semibold text-gray-200 mb-2">
              Utenti
            </h3>
            <p className="text-sm text-gray-400">
              Gestione utenti in arrivo...
            </p>
          </div>

          {/* System Settings Section */}
          <div className="bg-dark-100 border border-dark-50 rounded-lg p-4">
            <h3 className="text-lg font-semibold text-gray-200 mb-2">
              Impostazioni Sistema
            </h3>
            <p className="text-sm text-gray-400">
              Configurazione LLM provider, budget, API keys in arrivo...
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
