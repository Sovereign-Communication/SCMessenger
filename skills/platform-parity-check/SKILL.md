---
name: platform-parity-check
description: Verify feature parity across Android, iOS, and Web/WASM for critical controls
---

# Platform Parity Check Skill

## Purpose
Ensure critical-path behavior is identical across all platforms per PHIL-006 and PHIL-010.

## Workflow
1. Identify the feature or control being checked
2. Verify implementation exists in: android/, iOS/, wasm/
3. Compare behavior against core/ contract
4. Check UniFFI binding alignment (api.udl)
5. Generate parity matrix report

## Verification Points
- Relay ON/OFF semantics identical
- Identity display/exchange identical
- Send/receive flow identical
- Settings/preferences aligned
- Error handling consistent

## Output
- PASS: All platforms aligned
- PARTIAL: Gaps identified with remediation
- FAIL: Critical divergence found
