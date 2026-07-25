# Wave H -- Human-Only Gates

Status: Active. Last updated: 2026-07-23 (operator clarification).

These items cannot be delegated to any agent. Each requires direct operator
action. No agent may move these to done/ without a recorded operator
confirmation.

| ID   | Gate                             | Status | Operator Action Required                                                                                                                              | Unblocks                                         |
|------|----------------------------------|--------|-------------------------------------------------------------------------------------------------------------------------------------------------------|--------------------------------------------------|
| H-01 | GitHub Actions billing           | RESOLVED 2026-07-23 | Repo is on Enterprise trial -- Actions and runners are working. No further action needed. | D-07 (CI runners), D-03 (iOS xcodebuild), iOS lane |
| H-02 | Physical two-device field trials | PENDING | WiFi Aware two-Android test; BLE sneakernet (tractor route) test; results logged to HANDOFF/review/ | T-02 final verification, Farm FD-2/FD-3 drills   |
| H-03 | Three P1-10 sign-offs            | PENDING | Approve: (1) peer_exchange semantics for GroupInfo sharing, (2) GroupInfo.port FFI field addition, (3) transport_memory privacy fingerprint design     | C-02, C-03, C-04 (adaptive port selection)       |
| H-04 | AWS node resume                  | RESOLVED 2026-07-23 | IAM user confirmed, EC2 freely accessible. Agents may activate B4 cloud node (infra/aws/setup_credentials.sh) and run 12-node farm-sim. | B4 AWS rig, 12-node farm-sim on real WAN          |
| H-05 | Final release sign-off           | PENDING | All waves A/D/C/E complete; farm FD drills logged; PQC adversarial reviews on file; release notes approved; Apple Developer account decision resolved  | v1.0.0 tag                                       |

## Notes

- H-01 RESOLVED: Enterprise trial confirmed by operator 2026-07-23. iOS CI lane and all other runner-dependent tasks are unblocked.
- H-04 RESOLVED: AWS IAM user confirmed by operator 2026-07-23. Farm-sim and cloud node tasks are unblocked.
- H-03 is the only remaining gate that blocks code work (C-02/C-03/C-04 adaptive port implementation).
- H-02 and H-05 are terminal gates sequenced at the end.

Status: H-01 and H-04 resolved. H-02, H-03, H-05 pending operator action.
