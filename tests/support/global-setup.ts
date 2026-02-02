/**
 * Global Setup
 *
 * Runs once before all tests. Use for:
 * - Authentication storage initialization
 * - Shared test data seeding
 * - Environment validation
 *
 * @see https://playwright.dev/docs/test-global-setup-teardown
 */
import {
  authStorageInit,
  setAuthProvider,
  configureAuthSession,
  authGlobalInit,
} from '@seontechnologies/playwright-utils/auth-session';
import apiAuthProvider from './auth/api-auth-provider';

async function globalSetup() {
  console.log('🧪 Global Setup: Initializing test environment...');

  // Ensure storage directories exist
  authStorageInit();

  // Configure auth session storage path
  configureAuthSession({
    authStoragePath: process.cwd() + '/tests/support/auth/sessions',
    debug: process.env.DEBUG === 'true',
  });

  // Set custom auth provider
  setAuthProvider(apiAuthProvider);

  // Pre-fetch token for default user (optional - speeds up first test)
  // Uncomment when auth endpoint is available:
  // await authGlobalInit();

  console.log('✅ Global Setup: Complete');
}

export default globalSetup;
