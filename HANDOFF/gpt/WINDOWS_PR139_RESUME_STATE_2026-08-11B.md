# Windows lane -- PR 139 resume state (authoritative, supersedes 2026-08-11A)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Written 2026-08-11 at operator request. **Read this FIRST on resume.** It replaces
`WINDOWS_PR139_RESUME_STATE_2026-08-11.md`, whose headline blocker was already stale.

## OPERATING MODE -- read before doing anything

Operator directives now in force, in priority order:

1. **EXTREME LOW TOKEN.** Native Anthropic spend is to be minimal. Do not re-derive
   state, do not re-read large docs, do not poll. Batch every shell check.
2. **DEFER TO GPT-MAC.** The Mac lane has fresh API quota and OWNS all analysis,
   implementation and review. Windows is SECONDARY.
3. **Windows does DEPLOYS ONLY** -- adb to the Pixel, ssh/docker to the AWS relay,
   local builds/pins, light verification, log pulls. Nothing else.
4. **Offload to qwenpaid** when it has quota (see lane status below).
5. Escalate decisions to the GPT-MAC lane, not to the operator.

## Lane status (verified 2026-08-11 ~17:20Z)

- **qwenpaid: QUOTA EXHAUSTED.** `Your token-plan 1-week quota has been exhausted. The
  quota will reset at 08-12 04:44:00 UTC.` Key is VALID (`sk-sp-`, 115 chars), endpoint
  `https://token-plan.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1/chat/completions`
  is correct. **Retry after 2026-08-12 04:44 UTC**, then make it primary per directive 4.
- Two traps cost a dispatch each, do not repeat them:
  - The plan supports **`qwen3.8-max`**, NOT `qwen3.8-max-preview`. The repo's
    `dispatch_dial.py`/ledger still name the `-preview` id; it hangs to timeout.
    Supported ids: `qwen3.8-max`, `qwen3.7-plus`, `qwen3.7-max`, `qwen3.6-flash`,
    `deepseek-v4-pro`, `deepseek-v4-flash-0731`, `glm-5.2`.
  - `~/.config/scmorc/qwenpaid.env` holds TWO keys. Parse `QWEN_PAID_API_KEY=` by name.
    A generic "last long token" grep picks the OpenRouter key and yields a misleading
    `invalid_api_key`.
- **`qwen` (DashScope free) WORKS** -- `qwen3-coder-plus-2025-09-23` completed a real
  dispatch today. Use this lane while qwenpaid is dry.
- `agy` is available and the operator has offered it to the Mac lane too.

## Node parity -- the gate's "one verified SHA" criterion

| Node | SHA | State |
|---|---|---|
| Windows CLI | **`053fd137`** | gen 8, PID 25268, pin sha256 `e9501c5c67fe867fb466f5f5ca7dd5c24e64c008b3d794b059686961abc72f7f`, binary `soak/bin/scmessenger-cli-053fd137.exe` |
| AWS relay | **`053fd137`** | container `testbotz/scmessenger:sha-053fd13`, PeerId `12D3KooWPJK6...` PRESERVED across the swap |
| Android | APK built at head + installed | isolated, see blocker below |
| Mac CLI | -- | intermittent, drops repeatedly |
| iOS | -- | unconfirmed |

Windows and the relay are the first two nodes on ONE verified SHA. An earlier Windows pin
at `a74ed978` was rejected on the gate's literal one-SHA criterion and rebuilt.

## THE BLOCKER IS NO LONGER VERSION DRIFT -- IT IS MESH FORMATION

- **Android reports `Mesh Stats: 0 peers (Core), 0 full, 0 headless`** while actively
  running mDNS discovery and listening on ~80 addresses. Its PeerId is
  `12D3KooWNnPi9wqUJ7Jypj6g4jHmW2PUTmynUs9sJY1h6SQbjLrG` (confirmed from its own
  `mDNS: ignoring self-resolved service` line -- **this peer is the ANDROID device**, not
  an iOS node; earlier docs are ambiguous on that).
- Windows sees exactly one peer, the relay.
- Unchased lead: the Android device has a `10.8.223.228` interface alongside LAN
  `192.168.0.135`, which looks like a VPN and may be why discovery resolves but
  connections do not establish. **Assigned to GPT-MAC.**

## Open defects (ALL assigned to GPT-MAC)

1. **IDENTITY UNIFICATION -- operator says TOP priority**, must be "complete and working
   perfectly". Reproduction case: the Mac node answered a delivered message on the
   rostered `WNC...` identity (`7afda32b`, receipt returned) while its `/identity`
   reported suffix `jUUWF`. One node, two identities. Prior work exists: start from
   `HANDOFF/done/P0_IDENTITY_002_Unified_Infallible_ID_Strategy.md` and
   `HANDOFF/done/BATCH_S3_T2_BLE_IDENTITY_HANDSHAKE.md`. Do NOT restart from scratch.
2. **Receipts never converge for offline-destination messages, no catch-up on
   reconnect.** 4 messages to the Mac peer: `736a6865` (online) DELIVERED, `c7f57d97`
   (offline) NOT, `e37ad13f` (offline) NOT, `7afda32b` (online) DELIVERED. The two
   failures each logged 4 `outbox_enqueue` + 3 `outbox_retry_attempt`, custody accepted,
   relayed via Android at 09:59:08Z, still nothing after the peer returned. Node-wide 22
   `outbox_reconnect_detected` / 22 `outbox_retry_attempt`. **Biggest single gate risk:
   any peer flap in the one-hour window silently destroys the ACK evidence.**
3. Android `acked_without_receipt_protection` held-message backlog -- recheck whether it
   clears now the device carries `c93dfec5`.

## Corrections to the record (do not re-derive)

- **CI IS AVAILABLE.** The claim in PR 139 that Actions runners are gone is FALSE. A push
  to the integration branch triggers Cross, Mobile, CI, Lint, Desktop CI and iOS Build &
  Test; Lint and Auto Label pass. This unblocks Mac's iOS provenance and the Android
  validation they lack a JDK for.
- Repository Hygiene fails ONLY on pre-existing trailing whitespace in
  `scripts/README.md`. Not from any recent commit.
- **`main` LACKS `c93dfec5`.** Docker `latest` is built from main, so pulling `latest`
  could never have fixed the relay. Build from the integration branch:
  `gh workflow run "Docker Publish" --ref tracking/pre-v040-tag-work`. Tagging is safe --
  `latest` is gated on `is_default_branch`, so only `sha-<commit>` and a branch tag move.
- Relay SSH WORKS with `~/.ssh/scm-node-key.pem`. The `AWS_RELAY_REBUILD_2026-08-04.md`
  claim that no local `.pem` exists is STALE. Relay identity lives in
  `/opt/scm-relay-data`, NOT in the container-only `config.json` (settings only), so a
  container swap preserves the PeerId.
- The inbox bridge silently swallowed 98 inbound messages between 04:55Z and 09:48Z.
  Fixed in `053fd137`. If a message seems unanswered in that window, it was lost.

## Branch contract -- AGREED, in force

GPT-MAC CONCURRED. Mac uses `mac/pr139-ble-parity`; Windows uses
`windows/pr139-ble-parity` (created and pushed). `tracking/pre-v040-tag-work` is the PR
139 integration head and takes **merges only** for application code; `HANDOFF/` and
`docs/` may land directly. GPT-MAC **OBJECTS** to merging
`android/pr139-transport-durability` until Windows supplies the Android build plus a
scoped review and Mac independently validates. That objection is ACCEPTED -- do not merge.

## Work in progress

- `tmp/win_ble_recipient_scope_ACCEPTED_pending_gate.md` -- an ACCEPTED worker diff for
  Windows BLE packet 5 (recipient-scoped GATT notifications: moves to
  `NotifyValueForSubscribedClientAsync`, zero-match and multi-match are distinct hard
  failures, never a broadcast fallback). **Not applied, not gated.** GPT-MAC offered to
  review it; send it to them rather than self-certifying. A first attempt was REJECTED for
  silently re-keying `REASSEMBLY_BUFFERS` from `String` to `u128`; the prompt at
  `tmp/win_ble_recipient_scope.prompt.md` now forbids that explicitly.
- `scripts/test_inbox_bridge_routing.py` is UNCOMMITTED and must stay that way until
  sanitized: it embeds the operator's REAL `identity_id` and `public_key` as fixtures and
  this repo is PUBLIC. 52/52 tests pass. Replace with synthetic 64-hex fixtures first.

## Disk (this host is chronically near-full)

Recovered from 2.0 GB to ~14 GB free. Reclaimed: `target/aarch64-linux-android` (owed
regeneration before the next Android NDK build -- it was regenerated once already),
`target/release`, the Ollama runtime (reinstallable), the cargo `.crate` cache, and
~12 GB of expanded Claude VM images under
`AppData/Roaming/Claude/vm_bundles/claudevm.bundle` (`rootfs.vhdx`, `sessiondata.vhdx`;
the `.zst` sources were KEPT and they re-expand on next launch).

Operator standing authorization: anything rebuildable or backed by GitHub may be deleted,
SCM user scope, `target/` especially; **do not ask, escalate to GPT-MAC**. The repo
preflight guard blocks recursive deletes outside `tmp/`, `target/` and the scratchpad --
`SCM_ALLOW_DESTRUCTIVE=1` is the documented override.

NOT deleted, and deliberately left for a GPT-MAC decision: stale worktrees
`scm-review-8621a4b5` (8.2 GB, detached HEAD), `scm-winlane` (1.8 GB),
`scm-android-gate` (917 MB, holds unmerged `android/pr139-transport-durability`),
`SCMessenger-w1` (1 uncommitted file). All clean except the last. Not obviously pushed,
so deleting them risks unrecoverable loss in a shared checkout.

## Resume order

1. Read the latest PR 139 comments for GPT-MAC's reply. That is the reliable channel.
2. If after 2026-08-12 04:44 UTC, re-probe qwenpaid with `qwen3.6-flash` and make it
   primary. Otherwise use the `qwen` DashScope lane.
3. Do NOT start the one-hour clock. The mesh does not form (Android at 0 peers).
4. Windows actions only: re-verify node/relay `/version`, redeploy if a new head lands,
   pull logs on request.
