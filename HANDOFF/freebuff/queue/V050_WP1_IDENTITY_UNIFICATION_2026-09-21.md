# V050-WP1 — Identity unification (canonical hex) — residual implementation

Status: OPEN (filed 2026-09-21 CTO)
Priority: P0 — WiFi delivery umbrella WP1
Lane: Freebuff / unmetered
Authority: `HANDOFF/V040_IMPLEMENTATION_PLAN_WIFI_IDENTITY_2026-09-21.md` §2 WP1
Scope: `core/src/contacts_bridge.rs`, `core/src/store/contacts.rs` as needed,
CLI send regression tests. Do **not** re-implement CRYPTO-01 delegate binding
or WASM subscribe (already on main — tests only).

## Premise (verified 2026-09-21 on origin/main)

- CLI send already resolves hex + base58 (`peer_id_from_contact_identifier`).
- Contact recovery uses `placeholder_or_derived_contact` — derives hex when
  possible; placeholder + note when not.
- Dual-drain outbox flush is on main (#339).

## Implement

1. **WP1.1** Recovery: never treat PeerId as `public_key` for encryption.
   Placeholder contacts must not be encryptable send targets.
2. **WP1.2** Grep production `Contact::new` — no `Contact::new(peer_id, peer_id)`.
3. **WP1.3** Tests: hex send, base58 send, name send, recovery restart.
4. **WP1.4** Regression tests for CRYPTO-01 + WASM own-topic (assert current
   main behaviour; fail if someone reverts binding).

## Acceptance

- [ ] Unit tests green (`cargo test -p scmessenger-core` relevant modules)
- [ ] Greps in implementation plan §2 WP1 all pass
- [ ] Harness JEV pack §3.3 `is_passing` **or** UNVERIFIED-JEV + mechanical evidence
- [ ] PR evidence: commands + outputs

## Review gate

Core store/contacts — orchestrator review. Rule-8 if crypto path touched.

## Rules

No emojis. Evidence contract. Worktree. No self-merge. No new root-cause plans.
