# TASK: Fix PQC-09 onion routing compile errors (5 distinct issues, exact fixes below)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: TODO -- BLOCKING compile gate. All root causes below are verified
against current source -- apply exactly, do not re-diagnose.

## Fix 1: core/src/crypto/pq/hybrid.rs -- HybridCiphertext needs more derives

Current (around line 43):
```rust
pub struct HybridCiphertext {
    pub x25519_ephemeral_public: [u8; 32],
    pub mlkem_ciphertext: Vec<u8>,
}
```
`onion.rs` embeds this struct directly inside `HybridOnionLayer`, which
derives `Debug, Clone, Serialize, Deserialize` -- but `HybridCiphertext`
itself only has whatever derives it currently has (check the actual
current derive line just above `pub struct HybridCiphertext` in the
provided file content). Add `Debug, Clone, Serialize, Deserialize` to its
derive list (add `serde::{Serialize, Deserialize}` to hybrid.rs's imports
if not already present). Do NOT change the struct's fields or any function
in this file.

## Fix 2: core/src/privacy/onion.rs -- make layer structs public

`ClassicalOnionLayer` (around line 30) and `HybridOnionLayer` (around line
41) are currently private (`struct X`, not `pub struct X`). Add `pub` to
both -- `core/src/privacy/mod.rs` needs to re-export them (see Fix 3).

## Fix 3: core/src/privacy/mod.rs -- fix broken re-export

Current (around line 14-16):
```rust
pub use onion::{
    construct_onion, peel_layer, OnionEnvelope, OnionHeader, OnionLayer, MAX_ONION_HOPS,
};
```
`OnionHeader` and `OnionLayer` no longer exist -- PQC-09 replaced them with
`ClassicalOnionLayer` and `HybridOnionLayer`. Verified via repo-wide grep:
nothing outside `onion.rs` and this one re-export line references the old
names, so this is a safe rename, not a breaking API change elsewhere. Fix:
```rust
pub use onion::{
    construct_onion, peel_layer, ClassicalOnionLayer, HybridOnionLayer,
    HopAddress, OnionEnvelope, MAX_ONION_HOPS,
};
```
(Added `HopAddress` too since it's the new path-element type external
callers will need; check whether it's already exported elsewhere first to
avoid a duplicate-export error.)

## Fix 4: core/src/privacy/onion.rs:168 -- type annotation

```rust
let routing_info = vec![];
```
needs an explicit type, e.g. `let routing_info: Vec<u8> = vec![];` (check
how `routing_info` is used just below this line to confirm `Vec<u8>` is
correct, adjust the element type if it's actually used as something else).

## Fix 5: core/src/iron_core.rs -- update 2 call sites to the new function signatures

Real current signatures (core/src/privacy/onion.rs, do not guess further):
```rust
pub fn construct_onion(
    path: Vec<HopAddress>,
    payload: &[u8],
    require_pq: bool,
) -> Result<OnionConstructionResult, OnionError>

pub fn peel_layer(
    envelope: &OnionEnvelope,
    relay_secret_key: &[u8; 32],
    relay_mlkem_keypair: Option<&crate::crypto::pq::MlKem768KeyPair>,
) -> Result<(Option<HopAddress>, Vec<u8>), OnionError>
```

### Call site A (~line 2101-2113, the `construct_onion` call)

Currently builds `path: Vec<[u8; 32]>` from hex-decoded relay public keys
and calls `construct_onion(path, &envelope_data)` (2 args, old signature).
Fix: wrap each decoded `[u8; 32]` in `HopAddress::Classical(...)` (these
are raw hex pubkeys with no `PublicKeyBundle` available at this call site,
so `Classical` is correct -- this UniFFI-facing function does not yet
support specifying hybrid hops; that is an acceptable, separate follow-up,
NOT something to add here), and pass `require_pq: false` as the third
argument (preserves current behavior -- does not force hybrid-only
circuits). Do not change this function's own public signature
(`relay_public_keys_json: String` parameter, `Result<Vec<u8>, IronCoreError>`
return) -- only the internal call to `construct_onion`.

Also: `construct_onion` now returns `OnionConstructionResult`, not
`OnionEnvelope` directly (check the actual struct -- it may wrap the
envelope plus the new `pq_hops`/`total_hops` metadata fields from the
PQC-09 spec). Adjust the `bincode::serialize(&envelope)` call at the end
of this function to serialize whatever `construct_onion` actually returns;
if `OnionConstructionResult` doesn't derive `Serialize` yet, that's the
`iron_core.rs:2114` E0277 error ("OnionConstructionResult: Serialize not
satisfied") -- add `Serialize`/`Deserialize` derives to
`OnionConstructionResult`'s definition in onion.rs (safe, additive change,
same reasoning as Fix 1) rather than restructuring this call site.

### Call site B (~line 2130-2138, the `peel_layer` call)

Currently calls `peel_layer(&envelope, &secret)` (2 args, old signature).
Fix: pass `None` as the third argument
(`peel_layer(&envelope, &secret, None)`) -- this UniFFI-facing function
(`peel_onion_layer`, taking only `relay_secret_key: Vec<u8>` with no
ML-KEM keypair parameter) cannot yet peel hybrid-suite onion layers; it
will correctly still peel classical (suite 0x01) layers. Do NOT add a new
parameter to `peel_onion_layer`'s own signature to thread through an
ML-KEM keypair -- that would be a UniFFI-facing API-contract change,
explicitly out of scope for a compile fix (escalate separately if hybrid
onion peeling needs to be exposed through this bridge function; PQC-09's
master-plan rule 7 says bridge exposure of new APIs is out of scope unless
a task says otherwise).

Then fix the `next_hop.map(|h| h.to_vec())` line just below: `next_hop` is
now `Option<HopAddress>`, not `Option<[u8; 32]>`, so `.to_vec()` doesn't
exist on it directly. `HopAddress` already has a `.x25519_public(&self) ->
[u8; 32]` method (defined in onion.rs) -- use
`next_hop.map(|h| h.x25519_public().to_vec())` instead.

## Do NOT

- Do not change `construct_onion`/`peel_layer`/`HopAddress`'s own logic or
  signatures -- only their callers and the two structs' derives (Fixes 1-2
  are additive derive-only changes; the rest are call-site fixes).
- Do not add ML-DSA, WASM, or CLI changes -- this file set only.
- Do not touch `cover.rs`/padding logic.

## Gate

```
cargo check -p scmessenger-core -j 2
cargo test -p scmessenger-core --lib -j 2
cargo test -p scmessenger-core onion -j 2
```

## Output format (MANDATORY)

`core/src/iron_core.rs` is 3823 lines -- do NOT return its full contents,
only the two small call-site edits as a unified diff. Return unified diffs
for all four files (each fix above is a small, localized change).
