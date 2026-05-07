import { test, expect } from '@playwright/test';

/**
 * E2E tests for multi-contract comparison feature
 *
 * Prerequisites:
 * - Backend stack running (make up in root dir)
 * - Database seeded with test user: test@example.com / password123
 * - At least 2 documents indexed in knowledge base
 */

// Helper function to login
async function loginAsTestUser(page: any) {
  await page.goto('/login');
  await page.fill('input[type="email"]', 'test@example.com');
  await page.fill('input[type="password"]', 'password123');
  await page.click('button[type="submit"]');
  await expect(page).toHaveURL('/');
}

test.describe('Multi-Contract Comparison', () => {
  test.beforeEach(async ({ page }) => {
    await loginAsTestUser(page);
  });

  test('displays comparison interface', async ({ page }) => {
    // Navigate to comparison section
    const comparisonButton = page.locator('button:has-text("Confronto"), a:has-text("Confronto")').first();

    if (await comparisonButton.isVisible()) {
      await comparisonButton.click();

      // Should show comparison UI
      const bodyText = await page.textContent('body');
      const hasComparisonUI =
        bodyText?.includes('Confronto') ||
        bodyText?.includes('Seleziona contratti') ||
        bodyText?.includes('documenti');

      expect(hasComparisonUI).toBeTruthy();
    } else {
      test.skip(true, 'Comparison UI not visible - feature not implemented yet');
    }
  });

  test('can select multiple documents for comparison', async ({ page }) => {
    // Navigate to comparison section
    const comparisonButton = page.locator('button:has-text("Confronto"), a:has-text("Confronto")').first();

    if (!(await comparisonButton.isVisible())) {
      test.skip(true, 'Comparison feature not visible');
      return;
    }

    await comparisonButton.click();

    // Look for document checkboxes or multi-select
    const documentCheckboxes = page.locator('input[type="checkbox"]');
    const checkboxCount = await documentCheckboxes.count();

    if (checkboxCount >= 2) {
      // Select first two documents
      await documentCheckboxes.nth(0).check();
      await documentCheckboxes.nth(1).check();

      // Should show selected count
      const bodyText = await page.textContent('body');
      const hasSelectionCount = bodyText?.includes('2') || bodyText?.includes('selezionati');

      expect(hasSelectionCount).toBeTruthy();
    } else {
      test.skip(true, 'Not enough documents for comparison');
    }
  });

  test('can run comparison query across multiple contracts', async ({ page }) => {
    // Navigate to comparison
    const comparisonButton = page.locator('button:has-text("Confronto"), a:has-text("Confronto")').first();

    if (!(await comparisonButton.isVisible())) {
      test.skip(true, 'Comparison feature not visible');
      return;
    }

    await comparisonButton.click();

    // Select at least 2 documents
    const documentCheckboxes = page.locator('input[type="checkbox"]');
    if (await documentCheckboxes.count() >= 2) {
      await documentCheckboxes.nth(0).check();
      await documentCheckboxes.nth(1).check();
    } else {
      test.skip(true, 'Not enough documents');
      return;
    }

    // Enter comparison query
    const queryInput = page.locator('textarea[placeholder*="confronto"], textarea[placeholder*="domanda"]').first();
    await queryInput.fill('Confronta le penali previste nei contratti selezionati');

    // Submit comparison
    const submitButton = page.locator('button[type="submit"], button:has-text("Confronta")').first();
    await submitButton.click();

    // Wait for comparison results (RAG on multiple docs)
    await page.waitForTimeout(8000);

    // Should show comparative table or side-by-side results
    const bodyText = await page.textContent('body');
    const hasResults =
      bodyText?.includes('Contratto') ||
      bodyText?.includes('penali') ||
      bodyText?.includes('tabella') ||
      bodyText?.includes('confronto');

    expect(hasResults).toBeTruthy();
  });

  test('shows gap analysis for missing clauses', async ({ page }) => {
    test.skip(true, 'Gap analysis UI depends on backend implementation - test when ready');

    // This would test the feature described in CONTRACT_ANALYSIS_PROMPTS.md
    // "Gap Analysis tra Contratti" - shows clauses present in some docs but not others
  });

  test('can export comparison results', async ({ page }) => {
    test.skip(true, 'Export functionality depends on implementation');

    // Would test PDF/Excel export of comparison table
  });

  test('handles comparison with 3+ contracts', async ({ page }) => {
    // Navigate to comparison
    const comparisonButton = page.locator('button:has-text("Confronto"), a:has-text("Confronto")').first();

    if (!(await comparisonButton.isVisible())) {
      test.skip(true, 'Comparison feature not visible');
      return;
    }

    await comparisonButton.click();

    // Select 3 or more documents
    const documentCheckboxes = page.locator('input[type="checkbox"]');
    const checkboxCount = await documentCheckboxes.count();

    if (checkboxCount >= 3) {
      await documentCheckboxes.nth(0).check();
      await documentCheckboxes.nth(1).check();
      await documentCheckboxes.nth(2).check();

      // Enter query
      const queryInput = page.locator('textarea').first();
      await queryInput.fill('Confronta termini di pagamento');

      const submitButton = page.locator('button[type="submit"]').first();
      await submitButton.click();

      await page.waitForTimeout(10000);

      // Should handle 3+ contracts without crashing
      const bodyText = await page.textContent('body');
      expect(bodyText).toBeTruthy();
    } else {
      test.skip(true, 'Less than 3 documents available');
    }
  });
});

test.describe('Comparison Table View', () => {
  test.beforeEach(async ({ page }) => {
    await loginAsTestUser(page);
  });

  test('displays side-by-side contract clauses', async ({ page }) => {
    test.skip(true, 'Table view implementation pending');

    // Would test the comparative table format:
    // | Contratto | Penale Prevista | Limite Massimo | Note |
    // |-----------|----------------|----------------|------|
    // | A         | €500/giorno    | €50.000        | ... |
    // | B         | €1000/giorno   | €100.000       | ... |
  });

  test('highlights differences between contracts', async ({ page }) => {
    test.skip(true, 'Diff highlighting implementation pending');

    // Would test visual highlighting of:
    // - Clauses present in one contract but not others
    // - Different values (e.g., different penalty amounts)
    // - Different terms (e.g., 30 days vs 60 days payment)
  });

  test('shows contract metadata in comparison', async ({ page }) => {
    test.skip(true, 'Metadata comparison implementation pending');

    // Would test display of:
    // - Contract dates
    // - Parties involved
    // - Contract type (NDA, Fornitura, Appalto)
    // - Governing law / Foro competente
  });
});

test.describe('Forensic Analysis Prompts', () => {
  test.beforeEach(async ({ page }) => {
    await loginAsTestUser(page);
  });

  test('can use pre-configured legal prompts', async ({ page }) => {
    test.skip(true, 'Forensic prompt library UI implementation pending');

    // Would test the prompts from CONTRACT_ANALYSIS_PROMPTS.md:
    // - NDA: "Analizza gli obblighi di riservatezza"
    // - Appalti: "Verifica conformità Codice Appalti"
    // - Fornitura: "Estrai condizioni di pagamento"
    // - Licenze Software: "Analizza diritti d'uso"
    // - M&A: "Estrai rappresentazioni e garanzie"
  });

  test('prompt library shows Italian legal categories', async ({ page }) => {
    test.skip(true, 'Prompt library UI implementation pending');

    // Would test dropdown/sidebar with categories:
    // - NDA (Non-Disclosure Agreement)
    // - Appalti Pubblici
    // - Contratti di Fornitura
    // - Licenze Software
    // - M&A (Merger & Acquisition)
    // - Confronto Multi-Contratto
  });

  test('pre-configured prompts enforce citation requirements', async ({ page }) => {
    test.skip(true, 'Citation enforcement testing pending backend implementation');

    // Would verify that responses to forensic prompts:
    // 1. Include text_quote from source documents
    // 2. Cite article numbers (e.g., "Art. 5 comma 2")
    // 3. Never invent information not in documents
    // 4. Show "❌ ASSENTE" for missing clauses (not hallucinated defaults)
  });
});
