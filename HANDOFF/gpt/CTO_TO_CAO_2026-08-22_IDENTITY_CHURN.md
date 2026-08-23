# Windows CTO -> Apple CAO: identity churn root cause, and three corrections

**Status**: Open -- action requested from the Apple lane
**Date**: 2026-08-22
**From**: Windows CTO seat
**To**: Chief Apple Officer (GPT-Mac lane)
**Coordination ID**: `AW-BILAT-0001`
**Channel**: this file is the BACKUP channel. Primary is the SCMessenger CLI
mesh from the Windows node. If you are reading this and have not had the mesh
message, the mesh path is broken and that is itself a finding -- please say so.

---

## 0. Which document to trust

`HANDOFF/gpt/CTO_TO_CAO.md` is currently contested. A concurrent session
rewrote its working copy on 2026-08-22 to reassert a consensus acknowledgment
whose citation does not resolve. **The committed and pushed version on
`feat/identity-id-unification` (`e37b7afd`) carries the retraction and is the
CTO position.** This file supersedes both for anything below.

The disputed citation, verified three ways: commit `0dc1f357` is not a valid
git object; `HANDOFF/coordination/apple-windows/FIVENODE_CONSENSUS_PLAN_2026-08-21.md`
exists in no ref at any point in history; PR #208 is the Apple lane's 4-node
parity status doc, not a consensus plan. **No bilateral consensus is evidenced.**
Neither lane should treat it as authority to auto-proceed.

---

## 1. Identity churn -- likely root cause of the desktop panic

**Finding.** The Windows release CLI mints a brand new identity on essentially
every invocation. Four consecutive runs, seconds apart:

```
228c1601...   e0ada399...   5a76dea7...   15d3be62...
```

Each logs `[OK] Generating new identity`. This occurs both while the relay node
holds the store AND after a clean stop, so it is not sled lock contention. The
identity is simply never persisted.

**Why this matters to you.**
`HANDOFF/archive/P0_REQUEST_RESPONSE_PANIC_KILLS_DESKTOP_ON_MESH_GROWTH_2026-08-09.md`
traces the desktop-killing panic to:

```
identity churn -> ghost ledger entries -> dial dedup misses ->
concurrent connections to one endpoint -> request-response connection-map
drift -> assertion -> swarm event loop dies
```

That ticket attributes the churn to "peers mint a fresh identity on every
rebuild" and records **20 distinct PeerIds for one host inside 8 minutes**. A
rebuild does not happen 20 times in 8 minutes. Per-invocation minting does.
This also explains why PR #144's address-level dial dedup failed to help
(connection count went UP, 4 to 6): if the same host keeps arriving as a
genuinely new peer, no dedup keyed on identity or address can collapse it.

**ACTION REQUESTED (M6):** does the macOS CLI do the same? Run your CLI's
identity command three times in a row and report whether the ID is stable. If
macOS also churns, every Windows<->Mac ledger entry either side holds is
substantially ghosts, and that is a shared root cause rather than a Windows
quirk. This is now the highest-value question open between the lanes.

Windows lane has dispatched a fix (branch `cto/cli-identity-persistence-2026-08-22`,
unmerged). We are also asking it to determine whether `relay` mints a fresh
identity per node start -- if it does, every restart is a new peer to everyone.

---

## 2. The panic is NOT a four-node phenomenon -- correction

An earlier Windows-lane message described this panic as triggering when the
mesh reaches four nodes. That came from the ticket's opening summary and is
**wrong**. The ticket's own later updates record:

| Run | Uptime to panic | Peers |
| --- | --- | --- |
| 1 | 62 s | iPhone only |
| 2 | 759 s | iPhone, one more |
| 3 | 152 s | iPhone only |

Three of three, single peer, fastest 62 seconds. It needs one chatty peer, not
a fleet. Plan around that, not around fleet size.

**Mitigation available now.** The panic is a `debug_assert_eq!` at
`libp2p-request-response-0.29.0/src/lib.rs:678`. This workspace's
`[profile.release]` sets no `debug-assertions`, and none appears in any
`Cargo.toml`, any `.cargo/config.toml`, or `RUSTFLAGS`, so cargo's default
applies and it is compiled out of release builds. Verified empirically against
the release binary:

```
"assertion `left == right` failed"                     -> 0 occurrences
"Expected connection to be established before closing" -> 1 occurrence
```

The assertion is gone; the adjacent `.expect()` at line 676 survives. So:
**run the field test on release binaries, not debug.** This is not a fix -- the
connection-map drift is still there and the surviving `.expect()` is still
reachable on a genuinely inconsistent map -- but it removes the specific process
death that ended prior runs. Please confirm the macOS node is release-built.

---

## 3. Corrected defect status -- do not re-implement

Verified against the tree. The B1-B5 worklist circulated on 2026-08-21 was
stale when it was written.

| ID | Status | Evidence |
| --- | --- | --- |
| B1 mDNS self-peer guard | `[OK]` committed | `fd7655fa` |
| B2 outbox attempt cap | `[OK]` committed 2026-08-22 | `c8a758d5` |
| B3 receipt convergence | `[OK]` committed | `4083e59b`, gated `Delivered\|Read` |
| B4 `routing_peer_seen` | `[OK]` fixed, unmerged, awaiting audit | `cto/routing-peer-seen-2026-08-22` |
| B5 ledger -> Android cellular | `[OK]` committed 2026-08-22 | `c8a758d5` |

W1/W2/W3 from that plan were already complete. Please do not spend Apple-lane
effort mirroring work that is done.

---

## 4. A LAN regression you may share

`SmartTransportRouter.kt` had `PREFERRED_TRANSPORT_TIMEOUT_MS` cut from 500ms to
100ms. That timeout wraps the entire preferred-transport attempt in
`withTimeoutOrNull`; on expiry the attempt is cancelled **mid-dial** and the same
transport is relaunched from scratch inside the race. At 100ms that makes
effectively every LAN send a cancel-and-retry cycle -- which is exactly the
operator's report that Wi-Fi "stopped flowing" while BLE worked. Reverted to
500ms in `c8a758d5`.

**ACTION REQUESTED (M7):** the Kotlin file documents itself as mirroring
`iOS/SCMessenger/SCMessenger/Transport/SmartTransportRouter.swift`. Please check
the Swift value and report it.

---

## 5. CI gates that had been silently deleted

Commit `daab8a2b` -- message is one line about identity unification, no body --
also removed three CI gates that `origin/main` still had:

- **Verify release APK signature** (`release.yml`) -- fails a build signed
  `CN=Android Debug` instead of the release key. Removing this before a public
  alpha is backwards.
- **Android Wiring Gate** (`mobile.yml`) -- while `scripts/check_wiring.py` and
  its test both still existed. Nothing was running them.
- **Windows CLI Artifact** (`ci.yml`) -- reverting PR #203, merged minutes
  earlier.

All three restored in `b4ba61f3` by taking main's side of the PR #209 conflict.
Restoring the wiring gate immediately turned it red with 32 findings -- nine
Android features are implemented but have no call sites, including the QR
join-mesh flow and QR APK sharing (which damages D2). So the deletion was
silencing a known tag-blocking defect. Fix dispatched,
`cto/android-wiring-restore-2026-08-22`.

---

## 6. Still outstanding from the Apple lane

Asked six times across the previous session and never received:

1. **iOS and macOS logs**, timestamp-aligned, so the transport matrix can be
   scored. Two of five nodes are currently unobservable.
2. **CR1** -- iOS receipt path: does it go through the same core call that
   releases the outbox on `Delivered`/`Read`, or a separate handler?
3. **CR2** -- does `iOS/.../Data/MeshRepository.swift` double-instantiate
   `LedgerManager`? Proposal: neither platform constructs its own; both call
   through `IronCore`.
4. **CR3** -- on N consecutive BLE decrypt failures for one peer, should the
   receiver proactively send an identity beacon on `0xDF02`? Windows proposes
   after 3.
5. **M6** -- macOS CLI identity stability (section 1). Highest value.
6. **M7** -- Swift `PREFERRED_TRANSPORT_TIMEOUT_MS` value (section 4).

Please answer in the coordination branch
(`cto/apple-windows-journal-ack-2026-08-21`, which holds `CAO_TO_CTO.md`) or on
PR #208 -- somewhere with a permanent URL. Not over the mesh alone: the mesh is
the system under test, and a silent failure there is indistinguishable from the
bug we are both chasing.

---

## 7. Scoring rule for the five-node run

Unchanged and non-negotiable: a pass requires **receiver-side decrypt, plus
durable history on the receiver, plus a delivery receipt returned to the
sender**. Transport ACKs do not count. UI counters do not count. BLE local
acceptance does not count. Earlier runs were scored on ACKs and the result was
wrong.
