# SCMessenger Phase-2 Prescriptions — Jev-selected implementation patterns

Post-0.5.0 refresh. Model: `jev-1.13.0`. 5581 prescription judgments, 5507 context-needs judgments. 0 API errors.
Tables `prescriptions`, `context_needs`, `battery_security`, `battery_testplan`,
`battery_concurrency` in `audit.db`.

## Execution-ready (>=0.95, generated/test excluded)

- **0.99** `process_gossip` (rust, core/src/routing/neighborhood.rs:224) [security] -> **validate-guard**
- **0.97** `on_wifi_direct_connection_info` (rust, core/src/mobile_bridge.rs:1809) [security] -> **validate-guard**
- **0.96** `readCharacteristic` (swift, iOS/SCMessenger/SCMessenger/Transport/BLECentralManager.swift:687) [wiring] -> **async-await**
- **0.96** `temp_storage_path` (rust, wasm/src/lib.rs:2364) [wiring] -> **propagate-question**
- **0.95** `on_proximity_data_received` (rust, core/src/mobile_bridge.rs:6040) [security] -> **validate-guard**

## Strong signal (0.80-0.95, generated/test excluded) — lane review required

- 0.94 `identity_hash_not_usable_as_recipient` (rust, core/src/iron_core.rs:5451) [security] -> validate-guard
- 0.94 `on_proximity_data_received` (rust, core/src/mobile_bridge.rs:5577) [security] -> validate-guard
- 0.93 `onCharacteristicReadRequest` (kotlin, android/app/src/main/java/com/scmessenger/android/transport/ble/BleGattServer.kt:294) [security] -> guard-early-return
- 0.93 `wp1_non_key_recipients_are_not_encryptable` (rust, core/src/iron_core.rs:6278) [security] -> validate-guard
- 0.93 `on_proximity_data_received` (rust, core/src/mobile_bridge.rs:5728) [security] -> validate-guard
- 0.93 `on_proximity_data_received` (rust, core/src/mobile_bridge.rs:5889) [security] -> validate-guard
- 0.93 `perform_sync` (rust, wasm/src/mesh.rs:215) [wiring] -> propagate-question
- 0.92 `on_ble_data_received` (rust, core/src/mobile_bridge.rs:5565) [security] -> validate-guard
- 0.92 `chain_hash` (rust, core/src/observability.rs:128) [wiring] -> propagate-question
- 0.92 `new` (rust, core/src/store/contacts.rs:136) [wiring] -> propagate-question
- 0.91 `register_identity_with_relay` (rust, cli/src/main.rs:52) [wiring] -> propagate-question
- 0.91 `self_certifying_peer` (rust, cli/src/main.rs:991) [wiring] -> propagate-question
- 0.91 `serve_apk_stream` (rust, cli/src/main.rs:1400) [security] -> validate-guard
- 0.91 `make_keypair_pubkey_and_identity_id` (rust, core/src/mobile_bridge.rs:6517) [security] -> propagate-question
- 0.91 `advertiser` (swift, iOS/SCMessenger/SCMessenger/Transport/MultipeerTransport.swift:402) [security] -> guard-let
- 0.90 `ensureServiceInitializedFireAndForget` (kotlin, android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:6359) [wiring] -> coroutine-scope
- 0.90 `on_ble_data_received` (rust, core/src/mobile_bridge.rs:5716) [security] -> validate-guard
- 0.90 `on_proximity_data_received` (rust, core/src/mobile_bridge.rs:6468) [security] -> validate-guard
- 0.90 `drain_for_peer` (rust, core/src/store/outbox.rs:838) [wiring] -> propagate-question
- 0.90 `bootstrap` (rust, core/src/transport/bootstrap.rs:205) [security] -> validate-guard
- 0.90 `flush` (swift, iOS/SCMessenger/SCMessenger/ContactManagerFix.swift:93) [wiring] -> throws-propagate
- 0.90 `connect` (rust, wasm/src/daemon_bridge.rs:518) [wiring] -> propagate-question
- 0.89 `onRuntimePermissionsGranted` (kotlin, android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:6107) [wiring] -> coroutine-scope
- 0.89 `testCommonPorts` (kotlin, android/app/src/main/java/com/scmessenger/android/network/NetworkDiagnostics.kt:98) [wiring] -> use-block
- 0.89 `sign_token_pq` (rust, core/src/relay/invite.rs:724) [security] -> propagate-question
- 0.88 `ensurePendingOutboxRetryLoop` (kotlin, android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:7816) [wiring] -> coroutine-scope
- 0.88 `isPortOpen` (kotlin, android/app/src/main/java/com/scmessenger/android/network/NetworkDiagnostics.kt:161) [wiring] -> use-block
- 0.88 `handle_service_discovered` (rust, core/src/mobile_bridge.rs:2761) [security] -> validate-guard
- 0.88 `sign_token_pq` (rust, core/src/relay/invite.rs:724) [wiring] -> propagate-question
- 0.88 `remove_group` (rust, core/src/transport/wifi_direct.rs:411) [wiring] -> propagate-question
- 0.88 `count` (swift, iOS/SCMessenger/SCMessenger/ContactManagerFix.swift:81) [wiring] -> throws-propagate
- 0.87 `record_log` (rust, core/src/iron_core.rs:2415) [wiring] -> propagate-question
- 0.87 `dataScanner` (swift, iOS/SCMessenger/SCMessenger/Views/Topics/JoinMeshView.swift:221) [wiring] -> guard-let
- 0.86 `notifyNetworkRecovered` (kotlin, android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:4391) [wiring] -> coroutine-scope
- 0.86 `try_begin_dial` (rust, cli/src/ledger.rs:419) [wiring] -> drop-guard
- 0.86 `register_device_id` (rust, core/src/mobile_bridge.rs:4368) [wiring] -> propagate-question
- 0.86 `updateRelayAvailability` (swift, iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift:6146) [wiring] -> split-function
- 0.86 `clearDiagnostics` (swift, iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift:6773) [wiring] -> throws-propagate
- 0.85 `resolveKnownPeerNickname` (kotlin, android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:9624) [wiring] -> split-function
- 0.85 `readHttpRequestLine` (kotlin, android/app/src/main/java/com/scmessenger/android/utils/ApkShareManager.kt:293) [wiring] -> use-block

(102 total at >=0.80 excluding generated/test; top 40 shown.)

## Batch by pattern x language (wiring facet, non-no-change)

### rust
- propagate-question: 644 functions (avg conf 0.47)
- validate-guard: 155 functions (avg conf 0.38)
- handle-local-fallback: 142 functions (avg conf 0.37)
- drop-guard: 136 functions (avg conf 0.36)
- state-machine: 60 functions (avg conf 0.38)
- return-value: 54 functions (avg conf 0.34)
- split-function: 35 functions (avg conf 0.34)
- channel-event: 30 functions (avg conf 0.36)

### kotlin
- result-sealed: 195 functions (avg conf 0.37)
- guard-early-return: 183 functions (avg conf 0.42)
- split-function: 115 functions (avg conf 0.45)
- result-stdlib: 104 functions (avg conf 0.38)
- coroutine-scope: 102 functions (avg conf 0.46)
- mutex-guard: 66 functions (avg conf 0.4)
- flow-collect: 34 functions (avg conf 0.38)
- use-block: 27 functions (avg conf 0.53)

### swift
- guard-let: 104 functions (avg conf 0.43)
- split-function: 89 functions (avg conf 0.42)
- result-type: 67 functions (avg conf 0.35)
- async-await: 60 functions (avg conf 0.42)
- throws-propagate: 48 functions (avg conf 0.48)
- defer-cleanup: 18 functions (avg conf 0.39)
- task-cancel: 8 functions (avg conf 0.43)

## Battery headlines (post-0.5.0)

- Security: input not validated: 150
- Security: fail-open risk: 45
- Testplan: hard-to-test: 10
- Concurrency: bug likely+: 20
- Context: critical need: 153

