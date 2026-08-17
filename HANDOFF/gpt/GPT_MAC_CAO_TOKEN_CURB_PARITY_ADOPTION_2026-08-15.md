# GPT-MAC CAO token-curb and Apple parity adoption plan

Status: ACTIVE - planning baseline only; no application code or live runtime changed
Last updated: 2026-08-15
Owner: GPT-MAC lane, Chief Apple Officer (CAO)
Scope: iOS, macOS CLI, desktop integration, and their cross-node contracts

## 0. Role decision and operating boundary

Use **CAO (Chief Apple Officer)** for this lane. CIO is a common Chief
Information Officer title and is less precise for ownership of Apple platform
behavior, Xcode, CoreBluetooth, signing, and macOS launchd evidence.

The lane owns Apple-platform implementation and authoritative `xcodebuild`
evidence. It does not own Windows Rust, Android, cloud-node deployment, merge to
`main`, release tags, or changes under `core/src/{crypto,transport,routing,privacy}`
without the Windows AUDIT-GATE. A cloud node is a full node; store-and-forward
custody is a behavior, not a separate node role.

The PR139 handoff remains HARD NO-GO. This plan does not deploy or activate a
responder, mutate contacts or devices, reload launchd or services, send or reply
to SCM messages, or claim delivery, acknowledgement, wake success, or
production readiness.

## 1. Evidence and provenance baseline

### Source anchor

- Inspection source: current `HEAD` `a29e53f3` on
  `gpt/ios-macos-launch-debug-20260810`.
- The checkout is shared and dirty. Existing modifications, especially
  `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift`, are not owned by
  this plan and must not be reverted, stashed, overwritten, or folded into a
  new commit.

### Artifact and runtime anchors

- No new runtime or device evidence was produced by this plan.
- The checked-in XCFramework headers are an artifact anchor only and must not
  be treated as the current source interface.
- Historical iPhone and macOS launch evidence in `HANDOFF/gpt/` remains
  historical until rerun against a named source commit and a fresh artifact.
  Source commit, generated binding digest, packaged artifact digest, and
  runtime/device evidence are separate fields in every future report.

### Static findings verified in this checkout

1. `iOS/SCMessenger/SCMessenger/Generated/api.swift:4447` exposes
   `MeshService.startSwarm(listenAddr:bootstrapAddrs:)`, while both packaged
   headers at `iOS/SCMessengerCore.xcframework/*/Headers/SCMessengerCore.swift:2289`
   expose only `startSwarm(listenAddr:)`. The generated file also exposes
   `encodeReceipt` and `decodeReceipt` that are absent from the packaged header.
   `project.pbxproj:443-461` builds the Rust static library directly and
   `project.pbxproj:494` compiles the generated Swift file, so this is primarily
   a stale-package/release-artifact risk today, not proof that the current app
   build is broken.
2. `MeshRepository.swift:981-987` passes ledger-derived bootstrap addresses,
   but `:962` and `:985` still publish/listen on hardcoded TCP port 9001.
   Actual-bound-address propagation is not yet proven.
3. `mDNSServiceDiscovery.swift:33` browses two service families and
   `:199` creates an `mdns-<name>` transport hint. That hint must never become
   an authenticated identity without core identity confirmation.
4. `SmartTransportRouter.swift:68-369` keeps an app-local transport score and
   races Multipeer, BLE, mDNS/TCP, and Internet candidates. The shared core
   routing decision and the Swift score must have one documented authority;
   duplicate authorities are not parity.
5. `MeshBackgroundService.swift:52-203` uses opportunistic
   `BGTaskScheduler`, which is the correct iOS constraint but cannot promise an
   Android-style continuous foreground custody service. `Info.plist:50-58`
   declares background modes, but no tracked Apple entitlements or APNs
   registration path was found.
6. `NotificationBackgroundProcessor.swift:37-45` simulates a background fetch,
   and `:163-170` returns a simulated network/power result. This file is in the
   application target and is used by tests; it cannot be evidence of real
   background or network functionality.
7. `cli/src/ble_daemon.rs:238-280` returns a simulated scan and accepts
   advertising without sending a GATT advertisement. `cli/src/ble_mesh.rs:555-583`
   explicitly leaves macOS peripheral advertising unimplemented. The macOS CLI
   therefore has a real central path but not full desktop BLE role parity.
8. `desktop_bridge/src/lib.rs:47-54` and
   `desktop_bridge/src/desktop_bridge.rs:226-270` make native desktop integration
   Linux-only or return non-Linux stubs. This is a desktop-integration gap,
   distinct from the macOS CLI's Rust/libp2p node path.
9. The iOS XCTest target is registered in `project.pbxproj:372-390` and its
   sources are listed at `:510-518`, but the historical 47-test result is not a
   current-head gate. Re-run build and test after every Apple source or binding
   change.

### Fresh Mac baseline gate (2026-08-15)

These commands ran against the dirty checkout's current source; they are
baseline evidence, not proof for an uncommitted future diff:

```text
xcodebuild build -project iOS/SCMessenger/SCMessenger.xcodeproj -scheme SCMessenger -configuration Debug -destination 'platform=iOS Simulator,id=A5B9D0CC-B5DD-4E3A-9298-C88D4C753177' CODE_SIGNING_ALLOWED=NO
** BUILD SUCCEEDED **

xcodebuild test -project iOS/SCMessenger/SCMessenger.xcodeproj -scheme SCMessengerTests -configuration Debug -destination 'platform=iOS Simulator,id=A5B9D0CC-B5DD-4E3A-9298-C88D4C753177' CODE_SIGNING_ALLOWED=NO
SCMessenger/api.swift:11763: Fatal error: UniFFI API checksum mismatch: try cleaning and rebuilding your project
** TEST FAILED **
```

Verbatim logs are retained under `tmp/cao-xcodebuild-baseline-20260815.log`
and `tmp/cao-xcodebuild-test-baseline-20260815.log`; they are local evidence
and must not be committed. This confirms P0-A is active: a clean app compile is
not sufficient because the test host traps before establishing its connection.

## 2. Token-curb policy

The scarce-resource rule is **free, narrow, evidence-backed dispatch first;
protected reasoning only for decisions that justify it**.

### Routing tiers

| Work | First lane | Rules |
| --- | --- | --- |
| Inventory, grep-backed fact extraction, formatting, one-line docs or config | agy Gemini Flash, exact pinned model | Read-only or one-file bounded scope; no architecture judgement; `--add-dir` is mandatory. |
| Small/medium settled Swift test or docs implementation | A newly keyed free API lake through the canonical dispatcher, or agy Gemini Flash medium for a bounded file | Existing API/request shape must be named; diff mode; no build by worker; owner runs the Mac gate. |
| Architecture, security, delivery truth, FFI contract, or ambiguous parity | THINK/MAX lane or Fusion Lite panel | Never route analysis/judgement to FLASH. Escalate API-contract or security changes. |
| Protected external pool | agy `claude-sonnet-4-6`, agy `claude-opus-4-6-thinking`, or `gpt-oss-120b-medium` | Treat the three as one scarce pool per operator direction. Use only for a named priority decision or repeated free-lane failure. Never select OSS by default. |

Observed local agy inventory on 2026-08-15 was version 1.1.12 with pinned
Gemini models including `gemini-3.7-flash-low`, `gemini-3.7-flash-medium`, and
`gemini-3.1-pro-high`. Availability is not a quota guarantee; record each
invocation and do not infer remaining capacity from the model list.

The canonical `scripts/delegate_task.py` provider choices do not currently
include `agy`. Until an adapter is reviewed and added, agy is an explicit
platform-local backend, not a silently substituted provider. Every invocation
must record `lake=agy`, exact model, task ID, result, and token counts when
available in `tmp/lakes/ledger.jsonl`. The first bounded Gemini inventory used
`gemini-3.7-flash-low`, made no file changes, and was recorded as
`agy/gemini-3.7-flash-low` with `RESULT: DONE` and `VERIFICATION: NONE`.

### Free API signup and wiring order

The operator may create independent local keys for the existing documented
candidate lakes. Recommended order is Mistral/Codestral, NVIDIA NIM, then
SambaNova or ModelScope. Store keys only in
`~/.config/scmorc/<lake>.env`; never paste or commit them. A lake is not
considered active until its endpoint, exact model ID, rate limits, error
mapping, and ledger recording have been probed. Do not add undocumented
providers to the router merely because a signup succeeded.

## 3. Orchestration adoption for GPT-MAC

The lane adopts the 2026-08-14 dynamic orchestration rules without taking over
an active PR139 orchestrator:

1. There is one canonical orchestration loop and one shared ledger. Do not
   create a second Apple-specific queue protocol or duplicate SCM/PR messages.
2. Delegate project work. The coordinator writes only task packets, evidence,
   and plan/state documents; implementation workers write scoped source/test
   diffs. The CAO lane independently runs the authoritative Mac gates and
   reviews the applied diff.
3. Before every local agy worker or Xcode/build launch, read fresh telemetry,
   run `scripts/resource_admission.py snapshot`, reserve the task-sized peak
   plus the default 10 percent margin, bind the process, sample the full tree,
   and release only after cleanup. Keep at most three direct workers and one
   build at a time. Unknown telemetry or unavailable capacity is BLOCKED, not
   a guess.
4. Every dispatch packet has exactly one hypothesis, exact scope, canonical
   path/API anchor, objective acceptance test, stop condition, permitted and
   forbidden tools, task kind, peak estimate, margin, reservation ID, and
   operator exception wording when applicable.
5. Workers use the structured footer:
   `RESULT`, `VERIFICATION: NONE`, `FILES`, and `NOTES`. A worker claim never
   substitutes for the CAO's real gate. Zero-diff or degraded output is
   requeued, not accepted.
6. Any implementation touching a UniFFI/UDL surface is regenerated from the
   canonical source; generated Swift/C bindings are never hand-edited. Any
   core transport/routing/privacy change returns to the Windows AUDIT-GATE.
7. Record source, generated artifact, packaged artifact, and runtime evidence
   separately. A queue admission, process health, local API response, or
   sender-side log is not delivery evidence.
8. Keep the Mac capability authority explicit: `xcodebuild` is authoritative
   for iOS; Windows is authoritative for Rust, Android, AWS, and Windows
   runtime gates; macOS CLI runtime evidence is authoritative only for the
   macOS lane.

The agy command contract is always explicit:

```text
agy --add-dir <repo> --model <exact-model> --effort <low|medium|high> \
    --mode <plan|accept-edits> --print-timeout 300s
```

Read-only work uses `--mode plan` and no file blocks. An unattended run must
use the platform's approved noninteractive permission setting. Never rely on
agy's default model or allow an automatic fallback to a protected Claude or
OSS model.

## 4. Apple behavioral parity target

Parity means equal node behavior and wire/security semantics, not pretending
that OS APIs are identical:

- **Identity and trust:** the same Rust identity, authenticated sender key,
  device binding, block policy, nickname rules, and safety-number semantics.
- **Message truth:** the same envelope, outbox, retry/claim/completion,
  receipt encoding/decoding, duplicate suppression, and receiver-backed proof.
  Swift/Kotlin/CLI adapters must not invent codecs or identity mappings.
- **Discovery and ledger:** invite/QR-seeded ledger sharing and gossip are the
  source of candidates. Configured addresses are candidates, not identity.
  mDNS, Multipeer display names, BLE addresses, and device UUIDs are transport
  hints until core authenticates them.
- **Transport:** actual bound addresses are advertised; no fixed-port or
  stale-artifact assumption is allowed. Direct BLE, Multipeer/LAN, QUIC/TCP,
  and circuit paths must feed one core routing/custody contract.
- **Background behavior:** Android foreground execution, iOS BGTaskScheduler,
  CoreBluetooth background modes, macOS launchd, and cloud-node service
  supervision are different mechanisms with explicit capability limits. iOS
  cannot claim continuous custody merely because a background task is queued.
- **Diagnostics:** all nodes emit sanitized message ID/hash, transport, route,
  source commit, artifact digest, and terminal state. No payloads, full peer
  IDs, private addresses, secrets, or device identifiers.

Honest capability mapping:

| Capability | Android | iOS | macOS CLI | Cloud node |
| --- | --- | --- | --- | --- |
| Shared identity/message core | required | required through UniFFI | required directly | required directly |
| BLE | central/peripheral, quota-managed | CoreBluetooth central/peripheral | central today; peripheral gap | not applicable |
| Wi-Fi Aware/Direct APIs | native | unsupported; must fail closed | unsupported | unsupported |
| Apple local equivalent | not applicable | Multipeer + mDNS/LAN | mDNS/LAN + libp2p | internet libp2p |
| Background custody | foreground service | opportunistic BGTask/CoreBluetooth/APNs design | launchd daemon | service supervisor |
| Notifications | Android notification service | local notification plus APNs gate | desktop notification path | no UI assumption |

## 5. Prioritized work packages

### P0-A: Binding and packaged-artifact parity

- Scope: generated Swift/C files, `scripts/copy-bindings.sh`,
  `scripts/build_xcframework.sh`, `scripts/verify_ios_bindings.sh`, and the
  checked-in XCFramework headers. Do not hand-edit generated outputs.
- Canonical anchor: `core/src/api.udl` and the three binding verification
  scripts.
- Acceptance: generated binding diff is empty; both XCFramework slices expose
  the same API; `xcodebuild build` and `xcodebuild test` pass on a resolved
  simulator; source and artifact digests are recorded separately.
- Stop: any UDL/API contract change, core transport change, missing Apple
  account, or generated-output ambiguity; route to Windows AUDIT-GATE or the
  operator.

### P1-A: Actual-address discovery and startup parity

- Scope: `MeshRepository.swift`, `mDNSServiceDiscovery.swift`, the matching
  Android repository/discovery files, and the shared FFI call sites. The
  current ledger-sourced bootstrap work is preserved; do not overwrite the
  dirty MeshRepository change.
- Canonical anchor: `MeshService.startSwarm(listenAddr:bootstrapAddrs:)`,
  `getListeners()`, `LedgerManager.getPreferredRelays`, and the mDNS TXT
  identity hint.
- Acceptance: no production hardcoded Apple listen/advertise port; the actual
  bound multiaddrs are propagated to discovery and ledger; mDNS/Multipeer
  hints cannot authenticate a user; matched iOS/Android/macOS tests prove
  cold-start, restart, and address change behavior.
- Stop: any shared core transport/routing or wire/API change; require the
  Windows security review and operator architecture approval.

### P1-B: macOS BLE role and failure contract

- Scope: `cli/src/ble_daemon.rs`, `cli/src/ble_mesh.rs`,
  `cli/src/ble_windows.rs`, and a narrowly designed macOS adapter only after
  the platform approach is approved; no live device mutation.
- Canonical anchor: `core/src/transport/ble/gatt.rs` DF01/DF02/DF03 service,
  identity, framing, fragmentation, receipt/sync, and stable identity rules.
- Acceptance: macOS central and, if required by the node contract, peripheral
  paths have real GATT behavior; simulated scan/advertising paths are not used
  in production; callback work is off the CoreBluetooth callback thread; the
  receiver proves both directions, fragmentation, reconnect, MAC rotation,
  and bounded failure recovery with relay/LAN disabled.
- Stop: any new native dependency or macOS peripheral architecture without
  operator approval; never turn a simulated result into a PASS.

### P1-C: iOS background and notification truth

- Scope: `MeshBackgroundService.swift`, `NotificationManager.swift`,
  `NotificationBackgroundProcessor.swift`, `Info.plist`, Apple entitlements,
  and the device test packet. No APNs deployment until the Apple account/team
  gate is resolved.
- Canonical anchor: BGTask identifiers, expiration handlers, outbox flush,
  CoreBluetooth background modes, and APNs remote-notification contract.
- Acceptance: test-only simulation is separated from production evidence;
  BGTask expiration is bounded and durable; APNs registration and a signed
  device wake are either proven or explicitly marked unavailable; receiver
  evidence distinguishes local notification display, wake admission, core
  execution, and message receipt.
- Stop: missing signing/team/profile, any live send requirement, or any claim
  that queued background work equals delivery.

### P2-A: One routing authority and test maturity

- Scope: `SmartTransportRouter.swift`, `LocalTransportFallback.swift`,
  transport adapters, and focused XCTest fixtures. No Rust routing edits in
  the Mac lane.
- Canonical anchor: shared core route/custody result and the adapter's
  `recordSuccess`/`recordFailure` boundary.
- Acceptance: one authoritative route result; a verified BLE or Multipeer
  success cannot be overwritten by a later no-route candidate; callback
  deduplication has race tests; every test fixture names whether it is
  simulated, mocked, or device-backed.
- Stop: any core routing change or unresolved race; escalate rather than
  broadening the patch.

## 6. Definition of mature completion

An Apple task is complete only when all of these are true:

1. The diff is narrowly scoped and contains no production simulation,
   placeholder, or fake-success path.
2. The source commit and generated/package artifact are named separately.
3. `xcodebuild build` and `xcodebuild test` are pasted verbatim for current
   source on the Mac; historical output is not reused.
4. The applied diff receives an independent review when it touches transport,
   identity, privacy, delivery, or FFI contracts.
5. The capability matrix says `implemented`, `wired`, `proven`, or
   `unsupported-by-platform` for every affected OS.
6. Device-backed acceptance uses receiver `inbox_receive` and an exact
   authenticated receipt. Sender status, queue admission, local API success,
   process health, and notification display alone are insufficient.
7. The result is recorded in the shared ledger and handoff with sanitized
   evidence and no secrets.

## 7. Operator decisions requested before implementation

1. Which free API accounts should be enabled first: Mistral/Codestral,
   NVIDIA NIM, SambaNova, or ModelScope? Keys should remain local; names and
   model IDs are sufficient for the lane packet.
2. Should agy be added as a reviewed canonical backend adapter, or remain an
   explicit Mac-only read-only backend until its quota and provider boundary
   are observable?
3. Is full macOS BLE peripheral parity required for the v1.0.0 farm, or is
   macOS-central plus other transports an operator-approved capability waiver?
4. Which Apple Developer team/account will authorize APNs entitlements and a
   signed physical-device wake test?
5. Confirm that P0-A binding/artifact parity is the first Apple implementation
   gate before new transport work.

Until these decisions are answered, the CAO lane will perform only bounded
read-only audits, documentation, and already-specified tests. No application
source is changed by this plan.
