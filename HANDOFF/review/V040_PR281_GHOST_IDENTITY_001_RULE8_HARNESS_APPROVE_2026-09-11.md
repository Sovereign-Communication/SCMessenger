# Rule-8 harness verdict — PR #281 GHOST-IDENTITY-001 (b258f1db..fe6f895f)

Date: 2026-09-11T06:22Z
Reviewer: sovereign-harness free-tier panel (3 models) + judge
Cost: **$0.00** (paid escalation permitted to $0.10; not needed)
Evidence: `Harness/audits/scmessenger/_runs/seat-gates/verify_20260911T062239Z.json`
Packet: `Harness/audits/scmessenger/_runs/rule8-281/prompt_selfcontained.txt`
Scope: core/src/transport/swarm.rs + Android GhostIdentityGate/TopicManager/seed accessors/loadPeers

## Verdict

**APPROVE** — agreement **high**, confidence **0.98**, defer=false

| Claim | Panel | Reason (condensed) |
|---|---|---|
| C1 fail-closed 64-hex ghost topic | **real** 3/3 | hex64 + no proven ledger → ghost; core missing → true |
| C2 skip ghost / keep proven+mesh-wide | **real** 3/3 | if-else skip; live 30d0fa67 still subscribes |
| C3 577fd171 shape is ghost | **real** 3/3 | success=0, fail>=3, 64-hex pk-as-peer_id |
| C4 TopicManager/seed/loadPeers wired | **real** 3/3 | 0 post-gate auto-sub; loadPeers 2 online, 0×577fd |
| C5 custody/ACK/proven-relay untouched | **real** 3/3 | diff is subscription/discovery only; ACKs observed |

Judge synthesis: APPROVE, no disagreements.

## Residual (non-blocking)

- Fail-closed `is_ghost_peer_topic==true` while `core_handle` is down could skip a *legitimate* new 64-hex identity topic for a short window at startup (panel note).
- G1 ledger row prune still OPEN (UI/topic gated; row remains).

## Standing-rule note

Precedent: prior #272 resolution used harness free-lane panel as rule-8 evidence
(`V040_CANDIDATE_272_FINAL_APPROVE_DELTA_e97c3f82_HARNESS_*.md`). This packet
follows the same pattern. Operator may still require a second non-harness
reviewer before merge to main — that is **not** claimed here.

## Cell-store-forward addendum (same pull)

Phone WIFI→CELLULAR 06:13:36Z → WIFI 06:16:12Z.

| Msg | During cell | After WiFi return |
|---|---|---|
| `22c5beda` "cell only te…" | pending | **received on Windows** |
| `b87c17b3` "cell test 2" | pending | **received on Windows** |
| `73232018` | pending flush | **AWS inbox_receive + delivery ACK** |
| `5f4e7a53` | pending flush | **AWS inbox_receive + delivery ACK** |

Store-and-forward via AWS during cell: **PASS**. peersDiscovered fluctuates 1–2
after reconnect (observed; not treated as a new defect this pass).
