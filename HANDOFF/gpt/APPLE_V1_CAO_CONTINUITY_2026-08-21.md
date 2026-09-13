# Apple v1.0 CAO continuity handoff

Status: ACTIVE - EMERGENCY CONTINUITY SNAPSHOT
Last updated: 2026-08-21

## Authority and safety boundary

This snapshot preserves the Apple lane state before an API reset. It authorizes
bounded work-ahead on isolated `gpt/*` branches and pull requests only. It does
not authorize merge to `main`, release/tag/version changes, HANDOFF queue moves,
core/Rust edits, or bypassing security, adversarial, device, or release gates.
Core/Rust changes require recorded CAO and CTO approval in addition to every
existing repository gate.

## Immutable Windows/CTO cursors

- CTO four-node kickoff: commit
  `3289fa5d15eb6b4e631e5830e477030886799e54`, merged by PR #202 into
  `upstream/main` at `4830305002f38b000f020cfaf0bb2bac41f3f7cc`.
- Tracked inbound path:
  `HANDOFF/gpt/WINDOWS_V040_V050_FOUR_NODE_PARITY_KICKOFF_2026-08-21.md`.
- Windows artifact PR #203 was still open at last observation. Its head had
  advanced to `bcfa931dfbf2d61886a9fc8c4085341640fbedc1`; earlier green checks do
  not apply to that newer head. Re-poll all checks before using it.
- Watch commits and immutable coordination IDs, not source-branch names. The
  PR #202 source branch was deleted after merge.

## Bilateral coordination contract

Coordination ID: `AW-BILAT-0001`.

The intended stable tracked paths are:

- `HANDOFF/coordination/apple-windows/INDEX.md`
- `HANDOFF/coordination/apple-windows/CAO_TO_CTO.md`
- `HANDOFF/coordination/apple-windows/CTO_TO_CAO.md`
- `HANDOFF/coordination/apple-windows/FOUR_NODE_GATE.md`

A corrected candidate exists locally from base `4830305002f38b000f020cfaf0bb2bac41f3f7cc`
with ordered manifest SHA-256
`41d6c1fa43ebc9362aa853c0d653e3cb7062137b5452b9e76709cd47ddc4b2d6`.
It passed rules, orchestration, schema, links, secret, emoji, terminology, and
whitespace checks. It has not received fresh independent validation and was
therefore deliberately excluded from this emergency commit. Resume by
independently validating that exact manifest, then forward-apply and publish it.

Every advisory/event must contain all 25 literal schema fields, including
`item_id` and `event_sequence`. CAO may file Android/Windows recommendations;
CTO may file iOS/macOS recommendations. Both lanes must acknowledge and record
disposition, evidence, branch, and exact commit.

## Four-node v0.4/v0.5 parity gate

Required topology: Windows CLI node, Pixel 6a Android node, macOS CLI node, and
physical iPhone iOS node. Each is a full node with store-and-forward custody.
Freeze exact commit/artifact hashes before testing. Execute the complete
M00-M20 matrix in both relevant directions, capture per-node timestamped logs,
redact secrets and message plaintext, and require joint CAO/CTO PASS. Simulator
evidence cannot replace physical Bluetooth, Multipeer/AWDL, APNs/background,
or four-node field evidence.

## iOS v0.5 implementation state

No app-source patch from this run is approved or uploaded.

The first delivery/request attempt was rejected at patch SHA-256
`fc9e5f8afb0960c3c71bf7720522840d716420fb227239caf70ddc1b7da2055a`.
The replacement attempt was rejected at patch SHA-256
`5ffa3a73e260d1df8f6aa10b2713b12788ba84449a3aaf4cfead6d75d02d0c23`
despite a generic-device build and 62/62 iPhone 17 Pro, iOS 26.5 simulator
tests. Its five remaining critical blockers are:

1. Same-item opportunity promotion can advance generation during an attempt
   and discard ACK or terminal truth.
2. The real ContactManager flush suppresses marker-persistence failure.
3. Moderation lookup failure is inconsistent: RequestsInbox hides the error
   while the reachable main inbox can expose pending contacts.
4. Initial send suppresses message-history persistence failure.
5. Per-item full-array compare-and-swap causes quadratic MainActor disk work
   for an indefinite queue.

The next worker must create a fresh plan and implementation from current main,
preserve legacy outbox compatibility and every previously repaired delivery
contract, add deterministic production-path tests for these five blockers, and
obtain new CRITICAL_VALIDATOR, SECOND_OPINION, and RELEASE_GATEKEEPER verdicts.
`IOS-V050-2` UI work remains blocked until that accepted forward-applied commit
exists.

## Cross-lane advisories to retain

- Android request loading uses destructive received-message draining.
- Android live chat delivery status is not observably wired on failure.
- Android API 29-32 discovery permission declarations and production mDNS test
  coverage need reconciliation.
- iOS notification previews/logs may expose plaintext and full peer IDs.
- iOS BLE reassembly needs integrity and resource bounds.
- Generated Swift/XCFramework/package slices may drift; Windows/FFI owns the
  authoritative reconciliation.
- iOS verification uses forbidden system temporary paths, omits XCTest, and
  does not fail warnings; iOS CI path filters omit relevant core surfaces.

## Apple v1.0 continuation order

1. Publish and mutually acknowledge the bilateral coordination files.
2. Repair, review, forward-apply, and upload the iOS delivery/request state
   patch.
3. Implement reachable blocked-peers/request rejection UI, delivery glyphs,
   retry controls, accessibility/localization, and UI tests.
4. Run no-download Apple preflight, then physical four-node evidence.
5. Address privacy/logging, BLE bounds, verification/CI, force unwraps,
   placeholder UI, background execution, deep links/share flows, signing,
   accessibility, and localization in small reviewed packets.
6. Keep macOS GUI implementation fail-closed until the operator settles the
   product and stack; read-only CLI/KMP/Catalyst/native comparison is safe.

The authoritative iOS gates are a generic-device `xcodebuild` and the complete
SCMessenger simulator XCTest suite using repository `scripts/build_lock.py`.
Do not download Xcode platforms without first verifying active downloads and
installed runtimes.

## Resume checklist

1. Fetch `upstream` and `origin`; verify `upstream/main` and all PR heads.
2. Read AGENTS.md, the Apple lane kickoff, execution plan, queue, CTO kickoff,
   and this file completely.
3. Recreate durable orchestration state; do not treat prior rejected reviews
   as approval for a new hash.
4. Verify PR #203 checks for its current head.
5. Independently validate and publish coordination manifest `41d6c1fa...`.
6. Replan the five blocked iOS persistence/concurrency issues from current main.
7. Never rebase shared branches; use fresh forward application.
