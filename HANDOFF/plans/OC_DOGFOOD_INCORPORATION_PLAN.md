# OpenClaw Dogfood Incorporation — Canonical Plan

Status: Active
Created: 2026-09-22
Operator directive: "OC is WIP... be on the lookout for D9 as well next. You
can verify the session locally github/OC/... Ensure we pick all of this up
and audit/incorporate/update/merge as needed to drive closer to 0.4.0."

Source of record: the OC session's own evidence files (read in full this
session): `~/Documents/GitHub/OC/SCM_NODES_AUDIT.md` (2026-09-22 04:20Z, the
per-node audit + defect ledger), `NODE.md` (node + deploy state),
`POSTMORTEM.md` (2026-09-21 instance loss), `PLAN.md` (provisioning plan).
The OC directory is not a git repo; its value is evidence and state, not
commits. Its terminology predates `docs/rules/NODE_MODEL.md` — read its
"relay"/"bridge" node language per that rule: two full nodes, different
purposes.

## What the OC session actually did (verified from its records)

1. Deployed an OpenClaw gateway (AWS `i-01df07a1b747b99a8`) dogfooding
   SCMessenger + Sovereign-Harness; recovered from the 2026-09-21 OOM crash +
   unauthorized-termination incident (postmortem on file).
2. Ran a live 3-device mesh: two cloud nodes + operator phone (Dx) + Windows
   driver (D6vZ), with custody verified end to end.
3. Diagnosed D1 (per-peer connection cap refusing the phone 169 times),
   implemented the two-tier fix **in this checkout** (uncommitted at the
   time), cross-built, deployed to both nodes with staged rollbacks, and
   proved it with an induced A/B: old build 9 denials; fixed build 0
   denials / 8 trims, repeated twice.
4. Ran the Harness-MCP dogfood loop on real SCM logs (`log_judgment` +
   `issue_sort`, ledger seq 230-234, cost $0.00) — the tooling's buckets
   matched the manual analysis.

## Incorporation status

| OC finding | Disposition | Where |
|---|---|---|
| D1 per-peer cap lockout | **Fix committed this session** (V040-T-CONN-04, two-tier policy + LRA trim + 9 unit tests, live A/B evidence on file). Rule-8 review still required before merge to main. | `core/src/transport/{per_peer_cap.rs,behaviour.rs,swarm.rs}`, harness `cli/src/bin/conn-fanout.rs` |
| D2 no durable relay reconnect (`cli/src/seed_dial.rs::sweep_decision` returns WatchOnly whenever `peer_count > 0`) | **Ticket filed — HIGH** | `HANDOFF/todo/D2_SEED_DIAL_REDIAL_MISSING_SEED_PEER.md` |
| D3 self-addressed poison message loops (16 self-dial failures + node marks itself dead; no cancel route) | **Ticket filed — MEDIUM** | `HANDOFF/todo/D3_SELF_RECIPIENT_POISON_GUARD.md` |
| D5/D6 relay DHT bootstrap empty + `bootstrap_nodes` env-only | **Ticket filed — LOW-MEDIUM, config-first** | `HANDOFF/todo/D5_D6_BOOTSTRAP_NODES_PERSISTENCE.md` |
| D8 custody-delivery sender status stays `delivered: false` | **Ticket filed — UNVERIFIED, evidence needed with Dx online** | `HANDOFF/todo/D8_CUSTODY_DELIVERY_RECEIPT_STATUS.md` |
| D9 libp2p 0.48 either-handler task panic | **Ticket filed — HIGH, next up per operator** | `HANDOFF/todo/D9_LIBP2P_EITHER_HANDLER_PANIC.md` |
| Version skew between the two cloud nodes | RESOLVED by OC (both on `v040tconn04-lru` artifact); future alignment goes through CI artifacts per the CI-primary doctrine | `docs/runbooks/CI_PRIMARY_BUILD.md` |
| Contact-derive artifact (bogus `Lucas-Pixel` rows, no DELETE route) | Tracked as D3 adjacent cleanup; needs a contacts-remove route | inside D3 ticket |
| Ops guardrails (snapshot before destructive ops, never terminate without operator approval, one-command redeploy) | OC-internal, recorded in its POSTMORTEM; no repo change required | OC NODE.md / POSTMORTEM.md |

## What this drives toward 0.4.0

- D1's fix removes a proven field blocker on the phone attach path — the
  exact path SHIP_PLAN D4/D7 (message + receipt, offline proximity) depends
  on. It needs: Rule-8 review -> merge -> CI artifact -> redeploy of both
  cloud nodes from CI-built bytes (closing the uncommitted-artifact gap).
- D9 is the next blocker-class defect on the connect path; operator named it
  next.
- D2 is the durable-reconnection guarantee the always-on node exists to
  provide; HIGH, right behind D9.

## Provenance and verification honesty

The deployed V040-T-CONN-04 binary was built from the uncommitted tree
(sha256 `1dd6029d...`, version string `v040tconn04-lru-d2816d4d-worktree`) —
acceptable as field emergency response, and now superseded: the source is
committed on `glm/canonical-outlier-audit` and CI is the verifier going
forward (`docs/runbooks/CI_PRIMARY_BUILD.md`). The A/B evidence came from the
induced harness, not yet from the phone itself; the phone's next attach
closes that gap (OC audit section 9, "Honest limit").
