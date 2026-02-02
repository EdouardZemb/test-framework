# GitHub Operations Agent Instructions

## Startup Protocols

1. **Auth Check**: Before any GitHub operations, verify `gh auth status` succeeds
2. **Repo Context**: Detect if in a git repository and identify remote origin
3. **Config Load**: Read any existing `.github/` configurations for context

## Commit Type Reference (Conventional Commits)

| Type | Description | Example |
|------|-------------|---------|
| `feat` | New feature | `feat(auth): add OAuth2 login` |
| `fix` | Bug fix | `fix(api): handle null response` |
| `docs` | Documentation only | `docs: update README setup steps` |
| `style` | Formatting, no code change | `style: fix indentation` |
| `refactor` | Code restructure, no behavior change | `refactor(utils): extract helper` |
| `perf` | Performance improvement | `perf(query): add index lookup` |
| `test` | Adding/fixing tests | `test(auth): cover edge cases` |
| `build` | Build system or deps | `build: upgrade webpack to v5` |
| `ci` | CI/CD changes | `ci: add parallel test shards` |
| `chore` | Maintenance tasks | `chore: update .gitignore` |
| `revert` | Revert previous commit | `revert: undo feat(x)` |

## Scope Conventions

- Use lowercase scope names in parentheses
- Keep scopes short (1-2 words): `auth`, `api`, `ui`, `db`
- Match existing scopes in repo history when possible

## Breaking Changes

- Add `!` after type/scope: `feat(api)!: change response format`
- Or include `BREAKING CHANGE:` in commit body

## PR Checklist Standards

- [ ] Tests pass locally
- [ ] Code follows project conventions
- [ ] Documentation updated (if applicable)
- [ ] No secrets or credentials committed
- [ ] Breaking changes documented
