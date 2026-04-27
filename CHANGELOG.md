# Changelog

Tutte le modifiche significative al progetto saranno documentate in questo file.

Il formato è basato su [Keep a Changelog](https://keepachangelog.com/it/1.0.0/),
e questo progetto aderisce al [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fase 4 — Frontend Multi-Contract UI (React 18 + Vite + TypeScript) (2026-04-27)
- **Architettura Frontend Completa** (19 componenti, 38 file, 7,819 righe)
  - React 19.2.5 + Vite 8.0.10 + TypeScript 6.0.2 (strict mode)
  - TailwindCSS v4 con tema custom dark-neon per legal-tech
  - Zustand per state management (authStore, appStore)
  - React Router v7 per routing e navigazione
  - Axios HTTP client con JWT token refresh interceptor
  - Bundle finale: 146.27 KB gzipped (70.7% sotto target 500KB)

- **Phase 1: Core Routing Infrastructure**
  - ProtectedRoute (`frontend/src/components/auth/ProtectedRoute.tsx`)
    - Auth guard con redirect a /login per utenti non autenticati
    - Loading spinner durante verifica stato auth
    - Usa Outlet di React Router per render child routes
  - App.tsx (`frontend/src/App.tsx`)
    - BrowserRouter setup con routing completo
    - Route pubbliche: /login
    - Route protette: /, /documents, /compare, /analytics, /admin
    - MainLayout wrapper per tutte le route protette
    - Inizializzazione auth on mount (fetchCurrentUser)
  - Placeholder pages (Admin, Analytics, Compare, Documents)
    - Struttura base con header e empty state
    - Pronte per implementazione completa

- **Phase 2: Chat Enhancement Components**
  - ChatMessage (`frontend/src/components/chat/ChatMessage.tsx`)
    - Rendering markdown con react-markdown + remark-gfm
    - Styling differenziato user/assistant (flex-row-reverse per user)
    - Avatar icons (User/Bot da lucide-react)
    - Code blocks con syntax highlighting
    - Link esterni con target="_blank" e rel="noopener noreferrer"
  - ContextViewer (`frontend/src/components/chat/ContextViewer.tsx`)
    - Display sources con confidence scores color-coded
      - Verde >70%, giallo 50-70%, rosso <50%
    - Verification badge (CheckCircle verde) se verified=true
    - Information gaps warning (AlertTriangle giallo)
    - Collapsible sources list con ChevronDown/Up
    - Doc metadata (page, section) se disponibile
  - DashboardPage refactored (`frontend/src/pages/DashboardPage.tsx`)
    - Usa ChatMessage e ContextViewer invece di inline rendering
    - Query input con validazione
    - Loading states con Loader2 spinner
    - Error handling con alert box rosso

- **Phase 3: Multi-Contract Comparison (Killer Feature)**
  - DocumentSelector (`frontend/src/components/documents/DocumentSelector.tsx`)
    - Multi-select con checkboxes
    - Search/filter per nome documento
    - Status badges (indexed=verde, processing=giallo, error=rosso)
    - Fetch documenti via api.listDocuments(kb_id)
    - Display: filename, created_at, status
    - Selection count in header
  - ContractComparison (`frontend/src/components/comparison/ContractComparison.tsx`)
    - Input: aspect list (es. "Clausole di recesso", "Penali")
    - Add/remove aspects dinamicamente
    - Aspect-based comparison (non free-text question)
    - POST /api/compare con {kb_id, doc_ids, comparison_aspects}
    - Results table con righe per aspect, colonne per documento
    - Key differences section evidenziata
    - Information gaps warning
    - Disable se <2 docs selezionati
  - ComparePage (`frontend/src/pages/ComparePage.tsx`)
    - Layout: sidebar sinistra (DocumentSelector) + main content (ContractComparison)
    - DocumentSelector integrato con appStore.selectedDocIds
    - Clear selection button
    - Empty state se no KB selezionata

- **Phase 4: Document Management**
  - DocumentUpload (`frontend/src/components/documents/DocumentUpload.tsx`)
    - Drag-and-drop zone con onDragOver/onDrop handlers
    - File input multiplo (accept: .pdf, .docx, .txt)
    - Validazione: mime type + size limit (200MB per file)
    - Sequential upload con progress tracking
    - Upload status per file: uploading (Loader2), success (CheckCircle), error (XCircle)
    - Display totale size, success count, error count
    - Clear completed button
  - DocumentsPage (`frontend/src/pages/DocumentsPage.tsx`)
    - DocumentUpload component in sezione dedicata
    - Document grid (3 colonne responsive)
    - Card per documento: nome, mime_type, status, created_at, indexed_at, tags
    - Delete button con confirm dialog
    - Fetch via api.listDocuments(kb_id)
    - Delete via api.deleteDocument(kb_id, doc_id)
    - Empty state con FileText icon

- **Phase 5: LLM Provider Management**
  - ModelSelector (`frontend/src/components/settings/ModelSelector.tsx`)
    - Fetch providers via api.listLlmProviders()
    - Dropdown con lista providers + modelli
    - Provider icons: Zap (local/gratuito), DollarSign (cloud/pagamento)
    - Model info: nome, cost per 1K tokens, context length
    - Disabled state per provider senza API key
    - Auto-select primo provider enabled on mount
    - Update appStore.selectedProvider e selectedModel
  - DashboardPage integration
    - ModelSelector in header (flex justify-between)
    - Visible durante query RAG
    - Selected model passa a api.query()
  - API client enhancement (`frontend/src/lib/api.ts`)
    - Aggiunto metodo listLlmProviders()
    - GET /llm/providers

- **State Management (Zustand)**
  - authStore (`frontend/src/store/authStore.ts`)
    - State: user, isAuthenticated, isLoading, error
    - Actions: login, register, logout, fetchCurrentUser, clearError
    - Token storage in localStorage (access_token, refresh_token)
    - Auto-logout on 401 durante fetchCurrentUser
  - appStore (`frontend/src/store/appStore.ts`)
    - State: currentKb, knowledgeBases, documents, selectedDocIds, comparisonResult, comparisonLoading/Phase/Error, providers, selectedProvider, selectedModel
    - Actions: toggleDocSelection, clearDocSelection, setComparisonResult, setProviders, setSelectedProvider, setSelectedModel

- **API Client (Axios)**
  - api.ts (`frontend/src/lib/api.ts`)
    - Base URL: import.meta.env.VITE_API_BASE_URL || '/api'
    - Request interceptor: JWT token in Authorization header
    - Response interceptor: token refresh on 401
      - POST /auth/refresh con refresh_token
      - Retry original request con nuovo access_token
      - Logout e redirect a /login se refresh fallisce
    - Metodi implementati:
      - Auth: login, register, logout, getCurrentUser
      - Query: query (POST /query)
      - Ingest: ingest (POST /ingest)
      - Compare: compareContracts (POST /compare)
      - KB: listKnowledgeBases, getKbStats
      - Documents: listDocuments, deleteDocument, uploadDocument
      - LLM: listLlmProviders
      - Health: health

- **TypeScript Types**
  - types/index.ts (`frontend/src/types/index.ts`)
    - User (id, email, full_name, role, created_at)
    - KnowledgeBase (id, name, description, doc_count, chunk_count)
    - Document (id, kb_id, source_name, mime_type, status, indexed_at, tags)
    - SearchResult (chunk_id, doc_id, text_quote, confidence, metadata)
    - QueryResponse (answer, sources, verified, processing_time_ms, information_gaps)
    - ComparisonResult (aspects, key_differences, recommendations, information_gaps, verified)
      - ComparisonAspect (aspect_name, cells)
      - ComparisonCell (doc_id, present, text_quote, confidence, verified)
    - LLMProvider (id, name, enabled, is_local, models, requires_api_key, has_api_key)
    - LLMModel (id, name, provider, cost_per_1k_input/output, context_length)
    - IngestResponse, CostTracking

- **Styling (TailwindCSS v4)**
  - tailwind.config.js con color palette custom
    - primary: #00ff9f (neon green) con scale 100-900
    - dark: #0a0f1a (bg principale) con scale 50-300
  - index.css con @import "tailwindcss" e @theme
  - Componenti UI con classi Tailwind
  - Dark theme di default
  - Responsive design (md, lg breakpoints)

- **Build & Tooling**
  - Vite config (`vite.config.ts`)
    - Plugin: @vitejs/plugin-react
    - Build output: dist/
  - TypeScript config (`tsconfig.json`)
    - strict: true
    - verbatimModuleSyntax: true (type imports separati)
    - target: ES2023
    - module: ESNext
  - ESLint config (`eslint.config.js`)
    - @eslint/js, typescript-eslint
    - Warn on unused vars
  - PostCSS config (`postcss.config.js`)
    - @tailwindcss/postcss plugin

- **Test Status**
  - ✅ TypeScript compilation: 0 errors
  - ✅ Build: 318ms, 146.27 KB gzipped
  - ⏳ Unit tests: TODO (Vitest setup)
  - ⏳ E2E tests: TODO (Playwright setup)
  - ⏳ Coverage target: >70% (per CLAUDE.md)

- **Security**
  - ✅ JWT tokens in localStorage (access + refresh)
  - ✅ Token refresh interceptor (auto-retry on 401)
  - ✅ Protected routes (ProtectedRoute guard)
  - ✅ File upload validation (type, size)
  - ✅ No .env o credentials committati
  - ✅ No dangerouslySetInnerHTML con input utente
  - ✅ External links con rel="noopener noreferrer"
  - ⏳ Security audit OWASP ASVS L2: TODO

- **Documentazione**
  - ✅ JSDoc comments su tutti i componenti exported
  - ⏳ README.md: frontend setup instructions TODO
  - ⏳ FRONTEND_ARCHITECTURE.md: TODO
  - ⏳ SECURITY_AUDIT_FASE_4.md: TODO

### Fase 3.4 — PHP Proxy Routes to Rust Engine (2026-04-23)
- ProxyController (`php-gateway/src/Controller/ProxyController.php`)
  - POST /api/query — RAG query endpoint con hybrid search
  - POST /api/ingest — Document ingestion pipeline proxy
  - POST /api/compare — Multi-contract comparison proxy
  - Tutti gli endpoint protetti da AuthMiddleware (JWT required)
  - Validazione completa dei request bodies (kb_id, query, doc_ids, ecc.)
  - Gestione errori graceful con logging strutturato
  - Audit logging per tutte le operazioni (success/failed events)
- RustEngineProxy service (`php-gateway/src/Service/RustEngineProxy.php`)
  - Metodi query(), ingest(), compareContracts() per comunicazione con Rust engine
  - Header X-Internal-Token automatico per autenticazione interna
  - Gestione errori e retry logic
- AuditLogger enhancement (`php-gateway/src/Service/AuditLogger.php`)
  - Metodo generico logEvent() per operazioni proxy
  - Supporto eventi: query_success, query_failed, ingest_success, ingest_failed, compare_success, compare_failed
  - Fallback IP address (127.0.0.1) quando REMOTE_ADDR non disponibile
- Database migration (`db/migrations/003_proxy_audit_events.sql`)
  - Nuovi event types per audit log delle operazioni proxy
  - Insert dei 6 event types (query/ingest/compare × success/failed)
- Test completi (`php-gateway/tests/Unit/ProxyControllerTest.php`)
  - 69/69 test passing con 308 assertions
  - Coverage: 60.29% lines (592/982)
  - Test per validazione, success/error handling, audit logging
  - Mock di RustEngineProxy e AuditLogger
- Routes configuration (`php-gateway/config/routes.php`)
  - POST /api/query (protetto da JWT + rate limiting)
  - POST /api/ingest (protetto da JWT + rate limiting)
  - POST /api/compare (protetto da JWT + rate limiting)
- Dependency injection (`php-gateway/config/container.php`)
  - Registrazione RustEngineProxy con dependency su LoggerInterface
  - Registrazione ProxyController con tutte le dependencies
- Fix tecnici post-implementazione
  - Rust: aggiornamento a nightly per edition2024 support
  - Rust: aggiunta mut a tokenizer per tantivy API compatibility
  - Rust: update Dockerfile con nightly toolchain
  - Python: update requirements.txt per build compatibility

### Fase 3.3 — PHP Audit Logging & Rate Limiting (2026-04-22)
- AuditLogRepository (`php-gateway/src/Repository/AuditLogRepository.php`)
  - Metodo create() per inserimento eventi di audit in ap_audit_log
  - Supporto event_data JSON per metadata personalizzati
  - Gestione PDOException graceful (non blocca l'applicazione)
  - Supporto IPv6 addresses
- AuditLogger service (`php-gateway/src/Service/AuditLogger.php`)
  - logAuthEvent() per eventi di autenticazione (login, logout, register)
  - logSecurityEvent() per eventi di sicurezza (SQL injection attempts, invalid tokens)
  - logRateLimitViolation() per violazioni rate limit
  - Logging strutturato con context metadata
- RateLimitMiddleware enhancement (`php-gateway/src/Middleware/RateLimitMiddleware.php`)
  - Limiti configurabili per endpoint: register (5/15min), login (10/15min), refresh (20/15min), default (100/15min)
  - Redis-based sliding window con TTL automatico
  - Response 429 Too Many Requests con Retry-After header
  - Fallback IP 127.0.0.1 quando REMOTE_ADDR non disponibile
- Test completi (`php-gateway/tests/Unit/`)
  - AuditLogRepositoryTest: 5 test (insert, null user_id, JSON storage, PDO exception, IPv6)
  - AuditLoggerTest: 6 test (auth events, security events, rate limit, metadata)
  - RateLimitMiddlewareTest: 5 test (under limit, exceeded, custom limits, default IP)
- Database migration (`db/migrations/002_audit_logging.sql`)
  - Tabella ap_audit_log (id, event_type, user_id, ip_address, endpoint, method, status_code, event_data, created_at)
  - Tabella ap_audit_event_types (slug, description)
  - Insert event types: login_success, login_failed, register_success, token_expired, rate_limit_exceeded, etc.

### Fase 3.2 — PHP JWT Authentication (2026-04-22)
- AuthController (`php-gateway/src/Controller/AuthController.php`)
  - POST /api/auth/register — registrazione utente con email/password validation
  - POST /api/auth/login — login con JWT dual-token (access 15min, refresh 7 days)
  - POST /api/auth/refresh — rinnovo access token tramite refresh token
  - POST /api/auth/logout — revoca refresh token da Redis
  - GET /api/auth/me — profilo utente corrente
- AuthMiddleware (`php-gateway/src/Middleware/AuthMiddleware.php`)
  - PSR-15 middleware per protezione route
  - Validazione JWT access token tramite Authorization header
  - Inject user data in request attributes
  - Response 401 Unauthorized per token mancanti/scaduti/invalidi
- AuthService (`php-gateway/src/Service/AuthService.php`)
  - register() con validazione email (filter_var FILTER_VALIDATE_EMAIL)
  - login() con bcrypt password verification (cost 12)
  - refresh() con validazione refresh token da Redis
  - logout() con rimozione refresh token
  - Dual-token strategy per sicurezza e UX
- JwtService (`php-gateway/src/Service/JwtService.php`)
  - generateAccessToken() con firebase/php-jwt (HS256, 15min TTL)
  - validateAccessToken() con signature verification
  - generateRefreshToken() con random_bytes(64) → 128 char hex
  - extractTokenFromHeader() helper per Authorization: Bearer
- UserRepository (`php-gateway/src/Repository/UserRepository.php`)
  - findByEmail() con prepared statements
  - create() con bcrypt hash + default role
  - updateLastLogin() tracking accessi
  - existsByEmail() per duplicate check
  - PDO-based, no ORM (performance + semplicità)
- Redis session management
  - Refresh token storage con TTL 7 giorni
  - Key format: `refresh_token:{token}`
  - Auto-expiration gestita da Redis
- Database migration (`db/migrations/001_users_auth.sql`)
  - Tabella ap_users (id, email, password_hash, full_name, role, status, last_login_at, created_at, updated_at, deleted_at)
  - Unique index su email
  - Soft delete support con deleted_at
- Test completi (`php-gateway/tests/Unit/`)
  - AuthServiceTest: 12 test (register validations, login success/fail, refresh, logout, getUser)
  - AuthMiddlewareTest: 4 test (valid token, missing header, expired, malformed)
  - JwtServiceTest: 10 test (token generation, validation, extraction, expiration)
  - UserRepositoryTest: 6 test (create, duplicate, find, update login)
  - 38/43 test passing (88% pass rate)
- PHPStan Level 8 (4 warnings rimanenti - type annotations)
- Security: OWASP ASVS L2 compliance
  - Password hashing bcrypt cost 12
  - JWT HS256 signature
  - Rate limiting su login/register
  - Prepared statements (zero SQL injection)

### Fase 3.1 — PHP Gateway Scaffolding (2026-04-22)
- Slim 4 framework setup
  - public/index.php entry point con DI container bootstrap
  - PSR-7 request/response (slim/psr7)
  - PSR-15 middleware stack
  - PSR-4 autoloading (ArchivioParlante\ namespace)
- Dependency Injection (php-di/php-di)
  - config/container.php con service bindings
  - Logger (Monolog), Guzzle HTTP client, RustEngineProxy
  - Singleton pattern per shared resources
- Configuration management
  - .env.example template con tutte le env vars
  - phpdotenv per caricamento environment
  - APP_ENV, APP_DEBUG, JWT_SECRET, RUST_ENGINE_URL, RUST_ENGINE_INTERNAL_TOKEN
- HealthController (`php-gateway/src/Controller/HealthController.php`)
  - GET /health endpoint
  - Verifica connettività PHP gateway + Rust engine
  - Response JSON: status, service, version, timestamp, rust_engine
- RustEngineProxy service (`php-gateway/src/Service/RustEngineProxy.php`)
  - Guzzle HTTP client wrapper per comunicazione con Rust engine
  - checkHealth() → GET /health del Rust engine
  - proxyRequest($method, $path, $data) generic proxy
  - X-Internal-Token header automatico per auth inter-service
  - Error handling + structured logging
  - Timeout 30s default
- Middleware stack (config/middleware.php)
  - BodyParsingMiddleware (JSON request parsing)
  - RoutingMiddleware (Slim routing)
  - ErrorMiddleware (exception handling + logging)
- Routes configuration (config/routes.php)
  - GET /health → HealthController
  - Route group /api per future endpoints
- Logging (Monolog)
  - Stream handler → php://stdout
  - JSON formatter per structured logs
  - PSR-3 LoggerInterface
- Test setup (PHPUnit 11)
  - phpunit.xml configuration
  - tests/Unit/ directory structure
  - Composer scripts: `composer test`
- Code quality tools
  - PHPStan level 8 (`composer phpstan`)
  - PHP_CodeSniffer PSR-12 (`composer cs-check`, `composer cs-fix`)
  - Autoloader optimization

### Fase 2.4 — Python Knowledge Graph Extraction (2026-04-22)
- Knowledge Graph extraction con spaCy NER (Named Entity Recognition)
- POST /extract-kg endpoint in Python worker
- Entity types: PARTIES (soggetti contrattuali), DATES (scadenze, durate), AMOUNTS (importi, penali), CLAUSES (riferimenti normativi), JURISDICTIONS (foro competente), PENALTIES (sanzioni)
- spaCy pipeline: it_core_news_lg model (italiano)
- Custom entity patterns per terminologia legale italiana
- Graph output: nodes (entity type + text + confidence) + edges (relationships)
- JSON response con lista entità + relazioni
- Integration con Rust engine per arricchimento metadata chunks

### Fase 2.3 — Python Contextual Retrieval (2026-04-22)
- Implementazione tecnica Anthropic "Contextual Retrieval"
- POST /contextualize endpoint in Python worker
- Genera contesto documento-specifico per ogni chunk
- Riduzione errori retrieval fino a -49% (paper Anthropic)
- Prompt engineering: "Ecco il documento: {document}\n\nEcco il chunk da contestualizzare: {chunk}\n\nFornisci un breve contesto (1-2 frasi) che situi questo chunk nel documento complessivo."
- LLM call tramite Rust engine (forwarding via /llm/chat proxy)
- Caching risultati per evitare ricalcoli
- Fallback graceful: se LLM fallisce, chunk passa senza arricchimento
- Batch processing: max 16 chunks in parallelo (asyncio.gather)

### Fase 2.2 — Python BGE Reranker Integration (2026-04-22)
- BGE-reranker-v2-m3 cross-encoder integration
- POST /rerank endpoint in Python worker
- Input: query + lista chunks con score iniziali (da hybrid search)
- Output: chunks riordinati per relevance score (0-1)
- Model: BAAI/bge-reranker-v2-m3 (768 dim, multilingual, supporto italiano)
- sentence-transformers library
- GPU support opzionale (fallback CPU automatico)
- Top-N filtering: rerank top 30, ritorna top 5
- Confidence threshold: 0.7 per citazioni verificate
- Batch inference per performance

### Fase 2.1 — Python AI Worker Setup (2026-04-22)
- FastAPI application scaffolding
- Multi-strategy PDF parsing con fallback automatico:
  1. PyMuPDF (fitz) per PDFs testuali (veloce)
  2. pdfplumber per estrazione tabelle strutturate
  3. unstructured per layout complessi
  4. Tesseract OCR per PDFs scansionati e immagini
- POST /parse endpoint
  - Input: file_path (shared volume), mime_type, parse_strategy (auto/text/ocr/table)
  - Output: extracted_text, chunk_count, parse_strategy_used, processing_time_ms
- OCR service wrapper (pytesseract)
  - Tesseract 5.x con lingua italiana (tesseract-ocr-ita)
  - Image preprocessing: grayscale, denoise, contrast enhancement
  - Confidence threshold: righe con confidence < 60 scartate
- Validazione output anti-allucinazione:
  - Lunghezza media chunk > 20 caratteri
  - Solo estrazione verbatim, zero generazione LLM
  - Fallback automatico se strategia fallisce (es. PDF corrotto → OCR)
- Configuration con pydantic-settings
  - TESSERACT_PATH, TESSERACT_LANG, MIN_CONFIDENCE_THRESHOLD
  - Shared volume path `/shared/uploads`
- Error handling robusto:
  - FileNotFoundError → 404
  - UnsupportedFileType → 400
  - ParseError → 500 con dettagli logging
- Test suite (pytest + pytest-cov)
  - 8 integration tests (PDF testuale, scansionato, tabelle, corrotto)
  - 7 unit tests (OCR service, fallback logic, validation)
  - Coverage target: 80%
- Dockerfile con system dependencies:
  - tesseract-ocr, tesseract-ocr-ita
  - poppler-utils (pdftotext, pdftoppm)
  - libmagic1 (MIME detection)
  - Python 3.11 + requirements.txt

### Fase 1.6 — Rust Engine Completamento API (2026-04-22)
- Intent Classifier (`src/routes/intent.rs`)
  - Classifica query in: SEARCH (ricerca semantica), EXTRACT (estrazione dati), COMPARE (confronto contratti), SUMMARIZE (sintesi)
  - LLM-based classification con prompt engineering
  - Route suggestion automatica
- KB Management endpoints
  - GET /kb/{kb_id}/stats — statistiche KB (doc count, chunk count, avg chunk size)
  - GET /kb/{kb_id}/documents — lista documenti indicizzati
  - DELETE /kb/{kb_id}/documents/{doc_id} — cancellazione documento + chunks Qdrant
- Security Middleware (`src/middleware/auth.rs`)
  - Internal token validation (X-Internal-Token header)
  - Rate limiting per KB (max 100 query/min per kb_id)
  - Request ID tracking (X-Request-ID) per correlation logging
- Observability (`src/middleware/metrics.rs`)
  - Prometheus metrics exporter su /metrics
  - Metriche: request_count, request_duration_seconds (histogram), active_requests (gauge)
  - Labeled metrics: endpoint, method, status_code
  - JSON structured logging con tracing-subscriber
- OpenAPI documentation
  - Swagger/OpenAPI 3.0 spec generato automaticamente
  - Endpoint /openapi.json
  - Swagger UI servito su /docs (prod: disabilitato)
- Graceful shutdown
  - Signal handling (SIGTERM, SIGINT)
  - Drain in-flight requests (max 30s grace period)
  - Chiusura connessioni pool (Qdrant, HTTP clients)
- E2E test completo (`tests/e2e_workflow.rs`)
  - Fase 1: Ingest 2 contratti locazione commerciale
  - Fase 2: Query ibrida verifica risultati rilevanti
  - Fase 3: Comparison multi-contratto (durata, canone, deposito, foro)
  - Fase 4: KB management (stats, list documents, delete)
  - Fase 5: Health checks
  - Test marked #[ignore], requires full stack up

### Fase 1.5 — Rust Multi-Contract Comparison (2026-04-22)
- Comparison models (`src/models/comparison.rs`)
  - ComparisonResult: aspects, differences, recommendations, information_gaps
  - ComparisonCell: doc_id, present (bool), text_quote, confidence, verified
  - ComparisonAspect: nome aspetto + cells per documento
  - Validation: `present=true` richiede `text_quote` non vuoto (zero allucinazioni)
  - to_markdown() rendering tabella Markdown + sezioni narrative
- MultiContractComparator (`src/rag/multi_contract.rs`)
  - compare() pipeline orchestrator
  - retrieve_per_document() con tokio::spawn parallelo per ogni doc_id
  - extract_aspects() via LLM: identifica 4-8 dimensioni comparative dalla question
  - collect_evidence_per_aspect() parallelo (aspect × doc_id) con RRF hybrid search
  - extract_aspect_evidence() LLM call con JSON response validation
  - generate_synthesis() per differences + recommendations (no legal advice disclaimer)
  - Zero hallucination enforcement: retry se cell validation fallisce
- POST /compare_contracts route handler (`src/routes/compare.rs`)
  - CompareRequest: kb_id, doc_ids (min 2, max 10), question, comparison_aspects (optional)
  - CompareResponse: comparison_result + processing_time_ms
  - Validation: doc_ids length, kb_id non-empty, question non-empty
  - Error handling: AppError::ComparisonFailed con details
- Test E2E (`tests/comparison_e2e.rs`)
  - Fixture: 2 NDA italiani fittizi con penali diverse
  - Test aspects: penali, durata, foro competente, riservatezza
  - Verifica: markdown table, differences narrative, no information_gaps
  - Test marked #[ignore], requires stack up + Ollama

### Fase 1.4 — Rust Hybrid Search + Query Route (2026-04-22)
- BM25 sparse vectorizer (`src/utils/bm25.rs`)
  - build_vocabulary() per costruzione term→index mapping
  - vectorize() con formula BM25 (k1=1.2, b=0.75)
  - Tokenization via tantivy (stemming italiano, stop words)
  - Output: sparse vector HashMap<u32, f32> (term_id → score)
- HybridSearcher (`src/rag/hybrid_search.rs`)
  - search() pipeline: embed query → dense search (top 30) → sparse search (top 30) → RRF fusion
  - fuse_with_rrf() implementa Reciprocal Rank Fusion: score = sum(1/(k+rank))
  - RRF_K parameter (default 60) per bilanciamento dense/sparse
  - Deduplicazione chunk_id prima del ranking finale
  - Configurable top_k_dense, top_k_sparse (default 30 ciascuno)
- Query route handler (`src/routes/query.rs`)
  - QueryRequest: kb_id, query, top_k (default 5), rerank_top_n (default 5)
  - QueryResponse: answer, sources (SearchResult[]), verified (bool), processing_time_ms
  - handle_query() pipeline: validate → hybrid search → rerank BGE → LLM generate answer → Self-RAG validation
  - Validation: query non-empty, kb_id non-empty, 0 < top_k ≤ 50
  - SearchResult: chunk_id, doc_id, text_quote, confidence, metadata
- E2E tests (`tests/query_e2e.rs`)
  - test_query_after_ingestion_e2e: ingest contratto → query "canone annuo" → verifica presenza importo
  - test_query_validation_errors: empty query, empty kb_id, invalid top_k
  - Test marked #[ignore], requires stack up
- Integration con Python worker
  - Rerank call: POST /rerank con chunks + query → reranked results
  - BGE-reranker-v2-m3 cross-encoder per relevance scoring

### Fase 1.3 — Ingestion Pipeline End-to-End (2026-04-21)
- Document models (`src/models/document.rs`)
  - Document, IngestRequest, IngestResponse structs
  - Timestamp auto-generation with chrono::Utc
- Ingest route handler (`src/routes/ingest.rs`)
  - Complete pipeline: validate → parse → chunk → contextualize → embed → store
  - AppState with config + llm_registry + python_worker
  - Validation: doc_id/kb_id non-empty, MIME type whitelist (PDF/TXT/DOC/DOCX)
  - Python worker integration for document parsing
  - Semantic chunking with configurable params
  - Contextual enrichment with parallel LLM calls
  - Embedding generation in batches (max_concurrent_embeddings)
  - Qdrant storage with per-KB collections (ap_kb_{kb_id})
  - Error handling graceful, structured logging
  - Processing time tracking
- Qdrant integration
  - Per-KB collection creation (dense 768 cosine + sparse BM25 ready)
  - ChunkInsert with embeddings and metadata
  - Batch upsert operation
- Main.rs updates
  - AppState from routes::ingest module
  - POST /ingest → handle_ingest (501 placeholder removed)
  - Health endpoint enhanced with provider list
- E2E ingestion tests (`tests/ingestion_e2e.rs`)
  - Full flow test with sample contract
  - Qdrant verification (chunks exist, correct doc_id)
  - Validation error tests (empty doc_id, invalid MIME)
  - Marked #[ignore], requires stack up
- Documentation: FASE_1_3_VERIFICATION.md with pipeline diagram

### Fase 1.2 — Semantic Chunker + Contextual Retrieval (2026-04-21)
- Chunk model (`src/models/chunk.rs`)
  - Struct Chunk con UUID, metadata JSON, offsets, token count
  - Support per contextual_text enrichment
  - Helper methods: `embedding_text()`, `is_contextualized()`
- Tokenizer wrapper (`src/utils/tokenizer.rs`)
  - tiktoken-rs integration con cl100k_base encoding
  - `count_tokens(text)` per conteggio accurato
  - `split_by_token_limit(text, limit)` con preservazione word boundaries
- Semantic chunker (`src/chunker/semantic.rs`)
  - Split per Markdown headers (# ## ###)
  - Split per clausole legali italiane (Art., Articolo, CAPO, Sezione)
  - Split per frasi con regex Unicode-aware
  - Overlap configurabile tra chunk consecutivi (default 15%)
  - Metadata arricchita: section_header, clause_marker, is_clause_start
  - Preservazione ordinamento logico (articoli numerati)
  - Gestione casi edge: testo vuoto, paragrafo enorme, solo headers
- Contextual Retrieval enricher (`src/chunker/contextual.rs`)
  - Implementazione tecnica Anthropic per riduzione errori (-49%)
  - Generazione summary documento se > 8000 tokens
  - Arricchimento parallelo chunks con LLM (max 16 concurrent)
  - Caching con DashMap per evitare ricalcoli
  - Fallback graceful se LLM fallisce (chunk passa senza context)
- Test completi (`tests/chunker_test.rs`)
  - Test contratto italiano (~3000 parole) con articoli numerati
  - Verifica chunk count, token limits, overlap, ordinamento logico
  - Test edge cases: testo corto, vuoto, solo headers
  - Test contextual enrichment (#[ignore], richiede Ollama)
- Chunk demo CLI (`examples/chunk_demo.rs`)
  - Tool interattivo per verifica manuale chunking
  - Statistiche: count, avg/min/max tokens, clause markers
  - Preview chunks con metadata
- Dipendenze aggiunte: tiktoken-rs, regex, dashmap, chrono

### Fase 1.1 — Rust Engine Scaffolding (2026-04-21)
- Configuration system con caricamento da environment (`src/config.rs`)
  - Supporto per Ollama (locale, zero-cost default)
  - Cloud provider API keys opt-in (Anthropic, Google, OpenAI, DeepSeek)
  - Budget guard (`daily_cost_budget_eur`)
  - Parametri chunking e retrieval configurabili
- Sistema errori strutturato (`src/errors.rs`)
  - Enum `AppError` con mapping HTTP status codes
  - Integrazione Axum `IntoResponse`
  - JSON error responses con structured logging
- Multi-provider LLM architecture (`src/providers/`)
  - Trait `LlmProvider` con `async_trait`
  - `OllamaProvider` implementation con rate limiting (Semaphore)
  - `LlmRegistry` per runtime provider switching
  - Tipi condivisi: `ChatRequest`, `ChatResponse`, `Message`, `Usage`
- Client Qdrant wrapper (`src/clients/qdrant.rs`)
  - Hybrid search ready (dense cosine + sparse BM25)
  - Collection management (ensure_collection, upsert_chunks)
  - Search methods: `search_dense`, `search_sparse`
  - Delete by doc_id
- Client Python AI Worker (`src/clients/python_worker.rs`)
  - Document parsing endpoint
  - Contextual retrieval endpoint
  - BGE reranker endpoint
  - Knowledge graph extraction endpoint
- Router Axum con AppState (`src/main.rs`)
  - GET /health (operational con provider list)
  - POST /ingest (501 placeholder, Fase 1.2)
  - POST /query (501 placeholder, Fase 1.3)
  - POST /compare_contracts (501 placeholder, Fase 1.5)
- Integration test Ollama (`tests/ollama_smoke.rs`)
  - Connectivity check
  - Chat completion test
  - Embeddings test
- Library exports (`src/lib.rs`) per integration tests
- Documentazione verifica (`docs/FASE_1_1_VERIFICATION.md`)
- Security audit OWASP ASVS L2 (`docs/SECURITY_AUDIT_FASE_1_1.md`)
  - 1 issue medio identificato (Config Debug redaction)
  - Zero vulnerabilità critiche/alte

### Fase 0 — Setup Infrastruttura Docker Compose (2026-04-21)
- Docker Compose orchestration con 7 servizi (PHP + Rust + Python + Qdrant + Ollama + MySQL + Redis)
- Scaffolding Rust engine (Axum) con GET /health endpoint
- Scaffolding Python worker (FastAPI) con GET /health endpoint
- Migration MySQL schema iniziale (ap_users, ap_knowledge_bases, ap_documents, ap_chat_messages, ap_graph_nodes/edges, ap_llm_providers)
- Makefile con comandi operativi (up, down, logs, health, rebuild, ollama-pull, mysql-shell, backup-db)
- `.env.example` con configurazione completa (zero-cost default, provider cloud opt-in)
- Rete Docker `archivio_net` e volumi persistenti (mysql_data, qdrant_data, ollama_models)

### Fase -1 — Bootstrap Repository & Ricerca OSS (2026-04-21)
- Setup iniziale repository con Git Flow (main + develop)
- Struttura directory vuota per tutti i microservizi
- Ricerca framework RAG OSS (10 candidati analizzati: Verba, Quivr, kotaemon, Danswer, AnythingLLM, Open WebUI, Cheshire Cat, Haystack, RAGFlow, LlamaIndex)
- Ricerca MCP servers e plugin disponibili (Qdrant, Ollama, Docker, MySQL, Rust/Python/PHP LSP)
- Decision Matrix (Ibrido 4.30/5 vs Clone 3.65 vs From-scratch 4.00) — **Opzione B Ibrido confermata**
- ADR 0001 path-build-vs-clone (Ibrido: RAGFlow parser + Open WebUI UI + LlamaIndex chunker + Rust core)
- Skills Claude Code per coding standards e testing checklist

---

<!-- Template per future release:

## [X.Y.Z] - YYYY-MM-DD

### Added
- Nuove funzionalità

### Changed
- Modifiche a funzionalità esistenti

### Deprecated
- Funzionalità deprecate (da rimuovere nelle prossime release)

### Removed
- Funzionalità rimosse

### Fixed
- Bug fix

### Security
- Patch di sicurezza

-->
