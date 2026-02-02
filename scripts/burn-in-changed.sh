#!/bin/bash
# Burn-In Test Runner
# Runs changed test files multiple times to detect flakiness
#
# Usage: ./scripts/burn-in-changed.sh [iterations] [base-branch]
# Example: ./scripts/burn-in-changed.sh 10 main

set -e

# Configuration
ITERATIONS=${1:-5}
BASE_BRANCH=${2:-main}
SPEC_PATTERN='\.(spec|test)\.(ts|js)$'

echo "🔥 Burn-In Test Runner"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Iterations: $ITERATIONS"
echo "Base branch: $BASE_BRANCH"
echo ""

# Detect changed test files
echo "📋 Detecting changed test files..."
CHANGED_SPECS=$(git diff --name-only "$BASE_BRANCH"...HEAD 2>/dev/null | grep -E "$SPEC_PATTERN" || echo "")

if [ -z "$CHANGED_SPECS" ]; then
  echo "✅ No test files changed. Skipping burn-in."
  exit 0
fi

echo "Changed test files:"
echo "$CHANGED_SPECS" | sed 's/^/  - /'
echo ""

# Count specs
SPEC_COUNT=$(echo "$CHANGED_SPECS" | wc -l | xargs)
echo "Running burn-in on $SPEC_COUNT test file(s)..."
echo ""

# Burn-in loop
for i in $(seq 1 "$ITERATIONS"); do
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "🔄 Iteration $i/$ITERATIONS"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

  # shellcheck disable=SC2086
  if npx playwright test $CHANGED_SPECS --reporter=list; then
    echo "✅ Iteration $i passed"
  else
    echo ""
    echo "🛑 BURN-IN FAILED on iteration $i"
    echo ""
    echo "The changed test(s) are FLAKY and should be fixed before merging."
    echo "Check test-results/ for failure artifacts."
    exit 1
  fi

  echo ""
done

# Success
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎉 BURN-IN PASSED"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "All $ITERATIONS iterations passed for $SPEC_COUNT test file(s)"
echo "Changed specs are stable and ready to merge."
echo ""

exit 0
