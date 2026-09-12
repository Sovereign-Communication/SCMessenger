# V040 3-node deploy evidence — candidate 238a8c53

UTC window: 2026-09-10 ~14:19–14:55Z
Candidate: `unified/v040-3node-parity` @ `238a8c53`
PR: https://github.com/Sovereign-Communication/SCMessenger/pull/281

## One-candidate parity

| Node | Artifact | `/version` git_hash | Provenance |
|---|---|---|---|
| Windows CLI | release exe, SHA256 staged outside target | `238a8c53` | `0.4.0 (5f1f29bf:unified/v040-3node-parity)` (docs-only delta after rebuild) |
| AWS | `testbotz/scmessenger:sha-238a8c5` | `238a8c5335a00fec…` | `0.4.0 (238a8c53…:unified/v040-3node-parity)` |
| Pixel 6a | `app-debug-238a8c53.apk` SHA256 `FE93C965…` | app logs `git=238a8c53` | lastUpdate 2026-09-10 05:31Z; `MeshApplication: version=0.4.0 (14), git=238a8c53, ref=unified/v040-3node-parity` |

AWS image digest: `sha256:155d137073499a715cc5cd23e5f69e3f3d1c506a3bba3e4efdd8953e5b0f953e`
Identity mount: `/opt/scm-relay-data` → `/data` (verified present)
Duplicate EC2 `i-0b735c4f` **terminated**; only `i-0b41aab7` running.

## Transport lanes (bidirectional, from logs)

### Windows↔AWS

- Windows log: `[TRANSPORT-LANE] peer=12D3KooWGvCW… direction=outbound transport=tcp addr=/ip4/18.234.62.247/tcp/9001`
- AWS log: `[TRANSPORT-LANE] peer=12D3KooWD6vZ… direction=inbound transport=tcp addr=/ip4/147.81.41.188/tcp/…`
- Message probe Win→AWS: `9562b71f-7163-4f26-ba59-6b69ad64e2c8`
  - Windows: `[OK] Message delivered successfully to 12D3KooWGvCW… (260ms)` + receipt cleared + marked delivered
  - AWS: `inbox_receive` of same id + `Sending delivery ACK for 9562b71f… to 12D3KooWD6vZ…`
- Message probe AWS→Win: `becec574-97d0-46b7-a62c-259517018ebb` text `v040-parity-probe-rev-from-aws`
  - Windows received (inbox + history received_count 4549→4550)

### Windows↔Phone (Pixel)

- Windows: inbound TRANSPORT-LANE from `12D3KooWR9io…` and outbound to LAN `192.168.0.134:9001`
- Windows: frequent `inbox_receive` from phone identity `9a230574…` (~every 60–90s)
- Windows: `Sending delivery ACK … to 12D3KooWR9io…` for multiple message ids
- Phone logcat: `peersDiscovered=2` steady (Windows + AWS)

### Phone↔AWS

- AWS diagnostics peers include `12D3KooWR9io…` (phone)
- AWS `inbox_receive` from sender `9a230574…` (phone) on cadence
- Phone peersDiscovered=2 includes AWS

## Operational notes

- AWS→Windows public `:9001` times out (NAT). Windows **outbound** dial to AWS is the working path; bootstrap config + `SC_BOOTSTRAP_NODES` both pin AWS.
- Windows config BOM (UTF-8 EF BB BF) broke JSON parse on relaunch — rewritten as UTF-8 no BOM. Backup: `config.json.bak-before-bootstrap-pin-20260910`.
- `bootstrap_nodes` now persists AWS multiaddr (W2/E6 closed on this deploy).
- Project `mimocode.json` neutralized (MiMo request failures) with `.bak-openrouter-override-20260910`.
- Disk reclaimed ~7GB by deleting Fresh `target/debug` after staging release exe.

## Still open for 0.4.0 tag

1. **Operator:** start mesh from Pixel app UI (permission dialog / service was
   STOPPED after reinstall; identity preserved — `initialized=true`, contacts
   present). Then confirm `peersDiscovered=2` in passive logcat.
2. Rule-8 independent APPROVE for transport diffs (merge-to-main gate).
3. BLE leg requires Windows BT radio reboot (hardware MT7921 wedge).
4. Tag `v0.4.0` only after operator final matrix including BLE isolation probe.

## Evidence paths

- Deploy dir: `MiMoSCMessengerFresh/tmp/DEPLOY_5f1f29bf_20260910T033521/`
- Android log pull: `MiMoSCMessengerFresh/tmp/android_logpull/`
- Rollback: `MiMoSCMessengerFresh/tmp/radio-238a8c53/` + earlier `radio-live-rollback-20260910/`
