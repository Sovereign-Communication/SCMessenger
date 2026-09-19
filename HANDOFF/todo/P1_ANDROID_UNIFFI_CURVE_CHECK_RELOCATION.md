# P1 — Relocate Android public-key curve validation behind UniFFI (bod-dd336324 remedy)

- **Priority:** P1 — must land before the v0.4.0 tag
- **Filed:** 2026-09-13, Buffy (Freebuff recovery session)
- **Doctrine basis:** Board resolution bod-dd336324 — "Kotlin-level BigInteger
  Ed25519 curve decompression and Legendre symbol arithmetic are rejected in
  favor of IronCore identity resolution via UniFFI." Rust core is the sole
  cryptographic authority; Android adapters are dumb byte pipes.
- **Status:** OPEN

## Verified facts (commands cited, run 2026-09-13)

1. `git show 956ec371:android/app/src/main/java/com/scmessenger/android/security/PeerIdValidator.kt`
   shows `isValidEd25519Point` (Kotlin BigInteger decompression +
   `x2.modPow(...)` Legendre check) **pre-exists on origin/main** — lines
   78-99 at the PR #281 merge commit.
2. `git diff 956ec371 d35d3883 -- ...PeerIdValidator.kt` shows PR #282 adds
   ONLY helper validators (`isBlePeerId`, `isTransportPeerId`,
   `isIdentityHash`, `isPublicKeyHex`), docs, and tests. **No new BigInteger
   or curve-operation lines.** The PR does not worsen the debt.
3. Panels bod-40a0e31f / bod-2522bb06 / bod-bbb49423 all treated the
   pre-existing debt as out of scope for the merge vote; this ticket is the
   execution vehicle ordered by bod-dd336324.

## Scope

- Move the validity check for Ed25519 public-key hex strings behind a
  UniFFI-exposed core function (e.g. `is_valid_public_key` on the mobile
  bridge surface; verify the exact exposed symbol in `core/src/mobile_bridge/`
  during implementation — do not assume it exists).
- Kotlin keeps only: hex-format sanity checks (`isPublicKeyHex`) and length
  checks. All curve math (decompress, Legendre, point-on-curve) is deleted
  from Kotlin once the core call is wired and its call sites are reachable
  (wire it or it is dead — rule 16; run `scripts/check_wiring.py`).
- Update/extend the PeerIdValidator unit tests to cover the boundary cases
  the old Kotlin code covered (identity hash, transport id, malformed hex,
  valid point, invalid point).
- Platforms: Android first (that is where the debt lives); audit iOS/WASM for
  the same pattern while in the file family.

## Acceptance

- `grep -rn "modPow\|BigInteger" android/app/src/main/java/com/scmessenger/android/security/`
  returns zero curve-math hits.

## Scope correction (2026-09-14, passive audit of PR #288)

The debt is THREE copies, not one. Live grep at branch tip (post-`cc3508f5`)
found Kotlin Ed25519 decompression + Legendre/modPow math in:

1. `utils/PeerIdValidator.kt:112` (`isValidEd25519Point`) -- the copy this
   ticket originally scoped (`security/` in the filed evidence; the file
   moved/was duplicated into `utils/` at some lineage). Acceptance grep above
   misses this location -- extend the grep to all of
   `com/scmessenger/android/{security,utils}/`.
2. `ui/viewmodels/DashboardViewModel.kt:41` (`isValidEd25519Point` + private
   ED25519_P/D/SQRT_M1/PLUS3_OVER8 constants, lines 28-38).
3. `ui/viewmodels/ContactsViewModel.kt:38` (`isValidEd25519PointContact` +
   private CONTACT_ED25519_* constants, lines 25-34).

All three call sites must route through the UniFFI check; per-copy duplication
of the curve constants is the defect pattern rule 3 (IronCore sole crypto
authority) exists to kill. Updated acceptance: zero `modPow|BigInteger` curve
hits under `com/scmessenger/android/` outside base58 encoding (PeerKeyUtils.kt
uses BigInteger for base58 only -- that usage is fine and stays).
- The UniFFI function is registered in the bindings surface and called from
  the validator path (evidence: call-site grep + check_wiring.py green).
- Adversarial review on file for the core-side exposure (rule 8: the UniFFI
  surface addition touches core/) before merge.
