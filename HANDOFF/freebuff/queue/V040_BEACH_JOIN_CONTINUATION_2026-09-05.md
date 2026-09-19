# V040 beach-join continuation -- finish current mission, then execute the beach-join plan

Status: OPEN (filed 2026-09-05, operator directive)
Priority: P0 -- this is the operator's stated path to stranger-ready sharing
Lane: Freebuff / DeepSeek V4 Flash (or any unmetered model)
Scope: `android/` (Phase 1), `core/src/store/` + ingest call sites (Phase 2),
`core/src/identity/keys.rs` + `core/src/store/contacts.rs` (Phase 3).
Do NOT touch `core/src/transport/swarm.rs` connection bookkeeping, the
`is_dialer()` ledger guard, BLE ingress semantics in `mobile_bridge.rs`, or
any file outside the phase scope without filing an inbox note first.

## Mission order (read this first)

1. **Finish your current mission first.** Complete whatever in-flight work the
   queue index (`HANDOFF/freebuff/README.md`) lists ahead of this file --
   including the #276 follow-through and the 3-node You-Pixel-AWS validation --
   and report per the inbox return contract before starting below. Do not
   interleave the two missions in one PR.
2. **Then execute the beach-join plan in phase order:** Phase 0, then Phase 1
   and the Phase 2 ingest in parallel (disjoint files), then Phase 2 device
   proofs, then Phase 3 review, then stop. One PR per phase step. No self-merge.

## The design (verified, do not re-derive -- implement it)

Full audit: `HANDOFF/plans/BEACH_JOIN_AUDIT_AND_PLAN_2026-09-05.md`. Read it
whole before writing code. Operator rulings baked in: auto-hotspot flow;
stranger-grade trust ships in the same package (Phase 3 blocks beach use).

Use case: stranger with no WiFi/internet scans one QR on your phone, gets the
APK over your phone's local-only hotspot with hash verification, installs,
joins the mesh, and learns the always-on cloud node address -- so both phones
resume via the cloud node after parting.

## Phase 0 -- formats first (spec, no code)

Write the join-bundle v1 and single-QR payload spec as the header comment (or
a short doc next to the code that implements it in Phase 1):
`WIFI:S:<ssid>;P:<pass>;;` + `http://<hotspot-ip>:<port>/scmessenger.apk` +
`#bundle=<base64-v1>` + `#sha256=<hex>` + mandatory format version. Bundle v1
fields: `cloud_seed_addrs[]`, inviter contact bundle, bundle signature,
APK SHA-256, version. Phases 1-2 implement exactly this; changing it later is
a wire-format migration, so get the version field right now.

Acceptance: spec on the page; Phases 1-2 cite it. No code required.

## Phase 1 -- hotspot share + verified install (Android-only, no Rule-8)

Starting points (all wired, all read this session):
- `android/.../utils/ApkShareManager.kt:109` `startLocalApkHost()` serves the
  APK on `ServerSocket(0)`; URL built at `:116,131`.
- QR render `ui/components/QrCode.kt:51`, dialog `ui/dialogs/ApkShareDialog.kt:46`,
  entry `ui/screens/SettingsScreen.kt:262,456-458`.
- Join/scan screen `ui/join/JoinMeshScreen.kt:161-184`, route `ui/MeshApp.kt:410`.

Work:
1. Sender opens a `LocalOnlyHotspotReservation` on Share; bind the host to the
   hotspot interface IP. **Replace first-IPv4-pickup**
   (`ApkShareManager.kt:81-99` `getLocalIpAddress`): with no WiFi it returns
   the cellular CGNAT address, which is unreachable and leaks presence -- and
   its own comment (`:128-130`) records a past mis-seeding incident from this
   URL. Select the hotspot-subnet address explicitly; reservation loss stops
   the host.
2. QR encodes WIFI creds + URL + SHA-256 + bundle pointer (Phase 0 format).
3. Receiver fetch + **SHA-256 verify-before-install, fail closed on mismatch**.
   Test the unknown-sources handoff on a real device path; if no second
   handset is available, mark device legs `UNVERIFIED` and say which leg.
4. Hotspot-IP disclosure hygiene: confirm hotspot-local observations do not
   leak as globally dialable (RFC1918 same-subnet filter,
   `core/src/store/ledger_entry.rs` `addr_filter`); add a regression test if
   the filter needs it. Read-only there unless the test fails.

Scope correction (do not "fix" these -- all verified correct):
- The `is_dialer()` guard on the ledger writer stays; it is load-bearing.
- BLE ingress (`mobile_bridge.rs:1582`) stays receive_message-only; do not
  synthesize `ConnectionEstablished` from BLE (swarm bookkeeping assumes a
  libp2p connection that does not exist there).
- `MeshVpnService` orphan status is out of scope for this package.

Acceptance: second-handset proof with WiFi/cell off -- Pixel A shares to a
wiped handset, hash-verified install, app opens. CI green
(`Android JVM Unit Tests`, `Android Wiring Gate`, Lint). Device legs without
hardware are `UNVERIFIED`, never claimed.

## Phase 2 -- seed import + cloud resume (Rule-8 review required)

1. Bundle ingest: parse v1, persist cloud multiaddrs to the seed/ledger store
   at the `MeshRepository.kt:3848` site (starts with `listOf()` today), so the
   existing boot seed dial (`mobile_bridge.rs:861-871`) and relay re-dial have
   candidates. Persist the inviter contact **through `verify_bundle`**.
   (`JoinMeshScreen.kt:348-419` `parseAndJoin` is dial-only legacy -- replace,
   do not extend: its `"bootstrap_peers"` format is superseded by v1 and its
   referenced `handle_join_bundle` does not exist.)
2. Prove hotspot-LAN gossip top-up on device: mDNS + `ConnectionEstablished` +
   ledger exchange (`swarm.rs:5497-5527`) between the two phones on the hotspot
   subnet. If it does not fire, STOP and file an inbox note with log evidence
   instead of redesigning around it.
3. Depart-and-return proof: both phones leave hotspot range, regain cell,
   re-mesh via the cloud node unaided; custody delivers a message sent while
   one side was offline. Score from receiver-side display + durable history +
   receipt and the swarm audit log -- not the API custody counter alone
   (re-verify which store it reads: `iron_core.rs:398` vs `swarm.rs:3019`).

Acceptance: depart-and-return proof on the reviewed head; full gate set on the
Windows host (core check --all-targets, lib suite, clippy `-D warnings`, fmt,
wasm32) + CI green. Rule-8 adversarial APPROVE from a non-author reviewer
before merge (touches store/transport surface).

## Phase 3 -- trust wiring + adversarial gate (blocks beach use)

Wire `verify_bundle` (`core/src/identity/keys.rs:452`) at every
bundle-ingestion site, `save_contact_bundle` (`core/src/store/contacts.rs:669`)
behind it, reject-unsigned fail closed. APK provenance story (hash gate from
Phase 1 + signature pinning). Rule-8 APPROVE, plain `Verdict: APPROVE` only.
Then stop: merges and the tag decision are the operator/CEO's, never yours.

## Rules for every phase

- No emojis. `[OK]` / `[FAIL]` / `[WARNING]` / `[INFO]`.
- No `unwrap()` in production paths; no hardcoded UI strings (strings.xml).
- Shared checkout: touch only the phase scope. Never `commit -a`, `clean`,
  `reset`, or `rm -rf`. Never merge, tag, or force-push.
- Never read `$?` after a pipe. Never `cargo clean`; use
  `scripts/clean_target.sh`. One build tool at a time (check tasklist first).
- If a premise here does not survive contact with the code, STOP and write
  `HANDOFF/freebuff/inbox/V040_BEACH_<what>_2026-09-0X.md`
  (`Task:` + `Type: PREMISE-WRONG`, exact command + output) instead of
  implementing a fix to a nonexistent problem.
- Completion note per phase to `inbox/`: PR number, what changed, any
  acceptance criterion left `UNVERIFIED`.
