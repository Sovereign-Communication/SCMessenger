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
