# Runbook: Tri-node log triangulation verifier

Status: Active
Last updated: 2026-10-08
Created: 2026-10-07
Tool: `scripts/tri_node_verify.py` (package `scripts/trinode/`, stdlib only, Python 3.11+)
Device authority: `HANDOFF/RUNBOOK_PIXEL_PASSIVE_VERIFICATION_2026-09-11.md` -- the
collectors only read: `logcat -d`, `run-as ... cat`, `docker logs`, `GET /api/diagnostics`.

## What it does

Collects (or ingests) logs from three nodes -- Pixel 6a (Android), the local
Windows CLI node, the AWS relay container `scm-node` -- normalizes them to
JSONL events, estimates per-node clock skew, joins on message id, and scores
each message. The evidence standard is mandatory and encoded:

> A message is **VERIFIED** only with receiver-side decrypt + durable history
> write + delivery receipt, causally ordered within the measured skew. Transport
> ACKs (`[OK] Message delivered successfully`, `delivery_attempt ... outcome=acked`),
> UI counters and BLE local acceptance are parsed and displayed but never counted.

Classes: `VERIFIED`, `PARTIAL` (lists missing legs), `CONTRADICTED` (receipt or
delivered state without receiver decrypt while an endpoint log covers that time,
a `delivered`/`failed` state regression per
`scripts/verify_delivery_state_monotonicity.sh`, or a causality violation beyond
skew tolerance). The marker table (with `file:line` of every emitter) lives in
`scripts/trinode/markers.py`; this is the only place to edit when a log line changes.

Exit codes: `0` PASS, `1` FAIL (any non-VERIFIED message, failed scenario step,
contradiction), `2` INSUFFICIENT DATA (missing node logs, no chat message ids,
required message absent).

## Live pass (operator runs this; nothing here was run against devices when the tool was built)

Run from a checkout of this branch. Never build locally for this: use the CI APK
(`docs/runbooks/CI_APK_TO_PHONE.md`) and the published `testbotz/scmessenger` image.

1. Pre-flight: Pixel on wireless ADB, Windows node running, AWS container up.

```bash
adb connect <pixel-ip>:<port>            # wireless ADB, port from Developer options
adb devices -l
curl -s http://localhost:9876/api/diagnostics | head -c 300     # Windows node alive (CLI control API port 9876, NOT 9001)
```

2. Note the three libp2p peer ids (they make skew estimation and scenario steps exact):

```bash
export SCM_AWS_PEER=<aws relay peer id>        # from invite / bootstrap multiaddr /p2p/<id>
export SCM_WIN_PEER=<windows node peer id>
export SCM_AND_PEER=<android peer id>
```

3. Mark the start time, run the scenario on the devices (invite -> Windows joins AWS ->
   Android joins Windows -> send messages both directions, Android->Windows and
   Windows->Android), wait for receipts, then collect and verify. The AWS host is
   ephemeral: pass it per run, it is never stored (the manifest keeps only a salted fingerprint).

```bash
export SCM_AWS_HOST=<current aws ip or dns>
export SCM_AWS_USER=<ssh user>
export SCM_AWS_KEY=<path to ssh private key>

python scripts/tri_node_verify.py \
  --android --adb-serial <pixel-ip>:<port> \
  --windows \
  --aws --since 2h \
  --peer-id aws=$SCM_AWS_PEER --peer-id windows=$SCM_WIN_PEER --peer-id android=$SCM_AND_PEER \
  --scenario invite-aws-windows-android
echo "exit=$?"
```

Evidence lands in `tmp/evidence/<UTC date>/<run id>/` (`raw/`, `events.jsonl`,
`manifest.json` with sha256 of every raw file and the skew table, `verdict.md`,
`verdict.json`). `tmp/` is gitignored; reference the run id from
`HANDOFF/plans/V040_READINESS_3NODE_CONFIRMATION_2026-09-22.md`.

4. Re-score previously captured logs (any node subset; layout `<dir>/<node>/<files>`):

```bash
python scripts/tri_node_verify.py --from-dir <captured-dir> \
  --peer-id aws=... --peer-id windows=... --peer-id android=... --scenario invite-aws-windows-android
# flat directory:
python scripts/tri_node_verify.py --from-dir <dir> --map windows='scm.log.*' --map aws=docker.log
```

Useful options: `--require-msg <id>` (must be VERIFIED), `--tol-s 2` (ordering
tolerance), `--strict-markers` (reject inferred / weak scenario evidence),
`--android-tz-offset-min N` (zone-less Android `mesh_diagnostics.log` stamps;
logcat from this tool is UTC), `--aws-diag-url` (evaluated on the AWS host; default is
the local API port 9876), `--win-diag-url` (default `http://localhost:9876/api/diagnostics`;
the CLI control API listens on 9876, the old default 9001 was wrong), `--win-log-dir`.

Read the result: open `verdict.md`. Every scenario step shows its evidence lines as
`node:event@timestamp [marker] raw/<node>/<file>:<line>`. If a step says INFERRED or
WEAK, the code does not yet emit the explicit marker (see the gap list).

## Known limits of current builds (read before trusting a FAIL)

Run against the real 2026-09-29 capture (branch `backup/evidence/20260929-cell-test`) the
tool finds topology and ledger events and recovers clock skew (Android about +1.8 s
relative to the relay) but **zero message-level markers**: neither Windows nor the relay
logs any message id on receive, and the Android diagnostics excerpt contains none. With
today's code a first-time inbound message cannot reach VERIFIED, because the durable
history write is never logged. That is a code gap, not a tool bug; the tool reports
PARTIAL with the exact missing leg instead of guessing.

## Observability markers (transport status, routing feed, drops, stop)

Added by the 2026-10-08 passive-audit follow-up so every transport and message leg is
checkable from pulled logs alone, with no manual radio toggling. All of them are parsed by
`scripts/trinode/markers.py`; `verdict.md` gains a per-node **Transport availability**
table. A transport row reads `NO-MARKER` when that node's log never mentioned it (older
build, or the transport never initialised): the log is silent, so its state is UNKNOWN,
not "off".

| Marker | Emitted by | Meaning |
|---|---|---|
| `[TRANSPORT] kind=<tcp4\|tcp6\|quic|circuit|dcutr\|relay\|dcutr\|mdns\|ble\|wifi_direct\|wifi_aware\|cellular> state=<unavailable\|available\|listening\|connected\|error> [peers=<n>] detail=<reason>` | CLI/AWS `cli/src/transport_status.rs`; Android `transport/TransportStatus*.kt` | State at startup and on every change; `peers=` lines are the connected-peer count per transport every 5 min. `detail` spaces become `_`. BLE reasons include `no adapter`, `no D-Bus`, `adapter off`, `permission ... not granted`. |
| `[ROUTING] peer_seen peer=<short> source=<transport>` | `core/src/iron_core.rs routing_peer_seen` | The routing engine was fed a sighting. First per peer, then at most every 5 min. |
| `ledger_address_learned peer=<id> via=<ledger_exchange:peer16\|unknown> addr=<multiaddr>` | `core/src/store/ledger_entry.rs merge_shared_entries_via` | A node learned `peer`'s address from `via`'s ledger. The CLI now passes the source peer. |
| `rx_decrypt` / `rx_history` / `custody_accept` | `core/src/message_events.rs` | Receiver decrypt, durable history (`result=ok\|failed`), relay custody. Scored legs; a failed history write is NOT the history leg. |
| `[RX-DROP] msg=<id> stage=<stage> reason=<r>` | sibling PR | Receiver dropped a message. Shown as a note on the message and in a drops table; never changes the class. |
| `[MESH-STOP] requested\|swarm_shutdown ok\|timeout ms=\|rust_stop ok\|timeout ms=\|foreground_removed\|complete` | sibling PR | Stop sequence; `verdict.md` flags a sequence containing a timeout or lacking `complete` as NOT CLEAN. |
| `[INVITE] imported source=<join_bundle\|seed_import> ...` | Android `JoinMeshScreen.kt`, `MeshRepository.importSeedAddresses` | Invite redeemed (counts only). The CLI/FFI invite redeem path is #486/#501 and has no marker until it lands. |
| `[LOG-DEDUPE] key=<k> suppressed=<n> window_ms=<ms>` | Android `LogDeduper` | Spam lines (address snapshots, stats, mDNS) were rate-limited; this line says how many. |

Android `mesh_diagnostics.log` now rotates at 2 MB with 4 history files (10 MB total)
instead of 100 KB x 5 (about one rotation a minute), so a normal day stays on the device.

## Gap list: log markers that do not exist on origin/main yet

Protected = under `core/src/{crypto,transport,routing,privacy}/` (merge-blocked until
adversarial review). Format proposals match what `markers.py` already parses, so adding
the line lights up the tool with no further change.

| ID | Missing marker | Where it should be emitted | Protected |
|---|---|---|---|
| G1 | Explicit ledger-learned-address: `ledger_address_learned peer=<id> via=<id> addr=<multiaddr>` per accepted entry. Today only a count exists (`cli/src/main.rs:4440`, `swarm.rs:5992`); the tool must infer "Android learned AWS via Windows". | Per-entry in `core/src/store/ledger_entry.rs:2181` (`merge_shared_entries`); `via` is only known to the callers in `core/src/transport/swarm.rs:5811` and `:5992` | store: no; swarm callers: **yes** |
| G2 | Explicit receiver decrypt line with message id, e.g. `msg_rx peer=<id> msg=<id> decrypt=ok`. Today inferred from `Sending delivery ACK` (`cli/src/main.rs:3339`) and Android `onMessageReceived pairing` (`MeshRepository.kt:2201`). | `core/src/iron_core.rs:3650` (`receive_message` success path, before history add) | no |
| G3 | Durable history write on first receipt, e.g. `msg_rx_processed peer=<id> msg=<id> decrypt=ok history=written\|failed`. `core/src/iron_core.rs:3976` discards the result (`let _ = self.history_manager.add(...)`); Android `MeshRepository.kt:2489` (`historyManager?.add(record)`) is silent. This is the blocker for any VERIFIED. | `core/src/iron_core.rs:3976`; `android/.../MeshRepository.kt:2489` | no |
| G4 | Message id on relay custody acceptance, e.g. `custody_accept msg=<id> from=<peer> dest=<peer>`. The relay logs custody accepts without an id (`core/src/store/relay_custody.rs:816`, `:832`, `:839`; `relay_message_id` is in scope) and only a running count at `swarm.rs:4750`. | `core/src/store/relay_custody.rs:816/832/839`; any swarm-side accept path | store: no; swarm: **yes** |
| G5 | CLI invite/bootstrap import marker, e.g. `invite_imported seeds=<n> names=<peer-id>`. Android has one (`MeshRepository.kt:6603`); the CLI logs only `Dialing bootstrap` (`swarm.rs:4256`, protected, adequate) and `[SEED-DIAL]` heartbeats at DEBUG (`cli/src/seed_dial.rs:69`). Scenario step 1 can only prove "dial names AWS", not "from an invite". | CLI invite/bootstrap parsing path in `cli/src/main.rs` | no |
| G6 | CLI/core outbound send marker with message id at preparation: `delivery_state msg=<id> state=pending detail=message_prepared_local_history_written` (Android parity, `MeshRepository.kt:5877`). Windows-sent messages have no send-side marker, so sender attribution relies on the receipt line. | `core/src/iron_core.rs:1237` (`prepare_message_with_id`) | no |
| G7 | Level fixes so decisive lines survive the default INFO level: `Delivery ACK received` is DEBUG at `cli/src/main.rs:3455` (INFO at `:4608`); Android receipt-sent is `Timber.d` at `MeshRepository.kt:2945`; duplicate-inbound is `Timber.d` at `:2455`. | those lines | no |
| G8 | Android `mesh_diagnostics.log` (Timber file sink) stamps carry no zone and no year (`2026-09-29 11:10:44.782 D/Mesh:` was UTC-10 in the capture). Use logcat `-v year -v UTC` (the collector does) or emit UTC with offset. | Android file logger setup | no |
| G9 | Windows `scm.log.*` and AWS diagnostics expose no per-message history or per-message custody query; `/api/diagnostics` has counters only (`history_stats`, `custody_audit_count`), recorded as corroboration but never scored (UI/counter rule). | diagnostics endpoint | no |

## Self-test

```bash
python -m unittest discover -s scripts/trinode/tests -t scripts -v
```

Fixtures: `scripts/trinode/tests/fixtures/real/` holds small excerpts of the real
2026-09-29 capture (peer ids, public IPs and the Windows key replaced with fixture
values / TEST-NET addresses). Message-level scenarios are synthesized by
`scripts/trinode/tests/synth.py` using line shapes from `markers.py`; they are never
presented as captures.
