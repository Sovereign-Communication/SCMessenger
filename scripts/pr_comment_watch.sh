#!/usr/bin/env bash
# pr_comment_watch.sh -- poll a GitHub PR for new comments and append them to a
# log, so a lane that is busy building or soaking does not miss cross-lane
# traffic. Read-only: it never posts, never merges, never mutates the PR.
#
# Usage: scripts/pr_comment_watch.sh <pr-number> [interval-seconds] [log-path]
#
# State: <log-path>.seen holds the createdAt of the newest comment already
# recorded, so a restart does not re-log the whole thread (PR 139 has 149).

set -uo pipefail

PR="${1:?usage: pr_comment_watch.sh <pr-number> [interval] [log]}"
INTERVAL="${2:-120}"
LOG="${3:-tmp/logs/pr${PR}_comments.log}"
SEEN="${LOG}.seen"
ERRFILE="${LOG}.stderr"

mkdir -p "$(dirname "$LOG")"
touch "$LOG"
[ -f "$SEEN" ] || echo "1970-01-01T00:00:00Z" > "$SEEN"

# Prefer jq; some hosts in this fleet don't have it installed (confirmed
# absent on this host 2026-08-11 -- `which jq` / `where.exe jq` both come up
# empty), so fall back to an equivalent python3 JSON filter. Output is
# byte-identical to the jq path either way.
if command -v jq >/dev/null 2>&1; then
    HAVE_JQ=1
else
    HAVE_JQ=0
fi

json_len() {  # stdin: JSON array -> stdout: element count
    if [ "$HAVE_JQ" -eq 1 ]; then
        jq 'length'
    else
        python3 -c 'import json,sys; print(len(json.load(sys.stdin)))'
    fi
}

json_format() {  # stdin: JSON array of comments -> stdout: log-formatted entries
    if [ "$HAVE_JQ" -eq 1 ]; then
        jq -r '.[] | "=====[" + .createdAt + "] " + .author.login + "=====\n" + .body'
    else
        python3 -c '
import json, sys
for c in json.load(sys.stdin):
    print("=====[" + c["createdAt"] + "] " + c["author"]["login"] + "=====")
    print(c["body"])
'
    fi
}

json_newest() {  # stdin: JSON array of comments -> stdout: newest createdAt
    if [ "$HAVE_JQ" -eq 1 ]; then
        jq -r 'sort_by(.createdAt) | last | .createdAt'
    else
        python3 -c 'import json,sys; print(sorted(c["createdAt"] for c in json.load(sys.stdin))[-1])'
    fi
}

echo "[INFO] watching PR #${PR} every ${INTERVAL}s -> ${LOG}" | tee -a "$LOG"

consecutive_failures=0
FAILURE_ESCALATE_AT=3

while true; do
    last_seen="$(cat "$SEEN" 2>/dev/null || echo '1970-01-01T00:00:00Z')"

    # One API call per cycle. jq filters server-side via --jq so only genuinely
    # new comments cross the boundary. stderr now goes to ERRFILE (not
    # /dev/null) so a failing poll is visible in the log instead of spinning
    # silently forever.
    new_json="$(gh pr view "$PR" --json comments \
        --jq "[.comments[] | select(.createdAt > \"${last_seen}\")]" 2>"$ERRFILE")"
    gh_status=$?
    gh_stderr="$(cat "$ERRFILE" 2>/dev/null)"

    if [ "$gh_status" -ne 0 ]; then
        consecutive_failures=$((consecutive_failures + 1))
        echo "[WARNING] gh pr view failed (exit ${gh_status}, ${consecutive_failures} in a row): ${gh_stderr:-<no stderr>}" >> "$LOG"
        if [ "$consecutive_failures" -ge "$FAILURE_ESCALATE_AT" ]; then
            echo "[FAIL] gh pr view has failed ${consecutive_failures} consecutive cycles on PR #${PR}; cursor stuck at ${last_seen}" >> "$LOG"
        fi
        sleep "$INTERVAL"
        continue
    fi
    consecutive_failures=0

    if [ -n "$new_json" ] && [ "$new_json" != "[]" ]; then
        count="$(echo "$new_json" | json_len 2>/dev/null || echo 0)"
        if [ "${count:-0}" -gt 0 ]; then
            echo "$new_json" | json_format >> "$LOG"
            newest="$(echo "$new_json" | json_newest)"
            echo "$newest" > "$SEEN"
            echo "[NEW] ${count} comment(s) on PR #${PR}, newest ${newest}"
        fi
    fi

    sleep "$INTERVAL"
done
