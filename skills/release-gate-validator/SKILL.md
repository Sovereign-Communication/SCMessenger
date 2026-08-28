---
name: release-gate-validator
description: Validate release readiness against milestone criteria and risk registers
---

# Release Gate Validator Skill

## Workflow
1. Load active milestone plan (docs/MILESTONE_PLAN_V0.2.0_ALPHA.md)
2. Check all gates marked complete
3. Verify residual risk register status
4. Run interop matrix validation
5. Confirm build verification passes
6. Check documentation chain is current

## Required Checks
- cargo test --workspace passes
- ./gradlew assembleDebug passes
- iOS archive builds
- Interop matrix gates green
- Risk register items addressed or accepted
- REMAINING_WORK_TRACKING.md current

## Output
- Release candidate status
- Blocking issues list
- Risk acceptance requirements
