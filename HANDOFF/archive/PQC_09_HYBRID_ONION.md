# TASK: PQC-09 — Hybrid per-hop onion routing

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Read `PQC_00_MASTER_PLAN.md` first. Depends on: PQC-05, PQC-03. Wave 3. Min tier: Sonnet. ADVERSARIAL REVIEW MANDATORY (privacy/ module).

## Why

Audit finding F4: `core/src/privacy/onion.rs` wraps each layer with ephemeral X25519 to the relay's public key. Recorded onion traffic yields full route reconstruction post-CRQC. Each hop must move to the PQC-05 hybrid KEM.

## Design (fixed)

- Relay addressing today: 32-byte X25519 public per hop (`path: Vec<[u8; 32]>`). Hybrid hops need the relay's `PublicKeyBundle` (PQC-03). Relay records (registry/bootstrap — `rg -n "relay" core/src/relay/bootstrap.rs core/src/relay --type rust -l`) gain the bundle; a relay without a bundle is a "classical hop".
- Layer format v2 behind a tag byte (same discipline as PQC-02): `0x02 || bincode(OnionLayerV2 { hybrid_ct: HybridCiphertext, payload: Vec<u8> })`; classical v1 layers keep current bytes. Per-layer key: the 32-byte PQC-05 output feeding the existing per-layer AEAD unchanged.
- Mixed paths: ALLOWED by default (a classical hop is still better than no onion), but the circuit result must expose `pq_hops: u8 / total_hops: u8` so callers can log/report; `require_pq = true` refuses to build a circuit unless ALL hops are hybrid.
- Overhead: +1120 B per hybrid layer (1088 ct + framing) — confirm against drift fragmentation for BLE paths; record measured envelope sizes for a 3-hop circuit in this file.

## Steps

1. Extend relay records with optional bundles (sled compatibility discipline from PQC-03 applies — state encoding choice).
2. `create_onion_envelope` / `peel_onion_layer` v2 paths (anchors: `onion.rs` functions at lines ~94-291; keep v1 peel forever).
3. Tests: 3-hop all-hybrid roundtrip; mixed path (hybrid-classical-hybrid) roundtrip; tampered hybrid layer fails cleanly at that hop only; v1-only circuit unchanged (fixture bytes); strict-mode refusal on mixed path; proptest: arbitrary bytes into `peel_onion_layer` never panic.

## Definition of Done

- [ ] Standard gates PASS.
- [ ] `cargo test -p scmessenger-core onion` green (unit + new tests); `--test integration_relay_custody` still green.
- [ ] Measured size table written into this file.
- [ ] Adversarial review verdict recorded here.
- [ ] File moved to HANDOFF/done/ + committed.

## Do NOT

- Change cover-traffic/padding logic (`privacy/cover.rs`) except where hint sizes must match new layer sizes — if padding assumptions break, list them here and escalate rather than guessing.

## Correction (2026-07-11, verified against current source)

The function names in this spec are stale. The REAL current functions in
`core/src/privacy/onion.rs` are `construct_onion` (not
`create_onion_envelope`) and `peel_layer` (not `peel_onion_layer`), at
different line numbers than stated above (file is 508 lines total). Read
the actual file (provided in context) and work against its real structure;
ignore the specific line-number ranges in "Steps" item 2 above.
