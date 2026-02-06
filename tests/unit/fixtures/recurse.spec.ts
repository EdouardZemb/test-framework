/**
 * Recurse Fixture – Unit Tests
 *
 * Validates the recurse polling utility exposed via merged-fixtures.
 * Tests use the fixture through Playwright's test.extend mechanism.
 *
 * Priority: P1 (support utility – used by async-wait patterns)
 */
import { test, expect } from '../../support/fixtures/merged-fixtures';

test.describe('recurse', () => {
  test('resolves immediately when predicate is true on first call', async ({ recurse }) => {
    let callCount = 0;

    const result = await recurse(
      async () => {
        callCount++;
        return 42;
      },
      (value) => value === 42,
    );

    expect(result).toBe(42);
    expect(callCount).toBe(1);
  });

  test('polls until predicate becomes true', async ({ recurse }) => {
    let callCount = 0;

    const result = await recurse(
      async () => {
        callCount++;
        return callCount;
      },
      (value) => value >= 3,
      { interval: 50 },
    );

    expect(result).toBe(3);
    expect(callCount).toBe(3);
  });

  test('throws timeout error when predicate never becomes true', async ({ recurse }) => {
    await expect(
      recurse(
        async () => false,
        (value) => value === true,
        { timeout: 200, interval: 50 },
      ),
    ).rejects.toThrow(/timeout/i);
  });

  test('respects custom timeout option', async ({ recurse }) => {
    const start = Date.now();

    await expect(
      recurse(
        async () => 'pending',
        (v) => v === 'done',
        { timeout: 300, interval: 50 },
      ),
    ).rejects.toThrow(/300ms/);

    const elapsed = Date.now() - start;
    expect(elapsed).toBeGreaterThanOrEqual(250);
    expect(elapsed).toBeLessThan(600);
  });

  test('respects custom interval option', async ({ recurse }) => {
    let callCount = 0;
    const start = Date.now();

    await expect(
      recurse(
        async () => {
          callCount++;
          return callCount;
        },
        () => false,
        { timeout: 250, interval: 100 },
      ),
    ).rejects.toThrow(/timeout/i);

    // With 100ms interval over ~250ms, expect roughly 3 calls (initial + 2 retries)
    expect(callCount).toBeGreaterThanOrEqual(2);
    expect(callCount).toBeLessThanOrEqual(4);
  });
});
