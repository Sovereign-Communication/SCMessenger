# Naming policy

The repository has one naming policy and it lives in exactly one place:

    naming_policy.json        the single source of truth
    scripts/check_naming.py   the only enforcement point

This file deliberately does **not** restate the table. It points at it. That is
a decision taken from the audit's own lesson: `naming-audit/GLOSSARY.md` records
that an earlier version of the policy "was recorded nowhere", that fifteen
corpus definitions were tried against its published figures and none
reproduced them, and that a definition copied into a second place "is a
definition that will be wrong in two places". A prose copy of the policy would
be the second place, and it would be wrong within one release.

## What is enforced, and what is not

Every rule in `naming_policy.json` carries a `status` taken from
`naming-audit/GLOSSARY.md` and a `tier`:

| tier | effect | which statuses may use it |
|---|---|---|
| `block` | prints `[FAIL]`, exits non-zero, blocks the commit and the merge | `FROZEN`, `ACCEPTED-AS-IS`, `DECIDED-BY-DOCTRINE` |
| `warn` | prints `[WARNING]`, never changes the exit code | anything, including `PROPOSED` and `BLOCKED` |

The gate **refuses to load** a policy in which a `block` rule carries a status
outside the first row. This is the load-bearing constraint of the whole
mechanism: most of the glossary is `PROPOSED` or `BLOCKED` on an open question,
and an unratified recommendation must never be able to block someone's commit.
Promoting one to `block` is therefore a two-key edit to `naming_policy.json` —
a visible policy change, in its own commit, with the glossary entry updated
first. The gate's self-test asserts this refusal, so weakening it breaks CI.

### A ratified concept is not a ratified remedy

That check alone is not enough, because one rule routinely bundles terms the
audit rated differently. Every denied term therefore also carries the action
the audit proposed for it — `rename`, `delete`, or `keep-as-is` — and the
status the audit gave **that action**, in the rule's `remedies` list. A term
produces `[FAIL]` only when the rule's status *and* that term's remedy status
are both ratified. Otherwise it is still scanned and still printed, but as
`[WARNING]` with the reason attached.

Two terms are in that state today:

| term | rule | action | status | why |
|---|---|---|---|---|
| `isRelay` | A-2 | `delete` | `PROPOSED` | GLOSSARY A-2 rates the vocabulary `DECIDED-BY-DOCTRINE` but says "the `isRelay` deletion is `PROPOSED`". |
| `StoredMessage` | A-3 | `delete` | `PROPOSED` | A-3's own inventory says the name "is not free" because F-02 offers it as an alternative, and "A-3's decision below deliberately does not" take it up. F-02 and F-33 are findings carrying a severity, not decisions. |

Every denied term must be classified. A term with no remedy, a remedy for a
term the rule does not deny, an action outside the three above, and a `block`
rule whose every remedy is unratified are all **policy errors that refuse to
load**. Without those guards the two lists drift apart and an unratified
remedy quietly inherits the concept's status again — which is the defect this
mechanism exists to close.

The point is not politeness toward unratified findings. A gate that blocks on
something the audit never decided gets bypassed, and a bypassed gate is worse
than no gate.

`legitimate` entries record terms the audit deliberately accepted
(`mycorrhizal`, `triad`, `lastSeenMs`, the Q-3-blocked `routePeerId` family).
They exist so that a later consistency sweep does not "correct" a name that was
considered and kept. They are never reported.

## The gate is a ratchet

It judges only the lines a diff **adds**. The tree already contains 57
`isRelay`, 38 `delete_*` and 165 `lastSeen`; a gate that counted them would be
red on the day it landed, which is how gates come to be bypassed. A ratchet
starts green and tightens. It never re-litigates history a human already
reviewed.

The cost is stated rather than hidden: **pre-existing violations are not
reported by this gate.** They are the audit's backlog — F-13 and the rest. To
count them, use the instrument the audit already shipped:

    python scripts/measure_uncompiled_counts.py

## Where it runs

| where | invocation | what it judges |
|---|---|---|
| pre-commit | `.githooks/pre-commit` -> `scripts/rules_check.py --staged` | lines the index adds |
| CI | the `Naming policy` job | lines the branch adds since the base commit |
| CI | the `Prove the gate can still fail` step | the gate's own self-test |
| manually | `python scripts/check_naming.py --repo-root . --self-test` | the self-test |

`scripts/rules_check.py` runs the naming check in **staged mode only**. An
explicit file list carries no diff to take a ratchet against, so the only
alternative is a whole-file scan, which would fail on the backlog above and
make the gate useless on the path it exists to protect.

## Changing the policy

1. Decide the name in `naming-audit/GLOSSARY.md` first. The policy follows the
   glossary; it does not lead it.
2. Edit `naming_policy.json` **in its own commit**, naming the glossary entry
   in the rule's `audit_ref`.
3. Classify every denied term in `remedies`, with the action the glossary
   proposes and the status the glossary gives **that action**. If the glossary
   does not settle it, the status is not one of `enforceable_statuses` and the
   term warns instead of blocking. Leave it out and the policy will not load.
4. Run `python scripts/check_naming.py --repo-root . --self-test`. Add a case
   for the new rule: one line it must reject, and — where the term is close to
   one already permitted — one it must not.

Do not add a rule to make a particular commit pass. That is the failure mode
this file exists to prevent, and it is why the refusal in the first section is
enforced by the loader rather than by review.

## Exit codes

| code | meaning |
|---|---|
| 0 | clean (`[WARNING]` lines may still have printed) |
| 1 | at least one `[FAIL]` |
| 2 | `BLOCKED` — policy unreadable or invalid, not a git repository, or run against a repository other than the one containing it |

Exit 2 is deliberately distinct from 0. A gate that cannot see what it is
supposed to judge must not read as "nothing to check".
