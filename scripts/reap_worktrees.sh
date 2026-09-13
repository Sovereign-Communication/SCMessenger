#!/usr/bin/env bash
# reap_worktrees.sh -- report (and optionally remove) worktrees nobody is using.
#
# WHY THIS EXISTS
# AGENTS.md and CLAUDE.md tell every concurrent agent to take its own
# `git worktree`. Nothing ever told anyone to give it back. On 2026-08-15 there
# were 8 worktrees on a disk at 97% (7.5 GB free), two of them abandoned
# Antigravity subagent trees 344 commits behind, and low disk on this machine
# manifests as rustc crashes that read like source corruption
# (STATUS_STACK_BUFFER_OVERRUN, "can't find crate") -- so worktree litter shows
# up as fake compiler bugs, hours later, in someone else's session.
#
# A creation rule without a reaping rule is a leak with extra steps. This is the
# reaping rule, as a script rather than a paragraph, because the paragraph
# version of this lesson has already failed once.
#
#   scripts/reap_worktrees.sh            # report only (default, safe)
#   scripts/reap_worktrees.sh --remove   # remove only the ones judged SAFE
#   scripts/reap_worktrees.sh --tmp      # also report tmp/ TTL candidates (14d)
#   scripts/reap_worktrees.sh --tmp --remove
#
# SAFE means ALL of: not the main checkout, no uncommitted changes, no untracked
# files, and (fully merged OR its claim in tmp/_wt_claims.txt is absent or
# expired). Anything else is reported and left alone -- deleting a worktree with
# unpushed work is the same unrecoverable mistake as `git checkout -- .`.
# Note: a clean worktree's commits always survive removal via its branch ref;
# what dies with the directory is uncommitted work only -- which the DIRTY
# check above already refuses to touch.
#
# CLAIMS: an agent squatting on a worktree writes one line to tmp/_wt_claims.txt:
#     2026-09-13 alice tmp/wt-my-feature
# (ISO date, owner, repo-relative path). A fresh claim (<= CLAIM_TTL_DAYS)
# protects even a fully-merged tree, because the owner may be mid-follow-up.
# Claims expire silently -- the reaper then treats the tree as unclaimed.
#
# tmp/ TTL: evidence dirs and CI droppings under tmp/ older than TMP_TTL_DAYS
# are reported (and removed with --tmp --remove), EXCEPT registered worktrees
# and directories containing a non-empty .keep file (format: "owner: reason").
# An empty .keep is a visibility failure, not a keep: it is reported and
# ignored (rule 15 -- the verdict fails closed, the visibility fails open).

set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 1

REMOVE=0
SWEEP_TMP=0
for arg in "$@"; do
  case "$arg" in
    --remove) REMOVE=1 ;;
    --tmp)    SWEEP_TMP=1 ;;
    *) echo "[ERROR] unknown argument: $arg (expected --remove and/or --tmp)"; exit 2 ;;
  esac
done

CLAIM_TTL_DAYS=14
TMP_TTL_DAYS=14

# claim_status <abs-path> -> prints "owner|age_days" for the FRESHEST matching
# claim, or nothing if unclaimed. Date-parse failure counts as age 0: a claim
# we cannot read still protects the tree (conservative, fail-closed on the
# verdict, fail-open on visibility -- the raw line gets printed as unparseable).
claim_status() {
  local wt="$1"
  [ -f "$WT_CLAIMS" ] || return 0
  local best_days=99999 best_owner=""
  local now epoch days owner cpath line_date
  now=$(date +%s)
  while read -r line_date owner cpath; do
    [ -n "${cpath:-}" ] || continue
    case "$wt" in
      "$cpath"|*/"$cpath"|"$cpath"/*) ;;
      *) continue ;;
    esac
    if epoch=$(date -d "$line_date" +%s 2>/dev/null); then
      days=$(( (now - epoch) / 86400 ))
      [ "$days" -lt 0 ] && days=0
    else
      echo "[WARNING] unparseable claim date in $WT_CLAIMS: $line_date $owner $cpath" >&2
      days=0
    fi
    if [ "$days" -lt "$best_days" ]; then best_days=$days; best_owner=$owner; fi
  done < "$WT_CLAIMS"
  [ -n "$best_owner" ] && echo "$best_owner|$best_days"
  return 0
}

MAIN=$(git rev-parse --show-toplevel)
WT_CLAIMS="$MAIN/tmp/_wt_claims.txt"
printf '%-58s %-26s %-10s %s\n' "WORKTREE" "BRANCH" "STATE" "DISPOSITION"
printf '%.0s-' {1..118}; echo

TMPDIR="$MAIN/tmp"
mkdir -p "$TMPDIR"
WT_FILE="$TMPDIR/_wt.txt"
trap 'rm -f "$WT_FILE"' EXIT

safe_list=()
git worktree list --porcelain 2>/dev/null > "$WT_FILE"
path=""; branch=""; head=""
DURABLE=("origin/tracking/pre-v040-tag-work" "origin/main")

while IFS= read -r line; do
  case "$line" in
    worktree\ *) path="${line#worktree }" ;;
    HEAD\ *)     head="${line#HEAD }" ;;
    branch\ *)   branch="${line#branch refs/heads/}" ;;
    detached)    branch="(detached)" ;;
    "")
      [ -z "$path" ] && continue
      short="${path#"$(dirname "$MAIN")"/}"
      [ ${#short} -gt 56 ] && short="...${short: -53}"

      if [ "$path" = "$MAIN" ]; then
        printf '%-58s %-26s %-10s %s\n' "$short" "${branch:-?}" "MAIN" "keep (this is the checkout)"
        path=""; branch=""; head=""; continue
      fi
      if [ ! -d "$path" ]; then
        printf '%-58s %-26s %-10s %s\n' "$short" "${branch:-?}" "MISSING" "prunable: git worktree prune"
        path=""; branch=""; head=""; continue
      fi

      # Filter out unstaged markdown line-ending churn from #169
      real_dirty=$(git -C "$path" status --porcelain 2>/dev/null | grep -vE "^ M .*\.md\"?$" || true)
      if [ -n "$real_dirty" ]; then
        n=$(printf "%s\n" "$real_dirty" | grep -c . || echo "0")
        printf '%-58s %-26s %-10s %s\n' "$short" "${branch:-?}" "DIRTY" "KEEP -- $n uncommitted path(s); ask the owner"
        path=""; branch=""; head=""; continue
      fi

      unpushed=$(git -C "$path" rev-list --count "$head" --not --remotes 2>/dev/null || echo "?")
      if [ "$unpushed" != "0" ]; then
        printf '%-58s %-26s %-10s %s\n' "$short" "${branch:-?}" "UNPUSHED" "KEEP -- $unpushed commit(s) on no remote"
        path=""; branch=""; head=""; continue
      fi

      is_merged=0
      merge_detail=""
      errors=0
      for ref in "${DURABLE[@]}"; do
        git -C "$path" merge-base --is-ancestor "$head" "$ref" 2>/dev/null
        rc=$?
        if [ "$rc" -eq 0 ]; then
          is_merged=1
          merge_detail="merged into $ref"
          break
        elif [ "$rc" -ne 1 ]; then
          errors=1
        fi
      done

      if [ "$is_merged" -eq 0 ]; then
        # Fall back to GitHub PR query
        pr_num=$(gh pr list --state merged --search "$head" --json number --jq '.[0].number' 2>/dev/null || true)
        if [ -n "$pr_num" ] && [ "$pr_num" != "null" ]; then
          is_merged=1
          merge_detail="merged via PR #$pr_num"
        fi
      fi

      if [ "$is_merged" -eq 1 ]; then
        claim=$(claim_status "$path")
        if [ -n "$claim" ]; then
          owner=${claim%%|*}; cdays=${claim##*|}
          if [ "$cdays" -le "$CLAIM_TTL_DAYS" ]; then
            printf '%-58s %-26s %-10s %s\n' "$short" "${branch:-?}" "clean" "KEEP -- merged, but claimed by $owner ${cdays}d ago"
            path=""; branch=""; head=""; continue
          fi
        fi
        printf '%-58s %-26s %-10s %s\n' "$short" "${branch:-?}" "clean" "SAFE -- $merge_detail"
        safe_list+=("$path")
      elif [ "$errors" -ne 0 ]; then
        printf '%-58s %-26s %-10s %s\n' "$short" "${branch:-?}" "UNKNOWN" "KEEP -- git merge-base error (exit 128)"
      else
        ahead=$(git -C "$path" rev-list --count origin/main.."$head" 2>/dev/null || echo "?")
        claim=$(claim_status "$path")
        if [ -n "$claim" ]; then
          owner=${claim%%|*}; cdays=${claim##*|}
          printf '%-58s %-26s %-10s %s\n' "$short" "${branch:-?}" "clean" "KEEP -- claimed by $owner ${cdays}d ago; $ahead commit(s) not in main"
        else
          printf '%-58s %-26s %-10s %s\n' "$short" "${branch:-?}" "clean" "KEEP -- $ahead commit(s) not in main (unclaimed; removal is a human call)"
        fi
      fi
      path=""; branch=""; head=""
      ;;
  esac
done < "$WT_FILE"

# --- tmp/ TTL sweep -----------------------------------------------------------
if [ "$SWEEP_TMP" -eq 1 ]; then
  echo
  echo "== tmp/ entries older than ${TMP_TTL_DAYS}d =="
  now_ts=$(date +%s)
  tmp_candidates=()
  for entry in "$MAIN"/tmp/*; do
    [ -e "$entry" ] || continue
    name=$(basename "$entry")
    # The reaper's own bookkeeping is never sweepable.
    case "$name" in
      _wt_claims.txt|_wt.txt) continue ;;
    esac
    # Registered worktrees are the section above's problem, not this one's.
    if grep -qF "$entry" "$WT_FILE" 2>/dev/null; then
      printf '  %-52s SKIP -- registered worktree\n' "$name"
      continue
    fi
    keep_file="$entry/.keep"
    if [ -f "$keep_file" ]; then
      if [ -s "$keep_file" ]; then
        printf '  %-52s SKIP -- .keep: %s\n' "$name" "$(head -c 70 "$keep_file" | tr '\n' ' ')"
        continue
      fi
      printf '  %-52s [WARNING] empty .keep ignored -- write "owner: reason" in it\n' "$name"
    fi
    # GNU stat (Git Bash, Linux) first: -c %Y. BSD stat (macOS) second: -f %m.
    # Do NOT use `|| echo 0` here: mtime 0 would make every entry look ancient
    # and fail OPEN toward deletion. Unreadable mtime = skip, loudly.
    mtime=$(stat -c %Y "$entry" 2>/dev/null)
    if [ -z "$mtime" ]; then
      mtime=$(stat -f %m "$entry" 2>/dev/null)
    fi
    if [ -z "$mtime" ] || [ "$mtime" -eq 0 ] 2>/dev/null; then
      printf '  %-52s [WARNING] mtime unreadable -- skipped\n' "$name"
      continue
    fi
    age_days=$(( (now_ts - mtime) / 86400 ))
    if [ "$age_days" -lt "$TMP_TTL_DAYS" ]; then
      continue
    fi
    size=$(du -sm "$entry" 2>/dev/null | cut -f1)
    printf '  %-52s %4s MB  %3dd old\n' "$name" "${size:-?}" "$age_days"
    tmp_candidates+=("$entry")
  done
  if [ "${#tmp_candidates[@]}" -eq 0 ]; then
    echo "nothing past TTL."
  elif [ "$REMOVE" -eq 1 ]; then
    for p in "${tmp_candidates[@]}"; do
      if rm -rf "$p"; then
        echo "[OK]   removed $p"
      else
        echo "[FAIL] could not remove $p (left in place)"
      fi
    done
  else
    echo "report only. re-run with --tmp --remove to delete these."
  fi
fi

echo
echo "safe to remove: ${#safe_list[@]}"
if [ "${#safe_list[@]}" -eq 0 ]; then
  echo "nothing to reap."
  exit 0
fi
for p in "${safe_list[@]}"; do echo "    $p"; done

if [ "$REMOVE" -eq 0 ]; then
  echo
  echo "report only. re-run with --remove to delete the SAFE ones."
  exit 0
fi

for p in "${safe_list[@]}"; do
  if git worktree remove "$p" 2>/dev/null; then
    echo "[OK]   removed $p"
  else
    echo "[FAIL] could not remove $p (left in place)"
  fi
done
git worktree prune
echo "[OK] pruned stale administrative entries"
