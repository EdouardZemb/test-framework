/**
 * Custom Project Fixtures
 *
 * Add project-specific fixtures here. These will be merged with
 * playwright-utils fixtures in merged-fixtures.ts.
 */
import { test as base } from '@playwright/test';
import { createUser } from '../factories/user-factory';

/**
 * Custom fixtures extending Playwright's base test.
 *
 * Add fixtures for:
 * - Test data seeding
 * - Database helpers
 * - Custom authentication
 * - Project-specific utilities
 */
export const test = base.extend<{
  /**
   * Auto-seeded test user for the current test.
   * User is created before test and cleaned up after.
   */
  testUser: Awaited<ReturnType<typeof createUser>>;
}>({
  testUser: async ({ request }, use) => {
    // Create user via factory
    const user = createUser();

    // Seed via API (uncomment when API is available)
    // const response = await request.post('/api/users', { data: user });
    // if (!response.ok()) {
    //   throw new Error(`Failed to seed user: ${response.status()}`);
    // }

    // Provide user to test
    await use(user);

    // Cleanup after test (uncomment when API is available)
    // await request.delete(`/api/users/${user.id}`);
  },
});
