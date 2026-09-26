# Five-node unified transport test -- plan and readiness gate

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Active
Written: 2026-08-09 ~16:30Z
Orchestrator: Windows/Claude lane (primary; GPT-MAC spun down for the night)
Anchor: `d48558a8` on `tracking/pre-v040-tag-work` (PR #139)

The goal is one working app meshing across five nodes. This document is the
readiness gate: no node is "ready" until its row is evidenced, and the run is
not scoreable until every participating row is green.

---

## 1. Node inventory and ownership

| # | Node | Platform | Owner | Anchor | Status |
|---|---|---|---|---|---|
| 1 | Windows CLI | win32 desktop | **Windows lane (us)** | `d48558a8` exact | RUNNING, PeerId stable |
| 2 | Android | Pixel 6a | **Windows lane (us)** | needs build+install | **BLOCKED -- device offline** |
| 3 | AWS Ubuntu headless | t3.micro relay | **Windows lane (us)** | old image, `6b2573fa` | STALE -- needs redeploy |
| 4 | macOS CLI | darwin | GPT-MAC | `d48558a8` | offline for the night |
| 5 | iOS | iPhone | GPT-MAC | `d48558a8` | offline for the night |

**Tonight's achievable scope: a 3-node run across nodes 1, 2 and 3.** Nodes 4
and 5 resume when GPT-MAC returns. A 3-node run is a genuine test -- it exercises
LAN direct, relay circuit, and mobile-to-desktop paths -- but it is NOT the
five-node gate and must not be recorded as one.

---

## 2. Readiness gate -- every row needs evidence, not assertion

A node is READY only when all of these are true and recorded:

| Check | How it is evidenced |
|---|---|
| Correct build | `--version` reports the anchor SHA **and** worktree HEAD + clean `git status --porcelain` + binary mtime agree |
| Identity preserved | PeerId matches the node's prior PeerId; data directory not wiped |
| Listening | `/api/listeners` shows concrete addresses; `netstat` confirms the ports are bound to the process |
| Reachable | another node establishes an inbound connection to it |
| Sees the mesh | `/api/peers` lists the other participating nodes by their real PeerIds |
| Survives | stays alive for the full run window without the swarm loop dying |

The last row is the one that has failed all day. See Section 5.

---

## 3. Per-node work required

### Node 1 -- Windows CLI (READY)

Running exact `d48558a8`, PID 17568, PeerId
`12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw`, stable across five SHAs
today. Provenance verified.

Known platform limitation: **mDNS is dead on Windows.** `libp2p-mdns` fails
~200 ms after start with `WSAEMSGSIZE` (os error 10040) on every run, so Windows
cannot contribute mDNS discovery evidence. It falls back to promiscuous ledger
dialing. Analysis recommends filtering nested `p2p-circuit` addresses out of
mDNS advertisements to bring packet size under the receive buffer; that work is
in `core/src/transport/`, which is merge-blocked and needs an operator decision
plus adversarial review before implementation.

### Node 2 -- Android (BLOCKED, then build + install)

**Blocker: the Pixel is not connected to ADB.** `adb devices` is empty and mDNS
discovery returns nothing. Wireless ADB pairing does not survive a device reboot
or a network change, and the port rotates. This needs a physical action -- USB
cable, or re-pairing wireless debugging from the device's Developer Options.
**This is an operator dependency; no agent can resolve it remotely.**

Once connected:
1. Build the debug APK from the anchor.
2. `adb install -r` to preserve identity -- upgrade-in-place is proven to work
   when the signing key matches (verified 2026-08-09: `lastUpdateTime` moved
   while `firstInstallTime`, `contacts.db`, `history.db`, `ledger.json`,
   `pending_outbox.json`, `relay_custody` and identity blobs all survived).
   A key change (CI or release signing) forces a wipe -- use the local key.
3. `adb shell logcat -G 16M` before the run. The default 256 KiB buffer evicted
   the app logs during the last field test and cost a whole analysis pass.
4. Confirm PeerId is unchanged after install.

Android carries the mDNS parity fix in PR #144 (rejects fabricated `mdns-*`
peer ids, retains the full pinned `/p2p/<PeerId>` multiaddr, advertises a
concrete `dnsaddr` instead of the `0.0.0.0` wildcard). That fix addresses the
exact failure GPT-MAC's macOS soak reported: mDNS advertised a pinned address,
the connection did not complete, inbound negotiation aborted.

### Node 3 -- AWS Ubuntu headless (STALE, needs redeploy)

Currently healthy but running a **stale image** from commit `6b2573fa`
(PR 136+137+138). The current anchor is `d48558a8` -- roughly 60 commits of
mesh, relay and CLI work newer.

- Public IP `54.226.67.101`, health endpoint returns `{"status":"healthy"}`.
- Bootstrap multiaddr `/ip4/54.226.67.101/tcp/9001`.
- **Never build on the t3.micro.** A previous attempt OOM'd after 16 hours.
  Pull a prebuilt image (`docker pull testbotz/scmessenger:latest`) built by CI.
- Therefore: a candidate image must be published from the anchor BEFORE the AWS
  node can be updated. No candidate image currently exists for `d48558a8`.

### Nodes 4 and 5 -- macOS and iOS (GPT-MAC)

Not ours. Handoff notes in Section 6.

---

## 4. Sequence

1. **Land PR #144** into the PR branch once CI is green. It carries the Android
   parity fix and the dial-dedup fix, both of which the run depends on.
2. **Publish a candidate image** from the resulting head so AWS can be updated
   without building on the instance.
3. **Reconnect the Pixel** (operator action), then build and install the APK.
4. **Redeploy AWS** from the published image.
5. **Restart the Windows node** on the final head.
6. **Verify every row of Section 2** for nodes 1-3 before sending a single
   message. A run scored against an unverified node is wasted.
7. **Run the 3-node protocol** (Section 7).

---

## 5. The blocker that decides whether any of this is scoreable

**The desktop node dies on the request-response assertion.** Reproduced 3 of 3
runs on `ebd723ab` at 62 s, 759 s and 152 s; a 4th run survived 40+ minutes.
Same binary, so timing is highly variable and no single green run proves a fix.

- Root cause of the assertion: connection-map drift inside
  `libp2p-request-response` 0.29.0. It is a `debug_assert`, so **release builds
  do not panic -- they accumulate the drift silently.** Switching to release to
  stabilise a run would hide the defect, not fix it. That is not a way through.
- 0.29.0 is the latest published version. There is no upstream bump.
- Root cause of the **trigger**: the in-flight dial guard keyed on PeerId while
  the contended resource is the address, so ghost identities multiplied
  concurrent connections to one host:port. Fixed in PR #144.

**PR #144 removes the trigger, not the defect.** If the fleet stays up after it
lands, that is encouraging, not proof. The honest test is the uptime
distribution across many runs, before versus after -- which is why the Windows
node is being run repeatedly on unpatched `d48558a8` to build the "before"
baseline.

---

## 6. Handoff to GPT-MAC (nodes 4 and 5)

- Anchor: `d48558a8`, or whatever head PR #144 produces once merged.
- PR #144 is open for review against `tracking/pre-v040-tag-work`.
- The reconciliation merge of `origin/main` is agreed but deferred until the two
  runtime gates pass, at GPT-MAC's request. It is a clean merge -- measured,
  zero conflicts.
- Still unanswered: macOS build provenance (worktree HEAD + clean status +
  binary mtime of the running process, not the SHA intended). GPT-MAC has since
  stated the macOS driver has stable identity, which if it holds resolves the
  ghost-identity question.

---

## 7. Three-node run protocol (nodes 1-3)

Do not send a message until every Section 2 row is evidenced for all three.

1. Record each node's provenance, PeerId, listeners, and peers.
2. Confirm each node sees the other two by real PeerId.
3. Windows -> Android: record message id, route, transport ACK, receipt,
   and receiver-side decrypt + durable history row.
4. Android -> Windows: same.
5. Both directions through the AWS relay circuit specifically, not just direct.
6. Restart one node mid-run; confirm reconnection and no message loss.
7. Record every node's uptime for the window, and whether any swarm loop died.

**Scoring rules, learned the hard way:**
- Score on receiver-side decrypt + durable history + receipt. **Never** on
  transport ACKs, UI counts, or BLE local acceptance.
- `/api/send` returning non-200 is NOT proof of non-delivery -- the retry
  machinery has delivered messages 28 ms after the API reported failure.
- Desktop senders may have no sender-side history row; that is a known CLI
  defect, not evidence of failure.
- A node that died mid-window invalidates every delivery result attributed to
  it for that window.

---

## 8. Address churn is a TEST CONDITION, not a defect (operator direction, 2026-08-10)

**Do not pin addresses. Do not allocate an Elastic IP for the AWS relay.**

Changing public IPs are the realistic condition this product must survive: a home
connection gets a new public IP regularly, and every peer must find its way back
without human intervention. The mesh is required to reconcile that through
**ledger sharing**, not through stable addressing.

This inverts the usual instinct. An IP change is not something to eliminate from
the test environment -- it is something to provoke and then verify recovery from.

### The invariant being tested

**Identity is stable. Addresses are fluid.**

- A node's libp2p PeerId MUST survive restarts, rebuilds and reinstalls.
- A node's multiaddrs are expected to change without warning.
- Peers MUST learn new addresses for a known PeerId via ledger exchange and
  re-establish, with no manual contact entry and no manual dial.

A design that needs a fixed IP has failed this test even if every message
delivers.

### Evidence that this already partly works (2026-08-09/10 run)

The Pixel's LAN address changed mid-session and the Windows node followed it
without intervention:

```
16:31:10  Connected to 12D3KooWNnPi9w... via /ip4/192.168.0.111/tcp/44xxx
16:48:15  Connected to 12D3KooWNnPi9w... via /ip4/192.168.0.107/tcp/44xxx
```

Same PeerId, new address, reconnected. Ledger exchange is active and repeats on
every new connection (683 ledger-related events in one run; exchanges with the
Pixel at 22:13, 00:12 and 00:34).

Note what this reframes: the macOS "stale address" failures were most likely NOT
staleness. They were the dual-transport advertisement -- `/ws` advertised for
listeners that are plain TCP. Address churn was a red herring there.

### What the five-node run must therefore exercise

Add these to the protocol in Section 7:

1. **Provoke an address change mid-run.** Force at least one node onto a new
   address (toggle Wi-Fi on the Pixel, or let the AWS relay be replaced and
   acquire a new ephemeral IP). Do not warn the other nodes.
2. **Verify recovery without manual help.** Peers must re-establish to the same
   PeerId at the new address via ledger propagation alone. No manual contact
   entry, no manual dial command -- those invalidate the result.
3. **Measure convergence time** from address change to re-established
   connection. This is a real product metric: it is how long a user is
   unreachable after their ISP renumbers them.
4. **Confirm no ghost accumulation.** After the change, the old address should
   stop being dialled within a bounded time. Peers holding many dead addresses
   for one PeerId is the ledger-hygiene defect already filed, and address churn
   is what feeds it.

### Consequence for the AWS relay

The relay's public IP is ephemeral and will change on any stop or rebuild. That
is now **desirable**. Do not allocate an Elastic IP.

It does mean `HANDOFF/gpt/AWS_RELAY_CURRENT_ADDRESS.md` must be updated
immediately after any relay replacement, and that every lane must read it fresh
at use time rather than caching an IP -- which is already that file's stated
policy.

Access to the relay should therefore be by **instance id**, not IP:
EC2 Instance Connect (`ec2-instance-connect:SendSSHPublicKey`) is keyed to the
instance and survives address changes, which is why it is the preferred access
path over a pinned SSH endpoint.
