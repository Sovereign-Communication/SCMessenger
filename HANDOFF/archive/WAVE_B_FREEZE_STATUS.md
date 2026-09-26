# Wave B -- PQC Depth -- FREEZE STATUS

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: FROZEN. Last updated: 2026-07-17.

Reason: All Wave B items depend on E-01c (PQ root-key mixing implementation).
E-01c may not be dispatched until E-01b has an adversarial PASS on file.
E-01b may not be dispatched until the operator reviews the E-01a constraints
document at HANDOFF/review/E01a_attempt_constraints.md.

## Frozen Items

| ID   | Ticket / Description                  | Freeze Reason                                     | Unfreeze Condition                          |
|------|---------------------------------------|---------------------------------------------------|---------------------------------------------|
| B-01 | PQC-04 suite negotiation              | Verify current state first (was DONE 2026-07-10, confirm) | E-01c committed OR verified already done  |
| B-02 | PQC-09 hybrid onion                   | DOUBLE FROZEN: E-01c + AD-8 onion seam freeze     | E-01c committed AND explicit AD-8 operator lift |
| B-03 | PQC-10 ML-DSA identity signatures     | E-01c not landed                                  | E-01c committed                             |
| B-04 | PQC-11 relay/invite hybrid dual-sig   | E-01c not landed (standing rule: PQC-11 frozen until E-01 lands) | E-01c committed               |
| B-05 | PQC-12 TLS PQ groups                  | B-04 not landed                                   | B-04 committed                              |
| B-06 | PQC-13 verification suite             | E-01c not landed (standing rule: PQC-13 frozen until E-01 lands) | E-01c committed               |
| B-07 | PQC-14 docs + risk-register           | B-06 not landed                                   | B-06 committed                              |

## Dispatch Order on Unfreeze

1. B-01 -- verify first; may already be done (was PQC-04, completed 2026-07-10)
2. B-03 -- PQC-10 ML-DSA identity signatures
3. B-04 -- PQC-11 relay/invite hybrid auth
4. B-05 -- PQC-12 TLS PQ groups
5. B-06 -- PQC-13 verification suite (kani proofs, proptest cross-version)
6. B-07 -- PQC-14 docs and risk-register closure
7. B-02 -- PQC-09 hybrid onion (dispatch only after explicit AD-8 operator lift)

## Standing PQC Rules (must be respected by all workers on every B-wave task)

- Hybrid never pure: every PQC primitive retains classical fallback.
- Never remove legacy decrypt/verify: old format must remain decodable.
- Bincode format-tag discipline: every serialized type must carry a version tag.
- All crypto/ diffs carry REVIEW: adversarial (mandatory before close).
- All transport/ diffs carry REVIEW: crypto-security-auditor (mandatory).
