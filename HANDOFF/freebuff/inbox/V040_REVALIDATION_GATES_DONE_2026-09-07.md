Task: V040 (final-tree revalidation + unverified-gate evidence pass)
Type: DONE

## Scope
Candidate `cto/v040-candidate-2026-09-02` @ **85cb4c67** (tree **0989624382eb06e3**,
byte-identical to the R18-approved #278 tree; #272/#274/#276/#278 all merged; no tag,
origin/main untouched). Remaining UNVERIFIED mission gates exercised against the live
3-node fleet. Evidence: `tmp/run-evidence/reval/GATE-VERDICTS-2026-09-07.md` + raw
slices in that directory.

## Node identities / addresses used
- Windows CLI `12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw` (canonical 30d0fa67),
  control API 127.0.0.1:9876, log `.codebuff_deploy/windows/`
- AWS cloud node `12D3KooWGvCWJNoWnReNCT1q2LWb2gTbeBTa5sjxF49wZX3u2y31`, EC2
  `i-0b41aab756eabd514`, container `b390e822134b`, image `testbotz/scmessenger:sha-b0f7ac4`
  (0.4.0 tree b0f7ac4e). The stopped OLD instance `i-0b735c4f26aea42ed` was NOT touched.
- Pixel `12D3KooWR9ioPPRJ...`, APK sha256 039f5116 (tree-identical to 85cb4c67)

## Gate verdicts (exact commands + outputs in the verdicts file)
- Seed dialing: **PASS** (1,089 `[SEED-DIAL] peers=1 -- connected; re-check in 120s`)
- Ledger/peer propagation: **PASS** (boot migrations + post-churn gossip propagation to
  the phone, persisted across phone restart; minor: phone relay-circuit candidate rejected
  by AWS seed dialer `Missing destination peer id`)
- External-address filtering: **PASS** (39,815 `not a proven pair` suppressions)
- Inbound reachability: **PASS** with scope note (CLI is NAT'd; relay-mediated by design)
- Coordinated restart (AWS stop/start + CLI kill/restart + Pixel force-stop/relaunch):
  **PASS** — all three re-formed the mesh; post-restart Windows->Pixel `6610070b`
  delivered + receipted
- AWS IP churn: **FAIL** (design gap, recovered manually) — stop/start moved the public IP
  3.91.5.1 -> 18.234.62.247; NO node rediscovered the cloud node autonomously (CLI dialed
  the dead IP; AWS booted with 0 dialable candidates). Recovery required manually editing
  `%APPDATA%/scmessenger/config.json` `bootstrap_nodes` to the new IP and restarting the
  CLI (backup `config.json.bak-20260907-churn`), after which gossip propagated the new
  address fleet-wide. This is the ledger-sharing-first discovery gap (V050-B1/B2): a cloud
  node whose address changes while all peers hold only the dead address is undiscoverable.
- Cellular/BLE legs: **UNVERIFIED** — operator-supervised only per standing directive.

## Review chain status
- #278: R18 **APPROVE** at bfe6bc0b (filed `HANDOFF/review/V040_PR278_ANDROID_ANR_REVIEW_APPROVED_bfe6bc0b_2026-09-07.md`); CI 7/7, mergeStateStatus CLEAN
- Merge #278 executed 10:30:44Z (squash, tree-preserved); #272 verdicts re-pinned at 85cb4c67 (`HANDOFF/review/V040_CANDIDATE_272_VERDICT_REPIN_85cb4c67_2026-09-07.md`)
- Live-message legs (Windows<->Pixel) verified on-device earlier today (b1dd5f13: delivered, decrypted, displayed)

## SINGLE NEXT CEO DECISION
The tag decision. Remaining input: the operator-supervised BLE and cellular legs of the
re-validation. Everything else is green at 85cb4c67 except the AWS-IP-churn discovery gap,
which is a design item (V050 ledger-sharing-first discovery / cloud announce), not a local
fix — recommend recording it as a known limitation with the manual bootstrap-update
runbook, not a v0.4.0 blocker.

## Operator notes
- The Windows CLI is running (PID restarted at 16:34Z); its config now bootstraps to
  18.234.62.247. If AWS churns again, repeat the config edit or add an Elastic IP.
- No HANDOFF files other than this note were touched.

## AMENDMENT (17:20Z) — SAME-SHA AWS NODE: the pre-merge binary gap is closed

The original note's gate evidence ran against AWS binary 0.4.0 (b0f7ac4e) — pre-merge.
AWS was rebuilt and redeployed at the exact candidate SHA:
- Image `testbotz/scmessenger:sha-85cb4c6` from Docker Publish run 34144942973
  (workflow_dispatch at cto/v040-candidate-2026-09-02 == 85cb4c67); container `8946d8c3b449`
  with the same volume/env/ports; no IP churn this deploy.
- Boot log proves the build string: `0.4.0 (85cb4c67feb03d27fa004a2be6b1ce65b030eb06)`.
- All gates re-confirmed on the new binary: seed dial PASS (connected within 20s of boot),
  propagation PASS (CLI auto-reconnect 16s), ext-addr filtering PASS, inbound reachability
  PASS, E2E message PASS (f32e70ff: delivered + receipted, phone inbox_receive +
  onMessageReceived isChatEvent=true).
- **All three nodes now run ONE exact candidate SHA (85cb4c67, tree 09896243).**
- Evidence: tmp/run-evidence/reval/aws-rebuild-85cb4c67/ (aws-85cb4c6.log + phone slice);
  verdicts amended in tmp/run-evidence/reval/GATE-VERDICTS-2026-09-07.md.
The single next CEO decision is unchanged: the tag (operator-supervised BLE/cellular legs
remain the only open input).

## ADDENDUM — 3-node full-functionality confirmation (2026-09-07 ~20:35Z)
Evidence: tmp/run-evidence/reval/3NODE-FULL-CONFIRM-85cb4c67-2026-09-07.md
- Bidirectional Windows<->Pixel delivery with receipts: PASS (W->P `1c90f7d5` delivered +
  receipt from the phone's chat identity 9a230574; P->W carried via 5 user-sent messages
  in the CLI inbox 15:42-15:49Z plus per-send receipts).
- AWS store-and-forward custody acceptance on the 85cb4c67 binary: PASS — relay_custody
  "Accepted custody 12D3KooWSxdudq... for OFFLINE DESTINATION" at 20:28:13Z (a no-live-route
  message taken into custody and held on the cloud node).
- Mesh health on the same-SHA fleet: PASS — seed-dial steady cadence, relay circuit active,
  ext-addr filtering active, CLI sees Pixel (direct) + AWS (relay) as connected peers.

Mission gates: all PASS except BLE/cellular (operator-supervised, UNVERIFIED).
Single next decision: the CEO tag ruling.
