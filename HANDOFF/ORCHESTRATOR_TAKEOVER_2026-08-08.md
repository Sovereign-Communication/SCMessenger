# Orchestrator Takeover Packet -- 2026-08-08

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: Active
Last updated: 2026-08-08 (Windows orchestrator lane, standing down)
Successor: next `/orchestrate` session -- READ THIS FIRST

The previous session stood down deliberately for fresh context. Four subagents
were still running at stand-down; their outputs are file-backed (paths below).
Nothing is lost, but you must collect them.

---

## 0. CRITICAL STATUS + THE FIRST QUESTION TO ASK

### 0a. PR #139 is BLOCKED -- do NOT merge it

The rule 8 adversarial review completed and returned **BLOCK** with one
CRITICAL finding. Full report:
`docs/security/PR139_ADVERSARIAL_REVIEW_2026-08-08.md`.

**F1 (CRITICAL) -- the RFC1918 disclosure gate never checks the requester.**
`is_disclosable_on_rfc1918_network` compares the ledger *entry's* address class
against *our own* listener addresses, not the peer's. Because the node binds
`0.0.0.0` and libp2p expands that per-interface, our own LAN IP is always in
`my_addrs`, so the check returns true for **every** requester. Any unauthenticated
internet peer that completes a Noise handshake and sends one ledger-exchange
request receives our internal subnet map: LAN host:port pairs plus each
neighbour's libp2p PeerId. That is internal-network reconnaissance plus a
deanonymization primitive, and a comment three lines above the new code
describes this exact bug as previously fixed.

Four more findings compound it: F2 (class-granular not subnet-granular
matching), F3 (loopback and `169.254.x` disclosed to any peer we once dialed),
F4 (unproven remote-supplied addresses redistributed -- an SSRF/port-scan
amplifier), F5 (T1's libp2p-peer-id claim is not implemented and the transport
block gate fails open).

Five conditions must be met to lift the block -- see the verdict section of the
report. Four regression tests are also required; **the current suite passes
while the vulnerability is live**, which the reviewer flagged as its own
finding.

The review also confirmed real wins worth preserving: authenticated-key ingress
canonicalization, complete fail-closed ingress error handling, the `block_peer`
argument-order fix, and the lifecycle mutex. T3 fail-closed and T4 canonical-key
claims both verified [OK]. Crypto modules untouched; no `unsafe`; no dependency
delta.

Note: CI being 15/15 green does NOT contradict this. The tests do not model a
remote requester at all.

### 0b. ASK THE OPERATOR THIS FIRST

**Blocking question: which fix direction for the P0 UPnP panic?**

The desktop node panics and dies ~5m20s after start
(`libp2p-upnp-0.5.0` `behaviour.rs:497`, "mapping should exist"). This is the
top v0.4.0 release blocker. The fix lives in `core/src/transport/`, which is
**merge-blocked by AGENTS.md rule 8** -- it requires BOTH an operator decision
on direction AND an adversarial review before any code change. Do not
implement, do not "just try (a)", do not let a worker start on it until the
operator has chosen.

Present these options and wait for an answer:

- **(a) Gate or remove the libp2p `upnp` feature.** Smallest change. UPnP
  contributes nothing to LAN-only testing and is the sole source of the panic.
  Previous lane's recommendation for v0.4.0.
- **(b) Upgrade `libp2p-upnp` past 0.5.0** if a fix exists upstream.
- **(c) Isolate the UPnP behaviour** so its panic cannot kill the swarm event
  loop. Previous lane's recommendation as the durable fix, later.

Full evidence: `HANDOFF/todo/P0_UPNP_PANIC_KILLS_DESKTOP_NODE_2026-08-08.md`.

The other operator decisions (Section 5) can follow, but this one gates the
release and should be asked in your first message.

---

## 1. FIRST ACTIONS (after asking Section 0)

1. **Collect the in-flight subagent output** -- Section 4. Some files may not
   exist yet; if missing, the agent did not finish. Re-dispatch from the brief
   noted in that section rather than guessing at the result.
2. **Read the P0** -- `HANDOFF/todo/P0_UPNP_PANIC_KILLS_DESKTOP_NODE_2026-08-08.md`.
3. **Get the operator's remaining decisions** in Section 5. Several items are
   blocked on judgment, not on work.
4. **Delegate** `HANDOFF/todo/P1_PRUNE_CLAUDE_DATA_2026-08-08.md` (Claude data
   >10 GB). It is self-contained and safe to hand to a worker.

---

## 2. What landed on `main` this session

| Commit | Contents |
|---|---|
| `df90b23b` | Agent-context tiering: `CLAUDE.md` Tier 0, `docs/rules/` Tier 1, `.claude/rules/*` reduced to stubs, `preflight_guard.py` + tests |
| `d39ee14a` | Cross-harness safety: `.githooks/pre-push` (blocks non-fast-forward push / branch deletion for EVERY tool), destructive-op guard, `AGENTS.md` rules 11-12 |

Always-on agent context went 11,863 -> 4,016 bytes (-66%). `CLAUDE.md` is capped
at 3.5 KB; if you add to it, move something to `docs/rules/` in the same change.

**Both hooks are live** (`core.hooksPath` = `.githooks`). Expect `cargo clean`,
`git reset --hard`, `git checkout -- <paths>`, force-push and recursive
force-deletes outside `tmp/`/`target/` to be BLOCKED. Overrides are operator
decisions: `SCM_ALLOW_DESTRUCTIVE=1`, `SCM_ALLOW_FORCE_PUSH=1` (honoured both
from the environment and as an inline `VAR=1 cmd` prefix).

---

## 3. PR #139 state

- Branch `tracking/pre-v040-tag-work`, head **`6cb7033a`**, state OPEN, MERGEABLE.
- **CI fully GREEN: 15/15 checks passing.**
- The identifier-gate batch T1-T4 is implemented and tested (per the prior
  lane's report): dual-flavor block storage, Ed25519 curve-point gating,
  fail-closed pending requests, unified `message_request_key`.
- **The rule 8 adversarial review is COMPLETE and returned BLOCK.** See Section
  0a and `docs/security/PR139_ADVERSARIAL_REVIEW_2026-08-08.md`. The PR must not
  merge until the five conditions in that report's verdict are met and the four
  required regression tests exist.
- The prior lane's report ("ready for T1 review sign-off and merge decision")
  is now superseded: the review ran, and the answer was no.

`main` is 2+ commits ahead of the PR branch (the two commits above). The PR
branch does NOT yet contain the tiering/safety work. Decide whether to merge
main into the branch or leave the branch alone until #139 merges.

### Uncommitted in the working tree (deliberately)

These exist on disk and are NOT committed. Do not discard them.

- `HANDOFF/gpt/GPT_MAC_PR139_TAKEOVER_2026-08-07.md` -- **another session's
  work**, recovered from discarded commit `fbb9757d`. Not ours to commit.
- `HANDOFF/gpt/GPT_MAC_IOS_LOG_PULL_REQUEST_2026-08-08.md` -- new, this session.
- `HANDOFF/todo/P0_UPNP_PANIC_KILLS_DESKTOP_NODE_2026-08-08.md` -- new.
- `docs/fieldtest/ANDROID_LOG_OBSERVATIONS_2026-08-08_agy.md` -- new.
- This file, and `HANDOFF/todo/P1_PRUNE_CLAUDE_DATA_2026-08-08.md`.
- The tiering/safety files show as modified vs the branch because they live on
  `main`, not on this branch. That is expected -- do not "restore" them.

---

## 4. In-flight subagents at stand-down

All four were told to write their results into the repo. Collect these:

| Work | Model | Expected output file | If missing |
|---|---|---|---|
| **PR #139 adversarial review (rule 8 merge gate)** | Opus | `docs/security/PR139_ADVERSARIAL_REVIEW_2026-08-08.md` | **COMPLETE -- verdict BLOCK.** See Section 0a. (The reviewer is read-only by design and could not write the file itself; the orchestrator transcribed it verbatim.) |
| Android log observation catalog (native pass) | Sonnet | `docs/fieldtest/ANDROID_LOG_OBSERVATIONS_2026-08-08_native.md` | An independent agy pass already landed at `..._agy.md` -- that one is COMPLETE and usable on its own |
| Debug log-level verification | Sonnet | `docs/LOGGING_LEVELS_AUDIT_2026-08-08.md` | Re-dispatch; brief summarized in Section 7 |
| Selective `target/` cleanup | Sonnet | `docs/fieldtest/TARGET_CLEANUP_2026-08-08.md` | **COMPLETE** -- 6.4 GB -> 32 GB free, node survived, `generated-sources/` intact |

**Do not read the raw subagent JSONL transcripts** under
`AppData\Local\Temp\claude\...\tasks\*.output` -- they will overflow context.
Use the files above.

**The `crypto-security-auditor` subagent is misconfigured**: its definition
pins a deepseek model this account cannot access, so it dies immediately with
an API error. Pass an explicit `model` override (opus/sonnet) when using it,
or fix `.claude/agents/crypto-security-auditor.md`.

---

## 5. Open decisions -- operator buyoff required

1. **UPnP panic fix direction** (P0). Options: (a) gate/remove the libp2p
   `upnp` feature -- smallest, UPnP is useless for LAN testing and is the sole
   panic source; (b) upgrade `libp2p-upnp` past 0.5.0; (c) isolate the
   behaviour so its panic cannot kill the swarm loop. Previous lane recommended
   (a) for v0.4.0 with (c) as the durable fix. **`core/src/transport/` changes
   are merge-blocked pending rule 8 review -- do not implement without both.**
2. **PR #139 merge** once the review verdict lands.
3. Whether to commit the handoff/fieldtest docs listed in Section 3 to the PR
   branch, to `main`, or leave local.

---

## 6. Field-test findings (2026-08-08 13:35-13:55 HST, Android <-> iOS)

The exchange **succeeded**. Confirmed working: Android<->iOS messaging over
both BLE and TCP; ledger sharing; relay circuit reservations; transport
continuing after Bluetooth was switched off. BLE-only mode was NOT tested.

Defects and anomalies found:

1. **P0 -- desktop node panics ~5m20s after start.** `libp2p-upnp-0.5.0`
   `behaviour.rs:497` "mapping should exist", killing the swarm event loop; the
   CLI then correctly self-terminates rather than lingering as a zombie. This
   is the leading explanation for "macOS/Windows nodes were never detected" --
   a node that dies 5 minutes in is absent for most of a test window. Full
   ticket: `HANDOFF/todo/P0_UPNP_PANIC_KILLS_DESKTOP_NODE_2026-08-08.md`.
2. **mDNS is broken on Windows**: `libp2p_mdns ... failed reading datagram ...
   (os error 10040)` (WSAEMSGSIZE). Discovery falls back to promiscuous ledger
   dialing.
3. **Promiscuous dialing accepts any PeerID**: `Connected to <peer> ...
   (promiscuous mode -- any PeerID accepted)`. Deserves security review on its
   own; it is NOT in the current rule 8 review scope.
4. **Dead bootstrap**: the desktop node's only bootstrap is
   `/ip4/127.0.0.1/tcp/19001`, which nothing serves.
5. **Android bootstrap failing continuously**: `Bootstrap all-failed
   (consecutive=28)`, 60s retry loop, still failing at stand-down.
6. **Slow convergence**: the Windows node took ~5 minutes to reach the Android
   peer -- arriving almost exactly when it panicked.
7. Android app logging is extremely sparse: 151 app-owned lines out of 43,608
   captured. `MdnsServiceDiscovery` and `SubnetProbe` emitted ZERO lines
   despite being wired and having all permissions granted. Whether they never
   ran or merely logged below the captured level is UNRESOLVED and is the
   central question for the log-level audit.
8. **Lead worth chasing first: the Android Bluetooth stack itself died.**
   `com.google.android.bluetooth` went down at `13:41:18` and did not return
   until `13:41:59` (new pid) -- entirely inside the window the first app
   session was foregrounded, and squarely in the "Android stopped receiving"
   period. Two competing explanations remain open (routine cached-process
   reclaim vs. a Bluetooth stack crash) and the log cannot distinguish them.
   This is the single most promising thread for hiccup 1.
9. The `13:43:28` process death is now fully explained as an ordinary
   Recents/task-switcher swipe-away (`startRecentsTransition` ->
   `finishInner: toHome=true reason=requested` -> `Killing ... remove task`).
   No crash, ANR, or OOM anywhere near it. It was the operator closing the app,
   AFTER the receiving problem had already started.
10. **The macOS/Windows CLI nodes have ZERO footprint in the entire capture** --
    no peer id, no IP, no NsdManager event. Only one remote peer
    (`192.168.0.142`) appears anywhere. This independently corroborates the
    operator's account, and combined with the P0 UPnP panic supports "the
    desktop nodes were not present" over "discovery failed to find them".
11. Hiccup 2 (Bluetooth switched off, messaging continued) has **no log
    evidence at all** -- no `BluetoothAdapter` OFF transition appears before the
    capture ends at `14:02:29`. That event is outside the captured range.

Two independent audits of the same logs were produced deliberately
(`..._agy.md` and `..._native.md`); the second reviewer never read the first.
Cross-read them -- they emphasise different things.

### Correction carried forward (do not repeat this mistake)

`tasklist /FO CSV /NH` **silently returns nothing under Git Bash** -- it mangles
`/FO` into `C:/Program Files/Git/FO`, tasklist errors, and you get a false
"process not running". This produced two wrong conclusions this session. Use
plain `tasklist | grep -i <name>`.

---

## 7. Current runtime state at stand-down

- **Windows node RUNNING** (restarted after the panic, to confirm
  reproducibility). Log: `tmp/logs/win_node_run2.log`. Expect it to panic again
  ~5 minutes in -- check and record that in the P0 ticket.
- **Pixel 6a connected over wireless ADB** at `192.168.0.141:44389`. The mDNS
  port rotates; rediscover with `adb mdns services` if it drops.
- **Logcat buffers raised to 16 MiB** on the Pixel (were 256 KiB, which is why
  the test-window app logs were evicted). This persists until device reboot --
  re-apply `adb shell logcat -G 16M` before the next field test.
- **Disk: ~32 GB free on C: (87%)** -- the cleanup agent reclaimed ~26 GB from
  `target/` (4 Android triples + `debug/deps` + `debug/build`). Above the 25 GB
  gate threshold, so **build gates are UNBLOCKED**. Note the Android
  cross-compile triples were removed, so the next Android build is a cold one
  and will take significantly longer. `core/target/android-libs/` (1.9 GB) was
  left alone pending a decision on whether it is regenerable.
- LAN topology: Windows `192.168.0.121`, Android `192.168.0.141`,
  iOS `192.168.0.142`, all on `192.168.0.0/24`.
- Captured logs live in `tmp/logs/` (gitignored, do not commit, do not delete
  -- analysis is ongoing).

---

## 8. Outstanding requests to other lanes

- `HANDOFF/gpt/GPT_MAC_IOS_LOG_PULL_REQUEST_2026-08-08.md` -- iOS unified logs
  for the test window, plus the macOS node's state during it, plus a 4-node
  rerun with all nodes verified up first and a BLE-only leg. Not yet sent to
  the MAC lane; sending it is a takeover action.
