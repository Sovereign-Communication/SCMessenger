<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Triage Decision -- 2026-06-08

**Status:** ready
**Bucket:** pending-dispatch
**Cross-ref:** see `HANDOFF/STATE/2026-06-08_UNIFIED_BACKLOG.md`
**Decided by:** Claude Code (Overseer) sweep

**Rationale:** Ticket is a real remaining work item with no shipped code on the
integration branch. No blocker identified. Ready for `/orchestrate` dispatch on
the next cloud slot allocation. Per Lucas directive 2026-06-08 "I want it all
fixed," this is part of the ~30-ticket remaining backlog.

---
# MODEL: gemma4:31b:cloud
# BUDGET: 1800
# token_budget: 18000

# P2_TEST_001_Cross_Platform_Delivery_Harness

**Status:** VERIFIED REMAINING WORK
**Agent:** worker
**Budget:** 1800s (MIXED tier)
**Phase:** v0.2.1 P2 cross-platform verification
**Source:** docs/historical/plans/PRODUCTION_ROADMAP.md 1.4 (Cross-Platform Delivery) + planfromclaudeforhermes 2 Phase F.1
**Depends on:** P1_CORE_001, P1_CORE_002 (need production routing + drift to exercise)

---

## Verified Gap

Per `docs/historical/plans/PRODUCTION_ROADMAP.md` 1.4: "Synchronized AndroidiOS physical device delivery + receipt validation, BLE-only pairing send/receipt validation, Relay circuit delivery under cellular network conditions"  listed but no automated harness exists.

## Scope (~150 LoC, 1 file + dependencies)

### Harness file

Create `scripts/cross_platform_delivery_test.py`:

```python
#!/usr/bin/env python3
"""
Cross-platform delivery harness for SCMessenger.
Tests message delivery + receipt under various transport scenarios.

Usage:
    ./scripts/cross_platform_delivery_test.py --scenario <ble|wifi|relay|all> --device <android|ios|cli>
    ./scripts/cross_platform_delivery_test.py --scenario all --all-devices
"""

import argparse
import json
import subprocess
import sys
import time
from dataclasses import dataclass
from enum import Enum
from pathlib import Path

class Scenario(Enum):
    BLE = "ble"
    WIFI = "wifi"
    RELAY = "relay"
    ALL = "all"

class Device(Enum):
    ANDROID = "android"
    IOS = "ios"
    CLI = "cli"

@dataclass
class DeliveryResult:
    scenario: str
    sender: str
    receiver: str
    message: str
    sent_at: float
    received_at: float
    receipt_at: float
    success: bool
    
    @property
    def delivery_latency_ms(self) -> float:
        return (self.received_at - self.sent_at) * 1000
    
    @property
    def receipt_latency_ms(self) -> float:
        return (self.receipt_at - self.received_at) * 1000
    
    def to_dict(self) -> dict:
        return {
            "scenario": self.scenario,
            "sender": self.sender,
            "receiver": self.receiver,
            "message": self.message,
            "sent_at": self.sent_at,
            "received_at": self.received_at,
            "receipt_at": self.receipt_at,
            "delivery_latency_ms": self.delivery_latency_ms,
            "receipt_latency_ms": self.receipt_latency_ms,
            "success": self.success,
        }

def run_scenario_ble(sender: Device, receiver: Device) -> DeliveryResult:
    """BLE-only delivery. Both devices must be paired and within range."""
    # Implementation: invoke scm CLI on sender, poll for receipt on receiver
    ...

def run_scenario_wifi(sender: Device, receiver: Device) -> DeliveryResult:
    """WiFi Direct or local network delivery. Both on same SSID."""
    ...

def run_scenario_relay(sender: Device, receiver: Device) -> DeliveryResult:
    """Relay via bootstrap node. Receiver can be on cellular, behind NAT."""
    ...

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--scenario", choices=[s.value for s in Scenario], default="all")
    parser.add_argument("--device", choices=[d.value for d in Device], default="cli")
    parser.add_argument("--all-devices", action="store_true", help="Test all device pairs")
    parser.add_argument("--timeout-seconds", type=int, default=60)
    parser.add_argument("--output", default="cross_platform_results.json")
    args = parser.parse_args()
    
    scenarios = [Scenario(args.scenario)] if args.scenario != "all" else list(Scenario)
    devices = list(Device) if args.all_devices else [Device(args.device)]
    
    results = []
    for scenario in scenarios:
        for sender in devices:
            for receiver in devices:
                if sender == receiver:
                    continue
                result = run_scenario(scenario, sender, receiver)
                results.append(result)
                print(f"{'' if result.success else ''} {scenario.value} {sender.value}{receiver.value}: {result.delivery_latency_ms:.0f}ms delivery, {result.receipt_latency_ms:.0f}ms receipt")
    
    Path(args.output).write_text(json.dumps([r.to_dict() for r in results], indent=2))
    success_rate = sum(1 for r in results if r.success) / len(results) if results else 0
    print(f"\nSuccess rate: {success_rate * 100:.0f}% ({sum(1 for r in results if r.success)}/{len(results)})")
    sys.exit(0 if success_rate >= 0.8 else 1)

if __name__ == "__main__":
    main()
```

## File Targets

- `scripts/cross_platform_delivery_test.py` [NEW, executable]
- `requirements.txt` (NEW if not present; minimal: just stdlib for v1)

## Build Verification Commands

```bash
# Smoke: just check the script parses
python3 scripts/cross_platform_delivery_test.py --help

# Full run (requires all devices online)
./scripts/cross_platform_delivery_test.py --scenario all --all-devices --output /e/build-tools/test-results/cross-platform-$(date +%Y%m%d).json
```

## Acceptance Gates

1. `python3 scripts/cross_platform_delivery_test.py --help` works
2. Harness covers 3 scenarios: BLE, WiFi, Relay
3. Each scenario reports: delivery_latency_ms, receipt_latency_ms, success
4. JSON output is parseable and contains all fields
5. Exit code 0 if success rate  80%, 1 otherwise
6. Commit: `test: v0.2.1 cross-platform delivery harness (BLE/WiFi/Relay)`

## REQUIRES_USER_ACTION
Harness requires all device pairs to be online simultaneously. User coordinates Android + iOS + CLI on same network. Subagent cannot do this on its own.

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: PYTHON] [REQUIRES: GEMMA_4_31B] [DEPENDS_ON: P1_CORE_001, P1_CORE_002] [REQUIRES_USER_COORDINATION]
