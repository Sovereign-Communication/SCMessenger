# Alpha Test Setup: Lucas <-> Josh Cross-Internet Messaging

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Relay deploying (2026-07-18), instructions ready for both sides
Purpose: Validate reliable cross-internet messaging between two real people
on real, independent networks (Lucas on fiber, Josh on cellular/WiFi) ahead
of a wider alpha test.

## Architecture

"All nodes are relays" -- there is no special client/server split in
SCMessenger. This test has 4 endpoints plus 1 external rendezvous point:

```
Lucas (fiber)                                    Josh (cellular/WiFi)
+-------------------------+                      +-------------------------+
| Windows CLI (dedicated) |                      | Windows CLI (compiled   |
|   scm start              |                     |   from source, for       |
+-------------------------+                      |   testing/reference)     |
| Android emulator         |                      +-------------------------+
|   (scm_pixel_34, client) |                      | Physical Android phone  |
+-------------------------+                      |   (drives/tests -- the  |
                                                    |   real cellular/WiFi    |
                                                    |   test target)          |
                                                    +-------------------------+
              \                                              /
               \                                            /
                v                                          v
                   AWS alpha-relay (external, stable)
                   /ip4/<PUBLIC_IP>/tcp/9001
                   (real public internet, not a VPC/LAN)
```

**Why an external relay is needed:** there's no production relay network
deployed yet. Both of you are behind NAT (Lucas's home fiber router, Josh's
carrier-grade NAT on cellular). Direct P2P hole-punching may or may not
succeed depending on NAT type -- the AWS relay guarantees a reachable
rendezvous point either way, and since every node (including this one) is a
mandatory relay per the app's architecture, messages can also relay through
it if a direct path never forms.

## Relay Status

Instance: `i-0d302298a375dc4ec` (t3.micro, us-east-1)
Public IP: `100.56.248.69`

Check current status:
```bash
cd SCMessenger
bash infra/ec2/launch-alpha-relay.sh status us-east-1
```

Build takes 45-90 minutes on a t3.micro (serial cargo build, deliberately
capped to avoid OOM -- see infra/ec2/alpha-relay-userdata.sh comments).
Once ready, `curl http://100.56.248.69:9876/health` returns
`{"status":"healthy"}`.

**This relay is NOT the disposable farm-sim test fleet.** It has its own
security group, its own tags (`Purpose=AlphaRelay`), and its own teardown
script that requires typing the exact instance ID to confirm -- it won't be
casually torn down while you're testing.

## Bootstrap Address (both of you need this)

```
/ip4/100.56.248.69/tcp/9001
```

If the relay's public IP ever changes (e.g. after a restart), re-check with
`launch-alpha-relay.sh status us-east-1` -- it's a stopped/started instance,
not re-launched, so the IP should stay stable unless AWS itself reassigns it
(unlikely for a running instance, but Elastic IP would make this fully
permanent if it becomes a recurring issue).

---

## Lucas's Setup (Windows CLI + Android emulator)

### 1. Windows CLI (dedicated node)

Reference: `docs/CLI_WINDOWS.md`

```powershell
cd SCMessenger
cargo build --release -p scmessenger-cli
copy target\release\scmessenger-cli.exe $HOME\bin\scm.exe
# ensure $HOME\bin is on PATH, or copy to a dir already on PATH

scm init
scm config bootstrap add /ip4/100.56.248.69/tcp/9001
scm start
```

`scm start` runs interactively and stays up as your "dedicated" node --
leave this running in a terminal while testing.

### 2. Android emulator (client, for your own local testing)

The `scm_pixel_34` AVD (API 34, Google APIs, x86_64, Pixel 6a profile) is
already set up on this machine per `docs/CURRENT_STATE.md`.

```powershell
emulator -avd scm_pixel_34 -gpu host -no-audio -no-boot-anim
```

**GPU mode:** use `-gpu host` (hardware-accelerated rendering), not
`swiftshader_indirect`, on machines where WHPX/HAXM acceleration is
available (verify with `emulator -accel-check`) -- confirmed the case on
this Windows dev machine. `swiftshader_indirect` forces CPU-based software
graphics rendering; on a Compose UI app that renders continuously, this
starves the emulator's small vCPU allocation (`-cores 2` default) and was
observed to cause repeated main-thread ANRs ("SCMessenger isn't
responding") even though the CPU itself was WHPX-accelerated. Reserve
`swiftshader_indirect` for genuinely non-accelerated hosts (e.g. the cloud
worker path in `cloud/worker/android_worker_startup.sh`, which runs on
hardware without WHPX/KVM/HAXM).

Once booted, install/launch the app, then use the app's **Join Mesh**
screen (`android/app/src/main/java/com/scmessenger/android/ui/join/JoinMeshScreen.kt`)
to add the same bootstrap address: `/ip4/100.56.248.69/tcp/9001`

**Note:** the Android emulator's virtual network is NAT'd by the emulator
itself (separate from your host's real network stack) -- this is fine for
testing the emulator-to-relay-to-Josh path, but don't expect it to exercise
your home fiber's NAT behavior the way your Windows CLI node does.

---

## Josh's Setup (Windows compile + physical Android phone)

### 1. Windows (compile from source)

Same as Lucas's step 1 above -- Josh builds his own copy:

```powershell
git clone https://github.com/Sovereign-Communication/SCMessenger.git
cd SCMessenger
cargo build --release -p scmessenger-cli
copy target\release\scmessenger-cli.exe $HOME\bin\scm.exe
scm init
scm config bootstrap add /ip4/100.56.248.69/tcp/9001
```

Whether Josh runs `scm start` continuously or just uses this build to
verify/reference is up to him -- the primary test target on his side is
the physical Android phone below.

### 2. Physical Android phone (the real test -- cellular/WiFi)

This is the actual point of the test: Josh's real phone, on real cellular
data (his usual connection) and real home WiFi, talking to Lucas across
the real internet.

- Build the APK: `cd android && ./gradlew assembleDebug -x lint --quiet`
  (or Josh builds it himself on his own Windows machine, matching his
  "compile" role)
- Install on his physical device: `adb install app/build/outputs/apk/debug/app-debug.apk`
  (or transfer the APK directly if adb isn't set up on his end)
- Open the app, use **Join Mesh** to add: `/ip4/100.56.248.69/tcp/9001`
- Test with cellular data on, then again on home WiFi, to compare

---

## What to Actually Test

1. **Basic delivery**: Lucas's Windows CLI sends a message to Josh's phone
   (and vice versa) -- confirm it arrives.
2. **Cellular reliability**: Josh on cellular data (not WiFi) -- this is
   his "usual" connection per the stated requirement, so it's the
   most representative real-world condition.
3. **Dynamic connectivity**: Josh's phone switching between cellular and
   WiFi (e.g. leaving home, arriving home) -- does the mesh reconnect
   cleanly, or does messaging silently stop until the app is restarted?
4. **Lucas emulator <-> Josh phone**: confirms the relay correctly bridges
   two independent client types, not just CLI-to-CLI.
5. **Offline queueing**: send a message while the recipient's device is
   off/unreachable, confirm it's queued (via the relay's custody) and
   delivered once they reconnect.

## If Something Doesn't Work

- **Can't connect at all**: verify the relay is actually up
  (`curl http://100.56.248.69:9876/health`), and that both sides used the
  exact same bootstrap address.
- **Connects but messages don't arrive**: check each side's local status
  (`scm status` on CLI, Diagnostics screen on Android) for peer count --
  if 0 peers, the bootstrap/ledger-exchange handshake itself may be
  failing (this exact failure mode was diagnosed and fixed for the
  farm-sim topology earlier this session -- see git log for
  `SC_BOOTSTRAP_NODES` fixes if this recurs).
- **Josh's cellular carrier blocks something**: some carriers do
  aggressive NAT/firewall filtering. If direct connectivity to the relay
  fails specifically on cellular but works on WiFi, that's itself a
  useful, real finding for the alpha test -- document it rather than
  treating it as a setup error.

## Teardown (when testing is done)

```bash
bash infra/ec2/launch-alpha-relay.sh teardown us-east-1
# will prompt you to type the exact instance ID to confirm
```

Don't tear this down casually while Josh might still be testing --
unlike the farm-sim fleet, this one is meant to stay up.
