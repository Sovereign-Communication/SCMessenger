import json

functions = [
    {
        "id": "01_is_poison_circuit_listener",
        "file": "core/src/transport/swarm.rs",
        "fn": "is_poison_circuit_listener",
        "claims": [
            {"claim_id": "C1", "kind": "defect", "text": "Function does not validate that the Multiaddr contains at most one /p2p-circuit component before counting"},
            {"claim_id": "C2", "kind": "defect", "text": "Function returns true for un-tracked reservations with a single circuit, but spec requires false for tracked reservations only"},
            {"claim_id": "C3", "kind": "reassurance", "text": "Function correctly rejects addresses with zero circuit components"},
        ]
    },
    {
        "id": "02_is_self_endpoint",
        "file": "core/src/transport/swarm.rs",
        "fn": "is_self_endpoint",
        "claims": [
            {"claim_id": "C1", "kind": "defect", "text": "Wildcard bind (unspecified IP) claims all IPv6 candidates unconditionally, including public IPv6 relays on same port"},
            {"claim_id": "C2", "kind": "defect", "text": "Port-only matching before IP comparison allows cross-interface false positives when multiple local interfaces share a port"},
            {"claim_id": "C3", "kind": "defect", "text": "No validation that the candidate address is actually reachable via the matched local interface"},
            {"claim_id": "C4", "kind": "reassurance", "text": "Exact IP+port match correctly identifies self endpoints"},
        ]
    },
    {
        "id": "03_is_valid_reservation_base",
        "file": "core/src/transport/swarm.rs",
        "fn": "is_valid_reservation_base",
        "claims": [
            {"claim_id": "C1", "kind": "defect", "text": "Does not validate that the base address is dialable (reachable) before accepting as reservation base"},
            {"claim_id": "C2", "kind": "reassurance", "text": "Correctly rejects bases containing circuit components (prevents nesting)"},
            {"claim_id": "C3", "kind": "reassurance", "text": "Correctly delegates self-endpoint check to is_self_endpoint"},
        ]
    },
    {
        "id": "04_is_canonical_reservation_addr",
        "file": "core/src/transport/swarm.rs",
        "fn": "is_canonical_reservation_addr",
        "claims": [
            {"claim_id": "C1", "kind": "defect", "text": "Allows non-circuit components BEFORE the circuit (e.g. /tcp/.../p2p-circuit) which breaks canonical form"},
            {"claim_id": "C2", "kind": "reassurance", "text": "Correctly rejects addresses with zero or multiple /p2p-circuit components"},
            {"claim_id": "C3", "kind": "reassurance", "text": "Correctly rejects components after the circuit component"},
        ]
    },
    {
        "id": "05_set_configured_external_address",
        "file": "core/src/transport/swarm.rs",
        "fn": "set_configured_external_address",
        "claims": [
            {"claim_id": "C1", "kind": "defect", "text": "No validation that the SocketAddr is a public/routable address before storing as external"},
            {"claim_id": "C2", "kind": "defect", "text": "Error conversion loses the original SendError context (channel disconnected vs full)"},
            {"claim_id": "C3", "kind": "reassurance", "text": "Correctly returns error when swarm task is not running"},
        ]
    },
    {
        "id": "06_reset_delivery_attempts_for_destination",
        "file": "core/src/store/relay_custody.rs",
        "fn": "reset_delivery_attempts_for_destination",
        "claims": [
            {"claim_id": "C1", "kind": "defect", "text": "Scans entire prefix then filters in-memory; O(N) scan with no index on delivery_attempts"},
            {"claim_id": "C2", "kind": "defect", "text": "Resets delivery_attempts for non-Delivered records only, but does not check if record is already in retry backoff"},
            {"claim_id": "C3", "kind": "reassurance", "text": "Correctly skips Delivered state records"},
            {"claim_id": "C4", "kind": "reassurance", "text": "Logs reset count with destination peer ID for audit"},
        ]
    },
    {
        "id": "07_sort_by_reliability",
        "file": "core/src/routing/local.rs",
        "fn": "sort_by_reliability",
        "claims": [
            {"claim_id": "C1", "kind": "defect", "text": "Uses expect() on partial_cmp - panics if reliability_score is NaN (e.g. uninitialized peer)"},
            {"claim_id": "C2", "kind": "defect", "text": "No secondary sort stability guarantee beyond peer_id; equal scores may reorder non-deterministically"},
            {"claim_id": "C3", "kind": "reassurance", "text": "Correctly sorts descending by reliability_score with peer_id tiebreaker"},
        ]
    },
    {
        "id": "08_active_peer_selection_contract",
        "file": "core/src/routing/local.rs",
        "fn": "active_peer_selection_contract",
        "claims": [
            {"claim_id": "C1", "kind": "defect", "text": "Test function masquerading as contract; uses assert_eq! which panics instead of returning Result"},
            {"claim_id": "C2", "kind": "defect", "text": "Hardcodes hint value (100) and peer IDs; not a general contract verification"},
            {"claim_id": "C3", "kind": "reassurance", "text": "Validates active_peers ordering and peers_for_hint behavior for the happy path"},
        ]
    },
    {
        "id": "09_attemptEscalation",
        "file": "android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt",
        "fn": "attemptEscalation",
        "claims": [
            {"claim_id": "C1", "kind": "defect", "text": "No check if WiFi Aware/WiFi Direct are actually connected to the target peerId before marking as active"},
            {"claim_id": "C2", "kind": "defect", "text": "Unconditionally sets activeTransports to true without verifying the transport can actually reach peerId"},
            {"claim_id": "C3", "kind": "defect", "text": "No backoff or rate limiting on escalation attempts - could spam transport init"},
            {"claim_id": "C4", "kind": "reassurance", "text": "Logs escalation attempt for debugging"},
        ]
    },
    {
        "id": "10_sendViaTransport",
        "file": "android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt",
        "fn": "sendViaTransport",
        "claims": [
            {"claim_id": "C1", "kind": "defect", "text": "BLE fallback via GATT server connectedDevices.first() uses arbitrary peer when original peerId fails"},
            {"claim_id": "C2", "kind": "defect", "text": "No verification that the fallback peer is actually the intended recipient (peerId mismatch)"},
            {"claim_id": "C3", "kind": "defect", "text": "Returns false for INTERNET/TCP_MDNS without attempting SwarmBridge send"},
            {"claim_id": "C4", "kind": "reassurance", "text": "Correctly tries multiple BLE paths (L2CAP -> GATT client -> GATT server) before fallback"},
        ]
    },
    {
        "id": "11_initializeWifiAware",
        "file": "android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt",
        "fn": "initializeWifiAware",
        "claims": [
            {"claim_id": "C1", "kind": "defect", "text": "Catches all exceptions and sets wifiAware=null without distinguishing init failure from availability failure"},
            {"claim_id": "C2", "kind": "defect", "text": "No retry logic for transient WiFi Aware initialization failures"},
            {"claim_id": "C3", "kind": "reassurance", "text": "Correctly checks isAvailable() after construction"},
            {"claim_id": "C4", "kind": "reassurance", "text": "Callbacks properly wire peer discovery and data reception"},
        ]
    },
    {
        "id": "12_initializeWifiDirect",
        "file": "android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt",
        "fn": "initializeWifiDirect",
        "claims": [
            {"claim_id": "C1", "kind": "defect", "text": "Catches all exceptions and sets wifiDirect=null without retry or specific error handling"},
            {"claim_id": "C2", "kind": "defect", "text": "No validation that WiFi Direct is actually supported on the device before init"},
            {"claim_id": "C3", "kind": "reassurance", "text": "Callbacks properly wire peer discovery, data reception, and connection info"},
        ]
    },
    {
        "id": "13_mergeBootstrapCandidates",
        "file": "android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt",
        "fn": "mergeBootstrapCandidates",
        "claims": [
            {"claim_id": "C1", "kind": "defect", "text": "No validation that proven/seeds are valid multiaddr strings before adding to bootstrap list"},
            {"claim_id": "C2", "kind": "defect", "text": "Seeds are truncated to maxSeeds after filtering, but proven entries are not bounded"},
            {"claim_id": "C3", "kind": "reassurance", "text": "Correctly deduplicates and preserves proven entries first"},
            {"claim_id": "C4", "kind": "reassurance", "text": "Trims whitespace and filters empty strings"},
        ]
    },
    {
        "id": "14_recordConnectionFailure",
        "file": "android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt",
        "fn": "recordConnectionFailure",
        "claims": [
            {"claim_id": "C1", "kind": "defect", "text": "String-based detail filtering is fragile; 'Device offline' substring could match unrelated failures"},
            {"claim_id": "C2", "kind": "defect", "text": "No structured error taxonomy - relies on English error message substrings"},
            {"claim_id": "C3", "kind": "defect", "text": "Local/epoch failures are silently dropped from ledger, losing visibility into transient conditions"},
            {"claim_id": "C4", "kind": "reassurance", "text": "Correctly records endpoint-fault failures (refused/timeout/TLS) to ledger"},
        ]
    },
    {
        "id": "15_startAdvertisingInternal",
        "file": "android/app/src/main/java/com/scmessenger/android/transport/ble/BleAdvertiser.kt",
        "fn": "startAdvertisingInternal",
        "claims": [
            {"claim_id": "C1", "kind": "defect", "text": "No validation that identity data length <= 24 bytes before calling addServiceData (silently drops oversized)"},
            {"claim_id": "C2", "kind": "defect", "text": "SecurityException caught but isAdvertising flag not reset, leaving inconsistent state"},
            {"claim_id": "C3", "kind": "reassurance", "text": "Correctly checks BLUETOOTH_ADVERTISE permission before starting"},
            {"claim_id": "C4", "kind": "reassurance", "text": "Rotation timer started if interval configured"},
        ]
    },
    {
        "id": "16_stopAdvertisingInternal",
        "file": "android/app/src/main/java/com/scmessenger/android/transport/ble/BleAdvertiser.kt",
        "fn": "stopAdvertisingInternal",
        "claims": [
            {"claim_id": "C1", "kind": "defect", "text": "When SecurityException caught, isAdvertising set to false but advertiser.stopAdvertising NOT called"},
            {"claim_id": "C2", "kind": "defect", "text": "stopRotation() called even if stopAdvertising failed, potentially leaving timer running"},
            {"claim_id": "C3", "kind": "reassurance", "text": "Correctly checks permission and isAdvertising flag before stopping"},
        ]
    },
    {
        "id": "17_shareDiagnosticsBundle",
        "file": "android/app/src/main/java/com/scmessenger/android/utils/DiagnosticsShareController.kt",
        "fn": "shareDiagnosticsBundle",
        "claims": [
            {"claim_id": "C1", "kind": "defect", "text": "Catches all exceptions and returns Failed result, but does not distinguish between FileProvider misconfig and intent resolution failure"},
            {"claim_id": "C2", "kind": "defect", "text": "No size limit on bundleText - could create huge temp files"},
            {"claim_id": "C3", "kind": "reassurance", "text": "Uses Dispatchers.IO for file I/O off main thread"},
            {"claim_id": "C4", "kind": "reassurance", "text": "Grants FLAG_GRANT_READ_URI_PERMISSION for FileProvider"},
        ]
    },
    {
        "id": "18_resolveShareTarget",
        "file": "android/app/src/main/java/com/scmessenger/android/utils/DiagnosticsShareController.kt",
        "fn": "resolveShareTarget",
        "claims": [
            {"claim_id": "C1", "kind": "defect", "text": "Tries cacheDir first then filesDir, but both use same FileProvider authority - no isolation"},
            {"claim_id": "C2", "kind": "defect", "text": "IllegalArgumentException caught silently for each root; no logging of which root was actually used"},
            {"claim_id": "C3", "kind": "reassurance", "text": "Creates parent directories if missing"},
            {"claim_id": "C4", "kind": "reassurance", "text": "Returns null cleanly when no FileProvider root accepts the file"},
        ]
    },
    {
        "id": "19_refreshInfoCounts",
        "file": "android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt",
        "fn": "refreshInfoCounts",
        "claims": [
            {"claim_id": "C1", "kind": "defect", "text": "All three async calls run sequentially (not in parallel), adding latency"},
            {"claim_id": "C2", "kind": "defect", "text": "Catches Exception broadly and returns 0/empty string, masking specific failure modes"},
            {"claim_id": "C3", "kind": "defect", "text": "No loading state or error propagation to UI - _infoCounts updated with zeros on any failure"},
            {"claim_id": "C4", "kind": "reassurance", "text": "Uses Dispatchers.IO for database/IO operations"},
        ]
    },
]

for f in functions:
    with open(f"scmessenger_audit/source/{f['id']}.rs", "w") as sf:
        sf.write(f"// {f['file']} :: {f['fn']}\n")
        # We'll need the actual source - but for claims we just need the claims file
    
    claims_manifest = {
        "context": f"Security audit of {f['fn']} in {f['file']}. Function extracted from recent changes (last 7 days).",
        "claims": f["claims"]
    }
    with open(f"scmessenger_audit/claims/{f['id']}.json", "w") as cf:
        json.dump(claims_manifest, cf, indent=2)

print(f"Created {len(functions)} claims manifests")