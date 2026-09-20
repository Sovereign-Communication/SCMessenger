# Canonical Outlier Audit -- GLM 5.3 Flash iterative inventory

Status: FINAL for this pass (iterations 0-6 complete 2026-09-19). Tracking PR: https://github.com/Sovereign-Communication/SCMessenger/pull/335. Reports: HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_INDEX.md. Remediation is follow-on work; do not merge PR without orchestrator/operator review.
Priority: P1 -- feeds the 0.4.0 gate, 0.5.0 parity, and the 1.0.0 unification ledger
Lane: Freebuff / GLM 5.3 Flash
Capability class: FREEBUFF LANE (see AGENTS.md)
Mode: REPORT-ONLY. Do not implement product code fixes in this task.
Branch (fixed): `glm/canonical-outlier-audit`

---

## 0. What you are (capability class)

You are GLM 5.3 Flash running in Freebuff desktop. Freebuff has no headless
mode; the operator pastes this file as your whole brief. You have no memory of
prior sessions. Everything you need is on this page or in the files it names.

Rules that bind you (from AGENTS.md + docs/rules/FREEBUFF.md -- these override
anything you think you remember):

1. Shared checkout. Touch ONLY files this task creates or assigns. Never
   revert, stash, delete, or commit a file you did not create. A clean
   `git status` is NOT a goal.
2. You MAY open a PR for your own work on YOUR branch. You may NOT merge it.
   Green CI is necessary, not sufficient.
3. Never `git push` to `main`. Never force-push any branch. Never delete a
   branch/worktree. Never tag a release.
4. Never drive the Pixel / any handset UI. No `input tap`, no `am start` to
   reach a screen. Logs may be read if present; do not provoke the device.
5. Never edit UniFFI-generated bindings (`uniffi.api` Kotlin package,
   `core/target/generated-sources/`). Read-only.
6. Storage access only through `core/src/store/` / `IronCore`. This task does
   not change storage anyway.
7. Changes under `core/src/{crypto,transport,routing,privacy}/` need Rule-8
   adversarial review from a non-author before merge. This task does NOT
   implement those changes; you only inventory outliers there.
8. NO EMOJI anywhere -- code, docs, comments, logs, commit messages. Use
   `[OK]` / `[ERROR]` / `[WARNING]` / `[INFO]` / `[DONE]` / `[FAIL]` /
   `[OPEN]` / `[UNVERIFIED]`.
9. Temp/evidence files ONLY under repo-local `tmp/` -- never the system temp
   dir.
10. DESCRIBE ONLY WHAT YOU HAVE READ THIS SESSION. Every claim cites a command
    you ran (with output) or a `file:line` you opened. Not memory, not
    filenames, not what a prior audit said without re-reading it.
11. NO SILENT TRUNCATION. If you enumerate findings, files, PRs, or tags,
    print ALL of them. No `head -N` as a reporting limit. If a tool/API caps
    results, say so and mark the count `[WARNING]`. Prefer local git over
    GitHub API for branch/commit facts.
12. Every status line carries one of: exact command + output, a run URL, or
    `UNVERIFIED`. `UNVERIFIED` is an acceptable and useful answer.
13. Never read `$?` after a pipe. Capture first:
    `cmd > out.txt; rc=$?; <inspect out.txt>; exit $rc`
    Example trap: `cargo fmt --check | head; echo $?` reports `head`, always 0.
14. Builds: check that no other `cargo` / `gradlew` is live first (this host
    serializes builds). Run `python scripts/disk_budget.py` before any build.
    A BLOCKED exit (2) means do not build -- use CI artifacts if needed.
    Prefer CI as the wide verifier. This audit is mostly read + search.
15. If a premise in this file is wrong when you hit the code, STOP and write
    `HANDOFF/freebuff/inbox/` with `Type: PREMISE-WRONG`. Do not invent a fix.

---

## 1. Mission

Perform a comprehensive, evidence-backed inventory of **canonical outliers**
across SCMessenger code, docs, and process artifacts.

A **canonical outlier** is any place that contradicts an authority this repo
has already declared, or that uses a non-canonical form where a canonical form
exists.

This is NOT a security adversarial review (Rule-8 seats own that). This is
NOT a go/no-go on the 0.4.0 tag (operator/CTO own that). This produces the
finding set those seats use.

**Target consumers of this audit:**

| Release | How findings feed it |
|---|---|
| **0.4.0** | Outliers that block a trustworthy public alpha: false doc claims, identifier mismatches that break interop, wiring orphans that make shipped features unreachable, doctrine violations that ship wrong operator-facing language |
| **0.5.0** | Parity gaps and canonical-identifier / doctrine debt that would harden into iOS/macOS work if not named now |
| **1.0.0** | Unification debt: dual stores, dual ID schemes, stale authority docs, open canonical findings that must close before GA |

Tag every finding with `Target: 0.4.0 | 0.5.0 | 1.0.0 | process | unknown`.

---

## 2. Canonical authorities (read these first; they define "outlier")

Open and read each before searching. Cite them; do not paraphrase from memory.

| Authority | Path | What it pins |
|---|---|---|
| Universal agent contract | `AGENTS.md` | Nodes-not-relays doctrine, hard rules 1-17, Freebuff class, Rule 16 wiring, Rule 15 no-silent-truncation |
| Freebuff lane rules | `docs/rules/FREEBUFF.md` | Task contract, evidence contract, may/may-not |
| Ship plan (execution until v0.4.0 tag) | `SHIP_PLAN.md` | D1-D7 definition of done; section 6 Freebuff queue; false-claim ledger patterns |
| Long-horizon sequencing | `HANDOFF/V1_0_0_EXECUTION_PLAN.md` | 0.4.0 / 0.5.0 / 1.0.0 scope; superseded-for-execution header |
| Live dispatch pointer | `HANDOFF/todo/_QUEUE.md` | Points v0.4.0 work at Freebuff queue |
| Doc status index | `docs/DOCUMENT_STATUS_INDEX.md` | Which docs are claimed canonical |
| Documentation chain root | `DOCUMENTATION.md` | Entry chain; must not contradict code |
| Identity contract | `core/src/api.udl` (IdentityInfo comments), `core/src/message/types.rs` (UNIFICATION_V2), `core/src/identity/keys.rs` | Canonical identity = `public_key_hex`; peer_id and identity_id are derived |
| Prior doctrine inventory | `HANDOFF/audit/doctrine_violation_inventory.md` | Baseline + exception registry for "relay" noun usage |
| Prior identifier audit | `HANDOFF/audit/IDENTIFIER_PARITY_AUDIT_2026-09-15.md` | Prior PASS + latent hazards; re-verify, do not assume still true |
| Prior multi-dim audit | `HANDOFF/audit/MULTIDIMENSIONAL_AUDIT_2026-09-17.md` | Open 0.4.0 blockers and iteration loop |
| Prior shadow audit | `HANDOFF/audit/SHADOW_AUDIT_V040_V050_ADVERSARIAL_REVIEW_2026-09-16.md` | Critical/high findings that may still be open |
| Canonical doc reconcile ticket | `HANDOFF/freebuff/queue/V040_T11_CANONICAL_DOC_RECONCILE.md` | Correct-in-place style; what T11 already aimed at |
| Wiring checker | `scripts/check_wiring.py` | Executable form of Rule 16 |
| Docs sync gate | `scripts/docs_sync_check.sh` | Canonical-doc mechanical gate |

**Architecture doctrine (AGENTS.md), restated for search keys:**

- There are NO standalone relays. Only NODES; every node relays (store-and-forward custody).
- Say "cloud node" / "node". "Relay" is a VERB or a technical identifier
  (`RelayCustodyStore`, `cmd_relay`, `relay_custody_msg_*`), not a role noun.
- Discovery is LEDGER SHARING (invite/QR-seeded, gossip-propagated). Bootstrap
  address lists are a deprecated transitional mechanism.
- Full parity: CLI, Android, iOS, cloud run the same node with the same relay behavior.
- Canonical identity for persistence/exchange/cross-platform resolution is
  Ed25519 `public_key_hex` (64 hex). `libp2p_peer_id` (base58 `12D3KooW...`)
  and `identity_id` are derived metadata.

**Exception registry (NOT outliers -- do not file as violations):**

- libp2p circuit-relay protocol types/paths (`/p2p-circuit`, `libp2p::relay::*`)
- Rust module declarations (`pub mod relay;`) and crate paths that must keep compiling
- UniFFI / generated iOS bindings under `iOS/SCMessengerCore.xcframework/`
- Test hostname literals like `relay.example.com` when they are not participant nouns
- Wire protocol identifiers that are intentionally SCMessenger protocol names
  -- mark those `NEEDS-HUMAN` rather than "fix the string"

Full exception detail: re-read `HANDOFF/audit/doctrine_violation_inventory.md` section 1.

---

## 3. Audit dimensions (work these in order)

Each finding gets an ID: `CO-<DIM>-<nnn>` (example `CO-ID-007`).

### M0 -- Baseline snapshot (milestone 1)

1. Record: `git rev-parse HEAD`, `git status --short`, `git branch --show-current`.
2. Record tags (ALL): `git tag --list` and identify newest / whether `v0.4.0` final exists.
3. Record versions from source, not docs: `Cargo.toml` workspace version; Android
   `versionName`/`versionCode` if readable from gradle files.
4. Save raw command outputs under `tmp/canonical_audit_2026-09-19/`.
5. Write report file iteration 0 skeleton (path in section 5).

### DIM-A -- Doctrine noun / role outliers (nodes not relays)

Search code + active docs for "relay" used as a **network-participant noun**:
dedicated relay, bootstrap relay, relay node, relay server, "the relay",
architecture diagrams that draw a relay as a separate role.

Commands to start (adapt; do not stop at one grep):

```
rg -n -i "\b(relay node|relay nodes|dedicated relay|bootstrap relay|relay server|the relay)\b" --glob '!target/**' --glob '!**/node_modules/**'
rg -n "every node is a full relay|no standalone relay|nodes, not relays" -g '*.md' -g '*.rs' -g '*.kt'
```

Classify each hit: `EXCEPTION` | `DOCTRINE-VIOLATION` | `NEEDS-HUMAN` | `ALREADY-CORRECT-EXPLAINS-DOCTRINE`.
Compare against the prior inventory; file NEW outliers as findings and mark
prior inventory rows still open as `STILL-OPEN` with current file:line.

### DIM-B -- Canonical identifier outliers

Authority: identity = `public_key_hex`. Peer-id base58 and `identity_id` are derived.

Search for places that **persist, exchange, or key stores on a non-canonical form**
when the doctrine says they must not:

```
rg -n "peer_id|public_key_hex|identity_id|canonical" core/src/store core/src/identity cli/src --glob '!target/**'
rg -n "12D3KooW" core cli android/app/src/main --glob '!**/test/**' --glob '!**/generated*/**'
rg -n "queue:|peer_id.to_string|contact.peer_id" cli/src
```

Also re-check claims from `IDENTIFIER_PARITY_AUDIT_2026-09-15.md`:
- Kotlin still only mirrors peer-id math in cold-start fallback?
- Storage still only through IronCore?
- Any new dual-keying in outbox/contacts/ledger since that audit?

Outlier examples to file:
- Sled/outbox/contact keys written as base58 but read as 64-hex (or reverse)
- Docs that call peer_id "the canonical id"
- Android/iOS code that reimplements blake3 identity_id
- API/docs that instruct persisting `libp2p_peer_id` or `identity_id` as primary key

### DIM-C -- Wiring / reachability outliers (Rule 16)

```
python scripts/check_wiring.py
```

Paste FULL output into the report. Then spot-check any claimed dead routes /
unregistered composables / services without manifest entries by opening the
named file:line. File each unreachable feature as `CO-WIR-nnn` with the three
Rule-16 checks: implementation exists / referenced / reference itself reachable.

Android specifics to verify if the checker or path search flags them:
nav route defined but no `composable(route)`;
Service/Receiver without manifest entry.

### DIM-D -- Canonical doc vs code / doc vs doc outliers

Inventory the claimed-canonical set from `docs/DOCUMENT_STATUS_INDEX.md` and
`DOCUMENTATION.md`. For each path: exists? last-updated header? claims match
source?

Checks that must run as commands, not prose:

```
# version claims
rg -n "v0\.[0-9]|0\.[0-9]\.[0-9]|v1\.0\.0" Cargo.toml DOCUMENTATION.md docs/CURRENT_STATE.md SHIP_PLAN.md AGENTS.md CLAUDE.md 2>/dev/null
git tag --list
git status -sb
```

Specific known-stale patterns to verify still stale or now fixed:
- `docs/CURRENT_STATE.md` release-line claims vs `Cargo.toml`
- Whether any doc still describes a standalone "relay" product role
- `DOCUMENTATION.md` links -- do paths exist?
- `SHIP_PLAN.md` D1-D7 vs what HANDOFF claims is done
- Contradictions between `HANDOFF/CTO_STATE.md`, `MULTIDIMENSIONAL_AUDIT_2026-09-17.md` section 9, and actual open todo tickets
- T11 (`V040_T11_CANONICAL_DOC_RECONCILE.md`) -- did it land? What remains?

Style if you later fix docs (NOT this task unless operator issues a follow-on):
correct in place, keep the original visible, date the correction. Do not delete
a wrong claim to make a contradiction disappear.

```
bash scripts/docs_sync_check.sh
```

Capture exit code correctly (no pipe before `$?`). Record whether the gate is
green or red on current HEAD.

### DIM-E -- Architecture doctrine outliers in code/comments/docs

Search for:
- Hardcoded bootstrap address lists presented as current design (deprecated)
- Claims of anonymous packet forwarders / dedicated bootstrap relays
- Direct sled access bypassing `IronCore` / `core/src/store/`
- Hardcoded IPs / AWS addresses in active (non-historical) docs
  (`HANDOFF/gpt/AWS_RELAY_CURRENT_ADDRESS.md` is the known address pointer --
  cite it; flag other copies)

```
rg -n "bootstrap (nodes|peers|relays)|hardcoded" docs HANDOFF core/src cli/src --glob '!HANDOFF/audit/**' --glob '!docs/historical/**'
rg -n "sled::" core/src cli/src --glob '!target/**'
```

### DIM-F -- Cross-platform parity / 0.5.0 / 1.0.0 debt candidates

From docs + code surfaces, list claimed-vs-actual capability gaps that would
become canonical debt if iOS/macOS starts from false docs:

- FFI surface contract / UniFFI claims vs `core/src/api.udl`
- Android features with no CLI/iOS counterpart (or reverse) that docs claim are universal
- Open Apple-lane PRs/docs referenced in prior audits (file paths; do not invent PR state -- if you cannot query, mark `UNVERIFIED`)

### DIM-G -- Process / queue / audit-history outliers

- Freebuff `README.md` queue table vs files that actually exist in `queue/`
  and `inbox/` / `done/`
- Open findings in prior audits (`SHADOW_AUDIT_*`, `MULTIDIMENSIONAL_AUDIT_*`)
  that are still present in tree / still described as open in todo tickets
- Ticket files under `HANDOFF/todo/` whose premise contradicts AGENTS doctrine
  (for example still saying "relay" as a product role, or claiming iOS is 0.4.0)

Do NOT rewrite `_QUEUE.md` execution order. Do NOT relitigate operator-settled
scope. Report contradictions; do not re-decide them.

---

## 4. Finding format (every row)

```
### CO-XXX-nnn -- short title
- Dimension: DIM-A | B | C | D | E | F | G
- Severity: BLOCKER-0.4.0 | HIGH | MED | LOW | PROCESS | UNVERIFIED
- Target: 0.4.0 | 0.5.0 | 1.0.0 | process | unknown
- Location: file:line (or "doc-wide", or command evidence)
- Authority contradicted: <AGENTS.md rule / UDL / doctrine line / prior audit section>
- Evidence: exact command + relevant output snippet, OR quote + path
- Why it is an outlier: one or two sentences
- Suggested remediation class: docs-correct | inventory-ticket | code-fix | needs-rule8 | needs-operator | none
- Status: OPEN | STILL-OPEN | NEEDS-HUMAN | EXCEPTION
```

Severity guidance:
- `BLOCKER-0.4.0` only when the outlier makes a D1-D7 exit criterion untrustworthy
  or ships false operator-facing claims about identity/architecture/release state.
- Identifier dual-key bugs that strand messages are `BLOCKER-0.4.0` / `HIGH`.
- Pure comment doctrine violations are usually `LOW`/`MED` unless operator-facing.

Never claim a gate passed without output. Never claim a PR is green without a
run URL. If you cannot verify, `UNVERIFIED`.

---

## 5. Where to save (paths -- use these exactly)

| Artifact | Path |
|---|---|
| Working evidence (raw cmd outputs, rg dumps) | `tmp/canonical_audit_2026-09-19/` |
| Iteration reports (append-only, one file per iter) | `HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_iter<N>.md` |
| Master index (updated each milestone) | `HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_INDEX.md` |
| Lane replies / DONE / BLOCKED / PREMISE-WRONG | `HANDOFF/freebuff/inbox/AUDIT_CANONICAL_OUTLIER_iter<N>_<type>_2026-09-19.md` |
| Your git branch | `glm/canonical-outlier-audit` |
| Your PR | open from that branch into `main`; title: `[audit] Canonical outlier inventory iter N` |

Report header template:

```
# Canonical Outlier Audit -- iteration <N>
Task: HANDOFF/freebuff/queue/AUDIT_CANONICAL_OUTLIERS_V040_V050_V100_2026-09-19.md
Date: 2026-09-19
Branch: glm/canonical-outlier-audit
HEAD: <git rev-parse HEAD>
Model: GLM 5.3 Flash (Freebuff)
Mode: REPORT-ONLY

## Snapshot commands
<paste outputs>

## Counts
<dimension>: <n> findings -- source command listed

## Findings
...

## Not done / UNVERIFIED
...

## Next iteration aim
...
```

INDEX file sections: HEAD snapshot, cumulative counts by dimension/severity/target,
list of iteration report paths, open BLOCKER-0.4.0 list, inbox messages written.

---

## 6. Milestone save protocol (required -- do not lose work)

Freebuff is interactive. A crash or a paste cycle can drop a long session.
**Persist at every milestone below before continuing.** Chat history is not a
save.

### Milestones

| ID | When | Must have written before you continue |
|---|---|---|
| **M0** | After baseline snapshot | `iter0` report (skeleton + snapshot outputs) + INDEX started |
| **M1** | After DIM-A complete | `iter1` report with all DIM-A findings + INDEX update |
| **M2** | After DIM-B complete | `iter2` (DIM-B) + INDEX |
| **M3** | After DIM-C complete | `iter3` (DIM-C, include full `check_wiring.py` output) + INDEX |
| **M4** | After DIM-D complete | `iter4` (DIM-D + docs_sync_check exit) + INDEX |
| **M5** | After DIM-E and DIM-F complete | `iter5` + INDEX |
| **M6** | After DIM-G + final triage | `iter6` FINAL + INDEX marked final for this pass |

After writing files for a milestone:

1. **Stage explicit paths only** -- never `git add -A`, never `git commit -a`.
   Stage only:
   - `HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_*`
   - `HANDOFF/freebuff/inbox/AUDIT_CANONICAL_OUTLIER_*` (if any)
   - optionally a short evidence pointer file under `tmp/canonical_audit_2026-09-19/`
     if you want it in-repo; raw noise can stay uncommitted in tmp/
2. Commit message pattern:
   `docs(audit): canonical outlier iter <N> -- <dimension summary>`
3. Push **only** your branch:
   ```
   git rev-parse --abbrev-ref HEAD
   # must print: glm/canonical-outlier-audit
   git push -u origin glm/canonical-outlier-audit
   ```
   If the branch does not exist yet at M0:
   ```
   git checkout -b glm/canonical-outlier-audit
   ```
   Base from current `main` as you found it at M0; if `main` moves under you,
   note it in the report -- do **not** force-push, do **not** rebase shared
   branches without operator direction.
4. Open or update the PR from `glm/canonical-outlier-audit` -> `main`.
   In the PR body, link the latest INDEX path and iteration path.
5. Write inbox DONE for that milestone:
   ```
   Task: AUDIT_CANONICAL_OUTLIERS_V040_V050_V100_2026-09-19.md
   Type: DONE
   Milestone: M<N>
   PR: <url or "UNVERIFIED -- push failed: <output>">
   Report: HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_iter<N>.md
   Counts: <dimension totals>
   ```
6. Only then continue to the next dimension.

### Safety rails on save

- If push fails, do not retry with `--force`. Capture the error, write inbox
  `Type: BLOCKED`, stop that save attempt, keep files on disk.
- If you see dirty files you do not recognize, leave them alone. Stage your
  paths only.
- Never commit: secrets, `target/`, `*.log`, `*.pid`, keystores, identity keys.
- Never move files in `HANDOFF/freebuff/queue/` or `done/` -- only the
  orchestrator/operator moves queue state.
- You may commit your own audit reports under `HANDOFF/audit/` and your own
  inbox notes under `HANDOFF/freebuff/inbox/`.
- You may NOT merge your PR even if CI is green.

### Mid-session loss recovery

If the session dies, paste this task file again and say:

```
Resume canonical outlier audit. Branch glm/canonical-outlier-audit.
Read HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_INDEX.md first.
Continue from the first incomplete milestone listed there.
```

---

## 7. Work method (anti-hallucination)

1. Read the authority docs in section 2 before searching.
2. For each dimension: run commands, open hits, classify, file findings.
3. Prefer local git and on-disk files over GitHub API. If you use `gh` or an
   API and it paginates, print the full result count and mark ceiling hits
   `[WARNING]`.
4. Prior audits are claims until you re-verify against current HEAD. You may
   cite them as historical context; a STILL-OPEN finding needs a current
   file:line or current command output.
5. Do not implement product fixes. Do not "tidy" nearby code. Do not rewrite
   other agents' HANDOFF history files.
6. Do not add new summary docs beyond the INDEX + iteration reports named in
   section 5.
7. Optional mechanical exception: you MAY run `python scripts/check_wiring.py`
   and `bash scripts/docs_sync_check.sh` as read-only gates. Do not "fix" the
   docs sync failures in this task; record them.

---

## 8. Acceptance criteria (this pass is done when)

1. INDEX exists at
   `HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_INDEX.md` and lists
   every iteration file path actually written.
2. Dimensions A-G each have an explicit section in some iteration report --
   even if the section says "0 findings" with the command that produced that
   zero.
3. Every finding follows section 4 and carries evidence or `UNVERIFIED`.
4. Counts in INDEX match the findings listed (no "and N more").
5. Full `python scripts/check_wiring.py` output is pasted somewhere in the
   iteration set.
6. `bash scripts/docs_sync_check.sh` exit status recorded with correct `$?`
   capture.
7. `git tag --list` and `Cargo.toml` version are in the M0 snapshot.
8. Branch `glm/canonical-outlier-audit` has been pushed at least once; PR
   opened or updated; inbox DONE notes exist for completed milestones.
9. No source files under `core/`, `cli/`, `android/`, `iOS/` were modified.
10. Inbox file written for any PREMISE-WRONG or BLOCKED condition rather than
    silent improvisation.

---

## 9. Out of scope (do not do)

- Implementing fixes for findings (follow-on tickets after operator triage)
- Rule-8 security verdicts or "this PR is safe to merge" judgments
- Tagging releases, publishing GitHub releases, merging PRs
- Driving devices / UI automation
- Relitigating settled architecture (operator rulings in CTO_STATE / SHIP_PLAN / V1_0_0_EXECUTION_PLAN remain settled)
- Editing `HANDOFF/todo/_QUEUE.md` execution order
- Mass-renaming `relay` identifiers in code (exception registry + compile risk)
- Rewriting `HANDOFF/audit/` history files from other sessions
- Creating `learning.md` / scratch files outside `tmp/` and the assigned paths

---

## 10. Rules recap (the only repo rules injected)

- No emoji; use bracket status tokens.
- Shared checkout; stage explicit paths; never commit others' work.
- You may open a PR on `glm/canonical-outlier-audit`; you may not merge.
- No force-push; no push to `main`.
- Evidence or `UNVERIFIED` -- claims are not gates.
- No silent truncation; print all findings/counts you rely on.
- Never `$?` after a pipe.
- Temp only in `tmp/`.
- Premise failure -> inbox `PREMISE-WRONG`, stop inventing work.

---

## 11. Final message format (when this pass is complete)

```
RESULT: DONE|BLOCKED|FAILED
VERIFICATION: NONE|LOCAL(<commands run>)
FILES: <paths you created/touched>
NOTES:
- HEAD <sha>, branch glm/canonical-outlier-audit
- Findings: A= B= C= D= E= F= G= TOTAL=
- BLOCKER-0.4.0: <count> (list IDs)
- PR: <url or UNVERIFIED>
- docs_sync_check exit: <n or UNVERIFIED>
- check_wiring.py: <exit or UNVERIFIED>
```

Keep NOTES under 10 lines. Detail lives in the INDEX and iteration reports.
