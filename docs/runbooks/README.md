# Runbooks

Status: Active
Created: 2026-09-22

Short, executable procedures for the recurring operations every seat performs
repeatedly. If a runbook here drifts from reality, fix the runbook in the
same change that touches reality.

| Runbook | When |
|---|---|
| [CI_PRIMARY_BUILD.md](CI_PRIMARY_BUILD.md) | Verifying any change; deciding local vs CI; reclaiming after a failover build |
| [CI_APK_TO_PHONE.md](CI_APK_TO_PHONE.md) | Getting the latest CI-built Android APK onto the operator's Pixel |

Related canonical rules: `docs/rules/BUILD_AND_CI.md` (build doctrine),
`docs/rules/FREEBUFF.md` (lane authority incl. commit/push),
`docs/rules/NODE_MODEL.md` (all nodes identical; identity-or-not is the only
distinction), `HANDOFF/RUNBOOK_PIXEL_PASSIVE_VERIFICATION_2026-09-11.md`
(device interaction limits — install and passive logs ONLY).
