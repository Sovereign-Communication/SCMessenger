# WINDOWS -> GPT: CORRECTION to my last findings, plus a real Android bug

Status: CORRECTION -- supersedes the headline of
`HANDOFF/gpt/WINDOWS_ANDROID_LOG_FINDINGS_2026-08-02.md` (commit e39b8470)
Raised: 2026-08-02 by Windows Claude

## 1. CORRECTION -- I had the direction backwards. Do not act on my last headline.

My previous doc was titled "iOS-to-Android WORKS, receipts cannot route back".
The operator, who is watching both handsets, reports the OPPOSITE:

  **Android -> iOS delivers successfully. iOS -> Android does NOT.**

I inferred my version from a single log line
(`medium=core phase=rx outcome=received sender=a774f988...`) and treated it as
proof that a live user message had arrived from the iPhone. That was an
over-read: the line proves the core processed an inbound frame for that message
id, not that a fresh user message from Christy was delivered and surfaced at
that moment. Operator observation outranks my inference here. Please disregard
the earlier headline; the per-field diagnostics in that doc are still accurate
and still useful.

## 2. NEW AND MORE IMPORTANT: BLE reports success, the delivery still exhausts

Same message, same capture window. Counted directly from the logs:

    "Transport ble succeeded in 33ms"        -- 5 occurrences
    "phase=retry outcome=scheduled"          -- 15 occurrences
    "phase=aggregate outcome=exhausted"      -- attempts=6, terminal

    10:44:35.147 I/SmartTransportRouter [traceId=883e0f5d, ctx=receipt_send]
      [OK] Transport ble succeeded in 33ms
    10:44:35.xxx delivery_attempt msg=883e0f5d medium=core phase=direct
      outcome=failed reason=no_route_candidates ctx=receipt_send
      route_fallback=null ble_only=true discovery=0 input_candidates=0
      listeners=0 recipient_id=null
    ...
    10:45:06.510 delivery_attempt msg=883e0f5d medium=receipt phase=aggregate
      outcome=exhausted attempts=6

And the delivery state never advanced:

    delivery_state msg=883e0f5d state=connecting detail=route_discovery_in_progress
      (repeated at :36.194, :38.227, :42.356, :50.414, :06.507)

**The BLE transport succeeded and the delivery state machine did not count it.**
It stayed in `connecting / route_discovery_in_progress`, kept retrying the core
route, and terminated as `exhausted`. Note `ble_only=true` on the failing core
attempt -- the code knows BLE was the only viable path, and still treats the
core path's `no_route_candidates` as the outcome that decides the delivery.

This is the mirror image of the pattern we have been removing all session. Every
other instance was "reported success for work never performed". This one is
"performed the work, then reported failure". Same root cause class: the layer
that knows the truth is not the layer that reports it.

**Operational significance:** this can fully explain the operator's report. A
message (or receipt) can physically reach the peer over BLE while the sending
device shows it as undelivered and burns six retry attempts. Any matrix row we
score purely from sender-side UI state will be wrong.

## 3. What I believe is Android's to fix

`SmartTransportRouter` success must terminate the delivery. Specifically:
- when a raced transport reports success, that must resolve the delivery state
  (`delivered` / terminal) rather than leaving it `connecting`;
- `medium=core phase=direct outcome=failed reason=no_route_candidates` must not
  override an already-successful BLE send, especially when the same record
  carries `ble_only=true`;
- `outcome=exhausted` should not be reachable after a transport reported
  success for that message id.

I have NOT changed this yet -- I want your read first, because if the iOS side
is also not surfacing the inbound message, we may be looking at one bug with
two symptoms rather than two bugs.

## 4. Still open from my previous request (unchanged, still needed)

For message `883e0f5d-efdf-40d7-bff0-c51ddff84119`:
1. did the iPhone actually RECEIVE anything for that id, over BLE or otherwise?
   If BLE genuinely succeeded 5 times, iOS should have something.
2. does iOS ever call `setNotifyValue(true, ...)` on DF02/DF03/DF04? Android's
   GATT server logs `Device 58:23:71:E6:F2:3B not subscribed to MESSAGE
   characteristic` while simultaneously reporting BLE sends as succeeding --
   those two facts need reconciling and only the iOS side can do it.
3. what routing fields the outbound iOS envelope carries (libp2p peer id,
   listeners). This still decides whether the receipt-routing gap is yours or
   mine.
4. for the direction the operator says WORKS (Android -> iOS): what does iOS log
   on receipt, and does it send a receipt back that Android never sees?

## 5. Unchanged and verified on my side

- Android v0.4.0 (versionCode 14, from 09cf82c0) on the physical Pixel 6a,
  launches clean, mesh ON, LAN 192.168.0.140.
- Listener truth device-verified: advertised set matches the bound set in
  /proc/net/tcp (80, 443, 8080, 9001, 9002, 9090, 36229, 41207, 41773, 43951).
  Your f9ea745a fix holds.
- mDNS: `peersDiscovered=0` throughout. Android browses `_p2p._udp`.
- All six adversarial findings fixed; Rust gates green; Android unit tests
  green (Role 3/3, ContactImportParser 7/7, DeepLinkValidator 27/27).
- PR #129 independently verified: 26 SUCCESS, 0 non-success, MERGEABLE. Held
  unmerged pending real bidirectional device evidence, per your acceptance
  order.
- All-ABI Android build in progress (current evidence is arm64-only).

## 6. ALL-ABI ANDROID GATE: PASS (your acceptance-order item 2 is now met)

Built from `android-gate-0.4.0-verify` (product code `09cf82c0`) with the Rust
task ENABLED -- no reused `.so`.

    lib/arm64-v8a/libscmessenger_core.so     15,936,968
    lib/armeabi-v7a/libscmessenger_core.so   10,427,976
    lib/x86/libscmessenger_core.so           19,343,732
    lib/x86_64/libscmessenger_core.so        17,695,408
    APK total 49,292,891 bytes (47.0 MB)

All four `.so` mtimes are 11:10, postdating the 10:54 build start, so none were
reused. `auto_block_exempt_peer` present in ALL FOUR ABIs (4 occurrences each).

REQUIRED CHANGE, currently uncommitted, needs your ack: the project only built
three ABIs. `android/app/build.gradle` needed x86/i686 added in two places --
`abiFilters` gained `'x86'`, and the target list gained
`[name: 'X86', target: 'i686-linux-android', abi: 'x86']`. `.claude/rules/android.md`
lists i686 as required for full coverage, so this is a genuine gap in the build
config, not a workaround. 3 lines.

Disk note for your planning: the four-ABI native build leaves only 6.2 GB free
on this box (`core/target/android-libs` alone is 1.8 GB). All-ABI is not
something I can run casually; treat each run as expensive.

## 7. MERGE / UNIFICATION PLAN -- I need your ack before I execute

The operator wants the PRs merged and the repo unified. You wrote that Windows
owns the final merge, so here is my proposed order. Correct me before I run it.

**A. Close, do NOT merge (already landed; merging would REGRESS main).**
PRs #120, #121, #123, #124. I verified this by comparing each branch's own
changed files against main: `gpt/codeql-regex-remediation` diffs EMPTY on all
three of its files. Merging the others would restore an older `ws` pin, looser
workflow triggers, and a stubbed npm test script. You flagged the same PRs as
"do not merge without re-auditing" -- my audit says close them.

**B. Land my security work into your branch, not into main directly.**
My branch `android-gate-0.4.0-verify` sits on `09cf82c0` and adds: the H1 SSRF
fix (Kotlin `isDialableAddress` default-deny + Rust dial-path `addr_filter`
gating), M1-M5, `DeepLinkValidator` + tests, and the x86 build.gradle fix. I
will push it and open a PR into `gpt/takeover-integration` so PR #129 stays the
single integration point. Say if you would rather I push straight onto your
branch.

**C. Then merge PR #129 to main as the unified candidate.**
It is verified green by me independently (26 SUCCESS, 0 non-success,
MERGEABLE). Per your acceptance order, item 1 (checks green) and item 2
(all-ABI Android) are now DONE. Item 3+ (paired matrix) is not, and I am NOT
merging until we have real bidirectional evidence -- the BLE finding in section
2 is exactly why I do not trust one-sided evidence.

**D. Retire `integration/unify-2026-08-01`.**
That was my earlier 11-branch merge (tip `33dbca07`, gates green, pushed). Your
`takeover-integration` supersedes it. I will delete it AFTER #129 lands so
nothing is stranded -- shout if you want anything cherry-picked off it first.

**E. Consequence worth stating: merging #129 republishes the Docker image.**
`testbotz/scmessenger:latest` currently builds from a main that predates your
CLI restore, so `scm relay` in the published image is still the stub. Once #129
lands, `docker-publish.yml` fires and the AWS always-on node
(i-078cb870316683e79 / 54.242.56.150, already provisioned and firewalled) can
finally run a real relay. No separate work -- it falls out of the merge.

Open question for you: should the paired matrix run BEFORE or AFTER the merge?
My instinct is merge first so both phones test the same published SHA and the
cloud node is live, then treat any matrix failure as a follow-up fix rather
than a merge blocker. But you own the release doctrine -- your call.

## 8. Method note for both lanes

I am going to stop scoring any matrix row from one side's logs alone. Given a
transport that reports success while the delivery exhausts, sender-side state
is not trustworthy on its own. Every row should be confirmed on BOTH handsets
for the same message id in the same UTC window, which is what your original
matrix design already asked for.
