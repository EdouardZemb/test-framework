/**
 * Example E2E Tests
 *
 * Demonstrates recommended patterns:
 * - Given/When/Then format
 * - data-testid selector strategy
 * - Factory usage
 * - Network interception (when applicable)
 *
 * Import { test, expect } from merged-fixtures for all utilities.
 */
import { test, expect } from '../support/fixtures/merged-fixtures';
import { createUser } from '../support/factories';

test.describe('Example E2E Tests', () => {
  test('should demonstrate Given/When/Then format', async ({ page, log }) => {
    // GIVEN: A user navigates to the home page
    await log.step('Navigate to home page');
    await page.goto('/');

    // WHEN: The page loads
    await log.step('Wait for page to load');
    await page.waitForLoadState('domcontentloaded');

    // THEN: The main content should be visible
    await log.step('Verify main content is visible');
    // Uncomment when your app has this element:
    // await expect(page.getByTestId('main-content')).toBeVisible();

    // Placeholder assertion (remove when real assertions are added)
    expect(true).toBe(true);
  });

  test('should demonstrate factory usage', async ({ apiRequest, log }) => {
    // GIVEN: A user is created via factory
    await log.step('Create test user via factory');
    const user = createUser({ role: 'admin', email: 'admin@example.com' });

    // Verify factory produces expected structure
    expect(user.id).toBeDefined();
    expect(user.email).toBe('admin@example.com');
    expect(user.role).toBe('admin');
    expect(user.isActive).toBe(true);

    await log.step('User factory produced valid data', { userId: user.id, role: user.role });

    // WHEN: User is seeded via API (uncomment when API is available)
    // await log.step('Seed user via API');
    // const { status } = await apiRequest({
    //   method: 'POST',
    //   path: '/api/users',
    //   body: user,
    // });

    // THEN: API should return success
    // expect(status).toBe(201);
  });

  test('should demonstrate data-testid selector strategy', async ({ page, log }) => {
    await log.step('Navigate to page');
    await page.goto('/');

    // Preferred: data-testid selectors
    // await expect(page.getByTestId('login-button')).toBeVisible();
    // await expect(page.getByTestId('user-menu')).toBeHidden();

    // Acceptable: Accessible selectors
    // await expect(page.getByRole('button', { name: 'Login' })).toBeVisible();
    // await expect(page.getByLabel('Email')).toBeVisible();

    // Avoid: CSS selectors that depend on implementation details
    // ❌ page.locator('.btn-primary')
    // ❌ page.locator('#login-form > div:nth-child(2)')

    await log.step('Selector strategy demonstrated');
  });
});

test.describe('API Tests (No Browser)', () => {
  test('should make typed API request', async ({ apiRequest, log }) => {
    await log.step('Make API health check');

    // Example API request (uncomment when API is available)
    // const { status, body } = await apiRequest<{ status: string }>({
    //   method: 'GET',
    //   path: '/api/health',
    // });

    // expect(status).toBe(200);
    // expect(body.status).toBe('ok');

    await log.step('API request completed successfully');
  });

  test('should demonstrate schema validation', async ({ apiRequest }) => {
    // Example with Zod schema validation (uncomment when API is available)
    // import { z } from 'zod';

    // const HealthSchema = z.object({
    //   status: z.enum(['ok', 'degraded', 'down']),
    //   version: z.string(),
    // });

    // const { body } = await apiRequest({
    //   method: 'GET',
    //   path: '/api/health',
    //   validateSchema: HealthSchema,
    // });

    // Body is type-safe AND validated
    // expect(body.status).toBe('ok');
  });
});

test.describe('Network Error Monitoring', () => {
  test('should auto-fail on HTTP 4xx/5xx errors', async ({ page }) => {
    // Network error monitor is auto-enabled via merged-fixtures
    // If any API call returns 4xx/5xx, this test will fail automatically
    await page.goto('/');
  });

  test(
    'should opt-out for validation tests',
    { annotation: [{ type: 'skipNetworkMonitoring' }] },
    async ({ page }) => {
      // This test expects errors - monitoring disabled
      await page.goto('/');
      // Test error handling UI here
    },
  );
});
