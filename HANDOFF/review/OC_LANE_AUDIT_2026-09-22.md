# OC Lane Audit -- OpenClaw Dogfood Session (approval verdict)

Status: APPROVED, with conditions
Audited by: Buffy (Freebuff lane, Windows host), 2026-09-22
Session: OpenClaw gateway on AWS `i-01df07a1b747b99a8`, dogfooding
SCMessenger + Sovereign-Harness; operator-classified as a Freebuff-lane WIP
worker ("the vendor folder is the WIP OC -- audit that session... ensure
it's approved per repo rules").
Evidence base: the session's own records read in full this session
(`~/Documents/GitHub/OC/{SCM_NODES_AUDIT.md,NODE.md,POSTMORTEM.md,PLAN.md}`),
plus first-hand verification of its code artifacts in this checkout.

## Verdict: APPROVED, with conditions

The session's engineering is verified sound and its provenance is now
closed. The conditions below are requirements it must meet going forward,
not defects in what landed.

## What was audited, first-hand

1. **The D1 fix (V040-T-CONN-04).** Diff read in full this session against
   the session's own claims: new `core/src/transport/per_peer_cap.rs`
   (two-tier policy, deterministic LRA selection, 9 unit tests including the
   field-lockout regression test), `behaviour.rs` admission ceiling 16 with
   a no-collapse comment, `swarm.rs` retained-bound trim with
   `[CONN-CAP]` logging and `ConnectionClosed` bookkeeping. Commit as
   `7810845b` on `glm/canonical-outlier-audit`, open as PR #359.
2. **The fan-out harness** `cli/src/bin/conn-fanout.rs` -- test-only, not in
   production builds; verified by reading it.
3. **The vendor/ research artifact** -- `vendor/libp2p-swarm-0.48.0/`
   verified **byte-identical to the cargo registry source** (0 differences,
   `diff -rq`), i.e. a pinned-source reading copy for the D9 root-cause
   hunt, NOT a patched fork. No `[patch]` section exists; the build is
   untouched. Disposition: research artifact. Keep while D9 is open (the
   pinned line-110 evidence lives there); delete after D9 closes, and never
   commit it (untracked by intent).
4. **Deployment claims** (both nodes on the same artifact, staged rollbacks,
   A/B evidence) -- internally consistent and evidenced in
   `SCM_NODES_AUDIT.md` section 9; not independently re-verified over SSH
   from this seat. Marked as claimed-with-evidence rather than re-proven.
5. **D9 root cause** -- the session's vendored source enabled pinning
   `either.rs:110` to the `_ => unreachable!()` in
   `EitherHandler::on_behaviour_event`; recorded in the D9 ticket.

## Conditions (going forward)

1. **Provenance rule (now canon):** no deployment from an uncommitted tree.
   The 2026-09-22 deployments ran for hours before the source existed in a
   commit; `docs/runbooks/CI_PRIMARY_BUILD.md` now forbids exactly that.
2. **CI-primary:** build via CI and deploy CI artifacts; local
   cross-builds (zig-pinned libdbus) are the failover, with provenance
   recorded either way.
3. **Rule-8 gate honored:** the transport fix merges to main only after an
   adversarial APPROVE from a non-author reviewer (PR #359 is gated on it).
4. **Node-model language:** OC records predate `docs/rules/NODE_MODEL.md`;
   its "relay node"/"bridge node" phrasing must be read (and future records
   written) as "two full nodes, different purposes."
5. **Secrets hygiene:** the POSTMORTEM's rotation actions (OpenRouter key,
   gateway token) remain open operator items; no secret material from the
   OC directory belongs in any repo file.
6. **Destructive-infra gate:** per its own POSTMORTEM guardrail, no
   terminate/rebuild without operator approval and a snapshot; that
   guardrail is now a condition of this approval, not just a local note.

## Approvals granted by this audit

- The D1 fix stands as implemented (PR #359), pending only the Rule-8 gate.
- The OC session is recognized as a contributing Freebuff-lane worker under
  these conditions; its incorporation plan
  (`HANDOFF/plans/OC_DOGFOOD_INCORPORATION_PLAN.md`) is the integration
  record.
