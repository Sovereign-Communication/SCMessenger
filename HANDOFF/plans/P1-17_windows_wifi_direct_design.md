# P1-17 — Windows-side WiFi Direct: Feasibility + Role Decision (design note)

**Task:** P1-17 [OPUS+ spec -> SONNET] from `HANDOFF/V1_0_0_EXECUTION_PLAN.md` Section 2 (Stage D).
**Author:** Claude (native Cowork session), on operator direction (Lucas).
**Date:** 2026-07-05.
**Status:** DESIGN / DECISION-NEEDED — read-only pass, code- and lockfile-verified this session. No code changed. No commands run.
**Source ticket:** `HANDOFF/todo/P1_CORE_WINDOWS_WIFI_DIRECT_Peer_Absent.md`.
**Evidence trail:** `HANDOFF/plans/P1-15_transport_matrix_audit.md` section (d).
**Verification legend:** [V-READ] = confirmed by reading source / `Cargo.lock` this session. [DEVICE] = only settleable on real hardware. Nothing was compiled or run.

---

## 0. Question restated

The transport matrix (2.6) has a WiFi Direct cell for Windows <-> Android. P1-15(d) established the ground truth: **there is no Windows WiFi Direct participant today.** `core/src/transport/wifi_direct.rs` compiles on Windows but is inert — every action delegates to `crate::PlatformBridge`, which is only ever implemented in production by Android's `AndroidPlatformBridge.kt`. The CLI has no `PlatformBridge` impl and no WiFi Direct transport instantiation. `desktop_bridge/` has zero WiFi Direct references and is Linux-only (`zbus`, `cfg(target_os = "linux")`, `desktop_bridge/Cargo.toml:21`). [V-READ]

The plan's expected outcome (P1-17): **Android <-> Windows over a Direct group where Windows joins as a legacy client over the group's IP link (TCP dial to the group-owner IP on a negotiated/laddered port)**, OR an explicit operator-approved waiver narrowing this cell to Android <-> Android [BLOCKED-HW].

This note answers: is the legacy-client path actually feasible with available Windows APIs, what would it touch, and — build or waive?

---

## 1. Feasibility finding

### 1.1 The critical distinction: "legacy client" needs Wi-Fi *join*, not WiFi Direct P2P

There are two very different Windows mechanisms, and conflating them is the trap:

- **(A) Native WiFi Direct P2P (`Windows.Devices.WiFiDirect`).** WinRT `WiFiDirectDevice.FromIdAsync` / `WiFiDirectConnectionListener` — Windows acting as a *P2P peer* (its own GO or P2P client) speaking the Wi-Fi P2P protocol. This is the large, [DEVICE]-heavy path.
- **(B) Legacy-client join (ordinary Wi-Fi association).** An Android WiFi Direct group-owner is, at the radio layer, a **WPA2 SoftAP** with an SSID (`DIRECT-xy-...`) and a passphrase. Any device with a normal Wi-Fi stack — including Windows — can associate to that SSID as a plain Wi-Fi station **without any WiFi Direct P2P API at all**. Once associated, the GO hands out a DHCP lease (Android GO IP is conventionally `x.x.x.x`), and it becomes an ordinary IP link. Android exposes the SSID and passphrase via `WifiP2pManager` -> `WifiP2pGroup.getNetworkName()` / `getPassphrase()`.

**The plan explicitly specifies path (B)** ("Windows joins as legacy client over the group's IP link"). Path (B) does **not** require `Windows.Devices.WiFiDirect`. It requires:
1. getting the SSID + passphrase from the Android GO to the Windows box (a credential-ingress problem, not a radio problem), and
2. Windows performing a normal "connect to this SSID with this passphrase" Wi-Fi join, then
3. the existing libp2p/CLI TCP dial to the GO IP on the negotiated port (already the mechanism on the Android client side — `mobile_bridge.rs:1398`).

So the honest feasibility answer splits:

- **Legacy-client (B): FEASIBLE**, and materially smaller than a WiFiDirect native module. The only new Windows-native surface is a programmatic Wi-Fi join. Even that is optional for a first pass (see 1.3 — an operator/manual join reduces it to pure CLI dial + credential ingress with zero new native surface).
- **Native WiFiDirect P2P (A): technically available but out of scope** — larger new dependency surface, [DEVICE]-heavy, and unnecessary given (B) satisfies the plan. Recommend NOT building (A). It is called out only so the operator sees the fork.

### 1.2 What Windows APIs are actually available (and what's already in the tree)

Programmatic Wi-Fi join on Windows has three realistic shapes, smallest-surface first:

1. **`netsh wlan connect` + a WLAN profile XML** (shell-out, zero new crate). Windows connects to an SSID from a stored/added profile. The CLI already shells out for other ops; this adds an XML profile (`add profile`) then `connect`. No new dependency, no `unsafe`, no WinRT. Least elegant, most portable, testable today on the dev Windows box.
2. **Native WLAN API via the `windows` crate WinRT projection** — `Windows.Devices.WiFi` (`WiFiAdapter.ConnectAsync` with `WiFiAvailableNetwork` + `PasswordCredential`) or the classic Win32 `WlanConnect` (`Win32_NetworkManagement_WiFi`). This is a real native module but a *narrow* one (connect to SSID + passphrase, report link state).
3. **Full native WiFiDirect P2P (`Windows.Devices.WiFiDirect`)** — path (A), rejected above.

**Key lockfile finding [V-READ]:** the high-level `windows` WinRT projection crate is **already present transitively** in `Cargo.lock`:
- `windows 0.61.3` is pulled in by **`btleplug 0.11.8`** (`Cargo.lock:642-668`) — the BLE crate the CLI already depends on.
- `windows 0.62.2` is pulled in by **`if-watch`/`if-addrs`** (`Cargo.lock:2150-2166`) — libp2p's interface watcher.

So the WinRT projection machinery and its build tooling already compile as part of the current Windows build. **Neither pulls in the `Devices_WiFiDirect` or `Devices_WiFi` feature** — those are opt-in feature flags on the `windows` crate — so activating option 2 would still be a *new direct dependency with new features* on a workspace crate (a stack-addition escalation; see Section 4). But the finding matters: the cost of option 2 is "enable already-vendored feature flags + write a narrow join module", not "introduce the entire windows-rs stack from scratch". Option 1 (`netsh`) avoids even that.

### 1.3 The real work is credential + GO-IP ingress, not the radio

Regardless of join mechanism, the harder half is: **how do the group SSID, passphrase, and GO IP:port reach the Windows node?** Today they don't — the Android client learns the GO IP from an Android-only FFI callback (`on_wifi_direct_connection_info`, `mobile_bridge.rs:1374`), which has no Windows source. Options:

- **QR / manual paste (smallest, recommended first rung).** Android already renders QR for identity; extend or add a "group credential" QR/string carrying `{ssid, passphrase, go_ip, port}`. Operator scans/pastes on the Windows CLI; CLI joins + dials. Zero new wire protocol, no `peer_exchange` change. This alone closes the cell for the exit test with the least surface.
- **`peer_exchange` propagation (larger, coordinate with P1-12).** Piggyback group credentials on the advertise surface. This is a wire-semantics change and overlaps P1-10 §4 item 1 — do NOT do this in P1-17; it belongs to P1-12's audit-gated wire-format decision if pursued at all.

The GO **port** must be the negotiated/laddered port from P1-13, never a second hardcoded `9001` (the ticket's explicit "Do NOT"). Because P1-17 depends on P1-13 landing the negotiated port, and P1-13 depends on P1-11/P1-12, **P1-17 implementation queues behind P1-11/12/13** (see Section 5).

### 1.4 Feasibility verdict

**The legacy-client path (B) is feasible** and can be built with **zero new native dependency** in its smallest form (netsh join OR manual join + QR credential ingress + reuse of the existing CLI TCP dial). A native Wi-Fi join module (`windows` crate `Devices_WiFi` feature) is an *optional* refinement that improves UX but is a stack-addition escalation. The native WiFiDirect P2P module (A) is out of scope and not recommended.

There is one genuine [DEVICE] unknown that no amount of reading can settle: **whether this specific Windows machine's Wi-Fi adapter + driver will associate to a Pixel's `DIRECT-*` SoftAP and get a route to `x.x.x.x`.** Some Windows Wi-Fi drivers refuse or deprioritize `DIRECT-` SSIDs. This must be smoke-tested on hardware **before** committing implementation effort — it is the make-or-break gate and is cheap to check manually (Section 6).

---

## 2. Recommendation (build vs. waiver) — DECISION NEEDED FROM OPERATOR

**Recommendation: conditional BUILD via the legacy-client path, gated on a 20-minute hardware pre-check.**

Concretely, the recommended sequence is:

1. **[HUMAN + DEVICE pre-check, do this first, before any code]** On the dev Windows box + Pixel: manually create a WiFi Direct group on Android (or use an existing app path that does), read its SSID/passphrase, and manually join it from Windows (Settings -> Wi-Fi, or `netsh wlan connect`). Confirm Windows associates and can `ping`/TCP-reach the GO IP (`x.x.x.x`). **This is the feasibility gate.**
   - **If the manual join works:** proceed to build the [SONNET] ticket below (P1-17-IMPL) — the automation is then low-risk.
   - **If Windows refuses to associate to the `DIRECT-*` SoftAP** (driver-level rejection, no route): **take the waiver** — narrow the cell to Android <-> Android [BLOCKED-HW], because path (B) is physically blocked on this hardware and path (A) native WiFiDirect is out of scope. Record the waiver in matrix 2.6 with the driver-rejection reason.

2. **If BUILD:** implement the smallest-surface slice first (manual/QR credential ingress + CLI dial + reuse negotiated port), defer the automated `netsh`/native Wi-Fi join to a follow-up. See ticket P1-17-IMPL.

**Why not silently waive now:** the plan (P1-17) and the ticket both forbid a silent downgrade, and the feasibility case for (B) is genuinely positive on paper — it deserves the cheap hardware check before conceding the cell. **Why not commit to an unconditional build:** the [DEVICE] driver-association unknown (1.4) is real and can kill path (B) regardless of code quality. The operator makes the call *after* the pre-check.

**This is the "WiFi Direct Windows-side scope" open decision in plan Section 5.3 / open item 3.** It requires operator sign-off either way.

---

## 3. Implementation shape (if BUILD)

Decomposed into one [SONNET] ticket: `HANDOFF/todo/P1-17_Windows_WiFi_Direct_Legacy_Client.md` (written alongside this note). Summary of what it touches:

- **`cli/src/`** — new: accept group credentials (`{ssid, passphrase, go_ip, port}`) via a CLI command / QR-string paste; a Windows Wi-Fi join step (start with manual/`netsh`, native optional); then a TCP dial to `go_ip:port` reusing the existing dial path (`cli/src/main.rs` dial loop ~1462-1494, `SwarmCommand::Dial` via `swarm.rs:4104`).
- **`core/src/mobile_bridge.rs:1398`** — the Android-client hardcoded `tcp/9001` dial must be de-hardcoded by **P1-13 first**; P1-17 consumes the negotiated port and must not add a second hardcode.
- **`core/src/transport/wifi_direct.rs`** — `GroupInfo` (line 49) may gain a `port` field so the GO's actual bound port is carried, but this is the **P1-10 §4 item 2 sign-off** — P1-17 should prefer the "client tries the port ladder against `group_owner_ip`" no-wire-change alternative unless the operator already approved the `GroupInfo.port` field.
- **Android side (out of P1-17 Rust scope, coordinate):** expose the group SSID/passphrase for the QR/credential export (`WifiDirectTransport.kt` / `AndroidPlatformBridge.kt`). This is an Android-app change, tracked as a coordination dependency, not core Rust.

No change to `desktop_bridge/` is required for path (B) — it stays Linux-BLE-only.

---

## 4. REQUIRES OPERATOR SIGN-OFF (stack additions / contract changes)

Per the CLAUDE.md escalation rule ("Technology stack migrations or additions") and mirroring `P1-10_adaptive_port_selection_design.md` §4, the following are flagged separately, not buried:

1. **The scope decision itself (build vs. waiver).** [HUMAN] — Section 2. Decided after the hardware pre-check. Waiver, if taken, is recorded in matrix 2.6 as Android <-> Android [BLOCKED-HW] with the driver-rejection reason.

2. **New direct Windows-native dependency — ONLY if the operator wants the automated native Wi-Fi join (option 2, Section 1.2).** This would add the `windows` crate as a **direct** dependency of `scmessenger-cli` with the `Devices_WiFi` (WinRT) or `Win32_NetworkManagement_WiFi` feature enabled, `cfg(target_os = "windows")`-gated. The crate is already vendored transitively (via `btleplug`, `if-watch`) so no new download, but a **new direct dependency with new features on a workspace crate is a stack addition** and needs sign-off. **The recommended first slice avoids this entirely** by using manual/`netsh` join — item 2 is only on the table if the operator wants the polished automated path. Native WiFiDirect P2P (`Windows.Devices.WiFiDirect`, path A) is **not** recommended and is not being requested here.

3. **`GroupInfo` gains a `port` field** (`core/src/transport/wifi_direct.rs:49`) — this is the **same** contract change already flagged in `P1-10_adaptive_port_selection_design.md` §4 item 2 (crosses the FFI/JNI boundary; Kotlin `GroupInfo` must gain the field). P1-17 does not re-decide it; it inherits whatever P1-10/P1-12 decided. The no-wire-change alternative (port ladder against `group_owner_ip`) is available if the field is not approved.

4. **Credential-over-`peer_exchange` propagation is explicitly NOT proposed for P1-17.** If ever pursued, it is a wire-format change owned by P1-12 (overlaps P1-10 §4 item 1) and needs its own sign-off there. P1-17's recommended ingress is QR/manual, which is contract-free.

Item 1 is the real decision. Item 2 is a *conditional* stack-addition ask the operator can decline while still shipping the cell. Items 3–4 defer to existing P1-10/P1-12 sign-offs.

---

## 5. Blocking dependencies (do not imply this can start now)

- **Hardware pre-check (Section 1.4 / 2.1) gates everything.** No implementation until the manual Windows-join-to-Android-GO smoke test passes on the dev box + Pixel. If it fails, the outcome is the waiver, not a build.
- **P1-04** owns the `transport/` hotspot lane; **P1-11 -> P1-12 -> P1-13** deliver the negotiated/laddered port that P1-17 must consume (plan §2.5, `P1-10` §5). P1-17 implementation queues behind P1-13 so it does not add a second hardcoded port.
- **[AUDIT-GATE]:** P1-17 touches `core/src/transport/` (and the dial path). Mandatory `crypto-security-auditor` review before done, `release-gatekeeper` before merge, per `.claude/rules/security.md` and plan §1.1.
- **[DEVICE]:** the exit test (Section 6) is hardware-only — one Windows box + one Pixel (available per plan §Resources). No second Android needed for this cell (that constraint is WiFi Aware's, not Direct-legacy-client's).

---

## 6. Exit test (the plan's bar, made literal)

Pre-check (feasibility gate, manual, do first):
1. Android creates a WiFi Direct group (becomes GO). Read `getNetworkName()` (SSID) + `getPassphrase()`.
2. Windows manually joins that SSID with the passphrase (Settings or `netsh wlan connect`).
3. Confirm Windows gets an IP on the GO subnet and can TCP-reach the GO IP (`x.x.x.x` conventionally). PASS -> build; FAIL(driver rejects `DIRECT-*`) -> waive.

Full cell (after implementation):
4. Credentials reach Windows (QR/paste). CLI joins the group and TCP-dials `go_ip:<negotiated port from P1-13>`.
5. A message composed on the phone arrives on Windows over the Direct group link, and a message composed on Windows arrives on the phone — both directions, cold-start included, twice (per matrix 2.6 rule). Evidence to ledger.

---

## 7. Critical files (for the implementer / operator)

- `core/src/transport/wifi_direct.rs` — inert-on-Windows transport shim; `GroupInfo` (49); `set_on_message_received` is a no-op (211) — data flows over the IP link, not a callback.
- `core/src/mobile_bridge.rs:1374` — `on_wifi_direct_connection_info` (Android-only GO-IP source); `:1398` — the `tcp/9001` client-dial hardcode (P1-13 target; P1-17 must consume negotiated port).
- `cli/src/main.rs` — dial loop (~1462-1494), bootstrap/add-peer path (~2114, 2397); reuse for the Windows GO dial.
- `core/src/transport/swarm.rs:4104` — `SwarmCommand::Dial` (promiscuous dialer the CLI join would drive).
- `desktop_bridge/Cargo.toml:21` — Linux-only; no change needed for path (B).
- `Cargo.lock:642-668` (btleplug -> `windows 0.61.3`), `:2150-2166` (if-watch -> `windows 0.62.2`) — WinRT projection already vendored transitively; a direct `Devices_WiFi` dependency (if chosen) enables an already-present crate's opt-in feature.
- `android/.../transport/WifiDirectTransport.kt`, `android/.../service/AndroidPlatformBridge.kt` — Android GO side; source of SSID/passphrase for credential export (coordination dependency).
