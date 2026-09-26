# GPT HANDOFF -- iOS parity lane (Mac-only work)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: READY FOR KICKOFF
Created: 2026-07-28 (orchestrator takeover audit)
Executor: GPT-5.6 Sol session on the operator's MacBook
Repo: Sovereign-Communication/SCMessenger (public GitHub; clone fresh:
`git clone https://github.com/Sovereign-Communication/SCMessenger.git`)
Priority: parallel to the v0.4.0 wave; iOS is EXCLUDED from the v0.4.0 tag
(operator decision 2026-07-28) but farm-gating for v1.0.0 (half the farm
carries iPhones). Nothing here blocks or is blocked by the Windows-side
v0.4.0 security wave.

## Rules of engagement (binding)

1. No emojis anywhere in code, commits, docs, or output (repo hook-enforced).
   Use plain tags: [OK], [FAIL], [BLOCKED].
2. Worker contract: first line of every response is RESULT: DONE |
   RESULT: BLOCKED: <reason> | RESULT: FAILED: <reason> | PATCH: <n>,
   then max 10 lines of summary.
3. UPDATED operator directive 2026-07-28: the Mac session MAY commit and
   push to its OWN branches (naming: gpt/<lane-or-task>) and may open and
   manage its own pull requests -- branch, commit, push, PR, and iterate
   on review feedback directly. Still reserved to the Windows
   orchestrator: merging PRs into main, moving HANDOFF ticket files
   between todo/in_progress/done, release tags, and anything touching
   core/ Rust (routes back through the AUDIT-GATE on this side).
4. Gate for every task: `xcodebuild` output pasted verbatim (build + test).
   A clean compile is NOT completion: grep your own diff for
   simulate|mock|placeholder|in a real implementation before declaring DONE.
5. Touch only the files each task names. If a task forces a new file, that
   is fine only where marked (NEW) below.

## Task 1 -- U6 iOS receipt unification (mirror of DONE Android A-04)

Core exposes unified `encode_receipt()` / `decode_receipt()`
(core/src/crypto/receipt.rs, core/src/store/receipt_store.rs; FFI entry
iron_core.rs `decode_receipt` at ~:2837, live classify path ~:3064 calling
`delegate.on_receipt_received`). The Android side is DONE -- read
HANDOFF/done/P4_ANDROID_RECEIPT_UNIFICATION.md and
android/app/src/test/java/com/scmessenger/android/test/ReceiptUnificationTest.kt
for the landed pattern.

- Locate Swift receipt handling: glob iOS/ for CoreDelegateImpl.swift and
  SmartTransportRouter.swift; replace any local Swift-side receipt
  encode/decode with the generated UniFFI bindings' encodeReceipt/
  decodeReceipt.
- (NEW) iOS test ReceiptUnificationTest.swift: round-trip a receipt through
  core encode -> decode; assert field equality; assert no local Swift
  encode/decode remains (grep evidence).
- Bindings: regenerate the XCFramework if the UDL surface requires it
  (core gen_swift bin, gen-bindings feature); iOS/ path convention is
  uppercase-I everywhere (CI hygiene gate enforces this).
- Gate: xcodebuild build + the new test PASS.

## Task 2 -- Swift relay de-hardcode + discarded-bootstrap bug

iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift:
- :129 `private static let defaultBootstrapRelay =
  "/ip4/100.56.248.69/tcp/9001"` -- DELETE. Source bootstrap from the
  ledger (getPreferredRelays / dialableAddresses), mirroring the landed
  Android pattern (MeshRepository.kt getBootstrapNodesForSettings /
  ensureBootstrapRelayConnected; Android hardcode removed in commit
  f010a0f1 -- `git show f010a0f1` for reference).
- KNOWN BUG (2026-07-25 audit): computed bootstrapAddrs (~:848) is
  DISCARDED -- startSwarm receives an empty array (~:1062) while the log
  prints the unused count. Wire the computed addresses into startSwarm.
- Gate: xcodebuild build PASS + a boot smoke log showing startSwarm
  receiving non-empty bootstrap when a relay is configured.

## Task 3 -- D-03 XCTest target registration

HANDOFF/todo/D-03_iOS_XCTest_target_register_SC.md: register
SCMessengerTests in the .xcodeproj so `xcodebuild test -project
iOS/SCMessenger.xcodeproj -scheme SCMessengerTests` runs. The GitHub
macOS CI lane (ios-build-test.yml) exists but shows cancelled on main --
once Task 3 lands on a branch, the Windows orchestrator will align the
workflow and re-run it.

## Output expected back to the orchestrator

Per task: branch name or diff, xcodebuild evidence, files touched, and any
UniFFI/UDL surface changes (these need a Windows-side FFI snapshot
re-check). Flag anything that required touching core/ Rust -- that routes
back through the Windows AUDIT-GATE, not the Mac session.
