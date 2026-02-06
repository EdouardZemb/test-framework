/**
 * API Auth Provider – Unit Tests
 *
 * Validates the four pure functions exposed by apiAuthProvider:
 * getEnvironment, getUserIdentifier, extractToken, isTokenExpired.
 * manageAuthToken requires a live API and is covered by integration tests.
 *
 * Priority: P1 (critical auth infrastructure)
 */
import { test, expect } from '@playwright/test';
import apiAuthProvider from '../../support/auth/api-auth-provider';

test.describe('getEnvironment', () => {
  let originalTestEnv: string | undefined;

  test.beforeEach(() => {
    originalTestEnv = process.env.TEST_ENV;
  });

  test.afterEach(() => {
    if (originalTestEnv === undefined) {
      delete process.env.TEST_ENV;
    } else {
      process.env.TEST_ENV = originalTestEnv;
    }
  });

  test('[P1] returns options.environment when provided', () => {
    // Given an explicit environment in options
    const options = { environment: 'staging' };
    // When getEnvironment is called
    const result = apiAuthProvider.getEnvironment(options);
    // Then it returns the provided environment
    expect(result).toBe('staging');
  });

  test('[P1] falls back to TEST_ENV env var when no option provided', () => {
    // Given TEST_ENV is set but no option is provided
    process.env.TEST_ENV = 'ci-environment';
    // When getEnvironment is called with empty options
    const result = apiAuthProvider.getEnvironment({});
    // Then it returns the TEST_ENV value
    expect(result).toBe('ci-environment');
  });

  test('[P1] defaults to local when neither option nor env var is set', () => {
    // Given no option and no TEST_ENV
    delete process.env.TEST_ENV;
    // When getEnvironment is called with empty options
    const result = apiAuthProvider.getEnvironment({});
    // Then it defaults to 'local'
    expect(result).toBe('local');
  });
});

test.describe('getUserIdentifier', () => {
  test('[P1] returns options.userIdentifier when provided', () => {
    // Given an explicit user identifier in options
    const options = { userIdentifier: 'admin-tester' };
    // When getUserIdentifier is called
    const result = apiAuthProvider.getUserIdentifier(options);
    // Then it returns the provided identifier
    expect(result).toBe('admin-tester');
  });

  test('[P1] defaults to default-user when no option provided', () => {
    // Given no user identifier in options
    // When getUserIdentifier is called with empty options
    const result = apiAuthProvider.getUserIdentifier({});
    // Then it returns the default value
    expect(result).toBe('default-user');
  });
});

test.describe('extractToken', () => {
  test('[P1] extracts token from valid storage state', () => {
    // Given a storage state containing an auth_token entry
    const storageState = {
      cookies: [],
      origins: [{
        origin: 'http://localhost:3000',
        localStorage: [
          { name: 'auth_token', value: 'jwt-abc-123' },
          { name: 'token_expiry', value: '9999999999999' },
        ],
      }],
    };
    // When extractToken is called
    const result = apiAuthProvider.extractToken(storageState);
    // Then it returns the token value
    expect(result).toBe('jwt-abc-123');
  });

  test('[P1] returns undefined for empty origins array', () => {
    // Given a storage state with an empty origins array
    const storageState = { cookies: [], origins: [] };
    // When extractToken is called
    const result = apiAuthProvider.extractToken(storageState);
    // Then it returns undefined
    expect(result).toBeUndefined();
  });

  test('[P1] returns undefined when no origins property', () => {
    // Given a storage state without an origins property
    const storageState = { cookies: [] };
    // When extractToken is called
    const result = apiAuthProvider.extractToken(storageState);
    // Then it returns undefined
    expect(result).toBeUndefined();
  });

  test('[P1] returns undefined when no auth_token in localStorage', () => {
    // Given a storage state with localStorage entries but no auth_token
    const storageState = {
      cookies: [],
      origins: [{
        origin: 'http://localhost:3000',
        localStorage: [
          { name: 'theme', value: 'dark' },
          { name: 'language', value: 'en' },
        ],
      }],
    };
    // When extractToken is called
    const result = apiAuthProvider.extractToken(storageState);
    // Then it returns undefined
    expect(result).toBeUndefined();
  });
});

test.describe('isTokenExpired', () => {
  test('[P1] returns false when token is not expired', () => {
    // Given a storage state with a future expiry timestamp
    const futureExpiry = String(Date.now() + 3600 * 1000);
    const storageState = {
      cookies: [],
      origins: [{
        origin: 'http://localhost:3000',
        localStorage: [
          { name: 'auth_token', value: 'valid-token' },
          { name: 'token_expiry', value: futureExpiry },
        ],
      }],
    };
    // When isTokenExpired is called
    const result = apiAuthProvider.isTokenExpired(storageState);
    // Then it returns false
    expect(result).toBe(false);
  });

  test('[P1] returns true when token is expired', () => {
    // Given a storage state with a past expiry timestamp
    const pastExpiry = String(Date.now() - 3600 * 1000);
    const storageState = {
      cookies: [],
      origins: [{
        origin: 'http://localhost:3000',
        localStorage: [
          { name: 'auth_token', value: 'expired-token' },
          { name: 'token_expiry', value: pastExpiry },
        ],
      }],
    };
    // When isTokenExpired is called
    const result = apiAuthProvider.isTokenExpired(storageState);
    // Then it returns true
    expect(result).toBe(true);
  });

  test('[P1] returns true when no token_expiry entry exists', () => {
    // Given a storage state without a token_expiry entry
    const storageState = {
      cookies: [],
      origins: [{
        origin: 'http://localhost:3000',
        localStorage: [{ name: 'auth_token', value: 'some-token' }],
      }],
    };
    // When isTokenExpired is called
    const result = apiAuthProvider.isTokenExpired(storageState);
    // Then it returns true
    expect(result).toBe(true);
  });
});
