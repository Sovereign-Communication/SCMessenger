# TASK: Investigate/fix hardcoded WiFi Aware PMK derivation input

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: TODO, not yet investigated beyond the initial spot-check that found
it. Farm-relevant (WiFi Aware is a farm-proximity transport, even though
currently ranked below the P0 mDNS/QUIC-TCP/BLE trio).

## Finding (from `_QUEUE.md`'s B3 investigation note, 2026-07-11)

`core/src/mobile_bridge.rs` ~line 1422: WiFi Aware PMK (pairwise master key)
derivation uses a hardcoded `[0x42u8; 32]` byte array as the `derive_key`
input, rather than real per-session/per-peer shared secret material. If
accurate, this would make the derived PMK IDENTICAL across all peers and
sessions, defeating pairwise isolation - any two WiFi-Aware-connected devices
would derive the same key material regardless of who they're actually
talking to.

This was spotted incidentally while confirming WiFi Aware's `send()` is not
an orphaned stub (it isn't - B3 closed as false positive, delivery works via
a loopback TCP proxy). The PMK finding itself was NOT investigated further at
the time.

## CONFIRMED (2026-07-11, verified directly against current source)

Real line, `core/src/mobile_bridge.rs:1421`:
```rust
let pmk = blake3::derive_key("SCMessenger Wi-Fi Aware PMK", &[0x42u8; 32]);
```
This IS the real bug, not a misread - `"SCMessenger Wi-Fi Aware PMK"` is
blake3's domain-separation context string (fine as a constant), but
`&[0x42u8; 32]` is the actual KEY MATERIAL input, hardcoded identically for
every peer/session. Confirmed real, not a false positive like B3.

A Qwen dispatch was tried for the fix (full-file mode) and came back
UNUSABLE: it called an undefined `derive_wifi_aware_pmk(&core, &peer_id_parsed)`
helper that it never actually defined anywhere in its own response, AND its
"full file" reproduction was only 1815 lines against the real file's 4283 -
would have destroyed ~2468 lines of real code if blindly applied. NOT applied
(no --apply used). Re-dispatch this task in `--mode diff` only, never
full-file, given the file's size - see
[[feedback_delegate_task_full_file_mode_size_limit]] memory note.

## Constraint the real fix must work within

At the `on_wifi_aware_peer_discovered` call site (~line 1407-1443), only
`peer_id: String` (parses to a `libp2p::PeerId`) and
`core: Arc<Mutex<Option<Arc<IronCore>>>>` are available - there is no
guarantee the remote peer is already a known contact with a stored
`PublicKeyBundle`/X25519 key at this point (WiFi Aware discovery can fire
before any contact relationship exists). A real fix needs to either:
(a) derive the remote peer's X25519 public key from their libp2p `PeerId`
(this codebase already has ed25519-pubkey-from-PeerId extraction - see the
passing test `peer_id_public_key_extraction_roundtrips_for_ed25519_peers` in
`core/src/transport/swarm.rs` - plus the existing `ed25519_public_to_x25519`
helper in `core/src/crypto/encrypt.rs` - chain those two), combined with the
LOCAL node's own X25519 secret (check what `IronCore` exposes for this), then
real X25519 ECDH into blake3::derive_key; or (b) determine PMK derivation
should actually happen later, after a real session/contact relationship
exists, rather than at bare discovery time - needs a decision, not just a
mechanical fix.

## What to do

1. Read `core/src/mobile_bridge.rs` around line 1422 and trace where the
   `[0x42u8; 32]` value comes from and what `derive_key` call it feeds -
   confirm whether this is genuinely a hardcoded placeholder or whether it's
   combined with other real per-session entropy elsewhere (e.g. is `[0x42u8;
   32]` a fixed DOMAIN-SEPARATION constant/context string for a `blake3::derive_key`
   call rather than the actual key material - which would be fine/intentional
   - or is it actually standing in for a missing real shared secret?).
2. If it IS a real gap: identify what the actual shared-secret input should
   be (X25519 ECDH result between the two peers' identity keys, matching the
   pattern used elsewhere in this codebase e.g. `derive_layer_key` in
   `core/src/privacy/onion.rs`), and fix the derivation to use it.
3. If it's NOT a real gap (e.g. legitimate domain-separation constant): close
   this as a false positive with the specific evidence, same as B3.

## Gate

This touches `core/src/mobile_bridge.rs` (transport-adjacent) - if a real
fix is needed, mandatory adversarial review applies per repo security rules.
Standard compile gate + any existing WiFi Aware tests green either way.
