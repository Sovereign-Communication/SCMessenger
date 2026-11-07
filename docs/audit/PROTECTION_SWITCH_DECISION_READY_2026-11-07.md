# Protection switch - DECISION-READY (2026-11-07)

Filed under the mission's MERGES clause: "if the Actions-named protection contexts block a merge
after the port is green, file the protection switch DECISION-READY naming the exact setting".
This packet names the exact setting, the exact API call, and the rollback. **The lane has changed
nothing** - branch protection is a governance control and stays with the owner.

## 1. What protection requires today (read)

`branches/main/protection/required_status_checks` (read: 2026-11-07T03:37Z):

```json
{
  "strict": true,
  "contexts": ["Repository Hygiene Checks", "Lint", "Rust Linting",
               "Test (ubuntu-latest)", "Handoff ownership scope"],
  "checks":  ["Repository Hygiene Checks", "Lint", "Rust Linting",
              "Test (ubuntu-latest)", "Handoff ownership scope"]
}
```

`required_pull_request_reviews.required_approving_review_count` is **0**, so review count is not
part of the mechanical gate; clause 10's two non-author PASSes are a protocol precondition, not a
protection setting (and A16 records that the repo has exactly one collaborator, so the protocol
precondition is currently unsatisfiable - a separate owner input).

## 2. What the rail actually publishes (read, twice)

CircleCI posts **commit statuses**, one per job, in the OAuth per-job form. Read on two lane heads:

```
ci/circleci: Repository Hygiene Checks
ci/circleci: Lint
ci/circleci: Rust Linting
ci/circleci: Test (ubuntu-latest)
ci/circleci: Handoff ownership scope
```

(read: `commits/41fed8ac.../status` and `commits/0a4d50af.../status`, 2026-11-07)

The five strings protection requires and the five the rail publishes **do not intersect**. A green
CircleCI battery therefore cannot satisfy a single required context, and conversely a green
Actions set can satisfy protection while the admissible rail says nothing.

That is the governance defect this packet exists to close. It is not a port defect: the port's job
names were copied from the protection list on purpose, and the status-name form was recorded as
unknown until the first pipeline read, which is exactly what these two reads now supply.

## 3. The decision, with the exact setting

Three options, ordered by recommendation.

**Option A (recommended) - replace the five contexts with the five CircleCI strings.**

```json
PATCH /repos/{owner}/{repo}/branches/main/protection/required_status_checks
{
  "strict": true,
  "contexts": [
    "ci/circleci: Repository Hygiene Checks",
    "ci/circleci: Lint",
    "ci/circleci: Rust Linting",
    "ci/circleci: Test (ubuntu-latest)",
    "ci/circleci: Handoff ownership scope"
  ]
}
```

Why: the standing rule already makes CircleCI the only admissible CI verdict and the end state is
"main green and verifiable on CircleCI". Keeping the Actions names required would keep a
billing-locked rail as the merge gate's source of truth - a set that can go red for reasons that
have nothing to do with a change (A18: 15/15 reds on main were billing residue while the required
five read green from the self-hosted pool). Replacing converts protection into the same thing the
protocol treats as evidence.

Consequence to accept deliberately: protected landings become dependent on CircleCI availability,
and the self-hosted OCI pool no longer gates a merge. `strict: true` stays, so a PR must still be
up to date with main and must be re-run on the up-to-date head before merging.

**Option B - add the five CircleCI strings and keep the five Actions names** (`strict: true`).
Both rails must then be green. Safest mechanically, worst operationally: it re-imports the
billing-locked hosted surface, and any Actions outage blocks every landing even when the
admissible rail is green.

**Option C - leave protection alone and merge under the standing policy on CircleCI evidence.**
Mechanically allowed today (review count is 0, and the Actions names happen to read green from
the self-hosted pool), but it means the control and the evidence describe different systems, and
main's "green" would not be reproducible from protection's own definition. Not recommended.

## 4. Preconditions before this switch is safe

1. **Wave 1 green at an exact head on the rail** - not predicted, read. At the time of filing,
   `Repository Hygiene Checks`, `Lint`, `Rust Linting` and `Handoff ownership scope` read success
   on `0a4d50afc` and `Test (ubuntu-latest)` was still running; the switch should not be made
   from a partially-read battery.
2. **The rail must run on `main` itself.** Protection is evaluated on PR head commits, but "main
   green" needs a pipeline on a main push. The port landing supplies that.
3. **A confirmed connection type.** The statuses read as OAuth per-job form. If the project is
   later switched to the GitHub App connection, statuses become per-workflow and these five
   strings stop being produced - the switch would then need re-pointing, which is worth knowing
   before, not after.

## 5. Rollback

The current setting is recorded verbatim in section 1. Restoring it is a single `PATCH` with the
five bare names and `strict: true`. No data, code or secrets are involved, so the rollback is
cheap and total.

## 6. What the lane needs from the owner

One of: "switch to Option A", "switch to Option B", or "hold" with the reason. The lane will not
perform the change under its own authority in any case, and will not stall the burndown on it:
until the switch is made, landings proceed under the standing policy on CircleCI evidence at the
exact head, with the mismatch recorded on every landing note.

## Addendum (2026-11-07, wave 2) - the ordering contradiction, stated plainly

The mission's MERGES clause carries two rules that cannot both be satisfied in the current
repository state, and the second one is the reason this addendum exists:

- "every PR merges when its CircleCI battery is green at its head", and
- "#653 lands last".

Read today: **#653 is the only branch that carries `.circleci/config.yml`.** On all seventeen open
car heads the count of `ci/circleci:*` contexts is 0 and `git cat-file -e <head>:.circleci/config.yml`
returns absent (read in `docs/audit/CLUSTER_DISPOSITIONS_2026-11-07.md`, section 0). CircleCI
builds the config from the branch being built, so a car branch cannot produce a battery until the
port exists on the branch it is based on. "Last" and "only after" cannot both hold: holding the
port to the end guarantees no car can satisfy its own merge rule in the meantime.

Recommended resolution, stated as a decision rather than taken: **land the port first**, as the
first landing of the train, and let the cars follow it. That also supplies precondition 2 above
(the rail running on `main` itself), which no other landing can supply.

Two facts that keep this from being urgent in the wrong way:

1. The required Actions contexts are not permanently red. On `b3453a188` the self-hosted pool
   carried them to `Lint` success and `Test (ubuntu-latest)` success, and on `d1d6c1025` the read
   shows `Repository Hygiene Checks` in progress with two `Detect changed areas` jobs queued
   behind it. So a landing under the CURRENT setting is mechanically possible; it is simply slow
   (a two-slot pool) and it makes the merge gate's source of truth a rail the protocol does not
   accept as evidence.
2. Clause (f) is satisfied for this head on fresh annotations, not on the earlier shorthand: the
   four failing non-required Actions check-runs on `d1d6c1025` (`Kotlin Linting`,
   `JavaScript Linting`, `WASM`, `label`) all carry
   "The job was not started because your account is locked due to a billing issue"
   (read: `check-runs/<id>/annotations`, 2026-11-07). A job that never started is not a defect in
   the change.

Nothing in this addendum changes section 3: the lane still performs no protection change, no
force-push and no admin merge.
