# Runbook: CI APK to the Operator's Phone (fresh install)

Status: Active
Created: 2026-09-22
Device authority: `HANDOFF/RUNBOOK_PIXEL_PASSIVE_VERIFICATION_2026-09-11.md`
— a seat may install an APK and pull logs passively. Everything else on the
device belongs to the operator.

## 1. Source the APK from CI (never build it locally for installs)

```bash
gh run list --workflow mobile.yml --branch main --limit 3 \
  --json databaseId,headSha,conclusion,createdAt \
  --jq '.[] | "\(.databaseId) \(.headSha[0:8]) \(.conclusion) \(.createdAt)"'
# pick the newest run with conclusion == "success" on main
mkdir -p tmp/ci-apk
gh run download <run-id> -n android-debug-apk -D tmp/ci-apk/<run-id>/
```

Verify what you got:

```bash
cat tmp/ci-apk/<run-id>/output-metadata.json
# applicationId must be com.scmessenger.android
# versionName/versionCode recorded for the install note
# the run's headSha is the provenance commit -- record it
```

## 2. Connect the device

The Pixel 6a uses wireless ADB (see `docs/rules/ANDROID.md`). It will not
appear unless it is awake, unlocked once, and on the same LAN.

```bash
adb devices -l                 # check for an existing serial first
adb mdns services              # discovery (takes a few seconds; retry once)
```

If nothing appears, ask the operator to wake/unlock the phone and confirm
wireless debugging is on; then retry `adb mdns services` and
`adb devices -l`. If the serial was paired before but shows `offline`,
`adb disconnect <ip:port>` and rediscover. First-time pairing
(`adb pair <ip:port> <code>`) requires the operator to read the code off the
phone — a seat cannot complete pairing alone.

## 3. Install

"Fresh install" in this runbook means: install the CI APK over the existing
app with data preserved (replace-install). A wiped-data install needs
`pm clear` and is operator-gated (see the Pixel runbook; never without
identity export + explicit approval).

```bash
SER=<serial>
adb -s "$SER" install -r tmp/ci-apk/<run-id>/app-debug.apk
# Success line: "Success"
# If INSTALL_FAILED_UPDATE_INCOMPATIBLE: signature mismatch -- STOP and
# report; that means the installed app came from a different signing key
# and the operator decides (an uninstall would wipe identity).
```

Optional, only if the app is not already running (per the seat checklist):

```bash
adb -s "$SER" shell am start -n com.scmessenger.android/.ui.MainActivity
```

Then STOP. Notify the operator: "APK from run <run-id> (commit <sha>,
versionName <v>) installed; please open the app and exercise the path."

## 4. Passive verification (logs only)

```bash
mkdir -p tmp/evidence/<run-id>
adb -s "$SER" logcat -d -v time > tmp/evidence/<run-id>/logcat.txt
adb -s "$SER" exec-out run-as com.scmessenger.android \
  cat files/logs/scmessenger-mesh.log > tmp/evidence/<run-id>/mesh.log
```

`mesh.log` is often UTF-16 (`FF FE` magic) — decode before grepping. Score
results on receiver-side evidence, never transport ACKs or UI counters.

## 5. Cleanup

```bash
rm -rf tmp/ci-apk/<run-id> tmp/evidence/<run-id>   # after the evidence is recorded
```

`tmp/` is not a store. If the evidence matters, it goes into a HANDOFF file
first, then the tmp copy goes.
