# Local Android Build Environment — TODO

**Date:** 2026-08-06
**Context:** While fixing CI for PR #138 (`fix/android-mdns-parity-2026-08-05`), discovered that local `./gradlew :app:compileDebugKotlin` and `:app:testDebugUnitTest` fail without UniFFI-generated bindings.

## Blocker

Local Gradle builds fail with cascading errors from missing UniFFI-generated Kotlin types:
```
Unresolved reference: uniffi
Unresolved reference: dial  
Overrides nothing  (interface members from UniFFI-generated types)
Cannot infer a type for this parameter
```

**Root cause:** `generateUniFFIBindings` task requires the Rust toolchain (cargo + targets) to compile `scmessenger-core` and generate Kotlin bindings. Without it, `uniffi.api.*` types don't exist on the classpath.

**CI status:** CI has the full Rust toolchain and passes all builds/tests cleanly (verified in run 31068542130).

## Options

1. **Install Rust toolchain locally**
   - `rustup install stable`
   - `rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android`
   - Then run: `./gradlew generateUniFFIBindings` once to generate the bindings
   - After bindings are generated, subsequent compile/test tasks work without Rust

2. **Document the dependency**  
   Add note to `android/README.md` about the Rust/UniFFI prerequisite for local development

3. **Create a CI-only CI task that skips Rust steps**  
   Not recommended — test coverage requires the full build

## Priority

Low — only affects local dev without Rust. CI is the source of truth for builds.