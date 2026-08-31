# SCMessenger Ship Plan -- v0.4.0 Public Alpha

Status: Active
Created: 2026-08-14
Owner: Operator (Treystu)
Supersedes for execution purposes: `HANDOFF/todo/_QUEUE.md` (see Amnesty, S0-4)

This is the **only** execution queue until v0.4.0 is tagged and downloadable.
If a task is not on this page, it is not being worked on.

---

## 0. North star

> Two people who have never met, on two phones, with no shared network,
> exchange a message and both see a delivery receipt -- using a build a
> stranger downloaded from the GitHub releases page.

**Definition of done for v0.4.0:**

| # | Exit criterion | Evidence required |
|---|---|---|
| D1 | `main` is green | All CI lanes pass on a push to `main`, run URL recorded |
| D2 | Signed APK is downloadable | `gh release view v0.4.0-alpha.1` lists an APK asset |
| D3 | README explains the product and how to install | File is non-empty, links resolve |
| D4 | Two-device message + receipt | Receiver-side decrypt + durable history + receipt, per `project_fleet_run_scoring_evidence` -- NOT transport ACKs |
| D5 | No long-lived integration branch | PR #139 merged or closed; `main` is trunk |
| D6 | Transport racing demonstrated | Message delivered when first-choice transport is unavailable, proving fallback selects a working path. Receiver-side decrypt + durable history + receipt -- NOT transport ACKs, NOT UI counters, NOT BLE local acceptance |
| D7 | Offline proximity messaging demonstrated | Two devices exchanging a message with no internet available. Receiver-side decrypt + durable history + receipt -- NOT transport ACKs, NOT UI counters, NOT BLE local acceptance |

Anything that does not move D1-D7 is deferred. No exceptions until tag.

---

## 1. Credit discipline (read this before dispatching anything)

The plan is designed so that **Claude native tokens are spent only on verdicts**.
Roughly 80% of the work below is mechanical and belongs on free lanes.

| Lane | Cost | Use for | Do NOT use for |
|---|---|---|---|
| **Qwen Claude Code CLI** (`launch_claude.ps1`) | Free | PRIMARY. Scoped diffs, CI log triage, README drafting, doc archiving | Unscoped "analyze the codebase" tasks -- it rewrites code |
| **agy** (`--add-dir`, pinned `--model`) | Free | adb/UI poking, log greps, single build commands | Multi-step reasoning; needs resume not restart on timeout |
| **Fusion Lite** | ~2c/run, 10c hard cap | Pre-commit diff review, plan sanity checks | Implementation |
| **DashScope / OpenRouter / Groq** | Free | Overflow when Qwen quota is dry; Groq micro-validation only | Large-file full-mode edits (silent truncation) |
| **Claude native (this session)** | Expensive | Go/no-go verdicts, adversarial crypto review, merge decisions | Log greps, doc moves, formatting, ticket triage |

**Rules that protect the budget:**

1. Dispatch from a scratch cwd with `--add-dir` so `CLAUDE.md` and `docs/rules/`
   are not pre-loaded into every worker; inject only the rules that task needs.
2. One task file -> one provider. Parallel dispatch of the same `--task` file
   collides on the tmp output path and one result is silently lost.
3. Use `--mode diff` for anything under ~500 lines. Full-file mode truncates
   silently on large files.
4. Verify delegated *verification* claims. A worker reporting "gate passed" is a
   claim, not evidence -- require the run URL or the command output.
5. Never re-dispatch reactively. Authorize a batch, let it run, check once.

**Expected native spend for this whole plan: 5-8 verdict checkpoints.** Each one
is a short read of evidence plus a go/no-go. That is the budget.

---

## 2. Sequenced workstreams

Sprints are strictly ordered. S1 gates S2 gates S3. Do not parallelize across
sprints -- a red `main` makes every downstream result unverifiable.

### S0 -- Clear the decks (half a day, mostly free lanes)

| ID | Task | Lane | Done when |
|---|---|---|---|
| S0-1 | Commit or stash the current working-tree changes on `tracking/pre-v040-tag-work`. Shared checkout -- do not touch files you did not create. | Operator | `git status` shows only intentional work |
| S0-2 | Triage the 16 open PRs into MERGE / CLOSE / DEFER. The 13 dependabot PRs are one batch decision, not 13. | Qwen | A 16-line table with a verdict per PR |
| S0-3 | Merge or close PR #139. This is a decision, not a task -- if it cannot merge this week, close it and cherry-pick what matters. | **Native verdict** | D5 satisfied |
| S0-4 | Backlog amnesty: `git mv HANDOFF/todo/* HANDOFF/archive/` except items that map to D1-D7. Keep `_QUEUE.md`. | agy | `HANDOFF/todo` holds <= 10 files |
| S0-5 | Untrack root junk: `screen.png`, `window_dump.xml`, `local.properties`, stray `adb_logcat*.txt`. `local.properties` holds local SDK paths and should never have been tracked. | Qwen | `git ls-files` root listing is clean |

> S0-5 note: `.gitignore` already covers `*.pem` and `*apiKey*.csv`, so the
> untracked key and CSV in the working tree are ignored, not leaked. Confirmed
> 2026-08-14. Do not commit them.

### S1 -- Green main (the gate everything else depends on)

Four lanes are failing as of run `31659699771` (2026-08-13). Fix in this order;
each is independently mergeable.

| ID | Lane failing | What we know | Assigned lane |
|---|---|---|---|
| S1-1 | **Mobile** | Root cause identified: `Release signing is not configured; release tasks must fail.` Needs a signing config wired from `android/keystore.properties.template` + repo secrets. This is also a hard blocker for D2. | Qwen (config) + Operator (secrets) |
| S1-2 | **Repository Hygiene** | Previously fixed once by `7f369f50` (trailing whitespace) and regressed. Fix the check to be enforceable pre-push, not just in CI. | agy |
| S1-3 | **Lint** (Rust clippy/fmt) | Exact error not yet isolated -- the `--log-failed` output is dominated by toolchain-setup noise. First task is to extract the real error lines. | Qwen |
| S1-4 | **Docker Integration Suite** | Long-standing amber lane. If it cannot be fixed in one pass, mark it non-blocking and say so explicitly in the workflow -- do not leave a permanently red required check. | Qwen, then **native verdict** on blocking status |

**Local pre-push guard (do this once, saves CI cycles):**

```bash
cargo fmt --check; if [ $? -ne 0 ]; then echo "[FAIL] fmt"; fi
```

Never read `$?` after a pipe -- a piped gate can never fail.

**S1 exit:** one push to `main` where every lane is green. Record the run URL.
This is **native verdict checkpoint 1**, and it satisfies D1.

### S2 -- Make it downloadable

| ID | Task | Lane | Done when |
|---|---|---|---|
| S2-1 | Write `README.md`. It is currently 0 bytes. Use the existing repo description as the opening line; sections: what it is, threat model in three sentences, install (Android APK, CLI), build from source, project status honesty note. | Qwen drafts, **native edits** | File is non-empty and accurate |
| S2-2 | Wire release signing (depends on S1-1) and produce a signed APK from a tagged commit with `SCM_GIT_HASH` embedded -- `816422fc` already exports it. | Operator + agy | APK installs on the Pixel 6a |
| S2-3 | Tag `v0.4.0-alpha.1` and publish a release with the APK attached and real release notes drawn from `CHANGELOG.md`. Latest public release is v0.1.9 from March -- close that five-month gap. | Operator | D2 + D3 satisfied |
| S2-4 | Set the repo homepage URL to the install guide. Enable Discussions as the inbound channel. | Operator | Repo metadata updated |

**S2 exit: native verdict checkpoint 2** -- read the README as a stranger would
and confirm the download path works end to end.

### S3 -- Prove the north star

| ID | Task | Lane | Evidence |
|---|---|---|---|
| S3-1 | Rebuild all nodes to the tagged SHA. Per `HANDOFF/PR139_FIVE_NODE_GATE_STATUS_2026-08-13.md`, Windows CLI and AWS were on stale SHAs and macOS/iOS were offline -- that gate has never actually run clean. | agy + Operator | Every node reports the tag's git hash |
| S3-2 | Run the two-device test on the **released APK**, not a dev build. Cross-network: one on cellular, one on WiFi. | Operator + agy | Receiver decrypt + durable history + receipt |
| S3-3 | If it fails, the failure becomes the only ticket. Do not open a workstream -- fix and re-run. | Qwen impl | Re-run passes |
| S3-4 | Transport racing gate: message delivered when first-choice transport is unavailable, proving fallback selects a working path. | Operator + agy | Receiver-side decrypt + durable history + receipt (NOT transport ACKs, UI counters, or BLE local acceptance) |
| S3-5 | Offline proximity gate: two devices exchange a message with no internet available. | Operator + agy | Receiver-side decrypt + durable history + receipt (NOT transport ACKs, UI counters, or BLE local acceptance) |

**S3 exit: native verdict checkpoint 3** -- score the run on receiver-side
evidence only. Transport ACKs, UI counters, and BLE local acceptance do not
count. This satisfies D4, D6, and D7.

### S4 -- After the tag (do not start before it)

- **External crypto audit.** Hybrid X25519 + ML-KEM-768 is the differentiator and
  the liability. Self-review by the fleet that wrote it is not a credential.
  Budget real money here, not tokens.
- **Android last mile.** 162 unwired functions, 84 in `MeshRepository.kt`. Burn
  down only what D4 exercises; the rest is speculative surface.
- **Dependency debt.** Six months of unpatched deps on a security product.

---

## 3. Governance changes (permanent, start now)

1. **Red main is a stop-the-line event.** No feature work while a required lane
   is failing. A red main makes every other result unverifiable.
2. **Trunk-based.** Branches live under 48 hours. No more long-lived tracking
   branches -- that is how #139 became a second main.
3. **Agents are measured in commits merged to green main**, not handoff documents
   produced. The repo currently holds 1,695 markdown files / ~223k words against
   ~120k lines of Rust. Stop writing to each other.
4. **One doc per fact.** `docs/CURRENT_STATE.md` is the state; this file is the
   plan. New handoff docs require a reason that is not "context transfer".
5. **Concurrent-lane cap.** Give each agent its own `git worktree`. The shared
   checkout has already destroyed uncommitted work once.

---

## 4. Explicitly not doing (until after tag)

- v0.5.0 / v1.0.0 planning, PQC-14 close-out, farm drills, KMP/meeting mode
- iOS parity work (`iOS_V040_PARITY_IMPLEMENTATION_PLAN.md`) -- Android ships first
- The remaining 78 unwired non-Android functions
- Any new orchestration tooling, dashboard, or visualizer

Each of these is defensible on its own. Together they are why nothing has
reached a user since March.

---

## 5. Checkpoint ledger

Fill this in as the plan executes. Empty cells are the honest status.

| Checkpoint | Criterion | Date | Evidence |
|---|---|---|---|
| CP1 | D1 -- main green | 2026-08-23 | `main`@`b538f3ba`: every push-triggered workflow (CI, Lint, Repository Hygiene, Docker Publish, Docker Integration Suite, Cross, iOS Build & Test, Mobile) reports `conclusion: success`, verified via the GitHub Actions API. Two P0 fixes (#221, #222) remain open and DRAFT, blocking the tag, not `main`'s own greenness -- see `HANDOFF/CTO_STATE.md` 2026-08-23 checkpoint sections. |
| CP2 | D2 + D3 -- release published | | |
| CP3 | D4 -- two-device proof | | |
| CP4 | D5 -- #139 resolved | | |
| CP5 | D6 -- transport racing proof | | |
| CP6 | D7 -- offline proximity proof | | |

---

# 6. Endgame amendment -- 2026-08-31 CEO audit

Closes task L4-3 of `HANDOFF/API_RESET_EXECUTION_CHARTER_2026-08-28.md`.
Every line below was obtained by running a command on `main`@`69a8ba57`.
Sections 0-5 above stay as written; where they conflict with this section,
this section wins.

## 6.1 Corrected D1-D7 scoreboard

| # | Criterion | State | The one thing in the way |
|---|---|---|---|
| D1 | main green | **[OK]** | `69a8ba57`: CI, Lint, Cross, Mobile, Repository Hygiene, Docker Publish, CodeQL all `success`. Docker Integration Suite + iOS still running -- normal duration, not hung. |
| D2 | Signed APK downloadable | **[BLOCKED -- operator, ~2 min]** | `SCMESSENGER_KEY_ALIAS` does not match the keystore. Last release run `32817839477` (2026-08-25) failed at `packageRelease`. No release run has been attempted since. Preflight (#238) now fails this in seconds instead of 24 minutes. |
| D3 | README | **[OK]** | 4,309 bytes, accurate. SHIP_PLAN S2-1 ("0 bytes") is stale -- disregard it. |
| D4 | Two-device message + receipt | **[PARTIAL]** | Proven on the dev rig (Windows/Pixel/AWS live round-trips, msg `62cd3a30` clean post-#251). NOT scored on a released APK cross-network, because no release exists. Gated by D2. |
| D5 | No long-lived integration branch | **[OK]** | #139/#230/#231 merged. 212 stale refs is hygiene, not D5. |
| D6 | Transport racing | **[BLOCKED -- code]** | See 6.2. Still unprovable by construction. PR #239 did not close this. |
| D7 | Offline proximity | **[NOT STARTED]** | Sequenced after D2. BLE fragment path repaired (#250, #251); untested as a gate. |
| -- | Cloud-node parity (operator ruling 2026-08-29) | **[PARTIAL]** | Node 3 repaired and persistent as of 2026-08-31 (6.3a), but it still cannot rejoin unaided after an address change (6.3b). |

## 6.2 D6 has no call site

`IronCore::routing_peer_seen` (`core/src/iron_core.rs:2704`) has **zero callers**
repo-wide -- verified across `.rs`, `.kt`, `.swift`, excluding generated UniFFI
bindings and tests. PR #239 corrected the function's *body* (it now feeds
`OptimizedRoutingEngine::peer_seen`) but never added a caller, so nothing
changed at runtime and the charter's D6 finding still stands.

The only production feed into the routing engine is
`core/src/transport/swarm.rs:3863` (`record_message_activity` on a delivered
message). The `SwarmEvent::ConnectionEstablished` handler
(`swarm.rs:5277`) does **not** touch the routing engine at all.

**Fix location is exact and cheap.** That same handler already does
platform-neutral ledger convergence at `swarm.rs:5486`. Add the routing feed
in the same block: derive the transport type from the endpoint multiaddr and
call `peer_seen`. Everything needed (`parse_transport_type`, `parse_peer_id_32`)
landed with #239.

Consequence if skipped: adaptive-routing confidence stays 0.0 fleet-wide, and
any D6 claim of "fallback selected a working path" is unprovable regardless of
what the demo shows.

## 6.3 Claims in this repo that are now false -- do not act on them

- **`V040_COMPLETION_PLAN.md` section 0: "v0.4.0 requires ZERO feature LoC
  beyond rc.1 ... budget ~150-300 LoC."** Falsified by execution: 13 defect-fix
  commits landed 2026-08-29..31 totalling ~2,812 insertions, and the gates are
  still not scored. The remaining work is defect-driven, not configuration.
- **`HANDOFF/todo/V040_LEDGER_SEEDING_AND_GOSSIP.md` gap A ("nothing on mobile
  ever initiates").** Closed in code: `swarm.rs:5486` auto-initiates ledger
  exchange on `ConnectionEstablished` with per-peer dedupe, platform-neutral.
  Gap 5 (hardcoded node addresses) also finds no hits. Move the ticket to
  `HANDOFF/done/` with this citation.
- **`HANDOFF/todo/P0_DUAL_BIND_TCP_AND_WS_ON_SAME_PORT_2026-08-10.md`.** Closed
  in code per `CTO_STATE` 2026-08-29 (`multiport.rs:75-99`); live listeners
  confirm 9001 tcp-only / 9002 ws-only. It needs no operator ruling. Charter
  L0-4 and L5-1 are moot -- retire both.
- **SHIP_PLAN S2-1 (README 0 bytes), CP1 (#221/#222 blocking), S1-4 (Docker
  suite amber).** All stale; see 6.1 and the charter's Part 0.

## 6.3a Node 3 -- recovered 2026-08-31, and what it exposed

The cloud node was found dead to the mesh and has been repaired in this session.
Evidence is live, not quoted.

**What was wrong.** The instance had been replaced, so `54.226.67.101` -- the
address written into 35 files in this repo -- was gone. The live node was
`54.235.20.24`, running `35360758` (stale by 5 PRs), with `peers: []` and
`connection_path_state: Bootstrapping` for 13 hours. `docker inspect` reported
`Mounts: []`: the container had been started **without** the
`-v /opt/scm-relay-data:/data` bind mount, so PR #240's identity-persistence fix
was silently discarded and `/data` was the ephemeral container layer.
`/opt/scm-relay-data` did not exist on the box. Root cause: **two deploy paths.**
`scripts/aws_deploy.sh` carried the mount; the `launch.py` instance userdata did
not, and the instance had been created from the latter.

**What was done.** Identity rescued out of the container layer into
`/opt/scm-relay-data`, poisoned peer store backed up and cleared, node redeployed
at `main`@`69a8ba57` with the mount. Identity persistence then verified across a
full container restart: `identity_id 640a5dc8...`, `libp2p 12D3KooW9uRM...`,
public key `014b8105...` -- byte-identical before and after. That closes charter
task L0-1 with real evidence for the first time.

`scripts/aws_deploy.sh` is now the single supported deploy path: it discovers the
public IP from the EC2 API instead of hardcoding it, rescues a container-local
identity without overwriting a persisted one, and **fails the deploy if the mount
is absent**. The regression cannot repeat silently.

**IAM constraints, dry-run verified.** The `scmessenger-relay-orchestrator`
credentials have `run_instances` DENIED and `allocate_address` DENIED;
`terminate_instances` is ALLOWED. We cannot launch a replacement instance or
allocate an Elastic IP from this seat.

**Host OS note.** The node is Amazon Linux 2023 and always has been
(`ec2-user@` in `aws_deploy.sh`). The `ubuntu@` references in this repo belong to
the archived farm-sim workstream and a different key. The node runs in Docker, so
the host distribution does not affect mesh behaviour.

## 6.3b The operator ruling that reshapes the plan

> It should ledger share and re-join mesh automatically. If it moves, it should
> tell Windows and Android and then they can both ledger share the new IP with
> each other. This is how the mesh works -- automatic, not manual. Accept that
> every re-deploy is a new IP. That's how many nodes will be in the wild -- new
> IPs all the time, we need to set it up to work with this.
> -- Operator, 2026-08-31

Address churn is a design constraint, not an incident. An Elastic IP is therefore
explicitly **not** the fix, and manual re-seeding is not an acceptable workaround.
Three defects stand between the code and that model, all found live in this
session:

1. **The node never dials out on boot, and its seed list is empty anyway.**
   `connect_to_seed_peers()` (`swarm.rs:2528`) is called only from
   `mobile_bridge.rs:862`; the CLI node has no caller. Demonstrated bilaterally
   2026-08-31: the Windows node and the AWS node were brought up at the identical
   SHA `69a8ba57` and left alone -- both sat at `Bootstrapping` / `peers: []`
   indefinitely. Neither dials.

   Worse, wiring the dial alone would be a no-op. **Two peer stores exist and do
   not converge:** the gossiped core ledger (`storage/ledger.json`, which seeds
   `ConnectToSeedPeers`) held **0 entries on Windows and 1 on AWS**, while the
   CLI-local `peers.json` held **4,678 on Windows and 107 on AWS** and is never
   shared. The core ledger has exactly one production writer, `swarm.rs:5397`,
   correctly guarded by `endpoint.is_dialer()` -- so it only learns from
   connections we initiate, and we never initiate any. The loop is closed: no
   outbound dial means no ledger, and no ledger means nothing to dial.
   -> `V040_T1_NODE_BOOT_SEED_DIAL.md` (both halves)
2. **The duplication itself -- two peer stores that never converge.** The CLI
   store is uncapped and polluted (4,678 entries on Windows: thousands of
   ephemeral-port rows for one peer, RFC1918 and Docker-bridge addresses, a stale
   former self identity, placeholder junk, and the AWS node's own address under
   two different peer ids). The core store is capped, canonicalized, gossiped --
   and empty. Each half holds what the other needs: the CLI already implements
   `record_identified_peer` (listen-address recording) and
   `reap_stale_addresses_for_peer` (address supersession), wired to the store
   nobody shares. Operator ruling 2026-08-31: cherry-pick and converge onto the
   core store, with a disclosure rule so only locally verified entries are
   exported. -> `V040_T2_UNIFY_PEER_LEDGER_STORES.md`

The transport for automatic rejoin already exists: ledger gossip fires on
`ConnectionEstablished` (`swarm.rs:5486`, merged, platform-neutral). T1 supplies
the outbound dial; T2 makes the shared store the only store, so what gets gossiped
is real and what gets dialled is not junk. With both, a moved node rejoins with no
human action and the third node learns its address second-hand without ever
contacting it directly.

## 6.4 Sprint to tag -- ordered, and this is the whole list

Test topology for v0.4.0: **Android (Pixel 6a) + Windows CLI + AWS node**.

Dispatch model, per operator 2026-08-31: task files are written here and run
through **Freebuff desktop on DeepSeek V4 Flash** (unmetered). The freebuff CLI
v0.0.161 has no headless mode -- it exposes only `login`, `--continue`, `--cwd`
and ignores piped stdin -- so it cannot be driven from an agent session.

### N -- Node 3 (DONE, this session)

| ID | Task | State |
|---|---|---|
| N-1 | Redeploy at current main with persistent `/data` mount | **[OK]** `69a8ba57`, mount verified |
| N-2 | Prove identity survives a restart | **[OK]** `640a5dc8...` identical across restart |
| N-3 | Purge the poisoned peer store | **[OK]** backed up on the box, cleared |
| N-4 | Single deploy path that cannot silently drop the mount | **[OK]** `scripts/aws_deploy.sh` rewritten, IP-discovering, fails without the mount |
| N-5 | Correct the dead `54.226.67.101` in canonical docs | Open -- see G5 |

### G1 -- Unblock the release (operator, ~15 minutes, blocks four gates)

**Regenerate the keystore rather than hunt for the old one.** Verified
2026-08-31: **no Android APK has ever been published from this repo** -- every
public release (`v0.1.0`, `v0.1.1`, `v0.1.9`, `v0.2.1`) carries CLI binaries
only. There is no signing lineage to preserve, so a fresh key costs nothing
today and is locked in forever the moment a signed APK reaches a user. The old
`scmessenger-release.jks` is not on this machine (only `~/.android/debug.keystore`
is), so its alias cannot be verified without finding the file. Archaeology on an
unverifiable keystore is more expensive than a new one.

The test fleet does not argue against this: the Pixel runs a *debug*-signed
build, and D4/D6/D7 must be scored on the *released* APK, so an
uninstall-and-reinstall on every test device is already required. Regenerating
adds zero incremental cost **provided it happens before the fleet migrates.**

| ID | Task | Owner |
|---|---|---|
| G1-1 | Generate a fresh release keystore per `docs/ANDROID_RELEASE_SIGNING.md` (now corrected to `-storetype PKCS12`; the old JKS guidance is what produced the case-sensitivity mismatch). Back it up in two places **before** anything else | Operator -- never an agent |
| G1-2 | Verify locally before touching secrets: `scripts/verify_release_keystore.sh <keystore> <alias>`. Runs the exact check `release.yml` runs, prints the cert fingerprints, never prints the password or alias | Operator |
| G1-3 | Set the four secrets, then dress rehearsal with no tag burned: `gh workflow run release.yml -f artifacts_only=true`. `build-android` has no tag requirement, so this is a full signed build at zero risk | Operator sets, agent runs |
| G1-4 | Record the cert SHA-256 in the password manager and in `HANDOFF/CTO_STATE.md` (fingerprint only -- never the password). It is how a future build is proven to come from this key | Operator |

**Nothing downstream of G1 can be scored.** D4/D6/D7 all require a released APK
by this plan's own evidence standard, and no GitHub release exists for any
v0.4.0 tag.

### G2 -- Automatic rejoin (Freebuff / DeepSeek V4 Flash)

Order: T1 first, then T2 (or alongside). T4 and T5 are independent and may run
in parallel.

Task files live in `HANDOFF/freebuff/queue/`; the lane's rules are
`docs/rules/FREEBUFF.md`.

| ID | Task file | LoC | Review gate |
|---|---|---|---|
| G2-1 | `V040_T1_NODE_BOOT_SEED_DIAL.md` (two halves: seed-list bootstrap + boot dial) | 150-280 | none if confined to `cli/` |
| G2-2 | `V040_T2_UNIFY_PEER_LEDGER_STORES.md` -- **supersedes the former T2 and T3**, which were both symptom fixes for the same duplication | 400-700 | **Rule-8 mandatory** -- changes what the node discloses |
| G2-3 | `V040_T4_ROUTING_FEED_ON_CONNECTION_ESTABLISHED.md` (D6) | 60-120 | **Rule-8 mandatory** |
| G2-4 | `V040_T5_DOCS_SYNC_GATE_IS_RED.md` | 5-30 | none |

### G3 -- Score the gates (operator + hardware, after G1-2 and G2)

| ID | Gate | Evidence standard |
|---|---|---|
| G3-0 | **Churn gate.** Redeploy the AWS node so it takes a new public IP, change nothing on Windows or Android, and confirm all three re-mesh unaided | Windows and Android both reach the node at its new address with zero manual re-seeding. This is the operator's stated model, tested directly |
| G3-1 | D4: two devices, released APK, cross-network (one cellular, one WiFi) | Receiver decrypt + durable history + receipt. NOT transport ACKs, UI counters, or BLE local acceptance |
| G3-2 | D6: first-choice transport unavailable, fallback delivers | Same, plus non-zero routing confidence from T4 |
| G3-3 | D7: two devices, no internet | Same |
| G3-4 | Cloud-node parity: store-and-forward custody + connection assistance. Re-run `ANDROID_RELAY_INBOUND_EVIDENCE_2026-08-10_CELLULAR.md` as the regression case | Receiver-side custody delivery scored from the swarm audit log. `/api/diagnostics.custody_audit_count` now reads live post-#236 and post-mount -- but confirm it is moving before trusting it |
| G3-5 | Any failure becomes the single ticket. Fix, re-run. No new workstream | Re-run passes |

### G4 -- Tag and publish (operator)

| ID | Task |
|---|---|
| G4-1 | Promote to final `v0.4.0`, not another `rc`. `release.yml` marks any tag containing `rc`/`alpha`/`beta` as a **draft**, and a draft is not a public download. `verify_versions.sh` passes for a final `v0.4.0` |
| G4-2 | Publish gate: external crypto audit **COMMISSIONED** (firm, scope, price, dates). Standing board ruling. Commissioning is the gate, not completion |
| G4-3 | Fill the section 5 ledger from G3 evidence; delete `API_RESET_EXECUTION_CHARTER_2026-08-28.md`; retire `V040_COMPLETION_PLAN.md` |

### G5 -- Backlog truth (free lanes, blocks nothing)

Correct the dead `54.226.67.101` in `HANDOFF/gpt/AWS_RELAY_CURRENT_ADDRESS.md`,
`HANDOFF/CTO_STATE.md` and `HANDOFF/D4_NODE_REBUILD_RUNBOOK.md` -- and state the
address is discovered, never hardcoded, so the next replacement does not restart
this hunt.

Move to `HANDOFF/done/` with an evidence citation each: `V040_LEDGER_SEEDING_AND_GOSSIP`
(gap A closed at `swarm.rs:5486`), `P0_DUAL_BIND_TCP_AND_WS_ON_SAME_PORT` (closed
at `multiport.rs:75-99`), `P0_ANDROID_FINITE_RETRY_ABANDONMENT` (#245),
`D4_MOBILE_BRIDGE_HISTORY_FLAVOR_MATCHING` (#248), `RECEIPT_MARKER_ID_FLAVOR_MISMATCH`,
`P0_ANDROID_SELF_RATCHET_RESET`, `P0_DEEPLINK_PARSES_BUT_NEVER_DIALS`, both
`P1_ASYNC_DELIVERY_RECEIPTS_DO_NOT_CONVERGE*`, `RCA_DELIVERY_ACK_IMPLEMENTATION_PLAN`.
Archive the 11 `INBOX_2026-08-11*` files. Target: `HANDOFF/todo` under 10 files.

PR queue: 27 open. Merge #214/#212/#211/#141 (workflow-only, zero build surface);
close #223/#224/#225/#205/#206 (superseded checkpoint docs), #156 (Docker suite
passes -- premise dead) and #215 (superseded by #239 + T4); rebase #227/#209.
Leave #213/#210 deferred on the Kotlin/AGP toolchain floor.

## 6.5 Remaining LoC to the tag

Grounded in this project's measured defect-fix rate: 13 commits, 2026-08-29..31,
mean ~215 insertions, median ~190, roughly half of every diff being test code.

| Item | LoC | Confidence |
|---|---|---|
| T1 node boot seed dial **+ seed-list bootstrap** | 150-280 | Medium-high -- grew once the empty-ledger finding landed; two halves, both required |
| T2 unify the two peer stores (cherry-pick, migrate, disclosure rule) | 400-700 | Medium -- larger than the two symptom tickets it replaces, but removes the class of bug rather than two instances. Much of the logic is ported, not written: `record_identified_peer` and `reap_stale_addresses_for_peer` already exist in `cli/src/ledger.rs` |
| T4 routing feed on `ConnectionEstablished` (D6) | 60-120 | High -- exact call site, helpers already landed |
| T5 docs-sync repair | 5-30 | High |
| N-lane node recovery | 0 remaining | **Done this session** |
| G1, G4 (secrets, tag, publish) | 0 | Certain -- configuration and human decisions |
| G5 backlog + PR queue | 0 code | Certain |
| **G3 demo contingency** -- churn gate, D4 on a released APK, D6, D7, parity | **700-1,400** | **Medium** -- 3-7 defect PRs at the measured rate |
| **Total** | **~1,300-2,530** | Point estimate **~1,850** |

The contingency is lower than the 2026-08-31 morning estimate (900-1,700) for one
reason: three of the defects that demo day would have discovered have already been
found and specified as T1-T3, moving them out of the unknown column. It is still
the largest row and still the only one that has ever been underestimated here.

**What is genuinely close:** every remaining blocker is one operator command
(G1-1), five specified and located code changes (T1-T5), or evidence-gathering on
hardware. No architecture is missing and no feature is unbuilt.

---

## 6.6 Execution policy change -- 2026-08-31

Operator directive: work never stops because a node is unavailable. Full policy:
`docs/rules/CONTINUOUS_EXECUTION.md`. In short:

- **Tier A** (AWS Linux + Windows) is always available and is driven to **full
  v1.0.0 conformance**, continuously. This supersedes section 4's blanket
  deferral of v1.0.0 scope for Tier A work; the tag still sets priority order,
  but the queue may not run dry waiting on hardware.
- **Tier B** (Android) is coded to parity now and verified later. Device time is
  for verification and log capture, never for writing code.
- **Tier C** (iOS/macOS) stays out of scope until v0.5.0.
- "Blocked on hardware" is not a terminal state. It is a signal to descend the
  never-idle ladder in that document.

## 7. Discovered issue ledger

Every defect found while doing something else, with a disposition. "Noted in
passing" is not a disposition. Opened 2026-08-31; append, do not rewrite.

| # | Issue | Evidence | Disposition |
|---|---|---|---|
| I-01 | AWS node ran with **no `/data` mount**; identity written to the ephemeral container layer and lost on every redeploy | `docker inspect scm-node --format '{{json .Mounts}}'` returned `[]`; startup log `No keys found in store` | **FIXED** -- PR #259. Identity persistence verified across a full restart |
| I-02 | `scripts/aws_deploy.sh` hardcoded `54.226.67.101`, dead since the instance was replaced | `curl` timeout; EC2 API reports the instance at a different address | **FIXED** -- PR #259, now discovers the IP from the EC2 API |
| I-03 | `.codebuff_deploy/aws/launch.py` userdata omits the `-v /opt/scm-relay-data:/data` mount. **This is what caused I-01**, and it is still live | The running container was created from it, without the mount | **OPEN -- ticketed T6.** Untracked file owned by another session; do not edit it silently. Until fixed, any instance replacement re-breaks identity persistence |
| I-04 | Two peer stores that never converge; the gossiped one is empty, the local one is uncapped and polluted | Windows `ledger.json` 0 entries vs `peers.json` 4,678 | **TICKETED** -- `V040_T2_UNIFY_PEER_LEDGER_STORES.md` |
| I-05 | Ephemeral source ports recorded as dialable addresses | 14 entries for one peer on AWS; thousands on Windows | **TICKETED** -- T2 (defect A) |
| I-06 | Self-entries cause a continuous self-dial storm across the node's own listeners | `Dial error: Unexpected peer ID ... at /ip4/127.0.0.1/tcp/9001/...` on loop | **TICKETED** -- T2 (defect B) |
| I-07 | One identity recorded at five unrelated networks | `12D3KooWD6vZQrUqpyGa` (the Windows node) recorded at residential, cellular, the AWS node's own address, and two IPv6 /64s | **TICKETED** -- T2 (defect C) |
| I-08 | A dead address is never retired; PRs #256/#257 reset its failure counter on any success to the same peer | `54.226.67.101` still present at `fails=8, 9, 16, 60` | **TICKETED** -- T2 (supersession) |
| I-09 | `connect_to_seed_peers()` is never called by the CLI, so the gossiped ledger never seeds any dial | Zero occurrences of `connect_to_seed_peers`/`SEED-DIAL` in the node log | **TICKETED** -- `V040_T1_NODE_BOOT_SEED_DIAL.md` (Half 2) |
| I-10 | `routing_peer_seen` has zero callers; routing confidence pinned at 0.0, so D6 is unprovable | `grep -rn` across `.rs`/`.kt`/`.swift` excluding generated bindings | **TICKETED** -- `V040_T4_ROUTING_FEED_ON_CONNECTION_ESTABLISHED.md` |
| I-11 | `scripts/docs_sync_check.sh` fails on clean `main`, so every agent's finalize gate is red | Broken link to `DiagnosticsBundleFormatterTest.kt`, deleted in `149d3725` | **TICKETED** -- `V040_T5_DOCS_SYNC_GATE_IS_RED.md` |
| I-12 | The AWS node binds **33 listeners** including 80/443/8080/9090 and cross-dials its own, producing sustained negotiation-failure warnings | `/api/diagnostics.listeners`; `WARN High rate of incoming negotiation failures from /ip4/172.31.31.151/tcp/9090 -> /ip4/172.17.0.1/tcp/9001` | **OPEN -- ticketed T6.** Partly a symptom of I-06; the listener surface itself is a separate question |
| I-13 | The driver watcher died at 01:31 with `ERROR: bash not found` and stayed dead **7 hours**; nothing noticed | `watcher.log` last line before restart | **FIXED locally** -- `scratch/driver/watcher.ps1` now resolves bash from known install locations when `-NoProfile` strips PATH. Watcher restarted and logging. Untracked file, so not in PR #259 |
| I-14 | `scratch/driver/watcher_run.cmd` claims persistence via a `SCMessengerDriverWatcher` ONLOGON scheduled task. **That task is not registered.** Persistence is actually a Startup-folder shortcut | `Get-ScheduledTask` -> not found; `SCMessengerDriverWatcher.lnk` present in Startup | **ACCEPTED** -- the Startup shortcut works and survives reboot. The stale comment is misleading but harmless; corrected if that file is ever edited for another reason |
| I-15 | `HANDOFF_AUDIT/REPO_MAP.jsonl` contains stale AI-generated `calls` entries that assert call sites which do not exist in source. It misled an agent into believing `routing_peer_seen` had callers | Reported by the Freebuff lane, 2026-08-31; independently consistent with the zero-caller grep | **OPEN -- ticketed T6.** An artifact that lies about the codebase is worse than no artifact, because agents trust it |
| I-19 | A WS11 unit test was deleted on a **false rationale**: `149d3725` removed `DiagnosticsBundleFormatterTest.kt` as "orphaned (class deleted by iterations)", but `DiagnosticsBundleFormatter.kt` still exists and is consumed by `DiagnosticsScreen.kt`. Coverage of `format()` has been absent since 2026-08-14 | `find android -name DiagnosticsBundleFormatter.kt` finds it; `grep -rln` across `src/test` finds nothing | **TICKETED** -- `V040_T8_RESTORE_DIAGNOSTICS_FORMATTER_TEST.md`. Found by the Freebuff lane while doing T5; the register entry is `UNVERIFIED` until restored (PR #260) |
| I-20 | C: hit **100% full (51 MB free)** during the T2 build; `SCMessenger/target` alone is **41 GB** on a 237 GB disk. Build failures presented as `LNK1318` linker errors and `os error 112`, which read as toolchain faults rather than disk exhaustion | `df -h /c`; `du -sm target` -> 41128 | **ACCEPTED with a standing remedy.** `scripts/clean_target.sh --all` reclaims most of it scoped, keeping built binaries and protecting `core/target/generated-sources/`. Ruled 2026-08-31 in `HANDOFF/freebuff/inbox/RULING_2026-08-31_T2_disk_space.md`; no `rm -rf target`, ever |
| I-16 | 27 open PRs; 212 remote refs, 18 provably merged | `gh pr list`; `git ls-remote` | **PLANNED** -- SHIP_PLAN G5 |
| I-17 | `SCMESSENGER_KEY_ALIAS` does not match the keystore; `ANDROID_RELEASE_SIGNING.md` documented `-storetype JKS` while modern keytool writes PKCS12, which matches aliases case-sensitively | Release run `32817839477` failed at `packageRelease` | **DOC FIXED + PLANNED** -- PR #259 corrects the guidance and adds `scripts/verify_release_keystore.sh`; regeneration is G1, operator-only |
| I-18 | No Android APK has ever been published, so no signing lineage exists -- the keystore can be regenerated for free, but only until the first release | `gh release view` on all four public releases: CLI binaries only | **PLANNED** -- G1. This is a deadline, not a defect |
