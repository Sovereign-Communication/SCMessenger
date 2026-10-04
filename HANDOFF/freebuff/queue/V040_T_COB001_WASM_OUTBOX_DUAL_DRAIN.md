# V040-T-COB001 — Dual-drain IronCore outbox flush (wasm + all callers)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: OPEN (filed 2026-09-20 CTO; from CANONICAL_OUTLIER_AUDIT CO-B-001)
Priority: P1-HIGH — 0.4.0-target; message strand class
Lane: Freebuff
Scope: `core/src/iron_core.rs` `flush_outbox_for_peer` (+ tests).
Do **not** rewrite CLI dual-drain (already correct). Do not change enqueue key format.

## Finding (audit iter2)

`wasm/src/lib.rs` flushes with libp2p `PeerId.to_string()` (base58).
Enqueue keys are canonical hex. `IronCore::flush_outbox_for_peer` currently:

```
self.outbox.write().drain_for_peer(peer_id)
```

Single-form drain cannot match hex-keyed queue entries. CLI already dual-drains
(`cli/src/main.rs`): drain base58 form, then if
`extract_ed25519_public_key_from_peer_id` succeeds, drain hex form too.

## Implement (do not re-derive)

Mirror CLI inside core so **every** caller benefits:

1. Read `cli/src/main.rs` dual-drain block (command evidence on main).
2. Change `IronCore::flush_outbox_for_peer` to:
   - drain `peer_id` as given
   - if peer_id looks like base58 libp2p, extract ed25519 pk → hex, drain that too
   - if peer_id is already 64-hex, also attempt the reverse only if a known
     mapping exists — **do not invent** a second hex form
3. Prefer reusing existing core helpers (same as CLI) — no new crypto.
4. Unit tests: queue under hex key; flush with base58 → messages drained;
   queue under base58; flush with hex when extractable → drained.

## Scope correction

- CLI path is already fixed — do not “unify” it away.
- Wasm in-memory outbox is process-scoped; this still matters for wasm runtime
  and for any non-CLI core consumer.
- Related ticket `P1_CLI_OUTBOX_CANONICAL_DRAIN_AND_SLED_UNIFICATION` —
  orchestrator will re-scope/close after this lands (CO-G-002).

## Acceptance

1. `cargo test -p scmessenger-core` green.
2. New tests cover both key forms.
3. `git grep -n "fn flush_outbox_for_peer" core cli wasm` shows core dual-drain.
4. PR evidence: commands + outputs.

## Review gate

Core store path — orchestrator review required. **Harness verify** if the
implementation deviates from the CLI-extracted pattern (operator rule: <99%
confidence → harness). Straight port of CLI pattern: orchestrator + tests may
suffice at 99%+ after reading both sites.

## Rules

No emojis. Evidence contract. Worktree isolation. No self-merge.
