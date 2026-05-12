import { useState, useEffect } from 'react';
import { ChevronDown, Zap, DollarSign, Lock } from 'lucide-react';
import { useAppStore } from '../../store/appStore';
import api from '../../lib/api';
import type { LLMProvider } from '../../types';

/**
 * LLM provider and model selector component.
 * Allows users to switch between local and cloud LLM providers.
 */
export default function ModelSelector() {
  const {
    providers,
    selectedProvider,
    selectedModel,
    setProviders,
    setSelectedProvider,
    setSelectedModel,
  } = useAppStore();

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showDropdown, setShowDropdown] = useState(false);

  // Fetch providers on mount
  useEffect(() => {
    const fetchProviders = async () => {
      setLoading(true);
      setError(null);
      try {
        const data = await api.listLlmProviders();
        setProviders(data);

        // Auto-select first enabled provider and model
        if (data.length > 0 && !selectedProvider) {
          const firstEnabled = data.find((p: LLMProvider) => p.enabled);
          if (firstEnabled) {
            setSelectedProvider(firstEnabled.id);
            if (firstEnabled.models.length > 0) {
              setSelectedModel(firstEnabled.models[0].id);
            }
          }
        }
      } catch (err: unknown) {
        const errorMessage = err instanceof Error ? err.message : 'Errore durante il caricamento dei provider';
        setError(errorMessage);
      } finally {
        setLoading(false);
      }
    };

    fetchProviders();
  }, []);

  const currentProvider = providers.find((p) => p.id === selectedProvider);
  const currentModel = currentProvider?.models.find((m) => m.id === selectedModel);

  const handleProviderChange = (providerId: string) => {
    setSelectedProvider(providerId);
    const provider = providers.find((p) => p.id === providerId);
    if (provider && provider.models.length > 0) {
      setSelectedModel(provider.models[0].id);
    }
    setShowDropdown(false);
  };

  const handleModelChange = (modelId: string) => {
    setSelectedModel(modelId);
    setShowDropdown(false);
  };

  if (loading) {
    return (
      <div className="text-xs text-gray-500">Caricamento modelli...</div>
    );
  }

  if (error) {
    return (
      <div className="text-xs text-red-400" title={error}>
        Errore caricamento
      </div>
    );
  }

  if (!currentProvider || !currentModel) {
    return (
      <div className="text-xs text-gray-500">Nessun modello selezionato</div>
    );
  }

  return (
    <div className="relative">
      {/* Selected model display */}
      <button
        onClick={() => setShowDropdown(!showDropdown)}
        className="flex items-center gap-2 px-3 py-2 bg-dark-200 border border-dark-50 rounded-lg text-sm text-gray-300 hover:border-primary/50 transition-colors"
        title={currentProvider.is_local ? 'Local (gratuito)' : 'Cloud (a pagamento)'}
      >
        {currentProvider.is_local ? (
          <Zap className="w-4 h-4 text-green-400" />
        ) : (
          <DollarSign className="w-4 h-4 text-yellow-400" />
        )}
        <span className="font-medium">{currentModel.name}</span>
        <ChevronDown className={`w-4 h-4 transition-transform ${showDropdown ? 'rotate-180' : ''}`} />
      </button>

      {/* Dropdown */}
      {showDropdown && (
        <>
          {/* Backdrop */}
          <div
            className="fixed inset-0 z-10"
            onClick={() => setShowDropdown(false)}
          />

          {/* Dropdown menu */}
          <div className="absolute top-full right-0 mt-2 w-80 bg-dark-100 border border-dark-50 rounded-lg shadow-2xl z-20 max-h-96 overflow-auto">
            {providers.map((provider) => (
              <div key={provider.id} className="border-b border-dark-50 last:border-0">
                {/* Provider header */}
                <div
                  className={`px-4 py-3 ${
                    provider.enabled ? '' : 'opacity-50'
                  }`}
                >
                  <div className="flex items-center justify-between mb-2">
                    <div className="flex items-center gap-2">
                      {provider.is_local ? (
                        <Zap className="w-4 h-4 text-green-400" />
                      ) : (
                        <DollarSign className="w-4 h-4 text-yellow-400" />
                      )}
                      <span className="text-sm font-semibold text-gray-200">
                        {provider.name}
                      </span>
                    </div>
                    {!provider.enabled && (
                      <div title="Disabilitato">
                        <Lock className="w-3 h-3 text-gray-500" />
                      </div>
                    )}
                    {provider.requires_api_key && !provider.has_api_key && (
                      <span className="text-xs text-yellow-400">API key mancante</span>
                    )}
                  </div>

                  {/* Models list */}
                  {provider.enabled && (
                    <div className="space-y-1">
                      {provider.models.map((model) => {
                        const isSelected =
                          provider.id === selectedProvider && model.id === selectedModel;

                        return (
                          <button
                            key={model.id}
                            onClick={() => {
                              handleProviderChange(provider.id);
                              handleModelChange(model.id);
                            }}
                            className={`w-full text-left px-3 py-2 rounded text-sm transition-colors ${
                              isSelected
                                ? 'bg-primary/20 text-primary'
                                : 'text-gray-400 hover:bg-dark-50 hover:text-gray-300'
                            }`}
                          >
                            <div className="flex items-center justify-between mb-1">
                              <span className="font-medium">{model.name}</span>
                              {model.cost_per_1k_input !== undefined ? (
                                <span className="text-xs text-yellow-400">
                                  ${model.cost_per_1k_input.toFixed(3)}/1K
                                </span>
                              ) : (
                                <span className="text-xs text-green-400">Gratuito</span>
                              )}
                            </div>
                            <div className="text-xs text-gray-500">
                              Context: {(model.context_length / 1000).toFixed(0)}K tokens
                            </div>
                          </button>
                        );
                      })}
                    </div>
                  )}

                  {/* Disabled provider message */}
                  {!provider.enabled && (
                    <p className="text-xs text-gray-500">
                      {provider.requires_api_key
                        ? 'Configura API key in Admin'
                        : 'Provider disabilitato'}
                    </p>
                  )}
                </div>
              </div>
            ))}
          </div>
        </>
      )}
    </div>
  );
}
