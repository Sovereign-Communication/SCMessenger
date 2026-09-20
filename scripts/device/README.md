# Device verification scripts -- inventory and device-day runbook

Status: Active (T7 staging pass, 2026-09-19)
Last updated: 2026-09-19
Owner lane: Freebuff / DeepSeek V4 Flash
Implements: `HANDOFF/freebuff/queue/V040_T7_ANDROID_PARITY_STAGING.md`

Device time is the scarcest resource in this project. Everything here exists so
that when the handset appears the session is verification and log capture, never
authoring. One script per device-gated item; each one states its evidence bar and
its failure disposition before it runs, checks its preconditions first, and prints
one verdict per assertion.

## Authority

- **Seat split**: `docs/rules/ANDROID.md` -- Android agents are authorized for app
  updates and passive log collection only; active device/mesh driving belongs to
  the Windows, aidws and Ubuntu agents. Every script here drives the Windows/CLI
  side and prompts the **operator** for the handset's human path. The only things a
  script does to the phone itself are install, start/stop the app process, and
  read-only pulls.
- **Handset evidence**: `HANDOFF/RUNBOOK_PIXEL_PASSIVE_VERIFICATION_2026-09-11.md`
  (no `input tap` / `input swipe`, no `pm clear`, no forced `loadPeers`).
- **Evidence bar for the v0.4.0 gates**, verbatim from `SHIP_PLAN.md` G3:
  *receiver-side decrypt + durable history + receipt -- NOT transport ACKs, NOT UI
  counters, NOT BLE local acceptance*.

## Device-day order of operations

1. **Reinstall before anything is scored.** The fleet currently runs a
   *debug*-signed build and the scored runs must be on the *released* APK, so the
   first scored run needs an uninstall-and-reinstall (`docs/ANDROID_RELEASE_SIGNING.md`).
   Identity and history on the device **do not survive it**, so do this first, not
   mid-session.
2. **Build identity is checked, not assumed.** Every script compares the installed
   APK's SHA-256 against the intended artifact (`--apk <path>` or
   `SCM_EXPECTED_APK_SHA`) and compares the two sides' build stamps -- P1-04's rule
   that a cross-device session starts by comparing stamps.
3. **Start the CLI node and confirm it answers**: `curl -s $SCM_API/health`. Scripts
   abort with `[FAIL]` if it does not.
4. **Run the gate items first** (D4, D6, D7), then the churn gate, then the Phase 1
   validation passes. Each run writes its artifacts to `tmp/device/<item>-<utc>/`
   (`verdicts.log`, `run.env`, captured logs) and prints the directory at the end.
5. **Read one summary per script.** `FAIL` means an assertion failed, `PARTIAL`
   means nothing failed but something could not be evaluated, `PASS` means every
   assertion was evaluated and held.

## Inventory -- every device-gated item

Four contract elements, per T7: **1** merged code, **2** verification script,
**3** log-capture recipe, **4** pre-registered failure disposition. `[OK]` = present,
`[--]` = absent or not applicable to a scored run.

| Item | What the device is needed for | 1 | 2 | 3 | 4 | Script |
|---|---|---|---|---|---|---|
| **D4** message + receipt, handset <-> Windows | both directions, receiver-side decrypt + durable history + receipt | [OK] | [OK] | [OK] | [OK] | `d4_two_device_message.sh` |
| **D6** transport racing | first-choice transport actually removed, delivery still lands | [OK] | [OK] | [OK] | [OK] | `d6_transport_racing.sh` |
| **D7** offline proximity | no-internet exchange in both directions | [OK] | [OK] | [OK] | [OK] | `d7_offline_proximity.sh` |
| **Churn gate** | a moved cloud address rejoins unaided and gossips to the handset | [OK] | [OK] | [OK] | [OK] | `g3_churn_gate.sh` |
| **P1-09** LAN E2E (mDNS + TCP + WS) | discovery, dial, message + receipt, kill-and-recover, two passes | [OK] | [OK] | [OK] | [OK] | `p1_09_lan_e2e.sh` |
| **P1-14** hostile network | firewall rungs 9001/9002 -> 443 -> 80, delivery and remembered path | [OK] | [OK] | [OK] | [OK] | `p1_14_hostile_network.sh` |
| **P1-16** BLE worst-case cell | WiFi off, no internet, message both directions over BLE | [--] see note | [OK] | [OK] | [OK] | `p1_16_ble_data_path.sh` |
| **P1-17** WiFi Direct cell | Direct group exchange (needs a second device) | [--] waived | [OK] | [OK] | [OK] | `p1_17_wifi_direct.sh` |
| **P1-18** relay cells | custody held while the peer is away, then delivered and receipted | [OK] | [OK] | [OK] | [OK] | `p1_18_relay_cells.sh` |

Sources: `SHIP_PLAN.md` G3 (D4/D6/D7 and the churn gate, with the operator's
2026-08-31 address-churn directive at `SHIP_PLAN.md:295-301`),
`HANDOFF/V1_0_0_EXECUTION_PLAN.md` P1-09/P1-14/P1-16/P1-17/P1-18, and
`HANDOFF/todo/_QUEUE.md:326/333/340` for the waivers and post-exit debt.

Notes on the two items that are **not** marked ready on code alone:

- **P1-16**: element 1 is unproven by construction. P1-15's audit question --
  whether CLI `ble_mesh` <-> Android BLE is a data path today or discovery-only --
  is not settled in the plan text, and no source change for the BLE data path is in
  this pass. The script exists so that the answer is produced by running it: if
  delivery fails, that failure is the finding.
- **P1-17**: element 1 is waived. `HANDOFF/todo/_QUEUE.md:326` records the waiver
  and the corrected fleet has one handset, so the Android<->Android Direct cell
  cannot be formed. The script prints `[SKIP]` with that reason and runs only under
  `--force` with two devices attached.

### Device-gated defect tickets (no per-ticket script in this pass)

T7 asks for "any `HANDOFF/todo/` ticket whose acceptance needs a device". These
need the handset to *diagnose*, not to pass a gate, and their pass/fail condition
is not yet defined as a runnable assertion, so element 2 is `[--]` for them by an
explicit decision rather than by omission. They consume the same passive evidence
the scripts already pull (`logcat.txt`, `mesh.log` UTF-16-decoded,
`mesh_diagnostics.log`, `ledger.json`), so a session that runs any script leaves
what they need on disk.

| Ticket | What it needs the device for |
|---|---|
| `P0_SEND_CRYPTO_FAILS_VS_DELIVERED_2026-08-30.md` | on-device confirmation of the signing-key/envelope mismatch hypothesis |
| `P1_ASYNC_DELIVERY_RECEIPTS_DO_NOT_CONVERGE_2026-08-10.md` (+ `_LIVE_RCA_2026-08-25.md`) | receipt convergence observed on the handset |
| `P1_ANDROID_COMPOSE_CRASH_RECURRENCE_2026-09-15.md` | the crash class recurs on a real device, not in CI |
| `P1_ANDROID_CHAT_ORDER_CROSS_CLOCK_2026-09-17.md` | store-level ordering ground truth from the device |
| `D4_MOBILE_BRIDGE_HISTORY_FLAVOR_MATCHING_2026-08-30.md` | history flavor matching across the mobile bridge |
| `GHOST_LEDGER_PRUNE_G1_2026-09-11.md` | ledger prune behaviour observed on the handset |
| `ANDROID_CI_APK_SIGNATURE_BLOCKS_INPLACE_UPGRADE_2026-08-09.md` | signing divergence between local and CI builds |
| `P1_RELEASE_SIGNING_GATE_FAIL_CLOSED_2026-09-16.md` | release-signing gate behaviour on a device |
| `ANDROID_INBOUND_CRYPTOERROR_2026-08-09.md`, `ANDROID_FFI_IN_COMPOSITION_BUILD_KILLER_2026-09-10.md`, `P0_ANDROID_FINITE_RETRY_ABANDONMENT_2026-08-10.md`, `P0_ANDROID_SELF_RATCHET_RESET_2026-08-10.md`, `P1_ANDROID_UNIFFI_CURVE_CHECK_RELOCATION.md`, `P2_ANDROID_KOTLIN_WARNING_TRIAGE_2026-09-14.md`, `CORE_DIAL_CANDIDATE_DOUBLE_CIRCUIT_PRUNE_2026-09-10.md` | runtime behaviour that CI cannot reproduce |

None of these is marked ready. Marking one ready would need its own script with a
pass/fail stated in advance, which is device-side acceptance authoring this pass did
not do.

## The signing consequence, per scored item

The fleet runs a **debug**-signed build while D4/D6/D7 must be scored on the
**released** APK (`docs/ANDROID_RELEASE_SIGNING.md`), so a scored run needs
uninstall-and-reinstall first and the device's identity/history does not survive it.

`dev_require_signature` records the installed signer digest into the run directory
for every script. With `SCM_RELEASE_CERT_SHA256` set it asserts release-signing; with
it unset it prints `[WARNING]` that the run is unverified as release-signed -- it
never silently accepts a debug build as a scored one.

Scored items (release-signed APK required): **D4, D6, D7, churn gate**.
Rehearsable on a debug build without invalidating anything: **P1-09, P1-14, P1-16,
P1-18** -- but their evidence is only reusable if the build on the device is the
build the evidence is attributed to, which is what the stamp check enforces.

## Flags and environment

Common to every script (parsed in one place, `lib.sh`):

| Flag | Effect |
|---|---|
| `--apk <path>` | intended APK artifact to compare the installed one against |
| `--expected-sha <hex>` | alternative when the artifact is not on this machine |
| `--serial <serial>` | pin one adb device (required when more than one is attached) |
| `--api <url>` | Windows/CLI node API (default `http://127.0.0.1:9876`) |
| `--cloud <host:port>` | cloud node, for the items that need it |
| `--peer <peer-id>` | handset peer id, when `/api/peers` does not report one |
| `--confirmed` | do not wait on stdin for operator steps |
| `--run-root <dir>` | where run directories are written (default `tmp/device`) |

Per-item extras: `g3_churn_gate.sh` takes `--previous-cloud <ip>` and `--window <secs>`;
`p1_09_lan_e2e.sh` takes `--passes <n>`; `p1_16_ble_data_path.sh` takes
`--mac-rotation`; `p1_18_relay_cells.sh` takes `--relay-only`; `p1_17_wifi_direct.sh`
takes `--force`. An unrecognised flag exits 2 with the offending argument on stderr
rather than being ignored.

Capture windows are environment-tunable: `SCM_D6_WINDOW_SECS`,
`SCM_LAN_WINDOW_SECS`, `SCM_CHURN_WINDOW_SECS`, `SCM_RELAY_WINDOW_SECS`,
`SCM_RUNG_CAP_SECS`, `SCM_MAC_ROTATION_SECS`. Log-capture recipes are recorded per
run in `run.env`.

## Self-test

`scripts/device/selftest.sh` drives the machinery against a stub `adb`: the verdict
vocabulary, the no-device preconditions, the JSON field reader (present, absent and
malformed), the UTF-16 `mesh.log` decode and the TCP probe. It proves the scaffolding
works while the handset is away. It deliberately does **not** produce a verdict for
any item -- a stub cannot prove an item is parity-ready, and T7 acceptance 3 forbids
claiming otherwise.

## Verification status of this staging pass

Exercised on 2026-09-19 (no handset attached):

- `bash -n` clean on all eleven scripts.
- `scripts/device/selftest.sh`: every machinery check passed, exit 0.
- Every item script reaches its precondition and fails fast with
  `[FAIL] no device: adb reports no handset in 'device' state` -- no stack traces.
- `g3_churn_gate.sh` was specifically re-run twice after a defect was found in it:
  with no cloud address it now `[SKIP]`s the rejoin instead of reporting `[OK]`, and
  against an already-healthy rig it `[WARNING]`s that a rejoin cannot be attributed
  to an address change. A healthy rig reports `DirectPreferred` whether or not
  anything churned, so without that guard the gate would have printed a pass that
  proved nothing.

**Not verified, and not claimed**: no item has a device verdict. What a real session
must still establish is the content of each table row above -- in particular whether
BLE carries data at all (P1-16) and whether the fallback path is exercised end to end
(D6).
