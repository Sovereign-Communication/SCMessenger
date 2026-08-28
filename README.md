# SCMessenger

Sovereign end-to-end encrypted mesh messenger. No servers, no accounts.

Messages move over whatever path exists — Bluetooth LE, Wi-Fi, the local
network, or an internet relay — and the transports race, so a message takes
whichever one is actually working.* If there is no internet, phones in range
still talk.* The Rust core is shared by every client.

`*` In progress — not yet proven end to end. See Project status below.

---

## Project status — read this first

**Pre-release. Not yet suitable for anyone who needs their messaging to be
private from a capable adversary.**

The most recent published build is [`v0.1.9`](https://github.com/Sovereign-Communication/SCMessenger/releases)
from March 2026, and it is well behind the current code. A `v0.4.0-rc.1`
Android build is the next release; final `v0.4.0` follows once device demos
pass.

Specifically, and deliberately stated up front:

- **No independent security audit has been performed.** The cryptography has
  been reviewed only by the people and tools that wrote it. That is not a
  credential. Until an external review exists, treat the privacy claims below
  as intentions, not guarantees.
- The post-quantum work is real but not uniformly enforced on every path.
- Known limitations are tracked in [`docs/V1_KNOWN_LIMITATIONS.md`](docs/V1_KNOWN_LIMITATIONS.md).

Use it to experiment, to test the mesh, and to tell us what breaks. Do not use
it yet for anything where being wrong about the threat model would hurt you.

---

## Threat model, in three sentences

SCMessenger assumes the network is hostile and that no server can be trusted,
because there is no server: peers exchange messages directly, and relays carry
sealed payloads they cannot read. It protects the **content** of your messages
end to end, and aims to make **who is talking to whom** expensive to determine
by racing transports and mixing relayed traffic. It does **not** protect you
from a compromised device, from someone who has your unlocked phone, or from an
adversary who can watch every radio in the room.

## Cryptography

| Purpose | Primitive |
|---|---|
| Key exchange | X25519 + ML-KEM-768 (hybrid, classical + post-quantum) |
| Signatures | Ed25519 + ML-DSA-65 |
| Message encryption | ChaCha20-Poly1305 |
| Forward secrecy | Double Ratchet |

The hybrid construction means a recorded conversation stays confidential even
if either the classical or the post-quantum half is later broken. Protocol
detail: [`docs/PQC_HYBRID_PROTOCOL.md`](docs/PQC_HYBRID_PROTOCOL.md).

---

## Install

**Android** — [`docs/SIMPLE_INSTALL_ANDROID.md`](docs/SIMPLE_INSTALL_ANDROID.md).
No toolchain needed; install the APK from a release.

**iOS** — [`docs/SIMPLE_INSTALL_IOS.md`](docs/SIMPLE_INSTALL_IOS.md). Requires a
cable and a Mac; there is no App Store or TestFlight distribution yet.

**Desktop CLI** — [Windows](docs/CLI_WINDOWS.md) · [macOS](docs/CLI_MACOS.md) ·
[Linux](docs/CLI_LINUX.md). The CLI is a full mesh node and also serves the web
UI on localhost.

Longer form: [`docs/INSTALL.md`](docs/INSTALL.md).

## Build from source

Requires a recent stable Rust toolchain.

```bash
git clone https://github.com/Sovereign-Communication/SCMessenger.git
cd SCMessenger
cargo build --release -p scmessenger-cli
```

```bash
cargo test --workspace
```

Android additionally needs the Android SDK and NDK plus `cargo-ndk`; the Gradle
build drives the Rust cross-compile and generates the UniFFI bindings. See
[`docs/ANDROID_QUICKSTART_WINDOWS.md`](docs/ANDROID_QUICKSTART_WINDOWS.md).

## Repository layout

| Path | What it is |
|---|---|
| `core/` | Rust core — crypto, transport, routing, storage. Shared by every client |
| `cli/` | Desktop mesh node and web UI host |
| `android/` | Android client (Kotlin, Compose) |
| `iOS/` | iOS client (Swift) |
| `wasm/` | Browser UI for a local CLI daemon |
| `mobile/` | UniFFI bindings glue shared by Android/iOS builds |
| `desktop_bridge/` | Desktop bridge service crate |
| `docs/` | Architecture, protocol, platform guides |

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) and [`SECURITY.md`](SECURITY.md).
Changes to `core/src/{crypto,transport,routing,privacy}` require an adversarial
security review before merge.

## Licence

[The Unlicense](LICENSE) — public domain.
