# v0.4.0 Tag-Readiness Evidence Package

Prepared: 2026-09-15 (Freebuff lane, CTO seat). Purpose: the complete evidence
set the operator asked to review BEFORE any tag action. No tag has been
created; no push of tags; merge of PR #288 remains with the orchestrator per
standing order. Every claim below was re-derived from fresh commands this
session (rule 13); the command or artifact that proves it is named.

## 1. Functionality: three-node mesh, cellular path

Verdict: **PASS** (store-and-forward verified end-to-end; zero message loss).

- Cellular triangulation (full evidence:
  `HANDOFF/audit/CELLULAR_PATH_TRIANGULATION_2026-09-15.md`):
  - Direct Pixel<->Windows connection was NEVER achieved (DCUtR hole-punch
    fails under double NAT); the relay fallback engaged correctly. This is
    doctrine-conformant behavior: every node relays.
  - All cellular messages were stored and forwarded: the AWS cloud node held
    12/12 custody entries destined for Windows and burst-delivered them within
    ~1 second of the Windows node's recovery (04:22:14Z), zero loss.
  - Receipts converged immediately on recovery (04:22:24Z, both stuck
    messages "Processed application delivery receipt"); phone outbox empty.
- Live at time of writing: AWS delivered custody to Windows at 16:24:57Z;
  Windows node log writing within seconds of every probe (16:45+); cloud node
  uptime 16h on the current image.
- One defect surfaced by this analysis (Windows silent wedge, 2h45m) is
  remediated and bounded — see section 4.

## 2. Build parity: all three nodes on the unified 0.4.0 stack

Verdict: **PASS** (three implementations, one stack, identity preserved).

| Node | Build | Identity preserved | Evidence (this session) |
|---|---|---|---|
| AWS cloud | `sha-31776b4` image (glibc-pinned Dockerfile, commit 31776b48) | yes (ledger identity through /data bind-mount) | `docker ps` + boot logs; mesh reformed in 6s |
| Windows desktop | CI artifact `fb46f2a` == tree of `d7b4f77d2` (provenance: run 34932193671, `git merge-base --is-ancestor origin/main d7b4f77d2` => merge-preview tree identical) | yes (`local_peer_id=12D3KooWD6vZ...`, "Loaded existing identity", 16:45:16Z) | swap executed preserve-first, rollback staged `tmp/rollback-cli-f985b10.exe` |
| Pixel 6a | CI artifact APK `b39bfd2d` (run 34907771392, SHA256 aba7d865...) | fresh install per operator deletion | `adb install` + dumpsys versionName 0.4.0 |

Identifier parity: `HANDOFF/audit/IDENTIFIER_PARITY_AUDIT_2026-09-15.md`
(canonical identity = public_key_hex; peer_id/identity_id derived metadata;
Kotlin mirror byte-exact; iOS derives nothing).

## 3. CI / release-pipeline state

Verdict: **CONDITIONAL PASS** — all commit lanes green on the watchdog SHA;
release rehearsal re-running after an infra fix (was not a code failure).

- CI, Mobile, iOS Build & Test, Cross: **success** on `d7b4f77d2` (run
  34932193671). Commits after it are docs/workflow-only (zero binary delta,
  verified by `git diff --name-only d7b4f77d2..bac70105`).
- Release rehearsal attempt 1 (run 34932614680, artifacts_only) failed at
  "Build Android Release": `android-actions/setup-android@v3` attempted the
  deprecated `tools` SDK package (`Failed to find package 'tools'`). Root
  cause: infra drift in the action, NOT code. `mobile.yml` already carried the
  fix; release.yml was aligned in commit `12bdf292`
  (`packages: 'platform-tools'`).
- Rehearsal attempt 2 (run 34996353889) in progress at time of writing; the
  signed-APK job is the historic release blocker and all four signing secrets
  are present (`gh secret list`: KEYSTORE_BASE64, KEYSTORE_PASSWORD,
  KEY_ALIAS, KEY_PASSWORD).
- Tip `bac70105`: CI/Lint/iOS/Mobile/Cross in flight; Repository Hygiene and
  Auto Label already green.

## 4. Defect remediations this cycle

- **P1 Windows node silent wedge** (ticket
  `HANDOFF/todo/P1_WINDOWS_NODE_SILENT_WEDGE_2026-09-15.md`): root cause
  deferred to 0.5.0; remediation is the log-silence heartbeat watchdog
  (commit `d7b4f77d`): after 600s of no log output the node exits loudly,
  bounding the silent outage to 10 minutes. Verified by 2 black-box
  integration tests (`cli/tests/heartbeat_watchdog_integration.rs`, locally
  2 passed in 6.32s; CI Test lane green on `d7b4f77d2`). The watchdog now
  runs on the live Windows node.
- **P1 rustls RUSTSEC-2026-0285**: remediated — `Cargo.lock` holds rustls
  0.23.45, the deny.toml waiver is gone, tracking ticket
  `P1_SECURITY_RUSTLS_RUSTSEC_2026_0285_TRACKING.md` closes on merge to main
  (Dependabot alerts track the default branch).
- Remaining deny.toml waivers are documented transitive advisories with no
  upstream fix available (sled/hickory 0.25 tree) — accepted, documented.
- **PR #290** (Class-A Kotlin fix: SubnetProbe `hostAddress` null-receiver +
  latent IndexOutOfBounds): 7/7 checks green, stacked on this branch.

## 5. Governance (sovereign-harness BoD, paid tier)

Two recorded runs (`HANDOFF/BOD_STATE.md`, resolutions bod-78ae5652 and
bod-b326b530) on the P1 wedge disposition:

- Run 1: 4/5 APPROVE (scores 0.88-0.96). Run 2: 4/5 APPROVE (scores 0.88-0.98).
- Zero REJECT votes across both runs.
- The deepseek-v3.2 seat returned malformed output in BOTH runs; per the
  skill's fail-closed rule each run recorded DEFERRED_PANEL_SHORTFALL.
- Per the consensus rule, converting that shortfall into an APPROVED
  disposition requires an explicit operator ruling — this is the one item the
  harness could not close on its own.

## 6. What the operator is asked to rule on (the tag ask)

1. **Accept the P1 wedge disposition**: known issue for 0.4.0, blast radius
   bounded to 10 minutes by the watchdog, root-cause reproduction + fix
   tracked for 0.5.0 (ticket stays open). [Yes / No / More evidence]
2. **Confirm tag plan**: once (a) rehearsal run 34996353889 is green,
   (b) tip CI is green, and (c) PR #288 merges to main — tag `v0.4.0` on the
   merge commit. Merge and tag remain operator/orchestrator actions per
   standing order; this lane does not self-execute either.
3. **0.5.0 kickoff backlog** (filed, not started): wedge root-cause
   reproduction (priority), UniFFI relocation closing the peer_<hex>
   fallback, Android Kotlin warning burndown (P2 ticket), hickory advisory
   recheck when libp2p moves past the 0.25 hickory tree.

## 7. Known limitations (honest disclosure)

- DCUtR hole-punch fails under double NAT in the current test topology; relay
  custody carries all cell-path traffic by design. UX impact: delivery
  latency equals custody polling cadence when a direct path is unavailable.
- The 0.4.0 tag contains no Apple App Store / Play Store submission step;
  release pipeline produces signed artifacts for direct distribution.
- Windows node wedge root cause unknown (no fingerprint in logs, no panic);
  watchdog converts recurrence into a bounded, observable outage.
