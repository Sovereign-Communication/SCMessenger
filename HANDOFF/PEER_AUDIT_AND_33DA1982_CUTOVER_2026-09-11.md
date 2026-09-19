# Peer audit + one-candidate cutover 33da1982

UTC: 2026-09-11 ~01:10–01:17Z

## Duplicate-node audit (user saw “3 nodes”)

| Source | Count | IDs |
|---|---|---|
| EC2 instances | **1** | `i-0b41aab7` @ 18.234.62.247 only |
| Emulator AVD | **not running** | gone from ADB; peer `12D3KooWAv9…` no longer in live peer lists |
| Live fleet | **3 nodes** | Windows + AWS + Pixel — not 2× AWS |

The third identity seen earlier was the **emulator** (`12D3KooWAv9…`) while it was up.
It is not a duplicate AWS and not a ghost after restart.

## One-candidate SHA (now)

| Node | git_hash |
|---|---|
| Windows | `33da1982` |
| AWS (`sha-33da198`) | `33da1982b7e9438a60a84216164b2cf81c34e300` |
| Pixel APK | `f9a1f60b` app (has nearby-dedup + triad log; core transport identical) |

## Custody registration (both CLIs)

Windows and AWS now both log:
```
[CUSTODY] Registered local identity with peer … (relay-ready)
```
AWS also logs store/forward:
```
Accepted custody … for offline destination
[OK] Custody … delivered via peer_reconnect
```

Probes: Win→Phone `d004045f` **17ms**, later `a37775c5` **35ms** + ACKs.

## BLE after reboot

**Still down.** `MediaTek Bluetooth Adapter` Present=False;
`Unknown USB Device (Device Descriptor Request Failed)` remains.
`bthserv` is Running but no radio. Not fixed by reboot — needs hardware/driver cycle
or another machine.

## Phone → AWS direct dial (subagent RCA)

Not an AWS security-group hole (Windows reaches :9001). UniFFI maps all dial
failures to `NetworkException: Network error`; Kotlin classifier only understands
`java.net.*`. Fix: propagate Rust dial error text through IronCoreError, parse in
`classifyBootstrapError`. Circuit path already works.

## PR #281 CI

Green: Android, Windows CLI, multi-OS tests, Docker Publish, CodeQL, bindings.
Red (fast-fail config/tooling, not product): Lint, Repository Hygiene, Rust Linting.
iOS still pending. Safe to iterate locally; not merge-ready until those gates are fixed.
