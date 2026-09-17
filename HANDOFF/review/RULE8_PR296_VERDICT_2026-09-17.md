# Rule-8 adversarial review — PR #296 (CRYPTO-01, TRN-03)

Status: CLOSED — APPROVE-WITH-NOTES
Date: 2026-09-17
Reviewer: harness structured-claims panel (3 independent non-author models)
+ judge. Author (Freebuff lane) excluded from the panel by construction.
Method: `harness.cli verify` structured-claims mode — candidate-defect claims
manifest + verbatim diff source window, per-claim panel votes, deterministic
convergence tally, judge synthesis. Free tier attempted first and was
saturated (tier condition: reasoning-only and truncated outputs); paid
escalation per operator authorization capped at $0.05/use.
Artifacts (Harness checkout, outside the product tree):
- `Harness/audits/scmessenger/_runs/seat-gates/verify_pr296_rule8_20260917.json` (free attempt, unusable outputs)
- `Harness/audits/scmessenger/_runs/seat-gates/verify_pr296_claims_20260917.json` (2/3 voted, shortfall-deferred)
- `Harness/audits/scmessenger/_runs/seat-gates/verify_pr296_claims3_20260917.json` (3/3 voted — authoritative)
- Claims/source inputs: `SCMessenger/tmp/rule8_pr296_{claims.json,source.txt,prompt.md}`
Total paid spend for this review: ~$0.0059 (ceiling $0.05).

## Panel result (3/3 voted: deepseek-v4-flash, ling-3.0-flash, gpt-4o-mini; judge z-ai/glm-5.3-flash)

Tally: c1=not_real (1R/2NR); c2=not_real (1R/2NR); c3=not_real (1R/2NR);
c4=not_real (1R/2NR); **c5=real (2R/1NR)**.
Judge synthesis: only c5 real — the WASM own-topic else-branch fail-open when
the local peer id carries no inline Ed25519 public key silently continues
without own-topic subscription (medium, doctrine: fail-closed). c1-c4
rejected: canonical_peer_id and sender_public_key_hex are post-verification
values, not attacker-controlled; the delegate semantics change is the
intentional security fix; own-topic subscription + ghost guard restore proven
native-path parity; no demonstrated loop or bypass. Note: gpt-4o-mini
dissented alone on c1-c4; judge weighted the code-evidence-based positions.

## Seat disposition of c5 (verified against the diff, not taken on the vote)

The flagged branch logs `[WARN] Own peer topic not subscribed on wasm: local
peer id carries no inline Ed25519 public key` and continues. This is a
deliberate degrade, not a new hole: an RSA/identity-hash PeerId is outside
SCMessenger's identity scheme (all SCM peers are Ed25519-inlined), the native
path shares the same property, and the observable warn line is the tripwire.
No message is misdelivered or spoofed by this branch; it is the pre-existing
TRN-03 state on an impossible configuration. ACCEPTED as LOW with the
existing warn as the detection path. No code change required for v0.4.0.

## Verdict

APPROVE-WITH-NOTES. Mechanically verified: CRYPTO-01 closed (authenticated
values only), TRN-03 closed on the real configuration space, ghost guard
parity restored, no spoofing/loop/bypass mechanism found by 3 independent
reviewers + judge + seat. CI: all checks green (label, Wiring Gate, iOS
Build & Simulator Test, Android JVM Unit Tests, macOS Native Tests, Android
Debug APK, iOS Build); mergeable=MERGEABLE, mergeState=CLEAN at merge time.
