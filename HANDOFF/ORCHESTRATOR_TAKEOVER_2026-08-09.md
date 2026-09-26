# Orchestrator Takeover Packet -- 2026-08-09 (Windows/Claude lane standing down)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Active
Last updated: 2026-08-09
Successor: next `/orchestrate` session -- READ THIS FIRST
Supersedes: `HANDOFF/ORCHESTRATOR_TAKEOVER_2026-08-08.md` (that packet's PR #139
BLOCK section is now historical -- the block was lifted, see section 2)

---

## 0. THE ONE THING TO DO FIRST

**PR #139 is one CI matrix away from the five-node gate.** Nothing is blocked on
analysis any more. The remaining work is mechanical and sequenced:

1. Wait for the full CI matrix to pass on the current head.
2. Dispatch `release.yml` with **`artifacts_only=true`** from
   `tracking/pre-v040-tag-work`.
3. Dispatch `docker-publish.yml` from the same ref.
4. Record artifact SHA-256s and the **image digest**.
5. Rebuild the AWS node from that **digest** (see section 5 -- do NOT use
   `:latest`).
6. Coordinate the synchronized window with GPT-MAC (nodes 4/5) and the operator.

The operator has authorized the node rebuild and teardown. GPT-MAC (Codex, MAC
LANE) is driving the PR and owns iOS + macOS.

---

## 1. Terminology (operator correction, 2026-08-09)

It is a **node**, not a "relay". All nodes relay; the AWS box is the always-on
**node**. Older docs say "relay" throughout -- do not propagate it in new work.

---

## 2. PR #139 -- the rule 8 BLOCK has been lifted

The 2026-08-08 adversarial review returned BLOCK with F1 (CRITICAL) + F2-F5
(HIGH). **All five merge-blocking conditions are closed**, plus every should-fix
item except one LOW.

| Finding | Status |
|---|---|
| F1 requester never checked | Closed -- `is_disclosable_on_rfc1918_network` now takes the observed requester address, mandatory, fails closed on `None` |
| F2 class- not subnet-granular | Closed -- /24 IPv4, /64 ULA |
| F3 loopback/link-local via contact chain | Closed -- contact-chaining branch deleted entirely |
| F4 unproven addresses disclosable | Closed -- predicate reverted to `success_count > 0`, **now pinned by a regression test** |
| F5 libp2p peer id gate dead / fails open | Closed -- both swarm gates `unwrap_or(true)`; `resolved_identifiers` uses explicit prefix tagging |
| F6, F7 (~50% double-hash) | Closed by the prefix-tagging change |
| F8 reject TOCTOU blocked nobody | Closed -- falls back to blocking the canonical id |
| F9 dual-flavor side effects | Closed -- applied across both flavors |
| F10 one bad row bricks all unblocks | Closed -- `continue` instead of `?` |
| F11 writes under a read lock | Closed -- all `register_device_id` sites take `.write()` |
| F12 non-atomic dual write | Closed -- canonical row written first, fails closed |
| F13 fail-open default | Closed |
| F14 string-split multiaddr parsing | Closed -- real `Multiaddr` parsing |
| F15 hardcoded `"unknown"` reputation bucket | Closed -- `process_incoming_from(sender_peer_id: Option<&str>)` |

Additional issues found and fixed during this session:

- **R1** -- the requester was never checked for `/p2p-circuit`. Real on the
  outbound `ShareLedger` path (a dialed circuit retains the relay's `/ip4`);
  **not** on the response path (an inbound circuit is a bare `/p2p/<id>` with no
  IP, so it already failed closed). Fixed.
- **R2** -- `cmd_send_offline` passed `core_handle: None`, and separately was
  dropping the swarm event receiver on the floor. Both fixed.
- **P0 BLE L2CAP accept-spin** -- see section 4.
- **Cellular bootstrap** -- periodic/legacy bootstrap used an empty hardcoded
  candidate list instead of ledger relay addresses. Fixed by GPT-MAC. This is
  the likely cause of "cellular-only: Android did not route through any node".

### Two clean bills from the original review are void

The review certified "Crypto module untouched" and "no `Cargo.toml`/`Cargo.lock`
delta". **Both are now false**, by later commits. Each was re-reviewed in the PR
thread and a post-hoc review record was entered by GPT-MAC. Specifically:

- Crypto: a salt-generation change in `core/src/crypto/backup.rs`. Reviewed --
  no vulnerability. The Argon2id known-answer value was regenerated, which reset
  its anchor; **I re-anchored it independently against OpenSSL 3.5.7's ARGON2ID
  and it matched exactly** (`0d98e705...`). Reproduction commands are in the PR
  thread.
- Dependencies: tokio-tungstenite 0.21 -> 0.24, removing `rustls 0.22` and
  `rustls-webpki 0.102.8`. Verified: **zero packages added**, crate count
  646 -> 644. A strict reduction.

### Still open (operator-deferred, NOT gating)

- `HANDOFF/todo/P1_DISCLOSURE_CGNAT_AND_FAILOPEN_2026-08-09.md` -- D1 (CGNAT /24
  collision) and D2 (`concrete_local_ips.is_empty()` fail-open). Operator
  decision: deploy the first five-node test now, fix these **before the second**.
- F12 was closed, but the review's LOW items are otherwise done.

---

## 3. A rule 8 re-review is still owed, and I should not be the one to do it

GPT-MAC requested a fresh full adversarial review against the exact candidate. I
did substantial verification (documented in the PR thread) but **I am not a valid
rule 8 signatory for it**: I proposed the R1 fix shape and wrote the F4
regression test that landed, so I would be reviewing my own contributions.

`docs/FUSION_LITE.md` is also explicit that multi-model panels are **not** a
substitute for this gate on crypto/transport/routing/privacy code.

The sign-off should come from `crypto-security-auditor` (note: its definition
pins a deepseek model this account cannot reach -- **pass an explicit `model`
override**) or the read-only Qwen thinking dispatch per `docs/ORCHESTRATION.md`
Section 4.

---

## 4. P0 BLE L2CAP accept-spin -- fixed, trigger still unknown

`HANDOFF/todo/P0_BLE_L2CAP_ACCEPT_SPIN_2026-08-08.md` (on `main`, kept open
deliberately).

Found by pulling the Pixel: `BleL2capManager.startListening` retried
`BluetoothServerSocket.accept()` with no backoff, no cap, no socket re-creation.
**220,504 iterations across two bursts totalling 5m44s, up to ~1,037/sec,
producing 75.4% of a 3.8M-line logcat capture.**

Consequences: Android could not accept inbound L2CAP (this is the "BLE-only
intermittent, especially iOS -> Android" symptom -- that direction is where
Android must `accept()`), and the storm **evicted the entire 13:35-13:55
field-test window out of a 16 MiB buffer**. That is a strong candidate answer to
the previously UNRESOLVED question of why `MdnsServiceDiscovery` and
`SubnetProbe` logged zero lines on 2026-08-08 -- most likely evicted, not silent.

Fixed by GPT-MAC: socket closed and recreated, bounded exponential backoff,
one-line diagnostics, generation-safe stop/cancel, listener stays armed
indefinitely with a 30 s cap.

**The trigger is still unidentified.** Established: the socket is *born broken* --
the first `accept()` failure lands in the same second as the successful
`listenUsingInsecureL2capChannel()`, which reports success and allocates a PSM
either way. Ruled out: Wi-Fi off, Bluetooth stack death, screen wake, PSM value,
and app restart (2 of 4 processes never spun). Keep the ticket open.

---

## 5. AWS node -- access, state, and the trap that will bite you

- IAM user `scmessenger-relay-orchestrator`, account `101533648751`, us-east-1.
- **`aws` CLI is NOT on PATH.** It is installed via pip at
  `~/AppData/Roaming/Python/Python314/Scripts/aws.cmd`. Use the full path.
- Inventory verified 2026-08-09: **exactly one instance**,
  `i-006b14491d421bd0d` / `scm-always-on-node` / `54.226.67.101` / t3.micro.
  The operator requires this stays at exactly one.
- Operator has authorized **teardown and fresh rebuild** as needed.

### The `:latest` trap -- read before rebuilding

`HANDOFF/audit/AWS_RELAY_REBUILD_2026-08-04.md:156` documents
`docker pull testbotz/scmessenger:latest`. **That is wrong for a candidate
build.** `docker-publish.yml` tags `latest` with `enable={{is_default_branch}}`,
so a dispatch from `tracking/pre-v040-tag-work` publishes a branch tag and a sha
tag and does **not** move `latest`. Pulling `:latest` gets whatever `main` last
built, and the node comes up healthy reporting the wrong commit -- the same
silent-wrong-provenance failure as the stale image it replaces.

**Pull by digest.** Then verify `GET /version` reports the gated commit as a
hard gate, not a formality. That endpoint is new in this PR precisely so node
provenance is provable rather than asserted.

### Security posture (verified, and one correction)

I initially believed the node's HTTP API was publicly exposed. **It is not.**
`scm-node-sg`: tcp/9001 and udp/9001 open to `0.0.0.0/0` (correct, p2p), tcp/22
and **tcp/9876 restricted to `147.81.41.188/32`** -- which is this workstation.
My probes succeeded because this machine is allowlisted. Posture is correct.

**But a real finding stands and a firewall cannot fix it:**
`cli/src/api.rs:1201-1204` sets `CorsLayer::allow_origin(Any)` with
`allow_headers(Any)` on an API with no authentication serving `/api/contacts`,
`/api/history`, `/api/peers`, `/api/send`. On user devices the CLI binds
`127.0.0.1:9876`, so **any website the user visits can read their contacts and
history and send messages as them** from the browser. Pre-existing, not from this
PR, so it did not block the merge -- but it needs its own ticket.

---

## 6. Five-node gate readiness

Criteria are in `HANDOFF/plans/V040_V050_FIVE_NODE_GATE_PLAN_2026-08-05.md`
(G1 pairwise, G2 transport coverage, G3 delivery truth, G4 fleet convergence,
G5 liveness, G6 provenance). Run it **twice**.

**Scoring rule set by GPT-MAC, adopt it:** do NOT score from transport ACKs, UI
peer counts, or BLE local acceptance. Score on receiver decrypt + durable history
+ receipt + sender Delivered + outbox removal. (Note: the old "dial reports
success on queue" defect is **fixed** -- `PendingDialEntry` now holds the reply
until a real `ConnectionEstablished`/`OutgoingConnectionError`, 10 s timeout. Do
not repeat that claim.)

Lane split: Windows lane holds Windows CLI + Pixel 6a + AWS node. GPT-MAC owns
iOS + macOS **runtime** (CI covers Apple *builds*, so compile parity is checkable
without their hardware). The **operator is required** for the BLE-only leg
(Wi-Fi off) and the G5 network-disruption leg.

Pixel: on wireless ADB, logcat buffers at 16 MiB. **That resets on reboot** --
re-apply `adb shell logcat -G 16M` before any run. Two mDNS transports are
advertised for the one device, so pin `-t <id>`.

---

## 7. Delegation lanes -- measured results, 2026-08-09

Do not re-derive this; it cost real tokens and money to establish.

- **agy (gemini-3.1-pro-high)**: dispatched an evidence sweep over the PR diff.
  **Every item I verified was a false positive** -- the `unsafe` hit was the word
  "unsafe" inside an assertion *message*; "production panic paths" were inside a
  `mod tests` block; a bare function signature was reported as a panic. Its
  enclosing-function attribution was actively wrong. **Do not use agy for precise
  code-evidence work you can grep yourself.** It remains good for bulky work:
  adb/uiautomator poking, greps over 500 MB logs, single long build commands.
- **FusionLite**: two runs, $0.0153, **zero content both times**. Root cause was a
  missing `reasoning` parameter -- now **fixed** (see section 8).
- **deepseek-v4-flash-0731 direct via OpenRouter**: $0.006 total, produced **two
  genuine findings** (D1/D2 above) that survived my verification. Best
  value-per-cent of the three. Key at `~/.config/scmorc/openrouter_direct.env`,
  variable name is **`OpenRouter_Paid_Key`** (not `OPENROUTER_API_KEY`).
  **Set `reasoning: {"effort": "low"}`** or it returns empty content.

---

## 8. Tooling fixes landed this session (all on `main`)

- `fix(fusion-lite)` `035dca61` -- adds `--reasoning-effort` (default low) and a
  `message.reasoning` fallback. The same qwen panel that returned 0 characters
  now returns ~5,900 each.
- `docs(observability)` -- `scripts/run5.sh` had `scmessenger_core::mesh::delivery`
  in `OSX_RUST_LOG`. **There is no `mesh` module.** `EnvFilter` silently matches
  zero spans for an unknown target, so **delivery tracing was never enabled on any
  prior five-node run.** Corrected, and `tracing_init.rs` now gives debug builds a
  richer default since `RUST_LOG` cannot be set on mobile. Both are on the PR
  branch too.

---

## 9. Environment traps that produce *plausible emptiness*

Three separate silent-failure modes hit this session. They all look like
"no results" rather than an error:

1. **`git show <rev>:<dotted/path>` under Git Bash** -- MSYS rewrites the colon
   and git errors; with stderr redirected it reads as an empty file. Use
   `MSYS_NO_PATHCONV=1`. This nearly produced a wrong conclusion about
   `.github/workflows/release.yml`.
2. **Piping a gate through `tail`** -- `$?` reports `tail`'s status, so the gate
   cannot fail. Already documented in `CLAUDE.md`; I still walked into it. Re-run
   unpiped to get a real exit code.
3. **`tasklist /FO CSV /NH` under Git Bash** -- mangles `/FO` into a path, prints
   nothing, reads as "process not running". Use plain `tasklist | grep -i`.

Same family: the `:latest` docker tag (section 5) and FusionLite's empty results
(section 7). When a check returns nothing, verify it *ran* before concluding the
thing is absent.

---

## 10. Open decisions for the operator

1. Rule 8 re-review signatory (section 3) -- who, and does it gate the tag?
2. Whether the wildcard-CORS finding (section 5) needs a ticket now or after
   v0.4.0.
3. Timing of the synchronized five-node window -- needs the operator plus GPT-MAC
   simultaneously.
