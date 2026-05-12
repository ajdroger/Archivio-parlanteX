# 🎨 Frontend Architecture — Archivio Parlante

> React 18 + Vite + TypeScript + TailwindCSS v4 + Zustand

**Status**: Fase 4 completata (2026-04-27)  
**Bundle Size**: 146.27 KB gzipped (70.7% below 500KB target)  
**Build Time**: ~318ms  
**TypeScript Errors**: 0 (strict mode enabled)

---

## 📐 Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     Browser (SPA)                           │
│                                                             │
│  ┌────────────────────────────────────────────────────┐   │
│  │            React Router (v7)                       │   │
│  │  /login → LoginPage                                │   │
│  │  /      → <ProtectedRoute> → MainLayout            │   │
│  │    /documents     → DocumentsPage                  │   │
│  │    /compare       → ComparePage                    │   │
│  │    /analytics     → AnalyticsPage                  │   │
│  │    /admin         → AdminPage (role=admin)         │   │
│  └────────────────────────────────────────────────────┘   │
│                          ↓                                  │
│  ┌────────────────────────────────────────────────────┐   │
│  │         Zustand State Management                   │   │
│  │  • authStore (user, JWT, login/logout)             │   │
│  │  • appStore (KB, docs, comparison, LLM selection)  │   │
│  └────────────────────────────────────────────────────┘   │
│                          ↓                                  │
│  ┌────────────────────────────────────────────────────┐   │
│  │         Axios HTTP Client (lib/api.ts)             │   │
│  │  • JWT interceptor (Authorization header)          │   │
│  │  • Auto-refresh on 401 (refresh_token)             │   │
│  │  • Base URL: /api (proxied by Vite in dev)         │   │
│  └────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                          ↓ HTTP
┌─────────────────────────────────────────────────────────────┐
│              PHP Gateway (localhost:8080)                   │
│  POST /api/auth/login, /api/auth/register, etc.            │
│  POST /api/query, /api/ingest, /api/compare                │
│  GET  /llm/providers                                        │
└─────────────────────────────────────────────────────────────┘
```

---

## 🗂️ Directory Structure

```
frontend/
├── public/                     # Static assets
│   ├── favicon.svg
│   └── icons.svg               # Lucide icons sprite (optional)
│
├── src/
│   ├── components/
│   │   ├── auth/
│   │   │   └── ProtectedRoute.tsx        # Route guard (redirect to /login if !authenticated)
│   │   ├── chat/
│   │   │   ├── ChatMessage.tsx           # Markdown message display (user/assistant)
│   │   │   └── ContextViewer.tsx         # Sources with confidence scores, verification badges
│   │   ├── comparison/
│   │   │   └── ContractComparison.tsx    # Aspect-based multi-contract comparison UI
│   │   ├── documents/
│   │   │   ├── DocumentSelector.tsx      # Multi-select list with checkboxes, filter
│   │   │   └── DocumentUpload.tsx        # Drag-and-drop upload with progress tracking
│   │   ├── layout/
│   │   │   └── MainLayout.tsx            # Sidebar navigation wrapper
│   │   └── settings/
│   │       └── ModelSelector.tsx         # LLM provider/model dropdown selector
│   │
│   ├── pages/
│   │   ├── LoginPage.tsx                 # Login/register form
│   │   ├── DashboardPage.tsx             # RAG chat interface (main page)
│   │   ├── DocumentsPage.tsx             # Document management (upload, list, delete)
│   │   ├── ComparePage.tsx               # Multi-contract comparison page
│   │   ├── AnalyticsPage.tsx             # Usage analytics (placeholder)
│   │   └── AdminPage.tsx                 # Admin panel (KB/user management, placeholder)
│   │
│   ├── store/
│   │   ├── authStore.ts                  # Authentication state (Zustand)
│   │   └── appStore.ts                   # App state: KB, documents, comparison, LLM selection
│   │
│   ├── lib/
│   │   └── api.ts                        # Axios HTTP client with JWT interceptors
│   │
│   ├── types/
│   │   └── index.ts                      # TypeScript interfaces (User, Document, QueryResponse, etc.)
│   │
│   ├── App.tsx                           # React Router setup, auth initialization
│   ├── main.tsx                          # React entry point (ReactDOM.createRoot)
│   └── index.css                         # TailwindCSS v4 imports + @theme
│
├── dist/                       # Build output (gitignored)
├── node_modules/               # Dependencies (gitignored)
│
├── .env.local                  # Local env vars (gitignored)
├── .gitignore
├── eslint.config.js            # ESLint configuration
├── index.html                  # HTML entry point
├── package.json
├── package-lock.json
├── postcss.config.js           # PostCSS with @tailwindcss/postcss
├── README.md
├── tailwind.config.js          # TailwindCSS v4 config (custom theme)
├── tsconfig.json               # TypeScript config (strict mode)
├── tsconfig.app.json
├── tsconfig.node.json
└── vite.config.ts              # Vite config (@vitejs/plugin-react)
```

---

## 🧩 Component Hierarchy

### Authentication Flow

```
App.tsx
  └─ BrowserRouter
      ├─ Route /login → LoginPage
      │                   ├─ useAuthStore().login()
      │                   └─ Navigate to / on success
      │
      └─ Route <ProtectedRoute>         # Auth guard
            ├─ If !authenticated → Navigate to /login
            ├─ If loading → Loader2 spinner
            └─ If authenticated → Outlet (child routes)
                 │
                 └─ MainLayout wrapper
                      ├─ Sidebar (nav links, user info, logout)
                      └─ main content (children)
                           ├─ / → DashboardPage
                           ├─ /documents → DocumentsPage
                           ├─ /compare → ComparePage
                           ├─ /analytics → AnalyticsPage
                           └─ /admin → AdminPage (if role=admin)
```

### DashboardPage (RAG Chat)

```
DashboardPage
  ├─ Header
  │   ├─ Title: "💼 Chat RAG - {KB name}"
  │   └─ ModelSelector (LLM provider/model dropdown)
  │
  ├─ Messages Area (scrollable)
  │   ├─ If query && result:
  │   │   ├─ ChatMessage (user query)
  │   │   ├─ ChatMessage (assistant answer)
  │   │   ├─ ContextViewer (sources, verification, info gaps)
  │   │   └─ Processing time
  │   └─ If error: red alert box
  │
  └─ Input Form (sticky bottom)
      ├─ Textarea (query input)
      └─ Submit button (Send icon, disabled if loading or !query)
```

### ComparePage (Multi-Contract Comparison)

```
ComparePage
  ├─ Sidebar (left, fixed width)
  │   ├─ Header: "🔀 Confronto Contratti - {KB name}"
  │   ├─ DocumentSelector
  │   │   ├─ Search input (filter by name)
  │   │   ├─ Document list (checkboxes)
  │   │   └─ Selection count badge
  │   └─ Clear selection button
  │
  └─ Main Content (right, flex-1)
      ├─ Header: description
      └─ ContractComparison
          ├─ If no selection && no result:
          │   └─ Empty state (GitCompare icon, instructions)
          │
          ├─ If result:
          │   ├─ Comparison table (aspects × documents)
          │   ├─ Key differences section
          │   ├─ Information gaps warning
          │   └─ Processing time
          │
          └─ Input Form (sticky bottom)
              ├─ Aspect chips (removable)
              ├─ Add aspect input
              └─ Compare button (disabled if <2 docs)
```

### DocumentsPage (Document Management)

```
DocumentsPage
  ├─ Header: "📄 Documenti - {KB name}"
  │
  ├─ Upload Section
  │   └─ DocumentUpload
  │       ├─ Drag-and-drop zone
  │       ├─ File input (multiple, .pdf/.docx/.txt)
  │       ├─ Validation (mime type, size ≤200MB)
  │       └─ Upload progress list (per file status)
  │
  └─ Documents Section
      ├─ Header: count + refresh button
      ├─ If loading: spinner
      ├─ If error: red alert
      ├─ If empty: empty state (FileText icon)
      └─ If documents:
          └─ Grid (3 columns responsive)
              └─ Document cards
                  ├─ Name, mime type
                  ├─ Status badge (indexed/processing/error)
                  ├─ Dates (created, indexed)
                  ├─ Tags (if any)
                  └─ Delete button (with confirm dialog)
```

---

## 🔄 State Management (Zustand)

### authStore (`store/authStore.ts`)

**State:**
- `user: User | null` — Current logged-in user
- `isAuthenticated: boolean` — Auth status
- `isLoading: boolean` — Loading state (during login/fetch)
- `error: string | null` — Error message

**Actions:**
- `login(email, password)` → POST /api/auth/login, store tokens, set user
- `register(email, password, fullName)` → POST /api/auth/register, auto-login
- `logout()` → POST /api/auth/logout, clear tokens, clear user
- `fetchCurrentUser()` → GET /api/auth/me (on app mount, restore session)
- `clearError()` → Reset error state

**Persistence:**
- JWT tokens saved in `localStorage`: `access_token`, `refresh_token`
- On app load, `fetchCurrentUser()` is called if token exists
- On 401 during fetch, logout and redirect to /login

### appStore (`store/appStore.ts`)

**State:**
- `currentKb: KnowledgeBase | null` — Selected knowledge base
- `knowledgeBases: KnowledgeBase[]` — All available KBs
- `documents: Document[]` — Documents in current KB
- `selectedDocIds: string[]` — Multi-select for comparison
- `comparisonResult: ComparisonResult | null` — Latest comparison result
- `comparisonLoading: boolean` — Comparison in progress
- `comparisonPhase: string | null` — Current phase (if streaming)
- `comparisonError: string | null` — Comparison error
- `providers: LLMProvider[]` — Available LLM providers
- `selectedProvider: string | null` — Selected provider ID
- `selectedModel: string | null` — Selected model ID

**Actions:**
- `setCurrentKb(kb)` — Switch KB
- `setKnowledgeBases(kbs)` — Update KB list
- `setDocuments(docs)` — Update document list
- `toggleDocSelection(docId)` — Toggle doc in multi-select
- `clearDocSelection()` — Clear all selections
- `setComparisonResult(result)` — Store comparison result
- `setComparisonLoading(loading)` — Set loading state
- `setComparisonPhase(phase)` — Set current phase
- `setComparisonError(error)` — Set error message
- `clearComparison()` — Reset comparison state
- `setProviders(providers)` — Update provider list
- `setSelectedProvider(providerId)` — Select provider
- `setSelectedModel(modelId)` — Select model

**Usage Pattern:**
```tsx
const { currentKb, selectedDocIds, toggleDocSelection } = useAppStore();

// Toggle doc selection
<button onClick={() => toggleDocSelection(doc.id)}>
  Select
</button>
```

---

## 🌐 API Client (`lib/api.ts`)

### Axios Configuration

```typescript
const client = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || '/api',
  headers: { 'Content-Type': 'application/json' }
});
```

### Request Interceptor (JWT Injection)

```typescript
client.interceptors.request.use((config) => {
  const token = localStorage.getItem('access_token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});
```

### Response Interceptor (Token Refresh)

```typescript
client.interceptors.response.use(
  (response) => response,
  async (error) => {
    const originalRequest = error.config;
    
    // If 401 and not already retried
    if (error.response?.status === 401 && !originalRequest._retry) {
      originalRequest._retry = true;
      
      try {
        // Attempt token refresh
        const refreshToken = localStorage.getItem('refresh_token');
        const { data } = await axios.post('/api/auth/refresh', { refresh_token: refreshToken });
        
        // Store new access token
        localStorage.setItem('access_token', data.access_token);
        
        // Retry original request with new token
        originalRequest.headers.Authorization = `Bearer ${data.access_token}`;
        return client(originalRequest);
      } catch (refreshError) {
        // Refresh failed → logout
        localStorage.removeItem('access_token');
        localStorage.removeItem('refresh_token');
        window.location.href = '/login';
        return Promise.reject(refreshError);
      }
    }
    
    return Promise.reject(error);
  }
);
```

### Available Methods

| Method | Endpoint | Description |
|---|---|---|
| `login(email, password)` | POST /auth/login | Login, store tokens |
| `register(email, password, fullName)` | POST /auth/register | Register new user |
| `logout()` | POST /auth/logout | Logout, revoke refresh token |
| `getCurrentUser()` | GET /auth/me | Get current user profile |
| `query({ kb_id, query, top_k, rerank_top_n })` | POST /query | RAG query |
| `ingest({ doc_id, kb_id, file_path, mime_type })` | POST /ingest | Ingest document |
| `compareContracts({ kb_id, doc_ids, comparison_aspects })` | POST /compare | Compare contracts |
| `listKnowledgeBases()` | GET /kb | List all KBs |
| `getKbStats(kb_id)` | GET /kb/:id/stats | Get KB statistics |
| `listDocuments(kb_id)` | GET /kb/:id/documents | List documents in KB |
| `deleteDocument(kb_id, doc_id)` | DELETE /kb/:kb_id/documents/:doc_id | Delete document |
| `uploadDocument(kb_id, file)` | POST /kb/upload | Upload document (FormData) |
| `listLlmProviders()` | GET /llm/providers | List LLM providers |
| `health()` | GET /health | Health check |

---

## 🎨 Styling (TailwindCSS v4)

### Custom Theme (`tailwind.config.js`)

```javascript
export default {
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        primary: {
          DEFAULT: '#00ff9f',
          50: '#e6fff6',
          100: '#b3ffe6',
          // ... full scale
          900: '#003d2e',
        },
        dark: {
          DEFAULT: '#0a0f1a',
          50: '#2a3040',
          100: '#141b28',
          200: '#0f1520',
          300: '#0a0f1a',
        },
      },
    },
  },
  plugins: [],
};
```

### CSS Imports (`index.css`)

```css
@import "tailwindcss";

@theme {
  --color-primary: #00ff9f;
  --color-primary-50: #e6fff6;
  /* ... */
  --color-dark: #0a0f1a;
  --color-dark-50: #2a3040;
  /* ... */
}
```

### Common Patterns

**Button (primary):**
```tsx
<button className="px-6 py-2 bg-primary text-dark-300 font-medium rounded-lg hover:bg-primary-400 transition-colors">
  Confirm
</button>
```

**Input:**
```tsx
<input className="px-4 py-3 bg-dark-200 border border-dark-50 rounded-lg text-gray-100 focus:ring-2 focus:ring-primary" />
```

**Card:**
```tsx
<div className="bg-dark-100 border border-dark-50 rounded-lg p-4 hover:border-primary/50 transition-colors">
  Content
</div>
```

**Badge (status):**
```tsx
<span className="px-2 py-0.5 bg-green-900/30 border border-green-700 rounded text-green-300 text-xs">
  Indexed
</span>
```

---

## 🔒 Security Considerations

### JWT Storage

- **Access token**: `localStorage.getItem('access_token')` (15min TTL)
- **Refresh token**: `localStorage.getItem('refresh_token')` (7 days TTL)
- **Risk**: XSS can steal tokens from localStorage
- **Mitigation**: 
  - Strict CSP headers
  - No `dangerouslySetInnerHTML` with user input
  - `react-markdown` sanitizes by default

### Protected Routes

- `ProtectedRoute` component checks `isAuthenticated` before rendering
- Redirects to `/login` if not authenticated
- Loading state prevents flash of unauthenticated content

### File Upload Validation

- Client-side: `accept=".pdf,.docx,.txt"`, max 200MB
- MIME type check: `acceptedTypes.includes(file.type)`
- Server-side validation is authoritative (don't trust client)

### External Links

- All external links use `target="_blank" rel="noopener noreferrer"`
- Prevents `window.opener` attacks

### Input Sanitization

- No `eval()`, no `Function()`, no `innerHTML` manipulation
- Markdown rendering via `react-markdown` (sanitizes by default)
- Form inputs validated with TypeScript types

---

## 📊 Performance Optimizations

### Bundle Size

- **Target**: < 500KB gzipped
- **Actual**: 146.27 KB gzipped (70.7% below target)
- **Techniques**:
  - Code splitting by route (React.lazy, future)
  - Tree-shaking (Vite + ES modules)
  - Minification (Terser)
  - Gzip compression

### Build Time

- **Dev**: Instant HMR (~50ms per change)
- **Prod Build**: ~318ms (TypeScript compilation + Vite build)

### Runtime Performance

- **React 19** concurrent features
- **Zustand** minimal re-renders (selector-based)
- **Axios** request deduplication (via react-query, future)

### Future Optimizations

- [ ] Lazy load pages with `React.lazy()`
- [ ] Virtualize long document lists (react-window)
- [ ] Debounce search inputs (useDebounce)
- [ ] Cache API responses (react-query)
- [ ] Service Worker for offline support

---

## 🧪 Testing Strategy

### Unit Tests (Vitest + React Testing Library)

**Target Coverage**: >70%

**Test Files**: `<Component>.test.tsx`

**Example**:
```typescript
// components/auth/ProtectedRoute.test.tsx
import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import ProtectedRoute from './ProtectedRoute';
import { useAuthStore } from '../../store/authStore';

vi.mock('../../store/authStore');

test('redirects to /login when not authenticated', () => {
  (useAuthStore as any).mockReturnValue({
    isAuthenticated: false,
    isLoading: false,
  });
  
  render(
    <MemoryRouter>
      <ProtectedRoute />
    </MemoryRouter>
  );
  
  expect(window.location.pathname).toBe('/login');
});
```

### E2E Tests (Playwright)

**Test Scenarios**:
1. **Login flow**: User logs in → redirected to dashboard
2. **Upload flow**: User uploads document → appears in list
3. **Chat flow**: User asks question → sees answer with sources
4. **Comparison flow**: User selects 2+ docs → sees comparison table
5. **Model switch**: User switches LLM model → reflected in header

**Example**:
```typescript
// tests/e2e/login.spec.ts
import { test, expect } from '@playwright/test';

test('user can log in', async ({ page }) => {
  await page.goto('http://localhost:5173/login');
  await page.fill('input[type="email"]', 'test@example.com');
  await page.fill('input[type="password"]', 'password123');
  await page.click('button[type="submit"]');
  
  await expect(page).toHaveURL('http://localhost:5173/');
  await expect(page.locator('text=Chat RAG')).toBeVisible();
});
```

---

## 🚀 Deployment

### Production Build

```bash
cd frontend
npm run build
# Output: dist/ (static files)
```

### Serve Options

**Option 1: Static File Server**
```bash
npx serve -s dist -p 3000
```

**Option 2: Nginx**
```nginx
server {
  listen 80;
  server_name archivioparlante.com;
  root /var/www/frontend/dist;
  
  location / {
    try_files $uri $uri/ /index.html;
  }
  
  location /api {
    proxy_pass http://php-gateway:8080;
  }
}
```

**Option 3: Docker**
```dockerfile
FROM nginx:alpine
COPY dist/ /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
```

### Environment Variables

Production `.env`:
```env
VITE_API_BASE_URL=https://api.archivioparlante.com/api
```

---

## 📚 Resources

- **React 19 Docs**: https://react.dev/
- **Vite Docs**: https://vite.dev/
- **TailwindCSS v4**: https://tailwindcss.com/
- **Zustand**: https://zustand-demo.pmnd.rs/
- **React Router v7**: https://reactrouter.com/
- **Axios**: https://axios-http.com/
- **Vitest**: https://vitest.dev/
- **Playwright**: https://playwright.dev/

---

**Last Updated**: 2026-04-27  
**Maintainer**: Archivio Parlante Team
