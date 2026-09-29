#!/usr/bin/env python3
"""
Parse iOS log file and extract unique log entries with counts and frequency distribution.
Outputs:
- Total line count
- Unique log patterns (normalized) with counts
- Frequency distribution
- Logs warranting debug attention (errors, warnings, repeated failures)
"""

import re
import sys
from collections import Counter, defaultdict
from datetime import datetime

# Input file
LOG_FILE = "tmp/iOSdevicelogs.txt"

# Patterns to normalize (remove variable parts)
NORMALIZE_PATTERNS = [
    # Remove timestamps
    (r'\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z', '<TIMESTAMP>'),
    # Remove peer IDs (12D3KooW...)
    (r'12D3KooW[a-zA-Z0-9]+', '<PEER_ID>'),
    # Remove hex public keys (40+ hex chars)
    (r'[0-9a-f]{40,}', '<HEX_KEY>'),
    # Remove IP addresses
    (r'\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}', '<IP>'),
    # Remove port numbers
    (r':\d{4,5}', ':<PORT>'),
    # Remove UUIDs
    (r'[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}', '<UUID>'),
    # Remove message IDs
    (r'message_id=[^\s]+', 'message_id=<MSG_ID>'),
    # Remove numeric values that vary
    (r'attempt=\d+', 'attempt=<N>'),
    (r'pass=\d+', 'pass=<N>'),
    (r'candidate=\d+/\d+', 'candidate=<N>/<N>'),
    (r'recipient_recency=\d+', 'recipient_recency=<N>'),
    (r'relay_score=[\d.]+', 'relay_score=<F>'),
    (r'latest_success_order=\d+', 'latest_success_order=<N>'),
    (r'events_60s=\d+', 'events_60s=<N>'),
    # Remove battery percentages
    (r'battery=\d+%', 'battery=<PCT>'),
    # Remove relay budget values
    (r'\d+ msgs/hour', '<N> msgs/hour'),
    # Remove scan interval values
    (r'scan interval updated: [\d.]+', 'scan interval updated: <VAL>'),
    # Remove byte sizes
    (r'\d+ bytes', '<N> bytes'),
    # Remove uptime values
    (r'uptimeSecs=\d+', 'uptimeSecs=<N>'),
    # Remove memory addresses
    (r'0x[0-9a-f]+', '<ADDR>'),
    # Remove instance identifiers
    (r'instance=\d+\.\d+\.\d+\.\d+', 'instance=<IP>'),
]

# Severity classification
ERROR_PATTERNS = [
    r'ERROR', r'error', r'Error', r'fail', r'Fail', r'FAIL',
    r'', r'', r'cannot', r'Could not', r'API MISUSE'
]

WARNING_PATTERNS = [
    r'WARN', r'warn', r'️', r'warning'
]

def normalize_line(line):
    """Normalize a log line by removing variable parts."""
    normalized = line.strip()
    for pattern, replacement in NORMALIZE_PATTERNS:
        normalized = re.sub(pattern, replacement, normalized)
    return normalized

def classify_severity(line):
    """Classify log line severity."""
    for pattern in ERROR_PATTERNS:
        if re.search(pattern, line):
            return 'ERROR'
    for pattern in WARNING_PATTERNS:
        if re.search(pattern, line):
            return 'WARNING'
    return 'INFO'

def extract_log_component(line):
    """Extract the main component/module from the log line."""
    # Rust logs: [2m...[32m INFO [2m[module...
    rust_match = re.search(r'(?:INFO|WARN|ERROR)\s+\[2m(\S+)', line)
    if rust_match:
        return rust_match.group(1).split('::')[-1]
    
    # Android-style: Component followed by package
    android_match = re.search(r'\s{2,}(\w+)\s+com\.scmessenger', line)
    if android_match:
        return android_match.group(1)
    
    # iOS plain text logs
    plain_match = re.match(r'^([A-Z][a-zA-Z]+(?:\s+[A-Z][a-zA-Z]+)*):', line)
    if plain_match:
        return plain_match.group(1)
    
    # DIAG logs
    diag_match = re.match(r'^DIAG:\s+(\w+)', line)
    if diag_match:
        return diag_match.group(1)
    
    return 'OTHER'

def main():
    print(f"{'='*80}")
    print(f"iOS LOG ANALYSIS: {LOG_FILE}")
    print(f"{'='*80}\n")
    
    # Read log file
    try:
        with open(LOG_FILE, 'r', encoding='utf-8', errors='replace') as f:
            lines = f.readlines()
    except FileNotFoundError:
        print(f"ERROR: File {LOG_FILE} not found")
        sys.exit(1)
    
    total_lines = len(lines)
    print(f"Total lines: {total_lines:,}\n")
    
    # Collect statistics
    normalized_counter = Counter()
    severity_counter = Counter()
    component_counter = Counter()
    component_severity = defaultdict(Counter)
    first_occurrence = {}
    sample_lines = {}
    
    for i, line in enumerate(lines, 1):
        if not line.strip():
            continue
            
        normalized = normalize_line(line)
        severity = classify_severity(line)
        component = extract_log_component(line)
        
        normalized_counter[normalized] += 1
        severity_counter[severity] += 1
        component_counter[component] += 1
        component_severity[component][severity] += 1
        
        if normalized not in first_occurrence:
            first_occurrence[normalized] = i
            sample_lines[normalized] = line.strip()[:200]  # First 200 chars
    
    # Output: Severity Summary
    print(f"{'='*80}")
    print("SEVERITY DISTRIBUTION")
    print(f"{'='*80}")
    for sev in ['ERROR', 'WARNING', 'INFO']:
        count = severity_counter.get(sev, 0)
        pct = (count / total_lines * 100) if total_lines > 0 else 0
        bar = '█' * int(pct / 2)
        print(f"  {sev:8s}: {count:7,} ({pct:5.1f}%) {bar}")
    print()
    
    # Output: Top Components
    print(f"{'='*80}")
    print("TOP 30 COMPONENTS BY LINE COUNT")
    print(f"{'='*80}")
    for component, count in component_counter.most_common(30):
        errors = component_severity[component].get('ERROR', 0)
        warnings = component_severity[component].get('WARNING', 0)
        flag = '' if errors > 0 else ('️' if warnings > 0 else '')
        print(f"  {flag} {component:40s}: {count:7,} lines (E:{errors}, W:{warnings})")
    print()
    
    # Output: Unique Log Patterns (top 100)
    print(f"{'='*80}")
    print("TOP 100 UNIQUE LOG PATTERNS (by frequency)")
    print(f"{'='*80}")
    for idx, (normalized, count) in enumerate(normalized_counter.most_common(100), 1):
        severity = classify_severity(sample_lines.get(normalized, ''))
        first_line = first_occurrence.get(normalized, 0)
        sample = sample_lines.get(normalized, '')[:120]
        flag = '' if severity == 'ERROR' else ('️' if severity == 'WARNING' else '  ')
        print(f"\n{idx:3d}. {flag} Count: {count:6,} | First at line {first_line}")
        print(f"     Sample: {sample}")
        print(f"     Pattern: {normalized[:120]}")
    
    # Output: Errors Only
    print(f"\n{'='*80}")
    print("ERRORS AND FAILURES (sorted by frequency)")
    print(f"{'='*80}")
    error_patterns = [(n, c) for n, c in normalized_counter.items() 
                      if classify_severity(sample_lines.get(n, '')) == 'ERROR']
    error_patterns.sort(key=lambda x: x[1], reverse=True)
    
    for idx, (normalized, count) in enumerate(error_patterns[:50], 1):
        first_line = first_occurrence.get(normalized, 0)
        sample = sample_lines.get(normalized, '')[:150]
        print(f"\n{idx:3d}. Count: {count:6,} | First at line {first_line}")
        print(f"     Sample: {sample}")
    
    # Output: Warnings Only
    print(f"\n{'='*80}")
    print("WARNINGS (sorted by frequency)")
    print(f"{'='*80}")
    warning_patterns = [(n, c) for n, c in normalized_counter.items() 
                        if classify_severity(sample_lines.get(n, '')) == 'WARNING']
    warning_patterns.sort(key=lambda x: x[1], reverse=True)
    
    for idx, (normalized, count) in enumerate(warning_patterns[:50], 1):
        first_line = first_occurrence.get(normalized, 0)
        sample = sample_lines.get(normalized, '')[:150]
        print(f"\n{idx:3d}. Count: {count:6,} | First at line {first_line}")
        print(f"     Sample: {sample}")
    
    # Output: Debug Worthy Items
    print(f"\n{'='*80}")
    print(" ITEMS WARRANTING DEBUG INVESTIGATION")
    print(f"{'='*80}")
    
    debug_items = []
    
    # High-frequency errors
    for normalized, count in error_patterns:
        if count > 10:
            debug_items.append(('HIGH_FREQ_ERROR', normalized, count, 
                               sample_lines.get(normalized, '')[:150]))
    
    # High-frequency warnings
    for normalized, count in warning_patterns:
        if count > 50:
            debug_items.append(('HIGH_FREQ_WARNING', normalized, count,
                               sample_lines.get(normalized, '')[:150]))
    
    # Repeated patterns (>100 occurrences)
    for normalized, count in normalized_counter.most_common():
        if count > 100 and normalized not in [n for _, n, _, _ in debug_items]:
            debug_items.append(('HIGH_FREQUENCY', normalized, count,
                               sample_lines.get(normalized, '')[:150]))
    
    # Sort by count descending
    debug_items.sort(key=lambda x: x[2], reverse=True)
    
    for idx, (category, normalized, count, sample) in enumerate(debug_items[:30], 1):
        print(f"\n{idx:3d}. [{category}] Count: {count:,}")
        print(f"     Sample: {sample}")
        print(f"     Pattern: {normalized[:120]}")
    
    print(f"\n{'='*80}")
    print(f"ANALYSIS COMPLETE")
    print(f"Total unique patterns: {len(normalized_counter):,}")
    print(f"Total errors: {severity_counter.get('ERROR', 0):,}")
    print(f"Total warnings: {severity_counter.get('WARNING', 0):,}")
    print(f"{'='*80}")

if __name__ == '__main__':
    main()
