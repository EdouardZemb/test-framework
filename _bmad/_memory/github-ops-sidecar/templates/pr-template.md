# Pull Request Template

## Title Format

```
<type>(<scope>): <brief description>
```

Keep under 72 characters. Use same conventions as commits.

## Description Structure

```markdown
## Summary

Brief description of what this PR does and why.

## Changes

- Bullet list of specific changes
- Group by area if many changes

## Testing

How was this tested? Include:
- [ ] Unit tests added/updated
- [ ] Integration tests pass
- [ ] Manual testing performed

## Screenshots

(If UI changes, include before/after)

## Checklist

- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Tests pass locally
- [ ] Documentation updated
- [ ] No breaking changes (or documented if any)

## Related Issues

Closes #123
Relates to #456
```

## Labels to Consider

- `bug`, `feature`, `enhancement`, `documentation`
- `breaking-change`, `dependencies`
- `needs-review`, `work-in-progress`

## Draft PRs

Use draft PRs for:
- Work in progress needing early feedback
- Discussions before full implementation
- CI validation before ready for review
