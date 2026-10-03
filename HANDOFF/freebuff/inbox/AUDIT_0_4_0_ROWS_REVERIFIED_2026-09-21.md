# Canonical audit -- 0.4.0-target rows re-verified (2026-09-21)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: report + two rows fixed in the PR that carries this note
Filed: 2026-09-21, Freebuff lane
Source rows: `HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_INDEX.md`
(FINAL, iterations 0-6) and the remediation lanes in
`HANDOFF/V040_CTO_MASTER_PLAN_2026-09-20.md` "0.4.0-target findings".

Purpose: the master plan already had to correct two audit rows that were stale
(TRN-04, TRN-07). Before the tag, the 0.4.0-target rows were re-run the same way
-- from the commands, not from the audit's prose -- so nobody re-dispatches work
that is already done or believes a row is closed when it is not.

| Row | Audit claim | Re-run 2026-09-21 | Disposition |
|---|---|---|---|
| CO-A-002 | MED 0.4.0: operator-facing diagnostics name relay servers as a role | `grep -rn "relay server" android/` -> 4 live strings in `network/DiagnosticsReporter.kt` (the audit named 2; 2 more in the same block use `relay(s)` as the same role noun) | **FIXED** in this PR |
| CO-A-003 | LOW 0.4.0: `network_security_config.xml` comment names "public mesh-relay servers" | comment present at line 24 | **FIXED** in this PR |
| CO-B-002 | HIGH 0.4.0: `docs/ID_UNIFICATION_IMPLEMENTATION.md` is an orphan with non-canonical instructions | header reads "**Status:** SUPERSEDED / HISTORICAL", "Superseded: 2026-09-20 (canonical outlier audit CO-B-002)", with a canonical-authority section pointing at `core/src/api.udl` | **STALE ROW -- already fixed 2026-09-20** |
| CO-G-001 | MED process: 18 unindexed freebuff queue files | `ls HANDOFF/freebuff/queue/` = 50 files; `grep -o "queue/[A-Za-z0-9._-]*" HANDOFF/freebuff/README.md \| sort -u` = 12 referenced | **OPEN, larger than the audit measured** |
| CO-G-002 | MED 0.4.0: stale-premise P1 outbox ticket | `todo/P1_CLI_OUTBOX_CANONICAL_DRAIN_AND_SLED_UNIFICATION_2026-09-16.md` header reads "RE-SCOPED 2026-09-20 (CO-G-002) ... FIXED ON MAIN ... Do not re-implement" | **STALE ROW -- already fixed 2026-09-20** |
| CO-E-001 | MED 0.4.0: 28 stray AWS-IP copies incl. 3 active tickets | `grep -rln "3\.91\.5\.1\|18\.234\.62\.247"` (excluding `.git`) = **50 files** | **OPEN, larger than the audit measured; see below** |
| CO-E-002 | MED 0.4.0: Mixed-status docs keep the pre-ledger-sharing bootstrap premise | `docs/NAT_TRAVERSAL_PLAN.md:12,63` and `docs/GLOBAL_ROLLOUT_PLAN.md:21` still frame bootstrap nodes as current design | **FIXED** in this PR (dated doctrine notes, T11 style) |

## CO-E-001 -- the part that matters is not the count

50 files contain the two dead AWS addresses. The count is not the finding; the
classification is:

1. **Historical evidence (leave alone).** `HANDOFF/audit/*`,
   `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_*`, `HANDOFF/CEO_STATE.md`,
   `HANDOFF/CTO_STATE.md`, the `V040_3NODE_*` records, and the freebuff inbox
   notes are dated records of what was true then. Rewriting them would destroy
   evidence to satisfy a grep. They are not "copies of a config"; they are
   history.
2. **Active docs (must not carry a dead address as current).**
   `HANDOFF/gpt/AWS_RELAY_CURRENT_ADDRESS.md` names `18.234.62.247` under
   "## Current", with its own standing policy that the address is ephemeral and
   this file is the one place the orchestrator updates after a rebuild. Its
   "Current" section is therefore only as fresh as its last update and must be
   re-verified at use time. Active tickets that cite an address:
   `V040_T6_TIER_A_CONFORMANCE_HARNESS.md`,
   `V040_T_LEDGER_IP_CHURN_AUTONOMOUS.md`, `P1_CLI_SEND_CANONICAL_IDENTIFIER_PARITY_2026-09-16.md`,
   `todo/CELL_ROUTE_AWS_001_2026-09-11.md`,
   `V040_3NODE_LOG_ANALYSIS_2026-09-20.md`.
3. **Code.** Two production surfaces, both worth a decision rather than a
   sweep:
   - `core/src/transport/swarm.rs` -- four hits, **all inside `#[cfg(test)]`
     blocks** (lines 11196, 11896, 11899, 11945, read at those lines this
     session). Test literals; consistent with the audit's own CO-E-003
     verification that core/cli carry no routable IPs in production code.
   - `android/.../data/MeshRepository.kt:11142` -- **production logic**, a
     connectivity heuristic:

     ```kotlin
     return peerCount > 0 &&
         (host.startsWith("192.168.") || host.startsWith("10.") || host == "18.234.62.247")
     ```

     This is a latent defect independent of doctrine: the AWS address is
     ephemeral by policy, so the third clause silently stops matching after the
     next instance rebuild, and the heuristic then reports "not connected" for a
     host it used to accept. Two candidate fixes, both behaviour changes on the
     Pixel's connectivity surface: (a) drop the literal and rely on the known
     node addresses the app already holds (bootstrap/ledger), or (b) express it
     as "any non-LAN host we have a live peer on". Flagged, not changed: this is
     Android production behaviour right before a tag, and choosing between the
     two is a product call.

## Deliberately not done, and why

- **`CO-G-001` (queue index).** 50 files versus 12 referenced. The fix is an
  explicit "historical, not dispatchable" index section, not deletion -- several
  of those files are referenced by inbox notes and PR bodies. It needs one pass
  with the paste-set owner, not a bulk move.
- **The `relay server` noun in Rust doc comments.** A second class, separate
  from the operator-facing copy: `core/src/relay/mod.rs:3` ("Every node with
  internet connectivity is a relay server"), `core/src/relay/client.rs:163`,
  `core/src/relay/server.rs:56,101,114,119`,
  `core/src/wasm_support/transport.rs:3`, `wasm/src/mesh.rs:186`,
  `wasm/src/transport.rs:38,133,1167,1254`. These describe a `RelayServer` type
  that exists, so the technical-name exemption is arguable; the role-noun
  reading is not. One pass, after the operator's naming call -- not smuggled
  into a string-fix PR.

## Verification of this PR's own edits

- `bash scripts/docs_sync_check.sh` -> `docs-sync-check: PASS`
- `python scripts/rules_check.py` -> exit 0
- No test pins the old diagnostic copy:
  `grep -rn "relay servers may be down\|All relay servers unreachable\|relay(s) circuit-broken\|relay(s) in half-open" android/` -> no output
