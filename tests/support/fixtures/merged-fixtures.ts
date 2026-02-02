/**
 * Merged Fixtures
 *
 * Combines @seontechnologies/playwright-utils fixtures with custom project fixtures.
 * Import { test, expect } from this file in all test files.
 *
 * @see https://github.com/seontechnologies/playwright-utils
 */
import { mergeTests } from '@playwright/test';

// Playwright Utils fixtures (requires: npm install -D @seontechnologies/playwright-utils)
import { test as apiRequestFixture } from '@seontechnologies/playwright-utils/api-request/fixtures';
import { test as authFixture } from '@seontechnologies/playwright-utils/auth-session/fixtures';
import { test as recurseFixture } from '@seontechnologies/playwright-utils/recurse/fixtures';
import { test as logFixture } from '@seontechnologies/playwright-utils/log/fixtures';
import { test as networkErrorMonitorFixture } from '@seontechnologies/playwright-utils/network-error-monitor/fixtures';

// Custom project fixtures
import { test as customFixtures } from './custom-fixtures';

/**
 * Merged test object with all utilities available:
 * - apiRequest: Typed HTTP client with schema validation and retry
 * - authToken: Persistent authentication token management
 * - recurse: Polling for async operations
 * - log: Playwright report-integrated logging
 * - networkErrorMonitor: Automatic HTTP 4xx/5xx detection
 * - Custom fixtures from custom-fixtures.ts
 */
export const test = mergeTests(
  apiRequestFixture,
  authFixture,
  recurseFixture,
  logFixture,
  networkErrorMonitorFixture,
  customFixtures,
);

export { expect } from '@playwright/test';
