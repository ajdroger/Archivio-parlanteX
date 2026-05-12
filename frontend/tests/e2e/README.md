# E2E Tests (Playwright)

End-to-end tests for Archivio Parlante frontend.

## Prerequisites

1. **Backend stack running**:
   ```bash
   cd .. # Root directory
   make up
   ```

2. **Database seeded with test user**:
   ```sql
   INSERT INTO ap_users (email, full_name, password_hash, role, active)
   VALUES ('test@example.com', 'Test User', '$2y$12$...', 'user', 1);
   ```

3. **Frontend dev server** (started automatically by Playwright):
   ```bash
   npm run dev
   ```

## Running Tests

```bash
# Run all E2E tests
npm run test:e2e

# Run tests in UI mode (interactive)
npm run test:e2e:ui

# Run specific test file
npx playwright test tests/e2e/login.spec.ts

# Run with specific browser
npx playwright test --project=chromium

# Debug tests
npx playwright test --debug
```

## Test Structure

```
tests/e2e/
├── login.spec.ts          # Login/logout flow
├── chat.spec.ts           # RAG query flow (TODO)
├── documents.spec.ts      # Document upload/management (TODO)
├── comparison.spec.ts     # Multi-contract comparison (TODO)
└── README.md
```

## Writing New Tests

```typescript
import { test, expect } from '@playwright/test';

test.describe('Feature Name', () => {
  test.beforeEach(async ({ page }) => {
    // Setup (e.g., login, navigate)
    await page.goto('/');
  });

  test('should do something', async ({ page }) => {
    // Arrange
    await page.fill('input', 'value');
    
    // Act
    await page.click('button');
    
    // Assert
    await expect(page.locator('.result')).toBeVisible();
  });
});
```

## Best Practices

1. **Use data-testid for stable selectors**:
   ```tsx
   <button data-testid="submit-button">Submit</button>
   ```
   ```typescript
   await page.click('[data-testid="submit-button"]');
   ```

2. **Wait for network idle**:
   ```typescript
   await page.waitForLoadState('networkidle');
   ```

3. **Take screenshots for debugging**:
   ```typescript
   await page.screenshot({ path: 'screenshot.png' });
   ```

4. **Use beforeEach for common setup**:
   ```typescript
   test.beforeEach(async ({ page }) => {
     await loginAsTestUser(page);
   });
   ```

## Debugging

- **Headed mode**: `npx playwright test --headed`
- **UI mode**: `npx playwright test --ui`
- **Debug specific test**: `npx playwright test --debug login.spec.ts`
- **Trace viewer**: `npx playwright show-trace trace.zip`

## CI/CD Integration

```yaml
# .github/workflows/e2e.yml
- name: Run E2E tests
  run: |
    npm run build
    npm run test:e2e
  env:
    CI: true
```

## Test Coverage

Target: All critical user flows covered

- [ ] Login/Logout
- [ ] User registration
- [ ] RAG query with sources
- [ ] Document upload
- [ ] Document delete
- [ ] Multi-contract comparison
- [ ] LLM model switching
- [ ] Admin panel access (role-based)
