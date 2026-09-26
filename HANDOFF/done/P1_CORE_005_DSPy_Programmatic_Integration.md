# P1_CORE_005: DSPy Programmatic Integration

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Priority:** P1
**Platform:** Core / Swarm
**Status:** Open
**Assignee:** Primary Worker / Architect

## Objective
Port the existing `AgentSwarmCline/scmessenger_swarm/swarm.py` from raw prompt orchestration to a **DSPy programmatic framework**. This will ensure deterministic model routing and enable autonomous prompt optimization.

## Key Requirements
1.  **Define Signatures**: Create DSPy `Signatures` for all swarm roles (Architect, Coder, Verifier, Auditor).
2.  **Module Implementation**: Build a multi-hop pipeline that handles logic flow (e.g., Code -> Test -> Audit -> Fix) as a Python program.
3.  **Optimization Harness**: Setting up a `Teleprompter` to compile and optimize prompts for specific SCM scenarios using verified code as "Golden Examples".
4.  **Local Integration**: Ensure the DSPy logic correctly calls the local Ollama proxy `:cloud` endpoints.

## References
- [docs/DSP_SCM_PLAN.md](file:///c:/Users/kanal/Documents/SCMessenger/SCMessenger/docs/DSP_SCM_PLAN.md)
- [AI_STANDARDS.md](file:///c:/Users/kanal/Documents/SCMessenger/SCMessenger/docs/AI_STANDARDS.md)

## Verification
- `swarm_dspy.py` reproduces existing feature generation with < 5% variance in tool-use accuracy.
- Successful optimization run against `core/src/crypto` verification tasks.
