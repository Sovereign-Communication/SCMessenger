# V040 3-Node Checkpoint — PREFLIGHT (read-only readiness re-audit)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

UTC: 2026-09-17T19:54:44Z
Stage: PREFLIGHT (nodes already live; no node was stopped, started, or
reconfigured; no radio isolation and no probe phase was executed)
Controller: Freebuff lane, CTO seat, driven by operator directive
Package: `HANDOFF/V040_CTO_3NODE_BLE_CONTROLLER_PACKAGE_2026-09-08.md`

## Verdict

| Area | Verdict | Basis |
|---|---|---|
| Three-node reachability | PASS | Windows `/health` 200, AWS `/health` 200, Pixel authorized ADB `device` |
| Identity stability | PASS for Windows + AWS; UNVERIFIED for Pixel peer id | `/api/identity` on both nodes; Pixel peer id not read from the app UI |
| Artifact provenance | AWS PASS, Pixel PASS, Windows PARTIAL | image tag equals runtime hash; APK byte-identical to local build; Windows EXE is an unprovenanced in-tree build |
| Same-candidate fleet | **FAIL / BLOCKED** | three different revisions (table below) |
| Live 3-node message path | PASS (IP transport, not BLE isolation) | Pixel log shows delivery to the cloud node in 265 ms today |
| BLE-only isolation + probe | NOT RUN — UNVERIFIED | out of scope for a read-only re-audit |
| `v0.4.0` final tag | FAIL | latest tag is `v0.4.0-rc.1` |

## Fresh repo state (exact commands)

```
git branch --show-current   -> feat/v040-multi-transport-store-forward
git rev-parse HEAD          -> 629a3eefa2c7eeca2c3d3af3d0cb49bd2bc6626f
git rev-parse HEAD^{tree}   -> 8428b1f4a55e10aa9f64ccab0fb587db371df268
git rev-parse origin/main   -> eb55756957e2d01f558321b374258a3c05750181
git tag -l | sort           -> ... v0.3.5, v0.4.0-rc.1   (no final v0.4.0)
```

Merged into `origin/main` today (audited this session, each 33-34/34 lanes
green at merge): #288 carrier (V040 multi-transport store-and-forward),
#289, #283. #290/#292/#293/#294/#296/#297 landed on the carrier earlier.

## Node matrix

| Node | Revision running | Artifact hash | Distance behind `origin/main` | Identity |
|---|---|---|---|---|
| Windows CLI (local) | core `597e2c72` (2026-09-15 07:58 -1000), CLI `git_hash 7a2c16c3` (2026-09-16 10:41 -1000) | `target/release/scmessenger-cli.exe` sha256 `9b251c276d6b3f5167bd6c2ab159d007a42c151dd4957e4ec0ce1798f2d6eecb` (22,779,904 B, mtime 2026-09-16 11:19) | core 68 / CLI 43 | peer `12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw`, nickname `Claude-Windows-Driver`, identity `985a25f9…5826` |
| AWS cloud node | core `6acaa231` (2026-09-16 10:33 -1000, "nodes never subscribed to their own topic") | image `testbotz/scmessenger:sha-6acaa23` (tag matches runtime hash — provenance PASS) | 44 | peer `12D3KooWGvCWJNoWnReNCT1q2LWb2gTbeBTa5sjxF49wZX3u2y31` (unchanged from prior checkpoints), identity `37eb7561…d006`, **nickname `null`** |
| Pixel 6a `bluejay` | app `0.4.0`, `versionCode=15`, installed 2026-09-15 21:04, updated 2026-09-16 01:17; Android **17** | base.apk sha256 `4a0972b29556c4dadd0739667e2935aaededb5366ab6bed7e7c5a9f046ffe86b` — **byte-identical** to local `android/app/build/outputs/apk/debug/app-debug.apk` (same size 56,524,663 B, mtime 2026-09-16 01:16) | n/a (APK predates today's merges) | peer id not captured (UNVERIFIED) |

Both node cores verified as **ancestors of `origin/main`** (`git merge-base
--is-ancestor`), so nothing unreviewed is running — but none of the three is
at main's tip.

## Live evidence (2026-09-17)

Windows (PID 18124, listening 127.0.0.1:9876):
`/health {"status":"healthy"}`; `/version 0.4.0 build_time 2026-09-16T21:12:58Z
core_provenance 0.4.0 (597e2c72:feat/v040-multi-transport-store-forward)`.

AWS `i-0b41aab756eabd514` (us-east-1, public 18.234.62.247, private
172.31.18.74, launched 2026-09-07): `/health {"status":"healthy"}`;
`/version git_hash 6acaa2317f08b8095316c7775e482ef965dbc913`;
container `scm-node | testbotz/scmessenger:sha-6acaa23 | Up 23 hours`.
Note: fresh tag query returned only ONE instance — the A3 duplicate-tag
finding from `V040_3NODE_RCA_2026-09-09.md` no longer reproduces.

Pixel (wireless ADB `adb-26261JEGR01896-6pHTac`), app PID 22189. App file log
`files/logs/scmessenger-mesh.log` tail captured to
`tmp/cto_audit_20260917/pixel-mesh-tail.log` shows within minutes of this
checkpoint:
- `ledger_canonical_hex_live` x29 — libp2p peer ids rewritten to hex on
  ledger write (cloud-node id -> `69805e17…4a7c`);
- one `ROUTE_DECISION … route=direct destination=12D3KooWGvCWJNoWnReNCT1q2LWb2gTbeBTa5sjxF49wZX3u2y31`
  followed by `[OK] Message delivered successfully to 12D3KooWGvCWJNoWn… (265ms)`;
- Gossipsub `sc-mesh` messages received from BOTH the cloud node
  (`12D3KooWGvCWJNoWn…`) and the Windows node (`12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw`);
- one `[WARN] sending to a recipient with no contact record; proceeding` —
  the send still delivered.

`adb logcat -d -b crash` last entry is 2026-09-15 17:35 (lifecycle destroy
crash) — no crash on 09-16 or 09-17. `adb logcat` main buffer carries almost
no app signal (10 of 2,724 lines matched) — the app file log, not logcat, is
the usable Pixel evidence channel.

## Blockers for same-candidate 3-node certification

1. **Revision skew**: Windows 597e2c72 / AWS 6acaa231 / Pixel APK of
   2026-09-16 — three different revisions, none at `origin/main` `eb557569`.
   The Windows node predates the 2026-09-16 own-topic transport fix that the
   cloud node already runs.
2. **Redeploy required**: certifying the fleet at today's merged main needs a
   rebuilt Windows CLI, a re-pushed image (new `sha-*` tag), and a rebuilt +
   reinstalled Pixel APK.
3. **No final `v0.4.0` tag** (only `v0.4.0-rc.1`).
4. Minor: AWS reports `nickname: null`; AWS `/version build_time` is empty;
   Pixel peer id not read (UI read required).

## Evidence paths

- `tmp/cto_audit_20260917/pixel-base.apk` (pulled, sha256 above)
- `tmp/cto_audit_20260917/pixel-main.log` (2,724 lines, logcat sample)
- `tmp/cto_audit_20260917/pixel-mesh-tail.log` (60 lines, app file log tail)

These `tmp/` artifacts may be linked as evidence but do not replace this
tracked checkpoint.
