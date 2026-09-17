# Blind B — independent verdict: D2 build-number bump + keystore verification path

**Date:** 2026-09-13
**Reviewer:** Buffy (Freebuff lane) — same-lane disclosure per this file
series' tradition; the external BoD panel is the gate of record.
**Change reviewed:** 3-file/6-line build-number bump (Android versionCode 15,
iOS CFBundleVersion/CURRENT_PROJECT_VERSION 10) on
`feat/d2-keystore-verify-20260913`, enabling the release pipeline's
candidate-artifacts mode.
**Blind A of record:** APPROVED (see HANDOFF/BOD_STATE.md, D2 bump row),
$0.0031.

## Verdict: APPROVE

1. **The bump is the minimal enabler, proven by failure.** Dispatch
   34789843685 was refused by `scripts/verify_versions.sh` at
   "Android versionCode 14 is not greater than prior tagged build 14" — the
   gate is mechanical, and I reproduced it locally before changing anything.
   After the bump the same script prints [OK] on both lines. No smaller
   change exists: the gate demands strictly-greater build numbers.
2. **No version-scheme drift.** Marketing version stays 0.4.0 on all five
   surfaces (Cargo workspace, Android versionName, desktop, WASM, iOS). The
   "0.4.9 vs 0.4.0" question remains the operator's tag decision; this
   change does not prejudge it. Build numbers are internal monotonicity
   counters, incremented exactly when a build must supersede an older
   tagged one — which is what the D2 verification build is.
3. **iOS consistency enforced, not assumed.** Info.plist CFBundleVersion and
   the xcodeproj CURRENT_PROJECT_VERSION (4 build settings) moved in
   lockstep; the verifier cross-checks them and passed.
4. **No behavioral surface.** versionCode/versionName feed the APK manifest
   and nothing else; the verifier's other agreement checks still pass. The
   diff contains no logic.

## What I did NOT approve, and what stays operator-gated

- Tagging v0.4.0 (or 0.4.x), publishing anything, and the operator-scored
  gates (churn re-mesh, D4 on released APK, D6 fallback, D7 proximity) are
  untouched by this change and remain the operator's call.
- The dead Windows BT radio decision remains open.
- The keystore's store/key passwords remain operator-held; this lane never
  held them and the D2 verification is designed to conclude via CI secrets,
  which is exactly what the re-dispatch will test.

## Disposition

Merge the bump, re-dispatch `release.yml` `workflow_dispatch
artifacts_only=true` on the merged main, and let the pipeline's own preflight
adjudicate the D2 alias question. Record the run ID in
`HANDOFF/audit/D2_KEYSTORE_VERIFICATION_2026-09-13.md` (already updated with
the first refusal).
