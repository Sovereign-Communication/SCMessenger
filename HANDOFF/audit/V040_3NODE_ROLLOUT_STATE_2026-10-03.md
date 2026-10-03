# v0.4.0 three-node rollout state and log triangulation

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Date:** 2026-10-03
**Author:** Buffy (Freebuff session), 0.4.0 rollout closeout
**Tag:** `v0.4.0` = `21bbfe1d7a769dd7c3d1c3916de0ced280c8d3b7`
**Artifacts built from:** `58c8970b902b6e16c4f0b1dce2e87d7767d4611e` (`main`)
**Build provenance** (from the CI artifact itself, not inferred):
`git_sha=58c8970b... git_ref=main version=0.4.0`

## Rollout state

| Node | Build | Health | Identity | Source of proof |
|---|---|---|---|---|
| Windows local | `0.4.0 (58c8970b)` | `GET /health` -> `{"status":"healthy"}` | preserved across binary swap | supervisor + node log |
| AWS cloud node | `testbotz/scmessenger:sha-58c8970` | container `Up 2 hours`, mount guard passed | preserved (`/opt/scm-relay-data` retained) | `docker logs` |
| Pixel 6a (Android) | **pre-fix 0.4.0 vc15** | service foreground, log live | preserved, untouched | `files/logs/scmessenger-mesh.log` |

Two of three nodes are on the tagged build. The Pixel is not, and cannot be
until signing is resolved — see
`HANDOFF/audit/ANDROID_SIGNING_LINEAGE_2026-10-03.md`.

## Three-way log correlation

All three node logs were pulled this session:

| Source | Lines | Path pulled |
|---|---|---|
| Windows | 2495 | `%LOCALAPPDATA%/scmessenger/logs/scm.log.2026-10-03-08` |
| AWS | 7366 | `docker logs scm-node` |
| Pixel | 15471 | `files/logs/scmessenger-mesh.log` via adb |

### Version

| Node | Evidence |
|---|---|
| Windows | `scmessenger/0.4.0/full/relay/12D3KooWGvCWJNoWn...` |
| AWS | `CLI Version: 0.4.0 (58c8970b`, `scmessenger/0.4.0/full/relay/12D3KooWD6vZQrUq...` |
| Pixel | `scmessenger/0.4.0` |

All three report 0.4.0. Only Windows and AWS carry the tagged commit
`58c8970b`; the Pixel's build is not from this tag (F-2).

### Topology — Windows and AWS mutually corroborate

The same link is described from both ends, and the descriptions agree:

- Windows sees AWS as
  `69805e175cdc59b244f36001303e0b2e735f729557d2069a02688c0e69764a7c`
- AWS sees Windows as
  `30d0fa678c218b225bd9c20c262b2aededc9e8cd5cd44c45187f8d71bf05967e`

The AWS log records the outbox flush against
`peer_id=69805e17...` at `08:00:39.423650Z`, which is precisely the id
Windows publishes in its gossip topic subscription
(`/scmessenger/peer/69805e17`). Two independent processes agree on peer
identity and direction — the AWS node independently reports this host's
external address as `147.81.41.188` (`api.ipify.org`).

Relay custody is live: AWS logs
`Registered relay peer ... (agent: scmessenger/0.4.0/full/relay/12D3KooWD6vZQrUq...)`
and `[CIRCUIT-RELAY] Registered relay peer ... addr_count=10`.

### Health

All three were healthy at pull time. The Pixel's log shows ongoing
maintenance cycles and behaviour-adjustment ticks through `08:40:06Z`.

## F-1 (new finding) — the Pixel stores public keys in a `peer_id` field

From the device's own `files/ledger.json`:

```json
{
  "peer_id":    "30d0fa678c218b225bd9c20c262b2aededc9e8cd5cd44c45187f8d71bf05967e",
  "public_key": "30d0fa678c218b225bd9c20c262b2aededc9e8cd5cd44c45187f8d71bf05967e",
  "observed_peer_ids": ["30d0fa67..."],
  "multiaddr": "/ip4/192.168.0.121/tcp/443",
  "success_count": 42
}
```

`peer_id` and `public_key` are byte-identical, and both hold a 64-hex Ed25519
public key. The Rust nodes use a libp2p peer id (`12D3KooW...`, a multihash)
in that same conceptual slot. Measured consequences:

- All 5 ledger entries on the device are 64-hex keys; **none** is a
  `12D3KooW...` libp2p id.
- The Pixel does **not** know the third peer that both Rust nodes observe
  (`396019d1...`), because it keys peers differently.
- It is represented to the mesh at **gossip-topic level only**. Windows logs
  `Peer 12D3KooWDgLQ8jn8... subscribed to topic: /scmessenger/peer/69805e17`,
  and the core then applies `GHOST-IDENTITY-001 skip auto-subscribe ghost peer
  topic` for those keys.

So the Pixel participates in discovery but never establishes a libp2p peer
connection, and never appears in either Rust node's peer list. This is a
**schema/key-scheme divergence between the Android and Rust ledgers**, not a
transport fault: the Android side writes a public key where the Rust side
expects a peer id.

Not yet triaged as a ticket — it needs a decision on which scheme is
canonical, which is an architecture call rather than a mechanical fix.

## What remains before 0.4.0 is fully rolled out

1. Operator sets `SCMESSENGER_KEYSTORE_PASSWORD` and `SCMESSENGER_KEY_PASSWORD`
   (see the signing-lineage handoff).
2. Re-dispatch the release pipeline; confirm the signed APK/AAB is produced.
3. Decide the Pixel's migration path, accepting the documented identity-loss
   tradeoff, or pin one debug keystore across CI and the fleet.
4. Re-run this triangulation once the Pixel carries the tagged build, so all
   three sources corroborate commit as well as topology.
5. Triage F-1 (`peer_id` schema divergence) as its own ticket.
