# CTO state — live handoff

Status: Active
Last updated: 2026-08-16 (two CTO lanes reconciled into one seat)
Entry point: `/CTO`. This file is the whole context load.

**Seat: UNIFIED.** Two CTO sessions ran concurrently on 2026-08-15. Lane A
authored #149, #150, #151, #164, #167; lane B authored #152-#166. Both are
stopped. Their work was **complementary, not divergent** — every divergence
lane A flagged at closeout is resolved, and three of the four were already
resolved when it wrote them. See §9.

Everything below has a command next to it. **Re-derive before acting** — this
file ages, the repo does not.

---

## 1. The goal

Ship **v0.4.0 as an Android beta** the operator can hand to friends and family.
Then v0.5.0 iOS. `SHIP_PLAN.md` D1-D5 is the definition of done and the only
execution queue until the tag. **Nothing in v0.5.0/v1.0.0 scope starts before the
0.4.0 tag.**

Latest thing a stranger can download is **v0.1.9, from 2026-03-19.** That number
is the whole problem.

### Exit criteria status

| | Criterion | State |
|---|---|---|
| **D1** | `main` is green | **Blocked on #139, which is blocked on #169.** Two NEW CI blockers were found 2026-08-16 that no earlier handoff knew about; both are fixed in #169 and verified. See §9 |
| **D2** | Signed APK downloadable | **Unblocked** — all four signing secrets set 2026-08-15 17:08Z. Needs the tag |
| **D3** | README explains the product | **DONE** |
| **D4** | Two-device message + receipt | Blocked on D2 and a node rebuild; see §5 |
| **D5** | No long-lived integration branch | **Blocked on #139** (merging it satisfies this) |

---

## 2. Where the merge train stands

**Merged to `tracking` this sprint (11):** #149 UniFFI build fix + 7 restored
Android sources + orchestration tooling; #150 lane routing; #153 backlog amnesty
87→8; #157 circuit-relay transport prefix; #158 pr_scope truncation fix; #159 D4
runbook; #160 release notes; #161 dead-IP retirement; #162 five-layer integration
suites; #163 shared cargo cache docs; #164 hygiene; #146 Android durable delivery.

**Closed:** #147, with proof — `git log --oneline 7538e4e9 --not origin/tracking`
returned zero commits. It was a branch cut from `tracking` but aimed at `main`,
so GitHub rendered the whole tracking-vs-main delta as its own 102-file diff.
**Read the ancestry, not the diff stat.**

### Open right now

```
gh pr list --limit 20
gh pr checks 139 ; gh pr checks 165
bash scripts/pr_scope.sh 139        # the REPAIRED gate -- see §6
```

| PR | Base | What it is |
|---|---|---|
| **#169** | tracking | **MERGE THIS FIRST.** Fixes BOTH new D1 blockers — `.kt`/`.kts`/`.md` eol=lf, and rustfmt on a test file #162 landed unformatted. Verified: 1852 → 0 flagged, `cargo fmt --all -- --check` exit 0 |
| **#139** | main ← tracking | **THE TRUNK MERGE. D1 + D5 together.** Cannot go green until #169 lands |
| **#165** | tracking | Two transport defects fixed (see §4). 5 green, 1 in progress. **HELD** by the tracking freeze |
| #167 | tracking | Guard no longer fires on read-only `git`. **HELD.** Worth landing — the false positives are real; one fired on the CTO's own command 2026-08-16 |
| #168 | tracking | Stale-gate tripwire + 90m dispatch timeout floor. 67/67 verified. **HELD.** See §9 |
| #152 | main | Hygiene whitespace. **Likely redundant** once #139 carries #164 + #169 to main. Re-check, probably close |
| #154 | main | `apksigner verify` guard. **Merge this before tagging** — see §5 |
| #156 | main | Docker Integration Suite non-blocking + issue #155 |
| 13 dependabot | main | **DEFER all, close none.** They are the post-tag S4 queue. GitHub reports 7 vulnerabilities on the default branch, 3 high — real, but not before the tag |

**`tracking` is FROZEN except #169.** Every push to `tracking` restarts all 29
of #139's checks — that is what reset them at 01:0xZ on 2026-08-16 and cost an
hour. #165, #167, #168 and this document are queued behind the tag deliberately.

**#154, #156 and #152 cannot go green before #139 merges.** Verified from the
`Lint` log, not inferred: they are based on `main` and inherit its `cargo fmt`
break at `core/src/lib.rs:159`. The ordering is forced, not a preference.

Branch `archive/five-layer-original-drafts-20260816` preserves four obsolete
integration-test drafts, verbatim, before they were removed. Nothing references
it; it exists so the removal is a recorded decision. See §9.

---

## 3. Critical path to the tag

0. **#169 green → merge to tracking.** This is now step zero. Without it #139
   fails `Repository Hygiene Checks` AND `Rust Linting`. Merging it restarts
   #139's 29 checks; that cost is unavoidable and already priced in.
1. **#165 stays HELD** until after the trunk merge — merging it to tracking
   would restart #139 again for no D1 benefit.
2. **#139 → main.** This is D1 + D5. The repaired `pr_scope.sh` will raise five
   blockers; four are resolved and must be named explicitly rather than silently
   overridden:
   - *"100 commits, is this based on the branch you are merging into?"* —
     intentional here. `tracking` IS the long-lived integration branch, and
     merging it is precisely what D5 asks for.
   - *"touches merge-blocked directories"* — true, six files. **The
     crypto-security-auditor verdict exists** (§4). Its one HIGH finding is fixed
     by #157, which is merged into tracking, so #139 now carries the fix.
   - *"checks still running"* — must actually be green. Do not merge on a
     pending check.
   - *"no conflicts"* — clean.
3. `bash scripts/apply_branch_protection.sh --apply` (operator approved;
   `enforce_admins` true, **0** required approvals — raising it to 1 locks a
   single-operator repo out, GitHub forbids self-approval). **Do NOT list
   `Docker Integration Suite` as a required check** — see §6.
4. **Merge #154**, then tag `v0.4.0-alpha.1`.
5. Verify the published APK is genuinely release-signed (§5). **D2 + D3.**
6. Rebuild the AWS node to the tagged SHA, then run D4 (§5).

### Why every red lane on `main` is red

Verified from the literal CI logs, not inferred:

| Lane | Real cause | Fix |
|---|---|---|
| `CI` | its only failing job was `Lint` → `cargo fmt` on `core/src/lib.rs:159` | #139 |
| `Lint` | the same single fmt diff | #139 |
| `Mobile` | KSP `error.NonExistentClass` — the UniFFI bug | #139 (carries #149) |
| `Repository Hygiene` | trailing whitespace | #164 (merged) |
| `Docker Integration Suite` | UniFFI metadata stripping in the container | #156, non-blocking |

`CI`'s other five jobs (Test on windows/ubuntu/macos, FFI Surface Contract, Docs)
all PASSED on `main`. The lane was red on one formatting diff.

---

## 4. Security review of #139 — verdict on record

A `crypto-security-auditor` pass ran against the six merge-blocked files #139
touches (`core/src/crypto/backup.rs`, and `addr_filter/behaviour/dial_policy/
observation/swarm` under `core/src/transport/`; +1,645/-154).

**Verdict: NEEDS FIXES.** No CRITICAL hole; X25519 and XChaCha20-Poly1305
untouched. Nothing was found that admits, dials, or discloses to a peer that
should be rejected — the diff actually *tightens* several gaps (stale-connection
disclosure, fail-open block checks, nested relay circuits).

- **HIGH — `dial_policy.rs` `build_relay_addresses`. FIXED by #157 (merged).**
  The loop set `has_ip`/`has_port` for the Ip4/Ip6 and Tcp/Udp match arms but
  never pushed those components, so circuit-relay addresses lost their transport
  prefix entirely. libp2p's relay transport requires a concrete address and fails
  with `MissingRelayAddr` before any I/O, at debug log level only. Relay NAT
  traversal was broken mesh-wide, in the same change set that removed UPnP.
- **LOW** — blocked-peer status is a distinguishable oracle: Registration and
  Relay answer with an explicit `"blocked"` error while AddressReflection and
  LedgerExchange stay silent. Post-tag.
- **LOW/INFO** — `mdns_dial_attempted` is unbounded under LAN mDNS spoofing.
  Requires L2 adjacency. Post-tag.
- **INFO** — backup salt moved `OsRng` → `rand::random()`. Cryptographically
  equivalent; no action.

**Two further defects, found by the integration suites in #162, fixed in #165:**

- `transport/manager.rs` — `std::cmp::max(0, 100 - latency_ms as u64)`. The
  subtraction evaluates first and u64 has nothing below zero, so any link over
  100 ms panicked in debug and **wrapped to near `u64::MAX` in release, inverting
  transport selection so the worst path scored highest.** Every cellular/WAN link
  is over 100 ms, and D4 is a cross-network test.
- `transport/internet.rs` — timestamps are whole seconds, so a relay in the same
  second as registration gave `conn_duration == 0` and the `if conn_duration > 0`
  guard skipped the bandwidth limit entirely.

Both were **pre-existing** (present on `main`, untouched by #139).

---

## 5. D2 and D4 — what is actually required

### D2 — signing is unblocked, but not proven

All four secrets are set (verified by name only; no agent has ever seen a value):
`SCMESSENGER_KEYSTORE_BASE64`, `_KEYSTORE_PASSWORD`, `_KEY_ALIAS`, `_KEY_PASSWORD`.

**Secrets existing is not proof signing works.** The base64 went through a
PowerShell pipe, and `release.yml`'s signed steps are conditional on
`HAS_KEYSTORE` — a malformed secret still yields a green job and a **debug-signed
APK**. Merge **#154** before tagging; it adds `apksigner verify --print-certs` and
fails the job on an unsigned or debug-signed artifact. If tagging first, check by
hand — `CN=Android Debug` means it did not sign.

The keystore lives at `%USERPROFILE%\kiee\` on the operator's machine. Never in
the repo, never read/copied/printed by any agent, permanently operator-only.

### D4 — Pixel 6a ↔ the AWS node

Operator decision: D4 runs **Android ↔ the AWS node**. There are no "relays" in
this architecture — every node relays — so this is node-to-node and
**cross-platform**, which is *stronger* evidence than two Android handsets.

Verified live 2026-08-15 via the EC2 API as `user/scmessenger-relay-orchestrator`:

```
i-006b14491d421bd0d  running  t3.micro  us-east-1  tag scm-always-on-node
curl http://54.226.67.101:9876/health   -> {"status":"healthy"}  (256 ms)
```

- **The node is Amazon Linux 2023, NOT Ubuntu.** `ssh ubuntu@` gives
  `Permission denied`; **`ssh ec2-user@` works**. At least 8 repo docs still say
  `ubuntu@` and every one of them fails.
- **`HANDOFF/gpt/AWS_RELAY_CURRENT_ADDRESS.md` is the canonical address pointer
  and it is correct.** The IP is dynamic — the account holds zero Elastic IPs and
  `ec2:AllocateAddress` is an **explicit deny** in `SCMessengerRelayFreeTierOnly`.
  Do not try to route around that; it is a deliberate cost guardrail, and the
  product does not need a stable address (v0.4.0 removed hardcoded bootstrap
  addresses; discovery is invite/QR ledger seeding).
- **The node runs code from a closed branch** — `/version` reports `9f54b107` on
  `gpt/pr139-receipt-filter-20260811` (PR #147, closed). It must be rebuilt.
- **NEVER build on the t3.micro.** A previous attempt ran 16 hours and OOMed.
  Pull the CI-prebuilt image.
- **Ordering is forced:** `docker-publish.yml` only fires on push to `main`,
  publishing `sha-<7char>`. So **#139 → main → CI publishes the image → rebuild
  node → run D4.** There is no way to prove D4 before the trunk merge.
- Runbook: `HANDOFF/D4_NODE_REBUILD_RUNBOOK.md` (merged, #159). Identity baseline
  to prove a rebuild did not orphan the ledger:
  `libp2p_peer_id 12D3KooWKMUXfjvWeodBUJbSwBuRXBU3d6XSbP1AJXL9WhaS3yKy`.

Scoring is unchanged: **receiver-side decrypt + durable history + receipt.** Not
transport ACKs, not UI counters, not BLE local acceptance.

---

## 6. Tooling and infrastructure

| Script | Purpose |
|---|---|
| `scripts/pr_scope.sh` | executable "unless there's a reason not to?"; **fails closed** |
| `scripts/triage_lane.sh` | first moves on a red lane — history before hypothesis |
| `scripts/agy_run.sh` | dispatch with per-step progress + stall detection |
| `scripts/reap_worktrees.sh` | reap abandoned worktrees; refuses DIRTY ones |
| `scripts/clean_target.sh` | scoped artifact reclamation; never calls `cargo clean` |
| `scripts/apply_branch_protection.sh` | branch protection, dry-run verified |

**`pr_scope.sh` failed open on 2026-08-15 and has been repaired (#158).** It read
its file list from `gh pr view --json files`, which caps at **100 files**. #139
changes 253. The first 100 held none of the merge-blocked paths, so it printed
`[OK] clear of core/src/{crypto,transport,routing,privacy}` while six gated files
were invisible — on the largest PR in the repo, on the exact check it exists for.
It now derives from `git diff --name-only origin/<base>...origin/<head>`,
announces loudly if it ever falls back to the API, and fails closed when the API
returns exactly 100 files. **Any PR reporting exactly 100 changed files should be
assumed truncated.**

**Docker Integration Suite is non-blocking (#156)** via `continue-on-error: true`
on the single failing step — narrowly scoped, so other Docker breakage still
fails the job. It therefore reports green while that step is broken. **D1 is to be
evaluated with this lane explicitly excluded**, and it must NOT be listed as a
required check in branch protection. Issue #155 tracks the real fix.

**Shared cargo cache (#163).** Every dispatched worker used to build its own
`target/`; one reached 16 GB and filled a 237 GB disk to 99%, at which point rustc
failed with `no space on device` and the compile gate could not run at all. Use:

```
export CARGO_TARGET_DIR=C:/Users/SCM/Documents/GitHub/.scm-shared-target
export CARGO_INCREMENTAL=0
```

Concurrent builds then block on the cargo lock — which enforces the existing
"never two build tools at once" rule rather than fighting it. Documented in
`docs/rules/BUILD_AND_CI.md`. Disk was 4.2 GB free at worst, 34 GB after cleanup.

---

## 7. OPEN — do not guess

1. **Was `ebf5411b`'s deletion of 7 Android sources intentional?** Restored on
   #149 on the CTO's read that APK sharing is active work. If it was a deliberate
   strip-down, revert the restore. Note the Josh fork independently deleted the
   *tests* for two of them, preserved on `josh-fork/local-worktree-state-2026-08-15`.
2. **Josh single-transport build** — operator ruled it is NOT the v0.4.0 default;
   ships as **v0.3.9** if at all. The transport quarantine described in an earlier
   session summary is **not implemented**; `d0e3258a` is 4 files, +23/-5.
3. **README framing** — the CEO was asked to bless the honest-first tone before
   the tag. No reply as of handoff.
4. **Dependency debt** — 7 vulnerabilities on the default branch, 3 high. Deferred
   to post-tag S4, which is right for shipping but should not stay deferred long
   on a security product.

---

## 8. Standing lessons

**Open the file.** Repeatedly this project has classified an artifact without
reading it and been wrong: `GEMINI.md` was already correct; two "duplicate pairs"
were prefix collisions; #147 looked like 102 files of unique work and had zero
unique commits; the repo already had a maintained canonical node-address file
that ~99 documents ignored. **The repo is consistently more coherent than its
directory listing suggests.** `AGENTS.md` rules 13 and 14 exist because of this.

**Verify the mechanism before quantifying.** "AWS node is down" came from a
5-second curl cap against an address that had simply changed. The node was
healthy in 256 ms the whole time.

**Commit before you clear.** A disk cleanup nearly discarded four untracked
integration-test files. Committing them to a branch first is what surfaced two
shipping transport defects on their very first compile. Two other worktrees held
work that existed on **no** remote: `wip-w1-ledger` (20 commits + 73 uncommitted
lines, unpushed entirely) and the JoshFork clone's 5 working-tree changes. Both
are now on origin. **Survey for unpushed work before reclaiming anything.**

**Committing a file as-is is not the same as it passing CI.** Preserving four
untracked files faithfully introduced 9 lines of trailing whitespace and turned
`Repository Hygiene` red on the trunk merge. Run the repo's own checks against
anything you commit, even when you are only preserving someone else's content.

**Dispatch budgets: 90 minutes, not 45.** Three "capability failures" on this
project were too-short timeouts. A task ending in `cargo test --workspace` needs
90m; the relay-ladder fix died at 36 minutes mid-compile with the work complete
but unpushed. Its worktree survived, so nothing was lost — check the worktree
before re-dispatching.

**One CTO seat.** Two sessions ran concurrently on 2026-08-15 and both were
editing this file and capable of merging. They did not collide, but only by luck.
If you find evidence of another active session, establish who holds the seat
before merging anything.

**Destructive history:** `git checkout <ref> -- .` destroyed four files of another
session's uncommitted work. `cargo clean --target <triple>` wiped 44.7 GB. The
preflight hook now blocks both and prints the working form — if it fires, read
it; it is there because someone already paid for that lesson.

---

## 9. Reconciliation of the two CTO lanes — 2026-08-16

### The finding that mattered most

**A repo gate run from a stale checkout silently runs the stale gate, and stale
gates fail in the safe-looking direction.**

The shared checkout sat 23 commits behind `tracking`, on a side branch that had
already been merged in. `scripts/pr_scope.sh` there was therefore the pre-#158
version. §2 of this very document instructs `bash scripts/pr_scope.sh 139`. Run
that way, on the largest merge in the repo, on the exact check it exists for:

```
# stale shared checkout                     # current checkout
[OK] clear of core/src/{crypto,...}         [BLOCKER] touches merge-blocked directories:
                                              core/src/crypto/backup.rs
-> 3 blockers                                 core/src/transport/{addr_filter,behaviour,
                                              dial_policy,observation,swarm}.rs
                                            -> 5 blockers
```

Same PR, same minute. #158 had repaired the script; the repair simply was not
present where the runbook told people to run it.

**Resolved.** The shared checkout is now on `tracking` and current. **Run gates
from a checkout you have confirmed is current, or from a fresh worktree.**
#168 adds a preflight tripwire that blocks a gate whose script differs from the
canonical version — note it compares the *script blob*, not commit distance,
because a branch can be 25 commits behind and still hold the correct gate
(`scm-lane-b-pr-scope` is exactly that case).

### The two new D1 blockers

1. **`Repository Hygiene Checks` — 1852 flagged lines, 8 files.** The cause is
   not CRLF as such. `.gitattributes` declared `text eol=lf` for `*.rs`,
   `*.swift`, `*.sh`, `*.py` and Dockerfiles but **not `*.kt`, `*.kts`, `*.md`**.
   `git diff --check` honours those attributes, so Kotlin and Markdown were the
   only file types whose CR was seen as trailing whitespace. 35 `.rs` files in
   the delta also store CRLF and pass CI today purely by attribute. #169 adds
   the three missing declarations. **The blobs were NOT rewritten and did not
   need to be** — a commit message on that branch says otherwise and is wrong;
   the PR body records the correction.
2. **`Rust Linting` — `cargo fmt --all -- --check`** fails on
   `core/tests/integration_wan_swarm_node2.rs` (4 hunks). #162 landed it
   unformatted. Also fixed in #169.

Neither appeared in any earlier handoff. Both were found by running the CI
commands locally instead of waiting 45 minutes to be told.

### Lane A's closeout — every divergence checked

Three of its four were already resolved when written; the fourth was
mischaracterized:

- **Two competing handoff docs** — already unified. One `CTO_STATE.md`,
  byte-identical to tracking. #166 settled it.
- **`pr_scope.sh` conflict** — resolved in lane B's favour (#158); lane A
  retracted its own version.
- **Four untracked tests "contradict what shipped"** — they do not. They are
  **byte-identical to pushed commit `7c717e52`** and superseded by `4ac6c1e1`
  (#162), which fixed them to compile. They reference `TransportType::TCP` and
  `::QUIC`, which **do not exist** (real variants: BLE, WiFiAware, WiFiDirect,
  Internet, Local). Obsolete pre-fix drafts. Archived to
  `archive/five-layer-original-drafts-20260816`, then removed so the shared
  checkout could be brought current.
- **Backlog amnesty** — filler queue only, not critical path.

### On delegation, from three dispatches that all ran free

All three went to `agy-gemini` / `gemini-3.7-flash-high`, ~$0, 50s-6min each.
**Every one was re-verified by hand, and two had defects that only re-running
found:**

- **A worker's report claimed work it had not done** — "renormalized stored
  blobs to LF". `git cat-file blob` shows the CRLF still there. The fix is
  correct by a different mechanism; the *description* was false. Read the
  artifact, not the report.
- **A spec defect of mine shipped as a cry-wolf guard.** I specified the
  stale-gate trigger as "HEAD is behind base". Measuring real worktrees showed
  that blocks `scm-lane-b-pr-scope` — 25 commits behind, holding the *correct*
  script. Re-briefed to blob comparison. **When a guard fires on innocent
  input, people learn to set the override, and the one time it is right it gets
  waved through too.** That is why #167 matters.
- **I was wrong about the fix itself.** I told the worker "adding the attribute
  alone does not work," from an experiment that edited `.gitattributes` in the
  *working tree* while diffing two *commits* that lacked it. `git diff A...B`
  resolves attributes from the committed tree. **Test the mechanism the way CI
  will run it.**

Lane routing: **`scripts/lanes.json` is the authority, re-probe it.** Qwen and
DashScope are both dead (401). Only `agy-gemini` (free) and `agy-claude`
(metered) have a shell, so anything running `gh`/`cargo`/`gradlew`/`adb` goes
there. `gpt-oss-120b-medium` sits in agy's *Claude* pool despite the name.

### Housekeeping

Worktrees created for this session and safe to reap once #139 lands:
`scm-cto-gate`, `scm-orch-harden`, `scm-ws-fix`. Use
`scripts/reap_worktrees.sh` — it refuses dirty ones. Disk was at 91% (23 GB
free) at the time of writing.
