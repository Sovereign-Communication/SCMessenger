# PR139 Prime Orchestrate Restart Handoff - 2026-08-12

## Status and objective

Decision: **HARD NO-GO**. The objective remains active and must not be marked complete: complete the PR139/Sol plan across Android, iOS, macOS CLI, Windows, and cloud nodes; establish authoritative identity and nickname handling; deliver a durable two-sender responder for Lucas/Android and Christy/iOS; obtain receiver-backed evidence; prove five-node parity and G1-G6; and complete the 60-minute soak.

## Live cutover prohibition

No live deployment or responder activation, contact or device mutation, reply or send, launchd or service reload, or delivery claim is permitted until every listed hard gate and independent receiver evidence pass.

No live deployment, responder activation, contact/device mutation, reply/send, launchd/service reload, or delivery claim is allowed until all listed hard gates and independent evidence pass. This prohibition includes responder installation or alteration and remains in force until exact sender mapping, durable claim/lease/completion, idempotent Prime execution/completion, a single-send boundary, rollback, and receiver-backed evidence are independently verified.

## Immutable provenance anchors

Keep source and runtime/artifact evidence separate. Do not infer one from the other.

Every 64-hex value in this handoff is explicitly labeled as a Git commit, artifact digest, report digest, or checkpoint digest; no identity public key, private key, device UUID, PIN, message body, or secret is embedded.

- Source anchor: `ab4f448635ae7bca0592bf3f615fa818eeb765fc`.
- Runtime/artifact anchor: `9f54b1078ad512c895b68029c9e79a1870d7f286`.
- Receipt anchor: `73444f894f09de564159206b45332965daef6d6e`.
- Style anchor: `7538e4e99a76c34855ec8424438a4d7cb41d837`.
- Earlier recorded review checkout: `a29e53f384e038c1e35ee4e4f18972a008af5436`.
- The recently reported live runtime was `e7ac25c4...` on `gpt/pr139-receipt-filter-20260811`. This is separate, transient evidence and must be re-read fresh.

Preserve these dirty paths exactly; never revert, stash, or delete them:

- `HANDOFF/gpt/MAC_WINDOWS_BLE_PARITY_QUEUE_2026-08-11.md`
- `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift`
- `HANDOFF_AUDIT/turbofieldfare-audit/`
- `scripts/run_triplepass_turbofieldfare.py`

## Bounded artifact evidence

These are artifact-local findings, not live deployment or receiver evidence.

- Sol gate artifacts: `tmp/sol-pr139-completion-gate-20260812.md` and `tmp/sol-pr139-identity-parity-gate-20260812.md`. Their decision is **HARD NO-GO**.
- Isolated Android drafts: `tmp/android-boot-fix-20260812` and `tmp/terra-android-pr139-20260812`. They are undeployed and unverified.
- Isolated CLI ID/cursor drafts: `tmp/cli-history-id-contract-20260812` and `tmp/terra-history-cursor-pr139-20260812`. Storage lacks snapshot semantics.
- Tested ledger draft: `tmp/terra-responder-hardening-20260812`. Nine Python 3.9 tests passed; it is not deployed.
- New two-sender MVP: `tmp/pr139-mvp-responder-20260812`. Thirteen Python 3.9 tests passed; it is not deployed. Its checkpoint is `tmp/pr139-mvp-responder-20260812/CHECKPOINT.md`, checkpoint digest (SHA-256) `287b18e4620af283b362505e926e4b09ddd6d7d89f751473711769869858c8dc`. `git diff --check` remains the next action.
- Independent acceptance stopped because the interface did not yet exist: `tmp/pr139-mvp-acceptance-20260812/COMPATIBILITY_REPORT.md`, report digest (SHA-256) `d8e321e4e10809005ed50f31d077c49fe4529600fe1476f0332bae4c1527aa36`. Rerun it against the now-created MVP.

## Identity, device, and authentication boundaries

Contact-key evidence is metadata-only: the existing `Android` and `ChristyLove` public keys each derive exactly to their stored libp2p peer IDs. Do not embed the keys. The current received-history sender token from the bounded metadata audit mapped to neither pinned contact. Authenticated event routing therefore remains blocked and must fail closed.

Device-ID evidence is also metadata-only: the Android UUID was valid and uniquely paired in local identity metadata; the iOS UUID was unavailable. Do not embed UUIDs. Signed registration binds a UUIDv4 to an Ed25519 identity, but lifecycle propagation and projection remain incomplete. A device UUID never authorizes key rotation.

Critical authentication-boundary follow-up: `IronCore::receive_message` derives the authenticated sender identity/key but currently delegates plaintext `message.sender_id` in both delegate arguments. Android/iOS device-hint verification is therefore not envelope-key safe. Any transport patch requires adversarial security review.

## Live wake counters

The bounded live wake audit observed 8 history API outages, 442 bridge-unavailable warnings, 11 accepted, 10 queued, and 1 delivered. Admission-only boundary: all five counters are wake-pipeline admission or bridge observations, not proof of Prime execution or completion, SCM receiver delivery, acknowledgement, or live readiness. Prime agent-message queueing does not expose turn completion. None of these observations is delivery proof.

## Known blockers and secondary defects

Android blockers:

- Lifecycle recursion follows `MainActivity -> AndroidPlatformBridge -> MeshRepository pause/resume -> Core PlatformBridge callback`.
- `BootReceiver` launches a coroutine without `goAsync`.
- Auto-start defaults to false.
- `replay=0` creates startup-loss risk.

Mobile wake blockers:

- iOS Core ingress exists, but APNs entitlement, APNs registration, and remote-notification mode are absent; background tasks are opportunistic.
- Android has no FCM receiver.

Cloud blocker:

- There is no mobile-message wake endpoint. The startup script can generate a placeholder and conflicts with the tracked systemd `ExecStart`; remote deployment is unverified.

Secondary defects:

- `scripts/agent_monitor.sh` has an undefined `monitor_loop`, a launch-argument mismatch, and a `set -e` counter risk.
- Diagnostic scripts cover only adb/logcat.
- `.claude/skills/core_cli_driver.sh` hardcodes Windows Python.

## Worker and memory policy

Delegate 100% of project work to at most three concurrent direct workers; each worker has a hard RSS ceiling of 384 MiB and must be retired immediately when complete. Every dispatch states exactly one hypothesis, exact scope, canonical path or API request-shape anchor, acceptance test, stop condition, maximum tool calls, and RSS ceiling. Every bounded dispatch must contain exactly one hypothesis, an exact allowed scope with paths or endpoint, a canonical API, request-shape, or path anchor, an objective acceptance test, an explicit stop condition, permitted and forbidden tools, and a maximum tool-call count. Keep source evidence separate from runtime evidence.

## Fresh-session preflight

Before takeover, perform these steps in order:

1. Read `AGENTS.md`, `.claude/commands/orchestrate.md`, `docs/ORCHESTRATION.md`, this handoff, `HANDOFF/todo/_QUEUE.md`, and `HANDOFF/V1_0_0_EXECUTION_PLAN.md`.
2. Inspect the family and active agents before takeover, and confirm no prior orchestrator is still writing.
3. Re-measure HEAD/status, runtime `/version`, and process RSS read-only. Preserve the immutable source/runtime boundary.
4. Preserve the goal as active; do not mark it complete.
5. Preserve all listed dirty paths and do not revert, stash, or delete them.

## First three bounded delegations

### First dispatch 1 - MVP acceptance

- Hypothesis: the new isolated MVP satisfies the complete two-sender durability and single-send contract.
- Exact scope: only `tmp/pr139-mvp-responder-20260812` plus the isolated acceptance directory `tmp/pr139-mvp-acceptance-20260812`; no live endpoints.
- Canonical anchor: checkpoint `tmp/pr139-mvp-responder-20260812/CHECKPOINT.md` with checkpoint digest (SHA-256) `287b18e4620af283b362505e926e4b09ddd6d7d89f751473711769869858c8dc`, and compatibility report with report digest (SHA-256) `d8e321e4e10809005ed50f31d077c49fe4529600fe1476f0332bae4c1527aa36`.
- Acceptance test: validate all 13 Python 3.9 tests; run diff-check; validate parallel two-sender behavior, unknown/ambiguous fail-closed behavior, receipt/outbound filtering, claim/lease/reclaim, wrong-owner completion rejection, prompt-routing immutability, send retry without double-send, and crash recovery.
- Stop condition: stop on any live endpoint requirement, ambiguity, mutation outside scope, 384 MiB RSS, or 24 tool calls. Report PASS or BLOCKED without deployment.

### First dispatch 2 - History sender mapping

- Hypothesis: metadata-only history and contacts data can prove an exact authenticated mapping to the pinned `Android` or `ChristyLove` contact.
- Exact scope: only the metadata fields returned by the watcher's canonical `POST /api/history` request and `GET /api/contacts`; output no content and perform no mutation.
- Canonical request shape: the exact `POST /api/history` request used by the watcher, correlated read-only with `GET /api/contacts`.
- Acceptance test: prove an exact mapping to `Android` or `ChristyLove`; otherwise report BLOCKED and fail closed. Do not output public keys, UUIDs, PINs, or message bodies.
- Stop condition: stop on ambiguity, any mutation or content-output requirement, 384 MiB RSS, or 16 tool calls.

### First dispatch 3 - Prime execution and completion

- Hypothesis: the live bridge source and mock adapters are sufficient to define an idempotent completion contract with an enforceable single-send boundary.
- Exact scope: live bridge source and mock adapters only; make no live Prime or SCM call.
- Canonical contract: `claim -> running -> pinned send -> completed`, explicitly separating queue admission from execution completion and covering idempotency and crash windows.
- Acceptance test: specify and test ownership, state transitions, retries, crash recovery, and exactly-once-visible send behavior without treating admission as completion.
- Stop condition: stop on any need for a live Prime/SCM call, an unresolved double-send/crash window, 384 MiB RSS, or 18 tool calls. Report PASS or BLOCKED.

## Gate sequence

After the three delegations, require an independent Sol gate review before any deployment. Only after all hard gates pass may the authorized orchestrator stage a safe, reversible cutover followed by receiver proof, five-node parity, G1-G6, and the 60-minute soak. Until then the decision remains **HARD NO-GO**, the objective remains active, and no delivery or completion claim is permitted.
