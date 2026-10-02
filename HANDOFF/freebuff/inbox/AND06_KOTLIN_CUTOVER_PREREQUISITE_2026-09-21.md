# AND-06 -- the Kotlin collapse is done; the UniFFI cutover has a prerequisite nobody has run

Status: OPEN -- evidence + recommended design, no code written
Filed: 2026-09-21 by the Freebuff lane
Tickets: `HANDOFF/freebuff/queue/V040_T_AND06_KOTLIN_COLLAPSE.md`,
`HANDOFF/freebuff/queue/V040_T_AND06_UNIFI_CUTOVER.md`,
`HANDOFF/todo/P1_ANDROID_UNIFFI_CURVE_CHECK_RELOCATION.md`
Operator ruling being executed: 2026-09-20 interview Q2/Q5 -- "1) collapse
redundant Kotlin copies; 2) full UniFFI cutover via `is_valid_public_key`;
delete remaining curve math."

## 1. Step 1 (collapse) is already done, and the ticket premise is stale

The relocation ticket's 2026-09-14 scope correction says the debt is THREE
copies (`utils/PeerIdValidator.kt`, `ui/viewmodels/DashboardViewModel.kt`,
`ui/viewmodels/ContactsViewModel.kt`). Re-run on `origin/main` today, the
acceptance grep has one curve-math file left:

```
$ grep -rn "modPow\|BigInteger" android/app/src/main/java/com/scmessenger/android/
utils/PeerIdValidator.kt   -- the curve math (P, D, SQRT_M1, P_PLUS3_OVER8, modPow)
utils/PeerKeyUtils.kt      -- base58 only (BigInteger for the alphabet), explicitly exempt
```

`PeerIdValidator.kt` itself records the consolidation in place: "Consolidates
the three previously divergent copies (this file, DashboardViewModel,
ContactsViewModel)." So a `KOTLIN_COLLAPSE` ticket pasted now would find no
second copy to collapse. It should be closed or rewritten as "cutover only".

## 2. Step 2 (UniFFI cutover) has a real, unrun prerequisite

The core-side symbol exists:

```
core/src/api.udl
  namespace api {
    // AND-06: Ed25519 public-key validity, so platform adapters (Kotlin, Swift)
    // never reimplement curve arithmetic.
    boolean is_valid_public_key(string hex_str);
  }
```

and `PeerIdValidator.kt` says the follow-up is "staged ... after the
binding-init audit (see ticket)". That audit is the prerequisite, and here is
the decisive check for it:

```
$ grep -rln "uniffi" android/app/src/test/ | wc -l     -> 18 test files reference the API
$ grep -rn "uniffi\.api\.[a-z]" android/app/src/test/  -> only inside COMMENTS
                                                          (ReceiptUnificationTest x2,
                                                           test/ReceiptUnificationTest x1)
```

So the 18 files use UniFFI **types** (records and enums, which are pure Kotlin
data classes) and mock the repository layer; **no JVM unit test calls a UniFFI
namespace function**. The only real calls live in main sources, e.g.
`MeshRepository.kt:11747 getBuildProvenance() -> uniffi.api.getBuildProvenance()`,
and `MeshRepositoryTest` mocks `MeshRepository` rather than calling through.

That matters because `PeerIdValidator.normalizePublicKeyHex` is currently pure
Kotlin and is exercised directly by JVM unit tests, including
`PeerIdValidatorCurveVectorTest.kt`, whose whole purpose is to pin the Kotlin
copy to Rust-derived vectors. UniFFI loads the native library lazily on the
first call into it. Point `normalizePublicKeyHex` at
`uniffi.api.isValidPublicKey(...)` and every JVM unit test that reaches it stops
being a pure-Kotlin test and starts needing `libscmessenger_core.so` on the test
classpath -- which is why the existing code kept the pure copy and called it
"the single fallback authority" for exactly that reason.

**Blind cutover would turn the Android JVM Unit Tests lane red on a code path
that gates identity validation, right before a tag.** That is a Mobile-lane
failure, not a cosmetic one, so it was not attempted.

## 3. Recommended design (small, and it meets the acceptance grep)

Two moves, in this order:

1. **Move the curve math out of `src/main`.** The Kotlin BigInteger
   implementation becomes a test-source oracle,
   `android/app/src/test/java/.../Ed25519ReferenceCurve.kt`. Main sources go to
   zero curve math, which is the ticket's acceptance criterion, and the existing
   vector test keeps its reference implementation to compare against.
2. **Route main through a one-line seam.** `PeerIdValidator` keeps the hex
   format and length checks in Kotlin (the ticket explicitly keeps those) and
   delegates point validity to a single injected/overridable function whose
   production body is `uniffi.api.isValidPublicKey(hex)` and whose test body is
   the oracle. That is the same seam shape the repository layer already uses:
   main code calls the FFI, tests substitute.

   A seam is required rather than optional: the alternative -- call the FFI
   directly from the validator -- is exactly what breaks the JVM lane, and
   "keep a Kotlin fallback and call the FFI first" leaves the curve math in
   `src/main`, failing the acceptance grep that exists to kill it.

   Note the seam must be *reachable*, not merely present (AGENTS rule 16):
   `normalizePublicKeyHex` and every caller of it are the wiring, and
   `python scripts/check_wiring.py` is the check.

3. **iOS is a separate audit.** The same relocation ticket asks to audit
   iOS/WASM for the pattern while in the file family. Not done here; no claim.

## 4. What the orchestrator should decide

1. Close or rewrite `V040_T_AND06_KOTLIN_COLLAPSE.md` -- its premise (three
   copies to collapse) no longer survives contact with the code.
2. Confirm the two-move design above for the cutover, or name a different one.
   The design choice is the gate: the code itself is small once the seam is
   agreed.
3. Add one acceptance row to the cutover ticket: **the Android JVM Unit Tests
   lane must stay green**, since that lane is the reason the pure Kotlin copy
   survived this long, and it is the lane a blind cutover breaks.

No code was written for this row. The blocker is a design decision plus a lane
that cannot be verified from this checkout, and per AGENTS rule 9 the shape of
the fix is not this lane's to pick.
