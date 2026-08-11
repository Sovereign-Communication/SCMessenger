# PR #139 remediation record

Status: implementation complete for the locally verifiable security and release-gate findings; device and Windows field gates remain required.

## Implemented

- RFC1918, CGNAT, and ULA ledger disclosure requires the observed transport requester address as separate evidence.
- Private disclosure uses actual IPv4 `/24` or ULA IPv6 `/64` matching and fails closed for missing, relayed, loopback, link-local, unspecified, multicast, and broadcast addresses.
- Ledger responses only include successfully connected entries; unproven `public_key` rows are not redistributed.
- Transport block checks fail closed on storage/core lookup errors and when the block manager is unavailable.
- Blocked identifier flavors are explicit (`pk:` and `id:`); unprefixed values are not classified by curve-point validity, preventing identity-ID double hashing.
- Rejecting a stale message request falls back to blocking its request identifier directly.
- Block/unblock history and outbox cleanup cover explicit public-key aliases when contact provenance is available.
- Device registration mutates the block store under a write lock.
- Outbox transport-queued messages increment attempts and receive a receipt timeout before retry.
- Build provenance reads the stamp emitted by `core/build.rs`.
- Release metadata is aligned at `0.4.0` and the local shell validation gates are syntactically valid.

## Local evidence

- `cargo check --workspace --features test-utils`
- Focused RFC1918 and identity-flavor tests pass.
- `cargo test -p scmessenger-core --features test-utils --lib`: 1317 passed, 3 environment-only failures caused by local socket/permission restrictions, 5 ignored. The authoritative Windows CI run on the PR head previously reported 1317 passed, 0 failed, 5 ignored.
- `bash scripts/verify_versions.sh`
- `bash -n scripts/*.sh`
- `bash scripts/verify_platform_security.sh`

## Still requires field/authoritative-lane evidence

- Windows MSVC clippy/check/test and the desktop UPnP soak.
- Android/Windows/iOS synchronized five-node G1-G6 run on one SHA, twice.
- Fresh synchronized Android and Windows logs; older handoff logs predate the current PR head and are correlation context only.
- iOS device installation remains dependent on Apple signing/provisioning availability.

No release tag or field result is claimed by this record.
