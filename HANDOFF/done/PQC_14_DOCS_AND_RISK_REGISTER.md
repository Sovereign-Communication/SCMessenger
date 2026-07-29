# TASK: PQC-14 — Documentation, protocol spec, and risk register closure

Read `PQC_00_MASTER_PLAN.md` first. Depends on: all other PQC tasks. Wave 5. Min tier: Haiku.

## Deliverables

1. **New protocol spec** `docs/PQC_HYBRID_PROTOCOL.md` documenting AS IMPLEMENTED (read the landed code, do not copy task-file intentions that drifted): suite registry, EnvelopeV2 wire layout + tag bytes, PublicKeyBundle formats and tags, hybrid combiner construction + KDF contexts (exact strings), session init flow, PQ ratchet step rules, negotiation + transcript binding, onion layer v2, dual-signature rules, all format tag values in one table. Include `Status:` / `Last updated:` headers (docs-sync format).
2. **Update `docs/CURRENT_STATE.md`**: crypto section reflects hybrid suite; note which surfaces remain classical (per-envelope Ed25519 relay signatures, libp2p Noise) and why.
3. **Update `docs/QUANTUM_READINESS_AUDIT.md`**: append a dated "Remediation status" section mapping findings F1-F6 to landed tasks and residual exposures. Do not rewrite the original audit body.
4. **Risk register**: add entries to the CURRENT release's risk register (check which register is active — `docs/V0.2.1_RESIDUAL_RISK_REGISTER.md` or newer; follow its Status semantics) for: residual classical surfaces (envelope sigs, Noise), rollout-window acceptance of single-sig bundles/invites (PQC-10/11 compat), and `require_pq` default-off. Status per its own taxonomy (likely `Accepted` with rationale).
5. **`docs/DOCUMENT_STATUS_INDEX.md`**: add rows for the audit doc and the new protocol spec.
6. **`REMAINING_WORK_TRACKING.md`**: mark the PQC workstream section complete with per-task commit hashes.

## Definition of Done

- [ ] `bash scripts/docs_sync_check.sh` PASS (or `.ps1` on PowerShell) — this is the primary gate for this task.
- [ ] No emojis introduced anywhere.
- [ ] Machine-local paths absent (docs-sync enforces).
- [ ] Every format tag value and KDF context string in the spec verified against the code by grep, with the grep commands pasted into this file.
- [ ] File moved to HANDOFF/done/ + committed.

## Do NOT

- Update CLAUDE.md's release-line references or other unrelated stale content in the same commit (separate housekeeping).
- Document intended-but-not-landed behavior; the spec describes shipped code only.
