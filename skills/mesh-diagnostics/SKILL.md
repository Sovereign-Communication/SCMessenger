---
name: mesh-diagnostics
description: Diagnose libp2p mesh, relay, and NAT traversal issues
---

# Mesh Diagnostics Skill

## Use When
- Peer discovery fails
- Relay connections flap
- NAT traversal issues
- Message delivery failures

## Diagnostic Steps
1. Collect logs with scripts/comprehensive_log_capture.sh
2. Parse with scripts/mince_logs.py
3. Check relay flap windows with scripts/correlate_relay_flap_windows.sh
4. Verify bootstrap connectivity
5. Check NAT/reflection status

## Key Modules
- core/src/transport/ - Swarm behavior
- core/src/relay/ - Relay protocol
- core/src/routing/ - Routing engines

## Output
- Root cause analysis
- Remediation steps
- Configuration recommendations
