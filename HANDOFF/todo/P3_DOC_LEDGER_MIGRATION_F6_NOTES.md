# P3 — Doc line: ledger migration doc contradicts archiving behavior (Opus review F6)

- **Priority:** P3 (doc-only, [INFO] finding from the recorded rule-8 review)
- **Filed:** 2026-09-13, Buffy (Freebuff recovery session)
- **Origin:** F6 of `HANDOFF/freebuff/inbox/RULE8_PR262_PR263_VERDICT_OPUS.md`
  (PR #262/#263 adversarial review); surfaced while verifying closure of every
  follow-up from that verdict (see
  `HANDOFF/audit/T1_T2_CENSUS_DISPOSITION_2026-09-13.md`, ADDENDUM item 2).
- **Status:** RESOLVED 2026-09-13 -- fix on `fix/f6-ledger-migration-doc-20260913`,
  gate `bod-3b8d3ffe` APPROVED + Blind B
  (`HANDOFF/review/V040_F6_BLIND_B_ADVERSARIAL_VERDICT_2026-09-13.md`);
  closes on merge. Verified by two lanes: the assist-lane audit
  (`HANDOFF/audit/BACKUP_BRANCH_MERGE_MAP_2026-09-13.md`) drafted the same
  diff independently.

## Defect (code-verified on main `5f1cf702`, 2026-09-13)

`core/src/store/ledger_entry.rs:2240` (doc comment on the legacy-migration
result) says:

> `peers.json` is left in place; the caller simply stops writing it.

But the actual caller behavior is a rename/archive:
`cli/src/ledger.rs:720` (`archive_legacy_peers_json`). The reviewer's original
citation was `cli/src/ledger.rs:253`; the function now lives at `:720`.

## Why it matters

The mismatch is not cosmetic to every future reviewer: the doc line is what a
migration auditor reads first. "Left in place" implies the legacy file remains
a live input that could be re-read; the truth (renamed with a suffix, no
longer read or written) is a stronger and security-relevant property — the
poisoned 4,678-entry input is preserved for replay but is inert.

## Fix (one line)

Rewrite the doc line to state the archive behavior and name the function,
e.g.: "`peers.json` is archived by the caller (`archive_legacy_peers_json` in
`cli/src/ledger.rs`) — preserved for replay, never read again."

Also check whether the sibling doc sites (the module-level migration doc at
`cli/src/ledger.rs:1-30`) repeat the "left in place" phrasing.

## Acceptance

- The doc line matches observable behavior (rename + no further reads).
- A reviewer grepping "left in place" finds no contradicting claim.
