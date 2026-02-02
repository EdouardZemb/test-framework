#!/usr/bin/env bash

set -euo pipefail

PROGRAM_NAME="git-flow.sh"
DEVELOP_BRANCH="develop"

die() {
  echo "Error: $*" >&2
  exit 1
}

info() {
  echo "[git-flow] $*"
}

usage() {
  cat <<'EOF'
Usage:
  bash scripts/git-flow.sh init

  bash scripts/git-flow.sh feature start <name>
  bash scripts/git-flow.sh feature finish <name> [--keep] [--yes]

  bash scripts/git-flow.sh release start <version>
  bash scripts/git-flow.sh release finish <version> [--keep] [--no-tag] [--yes]

  bash scripts/git-flow.sh hotfix start <name>
  bash scripts/git-flow.sh hotfix finish <name> [--keep] [--no-tag] [--yes]

Notes:
  - Use names without spaces. Example: auth-login
  - "finish" commands merge branches and can delete source branches.
EOF
}

require_git_repo() {
  git rev-parse --is-inside-work-tree >/dev/null 2>&1 || die "Not inside a git repository."
}

branch_exists() {
  local branch="$1"
  git show-ref --verify --quiet "refs/heads/${branch}"
}

require_clean_worktree() {
  git update-index -q --refresh
  if ! git diff-files --quiet || ! git diff-index --quiet HEAD --; then
    die "Working tree is not clean. Commit or stash your changes first."
  fi
}

require_branch() {
  local branch="$1"
  branch_exists "$branch" || die "Branch '${branch}' does not exist."
}

get_primary_branch() {
  if branch_exists "main"; then
    echo "main"
    return
  fi

  if branch_exists "master"; then
    echo "master"
    return
  fi

  die "Neither 'main' nor 'master' branch exists."
}

normalize_name() {
  local raw="$1"
  [[ -n "$raw" ]] || die "Missing branch name."
  if [[ "$raw" =~ [[:space:]] ]]; then
    die "Branch name cannot contain spaces: '${raw}'"
  fi
  echo "$raw"
}

normalize_branch_name() {
  local prefix="$1"
  local raw="$2"
  if [[ "$raw" == "${prefix}/"* ]]; then
    echo "$raw"
  else
    echo "${prefix}/${raw}"
  fi
}

confirm_or_abort() {
  local message="$1"
  local auto_confirm="$2"

  if [[ "$auto_confirm" == "true" ]]; then
    return
  fi

  local answer
  read -r -p "${message} [y/N] " answer
  if [[ "$answer" != "y" && "$answer" != "Y" ]]; then
    die "Aborted by user."
  fi
}

parse_finish_flags() {
  KEEP_BRANCH="false"
  NO_TAG="false"
  AUTO_CONFIRM="false"

  while (($#)); do
    case "$1" in
      --keep)
        KEEP_BRANCH="true"
        ;;
      --no-tag)
        NO_TAG="true"
        ;;
      --yes)
        AUTO_CONFIRM="true"
        ;;
      *)
        die "Unknown option: $1"
        ;;
    esac
    shift
  done
}

create_branch_from() {
  local new_branch="$1"
  local base_branch="$2"

  require_branch "$base_branch"
  branch_exists "$new_branch" && die "Branch '${new_branch}' already exists."

  git checkout "$base_branch"
  git checkout -b "$new_branch"
  info "Created and checked out '${new_branch}' from '${base_branch}'."
}

delete_branch_if_needed() {
  local branch="$1"
  local keep="$2"

  if [[ "$keep" == "false" ]]; then
    git branch -d "$branch"
    info "Deleted '${branch}'."
  else
    info "Kept '${branch}' (--keep enabled)."
  fi
}

merge_branch() {
  local source_branch="$1"
  local target_branch="$2"
  local message="$3"

  require_branch "$source_branch"
  require_branch "$target_branch"

  git checkout "$target_branch"
  git merge --no-ff "$source_branch" -m "$message"
  info "Merged '${source_branch}' into '${target_branch}'."
}

create_tag_if_needed() {
  local version="$1"
  local skip="$2"

  if [[ "$skip" == "true" ]]; then
    info "Tag creation skipped (--no-tag enabled)."
    return
  fi

  local tag="$version"
  if [[ "$tag" != v* ]]; then
    tag="v${version}"
  fi

  if git show-ref --tags --verify --quiet "refs/tags/${tag}"; then
    die "Tag '${tag}' already exists."
  fi

  git tag -a "$tag" -m "Release ${tag}"
  info "Created tag '${tag}'."
}

cmd_init() {
  require_clean_worktree

  local primary_branch
  primary_branch="$(get_primary_branch)"

  if branch_exists "$DEVELOP_BRANCH"; then
    info "Branch '${DEVELOP_BRANCH}' already exists."
    return
  fi

  git checkout "$primary_branch"
  git checkout -b "$DEVELOP_BRANCH"
  info "Initialized Git Flow with '${DEVELOP_BRANCH}' from '${primary_branch}'."
}

cmd_feature_start() {
  local name="$1"
  name="$(normalize_name "$name")"
  local feature_branch
  feature_branch="$(normalize_branch_name "feature" "$name")"

  require_clean_worktree
  create_branch_from "$feature_branch" "$DEVELOP_BRANCH"
}

cmd_feature_finish() {
  local name="$1"
  shift
  parse_finish_flags "$@"

  local feature_branch
  feature_branch="$(normalize_branch_name "feature" "$(normalize_name "$name")")"

  require_clean_worktree
  confirm_or_abort "Merge '${feature_branch}' into '${DEVELOP_BRANCH}'?" "$AUTO_CONFIRM"
  merge_branch "$feature_branch" "$DEVELOP_BRANCH" "merge(feature): ${feature_branch} into ${DEVELOP_BRANCH}"
  delete_branch_if_needed "$feature_branch" "$KEEP_BRANCH"
}

cmd_release_start() {
  local version="$1"
  version="$(normalize_name "$version")"
  local release_branch
  release_branch="$(normalize_branch_name "release" "$version")"

  require_clean_worktree
  create_branch_from "$release_branch" "$DEVELOP_BRANCH"
}

cmd_release_finish() {
  local version="$1"
  shift
  parse_finish_flags "$@"

  local release_branch
  release_branch="$(normalize_branch_name "release" "$(normalize_name "$version")")"
  local primary_branch
  primary_branch="$(get_primary_branch)"

  require_clean_worktree
  confirm_or_abort "Finish '${release_branch}' (merge to '${primary_branch}' and '${DEVELOP_BRANCH}')?" "$AUTO_CONFIRM"

  merge_branch "$release_branch" "$primary_branch" "merge(release): ${release_branch} into ${primary_branch}"
  create_tag_if_needed "$version" "$NO_TAG"
  merge_branch "$release_branch" "$DEVELOP_BRANCH" "merge(release): ${release_branch} into ${DEVELOP_BRANCH}"
  delete_branch_if_needed "$release_branch" "$KEEP_BRANCH"
}

cmd_hotfix_start() {
  local name="$1"
  name="$(normalize_name "$name")"
  local hotfix_branch
  hotfix_branch="$(normalize_branch_name "hotfix" "$name")"
  local primary_branch
  primary_branch="$(get_primary_branch)"

  require_clean_worktree
  create_branch_from "$hotfix_branch" "$primary_branch"
}

cmd_hotfix_finish() {
  local name="$1"
  shift
  parse_finish_flags "$@"

  local hotfix_branch
  hotfix_branch="$(normalize_branch_name "hotfix" "$(normalize_name "$name")")"
  local primary_branch
  primary_branch="$(get_primary_branch)"

  require_clean_worktree
  confirm_or_abort "Finish '${hotfix_branch}' (merge to '${primary_branch}' and '${DEVELOP_BRANCH}')?" "$AUTO_CONFIRM"

  merge_branch "$hotfix_branch" "$primary_branch" "merge(hotfix): ${hotfix_branch} into ${primary_branch}"
  create_tag_if_needed "$name" "$NO_TAG"
  merge_branch "$hotfix_branch" "$DEVELOP_BRANCH" "merge(hotfix): ${hotfix_branch} into ${DEVELOP_BRANCH}"
  delete_branch_if_needed "$hotfix_branch" "$KEEP_BRANCH"
}

main() {
  require_git_repo

  local domain="${1:-}"
  local action="${2:-}"
  local arg="${3:-}"

  case "$domain" in
    init)
      [[ -z "$action" ]] || die "'init' does not accept subcommands."
      cmd_init
      ;;
    feature)
      case "$action" in
        start)
          [[ -n "$arg" ]] || die "Missing feature name."
          cmd_feature_start "$arg"
          ;;
        finish)
          [[ -n "$arg" ]] || die "Missing feature name."
          shift 3
          cmd_feature_finish "$arg" "$@"
          ;;
        *)
          usage
          die "Unknown feature action: '${action}'"
          ;;
      esac
      ;;
    release)
      case "$action" in
        start)
          [[ -n "$arg" ]] || die "Missing release version."
          cmd_release_start "$arg"
          ;;
        finish)
          [[ -n "$arg" ]] || die "Missing release version."
          shift 3
          cmd_release_finish "$arg" "$@"
          ;;
        *)
          usage
          die "Unknown release action: '${action}'"
          ;;
      esac
      ;;
    hotfix)
      case "$action" in
        start)
          [[ -n "$arg" ]] || die "Missing hotfix name."
          cmd_hotfix_start "$arg"
          ;;
        finish)
          [[ -n "$arg" ]] || die "Missing hotfix name."
          shift 3
          cmd_hotfix_finish "$arg" "$@"
          ;;
        *)
          usage
          die "Unknown hotfix action: '${action}'"
          ;;
      esac
      ;;
    help|-h|--help|"")
      usage
      ;;
    *)
      usage
      die "Unknown command: '${domain}'"
      ;;
  esac
}

main "$@"
