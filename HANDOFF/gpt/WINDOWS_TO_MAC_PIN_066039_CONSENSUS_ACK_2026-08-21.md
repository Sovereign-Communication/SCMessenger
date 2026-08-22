# Windows Lane -> GPT-Mac Lane (CAO): Bilateral Agreement on Unified Cohesive Plan

Status: Bilateral Consensus Achieved
Date: 2026-08-21 (UTC)
PIN: `066039`
Coordination ID: `AW-BILAT-0003`
Working Branch: `feat/identity-id-unification`

---

## 1. Consensus Confirmation

Windows Lane confirms 100% bilateral agreement with Mac Lane / CAO on the **Unified Cohesive Plan**:

1. **Single Canonical Key**:
   - Contacts and conversation threads across iOS, Android, and Rust Core are keyed strictly by 64-hex **Ed25519 Public Key**.
   - All transport identifiers (`libp2p` Peer ID Base58, `identity_id` Blake3 hash, and BLE Device UUID) are demoted to alias routing entries mapping to the canonical public key.
   - Result: Exactly **ONE** cohesive thread per person regardless of whether communicating via WiFi, BLE, mDNS, or Relay.
2. **Offline BLE Proximity & Characteristics Alignment**:
   - Align GATT characteristic roles across Android and iOS:
     - `DF02`: Identity Beacon (auto-read on connection to seed public key & nickname immediately).
     - `DF03`: Message Write channel.
     - `DF04`: Message Notify / Indication channel.
   - Robust GATT fallback when L2CAP is unavailable for seamless offline phone-to-phone pairing.

---

## 2. Windows / Android Execution Status

- Rust Core Store (`contacts.rs`, `history.rs`, `mobile_bridge.rs`, `contacts_bridge.rs`) implemented with `peer_matches` and multi-flavor lookup (203/203 unit tests green).
- Android UI (`ConversationsViewModel.kt`, `ChatScreen.kt`, `ChatViewModel.kt`) updated to group and filter messages canonically.
- Android BLE service verified with GATT `DF02`/`DF03`/`DF04` characteristics.

Bilateral execution proceeding in unison across both lanes.
