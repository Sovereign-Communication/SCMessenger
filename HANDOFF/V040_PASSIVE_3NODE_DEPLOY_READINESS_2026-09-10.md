# V040 passive 3-node deploy readiness checkpoint

Status: READY FOR FINAL 3-NODE TEST (operator-owned cutover)
UTC: 2026-09-10 (this session)
Workspace: `C:\Users\SCM\Documents\GitHub\MiMoSCMessengerFresh`
Branch: `unified/v040-3node-parity`
Candidate tip: `5f1f29bf` (docs/logging on top of unify `8b0bd6a8` / code `3ccf0ec2`)

## 1. Request-failure RCA (why MiMo worked outside SCMessenger)

**Root cause:** project `.mimocode/mimocode.json` disabled the working
MiMo/Xiaomi provider and pinned every agent to OpenRouter free models that
require an unset `OPENROUTER_API_KEY`. Sessions rooted under SCMessenger or
MiMoSCMessengerFresh loaded that override and failed; default-workspace MiMo
sessions never did.

**Fix (landed):**
- Prior config saved as `.mimocode/mimocode.json.bak-openrouter-override-20260910`
- Active config neutralized to schema + username only (host defaults inherit)
- Applied on: SCMessenger main, MiMoSCMessengerFresh, and worktree
  `scm-mimo-fix-reqs` (commit `0891397d`)
- RCA: `scm-mimo-fix-reqs/.mimocode/MIMO_REQUEST_FAILURE_RCA.md`

## 2. One-candidate parity state

| Node | Live git/provenance | Unified candidate | Action before final test |
|---|---|---|---|
| Windows CLI | `/version` `5c7caa1d`, provenance `ba474a7a` | `5f1f29bf` | Rebuild + relaunch from this SHA |
| AWS | `/version` `7ff317f0...` | `5f1f29bf` | Redeploy docker at same SHA (E3/E4) |
| Pixel 6a | installed APK from prior campaign tree | `5f1f29bf` | Operator installs APK from this SHA (E5) |

Live Windows diagnostics (pre-cutover probe): healthy, peers = Phone + AWS,
`external_addrs = ["147.81.41.188:9001","192.168.0.222:9001"]`, outbox 24.
**Still shows nested p2p-circuit listeners** — D10b guard is in the candidate
code but **not in the running binary**. Cutover closes this.

## 3. Gates recorded this session

| Gate | Result | Evidence |
|---|---|---|
| cargo check core+cli | PASS | Fresh workspace, after unify |
| Targeted transport tests | PASS 16/16 then 6/6 reconfirm | observation, configured-external, poison_listener (D10b), routing_peer_seen |
| mimocode request path | FIXED | neutral config + backup |
| Rollback staged outside target/ | DONE | `SCMessenger/tmp/radio-live-rollback-20260910/` SHA256 `660BE35D...` |
| Bootstrap no longer env-only | DONE (config) | `bootstrap_nodes` persisted AWS multiaddr in `%APPDATA%\scmessenger\config.json` (backup `config.json.bak-before-bootstrap-pin-20260910`) |
| Full core suite | NOT rerun this session (disk ~12 GB free; E9 wanted ≥20) | prior green: 1653/0/24 at E2 |
| Android assembleDebug | NOT rerun this session (disk) | prior unit BLE gates green |
| Rule-8 D10/D10b adversarial APPROVE | STILL PENDING | merge-to-main gate only; live deploy not blocked |

## 4. Transport log visibility (for passive scoring)

Added INFO lines so the final test can be scored from node logs alone:

- `[TRANSPORT-LANE] peer=... direction=outbound|inbound transport=... addr=...`
  on every `ConnectionEstablished`
- `[TRANSPORT-LANE] peer=... event=disconnected` on disconnect
- Existing: `Connected to ... via ...`, `Peer discovered/disconnected`,
  `Sending delivery ACK for ...`, `[OK] Message delivered successfully`,
  `mDNS discovered peer`, `[D10b] Poison-listener guard: ...`,
  `Seed peer dial failed`, external_addrs via `/api/diagnostics`

**Passive 3-node success criteria (bidirectional):**
For each ordered pair (Windows↔AWS, Windows↔Phone, AWS↔Phone):
1. Both sides log `direction=outbound` and `direction=inbound` for the peer
   (or equivalent connected/inbound evidence)
2. Delivery ACK or inbox receive appears on the recipient
3. No identity change; `/version` same SHA on all three

## 5. Explicit open items before / during final test

1. **Operator:** install Pixel APK built from `5f1f29bf` (E5).
2. **Operator/CTO:** redeploy AWS at `5f1f29bf`, capture `sha256sum` + `/version` (E3/E4).
3. **Operator:** stop old Windows node, relaunch with staged launcher env
   (`SC_BOOTSTRAP_NODES` still recommended; config bootstrap now also pinned).
   Rollback: `tmp/radio-live-rollback-20260910/scmessenger-cli-live.exe`.
4. **Operator:** reboot Windows laptop for MT7921 BT radio (D7 / BLE leg).
   Post-reboot checklist in `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260910T051500Z_OPERATOR_TEST_RCA_FIX.md`.
5. **Do not tag v0.4.0** until final matrix is green (X6 / operator ruling).
6. **Rule-8** independent APPROVE still required for merge to main.

## 6. Disk note

Free ~12 GB at readiness write time. Reclaim before full suite / Android
assemble if needed (`scripts/reclaim_safe.py` currently HOLD on dirty
worktrees). Main SCMessenger `target/` is ~32 GB — do not delete the live
exe path without the staged rollback already in place (it is).

## 7. Stop condition for this session

Passive 3-node **deploy readiness** is complete: request-failure root cause
fixed, candidate tree gates green, rollback + bootstrap pin + lane logs
landed. Final operator cutover + message matrix is the remaining step.
