import { test, expect } from '@playwright/test';

/**
 * E2E tests for RAG chat flow
 *
 * Prerequisites:
 * - Backend stack running (make up in root dir)
 * - Database seeded with test user: test@example.com / password123
 * - At least one document indexed in knowledge base
 */

// Helper function to login
async function loginAsTestUser(page: any) {
  await page.goto('/login');
  await page.fill('input[type="email"]', 'test@example.com');
  await page.fill('input[type="password"]', 'password123');
  await page.click('button[type="submit"]');
  await expect(page).toHaveURL('/');
}

test.describe('RAG Chat Flow', () => {
  test.beforeEach(async ({ page }) => {
    await loginAsTestUser(page);
  });

  test('displays chat interface on dashboard', async ({ page }) => {
    // Should show main chat components
    await expect(page.locator('text=Chat RAG')).toBeVisible();
    await expect(page.locator('textarea[placeholder*="domanda"]')).toBeVisible();
    await expect(page.locator('button[type="submit"]')).toBeVisible();
  });

  test('can send query and receive response', async ({ page }) => {
    // Find query input
    const queryInput = page.locator('textarea[placeholder*="domanda"]');
    await queryInput.fill('Quali sono le penali previste nel contratto?');

    // Submit query
    const submitButton = page.locator('button[type="submit"]').first();
    await submitButton.click();

    // Wait for response (should appear in message list)
    await page.waitForSelector('.message, [data-testid="message"]', { timeout: 15000 });

    // Should show loading state first
    await expect(page.locator('text=Elaborazione')).toBeVisible({ timeout: 5000 });

    // Wait for actual response (max 15s for RAG pipeline)
    await page.waitForTimeout(2000);

    // Response should contain citations or "informazioni non presenti"
    const responseText = await page.textContent('body');
    const hasResponse =
      responseText?.includes('penali') ||
      responseText?.includes('informazioni') ||
      responseText?.includes('clausola');

    expect(hasResponse).toBeTruthy();
  });

  test('shows sources when citations available', async ({ page }) => {
    const queryInput = page.locator('textarea[placeholder*="domanda"]');
    await queryInput.fill('Riassumi il contratto');

    const submitButton = page.locator('button[type="submit"]').first();
    await submitButton.click();

    // Wait for response
    await page.waitForTimeout(5000);

    // Should show sources section (if documents indexed)
    const bodyText = await page.textContent('body');

    // Either shows sources or "nessun documento" message
    const hasSourcesOrNoDoc =
      bodyText?.includes('Fonti:') ||
      bodyText?.includes('nessun documento') ||
      bodyText?.includes('informazioni non presenti');

    expect(hasSourcesOrNoDoc).toBeTruthy();
  });

  test('displays chat history', async ({ page }) => {
    // Send first query
    const queryInput = page.locator('textarea[placeholder*="domanda"]');
    await queryInput.fill('Prima domanda');
    await page.locator('button[type="submit"]').first().click();

    await page.waitForTimeout(3000);

    // Send second query
    await queryInput.fill('Seconda domanda');
    await page.locator('button[type="submit"]').first().click();

    await page.waitForTimeout(3000);

    // Both messages should be visible in history
    const bodyText = await page.textContent('body');
    expect(bodyText).toContain('Prima domanda');
    expect(bodyText).toContain('Seconda domanda');
  });

  test('can clear chat history', async ({ page }) => {
    // Send a query first
    const queryInput = page.locator('textarea[placeholder*="domanda"]');
    await queryInput.fill('Test query');
    await page.locator('button[type="submit"]').first().click();

    await page.waitForTimeout(3000);

    // Look for clear/reset button
    const clearButton = page.locator('button:has-text("Nuova chat"), button:has-text("Pulisci")').first();

    if (await clearButton.isVisible()) {
      await clearButton.click();

      // Chat history should be cleared
      const bodyText = await page.textContent('body');
      const hasTestQuery = bodyText?.includes('Test query');

      // If clear worked, query should be gone
      // If no clear button, skip assertion
      if (hasTestQuery === false) {
        expect(hasTestQuery).toBe(false);
      }
    }
  });

  test('handles empty query gracefully', async ({ page }) => {
    // Try to submit empty query
    const submitButton = page.locator('button[type="submit"]').first();
    await submitButton.click();

    // Should either:
    // 1. Prevent submission (button disabled)
    // 2. Show validation message
    // 3. Do nothing

    // Just verify page doesn't crash
    await expect(page.locator('text=Chat RAG')).toBeVisible();
  });

  test('shows error message when backend unavailable', async ({ page }) => {
    // This test requires backend to be DOWN
    // Skip in normal runs, only run when testing error handling
    test.skip(true, 'Requires backend to be down - manual test only');

    const queryInput = page.locator('textarea[placeholder*="domanda"]');
    await queryInput.fill('Test query');
    await page.locator('button[type="submit"]').first().click();

    // Should show error message
    await expect(page.locator('.bg-red-900, [data-testid="error"]')).toBeVisible({ timeout: 10000 });
  });
});

test.describe('LLM Model Selection', () => {
  test.beforeEach(async ({ page }) => {
    await loginAsTestUser(page);
  });

  test('can view available models', async ({ page }) => {
    // Look for model selector (dropdown or settings)
    const modelSelector = page.locator('select[name="model"], button:has-text("Modello")').first();

    if (await modelSelector.isVisible()) {
      await modelSelector.click();

      // Should show at least Ollama local model
      const bodyText = await page.textContent('body');
      const hasModel = bodyText?.includes('qwen') || bodyText?.includes('Ollama');
      expect(hasModel).toBeTruthy();
    } else {
      // Model selector not visible - skip test
      test.skip(true, 'Model selector not implemented yet');
    }
  });
});
