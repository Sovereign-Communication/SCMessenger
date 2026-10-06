#!/usr/bin/env bash
# backup_purge.sh v2 (SAFE) -- operator directives 2026-09-27/28:
#   Back up ALL local SCMessenger work to GitHub and prove it, then reclaim ONLY what is
#   non-destructive to remove. Uncommitted work (WIP) is never removed, reset or cleaned:
#   it is backed up to GitHub and LEFT IN PLACE.
#
# Canonical copy: scripts/backup_purge.sh on PR #403 (branch docs/merge-train-v040-v050-20260927).
# Paths below are absolute, so it runs from any cwd; outputs go to scm-train-state/backup_purge/.
#
# Usage (Git Bash):  bash C:/Users/SCM/Documents/GitHub/wt-train-plan/scripts/backup_purge.sh <mode> [--dry-run]
#   inventory      read-only: worktrees, dirty counts, unpublished branches, disk
#   capture-test P read-only for worktree P: build its backup commit, do not push
#   list-test P    read-only for worktree P: list its ignored non-build files and sizes
#   backup         push every working state + unpublished ref to origin under
#                  refs/heads/backup/<DATE>/...; upload ignored non-build files (tmp/, logs/,
#                  train state) to the DRAFT release backup-local-<DATE> (maintainers only)
#   verify         read-only: every ledger item has an identical copy on GitHub
#   purge          removes ONLY:
#                    (1) worktrees that are clean, fully on GitHub and hold no local-only data
#                    (2) lane scratch copies whose full working state is byte-identical to a
#                        rescue commit on GitHub
#                    (3) local branches whose commits are all on GitHub
#                    (4) regenerable build output, via scripts/reclaim_safe.py
#                  Everything else is kept and listed. --dry-run prints decisions only.
#   report         write RECOVERY.md (what is where, how to restore) and upload it
#
# Never, in any mode: reset, clean, stash, checkout or switch a checkout; delete tmp/ or any
# non-build file; upload secrets or node/app data stores; force-push; delete a remote ref;
# git clean -x; cargo clean; reflog expiry; gc --prune=now.
set -u
DATE=20260927
GH=/c/Users/SCM/Documents/GitHub
REPO=$GH/SCMessenger
STATE=$GH/scm-train-state
OUT=$STATE/backup_purge
NS=backup/$DATE
REL=backup-local-$DATE
SLUG=Sovereign-Communication/SCMessenger
KEEP_WT_RE='^C:/Users/SCM/Documents/GitHub/(SCMessenger|wt-train|wt-train-plan)$'
SCRATCH_PREFIX='C:/Users/SCM/Documents/GitHub/scm-train-state/scratch/'
MAX_BLOB=52428800      # >50 MB never goes into git (GitHub hard limit 100 MB)
SCAN_MAX=5242880       # content-scan text blobs up to 5 MB
PART=1900m             # release asset split size (GitHub limit 2 GiB)
NAME_RE='\.(pem|key|p12|pfx|jks|keystore|mobileprovision|kdbx)$|(^|/)\.env|\.env(\.|$)|secret|credential|passw|(^|/)id_(rsa|ed25519|ecdsa)'
# Keep-local (never uploaded): node/app data stores and device backups -- identity keys and
# message history live in these.
KEEP_RE='(^|/)(db|conf|snap\.[0-9a-f]+)$|/blobs/|pixel-data-backup|identity-backup|/shared_prefs/|\.(db|sqlite|sqlite3|realm|db-wal|db-shm)$'
# Secret content: known token formats, quoted key=value literals, env-style NAME=value lines.
# Deliberately NOT matching code like `secret_key: kp.signing_key` (no quoted literal).
TEXT_RE='-----BEGIN [A-Z ]*PRIVATE KEY|ghp_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{20,}|AKIA[0-9A-Z]{16}|aws_secret_access_key[[:space:]]*=|xox[baprs]-[A-Za-z0-9-]{10,}|sk-(ws-|or-v1-|ant-)?[A-Za-z0-9_-]{24,}|gsk_[A-Za-z0-9]{20,}|AIza[0-9A-Za-z_-]{35}|([Pp]assword|[Pp]asswd|[Aa]pi[_-]?[Kk]ey|[Ss]ecret[_-]?[Kk]ey|[Aa]uth[_-]?[Tt]oken|[Aa]ccess[_-]?[Tt]oken)["'"'"']?[[:space:]]*[:=][[:space:]]*["'"'"'][A-Za-z0-9/+_.=-]{12,}["'"'"']|^[[:space:]]*(export[[:space:]]+)?[A-Z0-9_]*(PASSWORD|PASSWD|SECRET|API_KEY|APIKEY|TOKEN|ACCESS_KEY)[A-Z0-9_]*[[:space:]]*=[[:space:]]*[^[:space:]$]{8,}'

MODE=${1:-inventory}; ARG2=${2:-}; DRY=0; [ "$ARG2" = "--dry-run" ] && DRY=1
mkdir -p "$OUT/assets" "$OUT/lists"
LEDGER=$OUT/ledger.tsv; EXCL=$OUT/excluded.tsv; LOG=$OUT/run.log
touch "$LEDGER" "$EXCL"
say()  { echo "$*" | tee -a "$LOG"; }
die()  { say "[FAIL] $*"; exit 1; }
win()  { cygpath -m "$1"; }
unx()  { cygpath -u "$1"; }
run()  { if [ "$DRY" = 1 ]; then say "[DRY] $*"; else say "[RUN] $*"; "$@"; fi; }
worktrees() { git -C "$REPO" worktree list --porcelain | sed -n 's/^worktree //p'; }
label_of()  { local p=${1#C:/Users/SCM/Documents/GitHub/}; p=${p//\//--}; echo "${p// /_}"; }
is_keep()   { printf '%s' "$1" | grep -qE "$KEEP_WT_RE"; }
is_scratch(){ case "$1" in "$SCRATCH_PREFIX"*) return 0;; esac; return 1; }
remote_sha(){ git -C "$REPO" ls-remote origin "refs/heads/$1" | cut -f1; }
ledger_add(){ printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$@" >> "$LEDGER"; }
no_builds() { ! tasklist 2>/dev/null | grep -qiE '^(cargo|rustc|gradle|java)\.exe'; }

# ---- capture: full working state of a worktree as a commit, without touching it ----
# Secret-flagged and >50 MB entries are dropped from the capture (HEAD version kept there)
# and recorded in excluded.tsv. They stay local; a worktree holding them is never removed.
scrub_index() { # idx wt label kind
  local idx=$1 wt=$2 label=$3 kind=$4 u f sha sz why
  u=$(unx "$wt")
  while IFS= read -r -d '' f; do
    sha=$(GIT_INDEX_FILE="$idx" git -C "$wt" rev-parse --verify -q ":$f" 2>/dev/null) || continue
    [ "$kind" = worktree ] && [ ! -f "$u/$f" ] && continue
    why=""
    if printf '%s' "$f" | grep -qiE "$NAME_RE"; then why=secret-name
    else
      sz=$(git -C "$wt" cat-file -s "$sha")
      if [ "$sz" -gt "$MAX_BLOB" ]; then why=oversize
      elif [ "$sz" -le "$SCAN_MAX" ] && git -C "$wt" cat-file -p "$sha" | grep -qIE -e "$TEXT_RE"; then why=secret-content; fi
    fi
    [ -z "$why" ] && continue
    GIT_INDEX_FILE="$idx" git -C "$wt" reset -q HEAD -- "$f" >/dev/null 2>&1 \
      || GIT_INDEX_FILE="$idx" git -C "$wt" rm -q --cached -- "$f" >/dev/null 2>&1
    printf '%s\t%s\t%s\t%s\t%s\n' "$label" "$kind" "$why" "$f" "$sha" >> "$EXCL"
  done < <(GIT_INDEX_FILE="$idx" git -C "$wt" diff --cached --name-only -z HEAD)
}
capture() { # wt label -> CAP_HEAD CAP_TREE CAP_C CAP_IC (C/IC empty when nothing to save)
  local wt=$1 label=$2 idx ridx htree itree
  awk -F'\t' -v l="$label" '$1!=l' "$EXCL" > "$EXCL.tmp" && mv "$EXCL.tmp" "$EXCL"
  CAP_HEAD=$(git -C "$wt" rev-parse HEAD); htree=$(git -C "$wt" rev-parse "HEAD^{tree}")
  idx="$(win "$OUT")/idx-$label"; rm -f "$idx" "$idx.real"
  ridx=$(git -C "$wt" rev-parse --path-format=absolute --git-path index)
  # Start from a copy of the REAL index (what git status sees): unchanged files keep their
  # cached blobs, so CRLF-committed files are not re-hashed into whole-file rewrites.
  cp "$ridx" "$idx" 2>/dev/null || GIT_INDEX_FILE="$idx" git -C "$wt" read-tree HEAD
  GIT_INDEX_FILE="$idx" git -C "$wt" add -A -- . 2>>"$LOG"
  scrub_index "$idx" "$wt" "$label" worktree
  CAP_TREE=$(GIT_INDEX_FILE="$idx" git -C "$wt" write-tree)
  itree=""
  if cp "$ridx" "$idx.real" 2>/dev/null; then
    scrub_index "$idx.real" "$wt" "$label" index
    itree=$(GIT_INDEX_FILE="$idx.real" git -C "$wt" write-tree 2>/dev/null) || itree=""
  fi
  CAP_C=""; CAP_IC=""
  [ "$CAP_TREE" != "$htree" ] && CAP_C=$(git -C "$wt" commit-tree "$CAP_TREE" -p "$CAP_HEAD" \
     -m "backup($DATE): full working state of $label (tracked edits + untracked; secrets/oversize excluded)")
  if [ -n "$itree" ] && [ "$itree" != "$htree" ] && [ "$itree" != "$CAP_TREE" ]; then
    CAP_IC=$(git -C "$wt" commit-tree "$itree" -p "$CAP_HEAD" -m "backup($DATE): staged index of $label")
  fi
  rm -f "$idx" "$idx.real"
}

# ---- ignored, non-build files (tmp/, logs/, local notes) -> release list, or keep-local ----
# Bulk operations only: per-file process spawns are far too slow on Windows.
classify() { # label root: lists/<label>.all -> lists/<label>.files + EXCL rows
  local label=$1 root=$2 L=$OUT/lists/$1 k
  grep -z -iE "$KEEP_RE" "$L.all" > "$L.keep"
  grep -z -v -iE "$KEEP_RE" "$L.all" | grep -z -iE "$NAME_RE" > "$L.secretname"
  grep -z -v -iE "$KEEP_RE" "$L.all" | grep -z -v -iE "$NAME_RE" > "$L.cand"
  ( cd "$root" && xargs -0 -r grep -l -Z -I -E -e "$TEXT_RE" < "$L.cand" 2>/dev/null ) > "$L.secretcontent"
  if [ -s "$L.secretcontent" ]; then
    grep -z -v -x -F -f <(tr '\0' '\n' < "$L.secretcontent") "$L.cand" > "$L.files"
  else cp "$L.cand" "$L.files"; fi
  for k in keep:keep-local secretname:secret-name secretcontent:secret-content; do
    tr '\0' '\n' < "$L.${k%%:*}" | awk -v l="$label" -v w="${k#*:}" 'NF{printf "%s\tignored\t%s\t%s\t-\n", l, w, $0}' >> "$EXCL"
  done
}
ignored_list() { # wt label -> lists/<label>.files: ignored, non-build files under the worktree
  local wt=$1 label=$2 u n rel prune=()
  u=$(unx "$wt"); printf '%s' "$u" > "$OUT/lists/$label.path"
  while IFS= read -r n; do
    case "$n" in "$wt"/*) rel=${n#"$wt"/}; prune+=( -path "./$rel" -prune -o );; esac
  done < <(worktrees)
  ( cd "$u" && find . "${prune[@]}" \( -name .git -o -name target -o -name node_modules -o -name .gradle \
      -o -name build -o -name __pycache__ -o -name .cxx -o -name .venv -o -name venv \) -prune -o \
      -type f ! -name '*.pyc' -printf '%P\0' ) \
    | git -C "$wt" check-ignore --stdin -z > "$OUT/lists/$label.all" 2>/dev/null
  classify "$label" "$u"
}
state_list() { # train state dir (not a git worktree): everything except scratch/ and backup_purge/
  printf '%s' "$STATE" > "$OUT/lists/scm-train-state.path"
  awk -F'\t' '$1!="scm-train-state"' "$EXCL" > "$EXCL.tmp" && mv "$EXCL.tmp" "$EXCL"
  ( cd "$STATE" && find . \( -path ./scratch -o -path ./backup_purge \) -prune -o -type f -printf '%P\0' ) \
    > "$OUT/lists/scm-train-state.all"
  classify scm-train-state "$STATE"
}
sizes_of() { # label -> "size<TAB>path" lines for its current list
  local L=$OUT/lists/$1
  ( cd "$(cat "$L.path")" && xargs -0 -r stat -c '%s	%n' < "$L.files" ) 2>/dev/null
}

do_inventory() {
  git -C "$REPO" fetch -q --prune origin
  say "== inventory $(date -u +%FT%TZ)  disk: $(df -h /c | tail -1)"
  local wt n keep b
  while IFS= read -r wt; do
    n=$(git -C "$wt" status --porcelain 2>/dev/null | wc -l)
    if is_keep "$wt"; then keep=KEEP; elif [ "$n" -gt 0 ]; then keep=KEEP-WIP; else keep=candidate; fi
    say "worktree | $keep | dirty=$n | $(git -C "$wt" rev-parse --abbrev-ref HEAD) | $wt"
  done < <(worktrees)
  say "local branches: $(git -C "$REPO" for-each-ref refs/heads | wc -l); stashes: $(git -C "$REPO" stash list | wc -l)"
  git -C "$REPO" for-each-ref refs/heads --format='%(refname:short)' | while read -r b; do
    n=$(git -C "$REPO" rev-list --count "refs/heads/$b" --not --remotes=origin)
    [ "$n" -gt 0 ] && say "unpublished branch | $n commit(s) | $b"
  done
  say "(dirty scratch copies may still be removed by purge if byte-identical to a rescue commit on GitHub)"
  no_builds || say "[WARNING] a build tool (cargo/rustc/gradle/java) is running; purge will refuse"
}

do_backup() {
  git -C "$REPO" fetch -q --prune origin
  : > "$LEDGER"
  local wt label ref b sha safe diffhits lst base src tb f
  while IFS= read -r wt; do
    label=$(label_of "$wt")
    capture "$wt" "$label"
    if [ -n "$CAP_C" ]; then
      ref="$NS/wip/$label/${CAP_TREE:0:12}"
      git -C "$REPO" push -q origin "$CAP_C:refs/heads/$ref" 2>>"$LOG" || say "[FAIL] push $ref"
      ledger_add wip "$label" "$wt" "$CAP_C" "$ref" "$CAP_TREE"
    elif [ -n "$(git -C "$REPO" rev-list -n1 "$CAP_HEAD" --not --remotes=origin)" ]; then
      ref="$NS/head/$label/${CAP_HEAD:0:12}"
      git -C "$REPO" push -q origin "$CAP_HEAD:refs/heads/$ref" 2>>"$LOG" || say "[FAIL] push $ref"
      ledger_add head "$label" "$wt" "$CAP_HEAD" "$ref" "-"
    else
      ledger_add clean "$label" "$wt" "$CAP_HEAD" "-" "$(git -C "$wt" rev-parse 'HEAD^{tree}')"
    fi
    if [ -n "$CAP_IC" ]; then
      ref="$NS/wip/$label/index-${CAP_IC:0:12}"
      git -C "$REPO" push -q origin "$CAP_IC:refs/heads/$ref" 2>>"$LOG" || say "[FAIL] push $ref"
      ledger_add index "$label" "$wt" "$CAP_IC" "$ref" "-"
    fi
    ignored_list "$wt" "$label"
    say "[OK] captured $label  wip=${CAP_C:0:12} index=${CAP_IC:0:12} ignored-files=$(tr -cd '\0' < "$OUT/lists/$label.files" | wc -c)"
  done < <(worktrees)
  # unpublished local branches and stashes (secret-scanned), under the backup namespace
  git -C "$REPO" for-each-ref refs/heads --format='%(refname:short)' | while read -r b; do
    [ -z "$(git -C "$REPO" rev-list -n1 "refs/heads/$b" --not --remotes=origin)" ] && continue
    sha=$(git -C "$REPO" rev-parse "refs/heads/$b"); safe=${b//\//--}
    diffhits=$(git -C "$REPO" log -p "refs/heads/$b" --not --remotes=origin | grep -E '^\+' | grep -cIE -e "$TEXT_RE")
    if [ "$diffhits" -gt 0 ]; then say "[KEPT-LOCAL] branch $b: $diffhits secret-pattern line(s); not pushed"; continue; fi
    ref="$NS/branch/$safe/${sha:0:12}"
    git -C "$REPO" push -q origin "$sha:refs/heads/$ref" 2>>"$LOG" || say "[FAIL] push $ref"
    ledger_add branch "$b" "-" "$sha" "$ref" "-"
  done
  git -C "$REPO" stash list --format='%gd %H' | while read -r s sha; do
    ref="$NS/stash/${s//[^0-9]/}/${sha:0:12}"
    git -C "$REPO" push -q origin "$sha:refs/heads/$ref" 2>>"$LOG" || say "[FAIL] push $ref"
    ledger_add stash "$s" "-" "$sha" "$ref" "-"
  done
  # non-git files -> draft release (secrets and node/app data are never uploaded)
  state_list
  rm -f "$OUT"/assets/*
  for lst in "$OUT"/lists/*.files; do
    base=$(basename "$lst" .files); src=$(cat "$OUT/lists/$base.path")
    sizes_of "$base" > "$OUT/lists/$base.sizes.bak"
    [ -s "$lst" ] || continue
    tb="$OUT/assets/$base.tar.gz"
    tar --null -czf "$tb" -C "$src" -T "$lst" 2>>"$LOG" || { say "[FAIL] tar $base"; continue; }
    if [ "$(stat -c %s "$tb")" -gt 1992294400 ]; then split -b "$PART" -d "$tb" "$tb.part-" && rm -f "$tb"; fi
  done
  ( cd "$OUT/assets" && sha256sum * > SHA256SUMS.txt )
  { echo "Local backup $DATE of SCMessenger checkouts (DRAFT: maintainers only; never publish)."
    echo; echo "Restore: download, check SHA256SUMS.txt, cat *.part-* > x.tar.gz if split, tar -xzf into the worktree root."
    echo "Git working states are on branches refs/heads/$NS/... (see RECOVERY.md)."; echo; echo '```'; cat "$OUT/assets/SHA256SUMS.txt"; echo '```'; } > "$OUT/MANIFEST.md"
  if gh release view "$REL" -R "$SLUG" >/dev/null 2>&1; then
    gh release upload "$REL" -R "$SLUG" --clobber "$OUT"/assets/* || die "release upload failed"
  else
    gh release create "$REL" -R "$SLUG" --draft --prerelease --target main \
      --title "Local backup $DATE (draft, maintainers only)" --notes-file "$OUT/MANIFEST.md" "$OUT"/assets/* || die "release create failed"
  fi
  for f in "$OUT"/assets/*; do ledger_add asset "$(basename "$f")" "-" "$(stat -c %s "$f")" "$REL" "$(sha256sum "$f" | cut -c1-64)"; done
  say "[OK] backup done: $(grep -c '^wip' "$LEDGER") wip, $(grep -c '^index' "$LEDGER") index, $(grep -c '^branch' "$LEDGER") branch, $(grep -c '^asset' "$LEDGER") asset; next: bash $0 verify"
}

do_verify() {
  git -C "$REPO" fetch -q --prune origin
  local kind label path sha ref extra ok=0 bad=0 remote_assets r
  remote_assets=$(gh release view "$REL" -R "$SLUG" --json assets --jq '.assets[] | "\(.name)\t\(.size)"' 2>/dev/null)
  : > "$OUT/verified.tsv"
  while IFS=$'\t' read -r kind label path sha ref extra; do
    case "$kind" in
      clean) r=$([ -z "$(git -C "$REPO" rev-list -n1 "$sha" --not --remotes=origin)" ] && echo ok || echo missing);;
      asset) r=$(printf '%s\n' "$remote_assets" | awk -F'\t' -v n="$label" -v s="$sha" '$1==n && $2==s{print "ok"}'); r=${r:-missing};;
      *)     r=$([ "$(remote_sha "$ref")" = "$sha" ] && echo ok || echo missing);;
    esac
    printf '%s\t%s\t%s\n' "$r" "$kind" "$label" >> "$OUT/verified.tsv"
    if [ "$r" = ok ]; then ok=$((ok+1)); else bad=$((bad+1)); say "[FAIL] not on GitHub: $kind $label $ref"; fi
  done < "$LEDGER"
  say "[INFO] verify: $ok ok, $bad missing"
  [ "$bad" -eq 0 ] && [ "$ok" -gt 0 ]
}

verified() { grep -qP "^ok\t$1\t\Q$2\E$" "$OUT/verified.tsv"; }
asset_ok() { # label: ignored files unchanged since backup AND their release asset(s) verified
  local base=$1 L=$OUT/lists/$1 a n=0
  sizes_of "$base" > "$L.sizes.now"
  cmp -s "$L.sizes.now" "$L.sizes.bak" 2>/dev/null || return 1   # changed since backup (or never backed up)
  [ -s "$L.files" ] || return 0                                  # nothing ignored to lose
  for a in $(ls "$OUT/assets/" | grep -E "^${base//./\\.}\.tar\.gz"); do n=$((n+1)); verified asset "$a" || return 1; done
  [ "$n" -gt 0 ]
}
dup_on_origin() { # tree -> 0 if an origin rescue/* commit has exactly this tree
  local t=$1 c
  for c in $(git -C "$REPO" for-each-ref --format='%(objectname)' refs/remotes/origin/rescue); do
    [ "$(git -C "$REPO" rev-parse "$c^{tree}")" = "$t" ] && return 0
  done
  return 1
}

do_purge() {
  [ -s "$OUT/verified.tsv" ] || die "run backup, then verify, first"
  no_builds || die "a build tool is running (cargo/rustc/gradle/java; gradle daemons: gradlew --stop); re-run purge after"
  git -C "$REPO" fetch -q --prune origin
  local before wt label why removed=0 keptwip=0 keptdata=0 blocked=0 out b
  before=$(df -h /c | tail -1)
  : > "$OUT/decisions.tsv"
  while IFS= read -r wt; do
    if is_keep "$wt"; then printf 'KEEP\t%s\n' "$wt" >> "$OUT/decisions.tsv"; continue; fi
    label=$(label_of "$wt")
    capture "$wt" "$label"
    ignored_list "$wt" "$label"
    if grep -qP "^\Q$label\E\t" "$EXCL"; then why="KEEP-LOCAL-DATA"
    elif [ -z "$CAP_C" ] && [ -z "$CAP_IC" ]; then
      if [ -n "$(git -C "$REPO" rev-list -n1 "$CAP_HEAD" --not --remotes=origin)" ]; then why="BLOCKED-HEAD-NOT-ON-GITHUB"
      elif ! asset_ok "$label"; then why="BLOCKED-IGNORED-FILES-CHANGED-OR-UNVERIFIED"
      else why="REMOVE-CLEAN"; fi
    elif is_scratch "$wt" && dup_on_origin "$CAP_TREE"; then
      if asset_ok "$label"; then why="REMOVE-DUPLICATE"; else why="BLOCKED-IGNORED-FILES-CHANGED-OR-UNVERIFIED"; fi
    else why="KEEP-WIP"; fi
    printf '%s\t%s\n' "$why" "$wt" >> "$OUT/decisions.tsv"
    case "$why" in
      REMOVE-*) say "[$why] $wt"; removed=$((removed+1))
        run git -C "$REPO" worktree remove --force "$wt" || say "[WARNING] remove failed (locked files?): $wt"
        if [ "$DRY" = 0 ] && [ -d "$(unx "$wt")" ]; then say "[WARNING] directory still present: $wt"; fi;;
      KEEP-WIP) say "[KEEP-WIP] $wt (backed up on GitHub, left in place)"; keptwip=$((keptwip+1));;
      KEEP-LOCAL-DATA) say "[KEEP-LOCAL-DATA] $wt (holds secret or node/app data; left in place)"; keptdata=$((keptdata+1));;
      *) say "[$why] $wt"; blocked=$((blocked+1));;
    esac
  done < <(worktrees)
  [ "$DRY" = 1 ] || git -C "$REPO" worktree prune
  # local branches: delete when every commit is on GitHub and no worktree has it checked out
  out=$(git -C "$REPO" worktree list --porcelain | sed -n 's#^branch refs/heads/##p')
  local nb=0 kb=0
  while read -r b; do
    [ "$b" = main ] && continue
    printf '%s\n' "$out" | grep -qxF "$b" && continue
    if [ -n "$(git -C "$REPO" rev-list -n1 "refs/heads/$b" --not --remotes=origin)" ]; then say "[KEPT] branch $b has unpublished commits"; kb=$((kb+1)); continue; fi
    nb=$((nb+1)); if [ "$DRY" = 1 ]; then echo "[DRY] git branch -D $b" >> "$LOG"; else git -C "$REPO" branch -q -D "$b"; fi
  done < <(git -C "$REPO" for-each-ref refs/heads --format='%(refname:short)')
  # regenerable build output only
  if [ "$DRY" = 1 ]; then ( cd "$REPO" && python scripts/reclaim_safe.py --reclaim --dry-run ) 2>&1 | tail -6 | tee -a "$LOG"
  else ( cd "$REPO" && python scripts/reclaim_safe.py --reclaim && python scripts/reclaim_safe.py --reclaim-shared-target ) 2>&1 | tail -10 | tee -a "$LOG"; fi
  say "[INFO] worktrees: removed=$removed keep-wip=$keptwip keep-local-data=$keptdata blocked=$blocked"
  say "[INFO] local branches deleted=$nb kept-unpublished=$kb (dry-run=$DRY)"
  say "[INFO] disk before: $before"; say "[INFO] disk after:  $(df -h /c | tail -1)"
}

do_report() {
  { echo "# SCMessenger local backup $DATE -- RECOVERY"; echo
    echo "Git working states and refs are on origin under refs/heads/$NS/:"; echo
    echo "| kind | source | sha | remote ref |"; echo "|---|---|---|---|"
    awk -F'\t' '$1!="asset"{printf "| %s | %s | %s | %s |\n",$1,$2,substr($4,1,12),$5}' "$LEDGER"; echo
    echo "Restore a working state:  git fetch origin <ref> && git worktree add <path> FETCH_HEAD"; echo
    echo "Non-git files: DRAFT release $REL (assets + SHA256SUMS.txt). Restore: gh release download $REL -R $SLUG"; echo
    echo "Purge decisions (worktrees):"; echo; echo '```'; cat "$OUT/decisions.tsv" 2>/dev/null; echo '```'; echo
    echo "Kept local on purpose (never uploaded): $(grep -cP '\t(secret|keep-local|oversize)' "$EXCL") item(s) --"
    echo "secrets and node/app data stores. Paths: $EXCL (local file; do not publish)."
  } > "$OUT/RECOVERY.md"
  gh release upload "$REL" -R "$SLUG" --clobber "$OUT/RECOVERY.md" >/dev/null 2>&1 && say "[OK] RECOVERY.md uploaded to $REL"
  say "[OK] $OUT/RECOVERY.md written"
}

case "$MODE" in
  inventory) do_inventory;;
  capture-test) wt=$(win "$ARG2"); label=$(label_of "$wt"); capture "$wt" "$label"
    say "capture-test $label head=${CAP_HEAD:0:12} tree=${CAP_TREE:0:12} wip=${CAP_C:-none} index=${CAP_IC:-none}"
    say "excluded for $label:"; grep -P "^\Q$label\E\t" "$EXCL" | cut -f2-4 | tee -a "$LOG";;
  list-test) wt=$(win "$ARG2"); label=$(label_of "$wt"); ignored_list "$wt" "$label"
    n=$(tr -cd '\0' < "$OUT/lists/$label.files" | wc -c)
    bytes=$(sizes_of "$label" | awk -F'\t' '{s+=$1} END{print s+0}')
    say "list-test $label: $n ignored non-build file(s), $((bytes/1048576)) MB to upload"
    say "excluded (secret/keep-local) for $label: $(grep -cP "^\Q$label\E\tignored" "$EXCL") item(s)";;
  backup) do_backup;;
  verify) do_verify;;
  purge) do_purge;;
  report) do_report;;
  *) die "unknown mode $MODE";;
esac
