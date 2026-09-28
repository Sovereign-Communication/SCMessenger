#!/usr/bin/env python3
import re
with open('tmp_opus_v2.txt', 'r', encoding='utf-8', errors='ignore') as f:
    text = f.read()

# Turn "--- File: X [sources] ---" into a terser "FILE: X | sources"
def repl(m):
    inner = m.group(1).strip()
    return f"FILE: {inner}"

text = re.sub(r'--- (File: .+?) ---', repl, text)

with open('tmp_opus_v2_clean.txt', 'w', encoding='utf-8') as f:
    f.write(text)

print("done", file=__import__('sys').stderr)
