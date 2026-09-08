# V040 #272 VERDICT RE-PIN -- 85cb4c67 (post-#278-merge head)

Date: 2026-09-07 (~10:20Z session clock)
PR: #272 (architecture candidate, branch cto/v040-candidate-2026-09-02)
Head being re-pinned: 85cb4c67feb03d27fa004a2be6b1ce65b030eb06
Prior pinned head: b0f7ac4eb2551dd3d6b7d28a308c1765c824007e
Trigger: CEO ruling 10:07Z (COORD_CHECKINS entry 20) -- merge #278, then re-pin
#272 verdicts at the new tip.

## Why the head moved

PR #278 (Android inbound + FFI/ANR fix chain) squash-merged onto the candidate
line: merge commit 85cb4c67, squash of branch freebuff/v040-android-mesh-resilience
@ bfe6bc0b55de412acd35fb82f4b058f9dad7b31e.

## Ancestor + delta proofs (commands run this session)

- `git merge-base --is-ancestor b0f7ac4e 85cb4c67` -> exit 0 (prior pinned head
  IS an ancestor; nothing rewritten).
- `git rev-parse 85cb4c67^{tree}` == `git rev-parse bfe6bc0b^{tree}` ==
  09896243... (squash preserved the exact reviewed/approved tree -- zero novel
  content).
- `git diff --name-only b0f7ac4e..85cb4c67`: exactly 9 files, +1771/-26 --
  android/app/src/main/AndroidManifest.xml, MeshRepository.kt,
  AndroidPlatformBridge.kt, MeshForegroundService.kt, NetworkDetector.kt,
  MainActivity.kt, CircuitBreaker.kt, MeshForegroundServiceTest.kt,
  core/src/mobile_bridge.rs. All of it the PR #278 reviewed delta.
  No other file touched.

## Gated-directory check

The 9-file delta contains NO files under core/src/{crypto,transport,routing,privacy}/
(grep over the name-only diff returned nothing; core/src/mobile_bridge.rs is the
UniFFI bridge, not a gated module). The only transport-adjacent file on the branch
(transport/manager.rs, transport/swarm.rs, store/outbox.rs) entered via PR #276
and was already covered by its own R1-R14 chain and the b0f7ac4e re-pin; it is
NOT in this delta.

## Coverage argument

1. Every #272 verdict at b0f7ac4e remains valid for everything not in the
   9-file delta: the reviewed ancestors (e97c3f82 line: 3891d11c, 177bd840,
   44fee3c4, 48672b18) are still ancestors of 85cb4c67 (proven transitively:
   b0f7ac4e ancestor of 85cb4c67, and the e97c3f82->b0f7ac4e span was proven
   in the prior re-pin). The FINAL-APPROVE delta re-pin + FLAG-5 deferral
   APPROVE chain covers the pre-merge tree wholesale.
2. The 9-file delta is exactly the PR #278 change, which carries its own
   complete non-author Rule-8 record: R1-R5 (Android mesh-resilience, plain
   APPROVE at 9a0bc715) + ANR/FFI chain R10-R18 closing in a plain APPROVE at
   bfe6bc0b (tmp/rev278_r18_response.md, model qwen3.8-2.4t-a95b, usage_src=api,
   ledger line 2026-09-07T09:05:29Z; review record
   V040_PR278_ANDROID_ANR_REVIEW_APPROVED_bfe6bc0b_2026-09-07.md). No
   un-reviewed content exists anywhere in the b0f7ac4e..85cb4c67 span.
3. CI: all 7 checks COMPLETED at bfe6bc0b pre-merge, mergeStateStatus CLEAN
   (re-verified twice this session). The candidate line runs its own CI on the
   merged head.
4. Live evidence at the approved head: APK sha256 039f5116 (byte-identical to
   the bfe6bc0b build) installed on the Pixel; zero ANR, zero crash-buffer
   entries post-install; FGS isForeground=true; coordinator independently
   verified on-device (COORD_WATCH_ROUNDS ROUND 18).

## Disposition

RE-PINNED: the #272 Rule-8 record (qwen FINAL APPROVE + resolution delta
APPROVE + FLAG-5 deferral APPROVE, re-pinned b0f7ac4e) plus PR #274's, #276's,
and now #278's own chains constitute complete non-author review coverage of
85cb4c67. No finding was waived; every prior APPROVE transfers by ancestor
proof; the only new code is Rule-8 APPROVED on its own record.

## Post-re-pin status (per CEO ruling sequence)

Remaining before tag decision: final-tree re-validation at 85cb4c67
(live-message test on the re-onboarded Pixel + BLE + cellular legs), then the
tag-decision report. Old AWS instance termination remains a separate explicit
CEO go.
