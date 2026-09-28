# DISPATCH: IMPLEMENT -- Android mDNS Discovery Parity (Ledger Gap Phase 1)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Branch: fix/android-mdns-parity-2026-08-05 (operator-approved, PR-first)
Ticket: HANDOFF/todo/LEDGER_SHARING_ANDROID_NODE_VISIBILITY_2026-08-05.md
Audit: HANDOFF/review/LEDGER_VISIBILITY_AUDIT_QWENPAID_2026-08-05.md

Mode: IMPLEMENTATION (unified diffs). Do not apply anything yourself; the
orchestrator reviews and applies. Output diffs ONLY for files in SCOPE.

## Problem

Field test 2026-08-05: freshly installed Android node sees only the one
peer it paired with; the iOS node on the same LAN sees 4 nodes + 1
headless. Phase 1 of the fix (operator decision "both, sequenced"): make
Android LOCAL discovery reliable so LAN fleet views converge without any
change to ledger disclosure policy. Known open item: Android 14+ (API 34)
can throw SecurityException on NsdManager listener registration without
the right local-network permission state; April logs show "mDNS fallback:
no LAN peers discovered within timeout".

## SCOPE (exactly these files)

- android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt
- android/app/src/test/java/com/scmessenger/android/transport/MdnsServiceDiscoveryTest.kt
- android/app/src/main/java/com/scmessenger/android/utils/Permissions.kt

## REQUIRED CHANGES

1. SECURITYEXCEPTION HARDENING (API 34+): wrap NsdManager
   registerService/discoverServices listener registration so a
   SecurityException is caught, logged with an actionable message, and
   retried after permission/Wi-Fi state changes instead of leaving
   discovery silently dead. Track the failure state so callers can query
   it (no silent timeouts).
2. PERMISSION GATING: discovery must only start when NEARBY_WIFI_DEVICES
   (and any API-level-required location permission) is granted; on denial
   emit an explicit, loggable failure rather than timing out. Keep the
   existing permission UX flow intact.
3. REGISTRATION RECOVERY: on Wi-Fi state change or listener loss,
   re-register idempotently (guard against duplicate registrations; the
   existing inFlightResolves tracking pattern shows the established style).
4. INTEROP ASSERTION: document and assert the advertised/discovered
   service type constant (`_p2p._udp` -- libp2p-mdns default). If iOS or
   CLI peers advertise differently, that is the interop bug -- add a test
   that pins the expected service type and note the cross-platform check
   in the report.
5. DIAGNOSTICS: emit structured log lines (start/stop/error/permission-
   denied/peer-found with address) via the app's existing logging path so
   a future field gap is diagnosable from logcat alone.
6. TESTS: extend MdnsServiceDiscoveryTest.kt covering (1), (2), (3) with
   the existing reflection-based listener pattern already in that file.

## HARD CONSTRAINTS

- Kotlin only; NO core/ Rust changes in this dispatch.
- NO changes to ledger disclosure policy (that is Phase 2, separate
  design).
- No emojis. Match existing Kotlin style in the module.
- If a change would require AndroidManifest.xml edits, DO NOT make them;
  list the required manifest change in the report instead.

## Report format (mandatory final block)

RESULT: DONE|BLOCKED|FAILED
VERIFICATION: NONE|CONTAINER(exact commands run)
FILES: <files with diffs>
NOTES: <max 8 lines: what changed, interop findings, what the orchestrator must run>
