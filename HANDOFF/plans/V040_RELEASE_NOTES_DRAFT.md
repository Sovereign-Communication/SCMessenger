# v0.4.0 Release Notes (DRAFT — not final until tag day)

## What SCMessenger is

Sovereign end-to-end encrypted mesh messaging. No servers, no accounts.
Messages move over whatever path exists — Bluetooth LE, Wi-Fi Aware /
Wi-Fi Direct, the local network, or an internet relay — and the Rust core
is shared by every client (Android, iOS, desktop CLI, WASM).

## Major changes since v0.1.9

### Post-quantum hybrid crypto completed

- ML-KEM-768 primitives and hybrid X25519 + ML-KEM-768 session
  establishment (PQC-01 through PQC-08), with suite negotiation
  (`0x01` legacy / `0x02` hybrid).
- PQ-augmented double ratchet wired into the live encrypt/decrypt send/
  receive path; legacy static-ECDH retirement gated with audit logging.
- Skipped ratchet keys persist across session reload.

### Transport parity across all radios

- BLE, Wi-Fi Aware, Wi-Fi Direct, TCP/mDNS, and internet relay paths on
  Android, iOS, Windows, macOS, Linux.
- Real pairwise Wi-Fi Aware PMK derived via X25519 ECDH; per-peer port
  negotiation via service-info TLV.
- Smart transport racing with fallback: first-choice transport
  unavailable -> another path delivers.

### Relay custody chain

- Drift/DTN custody-based relay store: sealed payloads relays cannot
  read, frame/envelope protocol, rate limiting, policy-driven forwarding.
- Custody state persists across relay process restarts; outbox
  flush-on-connect retry with receipt round-trip confirmation.

### UniFFI async bridge

- Migration from UDL scaffolding to proc-macro UniFFI definitions;
  suspend/async FFI methods tracked and re-baselined in surface
  snapshots.
- Robust binding generation (staged cdylib, CARGO_TARGET_DIR honored).

### Android stability fixes

- mDNS listener-collision crash, DNS resolver startup crash, identity
  race conditions, BLE scan stabilization and MAC-rotation session
  continuity, ANR prevention (heavy ops off main thread), notification
  parity, delivery-receipt envelope fix so send status converges to
  delivered.

## Install

**Android** — grab the APK from this release:
[Releases](https://github.com/Sovereign-Communication/SCMessenger/releases)
(guide: [docs/SIMPLE_INSTALL_ANDROID.md](docs/SIMPLE_INSTALL_ANDROID.md)).

Desktop CLI: [Windows](docs/CLI_WINDOWS.md) · [macOS](docs/CLI_MACOS.md) ·
[Linux](docs/CLI_LINUX.md). Build from source:
`cargo build --release -p scmessenger-cli`.

## Honest pre-release disclaimer

**Pre-release. Not yet suitable for anyone who needs their messaging to
be private from a capable adversary.**

No independent security audit has been performed. The cryptography has
been reviewed only by the people and tools that wrote it — treat privacy
claims as intentions, not guarantees. The post-quantum work is real but
not uniformly enforced on every path. Known limitations are tracked in
[docs/V1_KNOWN_LIMITATIONS.md](docs/V1_KNOWN_LIMITATIONS.md). Use it to
experiment, test the mesh, and tell us what breaks. Do not use it yet for
anything where being wrong about the threat model would hurt you.
