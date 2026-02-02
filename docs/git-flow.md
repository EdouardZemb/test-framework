# Git Flow for test-framework

This project now follows a Git Flow branching model:

- `main`: production-ready code
- `develop`: integration branch for upcoming release
- `feature/*`: feature branches created from `develop`
- `release/*`: release stabilization branches created from `develop`
- `hotfix/*`: urgent fixes created from `main`

## Initialize once

```bash
npm run gitflow:init
```

This creates `develop` from `main` (or `master`) if needed.

## Feature flow

```bash
npm run gitflow:feature:start -- login-api
# work + commits
npm run gitflow:feature:finish -- login-api --yes
```

## Release flow

```bash
npm run gitflow:release:start -- 1.2.0
# stabilization commits
npm run gitflow:release:finish -- 1.2.0 --yes
```

By default, `release finish`:

1. merges `release/*` into `main`
2. creates tag `v<version>`
3. merges back into `develop`
4. deletes the release branch

Options:

- `--keep`: keep source branch after finish
- `--no-tag`: skip tag creation
- `--yes`: non-interactive confirmation

## Hotfix flow

```bash
npm run gitflow:hotfix:start -- 1.2.1
# fix + commits
npm run gitflow:hotfix:finish -- 1.2.1 --yes
```

`hotfix finish` mirrors release finish but starts from `main`.

## PR policy in CI

Workflow `.github/workflows/git-flow-guard.yml` enforces:

- PR to `main`/`master`: source must be `release/*` or `hotfix/*`
- PR to `develop`: source must be `feature/*`, `release/*`, or `hotfix/*`

## GitHub branch protection (recommended)

After authenticating GitHub CLI:

```bash
gh auth login -h github.com
npm run gitflow:protect -- --yes
```

This applies protection to `main` (or `master`) and `develop`:

- pull request required
- 1 approval required
- stale review dismissal enabled
- CODEOWNERS review required
- conversation resolution required
- force push and deletion disabled
- linear history required
- status check required: `Validate Git Flow source branch`
