
## MERGE 4 — PR #278 — 2026-09-07 ~10:15Z

- CEO ruling: 10:07Z via coordinator channel (COORD_CHECKINS entry 20,
  explicit ask_questions confirmation relayed by operator).
- PR #278 squash-merged into cto/v040-candidate-2026-09-02:
  merge commit 85cb4c67feb03d27fa004a2be6b1ce65b030eb06 (short 85cb4c67),
  squash of freebuff/v040-android-mesh-resilience @ bfe6bc0b55de412acd35fb82f4b058f9dad7b31e.
- Tree proof: 85cb4c67^{tree} == bfe6bc0b^{tree} == 09896243 (squash preserved
  the exact reviewed/approved tree; zero novel content). b0f7ac4e is an
  ancestor (merge-base --is-ancestor exit 0).
- Delta vs prior tip b0f7ac4e: 9 files, +1771/-26 — Android Kotlin (D1-D3) +
  core/src/mobile_bridge.rs (FFI/ANR fix + R10-R18 hardening) + outbox/transport
  support. No crypto/routing/privacy files.
- Review basis: R1-R5 APPROVE (9a0bc715) + ANR chain R10-R18 closing in plain
  APPROVE at bfe6bc0b (verdict file
  V040_PR278_ANDROID_ANR_REVIEW_APPROVED_bfe6bc0b_2026-09-07.md).
- CI at bfe6bc0b: 7/7 COMPLETED, mergeStateStatus CLEAN pre-merge.
- Live evidence at approved head: APK 039f5116 installed on Pixel, zero
  ANR/crash post-install, FGS isForeground=true (check-in 4, coordinator-
  verified on-device).
- Post-merge sequence per CEO ruling: re-pin #272 verdicts at 85cb4c67 ->
  final-tree re-validation (live-message test included) -> tag-decision
  report. No tag, no release, origin/main untouched.
