/**
 * Merged Fixtures Composition – Unit Tests
 *
 * Validates that the merged-fixtures module correctly exports test and expect,
 * and that all expected fixtures are wired and accessible.
 *
 * Priority: P1 (critical path – every test file depends on merged-fixtures)
 */
import { test, expect } from '../../support/fixtures/merged-fixtures';

test.describe('merged-fixtures exports', () => {
  test('[P1] should export test function', async ({}) => {
    // Given the merged-fixtures module
    // When we import test
    // Then it should be a function (Playwright's extended test)
    expect(typeof test).toBe('function');
  });

  test('[P1] should export expect function', async ({}) => {
    // Given the merged-fixtures module
    // When we import expect
    // Then it should be a function (Playwright's expect)
    expect(typeof expect).toBe('function');
  });

  test('[P1] should provide all expected fixtures', async ({ apiRequest, authToken, log, recurse, testUser }) => {
    // Given a test using merged-fixtures
    // When all fixtures are destructured
    // Then each fixture should be defined
    expect(apiRequest).toBeDefined();
    expect(authToken).toBeDefined();
    expect(log).toBeDefined();
    expect(recurse).toBeDefined();
    expect(testUser).toBeDefined();
  });
});

test.describe('testUser fixture via merged-fixtures', () => {
  test('[P1] should return a user object with all required fields', async ({ testUser }) => {
    // Given the testUser fixture
    // When we inspect the returned object
    // Then it should contain all User fields
    expect(testUser).toHaveProperty('id');
    expect(testUser).toHaveProperty('email');
    expect(testUser).toHaveProperty('name');
    expect(testUser).toHaveProperty('role');
    expect(testUser).toHaveProperty('createdAt');
    expect(testUser).toHaveProperty('isActive');
  });

  test('[P1] should have role defaulting to user', async ({ testUser }) => {
    // Given the testUser fixture with no overrides
    // When we check the role
    // Then it should be the factory default
    expect(testUser.role).toBe('user');
  });

  test('[P1] should have isActive defaulting to true', async ({ testUser }) => {
    // Given the testUser fixture with no overrides
    // When we check the isActive flag
    // Then it should be the factory default
    expect(testUser.isActive).toBe(true);
  });
});

test.describe('testUser fixture uniqueness', () => {
  test('[P2] should generate unique users across tests', async ({ testUser }) => {
    // Given multiple user objects created via the testUser fixture
    // When we compare their ids
    // Then each id should be unique (factory uses faker.string.uuid)
    const users = Array.from({ length: 5 }, () => {
      // createUser is called internally by the fixture; we simulate
      // multiple fixture invocations by importing the factory directly
      return testUser;
    });
    // At minimum, the single testUser instance should have a valid truthy id
    expect(testUser.id).toBeTruthy();
    // Verify uniqueness by creating users directly via factory
    const { createUser } = await import('../../support/factories/user-factory');
    const ids = Array.from({ length: 10 }, () => createUser().id);
    const uniqueIds = new Set(ids);
    expect(uniqueIds.size).toBe(ids.length);
  });
});
