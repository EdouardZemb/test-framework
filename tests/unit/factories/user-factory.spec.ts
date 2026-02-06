/**
 * User Factory – Unit Tests
 *
 * Validates createUser, createAdminUser and createInactiveUser produce
 * correct defaults, honour overrides, and generate unique values.
 *
 * Priority: P0 (critical path – factories underpin all test data)
 */
import { test, expect } from '@playwright/test';
import { createUser, createAdminUser, createInactiveUser, type User } from '../../support/factories/user-factory';

test.describe('createUser', () => {
  test('returns an object with all required User fields', () => {
    const user = createUser();

    expect(user).toHaveProperty('id');
    expect(user).toHaveProperty('email');
    expect(user).toHaveProperty('name');
    expect(user).toHaveProperty('role');
    expect(user).toHaveProperty('createdAt');
    expect(user).toHaveProperty('isActive');
  });

  test('defaults to active user with role "user"', () => {
    const user = createUser();

    expect(user.role).toBe('user');
    expect(user.isActive).toBe(true);
  });

  test('generates unique IDs across successive calls', () => {
    const ids = Array.from({ length: 20 }, () => createUser().id);
    const unique = new Set(ids);

    expect(unique.size).toBe(ids.length);
  });

  test('generates unique emails across successive calls', () => {
    const emails = Array.from({ length: 20 }, () => createUser().email);
    const unique = new Set(emails);

    expect(unique.size).toBe(emails.length);
  });

  test('applies partial overrides while keeping other defaults', () => {
    const user = createUser({ email: 'fixed@example.com', role: 'moderator' });

    expect(user.email).toBe('fixed@example.com');
    expect(user.role).toBe('moderator');
    // Remaining fields still have defaults
    expect(user.id).toBeTruthy();
    expect(user.name).toBeTruthy();
    expect(user.isActive).toBe(true);
  });
});

test.describe('createAdminUser', () => {
  test('returns a user with role "admin"', () => {
    const admin = createAdminUser();

    expect(admin.role).toBe('admin');
    expect(admin.isActive).toBe(true);
  });

  test('allows further overrides on top of admin defaults', () => {
    const admin = createAdminUser({ name: 'Super Admin' });

    expect(admin.role).toBe('admin');
    expect(admin.name).toBe('Super Admin');
  });
});

test.describe('createInactiveUser', () => {
  test('returns a user with isActive false', () => {
    const inactive = createInactiveUser();

    expect(inactive.isActive).toBe(false);
    expect(inactive.role).toBe('user');
  });
});
