# V0.5.0+ Philosophy and Borrow Plan (Reticulum-derived)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Active (canonical post-v0.4.0 queue for Reticulum-derived work)
Created: 2026-09-21
Basis: `HANDOFF/plans/RETICULUM_AUDIT_2026-09-21.md` (read it first; every item
below cites its audit element)

---

## Governing rule

> Adopt Reticulum/LXMF **designs and philosophy only. Adopt zero RNS/LXMF
> code.** No wire-compatibility claims, no "RNS-like" self-description, no
> crypto downgrades. RNS is a network stack; SCMessenger is a messenger
> product -- borrowing happens at the design-note level, per element, behind
> the existing gates (`core/src/{crypto,transport,routing,privacy}`
> adversarial review, operator escalation on architecture).

## Sequencing rule

Nothing here starts before the v0.4.0 tag. The v0.4.0 exit criteria (SHIP_PLAN
D1-D7) outrank every item on this page. This plan exists so deferred work is
anchored and nothing is lost -- it is NOT a competing queue. When a workstream
below starts, it graduates into a proper HANDOFF ticket that cites back here.

## Deferred queue, in order

### v0.5.0 wave (docs + small gated code)

| ID | Item | Audit element | Gate / lane | Done when |
|---|---|---|---|---|
| V050-P1 | README/threat-model prose adoptions: "routes to a person, not a place" and "encryption is the substrate, not a feature" worked into the README threat-model section | 1.2, 1.13 | Docs only; no D1-D7 impact | README carries both lines, faithful to its existing honesty register |
| V050-P2 | `scm status` / `scm probe <peer>` diagnostics commands in the CLI (rnstatus/rnprobe posture: interface state, path/peer reachability) | 1.12 | `cli/` only, no core gates | Commands exist, output printed in full (no truncation), used in the next fleet run |
| V050-P3 | Gossip governance for ledger sync over constrained transports: per-interface rate caps, dedup table, priority queues for low-bandwidth (BLE) peers | 1.3 | **[AUDIT-GATE]** `core/src/transport/` | Design note + implementation + adversarial review on file |
| V050-P4 | Publish session-establishment byte budget and idle keepalive cost as official doc metrics (target metric style of RNS's "3 packets / 297 bytes / 0.45 bps") | 1.5 | Docs + measurement harness | Numbers measured, not estimated; published in docs |

### v0.5.0 / v1.0 wave (design notes first)

| ID | Item | Audit element | Gate / lane | Done when |
|---|---|---|---|---|
| V050-C1 | Custody-store peering/sync design note: study LXMF propagation-node protocol (FIRST ACTION: read lxmf router source; docs-only claims are insufficient per audit limits), then adapt to nodes-not-relays -- every node relays, custody stores peer and sync, recipients retrieve from any synced node | 1.7 + Correction 2 | **[AUDIT-GATE]** `core/src/store/`, `core/src/drift/`; operator escalation on architecture (rule 9) | Design note on file with source-read evidence; implementation ticket spawned only from the note |
| V050-Q1 | QR paper-message export/import (`scm qr-export` / `scm qr-import`): encrypted message encoded as QR for dead-zone sneakernet handoff, delivered when re-imported | 1.9 | `cli/` + envelope compaction pass | Round-trip demo: export -> scan/copy -> import -> delivered |

### v1.1+ research lane (design study, no commitments)

| ID | Item | Audit element | Gate / lane | Done when |
|---|---|---|---|---|
| V050-R1 | Envelope v3 metadata-minimization wish list: no source addressing on packets; review of on-wire `device_id` in the identity envelope sender block (`core/src/message/identity_envelope.rs:29`); sender-metadata review | 1.4 + Correction 1 | **[AUDIT-GATE]** `core/src/{crypto,drift,message}`; cross-platform (4 clients); operator escalation | Wish list written; scope decision escalated to operator |
| V050-R2 | LoRa / serial-pipe transport evaluation spike (RNode-class hardware via serial pipe; 5 bps class links) | 1.10 | **[AUDIT-GATE]** `core/src/transport/`; hardware procurement is operator decision | Go/no-go spike report with hardware cost and byte-budget math |
| V050-R3 | Initiator anonymity on session setup via the parked onion path (`core/src/privacy/`) | 1.5 | **[AUDIT-GATE]** `core/src/privacy/`; the parked path is already flagged unwired | Design note; wiring decision per rule 16 (wire it or it is dead) |

## Deferral integrity

- Every item cites its audit element; the audit file is the reasoning record,
  this file is the sequencing record.
- Nothing here may open a `HANDOFF/todo/` ticket before the v0.4.0 tag (S0-4
  amnesty keeps todo at <= 10 files); graduation into a ticket happens per the
  sequencing rule above.
- Items V050-R1..R3 are explicitly NOT commitments; they are research lanes
  pending operator direction (rule 10: do not relitigate settled scope).

## Explicitly rejected (do not re-propose without new evidence)

- Adopting the RNS Python stack or a Rust reimplementation chasing wire
  compatibility (audit 1.1; Brandolini's Reference warning).
- Swapping libp2p for RNS path machinery.
- Downgrading crypto (X25519/AES-CBC class) to gain RNS-style anonymity.
- Network Identity administrative keys / privileged admin domains (audit 1.11).
- Any anonymous packet forwarder (forbidden by AGENTS.md doctrine).
- Any interop or compatibility claim toward Reticulum; "inspired by" only.
