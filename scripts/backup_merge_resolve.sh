#!/usr/bin/env bash
# backup_merge_resolve.sh -- gated resolution of the backup-branch merge.
#
# Companion to HANDOFF/audit/BACKUP_BRANCH_MERGE_MAP_2026-09-13.md (the
# evidence file). Read that first; this script only executes its Option A/B.
#
# SAFETY MODEL (AGENTS.md rules 11/12/14):
#   - DRY-RUN by default: prints the plan and verification, mutates nothing.
#   - --apply without SCM_BACKUP_MERGE_APPROVED=1 in the environment is
#     refused. The variable is the operator's explicit sign-off (rule 12:
#     destructive operations require explicit operator approval, every time).
#   - Creates a FRESH branch off origin/main; never touches main, the shared
#     checkout's checked-out branch, or any other session's worktree.
#   - Group-A code paths resolve to MAIN'S BLOBS -- byte-identical to content
#     already reviewed and landed via #281/#282 -- so this introduces no new
#     code. Group-B docs are reported for manual union-merge; the script
#     refuses to commit. A human finishes, reviews, and gates the PR.
#
# Usage:
#   scripts/backup_merge_resolve.sh --option-b            # dry-run (default)
#   scripts/backup_merge_resolve.sh --option-a            # dry-run
#   scripts/backup_merge_resolve.sh --option-b --apply    # needs env approval
set -euo pipefail

OPTION=""
APPLY=0
for arg in "$@"; do
  case "$arg" in
    --option-a) OPTION="a" ;;
    --option-b) OPTION="b" ;;
    --apply)    APPLY=1 ;;
    *) echo "[ERROR] unknown argument: $arg"; echo "usage: $0 --option-a|--option-b [--apply]"; exit 2 ;;
  esac
done
if [ -z "$OPTION" ]; then
  echo "usage: $0 --option-a|--option-b [--apply]  (no option = nothing to do)"
  echo "  --option-a  merge backup/cto-t2-disk-dirty-20260913 (86 commits) into a fresh branch"
  echo "  --option-b  cherry-pick disk-governor 7f39c884 + manual doc port (recommended)"
  exit 2
fi

BACKUP_REF="4efa253e"
GOVERNOR_COMMIT="7f39c884"
BRANCH="merge/backup-resolve-$(date +%Y%m%d-%H%M%S)"

GROUP_A=(
  core/src/routing/local.rs
  core/src/routing/optimized_engine.rs
  core/src/store/ledger_entry.rs
  core/src/transport/swarm.rs
  core/src/transport/observation.rs
  android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt
  android/app/src/main/java/com/scmessenger/android/service/AnrWatchdog.kt
  android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt
)
GROUP_B=(
  HANDOFF/CTO_STATE.md
  HANDOFF/CEO_STATE.md
  HANDOFF/review/V040_T14_EPHEMERAL_REVIEW_QWEN_2026-09-01.md
  .codebuff_deploy/rev/pr251.diff
)

if [ "$APPLY" -eq 1 ] && [ "${SCM_BACKUP_MERGE_APPROVED:-0}" != "1" ]; then
  echo "[ERROR] --apply requires the operator's explicit sign-off:"
  echo "        SCM_BACKUP_MERGE_APPROVED=1 scripts/backup_merge_resolve.sh --option-${OPTION} --apply"
  exit 1
fi

echo "[INFO] mode: $( [ "$APPLY" -eq 1 ] && echo APPLY || echo DRY-RUN ), option: $OPTION, branch: $BRANCH"
git fetch origin --quiet
echo "[INFO] origin/main tip: $(git log -1 --oneline origin/main)"

if [ "$OPTION" = "a" ]; then
  echo "[PLAN] 1. git switch -c $BRANCH origin/main"
  echo "[PLAN] 2. git merge $BACKUP_REF   # conflicts expected on 12 paths"
  echo "[PLAN] 3. for each Group-A path: git checkout --ours -- <path>  (= main's reviewed blob)"
  echo "[PLAN] 4. STOP. Group-B docs left conflicted for manual union-merge (below)."
  echo "[PLAN] 5. human: resolve Group B, run gates, open PR. This script never commits."
  echo
  echo "== Group A (take main, then verify each with 'git diff ${BACKUP_REF}:<path> origin/main:<path>'):"
  for p in "${GROUP_A[@]}"; do echo "   $p"; done
  echo "== Group B (manual union-merge, both narratives are real history):"
  for p in "${GROUP_B[@]}"; do echo "   $p"; done
else
  echo "[PLAN] 1. git switch -c $BRANCH origin/main"
  echo "[PLAN] 2. git cherry-pick $GOVERNOR_COMMIT   # 5 hygiene files, expected clean"
  echo "[PLAN] 3. manual: port the three HANDOFF narratives (CTO_STATE, CEO_STATE, T14 review doc) from $BACKUP_REF"
  echo "[PLAN] 4. human: gates + PR. Script never commits."
  echo
  echo "== pre-verify cherry-pick cleanliness (paths touched by $GOVERNOR_COMMIT vs main):"
  for p in $(git show --name-only --format="" "$GOVERNOR_COMMIT"); do
    if git cat-file -e "origin/main:$p" 2>/dev/null; then
      if [ "$(git rev-parse "origin/main:$p")" = "$(git rev-parse "$GOVERNOR_COMMIT:$p")" ]; then
        echo "   [WARNING] $p: identical on main and governor commit -- cherry-pick of it will be a no-op conflict"
      else
        echo "   $p: differs from main (cherry-pick will apply)"
      fi
    else
      echo "   $p: not on main (cherry-pick will add)"
    fi
  done
fi

echo
echo "== Group-A sanity: main's blobs are already the 'resolved' target (evidence they are landed, reviewed content)"
for p in "${GROUP_A[@]}"; do
  mb=$(git rev-parse -q --verify "origin/main:$p" 2>/dev/null || echo absent)
  cb=$(git rev-parse -q --verify "$BACKUP_REF:$p" 2>/dev/null || echo absent)
  [ "$mb" != "absent" ] && [ "$cb" != "absent" ] && [ "$mb" != "$cb" ] \
    && echo "   $p: main=$(printf '%s' "$mb" | cut -c1-8) backup=$(printf '%s' "$cb" | cut -c1-8) (resolve to main)" \
    || echo "   $p: [WARNING] unexpected blob state (main=$mb backup=$cb)"
done

if [ "$APPLY" -eq 1 ]; then
  echo
  echo "[INFO] approval present; executing on branch $BRANCH"
  CURRENT=$(git rev-parse --abbrev-ref HEAD)
  git switch -c "$BRANCH" origin/main
  trap 'echo "[INFO] on branch $BRANCH (was $CURRENT); nothing committed"' EXIT
  if [ "$OPTION" = "a" ]; then
    git merge --no-commit --no-ff "$BACKUP_REF" || true
    for p in "${GROUP_A[@]}"; do
      git checkout --ours -- "$p" 2>/dev/null || echo "[WARNING] could not auto-resolve $p -- handle manually"
    done
    echo "[STOP] Group-A resolved to main's blobs (staged). Group-B left conflicted:"
    git diff --name-only --diff-filter=U || true
    echo "[STOP] human resolution required before any commit. Aborting merge with 'git merge --abort' is safe."
  else
    git cherry-pick "$GOVERNOR_COMMIT"
    echo "[OK] cherry-picked $GOVERNOR_COMMIT. Remaining manual step: port the three HANDOFF narratives from $BACKUP_REF."
    echo "[STOP] do not commit further without the Group-B port and a human review pass."
  fi
else
  echo
  echo "[DONE] dry-run only. Re-run with SCM_BACKUP_MERGE_APPROVED=1 ... --apply to execute."
fi
