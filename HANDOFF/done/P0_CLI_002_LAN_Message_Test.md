# CLI Daemon LAN Message Delivery Test

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Date:** 2026-04-23
**Agent:** CLIBetaTester
**Model:** qwen3-coder:480b:cloud

## Context

Android user (Luke) is on same LAN with app running.
- **Android Peer ID (Network):** `12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9`
- **Android Public Key:** `374d73ebc21d677f889b806d6065a7177a2c5748f368036bd073e949bba5a2d4`
- **Android Identity:** `caccf8657f8cf09f647503be0c6a51f27ee4ff616fd5b7153fd2c41b6b55747`
- **Local Daemon Peer ID:** `12D3KooWP3RGmGgRNtqGsfBCZgu8Wzao6qSsqYzLeLRmkqBdf5Ag`
- **Daemon Status:** Running, 0 connected peers but knows Android addresses from ledger

## Known Issues
- Daemon has 0 connected peers (all bootstrap dials failing)
- Android app also has 0 connected peers
- Both are on same LAN (192.168.0.x)
- mDNS disabled on Windows daemon (if-watch limitation)

## Mission

1. **Add Android as contact** using correct libp2p Peer ID: `12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9`
2. **Verify daemon health** using `core_cli_driver` skill
3. **Attempt direct LAN dial** to Android addresses in ledger
4. **Send test message** "Hey Luke, can you see this?"
5. **Monitor delivery** via logs for 5 minutes
6. **Document** results in `HANDOFF/IN_PROGRESS/CLI_LAN_TEST_2026-04-23.md`
7. **If failed, diagnose why** LAN connection isn't working

## Skill Commands Available
```bash
bash .claude/skills/core_cli_driver.sh dump_logs
bash .claude/skills/core_cli_driver.sh rpc '{"cmd":"status"}'
bash .claude/skills/core_cli_driver.sh stop
```

## Notes
- The Android app logcat shows addresses like `/ip4/192.168.0.113/tcp/xxxx`
- The daemon should be able to dial these directly if on same LAN
- Check if firewall or Android Doze mode is blocking incoming connections
- Document any crashes in `HANDOFF/IN_PROGRESS/CORE_CRASH_REPORT_<timestamp>.md`
