# Interview Protocol

Use this protocol to gather philosophy directly from the user before enforcing it.

## Session Rules

- Ask 2-4 questions at a time.
- Reflect back the answers in concise bullets before moving on.
- Ask follow-ups only when answers are ambiguous or conflicting.
- Capture confidence for each section: `high`, `medium`, `low`.

## Question Set

### 1. Mission and outcomes
- What must this app achieve that alternatives do not?
- Which user outcomes matter most in priority order?
- What outcomes are explicitly out of scope?

### 2. Non-negotiable principles
- Which principles are absolute, even if they slow development?
- Which principles are aspirational but can be traded off temporarily?
- What failures are unacceptable?

### 3. User trust model
- What user promises can never be broken?
- What data handling boundaries are mandatory?
- Where should transparency be shown to users?

### 4. Product behavior and UX
- What should interactions feel like (tone, friction level, speed)?
- What UX patterns are forbidden?
- What tradeoff is preferred when simplicity conflicts with power?

### 5. Engineering philosophy
- Which architectural qualities are required (modularity, offline-first, etc.)?
- How should complexity be managed?
- What debt is acceptable and what debt is forbidden?

### 6. Security, reliability, performance
- What is the minimum acceptable security posture?
- What reliability targets matter most (consistency, uptime, recoverability)?
- Where can performance be traded for safety or correctness?

### 7. Decision precedence
- When principles conflict, what tie-breaker order should be used?
- Who has final say on philosophy interpretation?
- What evidence is required to justify an exception?

### 8. Positive and negative examples
- Give 2 examples of a change that strongly fits the philosophy.
- Give 2 examples of a change that violates the philosophy.

## Canon Capture Template

Record answers into `reference/PHILOSOPHY_CANON.md` as concrete rules:

- Rule ID: short stable ID (`PHIL-001`)
- Rule text: one testable statement
- Scope: docs, code, UX, architecture, operations
- Criticality: `non-negotiable` or `negotiable`
- Verification: how to check compliance
- Exception path: required approval and evidence

## Confidence Handling

- If confidence is low in any section, mark related rules as provisional.
- Re-interview before hard enforcement on provisional rules.
