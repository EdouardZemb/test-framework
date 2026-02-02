# Test Framework Documentation

> Playwright-based E2E and API testing framework with `@seontechnologies/playwright-utils`.

## Quick Start

### Prerequisites

- Node.js >= 22.0.0 (see `.nvmrc`)
- npm >= 10.0.0

### Setup

```bash
# Use correct Node version
nvm use

# Install dependencies
npm install

# Install Playwright browsers
npx playwright install

# Copy environment config
cp .env.example .env
# Edit .env with your test credentials
```

### Running Tests

```bash
# Run all tests
npm test

# Run with Playwright UI (interactive)
npm run test:pw:ui

# Run in headed mode (see browser)
npm run test:pw:headed

# Debug mode (step through)
npm run test:pw:debug

# Run only API tests (no browser)
npm run test:pw:api

# Run only E2E tests
npm run test:pw:e2e

# View HTML report
npm run test:pw:report

# Generate test code with Codegen
npm run test:pw:codegen
```

## Architecture Overview

```
tests/
├── e2e/                          # Test specifications
│   ├── example.spec.ts           # E2E test examples
│   └── api.spec.ts               # Pure API tests
├── support/
│   ├── fixtures/
│   │   ├── merged-fixtures.ts    # ⭐ Single test import (all utilities)
│   │   └── custom-fixtures.ts    # Project-specific fixtures
│   ├── factories/
│   │   ├── user-factory.ts       # User data factory
│   │   └── index.ts              # Factory exports
│   ├── helpers/
│   │   ├── api-helpers.ts        # API seeding utilities
│   │   └── index.ts              # Helper exports
│   ├── auth/
│   │   ├── api-auth-provider.ts  # Auth provider for playwright-utils
│   │   └── sessions/             # Persisted auth tokens
│   ├── global-setup.ts           # Runs before all tests
│   └── global-teardown.ts        # Runs after all tests
└── README.md                     # This file
```

## Key Concepts

### Merged Fixtures

All tests should import from `merged-fixtures.ts`:

```typescript
import { test, expect } from '../support/fixtures/merged-fixtures';

test('my test', async ({
  page,           // Playwright page
  apiRequest,     // Typed HTTP client
  authToken,      // Persistent auth token
  recurse,        // Polling utility
  log,            // Report logging
  testUser,       // Auto-seeded user (custom)
}) => {
  // All utilities available
});
```

### Data Factories

Use factories for test data - never hardcode:

```typescript
import { createUser, createAdminUser } from '../support/factories';

// Default user (unique ID, email via faker)
const user = createUser();

// Admin user (explicit intent)
const admin = createUser({ role: 'admin' });

// Composed factory
const admin = createAdminUser({ email: 'admin@example.com' });
```

**Why factories?**
- ✅ Parallel-safe (unique IDs via faker)
- ✅ Schema evolution (update factory, not tests)
- ✅ Explicit intent (overrides show what matters)

### API-First Testing

Seed data via API, not UI:

```typescript
test('admin can delete users', async ({ apiRequest, page }) => {
  // GIVEN: User exists (seeded via API - fast!)
  const user = createUser();
  await apiRequest({ method: 'POST', path: '/api/users', body: user });

  // WHEN: Admin deletes user (UI interaction)
  await page.goto('/admin/users');
  await page.click(`[data-testid="delete-${user.id}"]`);

  // THEN: User is deleted
  await expect(page.getByText('User deleted')).toBeVisible();
});
```

### Selector Strategy

**Preferred order:**

1. `data-testid` attributes (most stable)
2. Accessible selectors (`getByRole`, `getByLabel`)
3. Text content (`getByText`) - for user-visible content

```typescript
// ✅ Best: data-testid
await page.getByTestId('submit-button').click();

// ✅ Good: Accessible selectors
await page.getByRole('button', { name: 'Submit' }).click();
await page.getByLabel('Email').fill('test@example.com');

// ⚠️ Avoid: CSS implementation details
// await page.locator('.btn-primary').click();
// await page.locator('#form > div:nth-child(2)').click();
```

## Best Practices

### Test Isolation

- Each test should be independent
- Use `beforeEach`/`afterEach` for setup/cleanup
- Never rely on test execution order

### Given/When/Then Format

```typescript
test('user can login', async ({ page, log }) => {
  // GIVEN: User is on login page
  await log.step('Navigate to login');
  await page.goto('/login');

  // WHEN: User enters credentials
  await log.step('Enter credentials');
  await page.fill('[data-testid="email"]', 'test@example.com');
  await page.fill('[data-testid="password"]', 'password');
  await page.click('[data-testid="submit"]');

  // THEN: User is redirected to dashboard
  await log.step('Verify redirect');
  await expect(page).toHaveURL('/dashboard');
});
```

### Network Error Monitoring

Network error monitor is auto-enabled. Tests fail on HTTP 4xx/5xx errors.

```typescript
// Opt-out for validation tests
test('should handle invalid input',
  { annotation: [{ type: 'skipNetworkMonitoring' }] },
  async ({ page }) => {
    // This test expects 400 errors
  }
);
```

### Cleanup

Always clean up test data:

```typescript
test.afterEach(async ({ request }) => {
  // Delete created resources
  for (const userId of createdUsers) {
    await request.delete(`/api/users/${userId}`);
  }
});
```

## CI Integration

### GitHub Actions Example

```yaml
name: E2E Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version-file: '.nvmrc'

      - run: npm ci
      - run: npx playwright install --with-deps

      - run: npm test
        env:
          CI: true
          BASE_URL: ${{ secrets.TEST_BASE_URL }}
          TEST_USER_EMAIL: ${{ secrets.TEST_USER_EMAIL }}
          TEST_USER_PASSWORD: ${{ secrets.TEST_USER_PASSWORD }}

      - uses: actions/upload-artifact@v4
        if: failure()
        with:
          name: test-results
          path: test-results/
```

### Sharding (Parallel CI)

```yaml
jobs:
  test:
    strategy:
      matrix:
        shard: [1/3, 2/3, 3/3]
    steps:
      - run: npx playwright test --shard=${{ matrix.shard }}
```

## Utilities Reference

### @seontechnologies/playwright-utils

| Utility | Purpose | Docs |
|---------|---------|------|
| `apiRequest` | Typed HTTP client with retry | [api-request.md](../_bmad/tea/testarch/knowledge/api-request.md) |
| `authToken` | Persistent auth tokens | [auth-session.md](../_bmad/tea/testarch/knowledge/auth-session.md) |
| `recurse` | Polling for async operations | [recurse.md](../_bmad/tea/testarch/knowledge/recurse.md) |
| `log` | Report-integrated logging | [log.md](../_bmad/tea/testarch/knowledge/log.md) |
| `networkErrorMonitor` | Auto HTTP error detection | [network-error-monitor.md](../_bmad/tea/testarch/knowledge/network-error-monitor.md) |

### TEA Knowledge Base

Full testing patterns and best practices:
- `_bmad/tea/testarch/knowledge/` - All knowledge fragments
- `_bmad/tea/testarch/tea-index.csv` - Fragment index

## Troubleshooting

### Tests fail with "Cannot find module '@seontechnologies/playwright-utils'"

```bash
npm install -D @seontechnologies/playwright-utils
```

### Tests fail with network errors but UI looks correct

Check `test-results/artifacts/network-errors.json` for backend errors that occurred during the test.

### Auth token expired

Delete cached sessions and re-run:

```bash
rm -rf tests/support/auth/sessions
npm test
```

### Flaky tests

1. Check for race conditions (use `await` properly)
2. Use `waitFor` helpers instead of hard delays
3. Review network-first patterns in knowledge base

---

*Generated by TEA Test Framework Workflow*
