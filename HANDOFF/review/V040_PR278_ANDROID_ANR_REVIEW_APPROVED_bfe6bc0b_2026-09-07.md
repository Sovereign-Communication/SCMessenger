# V040 PR #278 RULE-8 REVIEW CHAIN CLOSED -- PLAIN APPROVE (R18)

Date: 2026-09-07 (~09:05Z session clock)
PR: #278 (Android inbound-display + FFI/ANR fix chain, branch freebuff/v040-android-mesh-resilience)
Approved head: bfe6bc0b ("R17: owner-tagged lifecycle stashes block cross-thread echo clobber")
Prior reviewed head: 3a4ec3b8 (R16)
Reviewer lane: qwen3.8-2.4t-a95b (non-author, Rule-8); dispatcher --record default-on, usage_src=api

## Scope of the chain

Branch delta vs candidate base e97c3f82 (through bfe6bc0b): the Android FFI/ANR
root-cause fix (RCA-CONFIRMED-DROP-HOP-2026-09-04-2020Z.md: Kotlin re-entrant FFI
echo deadlocking the non-reentrant platform_bridge mutex -> ANR/crash) plus the
D1-D3 Android findings from the RCA rerun, all in android/ + core/src/mobile_bridge.rs.

## Review rounds (all non-author, dispatched via tmp/qwen_review_dispatch.py)

- R10 (257610f7 fix): 5 findings -> fixed (generation counter, restore-to-slot
  ownership, detached-state replay, Kotlin echo removal, expanded tests).
- R11 (43b2af02 fix): 5 findings -> fixed (atomic dispatch_bridge_event owner,
  coalescer with capped iterative drain, proximity-path visibility).
- R12 (ad40aec0 fix): 5 findings -> fixed (fixed-point drain owner + non-reentrant
  budget guard, depth-in-CS, thread-local RAII marker, re-stash on WindowOpen).
- R13 (5eb3dd03 fix): 5 findings -> fixed (RAII budget guard, notify()-based drain
  delivery, global in-flight marker, post-release recheck, cap-exit contract).
- R14 (6f5d9d90 fix): F1 CAS-acquire regression fixed; F2 TOCTOU closed;
  F3 refuted with evidence (marker keyed (event, thread) makes the cited
  scenario a stash, not a drop); F4/F5 documented contracts.
- R15 (e8e04fb7 fix): real correlation kept -- marker keyed (event, dispatching
  thread); external same-variant event from another thread stashed, never
  dropped (regression test added). Test-serial lock added after a one-off
  cross-test flake (4 consecutive greens).
- R16 (3a4ec3b8 fix): stash-aware echo suppression (overwrite only a different
  stashed event; pending==None = pure echo). Same-thread chain regression added.
- R17 (bfe6bc0b fix, this approval): F1 lost-state cell closed -- StashedLifecycle
  { event, owner: ThreadId }; echo may overwrite only a same-thread stash;
  cross-thread stash (newer external state) survives. F2 drain-span contract
  documented in the marker doc block. F3 regression
  cross_thread_stash_survives_same_thread_echo added.
- R18 verdict: APPROVE (tmp/rev278_r18_response.md, ledger line
  2026-09-07T09:05:29Z, in=4576 out=15322 usage_src=api).

## R18 residual findings (non-blocking, recorded as follow-ups)

1. Low: ThreadId owner is a physical-thread token, not a logical causal token.
   Sound under Android's thread-affine FFI; if cross-thread marshalling is ever
   introduced, augment owner with a monotonic dispatch/generation token. The
   degradation path is safe today (unmatched echoes stash and deliver once).
2. Low (test gap): reviewer asked for "external same-variant event during
   in-flight dispatch -> exactly one re-delivery". ALREADY COVERED by
   external_same_variant_event_from_another_thread_is_not_lost (asserts the
   stashed external Background is delivered exactly once, total 2 callbacks,
   no redelivery loop). No new test needed; noted for the record.
3. Info: optional debug_assert! that NOTIFYING_LIFECYCLE is present during
   drained lifecycle dispatch. Recorded as optional hardening, not required.

## Gates at bfe6bc0b (all run this session)

- cargo check -p scmessenger-core --lib: PASS (29.78s)
- mobile_bridge tests: 44/0 (was 43; +1 regression)
- full core suite x3 consecutive: 1426/0 each (5 ignored, pre-existing)
- clippy -p scmessenger-core --lib: 0 code findings (known build-script E0602
  quirk on -A clippy::empty_line_after_docs, pre-existing toolchain issue; CI
  adjudicates the documented command)
- cargo fmt --check: PASS
- gradlew :app:compileDebugKotlin :app:testDebugUnitTest: BUILD SUCCESSFUL
  (13m41s; core .so relinked with the fix)

## Disposition

Rule-8 gate SATISFIED for PR #278 at bfe6bc0b: plain APPROVE from a non-author
reviewer after full disposition of every finding R10-R18 (fixed or evidence-backed
refute; none waived). Next: APK rebuild from the approved head, install on Pixel,
passive live confirmation of the ANR signature (user pre-authorized the
fix-and-deploy loop). Merge of #278 remains a CEO decision per prior standing
directives.
