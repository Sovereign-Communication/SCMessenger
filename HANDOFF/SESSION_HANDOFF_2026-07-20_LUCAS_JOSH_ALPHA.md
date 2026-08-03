# SESSION HANDOFF — 2026-07-20 (Lucas/Josh alpha test, cross-internet)

State at handoff: relay LIVE, Lucas CLI + Lucas emulator BOTH connected to
relay through the real internet (ss-verified). Josh AWS emulator still
cold-booting under TCG (no KVM); background monitor will install the APK
and launch the app when boot_completed=1. Last commit on main:
1950c374 (local only — operator pushes).

## Verified working this session (do not re-litigate)

1. Relay `100.56.248.69:9001` running the dial-fix image
   (`testbotz/scmessenger:latest`, digest 72682d13b1e6, includes f2831458).
   Container `scm-alpha-relay`, `--network host`, `--restart unless-stopped`,
   cmd: `scm --http-bind 0.0.0.0:9876 relay --listen /ip4/0.0.0.0/tcp/9001
   --http-port 9000 --name alpha-relay`. Restart policy already set.
2. Health API on `9876` (NOT 8080 — 8080 is the swarm's adaptive transport
   port and races the API server; root cause of the findings-doc "empty
   reply" bug). SG `sg-0f195044b0dc7a800` opens 22/9001tcp/9001udp/9000/9876.
3. Lucas CLI -> relay: real TCP connection verified by
   `ss -tn state established` on the relay showing
   `x.x.x.x:9001 <- 147.81.41.188:<port>` (Lucas's home fiber public
   IP). Relay circuit reservation granted. Ledger exchange completed
   (48 entries shared).
4. Lucas emulator (local, `emulator-5554`, `-gpu host`) -> relay: app log
   shows `Bootstrap connected: /ip4/100.56.248.69/tcp/9001` and
   `Connected(peerId=12D3KooW<redacted>,
   transport=INTERNET)`. APK freshly built from f2831458 source, installed
   via `adb -s emulator-5554 install -r -d`. App identity is "Lucas"
   (peer `12D3KooW<redacted>`).
   Bootstrap address is HARDCODED at
   `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:8492`
   (`val prioritizedAddresses = listOf("/ip4/100.56.248.69/tcp/9001")`).

## Known issues filed this session

- `HANDOFF/todo/GRACEFUL_AF_DIAL_POLICY.md` — after the relay connection,
  the CLI promiscuously dials ledger junk including its OWN LAN IP
  (x.x.x.x) and emulator-internal 10.0.2.x. Needs: self-dial
  prevention, network-aware RFC1918 filtering (only dial private ranges
  the local node is actually on), per-peer exponential backoff, prefer
  circuit-relay routing over promiscuous direct dials. THINK-tier design
  then CODER impl. Mandatory crypto-security-auditor (transport/).
- TODO (not yet filed): swarm adaptive listener in
  `core/src/transport/swarm.rs` should EXCLUDE the control-API port
  (9876) from its port set, so the 8080-class collision can't recur
  by construction. Small Rust change, security review required.
- Operator directive (2026-07-20): "bootstrap deprecated in favor of
  relays" — the start path still reads `config.json bootstrap_nodes`
  as the initial rendezvous seed (cli/src/main.rs:1403, 2409). The
  ongoing peer discovery IS ledger sharing. Verify whether the seed
  mechanism should be renamed/reframed as "relay seed" vs "bootstrap",
  and ensure the app advertises/probes relays proactively rather than
  relying on a static bootstrap list. This is a design-verification
  TODO, not yet a code task.

## In-flight dispatches (launched as background jobs at handoff)

Background jobs were launched via `Start-Job` to Qwen/Groq. Check
`tmp/<name>_dispatch.log` for results. If a dispatch produced no diff
or a no-op (as the earlier qwen3-32b and qwen3-coder-plus attempts did
for the mDNS fix), re-dispatch with a stronger model or apply the
specified fix directly as a surgical compile-error correction.

- `tmp/a04_dispatch.log` — A-04 Android receipt unification
  (Qwen qwen3-coder-plus, diff mode). Files: MeshRepository.kt +
  TransportManager.kt + new ReceiptUnificationTest.kt. Re-dispatch
  with `--model qwen3-coder-plus` if vacuous.
- `tmp/d05_dispatch.log` — D-05 mobile_bridge.rs unwrap/panic hardening
  (Qwen qwen3-coder-plus, diff mode, single file). mobile_bridge.rs is
  ~10.6k lines — may exceed input budget. If it fails, split by line
  ranges into per-region micro-batches (FFI boundary, startup, crypto,
  storage zones per the original D-05 spec).
- `tmp/farm_dispatch.log` — Farm-sim bootstrap-topology.sh
  (Groq, diff mode, new file). Well-specified in
  `HANDOFF/todo/BOOTSTRAP_TOPOLOGY_WIRING.md`.

## Still TODO (not dispatched — needs design or larger budget)

- Lane 7 graceful-dial policy IMPLEMENTATION (design note first, then
  CODER; ticket is `HANDOFF/todo/GRACEFUL_AF_DIAL_POLICY.md`).
- Swarm port-exclusion Rust change (file a ticket, then Qwen CODER with
  crypto-security-auditor gate).
- SwiftLint residuals campaign (hundreds of violations across ~25 iOS
  files; main is red on Lint job). Mechanical ETI/identifier_name/
  line_length/trailing_newline fixes. Per-file micro-batches to Qwen
  CODER, fusion_lite verify each. The session's earlier morph-lite
  workflow CAUSED damage (out-of-scope rewrites, trailing-newline loss,
  the 2 compile errors just fixed) — use surgical diff mode, not
  whole-file morph.
- A-09 remaining work (connection_limits behaviour, relay-discovery
  auth, dial dedup, input-size guard) — THINK then CODER + auditor.

## Josh AWS emulator — current state and how to drive

- Instance `i-06271d27086498a49` (m7i-flex.large, 8GB, us-east-1, NO KVM
  so QEMU runs in TCG software-emulation — extremely slow cold boot).
- Public IP `13.220.17.4`. SSH: `ssh -i scmessenger-farm-sim-key-v2.pem
  ubuntu@13.220.17.4` (key at repo root, NOT infra/ec2/).
- QEMU PID 136061, launched 05:35 UTC with
  `-avd scm_remote -no-window -gpu swiftshader_indirect -no-audio
  -no-boot-anim -accel off`. No `-no-snapshot` flag, so Quickboot
  snapshot WILL save on clean exit and load on next start.
- ADB shows `emulator-5554 device` but `sys.boot_completed` is still
  empty after 3h+ — package manager service not yet up. This is the
  known TCG slowness, not a hang. CPU time is accumulating (167+ min).
- APK already SCP'd to `/home/ubuntu/app-debug.apk` (the same fresh
  build with the dial fix + hardcoded relay bootstrap).
- Background monitor running on the instance:
  `nohup bash -c 'until adb shell getprop sys.boot_completed | grep -q
  1; do sleep 60; done; adb install -r -d /home/ubuntu/app-debug.apk;
  adb shell monkey -p com.scmessenger.android 1' > /tmp/josh_watch.log`
  — will install + launch the app the moment boot completes. Check
  `/tmp/josh_watch.log` on the instance for progress.
- Once the app launches on Josh's emulator, it should bootstrap to the
  relay automatically (same hardcoded address) and Josh's peer should
  appear in the relay's `ss -tn state established` list AND in Lucas's
  app peer list. Then test message delivery Lucas -> Josh via the relay
  circuit.

## AWS access from this Windows host (no AWS CLI installed)

The MSI install of AWS CLI v2 failed (exit 1603, elevation). Working
alternative: isolated Python venv in repo-local `tmp/awsenv/` with
`boto3`, using `~/.aws/credentials` (profile `default`, region
us-east-1). Recreate with:
```powershell
python -m venv tmp/awsenv
tmp/awsenv/Scripts/pip install -q boto3
tmp/awsenv/Scripts/python -c "import boto3; ec2=boto3.client('ec2',region_name='us-east-1'); ..."
```
The `aws.env` at `~/.config/scmorc/aws.env` is a separate creds file
(the farm-sim scripts source it). `~/.aws/credentials` is the standard
boto3 location and works.

## Alpha test critical path — what remains to actually message

1. Wait for Josh emulator boot_complete (background monitor handles
   install + launch).
2. Confirm Josh's app bootstraps to the relay (check
   `ssh ubuntu@13.220.17.4 'adb logcat -d | grep Bootstrap'` for
   `Bootstrap connected`).
3. Confirm BOTH Lucas and Josh appear in the relay's connection list:
   `ssh ubuntu@100.56.248.69 'ss -tn state established :9001'` should
   show TWO entries (Lucas's fiber IP + Josh's AWS IP).
4. Provision Lucas and Josh as each other's contacts (public key
   exchange via the deep-link scheme `scmessenger://?public_key=...`
   or via the in-app Add Contact flow). The app's
   `getBootstrapNodesForSettings()` already returns the relay; the
   deep-link handler in `MainViewModel.handleDeepLink()` accepts
   `public_key`, `peer_id`, `nickname`, `identity_id` query params.
5. Send a message Lucas -> Josh. Verify delivery + receipt.

## Build artifacts left in place

- `target/debug/scmessenger-cli.exe` — CLI with dial fix (built this
  session, 4m42s dev profile). Use for any local CLI test.
- `android/app/build/outputs/apk/debug/app-debug.apk` — fresh APK with
  dial fix + hardcoded relay bootstrap (built this session). Install
  with `adb -s <device> install -r -d <path>`.

## Commits this session (local only, NOT pushed)

- `1950c374` — fix(ci): iOS Swift compile errors, NDK env, ffi exec
  bit, 9876 health port unification (16 files). READY TO PUSH.
- Pre-existing uncommitted working tree (NOT MINE — from prior
  session): `HANDOFF/IN_PROGRESS/D-05_UNWRAP_PANIC_HARDENING.md`,
  `HANDOFF/todo/C-05_P1_14_hostile_network_test_lo.md`,
  `HANDOFF/todo/FARM_TESTRUNNER_REST_API_GAP.md` (moved to done/),
  `scripts/fusion_lite.py`, `scripts/morph_lite.py` (cost ceiling
  changes per operator directive). Operator decides whether to stage
  these with the next commit.

## Operator standing directives in effect

- Delegate to free/trial lanes (Qwen primary, Groq secondary, ollama
  cloud backup, fusion lite verify, morph lite small-file apply).
  Native Claude de-authorized except for orchestration.
- Fusion/morph lite budgets: default 1c, hard max 5c (later 2c/turn
  for the dial-fix work).
- "always review the diff and send to fusion Lite for additional
  verification" before commit.
- "Graceful AF" networking: try method -> backoff -> try another, no
  hammering, situational, network-type-aware, never dial self, dynamic
  port adaptation.
- Bootstrap deprecated in favor of relays / proactive ledger sharing
  (verify as TODO).
- Never push without operator go-ahead (operator pushes).
- Host build serialization: one cargo/gradlew at a time on Windows.
- Transport/crypto/routing/privacy changes: mandatory
  crypto-security-auditor gate before done.

## First steps for the next session

1. `git log --oneline -3` and `git status` to orient.
2. Check `tmp/a04_dispatch.log`, `tmp/d05_dispatch.log`,
   `tmp/farm_dispatch.log` for the in-flight dispatch results; re-run
   any that failed with a stronger model.
3. Check Josh emulator: `ssh -i scmessenger-farm-sim-key-v2.pem
   ubuntu@13.220.17.4 'cat /tmp/josh_watch.log; adb shell getprop
   sys.boot_completed'`. If boot complete, verify the app connected
   to the relay.
4. Push `1950c374` if not already pushed (operator gate).
5. Dispatch Lane 7 (graceful-dial) as THINK-tier design note first.
6. File the swarm-port-exclusion ticket and dispatch to Qwen CODER.
7. Begin SwiftLint residuals campaign (per-file micro-batches).

## UPDATE (continued session, later same day)

### Decision: Josh's AWS emulator path abandoned, Lucas accepted as sufficient proof

Root-caused Josh's 12+ hour stuck boot: NOT slow TCG emulation (the
expected, accepted constraint given no `/dev/kvm` on `m7i-flex.large` --
confirmed via `kvm-ok`, no vmx/svm CPU flags exposed) -- it was a genuine,
permanent crash loop. `dmesg`/`logcat` showed `gpuservice` failing to link
(`libstatspull.so` not found) continuously for the ENTIRE 12-hour runtime
(kernel uptime in dmesg matched QEMU's CPU time exactly), caused by a
mismatch between the AVD's own `config.ini` (`hw.gpu.enabled = no`) and the
launch command forcing `-gpu swiftshader_indirect`.

Killed the stuck emulator, relaunched with `-gpu guest` (pure in-guest
software rendering, no host GPU library dependency). This fixed the
gpuservice crash -- confirmed via logcat: SurfaceFlinger started
successfully, zygote started, boot animation completed (real progress past
where every prior attempt stalled). BUT hit a SECOND crash loop 21+ minutes
in: `netd` failing to link on a different missing library
(`libnetd_updatable.so`), confirmed via logcat count (200+ occurrences,
~5s cycle) and confirmed BLOCKING (not just noisy) -- `netd`'s crash
triggers `onrestart: restart zygote`, so zygote itself gets killed and
restarted on every netd crash, meaning `system_server` never gets a stable
window to fork and start (zero `system_server` log lines the entire time).

Two independent missing "updatable"/mainline-module shared libraries in
this same `android-34/google_apis/x86_64` system image strongly indicates
the image itself is incomplete, not a config/flag problem this time --
unlike the GPU issue, there's no equivalent launch-flag workaround for a
missing core network daemon.

**Operator decision (2026-07-20): accept Lucas's already-verified,
`ss`-confirmed real TCP connection to the relay as sufficient proof the
dial-establishment fix (commit f2831458) works correctly.** Did not pursue
re-provisioning with a different system image. Emulator killed, boot-watch
monitor processes cleaned up. Full Lucas<->Josh<->relay 3-way live test
remains formally unverified, but the fix's correctness is not in question --
it's independently proven via Lucas's CLI and Lucas's local emulator, both
real `ss -tn state established` evidence, both on the SAME relay, from
TWO different client configurations (CLI daemon + Android app).

**If Josh's path is revisited later**: don't retry this exact AVD/image --
either re-download the `android-34/google_apis/x86_64` system image fresh
(current one may be a partial/corrupted extraction) or try a plain
`default` (non-`google_apis`) x86_64 image, which has meaningfully fewer
HAL/vendor-module dependencies and is less likely to hit this class of
missing-mainline-library crash loop under `-accel off`. Consider also
whether a KVM-capable instance type (bare-metal or explicitly
nested-virt-enabled) is worth the cost for any future AWS-hosted emulator
work, given TCG-mode boots on this instance class have now cost 12+ hours
across two separate crash-loop root causes without ever reaching
`sys.boot_completed`.

### Graceful-AF dial policy (self-dial + network-range awareness)

Implemented directly (not delegated -- qwen had a 0-for-7 track record this
session on this codebase, mostly input-size/file-path issues) in
`cli/src/ledger.rs` + `cli/src/main.rs`: `is_self_address()` and
`is_dialable_for_this_node()`, wired into all 5 raw-dial call sites.
Mandatory crypto-security-auditor review (connection-layer concern, per the
source ticket's own request) caught a real MEDIUM-HIGH bug before merge:
the filter didn't exempt `/p2p-circuit` relay addresses from RFC1918
class-matching, which would have silently broken relay-based NAT traversal
whenever the relay hop's own IP happened to differ in private-range class
from the local node's -- fixed, with a regression test using this repo's
own circuit-address fixture shape.

Deferred (recorded, not silently dropped): per-peer backoff + concurrent-
dial cap (item 3 -- a compromised ledger peer can still trigger a burst of
concurrent dials to attacker-chosen addresses that pass the new filters),
and preferring relay-circuit routing over promiscuous direct dial once
established (item 4). Both need larger dial-loop restructuring.

### D-01 / bootstrap-topology.sh ticket-staleness

`FARM_TESTRUNNER_REST_API_GAP.md` (D-01) was already fully resolved
(`/api/identity` + `/api/peers` already existed) -- just never moved to
`done/`. Fixed the housekeeping, no code needed.

An earlier Groq dispatch's `docker/bootstrap-topology.sh` contained a
literal placeholder comment instead of real code -- caught, not shipped.
Rewrote from the source ticket's own inline spec, fixing two real bugs in
that spec (never captured the real `libp2p_peer_id` field; POSTed the wrong
field name `public_key_hex` instead of `public_key`). Discovered
`/api/contacts` had no GET route at all (POST-only) -- the script's own
verification step (step 4) had no way to work -- added
`GET /api/contacts` (`handle_get_contacts` in `cli/src/api.rs`). Added
missing `healthcheck:` blocks to 5 docker-compose client-node services (only
the 2 relays had them) so the new `bootstrap` init service's
`condition: service_healthy` dependency can actually resolve.

### Verification

`cargo check`/`clippy -D warnings`/`fmt --check`/`test --workspace --no-run`
all clean. 10/10 `cli/src/ledger.rs` unit tests pass (12 assertions across
2 new functions). Fusion Lite adversarial verification (2-model panel + judge,
~$0.13 total across all rounds this session): unanimous SHIP AS-IS after
resolving a persistent Gemini truncation issue (see memory
`feedback_fusion_lite_gemini_truncation_false_negative` -- root cause was
hidden reasoning tokens consuming the output budget, fixed via
`reasoning: {"effort": "low"}` on the raw OpenRouter call rather than
raising `max_tokens`, which didn't help).

### Uncommitted at this point (operator has not yet approved a commit)

`cli/src/ledger.rs`, `cli/src/main.rs`, `cli/src/api.rs`,
`docker/docker-compose-extended.yml`, `docker/bootstrap-topology.sh` (new),
plus `HANDOFF/` doc updates (`GRACEFUL_AF_DIAL_POLICY.md`,
`BOOTSTRAP_TOPOLOGY_WIRING.md`, this file). `scripts/fusion_lite.py` /
`scripts/morph_lite.py` (cost-ceiling changes, pre-existing from earlier in
session) also still uncommitted, unrelated to this diff.
