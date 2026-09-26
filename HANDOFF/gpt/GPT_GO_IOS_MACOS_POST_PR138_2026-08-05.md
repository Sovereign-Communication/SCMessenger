# GPT: GO -- iOS + macOS install/update (post PR-136/137/138)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Date:** 2026-08-05
**Priority:** HIGH -- fleet rollout gate for the v0.4.0 five-node test
**From:** Orchestrator (Windows/Qwen)
**Status:** GO -- the wait from GPT_WAIT_FOR_PR136_BEFORE_FINAL_IOS_MACOS_BUILD_2026-08-04.md is OVER
**Budget note:** your lane is API-constrained for ~3 more days. This packet
is self-contained; execute top to bottom, no exploration needed.

---

## TL;DR

main is now `6b2573fa` with everything you waited for merged:
- PR #136 identity canonicalization + block gate (what you were waiting on)
- PR #137 transport liveness: zombie-transport reconciliation + auto-reconnect
  of stale peers (fixes the field-observed mid-session message halt)
- PR #138 Android mDNS hardening (Android-only; no iOS/macOS surface change)

Update BOTH your nodes (iOS app on Christy's iPhone + macOS CLI node) to
this HEAD. **This rollout is an IN-PLACE UPDATE TEST: keep existing
identities, contacts, and message history. Do NOT uninstall, do NOT wipe
identity.json/contacts.db, do NOT re-pair.** Run-2 identities were already
created post-PR-136 (canonical), and no PR since has changed the identity
format, so in-place is safe. This supersedes the fresh-install/wipe steps in
the 2026-08-04 wait packet.

---

## Steps

### 1. Sync and confirm HEAD

```bash
git fetch origin main && git checkout main && git pull origin main
git log --oneline -1        # expect: 6b2573fa Merge pull request #138 ...
grep -n "public_key_hex" core/src/iron_core.rs | head -5   # PR #136 present
```

### 2. macOS CLI node (in-place)

- Same checkout/build flow you used for run 2:
  `cargo build -p scmessenger-cli`
- Stop the running node process if any, start the new binary the same way
  (same data directory, same flags). Do not delete or move any data files.
- Expected startup log line carries the provenance stamp; it MUST read
  `0.4.0 (6b2573fa ...)`. Record the actual line.

### 3. iOS app on Christy's iPhone (in-place)

- Use your run-2 Xcode workflow (same scheme/signing), build and Run onto
  the device. With the same bundle id + signing this upgrades in place and
  preserves app data. Do not delete the app first.
- If the build fails on Swift binding mismatches only, regenerate bindings
  (`gen_swift`) once and rebuild -- PR #137 touched core transport code, so
  a binding regen is the expected-occasionally fallback; anything beyond
  that is a BLOCKED report, not a fix-it-yourself quest.
- After launch, confirm the app shows the existing identity/contacts/history.

### 4. Verify identity still canonical (both nodes)

Outbound sender_id / own identity must be a 64-hex-char public key (not a
blake3 hash). One log line or UI screenshot-worth of evidence per node.

### 5. Bootstrap / relay address

POLICY UNCHANGED (2026-08-04 directive): hardcoded IPs are ephemeral. The
AWS always-on node is being rebuilt by the orchestrator and its public IP
WILL change (no Elastic IP).
- If your nodes already have the relay configured and messaging still
  works after this update, change nothing.
- If the relay stops resolving after the orchestrator's rebuild, read the
  CURRENT address from `HANDOFF/gpt/AWS_RELAY_CURRENT_ADDRESS.md` (the
  orchestrator updates that single file after every rebuild -- read it
  fresh at use time, never copy an IP from any older doc), then update
  your nodes' bootstrap to `/ip4/<that IP>/tcp/9001`.

### 6. Report

Write `HANDOFF/gpt/IOS_MACOS_POST138_READY_2026-08-05.md` with REAL
evidence: git HEAD built, provenance stamp lines from both nodes, identity
canonical check results, data-preservation confirmation (contacts/history
still present after upgrade). Commit it on your own `gpt/*` branch and
push/PR per MAC LANE rules (AGENTS.md capability class MAC LANE).

If blocked at any step: stop, and report in 10 lines or fewer --
`RESULT: BLOCKED`, what failed, exact error, what you need. Do not burn
API budget improvising around it.

---

## Why in-place this time (context, one paragraph)

The operator is validating the UPDATE path as part of the v0.4.0 five-node
gate: all five fleet nodes must move from an older build to `6b2573fa`
without losing identity or messages, because real users will update this
way. Android is getting `adb install -r` over the CI debug APK; the AWS
node gets an image-level rebuild (no SSH access exists). Your two nodes are
the Xcode/cargo in-place cells. Plan doc if you want the full picture:
`HANDOFF/plans/V040_V050_FIVE_NODE_GATE_PLAN_2026-08-05.md` -- reading it
is optional; this packet is sufficient.
