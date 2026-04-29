import { describe, it, expect, beforeEach } from 'vitest';
import { useAppStore } from './appStore';
import type { KnowledgeBase, ComparisonResult } from '../types';

describe('appStore', () => {
  beforeEach(() => {
    // Reset store to initial state before each test
    useAppStore.setState({
      currentKb: null,
      knowledgeBases: [],
      selectedDocIds: [],
      comparisonResult: null,
      comparisonLoading: false,
      selectedProvider: null,
      selectedModel: null,
    });
  });

  describe('setCurrentKb', () => {
    it('sets the current knowledge base', () => {
      const mockKb: KnowledgeBase = {
        id: '1',
        name: 'Legal Contracts',
        description: 'Contract analysis KB',
        created_at: '2024-01-01',
        document_count: 10,
      };

      useAppStore.getState().setCurrentKb(mockKb);

      const state = useAppStore.getState();
      expect(state.currentKb).toEqual(mockKb);
    });

    it('clears current KB when null is passed', () => {
      const mockKb: KnowledgeBase = {
        id: '1',
        name: 'Legal Contracts',
        description: 'Contract analysis KB',
        created_at: '2024-01-01',
        document_count: 10,
      };

      useAppStore.getState().setCurrentKb(mockKb);
      useAppStore.getState().setCurrentKb(null);

      const state = useAppStore.getState();
      expect(state.currentKb).toBeNull();
    });
  });

  describe('setKnowledgeBases', () => {
    it('sets the knowledge bases list', () => {
      const mockKbs: KnowledgeBase[] = [
        {
          id: '1',
          name: 'KB 1',
          description: 'First KB',
          created_at: '2024-01-01',
          document_count: 5,
        },
        {
          id: '2',
          name: 'KB 2',
          description: 'Second KB',
          created_at: '2024-01-02',
          document_count: 10,
        },
      ];

      useAppStore.getState().setKnowledgeBases(mockKbs);

      const state = useAppStore.getState();
      expect(state.knowledgeBases).toEqual(mockKbs);
      expect(state.knowledgeBases).toHaveLength(2);
    });
  });

  describe('setDocuments', () => {
    it('sets the documents list', () => {
      const mockDocs = [
        {
          id: 'doc_1',
          source_name: 'contract1.pdf',
          kb_id: '1',
          status: 'indexed' as const,
          created_at: '2024-01-01',
        },
        {
          id: 'doc_2',
          source_name: 'contract2.pdf',
          kb_id: '1',
          status: 'processing' as const,
          created_at: '2024-01-02',
        },
      ];

      useAppStore.getState().setDocuments(mockDocs);

      const state = useAppStore.getState();
      expect(state.documents).toEqual(mockDocs);
      expect(state.documents).toHaveLength(2);
    });
  });

  describe('toggleDocSelection', () => {
    it('adds document ID if not already selected', () => {
      useAppStore.getState().toggleDocSelection('doc_1');

      const state = useAppStore.getState();
      expect(state.selectedDocIds).toContain('doc_1');
      expect(state.selectedDocIds).toHaveLength(1);
    });

    it('removes document ID if already selected', () => {
      useAppStore.setState({ selectedDocIds: ['doc_1', 'doc_2'] });

      useAppStore.getState().toggleDocSelection('doc_1');

      const state = useAppStore.getState();
      expect(state.selectedDocIds).not.toContain('doc_1');
      expect(state.selectedDocIds).toContain('doc_2');
      expect(state.selectedDocIds).toHaveLength(1);
    });

    it('handles multiple toggles correctly', () => {
      useAppStore.getState().toggleDocSelection('doc_1');
      useAppStore.getState().toggleDocSelection('doc_2');
      useAppStore.getState().toggleDocSelection('doc_3');
      useAppStore.getState().toggleDocSelection('doc_2'); // Remove doc_2

      const state = useAppStore.getState();
      expect(state.selectedDocIds).toEqual(['doc_1', 'doc_3']);
    });
  });

  describe('clearDocSelection', () => {
    it('clears all selected document IDs', () => {
      useAppStore.setState({ selectedDocIds: ['doc_1', 'doc_2', 'doc_3'] });

      useAppStore.getState().clearDocSelection();

      const state = useAppStore.getState();
      expect(state.selectedDocIds).toEqual([]);
    });
  });

  describe('setComparisonResult', () => {
    it('sets comparison result and loading state', () => {
      const mockResult: ComparisonResult = {
        comparison: 'Comparison analysis here',
        key_differences: ['Difference 1', 'Difference 2'],
        sources: [],
        information_gaps: [],
      };

      useAppStore.getState().setComparisonResult(mockResult);

      const state = useAppStore.getState();
      expect(state.comparisonResult).toEqual(mockResult);
      expect(state.comparisonLoading).toBe(false);
    });

    it('clears result when null is passed', () => {
      const mockResult: ComparisonResult = {
        comparison: 'Test',
        key_differences: [],
        sources: [],
        information_gaps: [],
      };

      useAppStore.getState().setComparisonResult(mockResult);
      useAppStore.getState().setComparisonResult(null);

      const state = useAppStore.getState();
      expect(state.comparisonResult).toBeNull();
    });
  });

  describe('setComparisonLoading', () => {
    it('sets loading state to true', () => {
      useAppStore.getState().setComparisonLoading(true);

      const state = useAppStore.getState();
      expect(state.comparisonLoading).toBe(true);
    });

    it('sets loading state to false', () => {
      useAppStore.setState({ comparisonLoading: true });

      useAppStore.getState().setComparisonLoading(false);

      const state = useAppStore.getState();
      expect(state.comparisonLoading).toBe(false);
    });
  });

  describe('setComparisonPhase', () => {
    it('sets comparison phase', () => {
      useAppStore.getState().setComparisonPhase('analyzing');

      const state = useAppStore.getState();
      expect(state.comparisonPhase).toBe('analyzing');
    });

    it('clears comparison phase when null', () => {
      useAppStore.setState({ comparisonPhase: 'analyzing' });

      useAppStore.getState().setComparisonPhase(null);

      const state = useAppStore.getState();
      expect(state.comparisonPhase).toBeNull();
    });
  });

  describe('setComparisonError', () => {
    it('sets comparison error', () => {
      useAppStore.getState().setComparisonError('Comparison failed');

      const state = useAppStore.getState();
      expect(state.comparisonError).toBe('Comparison failed');
    });

    it('clears comparison error when null', () => {
      useAppStore.setState({ comparisonError: 'Error' });

      useAppStore.getState().setComparisonError(null);

      const state = useAppStore.getState();
      expect(state.comparisonError).toBeNull();
    });
  });

  describe('clearComparison', () => {
    it('clears all comparison state', () => {
      useAppStore.setState({
        comparisonResult: {
          comparison: 'Test',
          key_differences: [],
          sources: [],
          information_gaps: [],
        },
        comparisonLoading: true,
        comparisonPhase: 'analyzing',
        comparisonError: 'Error',
      });

      useAppStore.getState().clearComparison();

      const state = useAppStore.getState();
      expect(state.comparisonResult).toBeNull();
      expect(state.comparisonLoading).toBe(false);
      expect(state.comparisonPhase).toBeNull();
      expect(state.comparisonError).toBeNull();
    });
  });

  describe('setProviders', () => {
    it('sets LLM providers list', () => {
      const mockProviders = [
        {
          id: 'ollama',
          name: 'Ollama (Local)',
          models: [],
          is_available: true,
        },
        {
          id: 'anthropic',
          name: 'Anthropic',
          models: [],
          is_available: false,
        },
      ];

      useAppStore.getState().setProviders(mockProviders);

      const state = useAppStore.getState();
      expect(state.providers).toEqual(mockProviders);
      expect(state.providers).toHaveLength(2);
    });
  });

  describe('LLM provider/model selection', () => {
    it('sets selected provider', () => {
      useAppStore.getState().setSelectedProvider('anthropic');

      const state = useAppStore.getState();
      expect(state.selectedProvider).toBe('anthropic');
    });

    it('sets selected model', () => {
      useAppStore.getState().setSelectedModel('claude-3-5-sonnet-20241022');

      const state = useAppStore.getState();
      expect(state.selectedModel).toBe('claude-3-5-sonnet-20241022');
    });

    it('clears provider and model when null', () => {
      useAppStore.setState({
        selectedProvider: 'ollama',
        selectedModel: 'qwen2.5:7b',
      });

      useAppStore.getState().setSelectedProvider(null);
      useAppStore.getState().setSelectedModel(null);

      const state = useAppStore.getState();
      expect(state.selectedProvider).toBeNull();
      expect(state.selectedModel).toBeNull();
    });
  });
});
