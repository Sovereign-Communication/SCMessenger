Task: AUDIT_CANONICAL_OUTLIERS_V040_V050_V100_2026-09-19.md
Type: DONE
Milestone: M5 (DIM-E + DIM-F)
PR: UNVERIFIED -- branch glm/canonical-outlier-audit pushed with iteration commits.
Report: HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_iter5.md
Counts: DIM-E = 3 findings (1 verified-consistent no-finding recorded); DIM-F = 3 findings.
Notes: Flagship: CO-E-001 MED -- the AWS address single-source policy (HANDOFF/gpt/AWS_RELAY_CURRENT_ADDRESS.md) is contradicted by 28 stray copies of 18.234.62.247 across tracked files, including 3 ACTIVE todo tickets (CELL_ROUTE_AWS_001, P1_CLI_SEND_CANONICAL_IDENTIFIER_PARITY, P1_WINDOWS_NODE_SILENT_WEDGE); full 29-item list printed with no elision. BOOTSTRAP.md:46 "no hardcoded routable IPs in core/CLI" verified TRUE (test/RFC-5737 literals only). CO-F-002: FEATURE_PARITY matrix re-audit demanded "before v0.4.0 sign-off" but unscheduled.
