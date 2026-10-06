Task: V040_V050_MERGE_TRAIN_UNIFY_2026-09-27.md / MT-00a
Type: PREMISE-WRONG

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->
Repository: scmessenger

**All merges are halted.** Section 14 A1 lists a condition that no honest
pre-merge state can satisfy. A1 cannot be executed as written, so per section
14.5 and the task file's PREMISE-WRONG rule I am stopping the merge path and
asking for a ruling. Nothing has been merged. #397 is fully verified and
merge-ready; it waits.

## The contradiction

A1, verbatim:

> - The car's JEV gate (2.3) is `>= 85` with hard gates clear.

A1 is a **pre-merge** checklist ("only when ALL of these hold" comes before
`gh pr merge`). But `pr_merged` is 25 of the harness's 100 points **and** is a
hard gate, so the bar cannot be reached until after the merge it is a
precondition for.

## Evidence -- all from `C:/Users/SCM/Documents/GitHub/Harness` (admitted v0.4.1)

Command: `sed -n '45,56p' harness/jev_completion.py | nl -ba -v45`

```
    45
    46  # Mechanical points when hard gates pass. Sum == 100.
    47  _HARD_GATE_POINTS = {
    48      "pr_merged": 25,
    49      "origin_evidence": 10,
    50      "required_tests_present": 20,
    51      "required_files_present": 0,
    52      "oc_handoff_verified": 0,
    53      "local_gates_green": 15,
    54      "ci_green": 15,
    55      "no_open_blockers": 15,
    56  }
```

Command: `sed -n '1011,1022p' harness/jev_completion.py` (line 1011 and 1022)

```
1011      all_hard_pass = all(gates.values())
...
1022      bar_pass = bool(all_hard_pass and combined >= float(min_score) and not blocking_axes)
```

Command: `grep -n "hard gate failed: pr_merged" harness/jev_completion.py`

```
1027          blockers.append("hard gate failed: pr_merged")
```

**Arithmetic.** Every other gate green gives `mechanical = 10 + 20 + 0 + 0 + 15 + 15 + 15 = 75`.
With `pr_merged` false, `all_hard_pass` is false, so `bar_pass` is false
regardless of the score, and 75 < 85 as well. The bar is unreachable
pre-merge on both the hard-gate axis and the score axis.

## The only way to pass it pre-merge is to falsify the status row

`pr_merged` is not read from GitHub. It is inferred from prose:

Command: `sed -n '828,840p' harness/jev_completion.py | nl -ba -v828`

```
828          mentions_pr = phase_status_mentions_pr(status_row, pattern)
829          open_pr = bool(
830              re.search(r"\b(?:PR|pull request)\s*(?:#\d+)?\s*(?:is\s+)?open\b", lowered)
831              or re.search(r"\bopen\s+(?:PR|pull request)(?:\s+#\d+)?\b", lowered)
832              or re.search(r"\bno pr\b", lowered)
833          )
834          merged_word = "merged" in lowered or "merge" in lowered
835          # Presence of a PR id is not merge evidence while the row still says open.
836          evidence["pr_merged"] = bool(mentions_pr and merged_word and not open_pr)
```

So `pr_merged` becomes true only when the status row says "merged" **and** does
not say the PR is open. Pre-merge, the true state is the opposite. The only way
to satisfy A1 before merging is to write a status row claiming a merge that has
not happened.

Appendix B forbids exactly that: "The evidence JSON holds only values captured
from commands", and "A COMPLETE `status_row` must not contain the words open,
fail, repair, pending or blocked." AGENTS.md rule 13 and rule 15 require the
same. The JEV pack has a `status_honesty` axis whose whole purpose is to catch
a completion claim that the evidence contradicts.

**So A1's JEV bullet is satisfiable only by writing a false record.** I will not
do that, which is why the merge path stops here rather than proceeding on a
technicality.

## This is not new behaviour, and TRAIN_STATE already shows it

Every pre-MT-00 gate run in the log has `pr_merged F` and scored 40-60: P0-1
25.0, P0-4 40.0, P0-6 40.0, P0-7 33.33, P0-8 none, T-1 60.0, T-2 60.0, T-4 60.0.
P0-1's log entry calls the residual an "expected pre-MT-00 fallback contract"
and says "the bar binds from MT-00". MT-00a **is** MT-00. The fallback contract
is ending exactly where it becomes load-bearing.

## What the task file says elsewhere, and why it does not resolve this

Task file 2.3(a):

> **Keyed JEV evidence gate from #396,** after MT-00a lands: the keyed JEV
> completion evidence and exact-SHA control-plane gate that #396 adds must pass
> **for the car's merge SHA**.

"For the car's merge SHA" is unavoidably post-merge. That is consistent with
reading A1's JEV bullet as a **car-completion** gate, not a pre-merge one. But
it is an interpretation, and A1's placement inside a pre-merge checklist says
the opposite. I am not going to pick the reading that unblocks me and call it
done; the operator wrote section 14 specifically to remove ambiguity from this
train, so the ambiguity should go back to them.

Note the second-order effect: **#396 is the PR that adds the keyed gate, and
#396 is one of the PRs #396's own rule would block.** The gate that is supposed
to authorise MT-00a cannot be met until the PR that adds it is merged. Any
ruling has to break that loop for #396 at minimum.

## #397 is verified merge-ready on the other five A1 conditions

Recorded so the ruling is the only thing between this PR and main.

| A1 condition (other than JEV) | State | Command |
|---|---|---|
| Branch up to date with main | [OK] base `faf22a57` == current main tip | `gh pr view 397 --json baseRefOid,headRefOid` -> base=faf22a57 head=0f4d0c84 |
| Branch is CLEAN / not BEHIND | [OK] mergeStateStatus CLEAN, MERGEABLE | same, `--json mergeable,mergeStateStatus` |
| Every required check green on head | [OK] 28/28 SUCCESS, unique conclusion `["SUCCESS"]` | `gh pr view 397 --json statusCheckRollup` |
| Required check list re-read | [OK] the 5 named contexts, unchanged | `gh api .../branches/main/protection/required_status_checks --jq '.contexts[]'` |
| Gated code has an A2 APPROVE | n/a -- not gated. Sole file is `.github/workflows/mobile.yml`, +30/-0 | `gh pr view 397 --json files` |
| No unresolved threads / requested changes | [OK] 0 review threads, 0 reviews | `gh api graphql` reviewThreads -> `{"total":0,"unresolved":[]}` |
| Failing non-required checks investigated | [OK] none fail on #397 | `gh pr view 397` -> all conclusions SUCCESS |
| main not red after a prior merge | [OK] 7/7 SUCCESS on faf22a57 | `gh run list --branch main --limit 12` |

The diff gates the Android job's steps on `android_relevant` from the existing
`detect-platform-change` action, mirroring the sibling iOS job, with a comment
block explaining that the gate fails open. It adds a condition, it does not
relax one.

## #396, for completeness

Head `2b68b7d6`, 28 checks pass, CodeQL the only failure: 2 high
`actions/cache-poisoning/poisonable-step` alerts (83 at ci.yml:142-144, 84 at
144-147). Full analysis is in TRAIN_STATE under the 2026-09-28 session. My
working verdict: **not exploitable at this SHA but a guarded shape, not an
absent one** -- the job has no `actions/cache` and no `cache:` key at all, and
a fail-closed guard at ci.yml:108-137 forces `candidate_sha == github.sha`
before any code runs. CodeQL models the `inputs.candidate_sha` -> `actions/checkout
ref:` edge, not the bash guard, so the alert would survive a future weakening.
My recommendation is to fix it structurally (checkout `ref: ${{ github.sha }}`,
keep `candidate_sha` as an asserted input) on a `freebuff/train-mt00a-*` branch
that supersedes #396, rather than merge past a high alert or dismiss it. That
work is not started and does not depend on this ruling.

## What I need

One line:

- `JEV POST-MERGE` -- A1's JEV bullet is the car-completion gate on the merge
  SHA, per 2.3(a)'s own wording. I merge on the other five conditions, then run
  the gate at car close and report the score.
- `JEV PRE-MERGE RELAXED` -- run the gate before each merge for the record,
  accept any score, and rely on the hard gates plus the keyed control-plane job
  from #396 in CI.
- `JEV EXEMPT MT-00a` -- exempt the very first PR-gated car, apply A1 in full
  from MT-01 on.
- `JEV STOP` -- halt and re-spec the gate.

Until one of those arrives I will not merge anything. Non-merge prep that
depends on no ruling (the #396 CodeQL fix branch, MT-01's gitleaks allowlist,
MT-00b's doctrine landing) continues.
