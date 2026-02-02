/**
 * API Helpers
 *
 * Utility functions for API testing and data seeding.
 */
import { APIRequestContext } from '@playwright/test';
import { User, createUser } from '../factories/user-factory';

/**
 * Seed a user via API.
 *
 * @param request - Playwright API request context
 * @param overrides - Optional user field overrides
 * @returns Created user object
 */
export async function seedUser(
  request: APIRequestContext,
  overrides: Partial<User> = {},
): Promise<User> {
  const user = createUser(overrides);

  const response = await request.post('/api/users', {
    data: user,
  });

  if (!response.ok()) {
    throw new Error(`Failed to seed user: ${response.status()} ${await response.text()}`);
  }

  return user;
}

/**
 * Delete a user via API.
 *
 * @param request - Playwright API request context
 * @param userId - User ID to delete
 */
export async function deleteUser(request: APIRequestContext, userId: string): Promise<void> {
  const response = await request.delete(`/api/users/${userId}`);

  if (!response.ok() && response.status() !== 404) {
    throw new Error(`Failed to delete user: ${response.status()}`);
  }
}

/**
 * Wait for a condition to be true with polling.
 *
 * @param fn - Function that returns a promise resolving to boolean
 * @param options - Polling options
 */
export async function waitFor(
  fn: () => Promise<boolean>,
  options: { timeout?: number; interval?: number } = {},
): Promise<void> {
  const { timeout = 30000, interval = 1000 } = options;
  const startTime = Date.now();

  while (Date.now() - startTime < timeout) {
    if (await fn()) {
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, interval));
  }

  throw new Error(`waitFor timed out after ${timeout}ms`);
}
