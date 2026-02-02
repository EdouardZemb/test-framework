#!/usr/bin/env bash

set -euo pipefail

BASE_BRANCH="${1:-${GITHUB_BASE_REF:-}}"
HEAD_BRANCH="${2:-${GITHUB_HEAD_REF:-}}"

if [[ -z "$BASE_BRANCH" || -z "$HEAD_BRANCH" ]]; then
  echo "Error: base/head branch are required." >&2
  echo "Usage: bash scripts/validate-gitflow-pr.sh <base-branch> <head-branch>" >&2
  exit 1
fi

case "$BASE_BRANCH" in
  main|master)
    EXPECTED_PATTERN='^(release|hotfix)/.+$'
    EXPECTED_HINT="release/* or hotfix/*"
    ;;
  develop)
    EXPECTED_PATTERN='^(feature|bugfix|release|hotfix)/.+$'
    EXPECTED_HINT="feature/*, bugfix/*, release/*, or hotfix/*"
    ;;
  *)
    echo "[git-flow-guard] Skipping validation for base branch '$BASE_BRANCH'."
    exit 0
    ;;
esac

if [[ "$HEAD_BRANCH" =~ $EXPECTED_PATTERN ]]; then
  echo "[git-flow-guard] OK: '$HEAD_BRANCH' -> '$BASE_BRANCH'"
  exit 0
fi

echo "::error::Git Flow violation: PR to '$BASE_BRANCH' must come from ${EXPECTED_HINT}. Current source: '$HEAD_BRANCH'."
exit 1
