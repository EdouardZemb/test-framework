#!/usr/bin/env bash

set -euo pipefail

DEFAULT_CHECK_CONTEXT="Validate Git Flow source branch"
AUTO_CONFIRM="false"

die() {
  echo "Error: $*" >&2
  exit 1
}

info() {
  echo "[git-flow-protection] $*"
}

usage() {
  cat <<'EOF'
Usage:
  bash scripts/configure-gitflow-protection.sh [owner/repo] [--yes]

Examples:
  bash scripts/configure-gitflow-protection.sh EdouardZemb/test-framework
  bash scripts/configure-gitflow-protection.sh --yes

Notes:
  - Requires authenticated GitHub CLI (`gh auth login`)
  - Applies branch protection to main (or master) and develop
EOF
}

confirm_or_abort() {
  if [[ "$AUTO_CONFIRM" == "true" ]]; then
    return
  fi

  local answer
  read -r -p "Apply Git Flow branch protection now? [y/N] " answer
  if [[ "$answer" != "y" && "$answer" != "Y" ]]; then
    die "Aborted by user."
  fi
}

parse_repo_from_origin() {
  local origin_url
  origin_url="$(git remote get-url origin 2>/dev/null || true)"
  [[ -n "$origin_url" ]] || die "No origin remote found. Provide owner/repo explicitly."

  if [[ "$origin_url" =~ github\.com[:/]([^/]+/[^/.]+)(\.git)?$ ]]; then
    echo "${BASH_REMATCH[1]}"
    return
  fi

  die "Could not parse owner/repo from origin URL: $origin_url"
}

get_default_branch() {
  local repo="$1"
  gh api "repos/${repo}" --jq ".default_branch"
}

apply_protection() {
  local repo="$1"
  local branch="$2"
  local response

  info "Applying protection to '${repo}:${branch}'..."
  if ! response="$(
    gh api \
      --method PUT \
      -H "Accept: application/vnd.github+json" \
      "repos/${repo}/branches/${branch}/protection" \
      --input - 2>&1 <<JSON
{
  "required_status_checks": {
    "strict": true,
    "contexts": ["${DEFAULT_CHECK_CONTEXT}"]
  },
  "enforce_admins": true,
  "required_pull_request_reviews": {
    "dismiss_stale_reviews": true,
    "require_code_owner_reviews": true,
    "required_approving_review_count": 1,
    "require_last_push_approval": false
  },
  "restrictions": null,
  "required_linear_history": true,
  "allow_force_pushes": false,
  "allow_deletions": false,
  "block_creations": false,
  "required_conversation_resolution": true,
  "lock_branch": false,
  "allow_fork_syncing": true
}
JSON
)"; then
    if echo "$response" | grep -qi "Branch not found"; then
      info "Skipping '${branch}': branch not found on remote."
      return
    fi
    die "Failed to apply protection on '${branch}': ${response}"
  fi

  echo "$response"
}

main() {
  local repo=""
  local default_branch=""

  while (($#)); do
    case "$1" in
      --yes)
        AUTO_CONFIRM="true"
        shift
        ;;
      -h|--help|help)
        usage
        exit 0
        ;;
      *)
        if [[ -z "$repo" ]]; then
          repo="$1"
          shift
        else
          die "Unexpected argument: $1"
        fi
        ;;
    esac
  done

  gh auth status >/dev/null 2>&1 || die "GitHub CLI is not authenticated. Run: gh auth login -h github.com"

  if [[ -z "$repo" ]]; then
    repo="$(parse_repo_from_origin)"
  fi

  default_branch="$(get_default_branch "$repo")"
  [[ -n "$default_branch" ]] || die "Unable to determine default branch for '${repo}'."

  confirm_or_abort
  apply_protection "$repo" "$default_branch"
  apply_protection "$repo" "develop"
  info "Done."
}

main "$@"
