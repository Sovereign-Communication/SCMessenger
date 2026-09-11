import re

def extract_fn(content, fn_name):
    # Build pattern without f-string brace issues
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

with open('core/src/transport/swarm.rs', 'r') as f:
    content = f.read()

fns = [
    'is_poison_circuit_listener',
    'is_self_endpoint',
    'is_valid_reservation_base',
    'is_canonical_reservation_addr',
    'set_configured_external_address',
]
for fn in fns:
    results = extract_fn(content, fn)
    for r in results:
        print('=== ' + fn + ' ===')
        print(r)
        print()