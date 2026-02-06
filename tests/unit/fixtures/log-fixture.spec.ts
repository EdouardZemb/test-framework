/**
 * Log Fixture – Unit Tests
 *
 * Validates the log reporting utility exposed via merged-fixtures.
 * Tests cover info, warn, error, and step methods.
 *
 * Priority: P2 (support utility – test reporting)
 */
import { test, expect } from '../../support/fixtures/merged-fixtures';

let consoleLogSpy: Array<string[]>;
let originalLog: typeof console.log;

let consoleWarnSpy: Array<string[]>;
let originalWarn: typeof console.warn;

let consoleErrorSpy: Array<string[]>;
let originalError: typeof console.error;

test.beforeEach(() => {
  consoleLogSpy = [];
  originalLog = console.log;
  console.log = (...args: string[]) => { consoleLogSpy.push(args); };

  consoleWarnSpy = [];
  originalWarn = console.warn;
  console.warn = (...args: string[]) => { consoleWarnSpy.push(args); };

  consoleErrorSpy = [];
  originalError = console.error;
  console.error = (...args: string[]) => { consoleErrorSpy.push(args); };
});

test.afterEach(() => {
  console.log = originalLog;
  console.warn = originalWarn;
  console.error = originalError;
});

test.describe('log.info', () => {
  test('[P2] should output message with info prefix', async ({ log }) => {
    // Given a log fixture instance
    // When info is called with a message
    log.info('test message');

    // Then console.log is called with the prefixed message
    expect(consoleLogSpy.some(args => args[0] === '\u2139\uFE0F  test message')).toBe(true);
  });
});

test.describe('log.warn', () => {
  test('[P2] should output message with warn prefix', async ({ log }) => {
    // Given a log fixture instance
    // When warn is called with a message
    log.warn('warning');

    // Then console.warn is called with the prefixed message
    expect(consoleWarnSpy.some(args => args[0] === '\u26A0\uFE0F  warning')).toBe(true);
  });
});

test.describe('log.error', () => {
  test('[P2] should output message with error prefix', async ({ log }) => {
    // Given a log fixture instance
    // When error is called with a message
    log.error('failure');

    // Then console.error is called with the prefixed message
    expect(consoleErrorSpy.some(args => args[0] === '\u274C failure')).toBe(true);
  });
});

test.describe('log.step', () => {
  test('[P2] should execute without error when called with message only', async ({ log }) => {
    // Given a log fixture instance
    // When step is called with a message only
    // Then it resolves without throwing
    await expect(log.step('step name')).resolves.toBeUndefined();
  });

  test('[P2] should execute without error when called with message and data', async ({ log }) => {
    // Given a log fixture instance
    // When step is called with a message and data object
    // Then it resolves without throwing
    await expect(log.step('step name', { key: 'value' })).resolves.toBeUndefined();
  });
});
