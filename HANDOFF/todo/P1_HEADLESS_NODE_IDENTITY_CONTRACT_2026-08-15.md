# P1 -- Correct the headless-node identity contract in AGENTS.md

Status: Open
Filed: 2026-08-15 (operator request after source verification)
Priority: P1 documentation/architecture contract. The current wording can cause
an implementer to add an incorrect application-identity gate to headless
store-and-forward nodes.

## Problem

`AGENTS.md:31` currently says:

```text
No anonymous packet forwarder exists or may be introduced: cmd_relay requires identity (cli/src/main.rs:2529).
```

This is ambiguous and currently incorrect if `identity` means the SCMessenger
user/application identity. The cited line is also stale: in the current source,
`cli/src/main.rs:2529` is unrelated to `cmd_relay`.

The architecture rule is still valid in a narrower sense: libp2p transport
cannot be anonymous. Every node needs a cryptographic transport keypair and a
PeerId for Noise, addressing, identify, and custody partitioning. That transport
identity is distinct from the optional persisted SCMessenger user identity
(`identity_id`, public key, device metadata, and consent).

## Verified implementation facts

1. `cli/src/main.rs::cmd_relay` constructs `IronCore`, ignores the result of
   `initialize_identity()`, and calls
   `load_or_create_headless_network_keypair()`.
2. `load_or_create_headless_network_keypair()` first reuses an existing
   `storage/relay_network_key.pb`, migrates an existing application identity
   key only when available, and otherwise generates and persists a new Ed25519
   libp2p transport keypair. A fresh install therefore does not need a user
   identity to start the headless node.
3. `cmd_relay` starts `transport::start_swarm_with_config(..., true, ...)`.
   The `true` selects headless mode; it does not require an application
   identity.
4. `core/src/mobile_bridge.rs::MeshService::resolve_swarm_keypair_and_mode`
   explicitly falls back to a persisted headless network key when
   `get_libp2p_keypair()` has no application identity. The existing test
   `test_fresh_install_without_identity_resolves_headless_mode_with_persisted_key`
   verifies this behavior.
5. `core/src/transport/behaviour.rs::IronCoreBehaviour::new` always enables the
   `/sc/relay/1.0.0` request/response behavior and the libp2p relay server.
   `core/src/transport/swarm.rs` handles custody/store-and-forward requests
   without checking for a local application identity. Normal abuse, budget,
   custody, and destination validation remain in force.
6. `docs/CURRENT_STATE.md:1963-1964` already describes a headless node as
   having no application identity, and `:2485` documents the persisted relay
   network key used for stable transport PeerIds.

## Required fix

Update `AGENTS.md` so it preserves the no-anonymous-transport rule while
explicitly allowing identity-free-at-the-application-layer headless nodes.
Prefer stable symbol/path references over a fragile line number. Suggested
contract language:

```text
No anonymous transport forwarder exists: every node must have a cryptographic
libp2p transport identity/PeerId. A headless node may have no persisted
SCMessenger user/application identity; `cmd_relay` uses its persisted or newly
generated `relay_network_key.pb` for transport and store-and-forward custody.
```

Do not describe headless nodes as standalone relay roles. Keep the repository's
nodes-not-relays doctrine and use `relay` as a behavior or code identifier.

## Scope constraints

- Documentation/contract correction only; do not change transport behavior,
  identity storage, custody policy, or node-role architecture in this task.
- Do not replace the transport keypair requirement with an anonymous forwarder.
- Do not add an application-identity gate to `cmd_relay`, `start_swarm`, or
  store-and-forward custody.
- Search for exact duplicates of the incorrect claim before closing the task.
  Current verification found only the `AGENTS.md` occurrence.
- Preserve unrelated shared-checkout modifications.

## Acceptance criteria

- [ ] `AGENTS.md` distinguishes transport identity/PeerId from SCMessenger
      application identity and no longer claims that `cmd_relay` requires the
      latter.
- [ ] The citation points to stable symbols or current source locations and
      does not claim that the stale `:2529` line is `cmd_relay`.
- [ ] The no-anonymous-transport rule remains explicit.
- [ ] Headless store-and-forward behavior remains explicitly permitted without
      a persisted user identity.
- [ ] Run the focused regression test from the repository environment:

      ```bash
      cargo test -p scmessenger-core --lib \
        mobile_bridge::tests::test_fresh_install_without_identity_resolves_headless_mode_with_persisted_key \
        -- --exact --nocapture
      ```

      Expected result: 1 passed, 0 failed. On non-Windows lanes this is
      advisory; the Windows host remains authoritative for Rust build gates.
- [ ] Report that no unrelated files were reverted or staged.

## Evidence anchors

- `AGENTS.md:20-33`
- `cli/src/main.rs::load_or_create_headless_network_keypair`
- `cli/src/main.rs::cmd_relay`
- `core/src/mobile_bridge.rs::MeshService::resolve_swarm_keypair_and_mode`
- `core/src/mobile_bridge.rs::tests::test_fresh_install_without_identity_resolves_headless_mode_with_persisted_key`
- `core/src/transport/behaviour.rs::IronCoreBehaviour::new`
- `core/src/transport/swarm.rs` relay protocol handler
- `docs/CURRENT_STATE.md:1963-1964,2485`
