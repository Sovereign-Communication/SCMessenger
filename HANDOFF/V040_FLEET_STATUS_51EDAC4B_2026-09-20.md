# Fleet status — 3 nodes on `51edac4b` (2026-09-20)

Operator: Android **not blocked** — fresh install authorized and executed.

## Same-SHA cutover

| Node | Build | Identity | Notes |
|---|---|---|---|
| Windows CLI | `0.4.0 (51edac4:main)` `/version` | `12D3KooWD6vZQrUq...` **preserved** | Binary `tmp/radio-candidates/51edac4b/`; peers includes cloud |
| AWS cloud | `0.4.0 (51edac4b...)` boot log | `12D3KooWGvCWJNo...` **preserved** | `scripts/aws_deploy.sh` `IMAGE_TAG=testbotz/scmessenger:sha-51edac4` |
| Pixel 6a | APK `app-debug.apk` from CI run `35515094877` (tree `51edac4b`) | **New identity** (fresh install) | `adb uninstall` + `adb install` Success; app **not running** until operator launches UI |

## Post-cutover mesh (Tier A)

Commands: Windows `/api/diagnostics`, AWS `tmp/aws_log_slice.sh`.

- Windows: `path=DirectPreferred`, peers = cloud, `custody_audit_count=6195`
- AWS: boot → transient `[SEED-DIAL] peers=0` with malformed `p2p-circuit`
  candidates (`Missing destination peer id`) → **`[SEED-DIAL] peers=1 -- connected`**
  within ~1 min; `Gossipsub ... from 12D3KooWD6vZQrUq...` every minute
- Custody retention sweep live on AWS (TRN-04)

## Pixel

- Package `0.4.0` installed (versionCode 15); process not started (operator UI).
- Mesh logs absent until first launch (`files/logs/scmessenger-mesh.log` not created).
- **Next operator step:** open the app, complete onboarding/identity, allow mesh join.

## Open code PRs (orchestrator train)

| PR | Purpose |
|---|---|
| **#341** | androidTest compile fix (Mobile lane) |

## Still DISPATCHABLE freebuff (on main)

conn-limits (Rule-8), AND-06 A1/A2, watchdog test, IP-churn, beach-join P0–1.

## Tag checklist delta

- [x] Tier A same-SHA `51edac4b` + seed-dial/gossip evidence
- [x] CO-B-001 merged
- [x] Audit dispositions on main
- [ ] Pixel identity + mesh join (operator launch)
- [ ] Mobile green after #341
- [ ] Wave-1 freebuff implementation PRs
- [ ] Operator phone session + harness/logs scoring pack
- [ ] Final `v0.4.0` tag
