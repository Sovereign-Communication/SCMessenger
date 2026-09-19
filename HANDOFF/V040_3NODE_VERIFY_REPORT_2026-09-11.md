# 3-node verification — Pixel reinstall + off-site vs home log compare

UTC window: 2026-09-10 ~20:09Z (off-site) through 2026-09-11 ~00:30Z (home)
Candidate: Windows `441a0214`, AWS `e8c8f52b`, Pixel APK reinstall 14:25 local
(`git=f9a1f60b` family, SHA256 `10C43A79…`)

## Pixel reinstall

- Package reinstalled 2026-09-10 14:25:35 local; mesh RUNNING immediately
- `SC_IDENTITY_OWN` now logs full triad:
  - p2p `12D3KooWFhvgUu1UF8BoQG8dBxjdjDvoWJfgCnhFgzoSDAWqExt7`
  - pk `577fd1715f9f95fae10da5ea01aa20ac6789dfd898c3351ca3b6f62b249c4fb8`
  - id `77210c717fc849bb8a24e9f4462eeb03947fa69785ee87c0d18e31f07300775a`
- `peersDiscovered=3` (Windows + AWS + emulator)

## Off-site (cell) vs home — log delta

| Check | Off-site ~20:09Z | After custody fix + reinstall |
|---|---|---|
| `identity_registration_missing` phone→AWS→Windows | **12 WARN** (all 20:09–20:10Z) | **0** after 20:36Z |
| Direct dial phone→AWS `:9001` | NetworkException / circuit OPEN | Still non-fatal fails; **circuit via Windows works** |
| Phone→AWS `inbox_receive` | Yes (sender `77210c71…`) | Continuous (~60–90s cadence) |
| peersDiscovered | 2 | **3** (incl. emulator) |
| Network | cellular / off LAN | WiFi Kana5G `192.168.0.134` |

## Bidirectional probes (home, after reinstall)

| Probe | Result (from logs) |
|---|---|
| Win→Phone `0bd7d8e2` | delivered **27ms** + receipt cleared + marked delivered |
| Win→AWS `58e516a0` | delivered **259ms** + AWS `inbox_receive` + ACK + marked delivered |
| AWS→Win `14b6917d` | Windows `inbox_receive` from AWS id `37eb7561…` + delivery ACK |

## Triangle peers (final)

```
Windows 441a0214  peers = [Phone WFhvg, Emulator WAv9, AWS WGvCW]
AWS e8c8f52b      peers = [Windows WD6vZ, Phone WFhvg, Emulator WAv9]
Phone             peersDiscovered = 3
```

## Identity / custody logging

- Windows: `[CUSTODY] Registered local identity with peer … (relay-ready)`
- Android: `SC_IDENTITY_OWN p2p_id=… pk=… id=…` (full triad)
- Zero custody rejections after 20:36Z

## Still not perfect (honest residuals)

1. **Cell-only retest not re-run** after reinstall — last cell session was
   pre-fix. Next away trip should show relay via AWS without
   `identity_registration_missing`.
2. Phone **direct dial to AWS public :9001 still fails** (non-fatal); mesh
   relies on circuit/relay + LAN. Worth a follow-up ticket (possible
   carrier NAT / AWS SG).
3. **BLE** still hardware-gated (Windows MT7921).
4. **AWS image** still `e8c8f52b` while Windows is `441a0214` (docs+CLI
   custody fix). Redeploy AWS at `441a0214` for exact SHA parity.
5. Emulator still online — can be shut down when not needed.

## Evidence paths

- Off-site pull: `tmp/android_logpull_field_20260910T101148/`
- Post-reinstall: `tmp/android_verify_20260910T142800/`
- Windows: `tmp/DEPLOY_CUSTODY/win.out`
