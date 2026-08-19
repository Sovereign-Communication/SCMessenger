#!/usr/bin/env bash
# pr_scope.sh -- answer "unless there's a reason not to?" for a pull request.
#
# WHY
# On 2026-08-15 PR #150 was described as "tooling-only, zero build risk" from
# memory of what had been authored in it. The operator said "merge it! (unless
# there's a reason not to?)". There were three:
#   - it was branched off `tracking` but aimed at `main`, so its diff was 100
#     commits / +17k lines -- effectively all of PR #139, which would have
#     merged sideways under a commit message about delegation scripts
#   - two required Android checks were FAILING
#   - it touched core/src/crypto and core/src/transport, which AGENTS.md rule 8
#     holds merge-blocked pending adversarial review
# All three were visible in one `gh pr view --json files`. None were visible in
# the author's recollection.
#
# So the question stopped being rhetorical and became this script. Run it before
# every merge. It prints reasons NOT to, or says there are none.
#
# On 2026-08-15 PR #139 produced a FALSE NEGATIVE on the crypto review gate
# because `gh pr view --json files` returns at most 100 files, and PR #139
# changes 215 files (232 in full diff). The six gated files:
#   core/src/crypto/backup.rs
#   core/src/transport/addr_filter.rs
#   core/src/transport/behaviour.rs
#   core/src/transport/dial_policy.rs
#   core/src/transport/observation.rs
#   core/src/transport/swarm.rs
# were past file #100, so the script printed [OK] clear while +1,645/-154 lines
# of gated crypto and transport changes with a HIGH severity bug went unseen.
# The script itself failed open, on the largest PR in the repo, on the exact check
# it was built for.
#
# Fix: derive file lists from git (`git diff --name-only origin/<base>...origin/<head>`),
# not the GitHub API. Fall back to the API only if git refs cannot be fetched (loudly
# reported in output), and fail closed if the API returns exactly 100 files
# (truncation tripwire).
#
# 2026-08-16 -- NO SILENT TRUNCATION, ANYWHERE.
# The 2026-08-15 repair fixed the file list and left four other caps in place.
# All four hid data inside a script whose only job is to show it:
#   - commit count came from `gh pr view --json commits`, capped at 100. PR #139
#     printed "100 commits"; git counts 193. Wrong by 48%.
#   - the merge-blocked file list was piped through `head -8`, hiding gated
#     crypto/transport files past the eighth -- in the check that exists to
#     surface them.
#   - failing check names were sliced to [:6], running ones to [:4].
# All removed. git is authoritative for both files and commits; the API is a
# loudly-announced fallback only.
#
# The governing rule, now AGENTS.md rule 15: VISIBILITY FAILS OPEN, THE VERDICT
# FAILS CLOSED. Reduced confidence is expressed by printing MORE -- a [WARNING]
# beside the number, its provenance, a tripwire when a value sits exactly on a
# known API cap -- never by showing less. Anything that cannot be determined
# becomes a [BLOCKER]. Truncating data and blocking a merge are opposite moves;
# only the second one is safe.
#
#   scripts/pr_scope.sh 150
#
# Exit 0 = no blockers found. Exit 1 = at least one blocker. Read-only.

set -uo pipefail
PR="${1:-}"
REPO="Sovereign-Communication/SCMessenger"

if [ -z "$PR" ]; then
  echo "usage: scripts/pr_scope.sh <pr-number>"
  exit 2
fi

J=$(gh pr view "$PR" --json title,state,baseRefName,headRefName,mergeable,mergeStateStatus,additions,deletions,files,commits 2>/dev/null)
if [ -z "$J" ]; then
  echo "[FAIL] could not read PR $PR"
  exit 2
fi

BASE_REF=$(echo "$J" | python3 -c 'import json,sys;print(json.load(sys.stdin)["baseRefName"])')
HEAD_REF=$(echo "$J" | python3 -c 'import json,sys;print(json.load(sys.stdin)["headRefName"])')

BLOCKERS=0
note() { echo "  [BLOCKER] $*"; BLOCKERS=$((BLOCKERS+1)); }
ok()   { echo "  [OK]      $*"; }
# warn() surfaces a caveat WITHOUT blocking. It exists so that reduced
# confidence never has to be expressed by hiding data. Visibility fails OPEN
# (always print everything, flag what is uncertain); the verdict fails CLOSED
# (anything undetermined becomes a [BLOCKER] via note()).
warn() { echo "  [WARNING] $*"; }

# Derive file list from git (reliable, no 100-file API pagination limit).
# Fall back to GitHub API only if git fetch/diff fails.
FILE_SOURCE="git"
FILES_LIST=""
API_FILES_COUNT=0
BASE_REV=""
HEAD_REV=""

if git fetch origin "+refs/heads/${BASE_REF}:refs/remotes/origin/${BASE_REF}" "+refs/heads/${HEAD_REF}:refs/remotes/origin/${HEAD_REF}" >/dev/null 2>&1 && \
   git rev-parse --verify "origin/${BASE_REF}" >/dev/null 2>&1 && \
   git rev-parse --verify "origin/${HEAD_REF}" >/dev/null 2>&1; then
  BASE_REV="origin/${BASE_REF}"; HEAD_REV="origin/${HEAD_REF}"
elif git fetch origin "${BASE_REF}" "${HEAD_REF}" >/dev/null 2>&1 && \
     git rev-parse --verify "origin/${BASE_REF}" >/dev/null 2>&1 && \
     git rev-parse --verify "origin/${HEAD_REF}" >/dev/null 2>&1; then
  BASE_REV="origin/${BASE_REF}"; HEAD_REV="origin/${HEAD_REF}"
elif git fetch origin "+refs/heads/${BASE_REF}:refs/remotes/origin/${BASE_REF}" "+refs/pull/${PR}/head:refs/remotes/origin/pr/${PR}" >/dev/null 2>&1 && \
     git rev-parse --verify "origin/${BASE_REF}" >/dev/null 2>&1 && \
     git rev-parse --verify "origin/pr/${PR}" >/dev/null 2>&1; then
  BASE_REV="origin/${BASE_REF}"; HEAD_REV="origin/pr/${PR}"
fi

if [ -n "$BASE_REV" ]; then
  FILES_LIST=$(git diff --name-only "${BASE_REV}...${HEAD_REV}" 2>/dev/null)
else
  FILE_SOURCE="api"
  FILES_LIST=$(echo "$J" | python3 -c 'import json,sys;print("\n".join(f["path"] for f in json.load(sys.stdin)["files"]))')
  API_FILES_COUNT=$(echo "$J" | python3 -c 'import json,sys;print(len(json.load(sys.stdin)["files"]))')
fi

# Commit count. git is AUTHORITATIVE. `gh pr view --json commits` silently caps
# at 100: on 2026-08-16 it reported PR #139 as "100 commits" where
# `git rev-list --count origin/main..origin/tracking/pre-v040-tag-work` = 193.
# Same API-cap class as the file-list bug above, in a different field. The
# blocker still fired, so this hid scope rather than admitting a bad merge --
# but a number that is wrong by 48% is not evidence, and rule 13 asks for
# evidence. No cap, no sampling: count them all, and say where the number
# came from.
COMMIT_SOURCE="git"
COMMITS=""
if [ -n "$BASE_REV" ]; then
  COMMITS=$(git rev-list --count "${BASE_REV}..${HEAD_REV}" 2>/dev/null)
fi
if [ -z "$COMMITS" ]; then
  COMMIT_SOURCE="api"
  COMMITS=$(echo "$J" | python3 -c 'import json,sys;print(len(json.load(sys.stdin)["commits"]))')
fi

TOTAL_FILES=$(python3 -c '
import sys
files = [line.strip() for line in sys.stdin if line.strip()]
print(len(files))
' <<< "$FILES_LIST")

echo "=============================================================="
echo " PR #$PR  $(echo "$J" | python3 -c 'import json,sys;print(json.load(sys.stdin)["title"])')"
echo "=============================================================="
# NOTE: no f-strings with quoted keys here. Inside a single-quoted shell string,
# escaped quotes inside an f-string become a SyntaxError. Cost two runs to learn
# (this script and scripts/agy_stream_watch.py). Use %-formatting.
echo "$J" | python3 -c '
import json, sys
d = json.load(sys.stdin)
print("  state       : %s  mergeable=%s  %s" % (d["state"], d["mergeable"], d["mergeStateStatus"]))
print("  base <- head: %s <- %s" % (d["baseRefName"], d["headRefName"]))
print("  size        : +%d/-%d across %d files, %s commits" % (
    d["additions"], d["deletions"], '"$TOTAL_FILES"', "'"$COMMITS"'"))
'
# Rule 13: every number above must say where it came from, so a reader can
# tell a measurement from an assumption without re-running the script.
echo "  provenance  : files=${FILE_SOURCE}  commits=${COMMIT_SOURCE}  (git = authoritative, api = capped at 100)"
echo
echo "-- reasons not to merge --"

# Fallback warning and truncation tripwire
if [ "$FILE_SOURCE" = "api" ]; then
  note "FALLBACK TO GITHUB API: git fetch failed; file list derived from API (max 100 files)"
  if [ "$API_FILES_COUNT" -eq 100 ]; then
    note "TRUNCATION TRIPWIRE: API returned exactly 100 files (diff likely truncated by API limit)"
  fi
fi

# 1. Scope: a PR far larger than its title suggests is usually mis-based.
if [ "$COMMIT_SOURCE" = "api" ]; then
  warn "commit count came from the GitHub API, which caps at 100 -- the true"
  warn "  count may be higher. Authoritative: git rev-list --count <base>..<head>"
  if [ "$COMMITS" -eq 100 ]; then
    note "TRUNCATION TRIPWIRE: API reported exactly 100 commits -- assume truncated"
  fi
fi
if [ "$COMMITS" -gt 20 ]; then
  note "$COMMITS commits. Is this branch based on the branch you are merging INTO?"
  note "  Check: git log --oneline <base>..<head> | wc -l"
else
  ok "$COMMITS commits, $TOTAL_FILES files -- scope is reviewable"
fi

# 2. Merge-blocked directories (AGENTS.md rule 8).
GATED=$(python3 -c '
import sys, re
pat = re.compile(r"^core/src/(crypto|transport|routing|privacy)/")
files = [line.strip() for line in sys.stdin if line.strip()]
hits = [f for f in files if pat.match(f)]
print("\n".join(hits))
' <<< "$FILES_LIST")

if [ -n "$GATED" ]; then
  GATED_N=$(printf '%s\n' "$GATED" | grep -c .)
  # Print EVERY gated file. This list was previously piped through `head -8`,
  # which silently hid merge-blocked files past the eighth -- inside the exact
  # check whose entire purpose is to make them visible. A reviewer cannot sign
  # off on files a tool declined to show them. No cap, ever.
  note "touches merge-blocked directories (AGENTS.md rule 8) -- $GATED_N file(s):"
  printf '%s\n' "$GATED" | sed 's/^/              /'
  note "  requires a crypto-security-auditor verdict before merge"
else
  if [ "$FILE_SOURCE" = "api" ] && [ "$API_FILES_COUNT" -eq 100 ]; then
    note "cannot verify merge-blocked directories: API file list truncated at 100 files"
  else
    ok "clear of core/src/{crypto,transport,routing,privacy}"
  fi
fi

# 3. Check state. FAILS CLOSED.
#
# The first version of this block wrote to /tmp and read it back with python3.
# Git Bash on Windows maps /tmp; python3 does not, so the read raised, the
# counts came back as empty strings, `${PENDING:-0}` defaulted to 0, and the
# whole thing fell through to "all checks green" while five of six checks were
# still IN_PROGRESS. A silent failure that produces a false PASS is worse than
# no check at all, in a script whose only job is to stop a bad merge.
# Two fixes: repo-local tmp/ per AGENTS.md rule 2, and no path where an error
# can be mistaken for success.
mkdir -p "$(git rev-parse --show-toplevel)/tmp"
CHECKS="$(git rev-parse --show-toplevel)/tmp/_prchecks_${PR}.json"
if ! gh pr checks "$PR" --json name,state > "$CHECKS" 2>/dev/null || [ ! -s "$CHECKS" ]; then
  note "could not read checks -- treating as a blocker, not as green"
else
  SUMMARY=$(python3 - "$CHECKS" <<'PYEOF' 2>&1
import json, sys
try:
    d = json.load(open(sys.argv[1], encoding="utf-8"))
except Exception as e:
    print("ERROR %s" % e)
    raise SystemExit(0)
failed = [c["name"] for c in d if c.get("state") == "FAILURE"]
busy = [c["name"] for c in d if c.get("state") in ("IN_PROGRESS", "QUEUED", "PENDING")]
if failed:
    # No [:6] / [:4] slicing. A truncated failure list sends you to fix the
    # lanes you can see while the ones past the cap stay red and unnamed.
    print("FAILED %s" % ", ".join(failed))
elif busy:
    print("BUSY %d %s" % (len(busy), ", ".join(busy)))
elif not d:
    print("ERROR no checks reported")
else:
    print("GREEN %d" % len(d))
PYEOF
)
  case "$SUMMARY" in
    GREEN*)  ok "all ${SUMMARY#GREEN } checks green" ;;
    BUSY*)   note "checks still running: ${SUMMARY#BUSY } -- not green YET" ;;
    FAILED*) note "failing checks: ${SUMMARY#FAILED }" ;;
    *)       note "check state unreadable ($SUMMARY) -- treating as a blocker" ;;
  esac
fi

# 4. Mergeability.
MG=$(echo "$J" | python3 -c 'import json,sys;print(json.load(sys.stdin)["mergeable"])')
[ "$MG" = "MERGEABLE" ] && ok "no conflicts" || note "mergeable=$MG"

echo
if [ "$BLOCKERS" -eq 0 ]; then
  echo "[OK] no reasons not to merge were found."
  exit 0
fi
echo "[STOP] $BLOCKERS reason(s) not to merge. Resolve or get an explicit operator"
echo "       decision naming each one. A 'yes' given before these were surfaced"
echo "       was not informed consent."
exit 1
