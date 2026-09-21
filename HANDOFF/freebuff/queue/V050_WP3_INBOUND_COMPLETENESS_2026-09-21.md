# V050-WP3 — Inbound completeness (own-topic + authenticated sender)

Status: OPEN (filed 2026-09-21 CTO)
Priority: P0 — umbrella WP3
Lane: Freebuff
Authority: `HANDOFF/V040_IMPLEMENTATION_PLAN_WIFI_IDENTITY_2026-09-21.md` §2 WP3
Scope: tests first; production only if a gap is proven on current main.

## Premise (verified 2026-09-21)

- WASM own-topic + `is_ghost_peer_topic` **present** on main.
- Delegate path constructs authenticated `sender_public_key_hex` / canonical id.

## Implement

1. Regression tests that **fail** if WASM own-topic subscribe is removed.
2. Ghost-guard: own topic never dropped when own key is known.
3. Grep all receive paths — none pass payload-only `sender_id` as
   `sender_public_key_hex`.
4. Fill any proven gap; do not re-theorize.

## Acceptance

- [ ] Mechanical gates green (tests / greps / Rule-8 as applicable)
- [ ] python scripts/jev_canonical_check.py --wp WP3 --state-file <state.json> exit 0
      (
esult.is_passing(0.70) on keyed JEV; **UNVERIFIED-JEV / fallback is NOT DONE**)
- [ ] PR evidence: commands + outputs (include JEV answers/cost from the check)

- [ ] Tests green
- [ ] Rule-8 if `core/src/transport` changes
- [ ] Ticket P1_CORE_IDENTITY_SPOOF / P1_GHOST_GUARD updated with merge cites

## Review gate

Rule-8 when transport/core dispatch changes.

## Rules

No emojis. Evidence contract. Worktree. No self-merge.
