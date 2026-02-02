# Commit Message Template

## Format

```
<type>(<scope>): <subject>

[optional body]

[optional footer(s)]
```

## Subject Line Rules

- Use imperative mood: "add" not "added" or "adds"
- No period at the end
- Max 50 characters (hard limit: 72)
- Lowercase after type/scope

## Body Guidelines

- Wrap at 72 characters
- Explain what and why, not how
- Separate from subject with blank line

## Footer Patterns

```
BREAKING CHANGE: <description>
Fixes #123
Closes #456
Co-authored-by: Name <email>
```

## Examples

### Simple feature
```
feat(auth): add password reset flow
```

### With body
```
fix(api): handle timeout on slow connections

The previous implementation would hang indefinitely when the
upstream service was slow to respond. Added 30s timeout with
retry logic.

Fixes #789
```

### Breaking change
```
feat(api)!: change pagination response format

BREAKING CHANGE: Pagination now uses cursor-based approach.
The `page` and `per_page` params are replaced with `cursor`
and `limit`.
```
