import re

def extract_rust_fn(content, fn_name):
    escaped = re.escape(fn_name)
    pattern = r'(?:pub\s+)?(?:async\s+)?fn ' + escaped + r'\s*\([^)]*\)\s*(?:->\s*[^{]+)?\s*\{'
    matches = list(re.finditer(pattern, content))
    results = []
    for m in matches:
        start = m.start()
        brace_count = 0
        in_string = False
        escape = False
        for i, c in enumerate(content[start:]):
            if escape:
                escape = False
                continue
            if c == '\\':
                escape = True
                continue
            if c == '"' and not escape:
                in_string = not in_string
                continue
            if in_string:
                continue
            if c == '{':
                brace_count += 1
            elif c == '}':
                brace_count -= 1
                if brace_count == 0:
                    end = start + i + 1
                    results.append(content[start:end])
                    break
    return results

def extract_kotlin_fn(content, fn_name):
    escaped = re.escape(fn_name)
    pattern = r'(?:private\s+)?(?:suspend\s+)?(?:internal\s+)?fun\s+' + escaped + r'\s*\([^)]*\)\s*(?::\s*[^{]+)?\s*\{'
    matches = list(re.finditer(pattern, content))
    results = []
    for m in matches:
        start = m.start()
        brace_count = 0
        in_string = False
        escape = False
        for i, c in enumerate(content[start:]):
            if escape:
                escape = False
                continue
            if c == '\\':
                escape = True
                continue
            if c == '"' and not escape:
                in_string = not in_string
                continue
            if in_string:
                continue
            if c == '{':
                brace_count += 1
            elif c == '}':
                brace_count -= 1
                if brace_count == 0:
                    end = start + i + 1
                    results.append(content[start:end])
                    break
    return results

targets = [
    ("core/src/transport/swarm.rs", [
        'is_poison_circuit_listener',
        'is_self_endpoint',
        'is_valid_reservation_base',
        'is_canonical_reservation_addr',
        'set_configured_external_address',
    ]),
    ("core/src/store/relay_custody.rs", [
        'reset_delivery_attempts_for_destination',
    ]),
    ("core/src/routing/local.rs", [
        'sort_by_reliability',
        'active_peer_selection_contract',
    ]),
    ("android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt", [
        'attemptEscalation',
        'sendViaTransport',
        'initializeWifiAware',
        'initializeWifiDirect',
    ]),
    ("android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt", [
        'mergeBootstrapCandidates',
        'recordConnectionFailure',
    ]),
    ("android/app/src/main/java/com/scmessenger/android/transport/ble/BleAdvertiser.kt", [
        'startAdvertisingInternal',
        'stopAdvertisingInternal',
    ]),
    ("android/app/src/main/java/com/scmessenger/android/utils/DiagnosticsShareController.kt", [
        'shareDiagnosticsBundle',
        'resolveShareTarget',
    ]),
    ("android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt", [
        'refreshInfoCounts',
    ]),
]

# Map function IDs to (filepath, fn_name)
id_map = {
    "01_is_poison_circuit_listener": ("core/src/transport/swarm.rs", "is_poison_circuit_listener"),
    "02_is_self_endpoint": ("core/src/transport/swarm.rs", "is_self_endpoint"),
    "03_is_valid_reservation_base": ("core/src/transport/swarm.rs", "is_valid_reservation_base"),
    "04_is_canonical_reservation_addr": ("core/src/transport/swarm.rs", "is_canonical_reservation_addr"),
    "05_set_configured_external_address": ("core/src/transport/swarm.rs", "set_configured_external_address"),
    "06_reset_delivery_attempts_for_destination": ("core/src/store/relay_custody.rs", "reset_delivery_attempts_for_destination"),
    "07_sort_by_reliability": ("core/src/routing/local.rs", "sort_by_reliability"),
    "08_active_peer_selection_contract": ("core/src/routing/local.rs", "active_peer_selection_contract"),
    "09_attemptEscalation": ("android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt", "attemptEscalation"),
    "10_sendViaTransport": ("android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt", "sendViaTransport"),
    "11_initializeWifiAware": ("android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt", "initializeWifiAware"),
    "12_initializeWifiDirect": ("android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt", "initializeWifiDirect"),
    "13_mergeBootstrapCandidates": ("android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt", "mergeBootstrapCandidates"),
    "14_recordConnectionFailure": ("android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt", "recordConnectionFailure"),
    "15_startAdvertisingInternal": ("android/app/src/main/java/com/scmessenger/android/transport/ble/BleAdvertiser.kt", "startAdvertisingInternal"),
    "16_stopAdvertisingInternal": ("android/app/src/main/java/com/scmessenger/android/transport/ble/BleAdvertiser.kt", "stopAdvertisingInternal"),
    "17_shareDiagnosticsBundle": ("android/app/src/main/java/com/scmessenger/android/utils/DiagnosticsShareController.kt", "shareDiagnosticsBundle"),
    "18_resolveShareTarget": ("android/app/src/main/java/com/scmessenger/android/utils/DiagnosticsShareController.kt", "resolveShareTarget"),
    "19_refreshInfoCounts": ("android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt", "refreshInfoCounts"),
}

for fid, (filepath, fn_name) in id_map.items():
    with open(filepath, 'r') as f:
        content = f.read()
    is_rust = filepath.endswith('.rs')
    if is_rust:
        results = extract_rust_fn(content, fn_name)
    else:
        results = extract_kotlin_fn(content, fn_name)
    if results:
        with open(f"scmessenger_audit/source/{fid}.txt", "w") as sf:
            sf.write(results[0])
        print(f"OK: {fid}")
    else:
        print(f"FAIL: {fid} not found")