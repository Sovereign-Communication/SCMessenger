# V040 CTO three-node BLE final handoff — blocked preflight

- Status: `BLOCKED`
- UTC: `2026-09-08T12:09:57Z`
- Controller: Freebuff `/cto`
- Stage reached: `PREFLIGHT` only
- Checkpoint: `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260908T120957Z_PREFLIGHT.md`

## Verdict

The tracked V040 procedure stopped before node readiness and before any radio
change or message probe. The Windows CLI node was not running, had no local
`9876` listener, and `curl http://127.0.0.1:9876/health` failed with curl exit
`7` and HTTP status `000`. This is an explicit Phase 0 stop condition in the
tracked controller package.

No node was started by this session. No Bluetooth, Wi-Fi, mobile-data, contact,
route, or message state was changed. No probe was sent. BLE, cellular,
and same-candidate three-node certification remain `UNVERIFIED`; no final tag or
release readiness is claimed.

## Fresh state summary

- Git branch: `cto/t2-disk-ruling-2026-08-31`
- Git HEAD/tree: `0e0d54dab43a3ab375e8e1f799d8d4a4168033de` /
  `2f371cef5115bd79bd0e97e2240011586723775a`
- Candidate ref/tree: `origin/cto/v040-candidate-2026-09-02` /
  `85cb4c67feb03d27fa004a2be6b1ce65b030eb06` /
  `0989624382eb06e3792babeaf7f60333202357b9`
- Fresh tags include `v0.4.0-rc.1`; no final `v0.4.0` tag.
- AWS EC2 read-only discovery found running instance
  `i-0b41aab756eabd514` at `18.234.62.247`; the old matching instance
  `i-0b735c4f26aea42ed` is stopped and was not touched.
- AWS `/health` returned HTTP 200; `/version` returned `0.4.0` and exact
  candidate commit `85cb4c67feb03d27fa004a2be6b1ce65b030eb06`; `/api/identity`
  returned peer `12D3KooWGvCWJNoWnReNCT1q2LWb2gTbeBTa5sjxF49wZX3u2y31`.
- Pixel ADB is authorized as a Pixel 6a (`bluejay`, Android 17/API 37), with
  package `0.4.0`, versionCode `14`, app PID `11582`, and installed APK SHA256
  `5090a835698c57e1e75def9dc70abf7597acda67bcba35fa9d681aa0da2daa47`.
  App-owned diagnostics identify peer `12D3KooWR9ioPPRJ2tGPbWj9NVKXAve2iwZX4csLbDd1Tn6Hpi3B`.
- Expected Windows artifact SHA256 is
  `d2f75243c0ff07d6b433068001ce5fddcf2df33066c32e2367f3be12ddfb8385`.

## Next action

An operator or authorized Windows lane must start exactly one approved candidate
Windows CLI instance using the procedure in
`HANDOFF/V040_CTO_3NODE_BLE_CONTROLLER_PACKAGE_2026-09-08.md`, then rerun the
fresh Phase 0 and Phase 1 checks. Resume only after Windows `/health`,
`/version`, `/api/identity`, process ownership, and artifact provenance are
freshly available. Do not reuse this blocked checkpoint as current reachability
evidence.
