# Cellular-path triangulation across 3 nodes - 2026-09-15

Question (operator): were messages sent off WiFi (cellular) successfully
stored/forwarded, or was a direct connection achieved? Triangulate across
all 3 nodes.

## Peer map (verified from logs on all three sides)

| Node | libp2p PeerId | Path during test |
|---|---|---|
| Pixel 6a | `12D3KooWKT1e1PU7p...` | cellular (WiFi off), user mobile |
| Windows | `12D3KooWD6vZQrUq...` | LAN 192.168.0.121, build f985b10 |
| AWS cloud | `12D3KooWGvCWJNoWn...` | 18.234.62.247, image sha-31776b4 |

## Verdict

1. **Direct connection Pixel<->Windows: NEVER achieved.** Windows log shows
   repeated `DCUtR hole-punch FAILED with 12D3KooWKT1e1... - will relay
   messages instead: Giving up after 3 dial attempts` (01:03Z-01:31Z window,
   6+ attempts). Expected: Pixel is behind carrier-grade NAT and Windows is
   behind home NAT (double-NAT hole punch is not traversable). The fallback
   design worked exactly as intended.
2. **Pixel -> AWS: all messages stored.** AWS `docker logs` shows 12
   `Accepted custody ... for offline destination 12D3KooWD6vZ...` entries
   between 01:57Z and 04:13Z (sender 12D3KooWKT1e1... = Pixel over cellular).
3. **AWS -> Windows delivery: DELAYED BY A WINDOWS NODE WEDGE, then 100%.**
   The Windows node silently wedged at ~01:32:50Z (see
   `HANDOFF/todo/P1_WINDOWS_NODE_SILENT_WEDGE_2026-09-15.md`) and stopped
   pulling. On restart at ~04:21:30Z, AWS burst-delivered all 12 held
   messages at 04:22:14Z within one second (delivery count for the Windows
   destination went 72 -> 100+). **Zero message loss.**
4. **Receipt convergence: completed.** Phone log shows both stuck messages
   (`59e89dfb-...`, `8b2011ac-...`) processed application delivery receipts
   at 04:22:24Z; `pending_outbox.json` drained to `[]`; delivery_state
   backlog (acked_without_receipt=94 seen at 18:13 HST) cleared.
5. **Windows -> Pixel over cellular: works direct-to-cloud.** Windows
   `ROUTE_DECISION ... route=direct ... destination=12D3KooWKT1e1...` then
   `[OK] Message delivered successfully to 12D3KooWKT1e1... (13ms)` at
   04:22:24Z - via the AWS relay path (circuit), delivered + inbox_receive on
   the Pixel side, contact 'Lucaso' learned from the identity envelope.

## Answer in one line

Every cellular message was **stored-and-forwarded through the AWS cloud node
(delivery guaranteed, zero loss)**; a direct Pixel<->Windows connection was
never established (double-NAT, DCUtR correctly gave up and fell back to
relay). The only failure in the chain was the Windows node silently wedging
for ~2h45m, which delayed delivery but lost nothing - and the moment it
recovered, the entire backlog drained automatically. That automatic drain IS
the eventual-delivery doctrine working.

## Defect found (1)

- P1 Windows silent wedge, 2h45m, no log fingerprint, watchdog blind to it:
  `HANDOFF/todo/P1_WINDOWS_NODE_SILENT_WEDGE_2026-09-15.md`.

## Pre-existing observations reconfirmed (no regression)

- Self-circuit anomaly on cloud node (`Circuit closed: 12D3KooWD6vZ... ->
  12D3KooWD6vZ...`) still present in logs - tracked since PR #218
  investigation; cosmetic so far (no misdelivery observed), keep tracked.
- `ghost-identity-001 skip auto-subscribe` fired correctly for the stale
  `30d0fa67...` peer topic on Windows - the GhostIdentityGate is live.
- Routing engine logs `decided_by=StoreAndCarry confidence=0.0` on some
  decisions - the D6 routing-confidence wiring remains unproven in the field
  (T4 re-measure still outstanding; already tracked).

## Evidence commands (this session)

- Pixel: `adb logcat -d -t 4000` (tmp/pixel_logcat_0915.txt);
  `run-as ... cat files/pending_outbox.json` -> `[]` post-restart;
  `tail -c 300000 files/logs/scmessenger-mesh.log | grep receipt`.
- AWS: `ssh ec2-user@18.234.62.247 docker logs --since 12h scm-node | grep
  -E 'custody|delivered'` (counts: 12 accepted-for-Windows, 72->100
  delivered-to-Windows, 0 delivered-to-Pixel needed - Pixel pulled
  directly).
- Windows: `%LOCALAPPDATA%\scmessenger\logs\scm.log.2026-09-15-01` (frozen at
  01:32:14Z) and `-04` (live post-restart: 102 ACKs, inbox_receive x3).
