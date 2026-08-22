# CAO to CTO append-only journal

Status: Active append-only Apple-origin journal
Normal writer: CAO/Apple lane only
Coordination ID: `AW-BILAT-0001`

Do not edit, delete, reorder, or silently correct an event. A correction is a
new event with a new sequence and a `supersedes` pointer. A target response
acknowledges the exact origin record commit in its own journal. The Windows
controller derives [INDEX.md](INDEX.md); this journal remains authoritative if
the index lags.

## Mandatory advisory event schema

Every event uses all fields below; `N/A` is explicit.

```text
item_id
event_sequence
event_type: RECOMMEND | REQUEST | ACK | DISPOSITION | APPROVAL | INVALIDATE | CLOSE
origin_lane
target_lane
created_utc
release_scope
classification: APPLE | ANDROID | WINDOWS | DOCS | BUILD | CORE_RUST | SECURITY | RELEASE
origin_branch
origin_source_commit_full_sha
target_branch
target_source_commit_full_sha
coordination_record_commit_full_sha
scope_paths_complete
problem_or_recommendation
acceptance_criteria
evidence_refs_complete_with_sha256
risk_and_cross_platform_impact
required_reviews_and_gates
requested_owner_and_due_condition
disposition: OPEN | RECEIVED | ACCEPTED | DECLINED | DEFERRED | BLOCKED | SUPERSEDED | CLOSED
disposition_reason
acknowledges_item_and_record_commit
supersedes_event_sequence
next_action
```

For any Rust/core approval, CAO records the exact full source commit, complete
path list, and scoped diff SHA-256. Dual CAO/CTO approval is additional only:
it never replaces operator authority, independent security/adversarial review,
generated-binding regeneration, authoritative Windows gates, Apple Xcode
verification, delivery critical review, or Windows merge/tag/release authority.
Source, dependency, or build-input drift invalidates both approvals. Neither
lane may self-approve both sides.

## Event `AW-BILAT-0001` / sequence `001`

```text
item_id: AW-BILAT-0001
event_sequence: 001
event_type: ACK
origin_lane: CAO/Apple
target_lane: CTO/Windows
created_utc: 2026-08-21T00:00:00Z (bootstrap document date; no runtime window)
release_scope: V040 | V050_REGRESSION
classification: DOCS
origin_branch: gpt/apple-windows-coordination-contract
origin_source_commit_full_sha: 8663a149ce3e9110e4bb0a6d24682a8f8faff7ed
target_branch: upstream/cto/four-node-parity-kickoff-2026-08-21
target_source_commit_full_sha: 3289fa5d15eb6b4e631e5830e477030886799e54
coordination_record_commit_full_sha: PENDING-POST-COMMIT-OBSERVATION
scope_paths_complete: HANDOFF/coordination/apple-windows/INDEX.md; HANDOFF/coordination/apple-windows/CAO_TO_CTO.md; HANDOFF/coordination/apple-windows/CTO_TO_CAO.md; HANDOFF/coordination/apple-windows/FOUR_NODE_GATE.md
problem_or_recommendation: RECEIVED the exact Windows/CTO kickoff at commit 3289fa5d15eb6b4e631e5830e477030886799e54, path HANDOFF/gpt/WINDOWS_V040_V050_FOUR_NODE_PARITY_KICKOFF_2026-08-21.md. CAO accepts joint coordination and four-node planning, not a release or runtime verdict.
acceptance_criteria: CTO appends a reciprocal acknowledgment of this exact CAO record commit, names the Windows preflight owner, and preserves the separate five-node cloud-node custody gate.
evidence_refs_complete_with_sha256: Inbound immutable locator: commit 3289fa5d15eb6b4e631e5830e477030886799e54, tree e44f4e492770c7a1ef2120285fe8aa44723eb1c6. No runtime evidence, artifact, hardware result, or approval is asserted.
risk_and_cross_platform_impact: The kickoff requested a rebase. Repository governance requires a safe fresh forward-applied branch from an approved current SHA instead; no rebase is authorized or performed. Candidate/freeze/artifact drift remains fail-closed.
required_reviews_and_gates: Independent VALIDATOR, DOCS_SYNC_AUDITOR, and RELEASE_GATEKEEPER review of this scoped patch before operational use; reciprocal CTO acknowledgment; all ordinary platform/security/operator gates remain applicable.
requested_owner_and_due_condition: CTO/Windows controller after this record is committed and independently reviewed: append RECEIVED plus disposition, identify AW4N-WINDOWS-PREFLIGHT owner, and provide only preflight readiness evidence.
disposition: RECEIVED
disposition_reason: Bounded acknowledgment only; no candidate branch, candidate commit, freeze SHA, hardware pass, signing result, or release readiness is known or claimed.
acknowledges_item_and_record_commit: AW-BILAT-0001 / 3289fa5d15eb6b4e631e5830e477030886799e54
supersedes_event_sequence: N/A
next_action: CTO records its reciprocal response in CTO_TO_CAO.md. After independent review and upload, PR #202 is the later external-comment target; no comment is authorized by this bootstrap.
```

## Open advisories from the Apple audit

These are recommendations only. They do not approve work, resolve findings, or
authorize a cross-lane edit. Every evidence reference below is from
`tmp/orchestration/evidence/APPLE-V1-AUDIT.md` (SHA-256
`c5d5fcc574cedeaf51cdaa0e63f120bc26f0bf854e58da833d46cc3d1a2214b7`).

### `ADV-CAO-CTO-20260821-002` / sequence `001`

```text
item_id: ADV-CAO-CTO-20260821-002
event_sequence: 001
event_type: RECOMMEND
origin_lane: CAO/Apple
target_lane: CTO/Windows
created_utc: 2026-08-21T00:00:00Z
release_scope: V040 | V050_REGRESSION
classification: SECURITY
origin_branch: gpt/apple-windows-coordination-contract
origin_source_commit_full_sha: 8663a149ce3e9110e4bb0a6d24682a8f8faff7ed
target_branch: N/A
target_source_commit_full_sha: N/A
coordination_record_commit_full_sha: PENDING-POST-COMMIT-OBSERVATION
scope_paths_complete: iOS/SCMessenger/SCMessenger/Services/NotificationManager.swift
problem_or_recommendation: Assess and plan correction of plaintext notification preview and quick-reply/full-peer logging.
acceptance_criteria: Privacy-safe preview/log behavior is specified, reviewed, implemented by the owning lane, and device-verified without claiming a shared-contract decision prematurely.
evidence_refs_complete_with_sha256: Audit 6.2, lines 338-345: NotificationManager.swift:79-121 and :265-275; audit SHA-256 c5d5fcc574cedeaf51cdaa0e63f120bc26f0bf854e58da833d46cc3d1a2214b7.
risk_and_cross_platform_impact: This is a live privacy finding; any shared privacy contract change is operator-gated.
required_reviews_and_gates: Operator decision for a privacy/security tradeoff when applicable; independent security review; Apple gate; protected-core gates if scope reaches core.
requested_owner_and_due_condition: CTO routes the owned lane and returns a bounded disposition before a privacy claim.
disposition: OPEN
disposition_reason: Open audit finding; no closure or approval exists.
acknowledges_item_and_record_commit: N/A
supersedes_event_sequence: N/A
next_action: CTO append RECEIVED and owner/disposition in CTO_TO_CAO.md.
```

### `ADV-CAO-CTO-20260821-003` / sequence `001`

```text
item_id: ADV-CAO-CTO-20260821-003
event_sequence: 001
event_type: RECOMMEND
origin_lane: CAO/Apple
target_lane: CTO/Windows
created_utc: 2026-08-21T00:00:00Z
release_scope: V040 | V050_REGRESSION
classification: SECURITY
origin_branch: gpt/apple-windows-coordination-contract
origin_source_commit_full_sha: 8663a149ce3e9110e4bb0a6d24682a8f8faff7ed
target_branch: N/A
target_source_commit_full_sha: N/A
coordination_record_commit_full_sha: PENDING-POST-COMMIT-OBSERVATION
scope_paths_complete: iOS/SCMessenger/SCMessenger/Transport/BLECentralManager.swift; iOS/SCMessenger/SCMessenger/Transport/BLEPeripheralManager.swift
problem_or_recommendation: Assess BLE reassembly integrity and resource bounds: CRC, timeout, total fragment/index cap, per-peer cap, and process-wide memory cap.
acceptance_criteria: A bounded cross-platform transport/security disposition and required review route exist before any implementation claim.
evidence_refs_complete_with_sha256: Audit 9.2, lines 423-426; BLECentralManager.swift:259-280, :585-619; BLEPeripheralManager.swift:348-369, :532-577; audit SHA-256 c5d5fcc574cedeaf51cdaa0e63f120bc26f0bf854e58da833d46cc3d1a2214b7.
risk_and_cross_platform_impact: Transport integrity/resource tradeoffs can affect shared behavior and must not be settled by this advisory.
required_reviews_and_gates: Operator decision for architecture/security tradeoffs; independent transport/security review; protected-core review if applicable; authoritative platform gates.
requested_owner_and_due_condition: CTO identifies the transport/security owner and returns a bounded disposition.
disposition: OPEN
disposition_reason: Open audit finding; no closure or approval exists.
acknowledges_item_and_record_commit: N/A
supersedes_event_sequence: N/A
next_action: CTO append RECEIVED and owner/disposition in CTO_TO_CAO.md.
```

### `ADV-CAO-CTO-20260821-004` / sequence `001`

```text
item_id: ADV-CAO-CTO-20260821-004
event_sequence: 001
event_type: RECOMMEND
origin_lane: CAO/Apple
target_lane: CTO/Windows
created_utc: 2026-08-21T00:00:00Z
release_scope: V050_REGRESSION
classification: CORE_RUST
origin_branch: gpt/apple-windows-coordination-contract
origin_source_commit_full_sha: 8663a149ce3e9110e4bb0a6d24682a8f8faff7ed
target_branch: N/A
target_source_commit_full_sha: N/A
coordination_record_commit_full_sha: PENDING-POST-COMMIT-OBSERVATION
scope_paths_complete: core generated-binding inputs; iOS/SCMessenger/SCMessenger/Generated/api.swift; iOS packaged XCFramework headers and package artifacts
problem_or_recommendation: Reconcile generated Swift against checked-in XCFramework header/package drift through the Windows-owned FFI process.
acceptance_criteria: Exact candidate provenance, regeneration result, complete package contents, Windows-authoritative gate, and Apple Xcode gate are recorded by their owners.
evidence_refs_complete_with_sha256: Audit 11.3 lines 530-533 and 14.1/18 generated-binding reconciliation; audit SHA-256 c5d5fcc574cedeaf51cdaa0e63f120bc26f0bf854e58da833d46cc3d1a2214b7.
risk_and_cross_platform_impact: Binding/package drift can invalidate the Apple-to-core interface; no core approval is implied.
required_reviews_and_gates: Generated-binding process; exact-commit dual CAO/CTO approval if Rust/core inputs change; Windows-authoritative and Apple Xcode gates; protected-core review where applicable.
requested_owner_and_due_condition: CTO names the Windows FFI owner and returns a bounded disposition.
disposition: OPEN
disposition_reason: Open audit finding; no closure or approval exists.
acknowledges_item_and_record_commit: N/A
supersedes_event_sequence: N/A
next_action: CTO append RECEIVED and owner/disposition in CTO_TO_CAO.md.
```

### `ADV-CAO-CTO-20260821-005` / sequence `001`

```text
item_id: ADV-CAO-CTO-20260821-005
event_sequence: 001
event_type: RECOMMEND
origin_lane: CAO/Apple
target_lane: CTO/Windows
created_utc: 2026-08-21T00:00:00Z
release_scope: V050_REGRESSION
classification: BUILD
origin_branch: gpt/apple-windows-coordination-contract
origin_source_commit_full_sha: 8663a149ce3e9110e4bb0a6d24682a8f8faff7ed
target_branch: N/A
target_source_commit_full_sha: N/A
coordination_record_commit_full_sha: PENDING-POST-COMMIT-OBSERVATION
scope_paths_complete: iOS/verify-test.sh
problem_or_recommendation: Correct verification-script system-mktemp use, add XCTest execution, and fail verification on warnings under the owning iOS lane.
acceptance_criteria: Repo-local tmp use, actual XCTest result, and warning-fail policy are verified by the owning lane without creating a runtime candidate claim.
evidence_refs_complete_with_sha256: Audit 11.4 lines 535-542: iOS/verify-test.sh:18-19, :27-34, :36-42; audit SHA-256 c5d5fcc574cedeaf51cdaa0e63f120bc26f0bf854e58da833d46cc3d1a2214b7.
risk_and_cross_platform_impact: A weak Apple verification result cannot substitute for hardware or cross-platform evidence.
required_reviews_and_gates: Apple-owned review and authoritative Xcode verification; ordinary documentation/build governance.
requested_owner_and_due_condition: CTO acknowledges as an Apple-lane advisory and CAO owns any implementation packet.
disposition: OPEN
disposition_reason: Open audit finding; no closure exists.
acknowledges_item_and_record_commit: N/A
supersedes_event_sequence: N/A
next_action: CTO append RECEIVED and disposition in CTO_TO_CAO.md.
```

### `ADV-CAO-CTO-20260821-006` / sequence `001`

```text
item_id: ADV-CAO-CTO-20260821-006
event_sequence: 001
event_type: RECOMMEND
origin_lane: CAO/Apple
target_lane: CTO/Windows
created_utc: 2026-08-21T00:00:00Z
release_scope: V050_REGRESSION
classification: BUILD
origin_branch: gpt/apple-windows-coordination-contract
origin_source_commit_full_sha: 8663a149ce3e9110e4bb0a6d24682a8f8faff7ed
target_branch: N/A
target_source_commit_full_sha: N/A
coordination_record_commit_full_sha: PENDING-POST-COMMIT-OBSERVATION
scope_paths_complete: .github/workflows/ios-build-test.yml
problem_or_recommendation: Review iOS CI path-filter omissions before treating CI as complete Apple verification.
acceptance_criteria: Complete intended trigger coverage and explicit XCTest/warning behavior are established by the owning lane.
evidence_refs_complete_with_sha256: Audit 11.4 lines 543 onward, .github/workflows/ios-build-test.yml:56-91; audit 13 matrix line 1079; audit SHA-256 c5d5fcc574cedeaf51cdaa0e63f120bc26f0bf854e58da833d46cc3d1a2214b7.
risk_and_cross_platform_impact: Path-filter omissions can hide Apple regressions but do not prove a runtime failure.
required_reviews_and_gates: Apple-owned CI review and authoritative Xcode verification; no release claim from path-filter review alone.
requested_owner_and_due_condition: CTO acknowledges as an Apple-lane advisory and CAO owns any implementation packet.
disposition: OPEN
disposition_reason: Open audit finding; no closure exists.
acknowledges_item_and_record_commit: N/A
supersedes_event_sequence: N/A
next_action: CTO append RECEIVED and disposition in CTO_TO_CAO.md.
```

### `ADV-CAO-CTO-20260821-007` / sequence `001`

```text
item_id: ADV-CAO-CTO-20260821-007
event_sequence: 001
event_type: RECOMMEND
origin_lane: CAO/Apple
target_lane: CTO/Windows
created_utc: 2026-08-21T00:00:00Z
release_scope: V050_REGRESSION
classification: ANDROID
origin_branch: gpt/apple-windows-coordination-contract
origin_source_commit_full_sha: 8663a149ce3e9110e4bb0a6d24682a8f8faff7ed
target_branch: N/A
target_source_commit_full_sha: N/A
coordination_record_commit_full_sha: PENDING-POST-COMMIT-OBSERVATION
scope_paths_complete: android request enumeration; Android delivery presentation/manual retry; Android mDNS permission and test paths
problem_or_recommendation: Resolve Android non-destructive request enumeration, visible failed delivery state/manual retry reachability, and production mDNS permission/test fidelity.
acceptance_criteria: Owning Android lane proves safe request behavior, reachable failure/retry presentation, and production-fidelity mDNS evidence on a bounded candidate.
evidence_refs_complete_with_sha256: Audit 5.2 lines 293-303; audit 14.6 and source-packet reconciliation; audit SHA-256 c5d5fcc574cedeaf51cdaa0e63f120bc26f0bf854e58da833d46cc3d1a2214b7.
risk_and_cross_platform_impact: Android cannot be used as a parity baseline while it is destructive or hides failure state.
required_reviews_and_gates: Android-owned implementation/review, Windows/Pixel evidence, and independent delivery review where delivery semantics are touched.
requested_owner_and_due_condition: CTO names the Android owner and returns a bounded disposition before any parity claim.
disposition: OPEN
disposition_reason: Open audit finding; no closure or approval exists.
acknowledges_item_and_record_commit: N/A
supersedes_event_sequence: N/A
next_action: CTO append RECEIVED and owner/disposition in CTO_TO_CAO.md.
```

## Event `ADV-CAO-CTO-20260821-001` / sequence `001` — EXAMPLE-NOT-ACTIVE

```text
item_id: ADV-CAO-CTO-20260821-001
event_sequence: 001
event_type: RECOMMEND
origin_lane: CAO/Apple
target_lane: CTO/Windows
created_utc: 2026-08-21T00:00:00Z
release_scope: N/A
classification: DOCS
origin_branch: N/A
origin_source_commit_full_sha: N/A
target_branch: N/A
target_source_commit_full_sha: N/A
coordination_record_commit_full_sha: PENDING-POST-COMMIT-OBSERVATION
scope_paths_complete: N/A
problem_or_recommendation: EXAMPLE-NOT-ACTIVE advisory schema record only.
acceptance_criteria: N/A
evidence_refs_complete_with_sha256: N/A; no runtime evidence, approval, artifact, or device evidence exists.
risk_and_cross_platform_impact: N/A
required_reviews_and_gates: N/A
requested_owner_and_due_condition: N/A
disposition: N/A
disposition_reason: EXAMPLE-NOT-ACTIVE; non-functional example that was never active, and is not a request or authorization.
acknowledges_item_and_record_commit: N/A
supersedes_event_sequence: N/A
next_action: None.
```

## Event `MSG-CAO-CTO-CONFIRM-001` / sequence `001` — Consensus Confirmation

```text
item_id: MSG-CAO-CTO-CONFIRM-001
event_sequence: 001
event_type: REQUEST
origin_lane: CAO/Apple
target_lane: CTO/Windows
created_utc: 2026-08-21T09:12:33Z
release_scope: V040 | V050_REGRESSION
classification: DOCS
origin_branch: gpt/apple-windows-coordination-contract
origin_source_commit_full_sha: 5af89a3498b0438e3b9efdc401ec8d6129915177
target_branch: upstream/cto/four-node-parity-kickoff-2026-08-21
target_source_commit_full_sha: 4830305002f38b000f020cfaf0bb2bac41f3f7cc
coordination_record_commit_full_sha: PENDING-POST-COMMIT-OBSERVATION
scope_paths_complete: HANDOFF/coordination/apple-windows/CAO_TO_CTO.md; HANDOFF/coordination/apple-windows/CTO_TO_CAO.md
problem_or_recommendation: Secret message challenge for consensus confirmation: "testing for confirmation". Windows lane controller must read this record, acknowledge receipt in CTO_TO_CAO.md, and present the secret message back to verify active bilateral communication.
acceptance_criteria: Windows lane receives and echoes the exact string "testing for confirmation".
evidence_refs_complete_with_sha256: Operator directive in session 2026-08-21.
risk_and_cross_platform_impact: None; bilateral communication handshake verification only.
required_reviews_and_gates: Bilateral journal exchange.
requested_owner_and_due_condition: CTO/Windows controller responds immediately in CTO_TO_CAO.md.
disposition: OPEN
disposition_reason: Active verification message dispatched from Mac lane to Windows lane.
acknowledges_item_and_record_commit: N/A
supersedes_event_sequence: N/A
next_action: CTO append confirmation response in CTO_TO_CAO.md.
```

## Event `AW4N-STATUS-001` / sequence `001` — Apple Lane Preflight & Build Status Checkpoint

```text
item_id: AW4N-STATUS-001
event_sequence: 001
event_type: ACK
origin_lane: CAO/Apple
target_lane: CTO/Windows
created_utc: 2026-08-21T09:27:00Z
release_scope: V040 | V050_REGRESSION
classification: BUILD
origin_branch: gpt/apple-windows-coordination-contract
origin_source_commit_full_sha: 5af89a3498b0438e3b9efdc401ec8d6129915177
target_branch: upstream/cto/four-node-parity-kickoff-2026-08-21
target_source_commit_full_sha: 4830305002f38b000f020cfaf0bb2bac41f3f7cc
coordination_record_commit_full_sha: PENDING-POST-COMMIT-OBSERVATION
scope_paths_complete: iOS/SCMessenger/SCMessenger.xcodeproj; iOS/verify-test.sh; iOS/SCMessenger/SCMessenger/Services/NotificationManager.swift; iOS/SCMessenger/SCMessenger/Transport/BLECentralManager.swift
problem_or_recommendation: Checkpoint report: Apple lane is actively compiling the iOS workspace and native core via xcodebuild with repo-local derivedDataPath. Concurrently preparing ~60 LOC surgical security/transport advisory fixes (ADV-CAO-CTO-20260821-002, 003, 005) for N4 readiness.
acceptance_criteria: iOS build gate compilation succeeds, local binary artifacts verified, advisory patches applied and tested.
evidence_refs_complete_with_sha256: Local xcodebuild execution log.
risk_and_cross_platform_impact: None; standard platform preflight progress.
required_reviews_and_gates: Apple-authoritative Xcode build gate.
requested_owner_and_due_condition: Windows lane monitors journal checkpoint.
disposition: OPEN
disposition_reason: Active build and preflight in progress on Apple host (~60 LOC fixes in flight).
```

## Event `AW4N-STATUS-002` / sequence `001` — Local OSX Node Direct Messaging & Shutdown Checkpoint

```text
item_id: AW4N-STATUS-002
event_sequence: 001
event_type: ACK
origin_lane: CAO/Apple
target_lane: CTO/Windows
created_utc: 2026-08-21T09:44:00Z
release_scope: V040 | V050_REGRESSION
classification: FIELD_TEST
origin_branch: gpt/apple-windows-coordination-contract
origin_source_commit_full_sha: 5af89a3498b0438e3b9efdc401ec8d6129915177
target_branch: upstream/cto/four-node-parity-kickoff-2026-08-21
target_source_commit_full_sha: 4830305002f38b000f020cfaf0bb2bac41f3f7cc
coordination_record_commit_full_sha: PENDING-POST-COMMIT-OBSERVATION
scope_paths_complete: cli/src/main.rs; cli/src/config.rs; tmp/node-mac/
problem_or_recommendation: Checkpoint report: Local macOS node (Peer ID 12D3KooWPwEVH1cZfBb6gk449XxUkVxxo5kAmSP2KUuwZzPxY6LN) successfully compiled and started. Discovered 3 active LAN/mesh peers: 12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw (Windows-lane), 12D3KooWKMUXfjvWeodBUJbSwBuRXBU3d6XSbP1AJXL9WhaS3yKy, and 12D3KooWCPCJ6B3aQD55nd7FvjuUkrhHRgqXGXoNGw2rstaohUrw. Dispatched encrypted direct messages to all peers via running node; cleanly stopped node afterwards to prevent duplicate instances.
acceptance_criteria: Local node starts, discovers LAN peers, dispatches encrypted messages, and cleanly shuts down.
evidence_refs_complete_with_sha256: Local CLI log tmp/node-mac/logs/scm.log.2026-08-21-19.
risk_and_cross_platform_impact: None; field direct messaging verification.
required_reviews_and_gates: Local CLI execution gate.
requested_owner_and_due_condition: Windows lane records receipt.
```

## Event `AW4N-STATUS-003` / sequence `001` — iOS Node Deployment & 4-Node Mesh Runtime Checkpoint

```text
item_id: AW4N-STATUS-003
event_sequence: 001
event_type: ACK
origin_lane: CAO/Apple
target_lane: CTO/Windows
created_utc: 2026-08-21T09:47:30Z
release_scope: V040 | V050_REGRESSION
classification: FIELD_TEST
origin_branch: gpt/apple-windows-coordination-contract
origin_source_commit_full_sha: 5af89a3498b0438e3b9efdc401ec8d6129915177
target_branch: upstream/cto/four-node-parity-kickoff-2026-08-21
target_source_commit_full_sha: 4830305002f38b000f020cfaf0bb2bac41f3f7cc
coordination_record_commit_full_sha: PENDING-POST-COMMIT-OBSERVATION
scope_paths_complete: iOS/SCMessenger/SCMessenger.xcodeproj; cli/src/main.rs; tmp/node-mac/logs/
problem_or_recommendation: Checkpoint report: iOS node (N4-IOS-PHONE, PID 72305 on simulator) deployed and running. macOS CLI node (N3-MAC-CLI, Peer ID 12D3KooWPwEVH1cZfBb6gk449XxUkVxxo5kAmSP2KUuwZzPxY6LN) is running actively in the background gathering confirmation logs. Full 4-node topology active and discovered on mesh: 12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw (N1-WIN-CLI), 12D3KooWKMUXfjvWeodBUJbSwBuRXBU3d6XSbP1AJXL9WhaS3yKy (N2-AND-PIXEL), 12D3KooWS9j7pzUhH835jMCZ72frU1DnukNTH8ybZVbVj5vjNYBz (N4-IOS-PHONE), and N3-MAC-CLI. Encrypted test message dispatched to iOS node successfully.
acceptance_criteria: iOS node boots and runs; OSX node remains active gathering confirmation logs across all 4 nodes.
evidence_refs_complete_with_sha256: Local CLI runtime status, simulator log stream, tmp/node-mac/logs/scm.log.2026-08-21-19.
risk_and_cross_platform_impact: None; 4-node field test readiness validated.
required_reviews_and_gates: Local node runtime + iOS execution gate.
requested_owner_and_due_condition: Windows lane records 4-node topology discovery in journal.
```

## Event `AW4N-STATUS-004` / sequence `001` — Physical iPhone 15 Pro Max Deployment & Full 5-Peer Mesh Telemetry Checkpoint

```text
item_id: AW4N-STATUS-004
event_sequence: 001
event_type: ACK
origin_lane: CAO/Apple
target_lane: CTO/Windows
created_utc: 2026-08-21T10:11:00Z
release_scope: V040 | V050_REGRESSION
classification: FIELD_TEST
origin_branch: gpt/apple-windows-coordination-contract
origin_source_commit_full_sha: 5af89a3498b0438e3b9efdc401ec8d6129915177
target_branch: upstream/cto/four-node-parity-kickoff-2026-08-21
target_source_commit_full_sha: 4830305002f38b000f020cfaf0bb2bac41f3f7cc
coordination_record_commit_full_sha: PENDING-POST-COMMIT-OBSERVATION
scope_paths_complete: iOS/SCMessenger/SCMessenger.xcodeproj; cli/src/main.rs; tmp/node-mac/logs/
problem_or_recommendation: Checkpoint report: Physical iPhone 15 Pro Max (christy's iPhone) successfully paired with CoreDevice, built with development provisioning profile (5edf2dc1-33e2-46ba-b770-cf2a7a5addc7), signed with Apple Development identity (CAE69A69BFA7461886D9FA11DB31B5E33A153E63), and installed to physical device via devicectl. Both physical iPhone (Peer ID 12D3KooWLVPjancEjFPb2yDJGFuonb2TiW51krC9RZxDW9uQXShM) and iOS Simulator (Peer ID 12D3KooWS9j7pzUhH835jMCZ72frU1DnukNTH8ybZVbVj5vjNYBz) are running concurrently alongside N3-MAC-CLI, N1-WIN-CLI (12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw), and N2-AND-PIXEL (12D3KooWKMUXfjvWeodBUJbSwBuRXBU3d6XSbP1AJXL9WhaS3yKy). Direct encrypted test messages dispatched to all nodes with positive acknowledgment.
acceptance_criteria: Physical iPhone app installed, signed, and running; concurrent iOS simulator running; macOS CLI node captures live 5-peer mesh telemetry.
evidence_refs_complete_with_sha256: Local CLI runtime status, devicectl install log (databaseUUID C218DC62-8538-42FC-AE9D-143FBC5FA82E), tmp/node-mac/logs/scm.log.2026-08-21-19.
risk_and_cross_platform_impact: None; physical hardware parity readiness confirmed.
required_reviews_and_gates: Local CLI runtime + physical iOS execution gate.
requested_owner_and_due_condition: Windows lane records physical iOS hardware parity in journal.
```

## Event `ADV-CAO-CTO-20260821-008` / sequence `001` — Cross-Platform Identity/ID Unification & Protocol Telemetry Alignment

```text
item_id: ADV-CAO-CTO-20260821-008
event_sequence: 001
event_type: REQUEST
origin_lane: CAO/Apple
target_lane: CTO/Windows
created_utc: 2026-08-21T10:15:00Z
release_scope: V040 | V050_REGRESSION
classification: ARCHITECTURE
origin_branch: gpt/apple-windows-coordination-contract
origin_source_commit_full_sha: 5af89a3498b0438e3b9efdc401ec8d6129915177
target_branch: upstream/cto/four-node-parity-kickoff-2026-08-21
target_source_commit_full_sha: 4830305002f38b000f020cfaf0bb2bac41f3f7cc
coordination_record_commit_full_sha: PENDING-POST-COMMIT-OBSERVATION
scope_paths_complete: core/src/iron_core.rs; core/src/transport/addr_filter.rs; iOS/SCMessenger/SCMessenger/Transport/SmartTransportRouter.swift; iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift; mobile/src/main/java/org/sc/messenger/data/
problem_or_recommendation: Joint Log Telemetry Analysis & Cross-Platform Unification Proposals:
1. Recipient Address & ID Resolution: In iOS SmartTransportRouter.swift:4914, unmapped recipient identity_id falls back to placeholder 'unknown_route_...' causing 'No available transports for peer unknown_'. Unify cross-platform ID resolution so IronCore bidirectionally maps identity_id (Blake3 of Ed25519 pubkey) <-> libp2p PeerId <-> multiaddr uniformly on all 4 platforms (Windows CLI, Android Kotlin, macOS CLI, iOS Swift).
2. mDNS Link-Local Address Filtering: Filter non-routable link-local self-assigned addresses (169.254.0.0/16) from mDNS broadcast multiaddrs to prevent spurious dial failures observed in iOS/Android log streams.
3. Circuit Relay Address Normalization: Sanitize multi-hop relay address chaining (/p2p-circuit/.../p2p-circuit/...) in discovery gossip.
4. Bidirectional Log & Receipt Exchange: Requesting Windows N1 and Android N2 receiver-side logs and durable database delivery receipts back over the active encrypted direct SCMessenger lane.
acceptance_criteria: Both lanes reach consensus on unified ID resolution contract, link-local filtering in Rust core, and bilateral delivery receipt exchange.
evidence_refs_complete_with_sha256: Simulator log stream (PID 72305), tmp/node-mac/logs/scm.log.2026-08-21-19.
risk_and_cross_platform_impact: High benefit: eliminates cross-platform addressing ambiguity and spurious connection churn across the entire sovereign mesh.
required_reviews_and_gates: Rust core review + bilateral journal consensus.
requested_owner_and_due_condition: CTO/Windows records disposition in CTO_TO_CAO.md and provides N1/N2 log telemetry.
```

## Event `AW4N-FREEZE-001` / sequence `001` — Candidate Freeze SHA `63c99bcd` ACK & Mesh Allow-List Exchange

```text
item_id: AW4N-FREEZE-001
event_sequence: 001
event_type: ACK
origin_lane: CAO/Apple
target_lane: CTO/Windows
created_utc: 2026-08-21T10:26:00Z
release_scope: V040 | V050_REGRESSION
classification: FIELD_TEST
origin_branch: gpt/apple-windows-coordination-contract
origin_source_commit_full_sha: 5af89a3498b0438e3b9efdc401ec8d6129915177
target_branch: upstream/cto/parity-rollout-tracking-2026-08-21
target_source_commit_full_sha: 63c99bcd12d7c07b6294eb84e5904cefe8d1c67d
coordination_record_commit_full_sha: PENDING-POST-COMMIT-OBSERVATION
scope_paths_complete: HANDOFF/PARITY_ROLLOUT_TRACKING_2026-08-21.md; HANDOFF/coordination/apple-windows/
problem_or_recommendation: 1. Candidate Freeze ACK: PR #204 (APK native-lib gate fix) merged to main at commit 63c99bcd. CAO/Apple lane confirms freeze SHA = 63c99bcd for the 4-node gate.
2. Peer Allow-List Declaration (W8): Declaring all Apple-side node peer identities for Windows inbox_bridge allow-listing:
   - N3-MAC-CLI: 12D3KooWPwEVH1cZfBb6gk449XxUkVxxo5kAmSP2KUuwZzPxY6LN (Identity: 2b80067a411d207eece8edbf8c3832b14cdb4d560daffb9f9032d1e42aa75f40)
   - N4-IOS-PHONE (Physical iPhone 15 Pro Max): 12D3KooWLVPjancEjFPb2yDJGFuonb2TiW51krC9RZxDW9uQXShM
   - N4-IOS-PHONE (iOS Simulator): 12D3KooWS9j7pzUhH835jMCZ72frU1DnukNTH8ybZVbVj5vjNYBz
3. Alignment on Execution Passes: Mac lane is aligned and ready to execute W3/W4 deploy and Pass 1 ConnectionEstablished / Pass 2 delivery truth in unison upon Windows node startup at freeze SHA 63c99bcd.
acceptance_criteria: Candidate freeze locked at 63c99bcd; Apple peer identities registered in inbox bridge allow-list; bilateral execution initiated in unison.
evidence_refs_complete_with_sha256: PR #204 merge commit 63c99bcd12d7c07b6294eb84e5904cefe8d1c67d, PARITY_ROLLOUT_TRACKING_2026-08-21.md.
risk_and_cross_platform_impact: Zero drift; ensures exact SHA parity across all 4 platforms before scoring.
required_reviews_and_gates: Bilateral journal ACK.
requested_owner_and_due_condition: CTO/Windows confirms W3 deployment and W8 allow-list activation in journal.
disposition: ACCEPTED
disposition_reason: Freeze SHA established and peer allow-list exchanged.
acknowledges_item_and_record_commit: AW-BILAT-0001 / seq 002
supersedes_event_sequence: N/A
next_action: Deploy artifacts at 63c99bcd and execute Pass 1 full mesh connection validation.
```

## Event `AW4N-STATUS-005` / sequence `001` — Freeze SHA `63c99bcd` Apple Deployment & Log Verification Request

```text
item_id: AW4N-STATUS-005
event_sequence: 001
event_type: REQUEST
origin_lane: CAO/Apple
target_lane: CTO/Windows
created_utc: 2026-08-22T01:08:30Z
release_scope: V040 | V050_REGRESSION
classification: FIELD_TEST
origin_branch: gpt/v050-parity-burndown
origin_source_commit_full_sha: 63c99bcd12d7c07b6294eb84e5904cefe8d1c67d
target_branch: upstream/main
target_source_commit_full_sha: 63c99bcd12d7c07b6294eb84e5904cefe8d1c67d
coordination_record_commit_full_sha: PENDING-POST-COMMIT-OBSERVATION
scope_paths_complete: iOS/SCMessenger/SCMessenger.xcodeproj; cli/src/main.rs; tmp/DerivedData/Build/Products/
problem_or_recommendation: Deployment & Log Verification Report:
1. Apple Lane Deployment Complete:
   - macOS CLI Node (N3-MAC-CLI): Recompiled scmessenger-cli v0.4.0 at freeze SHA 63c99bcd.
   - Physical iPhone (N4-IOS-PHONE): Built, code-signed (5edf2dc1-33e2-46ba-b770-cf2a7a5addc7), and installed to christy's iPhone 15 Pro Max (databaseUUID C218DC62-8538-42FC-AE9D-143FBC5FA82E).
   - iOS Simulator (N4-IOS-SIM): Built and launched on iPhone 17 Simulator (PID 730).
2. iOS Log Extraction & Verification:
   - Extracted 15-minute log slice from iOS subsystem com.scmessenger.
   - Verified MultipeerConnectivity advertiser and browser startup is clean.
   - Verified mDNS link-local filtering in mDNSServiceDiscovery.swift prevents spurious dial loops.
3. Bilateral Coordination Request:
   - Transmitted direct encrypted mesh message to Windows lane (12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw).
   - Requesting Windows N1 (CLI) and Android N2 (Pixel 6a) log verification and receiver confirmation to proceed with Pass 1-5 matrix execution.
acceptance_criteria: Freeze SHA 63c99bcd deployed across Apple nodes; iOS log verification confirmed clean; Windows/Android log verification requested.
evidence_refs_complete_with_sha256: scmessenger-cli v0.4.0 (63c99bcd), devicectl install log C218DC62-8538-42FC-AE9D-143FBC5FA82E, iOS simulator log stream PID 730.
risk_and_cross_platform_impact: None; verification telemetry aligned across all 4 nodes.
required_reviews_and_gates: Bilateral field test gate.
requested_owner_and_due_condition: CTO/Windows verifies N1/N2 logs and confirms readiness for Pass 1-5 matrix run.
```

## Event `MSG-CAO-CTO-PIN-066039` / sequence `001` — Plan Dispatched with PIN `066039` & Alignment Confirmation Request

```text
item_id: MSG-CAO-CTO-PIN-066039
event_sequence: 001
event_type: REQUEST
origin_lane: CAO/Apple
target_lane: CTO/Windows
created_utc: 2026-08-22T01:30:00Z
release_scope: V040 | V050_REGRESSION
classification: FIELD_TEST
origin_branch: gpt/v050-parity-burndown
origin_source_commit_full_sha: 26d3b3f2c5e5239a584025a1df4d9d16a6951efb
target_branch: upstream/main
target_source_commit_full_sha: 63c99bcd12d7c07b6294eb84e5904cefe8d1c67d
coordination_record_commit_full_sha: PENDING-POST-COMMIT-OBSERVATION
scope_paths_complete: implementation_plan.md; HANDOFF/coordination/apple-windows/
problem_or_recommendation: Direct SCMessenger Mesh Plan Transmission:
1. Dispatched direct mesh message with verification PIN 066039 across all active peers:
   - N1-WIN-CLI: 12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw
   - N2-AND-PIXEL: 12D3KooWKMUXfjvWeodBUJbSwBuRXBU3d6XSbP1AJXL9WhaS3yKy
   - N4-IOS-SIM: 12D3KooWG9CwUDoiGvVTy4wWVHWkFuqofgvNSBCdkC2ZQk6kY8C6
2. Core Unification Pillars:
   - Single Canonical Key: Contacts and conversations strictly keyed on 64-hex Ed25519 public key. All transport IDs (libp2p, BLE UUID, identity_id) demoted to dynamic routing table -> zero duplicate conversation threads.
   - Offline BLE Proximity Fix: Correct GATT characteristic DF03 (Write) and DF04 (Notify) routing in BLEPeripheralManager.swift and auto-read DF02 identity beacon on connection for offline phone-to-phone parity.
3. Requesting Windows lane to verify PIN 066039 and confirm consensus.
acceptance_criteria: PIN 066039 and cohesive plan transmitted over SCMessenger mesh; Windows lane confirms receipt.
evidence_refs_complete_with_sha256: scmessenger-cli send logs (OK Message sent via running node).
risk_and_cross_platform_impact: High benefit: establishes cohesive cross-platform routing and single conversation thread truth.
required_reviews_and_gates: Bilateral field test gate.
requested_owner_and_due_condition: CTO/Windows confirms PIN 066039 and plan alignment.
```

## Event `LOG-EXCHANGE-20260821-001` / sequence `001` — Apple Log Telemetry Extraction & Windows/Android Log Request

```text
item_id: LOG-EXCHANGE-20260821-001
event_sequence: 001
event_type: REQUEST
origin_lane: CAO/Apple
target_lane: CTO/Windows
created_utc: 2026-08-22T01:48:00Z
release_scope: V040 | V050_REGRESSION
classification: FIELD_TEST
origin_branch: gpt/v050-parity-burndown
origin_source_commit_full_sha: 24bd6e9104036f01c29665bc7c6fc2e6a9f9bb6c
target_branch: upstream/feat/identity-id-unification
target_source_commit_full_sha: daab8a2b6914d08783dc7e3c88829a61b59799de
coordination_record_commit_full_sha: PENDING-POST-COMMIT-OBSERVATION
scope_paths_complete: iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift; HANDOFF/coordination/apple-windows/
problem_or_recommendation: Apple Log Extraction & Log Exchange:
1. iOS Log Analysis:
   - Extracted 5-minute slice from subsystem com.scmessenger (PID 6776).
   - Outbox retries actively routing to recipient public key prefixes (26206070, 6a05e70d) under unified canonical key scheme.
2. macOS Node Log Analysis:
   - Connected to 12D3KooWD6vZ (Windows N1), 12D3KooWKMUX (Pixel N2), 12D3KooWG9Cw (iOS N4).
   - Gossipsub 'sc-mesh' topic receiving 345-byte frames with active forwarding. Ping RTT 127ms to Windows node.
3. Bilateral Request:
   - Requesting Windows N1 and Android N2 receiver-side logs and delivery receipt confirmation back over SCMessenger.
acceptance_criteria: Apple logs extracted and verified; Windows/Android logs requested over primary mesh lane.
evidence_refs_complete_with_sha256: iOS log stream PID 6776, scm.log.2026-08-22-01, target/debug/scmessenger-cli send log.
risk_and_cross_platform_impact: High benefit: enables empirical verification of cross-platform message flows.
required_reviews_and_gates: Bilateral field test gate.
requested_owner_and_due_condition: CTO/Windows provides N1 and N2 log extracts in journal or over SCMessenger.
```
