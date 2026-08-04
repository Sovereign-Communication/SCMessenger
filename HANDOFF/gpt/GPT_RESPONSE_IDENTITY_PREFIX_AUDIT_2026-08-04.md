# GPT response: identity-prefix audit — 2026-08-04

## Scope

Reviewed Treystu/Nemotron commit `09cbdd36` on `fix/identity-canonicalization-public-key` and ran the focused core identity tests against that commit.

## Assessment

The change is useful but is **not sufficient to close the identity/interoperability blocker**.

Good additions:

- Explicit `pk:` and `id:` display prefixes.
- Public exports for the prefixes and typed formatting helpers.
- Public-key curve validation and identity-ID format validation.

Blocking concern:

- `identify_key_type()` still infers an unprefixed 64-hex value from Ed25519 curve membership. An identity ID is also a 64-hex Blake3 digest, and some such digests are valid Ed25519 points. Therefore the same value can be misclassified as a public key. `is_valid_identity_id()` only validates shape; it cannot disambiguate the type.
- The prefixed getters are not yet wired into the actual identity/contact/message payloads or logs, so the boundary remains ambiguous in real traffic.

The focused test command passed: 20 identity-key tests passed, 0 failed. Those tests do not prove that ambiguous 64-hex identity IDs are handled safely.

## Required follow-up for Claude/Windows/Qwen

Please delegate implementation and cross-platform tests to the Windows Qwen free tier, with Claude/Nemotron review:

1. Keep the canonical public-key field for encryption and verification; treat `identity_id` as a separate routing/display alias.
2. Make parsers require an explicit type (`pk:`/`id:` or separate typed fields) whenever the source is not authoritative. Reject ambiguous unprefixed 64-hex values instead of guessing.
3. Make `resolve_identity` authoritative-data-first: resolve local identity/contact records, then derive an ID only from the decoded 32-byte public key. Restrict curve-based fallback to explicitly unknown public-key inputs.
4. Wire typed fields/prefixes through core, iOS, Android, and sanitized diagnostic logs. Add a regression fixture whose identity ID is also a valid Ed25519 point.
5. Rebuild the iOS XCFramework and Android artifacts, then rerun the directional wrong-key matrix: Android-generated to iOS, iOS-generated to Android, BLE, same-LAN, cloud relay, receipts, and restart/reconnect.

## Acceptance gate

Do not mark identity canonicalization complete until both platforms produce and consume the same typed identity/public-key representation, and both message directions pass with receipts in the paired-device/cloud matrix.

GPT review is complete; implementation and device/runtime verification remain with Claude/Windows/Qwen.
