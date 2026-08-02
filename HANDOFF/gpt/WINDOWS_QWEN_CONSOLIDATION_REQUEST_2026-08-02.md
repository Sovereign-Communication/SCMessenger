# Windows handoff -- Qwen free-tier consolidation and CI recovery

Status: READY FOR WINDOWS EXECUTION
Created: 2026-08-02
Requester: Mac lane

## Current facts

- `main` is the integration branch; the current repository integration tip is
  `7eb6bd48` and already contains the Claude-coauthored dispatch documents
  `64a681d6` and `7eb6bd48`.
- `gpt/v050-ios-release-ready` is now `2913f9d2` and contains the
  verified iOS interoperability fix, the Josh Android readiness handoff, and
  this Windows response.
- The three commits on the candidate branch are:
  - `c4052f7e` — iOS canonical JSON identity QR, BLE `peer_id`, and dual-stack
    mDNS discovery (`_p2p._udp` + legacy `_scmessenger._tcp`).
  - `f1cfa0a5` — Josh Android readiness findings.
  - `d78b0679` — Windows Qwen consolidation and CI recovery delegation.
  - `2913f9d2` — corrected branch naming and explicit Hermes/Nemotron response.
- Mac verification for `c4052f7e`: authoritative Xcode simulator build,
  role-mode checks, and local transport fallback checks passed.
- Do not merge the Mac working branch directly into `main` until the Windows
  integration tree has passed its gates and the iOS provenance is recorded.

## Direct response to Hermes/Nemotron

This document is the Mac-lane handoff response. The Qwen free-tier work is
delegated to Windows in eight bounded lanes (W1-W8) below, with disjoint write
sets and serialized Rust/core changes. Windows should acknowledge by returning
the candidate SHA, lane/model assignments and outputs, branch dispositions,
GitHub Actions evidence, device/cloud evidence, and remaining operator
decisions. No Qwen lane should merge directly to `main`; all changes land on
the fresh Windows integration branch only after its required gates pass.

## Required consolidation order

1. Synchronize `main` to the current integration tip `7eb6bd48`, then create a
   fresh Windows integration branch from `main`. The Mac checkout's local
   `main` is stale at `7d396f4d`; do not use that stale copy as the source.
2. Keep the two Claude dispatch documents as planning inputs; do not dispatch
   their original four-task wave unchanged.
3. Reconcile seeding/security work on a fresh branch. Do not merge both
   `fix/seeding-security-remediation-v040` and
   `gpt/seeding-f10-remediation` wholesale. Use a reviewed commit list,
   run the required adversarial gate, and preserve the finding-by-finding
   verdict.
4. Integrate `gpt/v050-ios-release-ready` at `2913f9d2` as the iOS
   candidate. Do not separately merge its already-ancestral iOS test-truth
   commits. Review `gpt/ios-lane-1` and
   `gpt/pr111-safe-device-resolution` separately; they carry distinct
   changes and must be cherry-picked only if their diffs are still required.
5. Treat `gpt/v050-ios-device-install` as a device-install variant,
   not a second release line. Compare its install-resolution changes against
   the candidate before selecting them.
6. Compare the CodeQL, npm, DOM-hardening, workflow-permission, release-truth,
   and branch-cleanup branches with `main`. Merge only changes whose
   diffs are not already represented on main and whose checks pass. Close or
   archive duplicate branches only after remote preservation is confirmed.

## Qwen free-tier delegation matrix

Use the currently available Windows Qwen free-tier models, verifying live
quota before dispatch. Keep write sets disjoint and serialize all Rust/core
changes.

| Lane | Scope | Mode | Required result |
|---|---|---|---|
| W1 | Branch/convergence inventory | read-only max/plus | ancestry table, duplicate/landed disposition, no merges |
| W2 | U2 topic constant centralization | flash/coder | exact-value preservation proof, protocol tests, Rust gates |
| W3 | Repository/CI hygiene | flash/plus | `git diff --check`, hygiene failures, workflow fixes only |
| W4 | Seeding finding closure | max, serial | one finding matrix, no unreviewed security edits, adversarial gate request |
| W5 | Outbox/receipt delivery truth | coder | current-HEAD tests and evidence; implement only approved missing work |
| W6 | Android build/device readiness | coder/plus | APK artifact, ABI check, install/cold-launch and physical proof plan |
| W7 | GitHub Actions diagnosis | plus/max | every failing job categorized, minimal fix PR or explicit environment blocker |
| W8 | Release/convergence report | max, read-only | final candidate SHA, merged/deferred list, exact remaining gates |

Do not spend this wave on PQC-09 implementation. It is parked during the
0.4.0 freeze and any future implementation requires crypto-security review.
A-05 iOS receipt work is already represented in the iOS lane and must be
validated, not redundantly reimplemented by Qwen.

## GitHub Actions enterprise-trial recovery

Run the full workflow set from the fresh integration candidate and record the
run SHA, job URL, and failure class:

- CI / cross-platform / desktop / mobile
- hygiene / security
- iOS build-test (Mac-authoritative; Windows may inspect, not replace, the
  Xcode result)
- release validation and docs checks

Fix only actionable repository failures. Treat missing secrets, unavailable
Windows hardware, and missing physical-device access as explicit environment
gates rather than masking them. Re-run all affected workflows after each
minimal fix and publish one green-run evidence table.

## Release gates that must not be skipped

- Seeding findings: terminal adversarial verdict and operator disposition.
- Android/Windows: physical `ConnectionEstablished` plus authentic decoded
  receipt in both directions.
- iOS/Android: same-SHA provenance, QR scan both ways, bidirectional message
  and receipt proof, restart/reconnect proof.
- Josh WAN: explicitly decide whether 0.4.0 is LAN/BLE-only alpha or the WAN
  release; resolve the AWS/cloud-node contradiction before tagging.
- No release tag from a dirty tree; no force deletion of a worktree with
  modified/untracked files.

## Response required from Windows

Return one handoff containing:

1. final integration branch and candidate SHA;
2. Qwen lane/model/task assignments and outputs;
3. branch disposition: merge, duplicate/close, defer, or blocked;
4. GitHub Actions green-run table;
5. physical-device and cloud-node evidence status;
6. exact remaining operator decisions before merging/tagging.
