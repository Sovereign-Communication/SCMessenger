# V0.4.0 -- Ledger seeding via invite, mobile ledger gossip, de-hardcode node addresses

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: PARTIAL -- items 1, 2, 3 (response half), 6 implemented; items 4, 5 open
Created: 2026-07-25
Last updated: 2026-07-25
Scope: v0.4.0
Owner: unassigned

## Implementation status

| Item | State |
|---|---|
| 1. Invite carries routing-only seed ledger | DONE -- signed, leak-tested |
| 2. `LedgerManager` seed import | DONE -- `import_seed_entries`, `export_seed_entries`, `seed_addresses` |
| 3. Ledger gossip reciprocity | **HALF DONE -- see gap A** |
| 4. Mobile startup dial sweep | OPEN |
| 5. Remove hardcoded node addresses (Android/iOS) | OPEN |
| 6. `ConnectToSeedPeers` (was `ConnectToBootstrapRelay`) | DONE -- awaits real `ConnectionEstablished` |

All five Rust gates verified green on this changeset (fmt, clippy default,
clippy --all-features, `cargo test --workspace --no-run`, wasm32 release).

### Gap A -- ledger gossip is still one-directional

Approach (b) makes every node RECIPROCATE when asked, so CLI-to-phone now
converges both ways. But **nothing on mobile ever INITIATES**: `share_ledger`
remains unexposed over UniFFI and its only callers repo-wide are
`cli/src/main.rs:1868,2037,2891`. So **phone-to-phone still exchanges
nothing** -- the original symptom is only half closed. The spec's earlier
claim that (b) "fixes CLI, mobile, and WASM at once" was WRONG for the
initiate direction.

Fix: auto-initiate a ledger exchange on `ConnectionEstablished` in core, so
every platform participates without client code that can be forgotten.
Needs per-peer rate limiting so a reconnect loop cannot spam exchanges.
Note the privacy property: combined with the response-side change, a node
discloses up to 64 known-peer routing records to any peer it connects to or
that connects to it.

### Gap B -- invite signature compatibility break

Adding `seed_ledger` to `get_signable_data()` changes the signed byte string
for ALL tokens, including empty ones. Pre-v0.4.0 invites will not verify
against v0.4.0 and vice versa. Invites default to 30-day expiry so the blast
radius is bounded, but any invite already circulating for the Josh alpha test
will stop working. Operator sign-off required.

### Gap C -- `ConnectToSeedPeers` attempts one candidate per invocation

Separate `swarm.dial()` calls emit independent outcome events and the first to
arrive resolves the reply, so a fast failure on candidate 2 can mask a slower
success on candidate 1. Callers retry. A correct multi-candidate version needs
its own aggregating pending-dial structure.

## Operator direction (authoritative)

1. **There are no dedicated relays.** There are only nodes, and every node is a
   full relay. (Already true in code: `relay::Behaviour` is constructed
   unconditionally in `core/src/transport/behaviour.rs:523` and identify always
   advertises `.../relay/...` at `behaviour.rs:512-517`.)
2. **Bootstrap-based discovery is obsolete.** Discovery is ledger sharing.
3. **Never hardcode an IP address.**
4. **Seed delivery is invite/QR only.** Nothing ships a node list; there is no
   DNS seed and no shipped seed file.
5. **An invite must carry the inviter's entire ledger, including the inviter's
   own address**, so an invitee starts with the fullest current view of the
   mesh that the inviter has.

## Why this is the whole cold-start story

A node with an empty ledger and no LAN peers has no way to find a peer on the
open internet -- with zero seed data and zero infrastructure there is no signal
that distinguishes an SCMessenger node from any other host. That is a property
of the problem, not a gap. Internet-wide scanning is not an option: it is
indistinguishable from hostile reconnaissance, gets user IPs blocklisted, and
does not converge at internet scale. Public-DHT rendezvous would work
technically but reintroduces exactly the third-party dependency this project
exists to avoid, and publishes the membership list.

So: **the invite is the seed.** After the first peer, ledger gossip covers
everything else.

## Work items

### 1. Carry a seed ledger inside the invite (core)

`core/src/relay/invite.rs`. Add to `InviteToken`:

```rust
/// Snapshot of the inviter's ledger, including the inviter's own dialable
/// address, so a fresh invitee starts with a warm view of the mesh.
#[serde(default)]
pub seed_ledger: Vec<SeedLedgerEntry>,
```

Use `#[serde(default)]` so existing v1/v2 tokens still deserialize.

**SECURITY -- non-negotiable:** `seed_ledger` MUST be covered by
`get_signable_data()` (`invite.rs:124`). If it is outside the signature, anyone
who intercepts or relays an invite can inject attacker-controlled multiaddrs
that the invitee will dial on first launch. Add a regression test that mutates
`seed_ledger` on a signed token and asserts verification FAILS.

**ROUTING ONLY -- NO IDENTITY (operator directive 2026-07-25).** The seed
ledger carries connectivity information and nothing else, so that identities
are never auto-propagated. A `SeedLedgerEntry` is:

```rust
pub struct SeedLedgerEntry {
    /// Peer-id-stripped dialable multiaddr, e.g. /ip4/A.B.C.D/tcp/9001
    pub multiaddr: String,
}
```

**Explicitly EXCLUDED** -- do not add these back "for convenience":
`peer_id`, `public_key`, `nickname`, `topics`, `success_count`,
`failure_count`, `last_seen`. Every one of them is identity or behavioural
metadata about a third party who did not consent to being in this invite.

Rationale: an invite should hand over *where to knock*, not *who lives there*.
The invitee dials the address, completes the Noise handshake, and learns the
peer identity from Identify at connect time -- the same way it would for any
mDNS- or ledger-learned peer. `LedgerManager::annotate_identity()` already
exists for exactly this: attach identity after the fact.

This is safe because transport peer identity is NOT what secures messages.
Message confidentiality is per-contact X25519/XChaCha20-Poly1305 established
out of band from public keys; connecting to an unintended node at a given
address leaks nothing and decrypts nothing. Omitting `peer_id` does forgo
dial-time identity pinning, which is an availability consideration, not a
confidentiality one.

**Size:** a bare multiaddr is roughly 25-30 bytes, so 16 entries is about 500
bytes -- comfortably inside the QR byte-mode budget (~2953 bytes at ECC L).
The earlier compression requirement is therefore dropped. Still:
- Cap at N = 16 ordered by `get_preferred_relays()` ranking, with the
  inviter's OWN address always first and never evicted.
- Add a test asserting a full 16-entry token encodes under the QR budget.
- Add a test asserting the encoded token contains NO peer id, public key, or
  nickname -- i.e. that identity cannot leak back in via a future refactor.
  Assert on the serialised bytes, not just the struct shape.

**Residual privacy note (reduced, not eliminated):** the invite still reveals
the inviter's own IP and up to 15 other node IPs. That is unavoidable if the
invite is to be useful, and it is now bare routing data with no identities
attached. Document this plainly in the invite UI copy and in
`docs/BOOTSTRAP.md`.

### 2. Import the seed ledger on invite acceptance (core)

`core/src/store/ledger_entry.rs` -- `LedgerManager` currently has no import
path (methods are `new/load/save/record_connection/record_failure/
annotate_identity/dialable_addresses/get_preferred_relays/all_known_topics/
summary`).

Add a UniFFI-exported `import_seed_entries(&self, entries: Vec<SeedLedgerEntry>)`
that merges without clobbering: never overwrite an existing entry's
success/failure counters, never lower an existing `last_seen`, and dedupe on
peer-id-stripped multiaddr (match the CLI's key convention in
`cli/src/ledger.rs`). Imported-but-unproven entries must NOT be reported by
`dialable_addresses()` as if they had a success history -- check how
`dialable_addresses()` filters (`success_count > 0 && failure_count < 5`) and
decide deliberately whether seeds start at zero and are surfaced through a
separate accessor. Write the decision in a comment.

### 3. Expose ledger gossip to mobile (core + bindings) -- THE REAL GAP

`SwarmHandle::share_ledger()` (`core/src/transport/swarm.rs:1889`) is the only
way to initiate `/sc/ledger-exchange/1.0.0`, and it is NOT exposed over UniFFI.
Consequence today: a phone RECEIVES ledgers but can never send one, and the
inbound handler replies with an empty peer list (`swarm.rs:3534`) on the
assumption the app layer reciprocates. So two mobile devices never exchange
peers at all.

Fix one of two ways -- prefer (b):
 (a) expose `share_ledger` on `SwarmBridge` and have the clients call it, or
 (b) have `swarm.rs` populate the ledger-exchange RESPONSE itself from
     `core.ledger_manager` when a core handle is present, so every node
     reciprocates automatically regardless of platform.
(b) is better: it fixes CLI, mobile, and WASM at once and cannot be forgotten
by a client author.

Add an integration test with two in-process nodes asserting bidirectional
convergence. `core/tests/integration_ledger_convergence.rs` already exists --
extend it rather than starting a new file.

### 4. Mobile startup dial sweep

Neither client has an equivalent of the CLI's startup `DialScheduler` sweep
over `dialable_addresses()` (`cli/src/main.rs:1659-1701`). Android uses
`dialableAddresses()` only for nickname/route lookup. Add a bounded startup
sweep on both platforms (respect the existing max-3-concurrent-dial and
per-peer backoff policy the CLI uses -- do not invent a new policy).

### 5. Remove hardcoded node addresses

- `android/.../data/MeshRepository.kt:87` `getBootstrapNodesForSettings()`
- `android/.../data/MeshRepository.kt:96` `DEFAULT_BOOTSTRAP_RELAY`
- `android/.../data/MeshRepository.kt:8820` `prioritizedAddresses`
- `iOS/.../Data/MeshRepository.swift:129` `defaultBootstrapRelay`
Replace with `getPreferredRelays(n)` / `dialableAddresses()`. The pattern
already exists at `MeshRepository.kt:9112`.

Also delete the now-dead `MeshRepository.kt:98-109` `BootstrapSource` /
`EnvironmentBootstrapSource` (no references).

**iOS live bug, fix while here:** `MeshRepository.swift:831-836` computes
`bootstrapAddrs` from `defaultBootstrapNodes` + `getPreferredRelays(limit:10)`,
then **discards it** and calls `startSwarm(..., bootstrapAddrs: [])` at `:848`
while logging the count it did not pass (`:851`). Same shape at `:1062-1067`.
Pass the computed value.

### 6. Fix the ConnectToBootstrapRelay false success

`core/src/transport/swarm.rs` `SwarmCommand::ConnectToBootstrapRelay` replies
`Ok(())` when `swarm.dial()` merely QUEUES the dial, so
"Connected to bootstrap relay" is logged on Android and iOS even when no
connection ever forms. Await `SwarmEvent::ConnectionEstablished` for the dialed
peer instead. Given items 1-5, rename the whole mechanism away from
"bootstrap relay" -- e.g. `ConnectToSeedPeers` sourcing from the ledger.
The wasm arm added in `6e4e172d` returns an explicit unsupported error and
should be kept in whatever shape survives the rename.

## Known adjacent issues (do not fix here, do not regress)

- Two `LedgerManager` instances race on the same file: `IronCore.ledger_manager`
  (`iron_core.rs:169`) and the client-constructed one (`MeshRepository.kt:932`,
  `MeshRepository.swift:623`). Both whole-file rewrite; last save wins. Not
  confirmed to be causing loss -- verify before changing.
- `swarm_get_best_relays()` / `swarm_get_bootstrap_candidates()`
  (`iron_core.rs:2545`, `:2567`) are live but always return empty because the
  relay `BootstrapManager` is constructed with `Vec::new()` seed peers.
- `core/src/relay/bootstrap.rs` and most of `core/src/transport/bootstrap.rs`
  are dead. `BootstrapManager::bootstrap()` has zero callers repo-wide.
- Env var mismatch: code reads `SC_BOOTSTRAP_NODES`; the root
  `docker-compose.yml` and several `docker/*.yml` set plain `BOOTSTRAP_NODES`,
  which nothing reads. `docker/Dockerfile:7-8` uses a third name,
  `SCMESSENGER_BOOTSTRAP_NODES`. Docs were corrected in `6e4e172d`; the compose
  files were deliberately not touched.

## Gates

Machine is shared with other Claude Code sessions. Before ANY build:
`tasklist | grep -iE "cargo|rustc|gradle|java|ndk"` -- this repo forbids
concurrent build-tool invocations (Gradle spawns cargo-ndk upstream).

```
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments
cargo clippy --workspace --all-features -- -D warnings
cargo test --workspace --no-run
cargo build --target wasm32-unknown-unknown -p scmessenger-wasm --release
cd android && ./gradlew assembleDebug -x lint --quiet
```

The wasm target is the one that catches non-exhaustive `match` on
`SwarmCommand` -- `swarm.rs` has a SECOND match inside a
`#[cfg(target_arch = "wasm32")]` block around line 5321. Adding or renaming a
variant requires updating both. This is exactly how `d6252c9c` turned CI red.
Windows: `CARGO_INCREMENTAL=0` and `-j2`.
