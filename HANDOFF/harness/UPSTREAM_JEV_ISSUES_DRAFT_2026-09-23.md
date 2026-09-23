# Upstream issue draft — Sovereign-Harness JEV (ready to paste on operator approval)

Status: DRAFT — outward-facing action (rule 14); file only with operator approval.
Source: SCM dogfood runs recorded in `HANDOFF/harness/JEV_DOGFOOD_RUN_2026-09-22.md`
(extensions 1-3), all reproducible with outputs saved there. Harness pinned at
`f07c814` (main @ v0.4.0). Two findings; A2's evidence is live-observed.

---

## Draft 1 — jev-phase: keyed semantic score scale contract is ambiguous

**Title:** jev-phase: semantic score scale contract is ambiguous; a 0-5 keyed response collapses a 100-mechanical phase to fail

**Body:**

Repro: `harness jev-phase --phase <any fully-green phase>` against a repo whose
phase passes all six hard gates.

Observed on Harness's own JEV-COMPLETION phase (native, no overrides): all six
hard gates green, mechanical score **100.0**, yet the keyed semantic evaluation
returned **2.76** (model `jev-1.13.0`, confidence 0.76), so
`combined = 0.7*100 + 0.3*2.76 = 70.83 < 85` → `can_mark_complete: false`.

A fully-green phase fails solely on the semantic term.

Root cause hypothesis (from `jev_completion.py`, `score_phase_completion` /
the semantic-score mapping): scores in `(0, 1]` are scaled ×100, but values
`> 1` are taken at face value. The keyed model's typed response legend appears
to run 0-5 (or 0-10), so a good answer ("4") reads as 4.0/100 — catastrophic —
while a bad answer on the same legend is indistinguishable from a great answer
on a 0-100 legend. The code cannot tell which legend the model used.

Two concrete fixes worth considering (either suffices):
1. Pin the semantic scale in the typed response contract AND clamp/mapping on
   read: accept 0-1 (×100) or 0-5/0-10 (×20/×10), reject 1 < x <= 1.5 range
   ambiguity by legend, not by guess.
2. Return the legend version in the keyed response so the blender can scale
   explicitly instead of inferring.

Behavior otherwise is exactly right: fail-closed held (the phase was NOT
marked complete), so this is a correctness/usability defect, not a safety one.

---

## Draft 2 — keyed sort: malformed probability vector falls back silently (works, but visibility could be deliberate)

**Title:** issue-sort keyed mode: malformed probability vector triggers structural fallback — flag the acceptance threshold as configurable?

**Body:**

During a live-sort of node log lines, the keyed model returned a probability
vector that failed validation ("bucket.probabilities must sum to 1"). The
TypeSafe layer caught it, kept the code-owned keyword match, and marked
`is_fallback: true` with the reason in the output. Fail-safe behavior worked
exactly as documented — this is not a bug report on the degrade itself.

Two observations offered for triage:

1. **The degrade is correct; the visibility question is whether the reason
   string should carry the model's raw return** so operators can distinguish
   "model agreed via keywords" from "model output was malformed" — today the
   reason string says the vector failed, but not what the model tried to say.
   (Low priority; current output is already honest.)

2. **Observed acceptance threshold for non-keyword-backed buckets sits
   between 0.38 and 0.72** from two data points in the same run: a bucket the
   model chose with no keyword support was *accepted* at confidence 0.72, while
   another model choice at 0.38 was *held* to `unmatched` by the code-owned
   layer. If there is a documented threshold, fine — but it is not visible in
   the output, and two operators reading the same output could reasonably
   disagree about why one line was accepted and the other held. Suggest either
   documenting the constant or emitting it per-run.

Environment: harness v0.4.0 (`f07c814`), unkeyed pack with 10 buckets,
Windows host, stdio CLI. Raw outputs available on request.

---

## Lane disposition

- Not filed by this lane (rule 14: outward-facing; operator decides).
- If the operator approves, both drafts paste as-is; Draft 1 is the
  load-bearing one (it currently mis-fails every fully-green phase in keyed
  mode, which makes `jev-phase` unusable as a completion gate until fixed).
- No SCM-side change is warranted: our runs fail-closed correctly in both
  cases, which is the safe direction.
