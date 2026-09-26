# SCMessenger owner handoff — Jev-only audit, #361 Android gate root cause, and proof-gap findings

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This is an assist-only handoff. The owning repository retains all decisions, edits, merges, and publication authority.

## Pinned release state

- Validated `main` at audit time: `d1c4a173835e7a90e117e2ad69e23953c323b8d5`.
- `main` has since advanced by exactly one commit, `7aa3a239b783fb5fd7b390fc3fbd7b0207ffdbe2` ("docs(freebuff): record the canonical JEV gate evidence and file the instrument ruling (#354)"), which adds 4 documentation files and changes **no product code**. Every product finding below holds unchanged at the new head.
- Audited tree: the pull-request head `a76d7d66d3c29bde722931847fceb4c5311b68d4` (3,438 tracked files) in a read-only detached checkout. No product worktree was modified and no branch was created in this repository.

## Audit method and receipts

- Two complete whole-tree element audits were run against the tree using native Jev over the network, with the dedicated per-call key. No secondary model provider was used at any point.
- Pass 1, the stock operator summary pack shipped with the auditing tool (`repo-summary` v1, axes `stage` / `brief_treatment` / `handling`): **3,638 of 3,638 candidates judged** (3,438 files + 200 symbols), 3,419 live, 219 fallback, 3,341,173 input tokens, $0.014033, `stop_reason=complete`.
- Pass 2, operator pack `scmessenger-native-audit-v1` (authored for this audit, product-surface / change-risk / proof axes): **3,638 of 3,638 judged**, 3,524 live, 114 fallback, 4,017,841 input tokens, $0.016875, `stop_reason=complete`.
- A log-triage pass over the failing Android CI job judged 51 extracted log items, 3 live, 48 fallback.
- Every native call is recorded in the hash-chained ledger with the observed model `jev-1.13.0` and `is_fallback=false`. Recomputed cost from the ledger matches billed cost to the cent-of-a-cent at the published rate of $0.0042 per million input tokens.
- **4.6% of all billed calls (333 of 7,279) returned a response the operator could not parse and were discarded after billing** ($0.001369, 4.4% of spend). This is an instrument defect and is documented in the operator's own repository, not here. It means the live-coverage figures above are an upper bound on usable signal, not a guarantee of classification quality.
- The 3,638 judgments include an `ambiguous` flag set whenever a distribution confidence falls below 0.5. 1,087 rows (29.9%) carry it. Ambiguous rows must not be read as settled classifications.

## Finding 1 — pull request #361's Android JVM gate fails because of two tests it adds that cannot pass (highest confidence; root cause verified, not inferred)

- **Symptom.** Pull request #361 (head `63f4a7d73ef05ace437b54d11c14a256c71eb923`) is the only open pull request with a red check: `Android JVM Unit Tests`, `BUILD FAILED in 20m 12s`. Its other 32 checks pass, including `Android Debug APK` and all four ABI builds.
- **Isolated failure.** Exactly two tests fail, both in `android/app/src/test/java/com/scmessenger/android/ui/viewmodels/MeshServiceViewModelTest.kt`:
  - `toggle during STARTING sends stop instead of becoming a no-op`
  - `toggle during STOPPING sends explicit start instead of becoming a no-op`
  Both fail identically with `java.lang.AssertionError: Verification failed ... Only one matching call to Context(#...)/startService(matcher<Intent>()) happened, but argument` and the recorded argument `null`.
- **Root cause, verified against the build configuration rather than taken from any commit message.** `android/app/build.gradle:232-234` sets `unitTests { returnDefaultValues = true }`, and the same file documents the consequence in its own words at line 308: "the android.jar stubs no-op under returnDefaultValues". `org.robolectric` is **not** a dependency anywhere in the file; it appears only in an explanatory comment at lines 315-319 recording its removal on 2026-07-27. Under those conditions `android.content.Intent`'s `action` getter is a stub that always returns `null`.
- **Therefore the assertion is unsatisfiable by construction.** Both tests assert `it.action == MeshForegroundService.ACTION_STOP` and `it.action == MeshForegroundService.ACTION_START`. A stub that always returns `null` can never equal either constant, so no code change could make these tests pass. The `null` argument in the CI failure output is the direct observable evidence of exactly that.
- **Not present on the green baseline.** The test file exists on `main`, where it declares 7 `@Test` methods and the Android gate is green. The diff `d1c4a17..63f4a7d7` for this file adds **only** these two tests plus one import. The regression is introduced by the pull request itself, not inherited.
- **The production code is correct and needs no change.** `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/MeshServiceViewModel.kt` builds a real intent with the correct action and calls the matching context entry point: line 87-91 constructs `Intent(context, MeshForegroundService::class.java)` with `action = ACTION_START` and calls `context.startForegroundService(intent)`; lines 106-109 do the same with `ACTION_STOP` and call `context.startService(intent)`. The defect is entirely in the test's observability assumption.
- **A correct fix already exists inside the repository.** Commit `e7466639b59f6c07f19090ecd9867b39ac36b58e` (pull request #372) replaces the action matcher with the context entry point and adds a negative assertion that preserves the start/stop discrimination: `verify(exactly = 1) { mockContext.startService(any<Intent>()) }` together with `verify(exactly = 0) { mockContext.startForegroundService(any<Intent>()) }`, and the mirror for the other direction. Its stated rationale matches the build-configuration evidence above independently.
- **Honest limitation of the proposed fix.** `any<Intent>()` in MockK matches a null argument, so the repaired test proves which context entry point was invoked and that the other was not, but no longer proves the action constant. That is a deliberate, correct trade for a JVM unit test in this module; the action value is covered by the real intent construction on the production path.

**Requested owner action for Finding 1**

1. Treat the Android JVM failure as a test-observability defect owned by pull request #361, not a production regression. No product code change is warranted.
2. Land the test-file hunk of `e7466639` (or an equivalent test-only correction) so #361's Android gate can go green, and re-run `Android JVM Unit Tests` as the acceptance signal.
3. Do not merge all of #372 to achieve this. Cherry-pick the test hunk alone so the unrelated transport, vendoring, and documentation changes in that pull request are not dragged along.
4. Correct the pull request #361 description if it claims the Android gate is expected to be green.

## Finding 2 — automated proof is absent across almost the entire product surface

- Judged across 3,638 elements, the `proof` axis places **2,010 of 2,320 documentation elements and 186 of 229 core Rust elements (81%), 153 of 191 Android app elements (80%), 24 of 35 FFI binding elements, and 323 of 336 CLI/headless/bridge elements** at `no_automated_proof`.
- Across the whole core Rust surface exactly **one** file is judged as covered by a unit test, and one by a CI workflow gate. The Android JVM gate is judged to cover only 2 elements.
- `audit_priority` concentrates the real risk in a narrow set: 125 of 229 core Rust elements and 176 of 336 CLI/headless/bridge elements are rated `p1` (central to correctness, transport, or the FFI and wire contract). The highest-value files by centrality score are `core/src/transport/manager.rs`, `core/src/transport/dial_policy.rs`, `core/src/transport/mesh_routing.rs`, `core/src/message/codec.rs`, `core/src/relay/protocol.rs`, `core/src/relay/client.rs`, `core/src/relay/server.rs`, `core/src/crypto/ratchet.rs`, `core/src/crypto/encrypt.rs`, `core/src/drift/frame.rs`, `core/src/wasm_support/transport.rs`, `cli/src/ble_mesh.rs`, and the Android mesh service and view-model files. All are rated `concurrency_or_transport` change risk except the codec, protocol, encryption, and desktop bridge files.
- `p0` (currently failing a gate) is concentrated in `build_ci_infra` at 34 elements and `cli_desktop_headless` at 4.
- **Caveat on method.** The element text given to the model is a bounded signature — path, kind, module summary, up to 18 symbols, up to 12 headings, truncated to 1,200 characters — and symbols are extracted only for Python files. This repository is Rust, Kotlin, and Swift, so the model largely reasoned over paths and summaries rather than parsed source. Treat the rankings as a triage ordering that is cheap to produce and safe to discard, not as a substitute for reading the files. The `no_automated_proof` signal in particular is partly an artefact of that thin input and should be confirmed by a test-coverage run before any remediation is scoped.

## Finding 3 — the repository is majority documentation by file count

- On current `main` (`7aa3a239`), **1,824 of 3,391 tracked files (53.8%) live under `HANDOFF/`** and **2,083 (61.4%) are Markdown**. Product code is a minority of the tree: 1,099 files are non-Markdown and outside `HANDOFF/`.
- Consequence for review capacity: 2,320 of 3,638 judged elements (63.8%) were classified `docs_and_handoff`, and 2,310 of those carry no automated proof. Any whole-tree review spends most of its budget on documents.
- Recommendation: scope future element audits to product directories (`core/`, `android/`, `iOS/`, `cli/`, `wasm/`, `headless/`, `desktop_bridge/`, `mobile/`, `ui/`, `shared/`, `.github/`) so review attention tracks product risk. Note that the audited tool's only exclusion flag covers its own output artifacts, so this scoping must be done by the caller.

## Finding 4 — the #372 four-file transport fix is structurally sound on read-only inspection (no sign-off claimed)

- The F2 activity-leak fix is complete in both event loops. `note_established_path` seeds the activity stamp, appends to the peer's oldest-first list, computes the least-recently-active redundant set, removes those entries from **both** the path list and the activity map, and only then hands them to the close callback. The policy never touches the swarm.
- Every close arm reclaims. The partial-close arm at `core/src/transport/swarm.rs:7230` calls `release_path` and, in the `else` branch where the peer entry is absent, still removes the activity entry directly. The last-close arm at line 7300 removes the peer entry and calls `release_peer` to drain whatever ids it still carried. The wasm loop mirrors this at lines 10121 and 10176. No close arm returns before reclaiming.
- The admission ceiling of 8 is now derived from the actual synthesised dial ladder (3 direct TCP ports + 1 `last_good` address + the capped relay ladder) rather than asserted, and `admitted_fan_out` and `retained_after_trim` keep the simulation and the policy in one module so the two event loops cannot drift.
- **This is a read-only structural review only. It is not Rule-8 sign-off, and it must not be recorded as such.** The author must not self-sign, and this assist-only lane must not sign on the author's behalf. An uninvolved reviewer must still examine the connection-cap leak, the WASM parity, the admission sizing, the JVM regression coverage, and the preservation of the existing security and privacy boundaries against current `main` and the security protocol. A green build is necessary but not sufficient.

## Requested owner action

1. Apply Finding 1 first: it is a single green check that currently blocks a merge candidate, and the fix already exists in the repository.
2. Treat Findings 2 and 3 as scoping input for the owner's own review capacity, not as a work order. Confirm the proof gap with a real coverage run before scoping any remediation.
3. Obtain the independent Rule-8 review that Finding 4 explicitly does not substitute for, and keep any fix minimal and owner-scoped in a clean branch from the current owner base.
4. Preserve the pinned release binary and image identity until the owner accepts the deployment state. Any later deployment must be reversible and must retain a rollback receipt.

## Stopping condition

Stop after one independent owner verdict. A CI result, a model judgment, a staged binary, or a service-health response is not a substitute for the required owner review and sign-off.

## Handoff boundary

No repository changes are requested by this document. Findings outside this repository must be split into their own owner-scoped handoff before transfer.
