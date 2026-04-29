export interface User {
  id: number;
  email: string;
  full_name: string;
  role: 'admin' | 'user';
  created_at: string;
}

export interface KnowledgeBase {
  id: string;
  name: string;
  description?: string;
  created_at: string;
  doc_count?: number;
  chunk_count?: number;
}

export interface Document {
  id: string;
  kb_id: string;
  source_name: string;
  mime_type: string;
  status: 'processing' | 'indexed' | 'error';
  indexed_at?: string;
  created_at: string;
  tags?: string[];
}

export interface SearchResult {
  chunk_id: string;
  doc_id: string;
  text_quote: string;
  confidence: number;
  metadata?: Record<string, any>;
}

export interface QueryResponse {
  answer: string;
  sources: SearchResult[];
  verified: boolean;
  processing_time_ms: number;
  information_gaps?: string[];
}

export interface ComparisonCell {
  doc_id: string;
  present: boolean;
  text_quote?: string;
  confidence?: number;
  verified: boolean;
}

export interface ComparisonAspect {
  aspect_name: string;
  cells: ComparisonCell[];
}

export interface ComparisonResult {
  aspects: ComparisonAspect[];
  key_differences: string[];
  recommendations?: string[];
  information_gaps: string[];
  verified: boolean;
  processing_time_ms: number;
}

export interface IngestResponse {
  doc_id: string;
  chunk_count: number;
  processing_time_ms: number;
  status: 'indexed' | 'error';
}

export interface LLMProvider {
  id: string;
  name: string;
  enabled: boolean;
  is_local: boolean;
  models: LLMModel[];
  requires_api_key: boolean;
  has_api_key: boolean;
}

export interface LLMModel {
  id: string;
  name: string;
  provider: string;
  cost_per_1k_input?: number;
  cost_per_1k_output?: number;
  context_length: number;
}

export interface CostTracking {
  today: number;
  this_month: number;
  budget_daily: number;
  budget_monthly: number;
}
