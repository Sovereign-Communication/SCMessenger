# v0.4.1 three-node rollout state and log triangulation

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Date:** 2026-10-03 (rewritten same day after the v0.4.1 rollout)
**Author:** Buffy (Freebuff session), release closeout
**Tag:** `v0.4.1` = `dcd67b94ecb4a5c4602e659ee0539b9e5e56fe3c`
**Build provenance** (read from the CI artifacts themselves, not inferred):
`git_sha=dcd67b94ecb4a5c4602e659ee0539b9e5e56fe3c git_ref=v0.4.1 version=0.4.1`

## v0.4.1 supersedes v0.4.0

`v0.4.0` (`21bbfe1d`) was cut before #431 (release chain) and #433 (signing
diagnostics), so neither fix had ever executed. `v0.4.1` is the first tag that
post-dates them and the first tag in project history whose push actually
started `release.yml`.

## Release chain: PROVEN, and no rebuild loop

Run `37116380941`:

```
37116380941  Multi-Platform Release Pipeline  ev=push  br=v0.4.1
```

Previously every tag push produced **zero** release runs, because
`auto-tag-release.yml` pushes with `GITHUB_TOKEN` and GitHub does not trigger
workflows from `GITHUB_TOKEN` pushes. #431 fixed that; this run is the proof.

**Loop check.** `auto-tag-release.yml` did not run on the tag push; no second
tag was created (only `v0.4.0`, `v0.4.0-rc.1`, `v0.4.1` exist); run count on
the tagged SHA went 10 -> 11, i.e. exactly one new run.

**Signing preflight: still failing, and now legible.** `Build Android Release`
failed at the preflight with, verbatim from the job log:

```
##[error]SCMESSENGER_KEY_ALIAS does not exist in the decoded keystore.
         keytool: keytool error: java.lang.Exception: Alias <***> does not exist
##[error]Fix: set SCMESSENGER_KEY_ALIAS to an alias that exists.
```

#433's classifier is working — the job log now contains keytool's own message
and a specific remedy, where every historical run said only "is not present in
the decoded keystore".

**This corrects the standing diagnosis, and I was wrong about it.** I previously
concluded from the local JKS header that the alias was correct (`scmessenger`,
lowercase, byte-exact) and that the *password* must be wrong. CI says the alias
does not exist. The local file and the `SCMESSENGER_KEYSTORE_BASE64` secret are
therefore **not** the same keystore, even though the local `.b64` is
byte-identical to the local `.jks`. Treat the **alias** secret as the primary
suspect, and re-verify against the decoded secret rather than the local file.
The resolution procedure is unchanged and still operator-held.

## Rollout state

| Node | Build | Health | Identity | Proof |
|---|---|---|---|---|
| Windows local | `0.4.1 (dcd67b94)` | `/health` -> healthy | **preserved**: `12D3KooWGvCWJNoWn...` unchanged | supervisor + node log |
| AWS cloud node | `testbotz/scmessenger:sha-dcd67b9` | healthy, mount guard passed | **preserved**: `37eb7561...`, `seniority_timestamp=1788524000` | deploy script + `docker logs` |
| Pixel 6a (Android) | **pre-fix 0.4.0 vc15** | unknown, device offline | preserved, untouched | see below |

**These binaries are behaviourally identical to the `58c8970b` ones they
replace.** `git diff 58c8970b v0.4.1 -- cli/src core/src Cargo.toml Cargo.lock`
returns **zero** files. The swap buys current provenance and a proven release
chain, not new behaviour: #432 is the Android `MeshForegroundService` and #433
is a CI workflow, neither of which the CLI binary compiles.

## How each node was rolled

**Windows.** Graceful `/api/shutdown` (supervisor recorded clean exit
`code=0`), binary replaced (sha256 `b7389d52...`, matches the artifact),
supervisor relaunched with identical args (`start`). Windows' binary does not
print a version on `--version`, so the running node's own log is the authority:
`CLI Version: 0.4.1 (dcd67b94...)`.

**AWS.** `scripts/aws_deploy.sh` with
`IMAGE_TAG=testbotz/scmessenger:sha-dcd67b9` and the documented `ec2-user@`
credential. The script's mount guard confirmed `/opt/scm-relay-data -> /data`
after restart and it reported `persisted identity already present, not
overwriting`.

## Peer topology — two-node mutual corroboration

Windows and AWS describe the same link from opposite ends:

- Windows sees AWS as `69805e175cdc59b2...`
- AWS sees Windows as `30d0fa678c218b225...`

Post-swap AWS logs `Connection established to [30, d0, fa, 67, ...]` (4 times
in the window) while Windows logs `Connection established to [69, 80, 5e, 17,
...]` — the same two keys, both directions. Relay custody is live on both:
`[CIRCUIT-RELAY] Registered relay peer` and `[CUSTODY] Registered local
identity with peer` both appear after the swap.

## Reconnect storm: absent, with a counting caveat

After the swap the Windows log contains **zero** `ConnectionClosed` /
`connection closed` events. A naive `grep -i disconnect` returns 29, but
reading the lines shows 28 are `Discovery dial to ... was not disconnected` —
ordinary discovery probes, not closes. The single genuine event is the AWS
container restart:

```
11:09:55.392  [ERROR] Disconnected from 12D3KooWGvCWJNoWn...
11:09:55.393  Self-heal: queueing redial for disconnected bootstrap peer ...
```

It self-healed in 49 s:

```
11:10:43.811  Connection established to [69, 80, 5e, 17, ...]
11:10:44.228  [CIRCUIT-RELAY] Registered relay peer relay_peer_id=12D3KooWGv...
```

One disconnect caused by the deployment, one reconnect, no storm, no repeat
flapping.

**Caveat that matters.** This is evidence about the CLI nodes only. The storm
originally observed (207 closes, 140 in one hour) was Pixel-to-Windows
flapping driven by the Android stop/start defect. Only a Pixel log can
demonstrate that symptom is gone, and no Pixel log exists yet.

## Pixel: operator-blocked AND offline

Two independent reasons there is no current Pixel evidence:

1. **Signing.** Needs `SCMESSENGER_KEYSTORE_PASSWORD` and
   `SCMESSENGER_KEY_PASSWORD`; per the corrected diagnosis above, also re-verify
   `SCMESSENGER_KEY_ALIAS` against the decoded secret. See
   `HANDOFF/audit/ANDROID_SIGNING_LINEAGE_2026-10-03.md`.
2. **Device unreachable at pull time.** No USB device, `adb mdns services`
   empty, and last-known address `192.168.0.121:5555` actively refused — the
   Pixel is off WiFi.

No device data was touched at any point in this work.

**The AND-SS-001 stop/start fix reaches the Pixel only through an Android APK.**
No CLI release can deliver it. Until the signing secrets are set and the device
is reachable, the mesh stop button remains broken on that device and the fleet
is **2 of 3**.

## F-1 (carried forward, still open) — Pixel stores public keys in a `peer_id` field

From the device's `files/ledger.json`, captured before the device went offline:

```json
{
  "peer_id":    "30d0fa678c218b225bd9c20c262b2aededc9e8cd5cd44c45187f8d71bf05967e",
  "public_key": "30d0fa678c218b225bd9c20c262b2aededc9e8cd5cd44c45187f8d71bf05967e",
  "observed_peer_ids": ["30d0fa67..."],
  "multiaddr": "/ip4/192.168.0.121/tcp/443",
  "success_count": 42
}
```

`peer_id` and `public_key` are byte-identical, both a 64-hex Ed25519 key. The
Rust nodes use a libp2p peer id (`12D3KooW...`, a multihash) in that slot.
Consequences: all 5 ledger entries are 64-hex, none is a libp2p id; the Pixel
does not know the third peer both Rust nodes see (`396019d1...`); and it is
represented at **gossip-topic level only** (Windows logs
`subscribed to topic: /scmessenger/peer/69805e17`, then the core applies
`GHOST-IDENTITY-001 skip auto-subscribe ghost peer topic`).

A schema/key-scheme divergence between the Android and Rust ledgers, not a
transport fault. Which scheme is canonical is an architecture decision.

## Remaining to reach 3 of 3

1. Operator sets the signing secrets and re-verify the alias against the
   decoded secret; re-dispatch the release pipeline.
2. Operator decides the Pixel migration path, accepting the documented
   identity-loss tradeoff in `ANDROID_SIGNING_LINEAGE_2026-10-03.md`.
3. Bring the Pixel back on the network and re-run this triangulation so all
   three sources corroborate commit as well as topology.
4. Triage F-1 as its own ticket.
