#!/usr/bin/env python3
# scripts/mince_logs.py
# Iterative tree-mincing exploratory analysis tool for SCMessenger logs.
# Extracts key information while excluding rotating variables (timestamps, IDs, etc.)
# Output is compatible with the LogSankey visualizer.

import sys
import re
import json
import argparse

def mince_line(line):
    # 1. Clean up line - remove timestamps
    # ISO-8601 / RFC3339
    line = re.sub(r'\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}[\.\d]*Z?', '', line)
    # Android Logcat style
    line = re.sub(r'\d{2}-\d{2}\s+\d{2}:\d{2}:\d{2}\.\d+', '', line)
    # Simple time HM/HMS
    line = re.sub(r'\[\d{2}:\d{2}(?::\d{2})?\]', '', line)
    
    # 2. Redact rotating variables
    # UUIDs
    line = re.sub(r'[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}', '<uuid>', line)
    # Hex addresses (0x...)
    line = re.sub(r'0x[a-fA-F0-9]{4,}', '<hex>', line)
    # Peer IDs (12D3Koo...)
    line = re.sub(r'12D3Koo[a-zA-Z0-9]{45}', '<peer_id>', line)
    # Multiaddrs with PeerId
    line = re.sub(r'/p2p/12D3Koo[a-zA-Z0-9]{45}', '/p2p/<peer_id>', line)
    # IP addresses
    line = re.sub(r'\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}', '<ip>', line)
    
    # 3. Extract segments using "tree-mincing" strategy
    # Split by characters that usually separate components in a distributed system
    # but keep dots if they look like namespaces (e.g. com.scmessenger)
    raw_segments = re.split(r'[:\s\->|()\[\]{}]+', line)
    
    # Filter and normalize
    segments = []
    for s in raw_segments:
        s = s.strip().strip(',.;')
        if not s: continue
        if len(s) < 2 and not s.isdigit(): continue
        # If it's a number, it might be rotating, but small ones are usually counts
        if s.isdigit() and int(s) > 1000:
            segments.append("<num>")
        else:
            segments.append(s)
            
    # Remove duplicates in sequence (e.g. "info: info:")
    deduped = []
    for s in segments:
        if not deduped or deduped[-1] != s:
            deduped.append(s)
            
    return deduped

def main():
    parser = argparse.ArgumentParser(description="Mince logs for Sankey visualization")
    parser.add_argument("input", nargs="?", type=argparse.FileType("r"), default=sys.stdin)
    parser.add_argument("--platform", default="Unknown", help="Platform tag for the logs")
    args = parser.parse_args()

    for line in args.input:
        line = line.strip()
        if not line: continue
        
        segments = mince_line(line)
        if not segments: continue
        
        # Prepend platform for top-level grouping
        if segments[0] != args.platform:
            segments.insert(0, args.platform)
            
        record = {
            "segments": segments,
            "levelTag": "INFO", # Could be extracted from segments if needed
            "msg": line
        }
        print(json.dumps(record))

if __name__ == "__main__":
    main()
