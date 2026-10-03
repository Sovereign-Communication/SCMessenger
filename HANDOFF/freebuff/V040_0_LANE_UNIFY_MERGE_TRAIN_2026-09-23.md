# V0.4.0 Lane Unification Merge Train -- 2026-09-23

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Active -- single sequenced plan for the working 0.4.0
Owner lane: Freebuff (this doc) -- merge authority: Windows orchestrator/operator
Dogfood context: AWS always-on + OpenClaw nodes dogfooding via Sovereign-Harness
and OpenClaw sessions; SCMessenger must drive the setup (Telegram is fallback).

## Rule 17 note

The Windows checkout is below the disk hard floor (disk_budget.py [FAIL]).
No local cargo/gradle builds from this point; CI is the builder and the
artifact source. Node binaries are staged in `tmp/radio-<sha>/` per runbook
convention.

## Lane inventory (as of 2026-09-23 17:0x UTC)

| Lane | Vehicle | State (evidence) |
|---|---|---|
| Freebuff (this) | PR #364 stacked on d9-libp2p-degrade | CI running; commits cc51e41a |
| D9 vendored swarm | PR #361 `d9-libp2p-degrade` | OPEN; rule-8 review pending |
| Two-tier conn policy | PR #359 `glm/canonical-outlier-audit` | OPEN; rule-8 review pending |
| Android stop-teardown timeout | PR #351 `freebuff/android-stop-teardown-timeout` | OPEN |
| Outbox retry diagnosis | PR #316 `freebuff/outbox-retry-fix` (docs) | OPEN; wiring delivered by #364 |
| WP1-WP4 (V050) | PRs #352, #349, #355, #356 | OPEN; JEV-keyed evidence |
| OpenClaw bridge ops | PR #363 `codex/openclaw-bridge-ops` | DRAFT; CI green |
| Harness lane state | PR #362 `claude/harness-lane-state-2026-09-22` | OPEN; docs-only |

## Merge order (blockers named)

1. **#361 D9 vendored-swarm degrade** -- must land first; #364 is stacked on
   it. Rule-8 adversarial APPROVE on file is the blocker (no self-approval).
2. **#364 stop/start serialization + outbox sweep** (this lane) -- delivers
   the AND-SS-001 and OUTBOX-SWEEP-001 burndown. Blockers: #361 merged,
   green CI, rule-8 adversarial APPROVE for the sweep arm.
3. **#351 bounded stop teardown** -- same file as #364 Defect 1; rebase or
   refactor after #364 lands (expect a trivial conflict in
   MeshForegroundService.kt; the timeout wrapper goes around the coalesced
   teardown).
4. **#359 two-tier conn policy** -- independent of the stop/start file;
   rule-8 review pending.
5. **#316** -- closes as delivered-by-#364 once #364 merges (diagnosis only).
6. **WP train (#352/#349/#355/#356)** -- V050 WPs, keyed JEV evidence per
   repo doctrine; sequence after the 0.4.0 burndown lands.
7. **#363 / #362** -- docs/draft; no 0.4.0 dependency.

## 0.4.0 readiness gates (all required)

1. #361 and #364 merged (rule-8 reviews on file), #351 rebased and merged.
2. Pixel running a CI-built APK whose provenance includes cc51e41a
   (runbook `docs/runbooks/CI_APK_TO_PHONE.md`).
3. Cloud nodes (AWS + OpenClaw) running a CI-built `windows`-equivalent CLI
   from a post-#361+#364 SHA; staged in `tmp/radio-<sha>/`.
4. Post-rollout 3-node audit: zero panics, `outbox_flush_skipped` absent
   from connect logs, undelivered counts draining on all three nodes.
5. Stop/start live exercise on the Pixel: Start -> Running, Stop -> Stopped,
   flood of STOPs coalesces to one teardown, Start after Stop comes back.

## Rollout steps (operator-gated actions flagged)

1. [seat] Download `android-debug-apk` artifact from #364 Mobile run.
2. [seat] Install on Pixel over existing app (data preserved) and pull
   passive logs.
3. [seat] Download `windows-cli-<sha>` artifact from #364 CI run; stage in
   `tmp/radio-<sha>/`; local Windows node restart is OPERATOR-GATED (live
   identity custody on this host).
4. [seat] SSH deploy to AWS + OpenClaw nodes, restart services, verify.
   [OPERATOR-GATED per NODE_MODEL: cloud nodes run operator identity.
   Ask before touching cloud node state.]
5. [seat] Post-rollout log audit + verdict update in this file (append-only).

## Addendum 2026-09-23 ~17:20 UTC (seat update)

- Docker Publish was broken for every post-D9 SHA: vendor/libp2p-swarm-0.48.0
  (workspace member since the D9 vendored-swarm change) was missing from the
  image build context. Fixed on #364 (45b0f8b8, DOCKER-VENDOR-001); the fix
  must ride along with #361's merge or every cloud deploy fails.
- PR #364 gained: fmt-only commit bdb5b256, docker fix 45b0f8b8.
- Operator constraints for the Pixel (supersedes earlier install notes):
  the app was UNINSTALLED -- next install is a FRESH install (new identity;
  mesh peers that held the old phone identity 779e9ea3... will hold stale
  entries). The seat may install the CI APK but must NOT drive the app:
  no UI interaction, no am-start of components beyond the runbook install
  step. Debug/RCA is by passive logcat/mesh.log pulls only.
- Node rollout targets: AWS always-on via pinned image testbotz/scmessenger:sha-45b0f8b
  (aws_deploy.sh, identity-preserved path); OpenClaw (13.217.204.112,
  ec2-user, plain binary at /home/ec2-user/scm-main/scmessenger, version
  v040d9degrade-672dffcb) gets the linux binary from the same image so all
  nodes run one provenance.
