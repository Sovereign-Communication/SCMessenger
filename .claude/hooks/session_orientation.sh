#!/usr/bin/env bash
# SessionStart hook: orient a fresh session on the CANONICAL state and warn on drift.
#
# Read-only, local refs only (no network), and it always exits 0: a hook must
# never block a session. MT-12 of the merge-train task file
# (HANDOFF/freebuff/queue/V040_V050_MERGE_TRAIN_UNIFY_2026-09-27.md): it warns
# when the rules files differ from origin/main or HEAD is far behind, and it
# names the authoritative execution pointer declared in SHIP_PLAN.md on
# origin/main instead of pointing at a superseded document.
set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR" || exit 0

BEHIND_LIMIT=20

echo "=== SCMessenger session orientation ==="

echo "--- git status (short) ---"
git status --short 2>/dev/null | head -20

echo "--- checkout vs origin/main (local refs; run 'git fetch origin main' to refresh) ---"
branch="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo '?')"
echo "branch: $branch @ $(git rev-parse --short HEAD 2>/dev/null || echo '?')"

if git rev-parse --verify -q origin/main >/dev/null 2>&1; then
  counts="$(git rev-list --left-right --count origin/main...HEAD 2>/dev/null || echo '0 0')"
  behind="${counts%%[[:space:]]*}"
  ahead="${counts##*[[:space:]]}"
  echo "behind origin/main: ${behind:-0}, ahead: ${ahead:-0}"

  if [ "${behind:-0}" -gt "$BEHIND_LIMIT" ] 2>/dev/null; then
    echo "[WARNING] HEAD is ${behind} commits behind origin/main (limit ${BEHIND_LIMIT}). Rules and plans in this checkout may be stale: merge or rebase origin/main, or start from a fresh worktree, before editing rules, plans or handoffs."
  fi

  if ! git diff --quiet origin/main -- AGENTS.md CLAUDE.md docs/rules SHIP_PLAN.md 2>/dev/null; then
    echo "[WARNING] AGENTS.md / CLAUDE.md / docs/rules / SHIP_PLAN.md differ from origin/main. Either your rules are stale or you carry unpushed rule edits. Read origin/main's versions before acting: git diff origin/main -- AGENTS.md CLAUDE.md docs/rules SHIP_PLAN.md"
  fi

  echo "--- authoritative execution pointer (origin/main:SHIP_PLAN.md) ---"
  ptr="$(git show origin/main:SHIP_PLAN.md 2>/dev/null | sed -n '1,40p' | grep -A8 'EXECUTION POINTER (authoritative)' | grep -o '`[^`]*\.md`' | head -1 | tr -d '`')"
  if [ -z "$ptr" ]; then
    echo "[WARNING] origin/main:SHIP_PLAN.md declares no 'EXECUTION POINTER (authoritative)': the canonical plan is not on main. Do not treat any other queue file as authoritative."
  elif git cat-file -e "origin/main:$ptr" 2>/dev/null; then
    echo "[OK] execution queue: $ptr (exists on origin/main)"
    git show "origin/main:$ptr" 2>/dev/null | grep -m1 '^\*\*Status:\*\*' | cut -c1-200
  else
    echo "[WARNING] the authoritative pointer names $ptr, which does not exist on origin/main."
  fi
else
  echo "[INFO] no origin/main ref in this checkout; drift checks skipped (run: git fetch origin main)"
fi

echo "--- HANDOFF backlog ---"
if [ -d HANDOFF/todo ]; then
  echo "todo: $(ls HANDOFF/todo 2>/dev/null | wc -l | tr -d ' ') files"
fi
if [ -d HANDOFF/IN_PROGRESS ]; then
  echo "in_progress: $(ls HANDOFF/IN_PROGRESS 2>/dev/null | wc -l | tr -d ' ') files"
fi

echo "========================================"
exit 0
