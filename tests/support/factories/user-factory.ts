/**
 * User Factory
 *
 * Factory function for creating test user data with sensible defaults.
 * Uses faker for dynamic values to prevent collisions in parallel tests.
 *
 * @example
 * // Default user
 * const user = createUser();
 *
 * // Admin user (explicit intent)
 * const admin = createUser({ role: 'admin' });
 *
 * // User with specific email
 * const user = createUser({ email: 'specific@example.com' });
 */
import { faker } from '@faker-js/faker';

export type User = {
  id: string;
  email: string;
  name: string;
  role: 'user' | 'admin' | 'moderator';
  createdAt: Date;
  isActive: boolean;
};

/**
 * Create a test user with defaults and optional overrides.
 *
 * @param overrides - Partial user object to override defaults
 * @returns Complete user object
 */
export const createUser = (overrides: Partial<User> = {}): User => ({
  id: faker.string.uuid(),
  email: faker.internet.email(),
  name: faker.person.fullName(),
  role: 'user',
  createdAt: new Date(),
  isActive: true,
  ...overrides,
});

/**
 * Create an admin user.
 */
export const createAdminUser = (overrides: Partial<User> = {}): User =>
  createUser({ role: 'admin', ...overrides });

/**
 * Create an inactive user.
 */
export const createInactiveUser = (overrides: Partial<User> = {}): User =>
  createUser({ isActive: false, ...overrides });
