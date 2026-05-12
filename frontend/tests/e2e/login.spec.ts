import { test, expect } from '@playwright/test';

/**
 * E2E tests for login flow
 *
 * Prerequisites:
 * - Backend stack running (make up in root dir)
 * - Database seeded with test user: test@example.com / password123
 */

test.describe('Login Flow', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to login page
    await page.goto('/login');
  });

  test('displays login form', async ({ page }) => {
    await expect(page.locator('h2')).toContainText('Accedi');
    await expect(page.locator('input[type="email"]')).toBeVisible();
    await expect(page.locator('input[type="password"]')).toBeVisible();
    await expect(page.locator('button[type="submit"]')).toBeVisible();
  });

  test('shows validation error for invalid email', async ({ page }) => {
    await page.fill('input[type="email"]', 'invalid-email');
    await page.fill('input[type="password"]', 'password123');

    // Browser native validation should prevent submission
    const emailInput = page.locator('input[type="email"]');
    await expect(emailInput).toHaveAttribute('type', 'email');
  });

  test('shows error for wrong credentials', async ({ page }) => {
    await page.fill('input[type="email"]', 'wrong@example.com');
    await page.fill('input[type="password"]', 'wrongpassword');
    await page.click('button[type="submit"]');

    // Wait for error message
    await expect(page.locator('.bg-red-900\\/30')).toBeVisible({ timeout: 5000 });
  });

  test('successful login redirects to dashboard', async ({ page }) => {
    // Fill login form
    await page.fill('input[type="email"]', 'test@example.com');
    await page.fill('input[type="password"]', 'password123');
    await page.click('button[type="submit"]');

    // Should redirect to dashboard
    await expect(page).toHaveURL('/', { timeout: 5000 });

    // Dashboard should show chat interface
    await expect(page.locator('text=Chat RAG')).toBeVisible();
    await expect(page.locator('textarea')).toBeVisible(); // Query input
  });

  test('can toggle to register mode', async ({ page }) => {
    // Click register toggle
    await page.click('text=Non hai un account? Registrati');

    // Should show register form
    await expect(page.locator('h2')).toContainText('Registrazione');
    await expect(page.locator('input[id="fullName"]')).toBeVisible();

    // Toggle back to login
    await page.click('text=Hai già un account? Accedi');
    await expect(page.locator('h2')).toContainText('Accedi');
  });

  test('persists session after page refresh', async ({ page, context }) => {
    // Login
    await page.fill('input[type="email"]', 'test@example.com');
    await page.fill('input[type="password"]', 'password123');
    await page.click('button[type="submit"]');

    await expect(page).toHaveURL('/');

    // Refresh page
    await page.reload();

    // Should still be on dashboard (not redirected to login)
    await expect(page).toHaveURL('/');
    await expect(page.locator('text=Chat RAG')).toBeVisible();
  });
});

test.describe('Logout Flow', () => {
  test('can logout from dashboard', async ({ page }) => {
    // Login first
    await page.goto('/login');
    await page.fill('input[type="email"]', 'test@example.com');
    await page.fill('input[type="password"]', 'password123');
    await page.click('button[type="submit"]');

    await expect(page).toHaveURL('/');

    // Click logout button in sidebar
    await page.click('text=Esci');

    // Should redirect to login
    await expect(page).toHaveURL('/login');
    await expect(page.locator('h2')).toContainText('Accedi');
  });
});
