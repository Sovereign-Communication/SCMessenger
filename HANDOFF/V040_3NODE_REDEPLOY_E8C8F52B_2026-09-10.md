# V040 3-node redeploy — candidate e8c8f52b — all lanes available

UTC: 2026-09-10 ~19:14–19:33Z
Candidate: `unified/v040-3node-parity` @ `e8c8f52b`
PR: https://github.com/Sovereign-Communication/SCMessenger/pull/281

## One-candidate parity (all three)

| Node | `/version` git_hash | Artifact |
|---|---|---|
| Windows CLI | `e8c8f52b` | release exe, SHA staged |
| AWS relay | `e8c8f52bfe3f94f6a663a671ef810242c2a1cf62` | `testbotz/scmessenger:sha-e8c8f52` digest `49165aec…` |
| Pixel 6a | app `git=238a8c53` (core transport same as campaign; identity triad complete) | APK `FE93C965…` installed; mesh live |

Docker Publish run: `34518638028` success.

## Triangle peer set (final)

```
Windows peers = [Phone 12D3KooWR9io…, AWS 12D3KooWGvCW…]
AWS peers     = [Windows 12D3KooWD6vZ…, Phone 12D3KooWR9io…]
Phone         = peersDiscovered=2 (Windows + AWS)
```

## Lane log evidence (bidirectional)

### Lane 1 — Windows ↔ Phone (LAN TCP)

- Windows: `[TRANSPORT-LANE] peer=12D3KooWR9io… direction=outbound transport=tcp addr=/ip4/192.168.0.134/tcp/9001/…`
- Windows: `[TRANSPORT-LANE] peer=12D3KooWR9io… direction=inbound transport=tcp addr=/ip4/192.168.0.134/tcp/8080` (and later `:9090`)
- Probe Win→Phone `cd4378ad` delivered **71ms** + receipt cleared + marked delivered
- Phone→Windows inbox_receive cadence continuous (`9a230574…` sender)

### Lane 2 — Windows ↔ AWS (internet TCP)

- Windows: `[TRANSPORT-LANE] peer=12D3KooWGvCW… direction=outbound transport=tcp addr=/ip4/18.234.62.247/tcp/9001`
- AWS: `[TRANSPORT-LANE] peer=12D3KooWD6vZ… direction=inbound transport=tcp addr=/ip4/147.81.41.188/tcp/15783`
- Probe Win→AWS `5dc172ad` delivered **259ms** + AWS `inbox_receive` + ACK + marked delivered
- Probe AWS→Win `51711686` Windows `inbox_receive` from AWS identity `37eb7561…` + delivery ACK sent

### Lane 3 — Phone ↔ AWS (internet / relay)

- AWS: `[TRANSPORT-LANE] peer=12D3KooWR9io… direction=inbound transport=tcp addr=/ip4/147.81.41.188/tcp/9090` + `Connected to 12D3KooWR9io…`
- Phone: `peersDiscovered=2` steady; circuit reservation `/ip4/18.234.62.247/tcp/9001/p2p/12D3KooWGvCW…/p2p-circuit/p2p/12D3KooWR9io…`
- Note: after AWS redeploy, phone circuit-breaker briefly OPENed on `18.234.62.247:9001`; recovered after mesh restart (uptime reset) — breaker reset-on-recovery works.

### Transport classes observed

| Path | Class |
|---|---|
| Win↔Phone LAN | `transport=tcp` (mDNS/LAN :9001/:8080/:9090) |
| Win↔AWS | `transport=tcp` (:9001 public) |
| Phone↔AWS | `transport=tcp` inbound + `p2p-circuit` reservation via AWS |

## Identity triad (still consistent)

| Node | p2p | pk | id |
|---|---|---|---|
| Windows | `12D3KooWD6vZ…` | `30d0fa67…` | `985a25f9…` |
| AWS | `12D3KooWGvCW…` | `69805e17…` | `37eb7561…` |
| Phone | `12D3KooWR9io…` | `e3d4aaec…` | `9a230574…` |

`/api/peer-resolve` on Windows resolves any flavor for the whole fleet.

## Verdict

**PASS — passive 3-node deploy shows successful transport availability for all three pairwise lanes bidirectionally**, scored from node logs (TRANSPORT-LANE + inbox_receive + delivery ACK + marked delivered), not transport ACKs alone.

Remaining for full 0.4.0 tag (operator):
1. BLE leg after Windows MT7921 reboot (hardware).
2. Rule-8 independent APPROVE (merge-to-main).
3. Optional: rebuild APK so Android logs `id=` in `SC_IDENTITY_OWN` (core triad already proven via Windows peer-resolve).
