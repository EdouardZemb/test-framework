/**
 * API Helpers – Unit Tests
 *
 * Validates the waitFor polling utility from api-helpers.
 * seedUser/deleteUser require a live API and are covered by integration tests.
 *
 * Priority: P1 (support utility – async polling helper)
 */
import { test, expect } from '@playwright/test';
import { waitFor } from '../../support/helpers/api-helpers';

test.describe('waitFor', () => {
  test('resolves immediately when fn returns true on first call', async () => {
    let callCount = 0;

    await waitFor(async () => {
      callCount++;
      return true;
    });

    expect(callCount).toBe(1);
  });

  test('polls until fn returns true', async () => {
    let callCount = 0;

    await waitFor(
      async () => {
        callCount++;
        return callCount >= 3;
      },
      { interval: 50 },
    );

    expect(callCount).toBe(3);
  });

  test('throws timeout error when fn never returns true', async () => {
    await expect(
      waitFor(async () => false, { timeout: 200, interval: 50 }),
    ).rejects.toThrow(/timed out after 200ms/i);
  });

  test('respects custom timeout and interval options', async () => {
    let callCount = 0;
    const start = Date.now();

    await expect(
      waitFor(
        async () => {
          callCount++;
          return false;
        },
        { timeout: 300, interval: 100 },
      ),
    ).rejects.toThrow(/timed out/i);

    const elapsed = Date.now() - start;
    // Should have waited close to 300ms
    expect(elapsed).toBeGreaterThanOrEqual(250);
    expect(elapsed).toBeLessThan(600);
    // With 100ms interval over ~300ms, expect roughly 3-4 calls
    expect(callCount).toBeGreaterThanOrEqual(2);
    expect(callCount).toBeLessThanOrEqual(5);
  });
});
