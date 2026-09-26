# ESCALATION: Android DNS Resolver Startup Failure (os error 2)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Status:** Escalated for human / Opus+ validation
**Priority:** P0 (Blocks all Android emulator / device transport execution)
**Date:** 2026-07-10
**Author:** Gemini (Antigravity)

---

## 1. The Problem

During emulator verification of the Android build (`scm_pixel_34`), the Swarm failed to start with a `NetworkException` / `Swarm failed to start listening` error. 

Reading the diagnostic logs from `/data/data/com.scmessenger.android/files/swarm_error.txt` revealed the underlying crash:
```
START_SWARM_WITH_CONFIG ERROR: Dns

Caused by:
    proto error: io error: No such file or directory (os error 2)
```

### Root Cause Analysis
In `core/src/transport/swarm.rs`, `libp2p::SwarmBuilder::with_tcp` and `with_websocket` are called. Under the hood, if the `"dns"` feature of `libp2p` is enabled, these helper methods wrap the TCP and WebSocket transports in `libp2p::dns::tokio::Transport::system`. 

The system resolver (via Hickory DNS / `trust-dns-resolver`) attempts to read `/etc/resolv.conf` to parse the host's DNS configuration. However, **Android does not have `/etc/resolv.conf`**, causing the system resolver initialization to fail immediately with `os error 2` (File not found). This prevents the Swarm from starting at all.

---

## 2. Proposed Options for Course Correction

### Option A: Use a custom DNS resolver with a static fallback
Wrap the TCP/WS transport manually using `libp2p::dns::tokio::Transport::custom` on Android, seeding it with a static public DNS resolver (e.g. Google's `8.8.8.8` or Cloudflare's `1.1.1.1`).
* **Pros:** Retains full DNS resolution capability on Android (needed if dialing WAN bootstrap relays via domain names).
* **Cons:** Hardcodes public DNS IPs (could be a privacy concern in highly sovereign networks).

### Option B: Bypassing DNS wrapping on Android entirely
Construct the transport using `with_other_transport` without wrapping it in libp2p-dns for the Android target.
* **Pros:** Simple, robust, completely avoids Hickory DNS initialization on Android.
* **Cons:** Android nodes would only be able to dial IP-based multiaddrs (e.g., `/ip4/192.168.0.121/tcp/9001`) and not domain-based ones (e.g., `/dns4/relay.scmessenger.org/...`). However, for LAN/BLE/WiFi Aware, this is already the case.

### Option C: Conditional DNS Compilation
Disable the `"dns"` feature for the Android build target in `core/Cargo.toml` so that libp2p compiles without Hickory DNS support entirely, falling back to raw TCP.

---

## 3. Request for Confirmation

Please review and confirm which option to proceed with. 
Once validated by a human or Opus+ tier agent, we will implement the fix in `core/src/transport/swarm.rs` to allow the Android Swarm to successfully listen.
