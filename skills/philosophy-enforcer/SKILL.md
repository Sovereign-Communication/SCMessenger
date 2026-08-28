---
name: philosophy-enforcer
description: Capture, maintain, and enforce the SCMessenger product and engineering philosophy through a required interview and gate-based compliance review. Use when users ask to define repo philosophy, align plans or code to philosophy, review whether a change matches app principles, or request enforcement of standards and intent across docs, architecture, UX, and implementation.
---

# Philosophy Enforcer

Run a disciplined workflow that first collects philosophy from the user, then turns it into an enforceable canon, then checks requested work against that canon with explicit pass/fail gates.

## Workflow

1. Run the required interview before enforcement unless a current canon already exists and the user explicitly says to skip interview updates.
2. Write or update the philosophy canon at `reference/PHILOSOPHY_CANON.md` in the repo.
3. Derive enforceable rules from the canon using the rubric in `references/enforcement-rubric.md`.
4. Evaluate the requested change against each rule.
5. Block, revise, or approve work based on gate outcomes.

## Required Interview

Use `references/interview-protocol.md` and ask questions in small batches.

Interview goals:
- Extract non-negotiable principles, not vague preferences.
- Capture explicit tradeoffs and anti-goals.
- Resolve conflicts between performance, privacy, usability, reliability, and velocity.
- Confirm examples of "good" and "bad" outcomes.

If the user is unavailable or declines details, proceed with best-effort assumptions, mark every assumption explicitly, and lower enforcement confidence.

## Canon Format

Maintain `reference/PHILOSOPHY_CANON.md` with these sections:
- Mission and product intent
- User promises and trust boundaries
- Engineering principles
- UX and interaction principles
- Security and privacy posture
- Reliability and operability posture
- Performance posture
- Explicit anti-goals
- Prioritized tradeoff matrix
- Decision examples (accepted and rejected)

Keep each rule testable and concise. Reject soft language like "generally" or "prefer" unless tied to a measurable condition.

## Enforcement Mode

For each relevant requested change (code, docs, architecture, tests, release plan):
1. Map the change to canon rules.
2. Score each rule: `PASS`, `CONDITIONAL`, or `FAIL`.
3. For non-pass results, provide concrete remediation.
4. Refuse final approval if any rule marked non-negotiable is `FAIL`.
5. Report residual risk when `CONDITIONAL` results remain.

## Output Contract

Return results in this order:
1. Interview deltas (new or changed philosophy)
2. Canon updates made
3. Enforcement table (rule -> status -> evidence)
4. Required remediations
5. Final verdict (`APPROVE`, `APPROVE WITH CONDITIONS`, `REJECT`)

## References

- Interview script and capture template: `references/interview-protocol.md`
- Enforcement gates and scoring rules: `references/enforcement-rubric.md`
