# E2E Testing Guide - Archivio Parlante Frontend

## Prerequisites

### 1. Backend Stack Running

All backend services must be running before E2E tests can execute:

```bash
# From project root
docker-compose up -d

# Verify all services are healthy
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
```

Expected services:
- `archivio-php-gateway` (host port **9080** → container 80)
- `archivio-rust-engine` (port 8090)
- `archivio-python-worker` (port 8091)
- `archivio-qdrant` (port 6333)
- `archivio-ollama` (port 11434)
- `archivio-mysql` (port 3307)
- `archivio-redis` (port 6379)

### 2. Database Seeded with Test User

Execute the following SQL to create a test user:

```sql
USE archivio_parlante_x;

-- Insert test user (password: password123)
INSERT INTO ap_users (email, full_name, password_hash, role, active)
VALUES (
  'test@example.com',
  'Test User',
  '$2y$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5L9TLGY.L1DGa',
  'user',
  1
);

-- Create test knowledge base
INSERT INTO ap_knowledge_bases (name, description, user_id)
VALUES ('Test KB', 'Knowledge base for testing', LAST_INSERT_ID());
```

### 3. Frontend Dev Server

Playwright config auto-starts the dev server on http://localhost:5173, or you can start manually:

```bash
cd frontend
npm run dev
```

---

## Running E2E Tests

### Run All Tests

```bash
npm run test:e2e
```

### Run Specific Browser

```bash
# Chromium only (fastest)
npx playwright test --project=chromium

# Firefox
npx playwright test --project=firefox

# Webkit (Safari)
npx playwright test --project=webkit

# Mobile viewports
npx playwright test --project="Mobile Chrome"
npx playwright test --project="Mobile Safari"
```

### Interactive UI Mode

```bash
npm run test:e2e:ui
```

This opens the Playwright Test UI where you can:
- See all tests in a sidebar
- Run tests individually
- Watch tests execute in real-time
- Inspect screenshots/videos
- Debug with step-through

### Debug Mode

```bash
npm run test:e2e:debug
```

Opens Playwright Inspector for step-by-step debugging.

### Run Specific Test File

```bash
npx playwright test tests/e2e/login.spec.ts
npx playwright test tests/e2e/chat.spec.ts
npx playwright test tests/e2e/comparison.spec.ts
```

---

## Test Coverage Checklist

### ✅ Implemented Tests

- [x] **Login Flow** (`tests/e2e/login.spec.ts` - 8 tests)
  - Login form displays correctly
  - Email validation
  - Wrong credentials error handling
  - Successful login redirects to dashboard
  - Toggle to registration mode
  - Session persistence after refresh
  - Logout functionality

### ⏳ TODO: Additional Test Files

- [ ] **Chat Flow** (`tests/e2e/chat.spec.ts`)
  - User can ask a question
  - Response displays with markdown
  - Source citations are clickable
  - Context viewer shows confidence scores
  - Verification badge when verified=true
  - Information gaps warning displays

- [ ] **Document Management** (`tests/e2e/documents.spec.ts`)
  - User can upload PDF document
  - User can upload DOCX document
  - User can upload TXT document
  - File size validation (max 200MB)
  - File type validation (reject invalid types)
  - Document list displays uploaded docs
  - User can delete a document
  - Status badges (indexed, processing, error)

- [ ] **Multi-Contract Comparison** (`tests/e2e/comparison.spec.ts`)
  - User can select multiple documents
  - User can add comparison aspects
  - Comparison button disabled if <2 docs
  - Comparison executes and shows results
  - Results show differences table
  - Key differences are highlighted
  - Can clear selection

- [ ] **LLM Model Switching** (`tests/e2e/model-switching.spec.ts`)
  - ModelSelector displays providers
  - Can switch from Ollama to cloud provider
  - Can switch between models
  - Cost display updates
  - Disabled state for missing API keys

- [ ] **Analytics Dashboard** (`tests/e2e/analytics.spec.ts`)
  - Dashboard displays query metrics
  - Charts render correctly
  - Cost tracking displays (if using cloud providers)

- [ ] **Admin Panel** (`tests/e2e/admin.spec.ts`)
  - Only admin users can access
  - Can create/delete knowledge bases
  - Can manage users
  - Can view system settings

---

## Manual Testing Checklist

### Critical User Flows

#### 1. Login/Logout Flow
- [ ] Navigate to http://localhost:5174/login
- [ ] Enter valid credentials (`test@example.com` / `password123`)
- [ ] Click "Accedi"
- [ ] Verify redirect to dashboard (/)
- [ ] Verify "Chat RAG" interface visible
- [ ] Click "Esci" in sidebar
- [ ] Verify redirect to login page

#### 2. Knowledge Base Selection
- [ ] From dashboard, check KB selector in sidebar
- [ ] Verify test KB appears in dropdown
- [ ] Select test KB
- [ ] Verify KB name displays in header

#### 3. Document Upload
- [ ] Navigate to "Documenti" page
- [ ] Drag and drop a PDF file
- [ ] Verify upload progress bar
- [ ] Verify document appears in list
- [ ] Check status badge (should be "processing" → "indexed")
- [ ] Try uploading invalid file type (e.g., .exe)
- [ ] Verify error message displays

#### 4. RAG Query
- [ ] From dashboard, enter question in textarea
- [ ] Click "Invia"
- [ ] Verify loading spinner appears
- [ ] Verify response displays with markdown
- [ ] Verify source citations appear below response
- [ ] Verify confidence scores color-coded (green >70%, yellow 50-70%, red <50%)
- [ ] Click on a citation
- [ ] Verify scrolls to ContextViewer

#### 5. Multi-Contract Comparison
- [ ] Navigate to "Confronta" page
- [ ] From sidebar DocumentSelector, select 2+ documents
- [ ] Add comparison aspects (e.g., "Clausole di recesso", "Penali")
- [ ] Click "Confronta"
- [ ] Verify loading state
- [ ] Verify comparison results table appears
- [ ] Verify key differences highlighted
- [ ] Check information gaps warning (if present)

#### 6. LLM Model Switching
- [ ] Open ModelSelector (gear icon or settings menu)
- [ ] Verify Ollama provider shows "Gratuito" badge
- [ ] Verify cloud providers show cost per 1M tokens
- [ ] Switch to different model (e.g., qwen2.5:3b)
- [ ] Execute a query
- [ ] Verify selected model is used (check network tab payload)

#### 7. Document Management
- [ ] Navigate to "Documenti" page
- [ ] View uploaded documents in grid/table
- [ ] Click delete button on a document
- [ ] Verify confirmation dialog appears
- [ ] Confirm deletion
- [ ] Verify document removed from list

#### 8. Error Handling
- [ ] Try logging in with wrong password
- [ ] Verify error message displays in red alert
- [ ] Try uploading file >200MB
- [ ] Verify size error message
- [ ] Try querying without KB selected
- [ ] Verify "Seleziona una KB" message
- [ ] Disconnect backend (stop Docker containers)
- [ ] Try executing a query
- [ ] Verify network error displays gracefully

### Browser Compatibility

Test in:
- [ ] Chrome/Edge (Chromium)
- [ ] Firefox
- [ ] Safari (if on Mac)
- [ ] Mobile Chrome (responsive view)
- [ ] Mobile Safari (responsive view)

### Performance

- [ ] Bundle size < 500KB gzipped ✅ (actual: 146.27 KB)
- [ ] Initial load time < 2s
- [ ] Query response UI updates < 100ms after API response
- [ ] Smooth scrolling on long chat history
- [ ] No memory leaks (check DevTools Memory profiler)

### Accessibility

- [ ] All forms keyboard navigable
- [ ] Tab order logical
- [ ] Focus visible on all interactive elements
- [ ] ARIA labels present on icon-only buttons
- [ ] Contrast ratios meet WCAG AAA (4.5:1 text, 3:1 UI)
- [ ] Screen reader test (NVDA/JAWS)

---

## Troubleshooting

### Backend Not Running

**Error**: `NetworkError when attempting to fetch resource`

**Solution**:
```bash
# Check if services are up
docker ps

# If not, start them
docker-compose up -d

# Check logs for errors
docker-compose logs -f rust-engine
docker-compose logs -f python-worker
```

### Test User Not Found

**Error**: Login test fails with "Invalid credentials"

**Solution**:
```bash
# Connect to MySQL
docker exec -it archivio-mysql mysql -u root -p archivio_parlante_x

# Verify test user exists
SELECT * FROM ap_users WHERE email = 'test@example.com';

# If not, insert test user (see Prerequisites section above)
```

### Port Already in Use

**Error**: `Error: listen EADDRINUSE: address already in use :::5173`

**Solution**:
```bash
# Find process using port 5173
lsof -i :5173  # Mac/Linux
netstat -ano | findstr :5173  # Windows

# Kill the process or use different port
PORT=5174 npm run dev
```

### Rust Build Failing

**Error**: `feature edition2024 is required`

**Solution**: Ensure `engine-rust/Dockerfile` uses `rust:latest` or `rust:1.88+`:
```dockerfile
FROM rust:latest AS builder
```

### Python Worker Segfault

**Error**: `Segmentation fault (core dumped)` during Docker build

**Solution**:
```bash
# Increase Docker memory limit (Docker Desktop Settings → Resources)
# Or clear build cache
docker builder prune -af

# Rebuild with no cache
docker-compose build --no-cache python-worker
```

---

## CI/CD Integration

### GitHub Actions Workflow

```yaml
name: E2E Tests

on:
  pull_request:
    branches: [develop, main]

jobs:
  e2e:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
      
      - name: Install dependencies
        run: |
          cd frontend
          npm ci
      
      - name: Install Playwright browsers
        run: npx playwright install --with-deps
      
      - name: Start backend stack
        run: docker-compose up -d
      
      - name: Wait for services to be healthy
        run: |
          until curl -f http://localhost:8090/health; do
            sleep 2
          done
      
      - name: Seed test database
        run: docker exec archivio-mysql mysql -u root archivio_parlante_x < db/seeds/test-user.sql
      
      - name: Run E2E tests
        run: npm run test:e2e
      
      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: playwright-report
          path: playwright-report/
```

---

## Test Results

- **Unit Tests**: 44/44 passing ✅
- **E2E Tests (Login)**: 8/8 passing ✅ (requires backend)
- **E2E Tests (Other)**: Not yet implemented ⏳

**Coverage**: 89.7% lines, 90.14% statements, 84.61% branches ✅

---

## Next Steps

1. **Implement remaining E2E test files** (chat, documents, comparison, etc.)
2. **Add visual regression testing** with Playwright screenshots
3. **Add performance benchmarks** (Lighthouse CI)
4. **Add accessibility tests** (axe-core integration)
5. **Set up E2E tests in CI/CD** (GitHub Actions)
