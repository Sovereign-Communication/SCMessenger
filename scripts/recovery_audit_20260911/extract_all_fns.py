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
    # Kotlin: fun name(...) : ReturnType { or fun name(...) {
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

# All target files and their functions
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
        'initializeBle',
        'initializeWifiAware',
        'initializeWifiDirect',
    ]),
    ("android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt", [
        'mergeBootstrapCandidates',
        'recordConnectionFailure',
    ]),
    ("android/app/src/main/java/com/scmessenger/android/service/AndroidPlatformBridge.kt", [
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
        'getLedgerSummary',
        'getNatStatus',
    ]),
]

all_functions = []
for filepath, fns in targets:
    try:
        with open(filepath, 'r') as f:
            content = f.read()
        is_rust = filepath.endswith('.rs')
        for fn in fns:
            if is_rust:
                results = extract_rust_fn(content, fn)
            else:
                results = extract_kotlin_fn(content, fn)
            if results:
                all_functions.append((filepath, fn, results[0]))
            else:
                print(f"NOT FOUND: {filepath} :: {fn}")
    except FileNotFoundError:
        print(f"FILE NOT FOUND: {filepath}")

print(f"Total extracted: {len(all_functions)}")
for filepath, fn, src in all_functions:
    print(f"\n=== {filepath} :: {fn} ===")
    print(src)