#!/usr/bin/env python3
import sys

# Read the minimal prompt
with open('tmp_opus_minimal.txt', 'r', encoding='utf-8', errors='ignore') as f:
    lines = f.readlines()

# Strip "--- File: X ---" separator lines
output = []
for line in lines:
    if not line.strip().startswith('--- File:'):
        output.append(line)

# Write back
with open('tmp_opus_final.txt', 'w', encoding='utf-8') as f:
    f.writelines(output)

print(f"Stripped {len(lines) - len(output)} separator lines", file=sys.stderr)
