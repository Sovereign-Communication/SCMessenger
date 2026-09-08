# V040 /cto continuation package: three-node BLE certification

Status: FINAL, tracked canonical procedure
Last evidence refresh: 2026-09-08T09:09:56Z
Owner: the next `/cto` session

This tracked file is the sole owner of the V040 three-node test procedure,
provenance matrix, checkpoint schema, evidence gates, stop conditions, and
closeout. Files under `tmp/cto/` and `tmp/run-evidence/` are ignored evidence
references only; they are never the authority for a fresh session.

## 1. Load order and state ownership

Read, in order:

1. `AGENTS.md`
2. `docs/rules/FREEBUFF.md`
3. `HANDOFF/CTO_STATE.md`
4. `HANDOFF/V040_CTO_3NODE_BLE_CONTROLLER_PACKAGE_2026-09-08.md`
5. `HANDOFF/V040_CTO_BLE_ARCHITECTURE_2026-09-08.md`
6. every `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_*.md`
7. the referenced historical evidence under `tmp/`, if needed

`HANDOFF/V040_CTO_RESUME_2026-09-05.md` is an optional local resume aid when
present; it is not required for a fresh checkout and cannot override the
tracked package or `CTO_STATE.md`.

The tracked package owns policy and the test state machine. Each immutable
tracked checkpoint owns one execution snapshot. Node APIs and logs are read-only
inputs. Never infer current reachability from a checkpoint or old log.

At every stage, re-run the relevant commands. On command failure, preserve the
complete error, mark the value UNKNOWN, and stop rather than reusing stale data.

## 2. Fresh matrix to re-confirm before action

Run and save complete output for:

```bash
git branch --show-current
git rev-parse HEAD
git rev-parse HEAD^{tree}
git rev-parse origin/cto/v040-candidate-2026-09-02
git rev-parse origin/cto/v040-candidate-2026-09-02^{tree}
git tag -l | sort
```

Historical values from the last refresh, not current proof:

| Node/ref | Last observed value | Required fresh gate |
|---|---|---|
| Shared checkout | `cto/t2-disk-ruling-2026-08-31` at `0e0d54da` | Re-query Git |
| Origin candidate | `85cb4c67feb03d27fa004a2be6b1ce65b030eb06` | Re-query Git |
| AWS cloud node | `/health` 200; `/version` `0.4.0`, hash `85cb4c67...`; peer `12D3KooWGvCWJNoWnReNCT1q2LWb2gTbeBTa5sjxF49wZX3u2y31` | Query `/health`, `/version`, `/api/identity` |
| Windows CLI | EXE SHA `d2f75243c0ff07d6b433068001ce5fddcf2df33066c32e2367f3be12ddfb8385`; historical runtime log says `94cfe7d`; last fresh check found process/API absent | Reconnect exact EXE, then query process, `/health`, `/version`, `/api/identity` |
| Pixel 6a | `0.4.0`/14; installed APK SHA equals local SHA `5090a835698c57e1e75def9dc70abf7597acda67bcba35fa9d681aa0da2daa47` | Query ADB, package metadata, pull/hash APK |

Known mismatches must remain explicit:

- local candidate refs may differ from `origin/cto/v040-candidate-2026-09-02`;
- the Windows artifact designation says `85cb4c67` while its historical runtime
  reports merge-ref `94cfe7d`;
- APK equality with a local file does not prove origin-candidate source
  provenance;
- no final `v0.4.0` tag was present in the last refresh.

Do not claim a same-candidate fleet until these are freshly reconciled or ruled
acceptable by the operator.

## 3. Immutable checkpoint schema

Create a new tracked file after each stage; never overwrite one:

`HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_<UTC-BASIC>_<STAGE>.md`

Stages:

- `PREFLIGHT`: refs, tags, all hashes, identities, reachability, blockers
- `NODE_READY`: exact Windows process/API, AWS API, and Pixel ADB/package state
- `RADIO_ISOLATED`: Bluetooth on, Wi-Fi/mobile data off, synchronized markers
- `PROBE_SENT`: one label, exact content, message ID, capture paths
- `CORRELATION`: complete post-marker evidence and verdict
- `CLEANUP`: restored connectivity, final node state, final verdict

Every checkpoint must contain UTC time, exact commands, complete output paths,
all three identities and artifact hashes, and explicit PASS, FAIL, BLOCKED, or
UNVERIFIED values. The final checkpoint must link a short tracked final handoff.
Ignored `tmp/` files may be linked as evidence, but may not replace the tracked
checkpoint or final handoff.

## 4. Exact execution sequence

### Phase 0: preflight

1. Read this package, the architecture note, and all existing checkpoints.
2. Re-derive Git refs/tags and the three-node matrix.
3. Query AWS `/health`, `/version`, `/api/identity`.
4. Query Windows `/health`, `/version`, `/api/identity`.
5. Query Pixel ADB state, package metadata, and pulled APK SHA256.
6. Write `PREFLIGHT`.
7. Stop if Windows is unreachable, any identity changes, or any artifact hash
   changes without an approved explanation.

### Phase 1: Windows node readiness

1. Confirm no other SCMessenger CLI owns port `9876` or node listen ports.
2. Verify the exact recorded EXE hash.
3. Start one instance only, with fresh repo-local log capture, recording the
   expanded command, PID, working directory, and hashes in `NODE_READY`:

```powershell
$exe = (Resolve-Path 'tmp/radio-85cb4c67/scmessenger-cli-85cb4c67.exe').Path
$dir = (Resolve-Path 'tmp/radio-85cb4c67').Path
$out = Join-Path $dir 'windows-cli-cto-live-<UTC-BASIC>.log'
$err = Join-Path $dir 'windows-cli-cto-live-<UTC-BASIC>.err.log'
Start-Process -FilePath $exe -ArgumentList 'start' -WorkingDirectory (Split-Path $exe) -RedirectStandardOutput $out -RedirectStandardError $err -PassThru
```

4. Require one process, `/health` 200, fresh `/version` and `/api/identity`,
   adapter availability, GATT Service Provider startup, LE advertising, and
   SCM service scanning.
5. If runtime still reports `94cfe7d`, same-candidate certification remains
   BLOCKED pending an explicit provenance ruling.

### Phase 2: Pixel BLE readiness

1. Confirm authorized ADB `device` and installed/local APK hash equality.
2. Open `com.scmessenger.android/.ui.MainActivity` if necessary.
3. Use the app's real mesh-service control; do not shell-force the service.
4. Start Android logcat and app diagnostics before the service action.
5. Require one repository-owned BLE stack: initialization, advertisement, GATT
   registration, and active scan. Write `NODE_READY`.

### Phase 3: BLE-only isolation

1. Keep Bluetooth enabled.
2. Disable Wi-Fi and mobile data through normal Pixel controls.
3. Record both radio states and relevant ADB/network diagnostics.
4. Keep AWS healthy and Windows running; do not trigger scans, alter contacts,
   resend messages, or change routes.
5. Write synchronized UTC markers to Windows and Android captures.
6. Write `RADIO_ISOLATED`; if isolation cannot be demonstrated, BLE is
   UNVERIFIED and the probe must not proceed.

AWS is a healthy cloud node and route-exclusion observer. The probe must not
fall back through AWS, TCP, mDNS, Wi-Fi, cellular, or relay delivery.

### Phase 4: exactly one probe

1. Create a label such as `V040-BLE-<UTC-BASIC>-01`.
2. Send exactly one message from Pixel to the authenticated Windows contact.
3. Record exact content and Android message ID.
4. Do not retry, send another message, reopen the conversation, or rescan until
   correlation completes.
5. Write `PROBE_SENT`.

### Phase 5: correlation

Capture the complete post-marker interval from Windows, Android logcat/app logs,
and AWS logs/API. BLE PASS requires all of:

- same label/content and message ID, or an unambiguous documented mapping;
- Windows BLE-specific GATT ingress plus successful decrypt/handling;
- matching Android BLE transport evidence;
- no TCP, mDNS, Wi-Fi, cellular, relay, AWS route, or generic swarm delivery;
- intended on-device receipt, not merely API acceptance or outbox presence.

Adapter availability, advertising, scanner registration, generic `inbox_receive`,
`routing_decision`, relay success, and API `accepted` are insufficient. Missing
transport-specific proof is UNVERIFIED or FAIL with the missing event named.
Write `CORRELATION` before ending evidence review.

### Phase 6: cleanup

1. Stop capture processes only; do not stop nodes unless separately directed.
2. Restore Wi-Fi and mobile data.
3. Re-query AWS, Windows, and Pixel with the same read-only checks.
4. Write `CLEANUP` and the final tracked handoff with complete evidence paths.
5. Preserve all earlier checkpoints and logs.

## 5. Stop conditions and completion

Stop before the probe if Windows is unreachable, its identity changes, the EXE
hash differs, AWS is not healthy/candidate `85cb4c67...`, or the Pixel APK hash
changes without explanation. Stop if radio isolation cannot be shown. Stop after
the first probe until correlation completes. Never send extra probes to improve
weak evidence. Keep tag/release decisions on hold.

BLE is UNVERIFIED until the transport-specific correlation gate passes. Cellular
is UNVERIFIED in this workflow. Same-candidate certification is BLOCKED until
Windows and Pixel provenance are resolved or formally ruled acceptable.

No BLE pass, same-candidate certification, cellular pass, final tag, or release
is claimed by this tracked package.

## 6. Historical evidence references

These ignored files may contain prior output, but they are not authorities:

- `tmp/cto/V040_CTO_3NODE_BLE_CONTROLLER_PACKAGE_2026-09-08.md`
- `tmp/cto/V040_CTO_BLE_ARCHITECTURE_2026-09-08.md`
- `tmp/cto/V040_RADIO_LEG_RESTART_HANDOFF_85cb4c67_2026-09-07.md`
- `tmp/cto/V040_RADIO_LEG_STATUS_2026-09-07.md`
- `tmp/radio-85cb4c67/windows-cli-live-2026-09-08.log`
- `tmp/run-evidence/reval/aws-rebuild-85cb4c67/aws-85cb4c6.log`

A fresh checkout must use this tracked file and new tracked checkpoints first.
