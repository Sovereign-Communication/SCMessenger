# SCMessenger Phase-2 Prescriptions — Jev-selected implementation patterns

Model: `jev-1.13.0`. 5,476 prescription judgments (5,076 wiring, 400 security) and
5,407 context-needs judgments across 5,407 flagged functions (Rust/Kotlin/Swift).
0 API errors, 0 missing answers. Tables `prescriptions`, `context_needs` in `audit.db`.
Raw: `phase2/prescriptions.jsonl`, `phase2/context.jsonl`.

## Headline findings

- Prescription confidence runs LOW (wiring mean 0.42, security mean 0.56) — consistent
  with the Jev calibration battery: Jev is a veto/escalation sensor, not an approver.
  Under the 0.95 gate, only **5 prescriptions are execution-ready** (below).
- 36% of wiring-flagged functions got `no-change`: the wiring flag is noisy, the
  prescription pass correctly filters it.
- Security flags are nearly all real: only 2% `no-change`.
- Context hunger is high: mean `needs_more_context` 0.78; 1,949 functions score
  important-or-critical context need. Jev most often wants a **callee** (2,326),
  then a **caller** (1,700), then a **sibling** (1,244).
- Context need is uncorrelated with prescription confidence (r=0.05): re-judging
  with more context is a separate workstream (phase 2d), not a fix for low confidence.

## Execution-ready (>=0.95, generated/test excluded)

- **0.98** `process_gossip` (rust, core/src/routing/neighborhood.rs:219) [security] -> **validate-guard**
- **0.98** `readCharacteristic` (swift, iOS/SCMessenger/SCMessenger/Transport/BLECentralManager.swift:687) [wiring] -> **async-await**
- **0.97** `chain_hash` (rust, core/src/observability.rs:128) [wiring] -> **propagate-question**
- **0.96** `on_wifi_direct_connection_info` (rust, core/src/mobile_bridge.rs:1803) [security] -> **validate-guard**
- **0.95** `remove_group` (rust, core/src/transport/wifi_direct.rs:401) [wiring] -> **propagate-question**

## Strong signal (0.80-0.95, generated/test excluded) — lane review required

- 0.94 `make_keypair_pubkey_and_identity_id` (rust, core/src/mobile_bridge.rs:6372) [security] -> propagate-question
- 0.94 `handle_message` (rust, core/src/wasm_support/transport.rs:132) [security] -> validate-guard
- 0.94 `temp_storage_path` (rust, wasm/src/lib.rs:2364) [wiring] -> propagate-question
- 0.93 `onCharacteristicReadRequest` (kotlin, android/app/src/main/java/com/scmessenger/android/transport/ble/BleGattServer.kt:294) [security] -> guard-early-return
- 0.93 `self_certifying_peer` (rust, cli/src/main.rs:990) [wiring] -> propagate-question
- 0.93 `identity_hash_not_usable_as_recipient` (rust, core/src/iron_core.rs:5436) [security] -> validate-guard
- 0.93 `sign_token` (rust, core/src/relay/invite.rs:713) [security] -> propagate-question
- 0.93 `bootstrap` (rust, core/src/transport/bootstrap.rs:205) [security] -> validate-guard
- 0.92 `stop_discovery` (rust, core/src/transport/wifi_direct.rs:384) [wiring] -> propagate-question
- 0.91 `startMonitoring` (kotlin, android/app/src/main/java/com/scmessenger/android/service/ServiceHealthMonitor.kt:52) [wiring] -> coroutine-scope
- 0.91 `try_begin_dial` (rust, cli/src/ledger.rs:419) [wiring] -> drop-guard
- 0.91 `serve_apk_stream` (rust, cli/src/main.rs:1399) [security] -> validate-guard
- 0.91 `on_ble_data_received` (rust, core/src/mobile_bridge.rs:5420) [security] -> validate-guard
- 0.90 `notifyNetworkRecovered` (kotlin, android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:4390) [wiring] -> coroutine-scope
- 0.90 `on_proximity_data_received` (rust, core/src/mobile_bridge.rs:5583) [security] -> validate-guard
- 0.90 `on_ble_data_received` (rust, core/src/mobile_bridge.rs:5883) [security] -> validate-guard
- 0.90 `sign_token_pq` (rust, core/src/relay/invite.rs:724) [wiring] -> propagate-question
- 0.90 `connect` (rust, wasm/src/daemon_bridge.rs:518) [wiring] -> propagate-question
- 0.89 `ensureServiceInitializedFireAndForget` (kotlin, android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:6358) [wiring] -> coroutine-scope
- 0.89 `ensurePendingOutboxRetryLoop` (kotlin, android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:7775) [wiring] -> coroutine-scope
- 0.89 `on_ble_data_received` (rust, core/src/mobile_bridge.rs:6311) [security] -> validate-guard
- 0.89 `peripheralManager` (swift, iOS/SCMessenger/SCMessenger/Transport/BLEPeripheralManager.swift:624) [wiring] -> guard-let
- 0.88 `onRuntimePermissionsGranted` (kotlin, android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:6106) [wiring] -> coroutine-scope
- 0.88 `emitDisconnectedIfChanged` (kotlin, android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:7469) [wiring] -> mutex-guard
- 0.88 `resolveKnownPeerNickname` (kotlin, android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:9583) [wiring] -> split-function
- 0.88 `new` (rust, core/src/store/contacts.rs:136) [wiring] -> propagate-question
- 0.88 `set_on_message_received` (rust, core/src/transport/wifi_aware.rs:299) [wiring] -> callback-closure
- 0.87 `register_identity_with_relay` (rust, cli/src/main.rs:51) [wiring] -> propagate-question
- 0.87 `get_external_addresses` (rust, core/src/mobile_bridge.rs:3927) [wiring] -> propagate-question
- 0.87 `on_ble_data_received` (rust, core/src/mobile_bridge.rs:5732) [security] -> validate-guard
- 0.87 `on_proximity_data_received` (rust, core/src/mobile_bridge.rs:5895) [security] -> validate-guard
- 0.87 `set_on_service_discovered` (rust, core/src/transport/wifi_aware.rs:293) [wiring] -> callback-closure
- 0.87 `onMessageReceived` (swift, iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift:1756) [wiring] -> split-function
- 0.87 `perform_sync` (rust, wasm/src/mesh.rs:215) [wiring] -> propagate-question
- 0.86 `isPortOpen` (kotlin, android/app/src/main/java/com/scmessenger/android/network/NetworkDiagnostics.kt:161) [wiring] -> use-block
- 0.86 `createResponderSocket` (kotlin, android/app/src/main/java/com/scmessenger/android/transport/WifiAwareTransport.kt:356) [wiring] -> use-block
- 0.86 `set_on_data_path_confirmed` (rust, core/src/mobile_bridge.rs:2852) [wiring] -> callback-closure
- 0.86 `exportDiagnostics` (swift, iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift:3413) [wiring] -> async-await
- 0.86 `updateRelayAvailability` (swift, iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift:6146) [wiring] -> split-function
- 0.86 `startBatteryMonitoring` (swift, iOS/SCMessenger/SCMessenger/Services/IosPlatformBridge.swift:187) [wiring] -> defer-cleanup

(102 total at >=0.80 excluding generated/test; top 40 shown. Full list in `audit.db`.)

## Batch by pattern x language (wiring facet, non-no-change)

### rust
- propagate-question: 651 functions (avg conf 0.46)
- handle-local-fallback: 154 functions (avg conf 0.37)
- drop-guard: 138 functions (avg conf 0.35)
- validate-guard: 138 functions (avg conf 0.39)
- state-machine: 57 functions (avg conf 0.38)
- return-value: 46 functions (avg conf 0.36)
- split-function: 36 functions (avg conf 0.33)
- complete-match: 32 functions (avg conf 0.38)

### kotlin
- result-sealed: 182 functions (avg conf 0.36)
- guard-early-return: 179 functions (avg conf 0.41)
- split-function: 118 functions (avg conf 0.44)
- coroutine-scope: 106 functions (avg conf 0.44)
- result-stdlib: 98 functions (avg conf 0.39)
- mutex-guard: 59 functions (avg conf 0.41)
- flow-collect: 35 functions (avg conf 0.35)
- use-block: 26 functions (avg conf 0.53)

### swift
- guard-let: 102 functions (avg conf 0.43)
- split-function: 83 functions (avg conf 0.44)
- result-type: 60 functions (avg conf 0.36)
- async-await: 59 functions (avg conf 0.42)
- throws-propagate: 51 functions (avg conf 0.47)
- defer-cleanup: 20 functions (avg conf 0.4)
- task-cancel: 9 functions (avg conf 0.41)
- precondition-invariant: 1 functions (avg conf 0.15)

## Confirmed wiring gaps after contextual rescoring (250 weakest, with caller/callee context)

- Seams: 110/250 confirmed bad (dropped call results/error paths), 94 exonerated by context.
- Caller contracts: mostly uncertain even with context (only 64/250 decisive) — flag as noisy.
- Completeness: 215/250 confirmed stub/partial — the completeness<=1 flags were real.
- Resources: 175/250 fine, only 4 confirmed leaks — resources_cleaned flags were mostly noise.

## Lane grouping: top files by prescription count

- 265 — `core/src/mobile_bridge.rs`
- 254 — `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
- 171 — `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift`
- 135 — `core/src/iron_core.rs`
- 88 — `iOS/SCMessenger/SCMessenger/Generated/api.swift`
- 82 — `core/src/store/ledger_entry.rs`
- 68 — `core/src/transport/swarm.rs`
- 56 — `iOS/SCMessengerCore.xcframework/ios-arm64/Headers/SCMessengerCore.swift`
- 48 — `iOS/SCMessengerCore.xcframework/ios-arm64-simulator/Headers/SCMessengerCore.swift`
- 43 — `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt`
- 42 — `wasm/src/lib.rs`
- 37 — `iOS/SCMessengerTests/NotificationVerificationTests.swift`
- 36 — `android/app/src/test/java/com/scmessenger/android/utils/DeepLinkValidatorTest.kt`
- 35 — `android/app/src/main/java/com/scmessenger/android/service/AndroidPlatformBridge.kt`
- 33 — `core/src/transport/manager.rs`

## Critical context needs (phase-2d candidates)

164 functions score context-need > 2.25 (critical). Top non-test:

- 2.82 `main` (rust, core/src/bin/gen_kotlin.rs) — Jev wants a **sibling**
- 2.81 `getMeshRepository` (kotlin, android/app/src/main/java/com/scmessenger/android/service/MeshSyncWorker.kt) — Jev wants a **caller**
- 2.74 `main` (rust, core/src/bin/gen_swift.rs) — Jev wants a **sibling**
- 2.74 `main` (rust, desktop_bridge/src/bin/gen_kotlin.rs) — Jev wants a **sibling**
- 2.69 `isPublicInternetMultiaddr` (kotlin, android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt) — Jev wants a **caller**
- 2.67 `notify` (kotlin, android/shared/src/commonMain/kotlin/com/scmessenger/shared/platform/PlatformNotifier.kt) — Jev wants a **caller**
- 2.65 `attemptDirectSwarmDelivery` (swift, iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift) — Jev wants a **caller**
- 2.63 `reset` (kotlin, android/app/src/main/java/com/scmessenger/android/utils/BackoffStrategy.kt) — Jev wants a **sibling**
- 2.61 `update_keepalive` (rust, core/src/iron_core.rs) — Jev wants a **caller**
- 2.60 `start` (kotlin, android/shared/src/commonMain/kotlin/com/scmessenger/shared/platform/PlatformNetworking.kt) — Jev wants a **sibling**
- 2.60 `auto_reply_is_limited_to_one_ack_per_message_id` (rust, cli/src/main.rs) — Jev wants a **none**
- 2.59 `cancel_request` (rust, wasm/src/daemon_bridge.rs) — Jev wants a **sibling**

Recommended: fetch the chosen related function and re-run the prescription with
expanded context; iterate until context-need drops below 1.5.

