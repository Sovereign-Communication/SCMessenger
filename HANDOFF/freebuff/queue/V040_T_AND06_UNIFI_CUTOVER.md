# V040-T-AND06-A2 — UniFFI cutover for public-key validation

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: OPEN (filed 2026-09-20; run AFTER A1 merges)
Priority: P1 -- Wave 1 step 3
Lane: Freebuff
Scope: Android call sites + any remaining FFI surface registration gaps.
Core export `is_valid_public_key` is already on main via PR #305 -- verify,
do not reinvent.

Depends on: `V040_T_AND06_KOTLIN_COLLAPSE.md` merged (one Kotlin path).

## Operator ruling

Full unification: finish UniFFI cutover; delete remaining curve math. Do it
right. `HANDOFF/V040_WORKING_FIRST_PATH_2026-09-20.md`.

## Premise (verify before coding)

On `origin/main`:

- Core `is_valid_public_key` exists (exported for adapters).
- Kotlin still may call local curve decompression after A1.

If the core symbol is missing or not reachable from the Android bridge, STOP
and write `inbox/` PREMISE-WRONG -- do not reimplement crypto in Kotlin.

## Work

1. Confirm UniFFI binding exposes the core validator to Kotlin.
2. Replace the single remaining Kotlin curve implementation body with a call
   to the core function. Keep hex-format/length checks in Kotlin.
3. Delete unused constants and curve code.
4. Extend tests: valid point, invalid point, malformed hex, identity-hash
   shapes the old tests covered.
5. `scripts/check_wiring.py` green.
6. Acceptance grep:

```
rg -n "modPow|isValidEd25519Point" android/app/src/main/java/com/scmessenger/android
```

Must return **no curve-math hits** (base58-only BigInteger in PeerKeyUtils may
remain).

## Scope correction

- Do not loosen validation relative to strict canonical decode fixed in #305
  (N-08). Match core behaviour.
- Do not edit generated UniFFI bindings by hand -- regenerate if snapshots
  drift.
- iOS/WASM: audit for the same pattern; fix only if cheap and in-scope.

## Acceptance

1. Grep clean as above.
2. Call sites reachable (wiring gate).
3. CI Android unit tests green.
4. **Rule-8 APPROVE on file** if core/`api.udl`/FFI snapshots change.

## Review gate

Rule-8 when core or FFI surface is touched. Android-only call-site rewires
still get orchestrator review via PR scope.

## Rules

No emojis. Evidence contract. No `unwrap()` in production paths.
