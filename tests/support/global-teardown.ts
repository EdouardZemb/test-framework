/**
 * Global Teardown
 *
 * Runs once after all tests. Use for:
 * - Cleanup of shared test data
 * - Resource cleanup
 * - Report generation
 *
 * @see https://playwright.dev/docs/test-global-setup-teardown
 */

async function globalTeardown() {
  console.log('🧹 Global Teardown: Cleaning up...');

  // Add cleanup logic here
  // - Delete test users created in globalSetup
  // - Clear test database
  // - Close external connections

  console.log('✅ Global Teardown: Complete');
}

export default globalTeardown;
