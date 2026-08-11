# P0 -- UPnP panic kills the desktop node ~5 minutes after start

Status: Fixed in PR #139 hardening branch; Windows soak still required
Severity: P0 (release blocker for v0.4.0)
Discovered: 2026-08-08 (Windows orchestrator lane, live node run)
Affects: all non-Android desktop lanes (Windows CLI, macOS CLI, likely iOS)

## Summary

`scmessenger-cli start` panics inside the upstream `libp2p-upnp` crate and
self-terminates after roughly five minutes of uptime. The node is fully
functional up to that point -- it discovers peers, exchanges ledgers and
accepts relay reservations -- then dies.

This is a strong candidate explanation for the 2026-08-08 field-test
observation that "the other nodes on the same LAN (macOS/Windows) were never
detected nor observed": a desktop node that dies ~5 minutes after launch is
simply absent for most of any test window.

## Evidence

Run 1, Windows node, `target/debug/scmessenger-cli.exe start` (0.4.0, 6cb7033a):

```
[OK] Network started                      2026-08-09T00:10:00Z
...
thread 'tokio-rt-worker' (19944) panicked at
  C:\Users\SCM\.cargo\registry\src\index.crates.io-.../libp2p-upnp-0.5.0/src/behaviour.rs:497:38:
mapping should exist
2026-08-09T00:15:20.617932Z ERROR scmessenger_cli: swarm_event_loop_died:
  the mesh is down but the process is still up; exiting so this node does not
  linger as a zombie
[FAIL] Swarm event loop died -- exiting rather than running without a mesh.
```

Uptime: 00:10:00 -> 00:15:20, **5m20s**. Process exit code 1.

The panic is an `expect("mapping should exist")` inside the dependency, not in
repo code. It is reached from the UPnP behaviour's port-mapping bookkeeping,
consistent with a mapping being removed or expiring while still referenced.

Note the CLI's own shutdown handling behaved **correctly**: it detected the dead
swarm event loop and exited rather than lingering as a zombie with no mesh.
That safety net is working as designed; the defect is upstream of it.

## Where UPnP enters the build

- `Cargo.toml:32` enables the libp2p `"upnp"` feature workspace-wide.
- `core/src/transport/behaviour.rs:24` `use libp2p::upnp;`
- `core/src/transport/behaviour.rs:72` `pub upnp: upnp::tokio::Behaviour,`
- `core/src/transport/behaviour.rs:520` `let upnp = upnp::tokio::Behaviour::default();`

UPnP is **unconditionally constructed** -- there is no feature gate, config flag
or platform cfg guarding it. Event handling is at
`core/src/transport/swarm.rs:4576-4590`.

## What was working before the panic (do not lose this)

The same run proves desktop LAN discovery and mesh formation DO work:

```
00:10:01  Connected to 12D3KooWJUJ1... via /ip4/192.168.0.142/tcp/9090 (promiscuous mode)
00:15:17  Reset backoff state after successful connection peer_id=12D3KooWNnPi9wqUJ7... (Android)
00:15:17  Ledger exchange response from 12D3KooWNnPi9wqUJ7...: they learned 1 new peers, sent 64 back
00:15:17  [OK] Relay circuit reservation ACCEPTED via 12D3KooWNnPi9wqUJ7...
```

So the Windows node reached BOTH iOS (`192.168.0.142`) and Android
(`192.168.0.141`), exchanged ledgers, and established relay circuits -- but
convergence to Android took ~5 minutes, arriving at almost exactly the moment
the node panicked.

## Related observations from the same run (separate tickets may be warranted)

1. **mDNS is broken on Windows.** `libp2p_mdns::behaviour::iface: failed reading
   datagram: ... (os error 10040)` (WSAEMSGSIZE -- receive buffer smaller than
   the datagram). Peer discovery fell back to promiscuous ledger dialing.
2. **Promiscuous dialing accepts any PeerID**: `Connected to <peer> ...
   (promiscuous mode -- any PeerID accepted)`. Security review warranted.
3. **Dead bootstrap entry**: the node's only bootstrap is
   `/ip4/127.0.0.1/tcp/19001`, which nothing serves.
4. **Ledger dial pressure**: 825 peers (719 reachable, 106 in backoff) swept
   promiscuously on startup, producing sustained failed-dial churn.
5. Android reports `Bootstrap all-failed (consecutive=28)` on a 60s retry loop.

## Reproduction

```bash
cd <repo>
./target/debug/scmessenger-cli.exe start > tmp/logs/node.log 2>&1
# wait ~6 minutes
grep -E "panicked|swarm_event_loop_died" tmp/logs/node.log
```

### Run 2 -- reproducibility NOT yet established

A second run was started at `2026-08-09T00:16:05Z`
(log: `tmp/logs/win_node_run2.log`, pid 10864).

At `2026-08-09T00:21:08Z` it had **5m03s uptime and 0 panics** -- i.e. it had
just reached the point where run 1 died (5m20s) without failing. The session
stood down at that moment, so the outcome is UNKNOWN.

**Do not treat the 5-minute figure as deterministic.** Run 1's panic is a
single observation. The `expect("mapping should exist")` in libp2p-upnp is
consistent with a race or a mapping-expiry timing dependency, which would make
failure intermittent rather than scheduled.

### Run 2 -- RESOLVED 2026-08-09: it panicked too, at a DIFFERENT uptime

The next orchestrator checked, as asked. Run 2 did not survive.

```
tmp/logs/win_node_run2.log:1471
thread 'tokio-rt-worker' (25816) panicked at
  ...\libp2p-upnp-0.5.0\src\behaviour.rs:497:38:
mapping should exist
2026-08-09T00:32:47.851685Z ERROR scmessenger_cli: swarm_event_loop_died: ...
[FAIL] Swarm event loop died -- exiting rather than running without a mesh.
```

- Start 2026-08-09T00:16:05Z, panic 00:32:47Z -- uptime **16m42s**.
- Identical panic site to run 1 (`behaviour.rs:497`, `mapping should exist`).
- No `scmessenger` process is running now; both runs self-terminated.

Conclusion: the defect is **reproducible (2 of 2 runs) but not scheduled** --
5m20s vs 16m42s. That rules out a fixed timer and is consistent with the
race / mapping-expiry hypothesis. The "5 minutes" figure in this ticket's
title and Summary is a single-observation artifact; do not design around it.
It also weakens (does not kill) the theory that UPnP death explains the
2026-08-08 "other LAN nodes never detected" report -- run 2 was alive for
nearly 17 minutes and the LAN peers were still not observed for most of it.
That observation needs its own root cause; see the field-finding tickets.

Next orchestrator: the remaining gate is the post-removal soak, below.

```bash
tasklist | grep -i scmess          # NOT `tasklist /FO CSV /NH` -- see caveat below
grep -E "panicked|swarm_event_loop_died" tmp/logs/win_node_run2.log
```

Caveat carried forward: `tasklist /FO CSV /NH` silently returns nothing under
Git Bash (it mangles `/FO` into a path), producing a false "process not
running". Use plain `tasklist | grep -i`.

## Resolution

The user-authorized remediation removes the optional `libp2p-upnp` feature and
the unconditionally constructed UPnP behaviour from the workspace. Relay v2,
DCUtR, AutoNAT, and ledger-based address exchange remain available for the
mesh; a gateway port-mapping dependency is no longer allowed to terminate the
swarm event loop. The Windows authoritative lane must still run a long-lived
soak and confirm that no `panicked` or `swarm_event_loop_died` lines occur.

## Not selected

The previously listed directions are superseded by the feature removal above.
No additional UPnP code path remains in the workspace; the Windows soak is the
remaining evidence gate.
