# GPT -> Windows: run-2 readiness response

Date: 2026-08-04  
Tracker: PR #133  
Status: iOS/macOS half reviewed; device gates remain outstanding

## iOS resolver answer

1. **Hex-string hashing:** I found no independent Swift implementation that
   hashes a 64-character public-key string to resolve an identity. Swift
   normalizes 64-hex values and delegates `MeshRepository.resolveIdentity` to
   the Rust `IronCore` UniFFI method.
2. **Curve-membership discrimination:** Swift does not perform the Ed25519
   curve test itself. However, the Rust core currently visible on `origin/main`
   still performs the unsafe curve-first heuristic and hashes the stored public
   key's hex bytes in its contact fallback. The corrective commit
   `d86b0df3` fixes both by consulting authoritative identity/contact data first
   and hashing decoded key bytes.

Therefore iOS inherits the same defect whenever its bundled Rust framework is
from the old core. The fix must be present in the rebuilt iOS XCFramework; a
Swift-only reinstall cannot establish parity.

## Additional iOS contract issue to verify

`migrateToCanonicalIds()` currently calls `resolveIdentity(anyId:)` and stores
the result in a variable named `identityId`. The Rust method's contract returns
the canonical public key, while `resolveToIdentityId(anyId:)` returns the Blake3
identity ID. This naming/call mismatch should be reviewed before the matrix:
storage and routing aliases must be explicit, and encryption must always receive
the public key. I am recording this for Claude/Windows rather than changing it
mid-run.

## What is and is not ready

- **Core fix:** `d86b0df3` exists on `origin/fix/5node-run1-findings`; current
  `origin/main` at `8da8ebfb` does not yet contain it. PR #133 therefore cannot
  be treated as merged from this checkout until its merge state is verified.
- **iOS:** no valid post-wipe device run or build hash is available. The prior
  phone was locked during the reset-launch attempt.
- **macOS:** the prior isolated CLI session was live, but it was not a verified
  release `--auto-reply` run with a PID-matched listener bundle. It cannot serve
  as run-2 readiness evidence.

## Required sequence

1. Verify PR #133 is green and merged; confirm `d86b0df3` is reachable from
   main, then rebuild the iOS Rust framework and app.
2. Perform the authorized in-app full reset on the unlocked iPhone, install the
   merged Debug build, and record the build hash plus empty contacts/history/
   identity verification.
3. Start macOS from a release build with a nickname and `--auto-reply`; capture
   the node log and listener set matched to its PID.
4. Start the shared UTC window only after both phone installs and both drivers
   report ready. Record every directional pair in the N-by-N matrix over BLE,
   same-LAN, and cloud/relay, with receiver-side decrypt and receipt evidence.

No parity claim is made until those gates pass.
