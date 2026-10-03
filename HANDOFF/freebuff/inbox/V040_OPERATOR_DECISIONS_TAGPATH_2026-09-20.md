# V040 operator decisions — FILLED 2026-09-20 (interview)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: ANSWERED — recorded from operator interview in the orchestrator session
Date: 2026-09-20
Authority: `HANDOFF/V040_WORKING_FIRST_PATH_2026-09-20.md` (rewritten to match)

This file supersedes the blank questionnaire that landed with PR #330.

---

## R1 — Release keystore / D2

**Ruling:** DEFERRED. "No tag, no secrets, just get it working first."

Do **not** re-dispatch keystore verify, secret re-set, or release rehearsal
until the working bar in the working-first path is met.

---

## R2 — SEC-03 sled / deny.toml

**Ruling:** **(b)** Start storage migration on a branch.

- Tag is not blocked on completing the migration.
- Engine swap still needs a further operator sign-off before merge to main
  (rule 9). Parallel exploration is authorized.
- Suggested candidates remain open (redb/fjall etc.); pick during branch work.

---

## R3 — AND-06 Kotlin curve math

**Ruling:** DO IT RIGHT — unification ordered.

1. **A-lite:** collapse redundant Kotlin copies (one validator path).
2. **A full:** finish UniFFI cutover to `is_valid_public_key`; delete remaining
   curve math from Android.

Not option B (dated accept and leave math in Kotlin).

---

## R4 — External crypto audit commission

**Ruling:** DEFER until mesh is reliable. Not on Wave 1 critical path.

---

## R5 — Scope / process waivers

**Rulings:**

- Reliable day-to-day mesh is the working definition (not formal D4/D6/D7 yet).
- **No scoring until it all lands.**
- Pixel UI: **operator always drives** the phone. Orchestrator may `adb install -r`
  anytime; otherwise passive logs only.
- Orchestrator may redeploy Windows/AWS from CI artifacts when appropriate.
- Beach-join **Phase 0-1 pulled forward** into Wave 1 (operator multi-select).
- AWS IP-churn autonomous rediscovery **in Wave 1**.

---

## R6 — Open PR batch

**Rulings:**

- **#325** — merge when green.
- **#322** — merge when green after check diagnosis.
- **#215** — already closed as superseded.
- Other legacy/dependabot/Apple PRs — post-wave disposition.

---

## Post-wave evidence bar (operator)

Harness (`tier_a_conformance.sh`) + Windows/AWS log slices + **one operator
phone session**. Then the mesh may be called working.

---

Recording complete. Orchestrator rewrote `HANDOFF/V040_WORKING_FIRST_PATH_2026-09-20.md`
and freebuff README DISPATCHABLE set to match.

---

## R7 — OpenRouter Jev fallback (2026-09-21)

**Ruling:** TypeSafe Jev ISE → use OpenRouter model `~typesafe/jev-latest`
via **`https://openrouter.ai/api/alpha/decisions`** (not chat/completions).

**Implemented (consumer):** `scripts/local_harness.py`
`evaluate_jev_with_openrouter_fallback`; `jev_canonical_check.py` uses it
unless `--no-openrouter`.

**Operator action required (OpenRouter account):** allow provider
**`typesafe`** for `~typesafe/jev-latest`. Current key already exists
(~/.config/harness/jev.env / openrouter env); without typesafe in
allowed-providers, OpenRouter returns provider-not-allowed and canonical
checks stay on TypeSafe only (or UNVERIFIED if TypeSafe is down).

**DONE rule unchanged:** keyed non-fallback `is_passing` (TypeSafe **or**
OpenRouter decisions parse) + mechanical evidence. Structural fallback is
never DONE.
