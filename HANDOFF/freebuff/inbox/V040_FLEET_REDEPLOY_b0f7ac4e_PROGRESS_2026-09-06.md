# V040 3-NODE FLEET REDEPLOY TO b0f7ac4e -- PROGRESS + ONE BLOCKER (2026-09-06 ~02:15Z)

Task: V040_CEO_APPROVAL_PUSH_276_TO_FULL_3NODE_GREEN_2026-09-04.md
Type: DONE (steps 4-5) / BLOCKED (step 6-7 Pixel leg)

## DONE this pass (all evidence-backed, commands run live)

1. **PR #276 merged** to `cto/v040-candidate-2026-09-02` via squash: candidate tip is now
   **b0f7ac4eb2551dd3d6b7d28a308c1765c824007e** (tree 1f61d390, byte-identical to reviewed
   branch head 6359f661). Merge logged in `V040_MERGE_EXECUTION_LOG_2026-09-03.md`.
2. **#272 verdicts re-pinned** at the new head:
   `HANDOFF/review/V040_CANDIDATE_272_VERDICT_REPIN_b0f7ac4e_2026-09-06.md`
   (ancestor proof e97c3f82 -> b0f7ac4e; FLAG-5 evidence items re-verified line-by-line at
   b0f7ac4e: listen_port_from_bound_addr UDP/QUIC rejection @ swarm.rs:466,
   sync_external_address /tcp-only @ :477, observation.rs ephemeral guards intact).
3. **Windows CLI leg LIVE at b0f7ac4e**: local release build, provenance
   `0.4.0 (b0f7ac4e:candidate-b0f7ac4e:1788657613)`, sha256
   0eef5b2be34dc4a9cd650ac43279c04f90af0e6245cf5b8a6ad5753fdb2a5803, staged in
   `.codebuff_deploy/wincli-b0f7ac4e/`. Old PID 19072 (e97c3f82 exe) killed; new PID 19476
   launched 02:03:08Z, log `.codebuff_deploy/windows/wincli-b0f7ac4e-20260905T160308.log`.
   OWN_IDENTITY 12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw; **Connected to AWS
   12D3KooWGvCWJ... via /ip4/3.91.5.1/tcp/9001 at 02:03:11Z** (two paths). Control API
   127.0.0.1:9876 responds; custody_audit_count 4875.
4. **AWS leg LIVE at b0f7ac4e**: docker-publish workflow_dispatch run 34004561704
   (ref cto/v040-candidate-2026-09-02) completed success -> image
   `testbotz/scmessenger:sha-b0f7ac4` pulled on the node. Container swap at ~02:01Z:
   stopped/removed c41cfcc2368c (sha-e97c3f8), started **b390e822134b** with the SAME
   mount `-v /opt/scm-relay-data:/data`, host network, restart=unless-stopped, same env.
   Boot log proves provenance: `CLI Version: 0.4.0 (b0f7ac4eb2551dd3d6b7d28a308c1765c824007e)`.
   Relay data dir preserved (logs/outbox/storage intact); /api/diagnostics responds
   (custody_audit_count 0 post-restart on the new boot's counters, history 9 msgs).
   Old rollback instance i-0b735c4f26aea42ed remains STOPPED, untouched.

## BLOCKED -- Pixel leg (needs operator, 30 seconds)

The Pixel 6a is **not visible over adb** (`adb devices` empty after daemon restart +
reconnect probe; USB cable/power state or authorization prompt is the usual cause).
Everything else is staged and waiting:

- APK: `.codebuff_deploy/pixel-apk-b0f7ac4e/app-debug.apk` (CI artifact from run
  34000858468, built at 6359f661 whose tree == b0f7ac4e; versionCode 14, 0.4.0,
  sha256 55c0a321a36be2ed8d34a7ea8a30ea317d9cce53dee4f50d50c5c046b56804d0).

**Operator request:** plug in / wake the Pixel and accept any USB-debugging prompt, then
say "go". I will run: uninstall old build -> `adb install -r` this APK -> launch app once ->
verify identity + listeners -> then run the full same-SHA validation:
Windows->Pixel Text **displays ON-DEVICE** + AWS-relayed hop, evidence under
`tmp/run-evidence/b0f7ac4e-3node-20260906/`.

## After the Pixel leg

Remaining: the 3-node validation gate itself (the RCA's own gate: on-device Text display,
not receipt-only), contact-canonicalization disposition IF display fails on the
176109b9/b9fe29c5 key mismatch, then the tag-readiness table. Tag decision stays with you.
