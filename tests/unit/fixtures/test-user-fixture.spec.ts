/**
 * Test User Fixture – Unit Tests
 *
 * Validates that the testUser fixture (backed by createUser factory)
 * produces user objects with correctly typed and formatted fields.
 *
 * Priority: P1 (critical path – testUser underpins all user-related tests)
 */
import { test, expect } from '../../support/fixtures/merged-fixtures';

test.describe('testUser field validation', () => {
  test('[P1] testUser should have a valid UUID id', async ({ testUser }) => {
    // Given the testUser fixture
    // When we inspect the id field
    // Then it should match UUID v4 format
    const uuidV4Regex = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
    expect(testUser.id).toMatch(uuidV4Regex);
  });

  test('[P1] testUser should have a valid email format', async ({ testUser }) => {
    // Given the testUser fixture
    // When we inspect the email field
    // Then it should match a standard email pattern
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    expect(testUser.email).toMatch(emailRegex);
  });

  test('[P1] testUser should have a non-empty name', async ({ testUser }) => {
    // Given the testUser fixture
    // When we inspect the name field
    // Then it should be a non-empty string
    expect(typeof testUser.name).toBe('string');
    expect(testUser.name.length).toBeGreaterThan(0);
  });

  test('[P2] testUser createdAt should be a recent Date', async ({ testUser }) => {
    // Given the testUser fixture
    // When we inspect the createdAt field
    // Then it should be a Date instance within the last 5 seconds
    expect(testUser.createdAt).toBeInstanceOf(Date);
    const now = Date.now();
    const fiveSecondsAgo = now - 5000;
    expect(testUser.createdAt.getTime()).toBeGreaterThanOrEqual(fiveSecondsAgo);
    expect(testUser.createdAt.getTime()).toBeLessThanOrEqual(now);
  });
});
