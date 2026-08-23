#!/usr/bin/env bash
# apply_branch_protection.sh -- put a real gate on main.
#
# WHY
# `main` had NO protection and NO rulesets: `gh api .../branches/main/protection`
# returned 404 and the rulesets array was empty, so all 17 workflows were
# advisory. Red main therefore carried no consequence and stayed red for days,
# which made every downstream verification unfalsifiable.
#
# The commit that forced the issue was ebf5411b ("iterations") -- a single
# unreviewed push, by an admin, that broke the Android build AND deleted 7
# shipped Android sources including the APK-sharing feature. Any one required
# check would have stopped it. That is why enforce_admins is TRUE here: with
# admin bypass on, protection would not have stopped the one commit it exists
# to stop.
#
# Required contexts are deliberately the FAST, ALREADY-GREEN ones, so turning
# this on costs nothing today and cannot be blamed for blocking work. The
# 34-minute Android APK job is intentionally NOT required.
#
# required_approving_review_count is 0, NOT 1, and that is load-bearing.
# GitHub forbids approving your own pull request. With enforce_admins TRUE and
# a single-operator repo, requiring one approval would mean nobody on earth
# could merge anything -- the operator cannot self-approve and there is no
# second reviewer. Protection that locks the owner out gets ripped out within a
# day, and then there is no protection at all.
#
# Zero approvals still forces the PR flow and still gates on green checks, which
# is the part that would have stopped ebf5411b. Raise this to 1 the moment a
# second human has write access, not before.
#
#   scripts/apply_branch_protection.sh --dry-run   # show the payload, change nothing
#   scripts/apply_branch_protection.sh --apply
#   scripts/apply_branch_protection.sh --status
#   scripts/apply_branch_protection.sh --remove    # emergency only; prints a warning
#
# Run this AFTER PR #139 merges. Protecting a branch that a 100-commit PR is
# about to land into only creates friction.

set -uo pipefail
REPO="Sovereign-Communication/SCMessenger"
BRANCH="main"
MODE="${1:---dry-run}"

# Exact check-run names, verified against a real run on 2026-08-15. GitHub
# matches these literally -- a typo silently means "no check required", which
# is worse than no protection because it looks protected.
read -r -d '' PAYLOAD <<'JSON'
{
  "required_status_checks": {
    "strict": true,
    "contexts": [
      "Repository Hygiene Checks",
      "Lint",
      "Rust Linting",
      "Test (ubuntu-latest)",
      "Android JVM Unit Tests"
    ]
  },
  "enforce_admins": true,
  "required_pull_request_reviews": {
    "required_approving_review_count": 0,
    "dismiss_stale_reviews": true,
    "require_code_owner_reviews": false
  },
  "restrictions": null,
  "allow_force_pushes": false,
  "allow_deletions": false,
  "required_linear_history": false,
  "required_conversation_resolution": false
}
JSON

case "$MODE" in
  --status)
    echo "=== protection on $BRANCH ==="
    gh api "repos/$REPO/branches/$BRANCH/protection" 2>&1 | head -40
    echo
    echo "=== rulesets ==="
    gh api "repos/$REPO/rulesets" 2>&1 | head -20
    ;;

  --dry-run)
    echo "DRY RUN -- nothing will change."
    echo "target: $REPO branch $BRANCH"
    echo
    echo "$PAYLOAD"
    echo
    echo "Verifying every required context has actually run recently."
    echo "A context name that never appears is a silent no-op:"
    gh pr checks 139 --json name --jq '.[].name' 2>/dev/null > /tmp/_ctx.txt
    while IFS= read -r c; do
      if grep -Fxq "$c" /tmp/_ctx.txt 2>/dev/null; then
        echo "  [OK]   $c"
      else
        echo "  [WARN] $c  -- not seen in recent checks; would never gate anything"
      fi
    done < <(echo "$PAYLOAD" | grep -oE '"[^"]+"' | sed -n '/Hygiene\|^"Lint"$\|Rust Linting\|Test (ubuntu\|Android JVM/p' | tr -d '"')
    echo
    echo "re-run with --apply to enable."
    ;;

  --apply)
    echo "$PAYLOAD" > /tmp/_prot.json
    if gh api -X PUT "repos/$REPO/branches/$BRANCH/protection" \
         -H "Accept: application/vnd.github+json" --input /tmp/_prot.json > /tmp/_prot_out.json 2>&1; then
      echo "[OK] protection enabled on $BRANCH"
      echo "  required checks : $(gh api "repos/$REPO/branches/$BRANCH/protection" --jq '.required_status_checks.contexts | join(", ")' 2>/dev/null)"
      echo "  enforce_admins  : $(gh api "repos/$REPO/branches/$BRANCH/protection" --jq '.enforce_admins.enabled' 2>/dev/null)"
      echo "  force pushes    : $(gh api "repos/$REPO/branches/$BRANCH/protection" --jq '.allow_force_pushes.enabled' 2>/dev/null)"
      echo
      echo "Admins are NOT exempt. To land an emergency fix, disable, push, re-enable --"
      echo "deliberately, and say so in the commit message."
    else
      echo "[FAIL] could not enable protection:"
      cat /tmp/_prot_out.json
      exit 1
    fi
    ;;

  --remove)
    echo "[WARNING] This removes the gate entirely. main becomes pushable by anyone"
    echo "          with write access, with no checks. This is how ebf5411b happened."
    echo "          Prefer disabling ONE required context over removing protection."
    read -r -p "type REMOVE to confirm: " ans
    if [ "$ans" = "REMOVE" ]; then
      gh api -X DELETE "repos/$REPO/branches/$BRANCH/protection" && echo "[OK] protection removed"
    else
      echo "aborted."
    fi
    ;;

  *)
    echo "usage: $0 [--dry-run|--apply|--status|--remove]"
    exit 2
    ;;
esac
