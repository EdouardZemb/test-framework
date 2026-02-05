/**
 * API Auth Provider
 *
 * Custom auth provider for API-based authentication.
 * Authenticates via API without browser - ideal for API-first testing.
 *
 * Customize this file to match your application's authentication mechanism.
 */
import type { APIRequestContext, BrowserContextOptions } from '@playwright/test';

/** Storage state format compatible with Playwright */
type StorageState = BrowserContextOptions['storageState'] & {
  origins?: Array<{
    origin: string;
    localStorage: Array<{ name: string; value: string }>;
  }>;
};

/** Auth provider options */
interface AuthOptions {
  environment?: string;
  userIdentifier?: string;
}

/** Auth provider interface */
export interface AuthProvider {
  getEnvironment: (options: AuthOptions) => string;
  getUserIdentifier: (options: AuthOptions) => string;
  extractToken: (storageState: StorageState) => string | undefined;
  isTokenExpired: (storageState: StorageState) => boolean;
  manageAuthToken: (request: APIRequestContext, options: AuthOptions) => Promise<StorageState>;
}

const apiAuthProvider: AuthProvider = {
  /**
   * Determine environment from options.
   */
  getEnvironment: (options) => options.environment || process.env.TEST_ENV || 'local',

  /**
   * Determine user identifier from options.
   */
  getUserIdentifier: (options) => options.userIdentifier || 'default-user',

  /**
   * Extract token from stored storage state.
   */
  extractToken: (storageState) => {
    const tokenEntry = storageState.origins?.[0]?.localStorage?.find(
      (item) => item.name === 'auth_token',
    );
    return tokenEntry?.value;
  },

  /**
   * Check if stored token is expired.
   */
  isTokenExpired: (storageState) => {
    const expiryEntry = storageState.origins?.[0]?.localStorage?.find(
      (item) => item.name === 'token_expiry',
    );
    if (!expiryEntry) return true;
    return Date.now() > parseInt(expiryEntry.value, 10);
  },

  /**
   * Main token acquisition logic.
   * Called when no valid token exists or token is expired.
   *
   * Customize this method for your authentication system:
   * - OAuth2 flow
   * - JWT authentication
   * - API key authentication
   * - Custom login endpoint
   */
  manageAuthToken: async (request, options) => {
    const email = process.env.TEST_USER_EMAIL;
    const password = process.env.TEST_USER_PASSWORD;

    if (!email || !password) {
      throw new Error(
        'TEST_USER_EMAIL and TEST_USER_PASSWORD must be set in environment variables',
      );
    }

    // Pure API login - no browser needed
    const response = await request.post('/api/auth/login', {
      data: { email, password },
    });

    if (!response.ok()) {
      throw new Error(`Authentication failed: ${response.status()} ${await response.text()}`);
    }

    const { token, expiresIn = 3600 } = await response.json();
    const expiryTime = Date.now() + expiresIn * 1000;

    // Return storage state format for disk persistence
    return {
      cookies: [],
      origins: [
        {
          origin: process.env.BASE_URL || 'http://localhost:3000',
          localStorage: [
            { name: 'auth_token', value: token },
            { name: 'token_expiry', value: String(expiryTime) },
          ],
        },
      ],
    };
  },
};

export default apiAuthProvider;
