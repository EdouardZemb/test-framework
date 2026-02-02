# GitHub Operations Memory

Purpose: Track repository configuration, user preferences, and release history.

## Repository Configuration

- Date: 2026-02-02
- Repository: `EdouardZemb/test-framework`
- Visibility: changed from `private` to `public` (to enable branch protection)
- Default branch: `main`
- Remote: `origin` -> `https://github.com/EdouardZemb/test-framework.git`
- Initial remote bootstrap commit: `e92efef` (`chore(repo): initialize remote repository`)
- Repository secrets audit (2026-02-02): none configured
- Repository secrets configured (2026-02-02): `TEST_USER_EMAIL`, `TEST_USER_PASSWORD`, `OPENAI_API_KEY`
- Repository metadata configured from `_bmad-output` docs (2026-02-02):
  - Description: `CLI for TRA QA process optimization: Jira/Squash triage, testability scoring, reporting automation, and privacy-first LLM assistance.`
  - Topics: `cli-tool`, `insuretech`, `jira`, `llm`, `privacy-by-design`, `qa-automation`, `reporting-automation`, `squash-tm`, `testability`
- Repo Health audit (2026-02-02):
  - Branch protection active on `main`, but `required_status_checks` not enforced.
  - Security analyzers disabled: vulnerability alerts, Dependabot security updates, secret scanning.
  - `.github` present with CI workflow + Dependabot config; no `CODEOWNERS` file detected.
- Repo Health hardening applied (2026-02-02):
  - Enabled: vulnerability alerts, Dependabot security updates, secret scanning, secret scanning push protection.
  - Branch protection updated: required status checks (`strict=true`) + CODEOWNERS reviews required.
  - Local baseline file created: `.github/CODEOWNERS` (`* @EdouardZemb`).
  - Direct commit to `main` for CODEOWNERS blocked by branch protection (`Changes must be made through a pull request`).
- 2026-02-02: Temporary protection relaxation requested/applied to bootstrap `CODEOWNERS` on remote.
  - Commit pushed to `main`: `af1b896` (`chore(repo): add CODEOWNERS baseline`)
  - Strict protection restored immediately after push (status checks + CODEOWNERS reviews required).

## User Preferences

- For remote bootstrap, create an initialization commit that excludes current local untracked/working changes.
- Preferred secret setup flow: combine bulk import (`.env`) + targeted repository secrets (`TEST_USER_EMAIL`, `TEST_USER_PASSWORD`, `OPENAI_API_KEY`).

## Release History

<!-- Track releases created with version, date, highlights -->

| Version | Date | Type | Highlights |
|---------|------|------|------------|

## Branch Protection Rules

- 2026-02-02: Initial attempt on private repo failed (`403`: requires Pro or public repo).
- 2026-02-02: After switching repo to public, protection enabled on `main`:
  - `enforce_admins`: true
  - Pull requests required with `1` approval
  - `dismiss_stale_reviews`: true
  - `required_conversation_resolution`: true
  - `allow_force_pushes`: false
  - `allow_deletions`: false
- 2026-02-02: Git Flow branch protection applied successfully on `main` and `develop`:
  - Required status check: `Validate Git Flow source branch` (`strict=true`)
  - PR reviews: 1 approval, stale dismiss enabled, CODEOWNERS required
  - `enforce_admins=true`, linear history + conversation resolution required
  - Force push / deletion disabled on both branches

## Workflow Preferences

- 2026-02-02: Merge strategy set to squash-only (`allow_merge_commit=false`, `allow_rebase_merge=false`, `allow_squash_merge=true`).
- 2026-02-02: `delete_branch_on_merge=true`.
- 2026-02-02: `allow_auto_merge=true`.
- 2026-02-02: Git Flow scaffolding added in repo (`scripts/git-flow.sh`, `docs/git-flow.md`, CI guard for branch source policy).
- 2026-02-02: Local `develop` branch created from `main`.
- 2026-02-02: Branch protection automation script added (`scripts/configure-gitflow-protection.sh`, npm `gitflow:protect`).
- 2026-02-02: Protection script hardened to target remote default branch + `develop`, and skip missing branch errors cleanly.
- 2026-02-02: Git Flow audit & fixes applied:
  - Added `bugfix/*` branch pattern support for PRs targeting `develop` in `scripts/validate-gitflow-pr.sh`
  - Required status checks updated on `main` and `develop` to include full E2E pipeline:
    - `Validate Git Flow source branch`
    - `Install Dependencies`
    - `E2E Tests (Shard 1/4)`, `E2E Tests (Shard 2/4)`, `E2E Tests (Shard 3/4)`, `E2E Tests (Shard 4/4)`
    - `Merge Reports`
