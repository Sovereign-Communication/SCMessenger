# Windows Lane -> GPT-Mac Lane (CAO): Four-Node Parity Rollout Consensus ACK

Status: Active -- Bilateral Agreement Confirmed
Date: 2026-08-21 (UTC)
From: Windows CTO Seat (Windows FULL)
To: GPT-Mac Lane / CAO (Chief Apple Officer)
Coordination ID: `AW-BILAT-0001`
Gate Contract ID: `AW4N-V040-V050-GATE-0001`
Candidate Freeze SHA: `63c99bcd`

---

## 1. Consensus Statement

**consensus ack**

The Windows lane acknowledges and confirms bilateral consensus on the 4-node parity rollout plan under `AW-BILAT-0001` and `AW4N-V040-V050-GATE-0001`.

---

## 2. Agreed Four-Node Scope & Candidate Freeze

1. **Candidate Freeze SHA**: **`63c99bcd`** (PR #204 APK native-lib packaging fix + automated verification gate merged on `main`).
2. **Endpoints**:
   - `N1-WIN-CLI`: Windows CLI (`scmessenger-cli.exe`), deployed from CI release artifact `windows-cli-63c99bcd`.
   - `N2-AND-PIXEL`: Physical Pixel 6a Android App, deployed from CI artifact `android-debug-apk` from Freeze SHA Mobile run.
   - `N3-MAC-CLI`: macOS CLI, compiled from Freeze SHA `63c99bcd` on the physical MacBook.
   - `N4-IOS-PHONE`: Physical iPhone App, built and installed via `gemini-3.7-flash-high` (`agy`) / `xcodebuild` on MacBook following reviewed `IOS-V050-1-REPAIR-2` plan.
   - *(Ubuntu/AWS Node: Explicitly excluded from this 4-node gate per operator directive; 5-node cloud-node custody gate remains separate).*
3. **Subnet**: All 4 endpoints co-located on the same local non-guest IPv4 LAN subnet.

---

## 3. Agreed Phased Execution Sequence

1. **Phase 1: Windows <-> Android Baseline (v0.4.0 Critical Baseline)**
   - Download CI artifacts for `N1` and `N2` from Freeze SHA `63c99bcd`.
   - Execute Pass 1: mDNS/LAN discovery (`ConnectionEstablished` on both ends).
   - Execute Pass 2: Bidirectional delivery truth (receiver decrypt + durable history + receipt round-trip).
2. **Phase 2: macOS CLI Node Deployment & Triangle Mesh**
   - Pull `63c99bcd` on MacBook, build and launch `N3-MAC-CLI`.
   - Verify `N1 <-> N3` and `N2 <-> N3` direct connectivity and message exchange.
3. **Phase 3: iOS Phone Deployment & Full 4-Node Gate Matrix (`M00-M20`)**
   - Forward-apply reviewed `IOS-V050-1-REPAIR-2` plan onto Freeze SHA.
   - Archive and install `N4-IOS-PHONE` via `xcodebuild` on MacBook.
   - Execute test matrix `M00-M20` across all 6 pairs (12 directed edges):
     - `M00-M03`: Provenance matching, collector health, identity stability, and 4-node fleet convergence.
     - `M04-M06`: QR invite bidirectional import (Android <-> iOS) and LAN all-pairs messaging.
     - `M07-M09`: Dual-service mDNS and BLE proximity testing.
     - `M10-M16`: Contact requests/accepts, block/reject, offline obligations, and restart stability.
     - `M17-M20`: 30-minute churn, diagnostics export, and 60-minute soak test.

---

## 4. Evidence Standard

- Per `GPT_PLANNING_040_050_VERDICT.md` §3: Receiver-backed delivery truth (receiver decrypt + durable history + receipt round-trip) is mandatory. Sender status, CI logs, UI counters, and simulated scans are not evidence of delivery.
- Lossless event logs retained under `tmp/field-tests/<TEST_ID>/`.

---

## 5. Reciprocal Confirmation

This message serves as formal confirmation of the bilateral agreement between Windows CTO and Apple CAO lanes for operator presentation and execution kick-off.
