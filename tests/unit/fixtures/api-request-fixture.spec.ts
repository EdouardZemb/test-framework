/**
 * API Request & Auth Token Fixtures – Unit Tests
 *
 * Validates the apiRequest and authToken fixtures exposed via merged-fixtures.
 * These tests verify fixture shape and default values without making network calls.
 *
 * Priority: P1 (core fixtures – used by all API test flows)
 */
import { test, expect } from '../../support/fixtures/merged-fixtures';

test.describe('apiRequest fixture', () => {
  test('[P1] apiRequest fixture should be a function', async ({ apiRequest }) => {
    // Given the apiRequest fixture is injected by Playwright
    // When we inspect its type
    // Then it should be a callable function
    expect(typeof apiRequest).toBe('function');
  });
});

test.describe('authToken fixture', () => {
  test('[P1] authToken fixture should return a string', async ({ authToken }) => {
    // Given the authToken fixture is injected by Playwright
    // When we inspect its type
    // Then it should be a string value
    expect(typeof authToken).toBe('string');
  });

  test('[P1] authToken fixture should return the stub token value', async ({ authToken }) => {
    // Given the authToken fixture provides a stub implementation
    // When the fixture is resolved
    // Then the token should match the known stub value
    expect(authToken).toBe('stub-auth-token');
  });
});
