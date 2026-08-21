# Four-node v0.4/v0.5 bilateral gate contract

Status: Active contract; no candidate or runtime attempt has been frozen
Coordination ID: `AW-BILAT-0001`
Gate contract ID: `AW4N-V040-V050-GATE-0001`

This focused four-node gate is additional to, and cannot replace, the separate
five-node cloud-node custody gate. Each endpoint is a full node that can perform
 custody and forwarding behavior. No endpoint is a distinct forwarding-only role.
Amendments are new versioned events approved by both lanes, never silent edits.

## Identity and topology

Attempt IDs are append-only:
`AW4N-V040-V050-GATE-0001-R<two-digit-runtime>-P<two-digit-pass>`. A runtime
code SHA change increments `R` and resets `P`; a repeat with no runtime drift
increments `P`. Documentation-only coordination commits do not change `R`.

All endpoints use the same non-guest IPv4 subnet during LAN testing; runtime
addresses are recorded then, never copied from a handoff.

| Node ID | Endpoint | Test role | Evidence owner |
| --- | --- | --- | --- |
| `N1-WIN-CLI` | Windows `scmessenger-cli.exe` | Normal messaging endpoint, driver, API/log observer | CTO/Windows |
| `N2-AND-PIXEL` | Physical Pixel 6a Android app | Normal mobile endpoint; QR, LAN, BLE, requests, UI, lifecycle | CTO/Windows |
| `N3-MAC-CLI` | macOS CLI | Normal messaging endpoint, driver, API/log observer | CAO/Apple |
| `N4-IOS-PHONE` | Physical iPhone app | Normal mobile endpoint; QR, LAN, BLE, requests, UI, lifecycle | CAO/Apple |

## Candidate freeze prerequisite

Both journals must first contain reciprocal `CANDIDATE_ACK` events with all
fields below. Commit/tree/checksums/artifact hashes are truth; branches are
locators. Uncommitted runtime changes are ineligible.

```text
coordination_id; gate_contract_id; test_id; release_scope: V040 | V050_REGRESSION
candidate_branch; candidate_commit_full_sha; candidate_tree_sha
candidate_diff_sha256_from_last_accepted_runtime; core_source_commit_full_sha
swift_binding_checksum; kotlin_binding_checksum; windows_cli_artifact_sha256
android_apk_sha256; macos_cli_artifact_sha256; ios_app_source_sha_and_archive_uuid
apple_handoff_branch; apple_handoff_record_commit; windows_handoff_branch
windows_handoff_record_commit; node_versions_and_os_builds
node_identity_fingerprints_redacted; utc_clock_offsets; collector_preflight_status
```

Runtime code, generated binding, build flag, or dependency drift invalidates
the attempt through reciprocal `CANDIDATE_INVALIDATED` events and requires the
next runtime ID. Neither lane continues because only its side did not change.

## M00-M20 evidence matrix

Allowed states: `PASS`, `FAIL`, `BLOCKED-EVIDENCE`, `BLOCKED-HW`, and
`NOT-IN-RELEASE-SCOPE`. Only the operator may approve a release waiver; it
remains visibly waived. v0.4 records v0.5-only rows as
`NOT-IN-RELEASE-SCOPE`, never `PASS`.

| ID | Required procedure | Required PASS evidence |
| --- | --- | --- |
| M00-PROVENANCE | All N1-N4 before traffic | Exact candidate, artifact, and binding fields match; each lane owns its nodes and both acknowledge. |
| M01-COLLECTORS | Start collectors/watchdogs before traffic; survive restart and cover whole window | Per-node heartbeats, start/end coverage, no eviction/truncation. |
| M02-IDENTITY | In-place update and restart without re-pair/wipe | Before/after redacted fingerprints and persisted contacts/history. |
| M03-FLEET | All nodes converge on other three, then reconverge after sequential restart | Full uncapped peer snapshots and discovery/ledger chain. |
| M04-QR-A2I | Android export; iPhone scans/imports; no unchecked auto-dial | Android provenance plus iOS validation, confirmation, contact result. |
| M05-QR-I2A | iPhone export; Android scans/imports; no unchecked auto-dial | iOS provenance plus Android validation, confirmation, contact result. |
| M06-LAN-ALL-PAIRS | Both directions of six pairs; 12 messages and receipts | Sender enqueue/send/outbox plus receiver ingest/decrypt/history/receipt and sender convergence per direction. |
| M07-MDNS | Production service types; stop/start discovery once | Full service type, peer fingerprint, pinned address, dial result, no duplicate amplification. |
| M08-BLE-A2I | Wi-Fi disabled on phones; both mobile directions | Central/peripheral role, service/characteristic, fragments/reassembly, ingest, receipt. |
| M09-BLE-CAPABILITY | Assess all six pairs; execute each supported pair/direction | Capability manifest and supported evidence; `N/A` needs exact adapter/source/hardware proof and operator acknowledgment. |
| M10-REQUEST-A2I | Unknown Android sender; iOS Accept preserves message and enables reply | Durable iOS request/accept/contact/history and bidirectional receipts. |
| M11-REQUEST-I2A | Unknown iOS sender; Android Accept preserves message and enables reply | Reciprocal durable evidence and receipt. |
| M12-REJECT-BLOCK | Each mobile rejects a new unknown sender; confirmation-gated block/delete, unblock, restart | State before/after/reload; no stale resurrection. |
| M13-DELIVERY-TRUTH | Queued, custody/forwarding observed, sent, receipt-delivered, retryable transient, terminal identity rejection | No delivered display without receipt; UI/repository correlation. |
| M14-OFFLINE-OBLIGATION | Each recipient offline in turn; sender restart; wait beyond old thresholds; restore | Same durable message remains eligible, delivers once, no abandonment/duplicate history. |
| M15-ROUTE-CHANGE | Toggle Wi-Fi/BLE and viable LAN route without restart | Bounded recovery, obligation promotion, candidate ladder, receipt. |
| M16-RESTART | Restart N1-N4 individually during queued/settled states | Collector survival, stable identity, reconvergence, drain, stable delivered state. |
| M17-FOUR-NODE-CHURN | All four concurrent for 30 minutes | No panic, ANR, crash, dead node, storm, flood, loss, duplicate, false delivery. |
| M18-BACKGROUND-NOTIFY | Supported mobile background/foreground notification behavior | Actual OS evidence and route; unsupported APNs remains blocked. |
| M19-DIAGNOSTICS | Export after run from both apps and CLI nodes | Complete run ID/SHA/route/error data with secrets/message content redacted. |
| M20-SOAK | 60 minutes after two complete matrix passes, periodic messages and one restart/network event | Zero unaccounted messages, false delivery, crash/panic/ANR, collector gap, identity drift; CAO/CTO co-sign. |

Each directional message uses a fresh opaque correlation ID and UTC send time.
Sender-lane evidence proves acceptance/outbox; receiver-lane evidence proves
ingest/decrypt/history/receipt. Local transport acceptance is not delivery
truth. CAO may pass/block Apple-owned evidence only; CTO may pass/block
Windows/Android-owned evidence only. Fresh independent EVIDENCE scoring and
CRITICAL_VALIDATOR delivery/custody/identity/transport/block/request/retry
adjudication are required; uncertainty fails closed. Windows owns final gate
disposition but cannot overrule any lane block, critical block, or missing
evidence.

## Per-node artifact manifest and redaction

Before a run each lane creates `tmp/field-tests/<TEST_ID>/` with
`run-manifest.json`, `matrix-results.json`, and for each N1-N4:
`artifact-manifest.json`, `collector-health.json`, `provenance.json`,
`events.jsonl`, `peers-before.json`, `peers-after.json`, `outbox-before.json`,
`outbox-after.json`, `crash-index.json`, and `raw/`. N2 includes logcat,
bugreport, and ANR material under `raw/`; N3 includes CLI/unified/process
material; N4 includes unified-log archive, diagnostic export, and crash reports.

`events.jsonl` is lossless and contains test ID, node ID, monotonic sequence,
UTC timestamp, correlation ID, event kind, transport, peer fingerprint,
delivery state, and raw byte/line locator (or query/time range). No capped API
result, `head`, `tail`, ring eviction, or summary replaces complete evidence.
The evidence worker hashes every file and checks coverage, sequence,
correlation completeness, restart continuity, heartbeat, redaction, and raw
locator validity. A missing/deaf collector is `BLOCKED-EVIDENCE`.

Raw logs, screenshots with identifiers, device exports, secrets, message
bodies, complete identities, serials, build artifacts, and private evidence
stay out of git. Tracked redacted summaries retain locator, SHA-256, byte size,
UTC coverage, collector health, redaction status, and retaining lane.

## Merge-resilient polling

Each controller runs, without output caps:

```bash
git fetch origin --prune
git log --all --format='%H %cI %D %s' -S'<IMMUTABLE_ID>' -- HANDOFF/coordination/apple-windows/
git show <OBSERVED_COMMIT>:HANDOFF/coordination/apple-windows/CAO_TO_CTO.md
git show <OBSERVED_COMMIT>:HANDOFF/coordination/apple-windows/CTO_TO_CAO.md
git show origin/main:HANDOFF/coordination/apple-windows/INDEX.md
```

The origin appends immutable ID, sequence, origin/target, candidate, scope,
evidence, disposition request, and next action. The target appends `RECEIVED`
plus `ACCEPTED`, `DECLINED`, `DEFERRED`, or `BLOCKED` with rationale, owner,
evidence, and exact origin record commit. Corrections are new events. Windows
indexes paired records after observing both. Both lanes post collector start,
`READY`, step completion, fail, stop, manifest hash, and verdict acknowledgment;
steps advance only after both acknowledge the same preceding event.

## Advisory and Rust/core prerequisite

CAO may recommend Windows/Android work and CTO may recommend Apple work; the
target lane exclusively plans and implements its paths. Advisories cannot
authorize architecture, security/privacy tradeoffs, API breaks, release timing,
account action, merges, or cross-lane edits. Owners check overlapping open
advisories before merge.

Every Rust/core, desktop-bridge, CLI Rust API, generated-binding input, or Cargo
dependency/feature change requires exact-commit CTO and CAO `APPROVAL` events
with full source commit, complete path list, and scoped diff SHA-256. These are
additive to operator, independent adversarial/security, generated-binding,
authoritative Windows, Apple Xcode, delivery critical-review, and Windows
merge/tag/release gates. Absence, staleness, unresolved critical findings, or a
failed gate is `BLOCKED`.
