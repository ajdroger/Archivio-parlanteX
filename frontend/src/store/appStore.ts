import { create } from 'zustand';
import type { KnowledgeBase, Document, ComparisonResult, LLMProvider } from '../types';

interface Workspace {
  id: string;
  name: string;
  user_role: 'admin' | 'member' | 'viewer';
  member_count: number;
  kb_count: number;
}

interface AppState {
  // Workspace state (Fase 6.3)
  currentWorkspace: Workspace | null;
  setCurrentWorkspace: (workspace: Workspace | null) => void;

  // Knowledge Base state
  currentKb: KnowledgeBase | null;
  knowledgeBases: KnowledgeBase[];
  documents: Document[];

  // Comparison state
  selectedDocIds: string[];
  comparisonResult: ComparisonResult | null;
  comparisonLoading: boolean;
  comparisonPhase: string | null;
  comparisonError: string | null;

  // LLM Provider state
  providers: LLMProvider[];
  selectedProvider: string | null;
  selectedModel: string | null;

  // Actions
  setCurrentKb: (kb: KnowledgeBase | null) => void;
  setKnowledgeBases: (kbs: KnowledgeBase[]) => void;
  setDocuments: (docs: Document[]) => void;

  toggleDocSelection: (docId: string) => void;
  clearDocSelection: () => void;
  setComparisonResult: (result: ComparisonResult | null) => void;
  setComparisonLoading: (loading: boolean) => void;
  setComparisonPhase: (phase: string | null) => void;
  setComparisonError: (error: string | null) => void;
  clearComparison: () => void;

  setProviders: (providers: LLMProvider[]) => void;
  setSelectedProvider: (providerId: string) => void;
  setSelectedModel: (modelId: string) => void;
}

export const useAppStore = create<AppState>((set) => ({
  // Initial state
  currentWorkspace: null,
  currentKb: null,
  knowledgeBases: [],
  documents: [],

  selectedDocIds: [],
  comparisonResult: null,
  comparisonLoading: false,
  comparisonPhase: null,
  comparisonError: null,

  providers: [],
  selectedProvider: null,
  selectedModel: null,

  // Actions
  setCurrentWorkspace: (workspace) => set({ currentWorkspace: workspace }),
  setCurrentKb: (kb) => set({ currentKb: kb }),
  setKnowledgeBases: (kbs) => set({ knowledgeBases: kbs }),
  setDocuments: (docs) => set({ documents: docs }),

  toggleDocSelection: (docId) =>
    set((state) => ({
      selectedDocIds: state.selectedDocIds.includes(docId)
        ? state.selectedDocIds.filter((id) => id !== docId)
        : [...state.selectedDocIds, docId],
    })),

  clearDocSelection: () => set({ selectedDocIds: [] }),

  setComparisonResult: (result) => set({ comparisonResult: result }),
  setComparisonLoading: (loading) => set({ comparisonLoading: loading }),
  setComparisonPhase: (phase) => set({ comparisonPhase: phase }),
  setComparisonError: (error) => set({ comparisonError: error }),

  clearComparison: () =>
    set({
      comparisonResult: null,
      comparisonLoading: false,
      comparisonPhase: null,
      comparisonError: null,
    }),

  setProviders: (providers) => set({ providers }),
  setSelectedProvider: (providerId) => set({ selectedProvider: providerId }),
  setSelectedModel: (modelId) => set({ selectedModel: modelId }),
}));
