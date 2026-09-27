#!/usr/bin/env python3
with open('tmp_opus_comprehensive.txt', 'r', encoding='utf-8', errors='ignore') as f:
    lines = f.readlines()

output = [line for line in lines if not line.strip().startswith('--- File:')]

with open('tmp_opus_comprehensive_clean.txt', 'w', encoding='utf-8') as f:
    f.writelines(output)

print(f"Stripped {len(lines) - len(output)} separator lines", file=__import__('sys').stderr)
