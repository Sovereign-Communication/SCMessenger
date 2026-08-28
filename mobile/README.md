# scmessenger-mobile

UniFFI mobile bindings crate for SCMessenger.

## Purpose

`scmessenger-mobile` exposes the core Rust API as native libraries for Android and iOS.
It is a thin wrapper over `scmessenger-core` and re-exports core types/interfaces through UniFFI-generated Kotlin and Swift bindings.

## Key Points

- Crate type: `cdylib`, `staticlib`
- Primary API source: `scmessenger-core` (`core/src/api.udl`)
- Consumer apps:
  - Android app (`android/`)
  - iOS app (`iOS/`)

## Build and Test

From repository root:

```bash
cargo build -p scmessenger-mobile
cargo test -p scmessenger-mobile
```

## Binding Generation

Binding generators live in `scmessenger-core`:

- Kotlin: `cargo run -p scmessenger-core --features gen-bindings --bin gen_kotlin`
- Swift: `cargo run -p scmessenger-core --features gen-bindings --bin gen_swift`

Platform helper scripts:

- Android setup check: `./android/verify-build-setup.sh`
- iOS setup check: `bash ./iOS/verify-test.sh`
