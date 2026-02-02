/**
 * Pure API Tests
 *
 * Demonstrates API-first testing patterns without browser.
 * Uses apiRequest fixture from @seontechnologies/playwright-utils.
 *
 * These tests are fast and reliable - no browser overhead.
 */
import { test, expect } from '../support/fixtures/merged-fixtures';
import { createUser, createAdminUser } from '../support/factories';

test.describe('User API', () => {
  test.skip('should create user via API', async ({ apiRequest, log }) => {
    // Skip until API is available
    const user = createUser({ email: 'test-api@example.com' });

    await log.step('Create user via POST /api/users');
    const { status, body } = await apiRequest<{ id: string; email: string }>({
      method: 'POST',
      path: '/api/users',
      body: user,
    });

    expect(status).toBe(201);
    expect(body.id).toBeDefined();
    expect(body.email).toBe(user.email);
  });

  test.skip('should get user by ID', async ({ apiRequest, log }) => {
    // Skip until API is available
    const user = createUser();

    // Seed user first
    await apiRequest({ method: 'POST', path: '/api/users', body: user });

    await log.step(`Get user ${user.id}`);
    const { status, body } = await apiRequest<{ id: string; name: string }>({
      method: 'GET',
      path: `/api/users/${user.id}`,
    });

    expect(status).toBe(200);
    expect(body.id).toBe(user.id);
    expect(body.name).toBe(user.name);
  });

  test.skip('should return 404 for non-existent user', async ({ apiRequest }) => {
    // Skip until API is available
    const { status } = await apiRequest({
      method: 'GET',
      path: '/api/users/non-existent-id',
    });

    expect(status).toBe(404);
  });
});

test.describe('Authenticated API', () => {
  test.skip('should access protected endpoint with auth token', async ({
    apiRequest,
    authToken,
    log,
  }) => {
    // Skip until auth endpoint is available
    await log.step('Access protected endpoint');

    const { status, body } = await apiRequest<{ email: string }>({
      method: 'GET',
      path: '/api/me',
      headers: { Authorization: `Bearer ${authToken}` },
    });

    expect(status).toBe(200);
    expect(body.email).toBeDefined();
  });

  test.skip('should reject unauthenticated requests', async ({ apiRequest }) => {
    // Skip until auth endpoint is available
    const { status } = await apiRequest({
      method: 'GET',
      path: '/api/me',
      // No auth header
    });

    expect(status).toBe(401);
  });
});

test.describe('Polling with Recurse', () => {
  test.skip('should poll until async job completes', async ({ apiRequest, recurse, log }) => {
    // Skip until job API is available

    // Create async job
    await log.step('Create async job');
    const { body: job } = await apiRequest<{ id: string }>({
      method: 'POST',
      path: '/api/jobs',
      body: { type: 'export' },
    });

    // Poll until complete
    await log.step(`Poll job ${job.id} until complete`);
    const completedJob = await recurse(
      () =>
        apiRequest<{ id: string; status: string; result?: unknown }>({
          method: 'GET',
          path: `/api/jobs/${job.id}`,
        }),
      (response) => response.body.status === 'completed',
      { timeout: 60000, interval: 2000 },
    );

    expect(completedJob.body.status).toBe('completed');
    expect(completedJob.body.result).toBeDefined();
  });
});
