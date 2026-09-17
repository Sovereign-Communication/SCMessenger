# Multi-dimensional audit — 0.4.0 scope (iteration 1 of N)

Author: CTO seat, Freebuff lane. Method: live evidence from all three nodes plus
command-verified code reads against one SHA. No local build was run
(AGENTS.md rule 17; disk was TIGHT at 11.4 GB free during this audit).

Provenance for every claim below:

| Item | Value |
|---|---|
| Single candidate SHA | `c2ce2f6406b1713e895c043a735f6b3f1782d043` (main, PR #295 merged 2026-09-17) |
| Windows CLI | CI artifact `windows-cli-c2ce2f6406b1713e895c043a735f6b3f1782d043`, exe sha256 `aa814b93932a5f422437ad99e5ef71074f460ae4ef2fcde954e5a27970cbde10`, `/version` -> `git_hash c2ce2f6`, `core_provenance 0.4.0 (c2ce2f6:main:1789680238)` |
| AWS cloud node | image `testbotz/scmessenger:sha-c2ce2f6`, `/version` -> `git_hash c2ce2f6406b1713e895c043a735f6b3f1782d043`, `RestartCount=0` |
| Pixel 6a (bluejay) | app `0.4.0` `versionCode 15`, APK sha256 `4a0972b29556c4dadd0739667e2935aaededb5366ab6bed7e7c5a9f046ffe86b` — **candidate NOT installed**, keystore divergence parked by operator |
| Evidence dir | `tmp/audit_20260917/` |

The two nodes now report the same commit. The Windows short form (`c2ce2f6`) and
the container's full form (`c2ce2f64...`) are the same commit rendered by two
build paths, not provenance drift.

## 0. Full debugging logging — ENABLED and verified

| Node | Logging | Evidence |
|---|---|---|
| Windows CLI | relaunched with `RUST_LOG=debug` (the CLI resolves level from `RUST_LOG` via `EnvFilter::try_from_default_env()`, `cli/src/main.rs:989`) | 654 `DEBUG` lines in the current log hour, `~/AppData/Local/scmessenger/logs/scm.log.2026-09-17-22` |
| AWS cloud node | recreated with `-e RUST_LOG=debug` | 259 `DEBUG` lines in the container's last 400 log lines |
| Pixel 6a | unchanged app-file JSON log (structured, already verbose: 103 MB across the install) | `tmp/audit_20260917/pixel-mesh-tail.log` |

Deliberate choice, stated: `debug`, not `trace`. The host is TIGHT on disk
(11.4 GB free) and a trace-level soak on two nodes can produce GBs in hours.
Trace is available on demand but is a disk-budget decision, not a default.

Also fixed this session: the Windows node's binary now lives OUTSIDE `target/`
(`tmp/radio-candidates/c2ce2f64/`). It had been running from
`target/release/scmessenger-cli.exe`, and the sanctioned disk reclaim deleted
that path out from under the running process. New finding N-02 below.

## 1. Transport

| Check | Result |
|---|---|
| Windows listeners | 18 entries: LAN TCP on 192.168.0.121:{80,443,8080,9090,9002/ws}, loopback equivalents, and a live circuit `/ip4/18.234.62.247/tcp/9001/p2p/12D3KooWGvCW.../p2p-circuit/p2p/12D3KooWD6vZ...` |
| AWS listeners | TCP + WS on the instance; no LAN mislabel on its own side |
| Windows peers | 2/2: Pixel `12D3KooWD776...` and cloud node `12D3KooWGvCW...`, both reputation 50.0 |
| AWS peers | Windows `12D3KooWD6vZ...` |
| Connection path state | `DirectPreferred` on both |
| Discovery | `mdns_enabled` true, `ble_enabled` true, `wifi_aware_enabled` true on both |
| Live message flow | Windows->Pixel direct 13 ms / 28 ms, Pixel->cloud 273 ms, plus a real `route=relay relay=12D3KooWD6vZ...` fallback (2026-09-17 21:1xZ, `tmp/pixel_3node_20260917/`) |
| BLE | enabled in config, **not exercised**; the Windows host's radio is a known gate |

Verdict: `PASS` for the IP dimensions proven live (direct, LAN, WAN, relay
fallback, custody). BLE isolation and cell-only remain `UNVERIFIED` in this
iteration.

New finding **N-01**: the Windows node's discovery view labels the *AWS cloud
node* as `"transport":"tcp/lan"` even though that peer is a WAN peer reached
through a relay circuit. Either the label is wrong or the classification is
purely "TCP, not BLE/wifi-aware". Not a functional break — the message path
works — but it is telemetry that would mislead an operator reading the UI, and
it is the kind of thing the Android UI surfaces. Needs a read of
`DiscoveredPeer`'s transport assignment.

## 2. Identity

| Check | Result |
|---|---|
| Windows | peer `12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw`, identity `985a25f9...5826`, nickname `Claude-Windows-Driver`, `self_certifying: true` |
| AWS | peer `12D3KooWGvCWJNoWnReNCT1q2LWb2gTbeBTa5sjxF49wZX3u2y31`, identity `37eb7561...d006`, `self_certifying: true` |
| Pixel | identity `f83ab16319ca5b801f1c088935f2215c6aae9aa246b992f01c0f27f06b03cbe5`, peer `12D3KooWD776...`, nickname `Lucas` (read from the cloud node's history of the phone's own identity envelope) |
| Identity continuity across two node swaps today | preserved on both nodes (`docker rm -f` + fresh container on a new image, and a CLI binary swap) |
| Spoof guard (CRYPTO-01) | merged in PR #296, Rule-8 reviewed; present in this build |

Verdict: `PASS`. Residual: the cloud node reports `nickname: null` (pre-existing
cosmetic gap), and the Pixel's peer id was read from peer logs rather than from
the app UI, which is the honest source available without driving the device.

## 3. Storage

| Check | Result |
|---|---|
| AWS `/data` identity store | mounted `/opt/scm-relay-data` -> `/data`, uid 10001 after the ownership fix, survives container replacement |
| AWS custody | `Relay custody audit log count: 7706` observed live |
| Windows node store | intact; identity and ledger survived the binary swap |
| Storage pressure | no pressure warnings in the captured window |
| Degraded-storage handling | covered by the Android lane (`refuse to start on degraded storage`, PR #227 lineage) — not exercised this iteration |

Verdict: `PASS` for the live paths; `UNVERIFIED` for forced-pressure and
degraded-store behaviour, which need a deliberate fault injection rather than a
passive read.

## 4. State / liveness

| Check | Result |
|---|---|
| Health | `/health {"status":"healthy"}` on both nodes |
| Restart count | AWS `RestartCount=0` after the recreate |
| Drift | `{"state":"Dormant","store_size":0}` on both |
| Watchdogs | `cli/src/main.rs`: the swarm event-loop watchdog plus a log-silence heartbeat watchdog with its own `logs/watchdog/watchdog.log` |
| Nodes on one SHA | yes, both on `c2ce2f64` |

Verdict: `PASS`, with new finding **N-03**: the log-silence heartbeat watchdog
*exits the process by design* when the log goes quiet for the configured window.
That is the correct remediation for the 2026-09-15 silent wedge, but it means a
healthy-but-quiet node terminates. That trade needs a positive test (quiet node
+ no wedge must not die) before it is called done.

## 5. Parity

| Check | Result |
|---|---|
| Android wiring/reachability gate (`scripts/check_wiring.py`) | `[OK] All components, composables, routes, and utilities are correctly wired.` exit 0 |
| FFI surface contract | CI lane green on this SHA |
| Cross-platform matrix | CI `Cross` lane green (Android ABI splits, WASM, iOS aarch64 + sim, Kotlin/Swift bindings all produced) |
| Test lanes | `Test (ubuntu/macos/windows)` green; Mobile + iOS Build & Test were still in flight at capture |
| Device-level parity | the Pixel runs the Sep-16 build, so the *candidate's* Android changes are not on the phone. This is the one parity gap and it is deliberate (operator parked the keystore question) |

Verdict: `PASS` at source/artifact level; `BLOCKED` at device level by the known
keystore divergence.

## 6. Unification

| Check | Result |
|---|---|
| Canonical hex canonicalisation live | `ledger_canonical_hex_live` and `contacts_canonical_hex_live` events observed on both the phone and the nodes, rewriting libp2p peer ids to 64-hex on write |
| Outbox unification | PR #297 merged (canonical 64-hex drain, single Sled storage entry point) |
| Storage entry point | `IronCore` remains the single entry point; audit found no bypass in the paths exercised |
| Identity triad parity | every node and peer reports (identity_id, libp2p_peer_id, public_key_hex) consistently |

Verdict: `PASS` for the canonicalisation and outbox paths observed live.

## 7. Findings re-baselined against this SHA

Every row below is from a command run in this session against current main.

| ID | Verdict now | Evidence |
|---|---|---|
| TRN-04 custody auth + retention | **OPEN (unchanged)** | `core/src/store/relay_custody.rs:587` `accept_custody` has no sender authentication in its head; `grep -n "ttl\|expir\|retention\|max_age" relay_custody.rs` returns **nothing** — still no retention bound |
| TRN-07 global relay budget | **OPEN (unchanged)** | `core/src/transport/swarm.rs:3901` and `:8183` still `let mut relay_budget: u32 = 200;`, gated at `:4978` (`relay_count_this_hour >= relay_budget`). A `set_relay_budget` setter exists (`:3183`); the default is still a global 200/hour with no per-peer dimension |
| AND-06 Kotlin curve math | **OPEN (reduced)** | `PeerIdValidator.kt` still carries **7** `BigInteger` references (audit described 15; #289 consolidated some, not the math) |
| SEC-03 sled advisories | **OPEN (unchanged)** | `deny.toml:9-17` still waives RUSTSEC-2025-0141, -2025-0057, -2026-0118, -2026-0119 as "Transitive via sled — no safe upgrade available" |
| TRN-05 ghost classification | **RE-BASELINED, mostly addressed** | `swarm.rs:3300` `is_ghost_peer_topic` now carries "Fail closed on ghost shape when we cannot consult the ledger" (`:3328`). Residual: the wasm fallback branch, dispositioned LOW by the #296 verdict |
| AND-04 mDNS dial injection | **RE-BASELINED, validation present** | `MdnsServiceDiscovery.kt` skips loopback/link-local (`:302`), parses `dnsaddr` TXT explicitly (`:320-323`). Residual: does not validate that a resolved address is in a plausible LAN range |
| TRN-02 pressure-probe bypass | **NO LONGER APPLIES at the audited site** | the placeholder the audit described is gone; `swarm.rs:3702-3711` documents that registration state, `enforce_storage_pressure`, `storage_pressure_state` and `/api/diagnostics` now share `RelayCustodyStore`'s probe |
| CLI-02 watchdog false termination | **SUPERSEDED / NEW RISK** | watchdog set rewritten since: event-loop watchdog (`main.rs:2673-2691`) plus log-silence heartbeat watchdog (`:2704+`). See N-03 |
| TRN-06 custody scan complexity | **NOT VERIFIED** | grep found no scan function; the write-path cost needs a real read, not a pattern match |
| AND-05 sync FFI on UI thread | **NOT VERIFIED** | no `runBlocking`/`Dispatchers.Main` match in `ChatViewModel.kt`; absence of a pattern is not a verdict |
| SEC-02 keystore alias | **OPEN, operator-owned** | `android/app/build.gradle:19,105` read `SCMESSENGER_KEY_ALIAS`/`KEYSTORE_ALIAS`; the CI-vs-device debug keystore divergence measured this session is the same family |
| GOV-01 governance lock ordering | **NOT VERIFIED** | no `core/src` file matches "governance"; finding needs relocation before it can be scored |

## 8. New findings from this session

| ID | Severity | Finding |
|---|---|---|
| N-01 | LOW-MED | Discovery labels the WAN cloud node as `transport: tcp/lan` |
| N-02 | MED (process) | A node binary inside `target/` can be deleted by a sanctioned disk reclaim while it is running. Node binaries now live in `tmp/radio-<sha>/`; the runbook and rule 17 should say so explicitly |
| N-03 | MED | Log-silence watchdog terminates the process by design; a healthy-but-quiet node is indistinguishable from a wedge without a positive test |
| N-04 | HIGH (operator-visible) | Chat ordering compares the phone's clock with the sender's clock; filed as `HANDOFF/todo/P1_ANDROID_CHAT_ORDER_CROSS_CLOCK_2026-09-17.md` |
| N-05 | HIGH (device) | CI debug keystore != device debug keystore, so no CI-built APK can be installed in place. Parked by operator |

## 9. 0.4.0 scope verdict: NOT COMPLETE

The audit does **not** confirm 0.4.0 completion. Concrete remaining set:

**Blocking a 0.4.0 tag**
1. TRN-04 — sender authentication + retention bound for relay custody.
2. TRN-07 — relay budget policy (per-peer plus global).
3. AND-06 — move curve validation out of Kotlin `BigInteger` into Rust/UniFFI.
4. SEC-03 — sled replacement, or a documented, dated acceptance.
5. The eight NOT-RE-VERIFIED rows above need a real code read, not a grep.

**Not blocking the tag, but operator-visible**
6. N-04 chat ordering (filed).
7. N-05 keystore (parked by operator).
8. N-03 watchdog positive test.

**No final tag exists**: newest tag is `v0.4.0-rc.1`.

## 10. Forward work: 0.5.0 iOS parity

Ordered so that each item is independently verifiable:

1. **Parity matrix first.** Produce a tracked CLI/Android/iOS capability matrix
   for the 0.4.0 UniFFI surface, generated from the FFI surface contract rather
   than written by hand, so gaps are enumerated rather than asserted.
2. **iOS lane health.** The `iOS Build & Test` lane and the Swift binding lane
   are the only iOS signals that run without a Mac in the loop; confirm both are
   green on the tag candidate and keep them green.
3. **Capability gaps to close for 0.5.0** (candidates, to be confirmed against
   the matrix): APK share/link parity on iOS, JoinMesh/seed import parity,
   diagnostics export parity, and notification/event handling parity with the
   AND-01/02 cold-start work.
4. **Apple docs drift** — PRs #207/#208 and `gpt/v050-parity-burndown` are still
   open and untouched by this train; they need rebase-vs-close rulings.

## 11. The iteration loop

This is iteration 1. Each subsequent iteration:

1. Re-run the live dimension probes (sections 1-6) against the current fleet and
   capture to a fresh `tmp/audit_<date>/`.
2. Re-run `python scripts/check_wiring.py` and confirm the CI lanes green on the
   candidate SHA.
3. Promote at least one row out of `UNVERIFIED` by reading the actual code path
   (not a pattern match) and recording the file:line.
4. Close at least one blocking finding with a merged, Rule-8-reviewed change
   where the gate applies.
5. Re-issue this document as iteration N+1 with the delta stated explicitly;
   never rewrite history in place.

The audit is only allowed to declare 0.4.0 complete when every blocking row in
section 9 is CLOSED with a merged commit, and no row remains `UNVERIFIED`.
