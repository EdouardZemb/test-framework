/**
 * Merged Fixtures
 *
 * Combines custom project fixtures with stub fixtures for utilities.
 * Import { test, expect } from this file in all test files.
 *
 * Note: apiRequest, authToken, log, recurse are stub implementations.
 * Replace with real implementations when available.
 */
import { test as base, expect, type APIRequestContext } from '@playwright/test';
import { createUser } from '../factories/user-factory';

/** API Request result */
interface ApiResponse<T = unknown> {
  status: number;
  body: T;
  headers: Record<string, string>;
}

/** API Request options */
interface ApiRequestOptions {
  method: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';
  path: string;
  body?: unknown;
  headers?: Record<string, string>;
}

/** Log interface for test reporting */
interface Log {
  step: (message: string, data?: Record<string, unknown>) => Promise<void>;
  info: (message: string) => void;
  warn: (message: string) => void;
  error: (message: string) => void;
}

/** Recurse options */
interface RecurseOptions {
  timeout?: number;
  interval?: number;
}

/** Custom fixtures type */
interface CustomFixtures {
  /** Typed HTTP client stub - replace with real implementation */
  apiRequest: <T = unknown>(options: ApiRequestOptions) => Promise<ApiResponse<T>>;
  /** Authentication token stub - replace with real implementation */
  authToken: string;
  /** Playwright report-integrated logging */
  log: Log;
  /** Polling utility for async operations */
  recurse: <T>(
    fn: () => Promise<T>,
    predicate: (result: T) => boolean,
    options?: RecurseOptions,
  ) => Promise<T>;
  /** Auto-seeded test user */
  testUser: Awaited<ReturnType<typeof createUser>>;
}

/**
 * Extended test with all fixtures available.
 */
export const test = base.extend<CustomFixtures>({
  // Stub apiRequest - makes real requests via Playwright's request context
  apiRequest: async ({ request }, use) => {
    const apiRequest = async <T = unknown>(options: ApiRequestOptions): Promise<ApiResponse<T>> => {
      const baseURL = process.env.BASE_URL || 'http://localhost:3000';
      const url = `${baseURL}${options.path}`;

      const response = await request.fetch(url, {
        method: options.method,
        data: options.body,
        headers: options.headers,
      });

      let body: T;
      try {
        body = await response.json();
      } catch {
        body = (await response.text()) as unknown as T;
      }

      return {
        status: response.status(),
        body,
        headers: response.headers(),
      };
    };

    await use(apiRequest);
  },

  // Stub authToken - returns placeholder
  authToken: async ({}, use) => {
    // TODO: Implement real auth token acquisition
    await use('stub-auth-token');
  },

  // Log fixture - integrates with Playwright's test.step
  log: async ({}, use) => {
    const log: Log = {
      step: async (message: string, data?: Record<string, unknown>) => {
        await test.step(message, async () => {
          if (data) {
            console.log(`  ${JSON.stringify(data)}`);
          }
        });
      },
      info: (message: string) => console.log(`ℹ️  ${message}`),
      warn: (message: string) => console.warn(`⚠️  ${message}`),
      error: (message: string) => console.error(`❌ ${message}`),
    };
    await use(log);
  },

  // Recurse fixture - polls until predicate is true
  recurse: async ({}, use) => {
    const recurse = async <T>(
      fn: () => Promise<T>,
      predicate: (result: T) => boolean,
      options?: RecurseOptions,
    ): Promise<T> => {
      const timeout = options?.timeout || 30000;
      const interval = options?.interval || 1000;
      const startTime = Date.now();

      while (Date.now() - startTime < timeout) {
        const result = await fn();
        if (predicate(result)) {
          return result;
        }
        await new Promise((resolve) => setTimeout(resolve, interval));
      }

      throw new Error(`Recurse timeout after ${timeout}ms`);
    };
    await use(recurse);
  },

  // Test user fixture
  testUser: async ({ request }, use) => {
    const user = createUser();
    await use(user);
  },
});

export { expect };
