# V040-T-AND06-A1 — Collapse redundant Kotlin Ed25519 validator copies

Status: OPEN (filed 2026-09-20; operator unification ruling)
Priority: P1 -- Wave 1 step 2 (A-lite), before UniFFI cutover
Lane: Freebuff
Scope: `android/app/src/main/java/com/scmessenger/android/**` validator and
call sites only. Do not change core/UniFFI in this PR. Do not delete the
curve math yet -- that is ticket A2.

## Operator ruling

"Unification -- collapse copies if redundant, then finish it. Do it right."
Interview 2026-09-20. See `HANDOFF/V040_WORKING_FIRST_PATH_2026-09-20.md`.

## The defect

Multiple Android copies of Ed25519 point validation + constants exist
(`PeerIdValidator.kt`, `DashboardViewModel.kt`, `ContactsViewModel.kt` family).
Doctrine: IronCore is the sole crypto authority; adapters are dumb byte pipes.
Duplication is the debt A2 will delete; this step makes that deletion safe.

## Work

1. Inventory every `isValidEd25519Point*` / ED25519_P / SQRT_M1 / related
   constant under `com/scmessenger/android/`.
2. Route all call sites through **one** Kotlin validator facade (single module
   path -- e.g. `PeerIdValidator` or a clearly named security util).
3. Delete duplicate constant blocks and duplicate functions in ViewModels;
   keep behaviour identical.
4. Update unit tests to the single path.
5. Run `python scripts/check_wiring.py` -- must stay green.
6. Acceptance grep (still allowed to hit the **one** remaining implementation):

```
rg -n "modPow|BigInteger" android/app/src/main/java/com/scmessenger/android --glob '!**/PeerKeyUtils.kt'
```

Expect at most the single facade copy; zero ViewModel private copies.
(`PeerKeyUtils.kt` base58 BigInteger is fine.)

## Scope correction

- Do not wire UniFFI yet (A2).
- Do not touch `core/`.
- Do not "improve" curve parameters.

## Acceptance

1. Single Kotlin implementation path; call sites updated.
2. Wiring gate green.
3. Unit tests green on Android JVM lane (CI).
4. PR evidence: grep output + test command.

## Review gate

None if android-only. If a reviewer finds a core touch, stop -- that is A2.

## Rules

No emojis. Evidence contract. Shared checkout scope only.
