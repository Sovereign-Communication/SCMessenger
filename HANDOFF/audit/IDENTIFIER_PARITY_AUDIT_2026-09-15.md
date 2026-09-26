# Canonical Identifier Parity Audit — 2026-09-15 (~00:40Z)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Operator directive: "ensure we are using unified canonical identifiers to ensure no
mismatch between versions (parity)." This audit was run while the mesh was LIVE with
three nodes on three build generations, which makes it a cross-version interop test
at the same time. Every claim below cites output obtained this session.

## Verdict

PASS — the canonical identifier scheme is unified across CLI, Android, iOS, and the
UDL contract. No split-identity defect is live. Two latent hazards found are
quarantined and tracked (see Findings).

## The doctrine, as declared (not as remembered)

Three independent authority statements, all read this session, all agree:

1. `core/src/api.udl:42-51` — "IdentityInfo — canonical identity = public_key_hex
   (Ed25519 hex). identity_id (Blake3 hash) and libp2p_peer_id are
   derived/operational metadata. Always use public_key_hex for persistence,
   exchange, and cross-platform resolution."
2. `core/src/message/types.rs:41` — "UNIFICATION_V2: canonical identity is
   public_key_hex; identity_id/libp2p_peer_id are derived metadata only."
3. `core/src/identity/keys.rs:63-76` — "public_key_hex is the only self-standing
   identifier... Ed25519 libp2p peer ids are identity multihashes... pubkey
   <-> peer_id is reversible; identity_id is one-way from pubkey."

## Canonical scheme (single sources of truth)

| Identifier | Authority | Derivation | Direction |
|---|---|---|---|
| public_key_hex (64 hex, Ed25519) | `identity/keys.rs` | held by identity; PERSISTED everywhere | canonical |
| libp2p_peer_id (12D3KooW...) | `store/ledger_entry.rs:435` `peer_id_from_public_key_hex` | protobuf identity multihash `00 24 08 01 12 20 <32B key>` -> base58btc | derived, reversible |
| key from peer_id | `store/ledger_entry.rs:454` `public_key_hex_from_libp2p_peer_id` | strict 38-byte decode + RE-DERIVE defense-in-depth (`ledger_entry.rs:468`) | derived, reversible |
| identity_id (64 hex) | `identity/keys.rs:39` `identity_id_from_public_key_hex` | blake3(pubkey bytes); REJECTS non-curve-point input so an identity_id can never be double-hashed | derived, one-way |
| message/convergence IDs | `transport/swarm.rs` (`relay_message_id` from `request.message_id`; topic `sc-receipt-convergence` at `swarm.rs:1126`) | core-only; Kotlin consumes via FFI, never recomputes | core-only |

## Implementation inventory (who derives identifiers, and how)

- **Rust core** — the sole canonical authority. All derivations above live here.
  Storage only through IronCore (AGENTS.md rule 7 respected by design).
- **Kotlin (`utils/PeerKeyUtils.kt`)** — documented byte-exact mirror of the Rust
  peer-id math: same 6-byte protobuf header, same strict 38-byte acceptance, same
  BigInteger base58 (its comment names the Rust functions it mirrors). The
  re-derive check (`PeerKeyUtils.kt:60-64`) mirrors `ledger_entry.rs:468`. Callers:
  7 call sites in `MeshRepository.kt` (9718, 9963, 10377, 10589, 10880, 10896,
  11043) — all in the cold-start/FFI-unavailable fallback path, exactly where the
  platform-parity doctrine requires a dumb local mirror instead of a core round-trip.
- **Kotlin does NOT reimplement blake3**: every `blake3` mention in
  `android/app/src/main/java` is a comment (searched this session; 0 code hits).
  `identity_id` is always obtained through `IronCore.resolveIdentity()` (FFI).
- **iOS (`Data/MeshRepository.swift`)** — searched `12D3KooW|toPeerId|multihash|base58`:
  only a VALIDATION guard at `MeshRepository.swift:6601-6605` (does not append
  `/p2p/` to non-libp2p IDs) and a strict base58BTC charset check (`:7280`). No
  third derivation exists. iOS constructs no identifiers itself.
- **UniFFI boundary (`api.udl`)** — `IdentityInfo` carries all three identifiers;
  the contract comment pins which one persists. Generated bindings were not edited
  (AGENTS.md rule 6 respected).

## Cross-version interop evidence (the "mismatch between versions" test)

Observed live, this session, at cloud-node boot (00:39:19Z logs):

- Cloud node redeployed `sha-ccce98c` (22h-old image) -> `sha-31776b4` (libp2p 0.57
  stack, glibc-pinned builder). Identity `12D3KooWGvCWJ...` PRESERVED through the
  redeploy via the `/data` bind-mount — the same peer id across 3 successive images.
- Within 6 seconds the fresh cloud node discovered gossip topics
  (`sc-lobby`, `sc-mesh`, `sc-receipt-convergence`, per-peer v1 topic) from the
  Pixel peer `12D3KooWKT1e1...` (Android APK build `b39bfd2d` tree).
- Windows node (build `c384850`, pre-bump) peer `12D3KooWD6vZ...` already in the
  cloud ledger from the earlier cell-path delivery test.
- Conclusion: peer-id format, topic names, and convergence-marker protocol are
  byte-compatible across the libp2p 0.56 -> 0.57 boundary AND across all three
  deployed build generations. No identifier drift was introduced by the bump.

## Findings and dispositions

1. **PASS — canonical scheme unified.** Two implementations of the peer-id math
   (Rust canonical, Kotlin cold-start mirror) are byte-exact and re-derive-checked;
   iOS derives nothing; UDL pins the contract.

2. **TRACKED (1.0.0) — Kotlin legacy multihash formats.** `PeerKeyUtils.kt:70-96`
   still ACCEPTS the old Kotlin-only 36-byte (0x12 + key + 2B checksum) and 34-byte
   formats for reading. Rust accepts none of them (`ledger_entry.rs:456` requires
   exactly 38 bytes). This is read-compat for ledger entries written by old Kotlin
   builds (checksum failure is logged, not fatal, by design). Quarantined: these
   formats can never be GENERATED by current code paths. Disposition: remove both
   branches at 1.0.0 when old ledgers are deprecated. Not a v0.4.0 item.

3. **TRACKED (structural fix via P1 ticket) — `generateFallbackPeerId` synthetic
   `peer_<hex>` format.** `PeerKeyUtils.kt:157-162` returns `peer_<8hex>` when
   generation fails. Rust never recognizes this shape; it exists only so
   cold-start has SOME identifier. Guards confirmed present:
   `GhostIdentityGate.kt:24-47` blocks persisting a dead pk-as-peer_id, and
   `MeshRepository.kt` (synthetic-nickname filtering) blocks synthetic strings
   overwriting real identity data. Disposition: the real fix is the recovered P1
   ticket `P1_ANDROID_UNIFFI_CURVE_CHECK_RELOCATION.md` (single UniFFI-side
   validator, removing the multi-copy fallback surface). v0.4.0 keeps the guard.

4. **No conflict with PR #289** (work-ahead curve-check unification, Kotlin-only):
   it consolidates the three curve-validation copies but does not touch
   `PeerKeyUtils` derivation or the canonical persistence rule. Audit verdict for
   that PR filed separately: `HANDOFF/freebuff/inbox/PR289_WORKAHEAD_AUDIT_VERDICT_2026-09-14.md`.

## Node version-parity state at audit time

| Node | Build | Identity | State |
|---|---|---|---|
| Cloud (AWS i-0b41aab756eabd514, 18.234.62.247) | `sha-31776b4` (tip tree) | preserved | LIVE, mesh reformed |
| Pixel 6a | APK from CI run 34907771392 (tree `b39bfd2d`; only delta to tip is wasm-scoping + docs) | fresh install | installed 13:56Z, gossiping with cloud |
| Windows | `c384850` (CI artifact, pre-bump tree) | preserved | LIVE — redeploy to tip artifact pending CI |

Windows redeploy to the tip CI artifact is the last parity step and was queued the
moment the artifact appears (CI run for `f6eda77a` in flight at audit time).

## COMPLETION (01:03Z, same session)

- Windows node redeployed from CI artifact of run 34914308625 (provenance manifest
  `0.4.0 (f985b10)`, run headSha `7ad7ad53d`, main verified ancestor -> merge-preview
  content == branch content). Identity preserved (`Loaded existing identity`),
  relaunched with the exact prior command (`start -p 9001`).
- Runtime evidence of the doctrine EXECUTING: on boot the node logged
  `ledger_canonical_hex_live` -- canonicalizing the Pixel's libp2p peer id
  `12D3KooWKT1e1...` to public_key_hex `8f1c7580...` on ledger write -- followed by
  `Inbound relay circuit established from 12D3KooWKT1e1...` (Pixel reached the
  Windows node through relay on the cell path).
- FINAL PARITY STATE: cloud `sha-31776b4`, Windows `f985b10`, Pixel `b39bfd2d` APK
  -- three different SHAs, ZERO runtime-content delta between them (remaining
  deltas are docs-only + wasm dep-scoping). All three peers visible to each other
  across gossip + relay + custody paths.
