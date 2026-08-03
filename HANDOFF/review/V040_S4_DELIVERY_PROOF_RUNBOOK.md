# 040-S4 RUNBOOK -- fresh CLI<->Android-emulator E2E delivery proof at current HEAD

Status: READY (execute after final S1 tree)
Authority: PR #115 (GPT plan) gate 040-S4; vehicle HANDOFF/todo/D-04.
Scope honesty: the local lab proves fresh CLI<->emulator delivery at
current HEAD with both endpoints on the cloud node (AWS) (mandate 0A.8). The
cross-internet Hawaii<->PA custody cell is 040-S5 (operator + Josh/Lucas,
infra-gated) and stays separate.

Cloud node facts: /ip4/100.56.248.69/tcp/9001; health http://100.56.248.69:9876/health
(port 9876, NOT 8080); image testbotz/scmessenger (container scm-alpha-relay).
Peer id per newer alpha doc: 12D3KooW<redacted>
(2026-07-20 proof recorded 12D3KooW<redacted>;
dialing is promiscuous -- record both, trust the live cloud node's identify).

Code-verified corrections at HEAD (override older docs):
- CLI binary is scmessenger-cli (cli/Cargo.toml), not scm.
- --http-bind is a GLOBAL flag (cli/src/main.rs:154-158).
- CLI initial-node seeding is config/env (bootstrap_nodes in config.json or
  SC_BOOTSTRAP_NODES env; cli/src/bootstrap.rs:36-46), NOT the ledger.
- Android ledger = Rust LedgerManager; fresh install EMPTY; no hardcoded
  cloud-node address remains (comment-only MeshRepository.kt:78).
- No REST route returns per-message delivery status. Sender-observed
  DELIVERED = CLI stdout "Delivered: <8-char-id>" (cli/src/main.rs:2081)
  and/or Android logcat "[RECEIPT-RX] Received from core: ... status=delivered"
  (MeshRepository.kt:2152).
- Queued-vs-connected FIXED at HEAD: SwarmBridge.dial resolves Ok only on
  ConnectionEstablished, Err on OutgoingConnectionError or 10s timeout
  (swarm.rs:5299-5324, 4557-4560, 4853-4856, 2836-2851). The 2026-07-25
  false-success caveat is STALE for the dial path.
- ConnectionEstablished log (both endpoints): "Connected to <peer> via
  <addr> (promiscuous mode — any PeerID accepted)" at INFO (swarm.rs:4535).
  Android logcat TAG for core tracing is UNKNOWN -- filter by TEXT.
- "Outgoing connection error ..." logs at DEBUG despite [WARNING] text
  (swarm.rs:4816) -- raise RUST_LOG to see dial failures.

## 0. Pre-flight (orchestrator; all green before starting)

0.1 Concurrency: `tasklist | findstr /I "cargo rustc gradle java ndk"` must
be empty. One build tool at a time (Windows rlib lock).
0.2 Disk: `bash scripts/preflight_disk.sh` -- need >=25 GB free. Next Rust
compile is COLD -> -j2 first, -j6 warm. Never `cargo clean --target X`
(it wipes all of target/).
0.3 Provenance baseline: `git rev-parse HEAD` = <HEAD_SHA>; every endpoint
must later show it in its "Core Provenance:" startup line.
0.4 Cloud node reachability (operator egress): PRECONDITION -- the cloud node must be reachable by PUBLIC route (public IP or DDNS + port forwards per H-04). 100.56.248.69 is a CGNAT-range tailnet address (ops-side convenience, NOT a product path -- repo philosophy: no third-party network dependencies); the 2026-07-28 probe found it unreachable from the Windows host. Operator to supply the public endpoint before this step. Verify:
  curl -fsS http://100.56.248.69:9876/health  -> {"status":"healthy"}
  powershell -NoProfile -Command "Test-NetConnection 100.56.248.69 -Port 9001 -InformationLevel Quiet"  -> True
If down: retry x2 @60s; then LAN-DIRECT fallback (Appendix A) or ABORT to
operator (infra, not code). Never fake cloud-node evidence.
0.5 Emulator (agy executes; orchestrator gates):
  emulator -avd scm_pixel_35 -no-window -no-audio -no-boot-anim -gpu swiftshader_indirect
  adb -s emulator-5554 wait-for-device
  adb -s emulator-5554 shell getprop sys.boot_completed  -> 1
If no boot in ~10 min: one cold retry (-no-snapshot-load), then escalate.
Do NOT chase the AWS Josh crash-loop class (corrupt AWS image, different AVD).
0.6 APK = LOCAL GRADLE BUILD. No CI artifact exists for an untagged HEAD;
a stale CI APK = the P1-04 artifact-skew class. Local build embeds the
same SCM_GIT_HASH provenance so step 3's stamp check is meaningful.
Emulator is x86_64 -> APK must contain lib/x86_64/libscmessenger_core.so.

## 1. Builds (orchestrator; strictly serial)

1.1 CLI (host): `set CARGO_INCREMENTAL=0` then
  cargo build -p scmessenger-cli -j2   (cold; -j6 warm)
Artifact: target/debug/scmessenger-cli.exe
1.2 APK (spawns cargo-ndk -- IS the next build; nothing concurrent):
  cd android && ./gradlew assembleDebug -x lint --quiet
Artifact: android/app/build/outputs/apk/debug/app-debug.apk -- DEBUG build
(release ReleaseTree drops INFO logs: MeshApplication.kt:43-45,139-146).
1.3 Verify x86_64 lib in APK:
  powershell -NoProfile -Command "Add-Type -A System.IO.Compression.FileSystem; [IO.Compression.ZipFile]::OpenRead('android/app/build/outputs/apk/debug/app-debug.apk').Entries.FullName | Select-String 'lib/x86_64/libscmessenger_core.so'"
Expect one match. Absent -> ABI misconfig; fix before continuing.

## 2. Ledger seeding on the fresh emulator (the crux)

Mechanism inventory at HEAD (use only the live one):
- Settings cloud-node entry -- DEAD write path (toggle+budget only; cloud-node list
  read-only/ledger-sourced, MeshRepository.kt:5495-5501).
- Deep link scmessenger://add|invite -- LIVE but cannot carry a cloud-node
  multiaddr (prefills a contact only; MainViewModel.kt:285-299).
- LAN/mDNS discovery of CLI -- LIVE but unreliable through emulator NAT;
  FALLBACK only.
- Signed invite import -- DEAD on Android (zero kt references; confirms
  the F2 review finding).
- Cloud-node-discovery HTTP endpoint -- ABSENT (only HttpURLConnection is a
  google.com connectivity probe, NetworkDiagnostics.kt:62-63).

PRIMARY SEED = unsigned contact-JSON import with a `listeners` array: the
only live path that writes the address to the ledger seed tier
(annotateIdentityInLedger, MeshRepository.kt:3960-3965) AND dials it
(connectToPeer). Parsers: MainViewModel.importContact (:218-270),
ContactsViewModel.importContact (:733-761). Constraint: public_key must be
exactly 64 hex chars or import is rejected (ContactsViewModel.kt:495-504).

2.1 Start CLI node (also used for the proof):
  set CARGO_INCREMENTAL=0
  set SC_BOOTSTRAP_NODES=/ip4/100.56.248.69/tcp/9001
  set RUST_LOG=info,scmessenger_core=debug
  target\debug\scmessenger-cli.exe --http-bind 127.0.0.1:9876 start --port 9100 > tmp\work_files\2026-07-28_wave2\cli.log 2>&1
Then:
  curl -s http://127.0.0.1:9876/api/identity -o tmp\work_files\2026-07-28_wave2\cli_identity.json
Record <CLI_PK_HEX> (public_key_hex) and <CLI_PEER_ID> (libp2p_peer_id).
2.2 Assert CLI on the cloud node (first ConnectionEstablished, CLI side):
  findstr /C:"Connected to" /C:"Core Provenance" tmp\work_files\2026-07-28_wave2\cli.log
Expect: "Core Provenance: 0.x.y (<HEAD_SHA>)" (hash must equal <HEAD_SHA>)
and "Connected to 12D3Koo... via /ip4/100.56.248.69/tcp/9001 (promiscuous
mode — any PeerID accepted)".
2.3 agy installs + launches + seeds (orchestrator supplies exact JSON):
Seed payload (double duty: registers cli-node contact AND seeds+dials the cloud node):
  {"public_key":"<CLI_PK_HEX>","peer_id":"<CLI_PEER_ID>","nickname":"cli-node","listeners":["/ip4/100.56.248.69/tcp/9001"]}
agy invocation (model name suffix IS the effort -- never add --effort):
  agy.exe -p --model "gemini-3.6-flash-high" --add-dir C:/Users/SCM/Documents/GitHub/SCMessenger --dangerously-skip-permissions --print-timeout 1800s "<scoped task: install -r -d the APK; pm grant POST_NOTIFICATIONS; launch via monkey; import the JSON exactly via Add Contact/Import; confirm cli-node appears; capture adb logcat -d to tmp/work_files/2026-07-28_wave2/android_logcat_seed.txt; report lines matching: Dialed seed relay from ledger / Connected to / Dial timed out / No known relay in ledger; read the app identity screen and report this device's 64-hex public key and 12D3Koo peer id verbatim; do not build or edit>"
agy reports <ANDROID_PK_HEX> and <ANDROID_PEER_ID>.
2.4 Verify seed took (cloud node ConnectionEstablished, Android side):
Expect "Dialed seed relay from ledger: /ip4/100.56.248.69/tcp/9001" (now
means CONNECTED, not queued) + core "Connected to ... via /ip4/100.56.248.69/tcp/9001".
"Dial timed out after 10s" or "No known relay in ledger yet" -> triage 6A.

## 3. Connection phase -- both sides on the cloud node + mutual discovery

3.1 Signatures: CLI second "Connected to <ANDROID_PEER_ID> via ..." once
discovered through the cloud node; Android "Connected to <CLI_PEER_ID> via ...";
CLI outbox flush "Flushing N queued message(s) to peer <ANDROID_PEER_ID>"
(cli/src/main.rs:2475-2479).
3.2 Socket evidence (host):
  powershell -NoProfile -Command "Get-NetTCPConnection -RemoteAddress 100.56.248.69 -RemotePort 9001 -State Established | Format-Table -AutoSize" > tmp\work_files\2026-07-28_wave2\netstat.txt
Expect >=1 Established row. Emulator leg is NATed through host, so host
established socket + Android logcat Connected line together evidence both.
Cloud-node-side ss -tn (both endpoints) is OPERATOR-ASSISTED corroboration
(SSH to 100.56.248.69); request it, never block on it, never fabricate.
3.3 Mutual peer knowledge:
  curl -s http://127.0.0.1:9876/api/peers -o tmp\work_files\2026-07-28_wave2\peers_cli.json
Expect <ANDROID_PEER_ID> listed. Absent -> triage 6B.

## 4. Delivery phase -- both directions + receipt round trip

4.0 Register Android as CLI contact (/api/send matches contact by peer_id
OR nickname; peer_id/public_key hold the Ed25519 PUBKEY HEX, cli/src/api.rs:567,570-592):
  curl -s -X POST http://127.0.0.1:9876/api/contacts -H "Content-Type: application/json" -d "{\"peer_id\":\"<ANDROID_PK_HEX>\",\"public_key\":\"<ANDROID_PK_HEX>\",\"name\":\"android\"}"
Expect {"success":true,"error":null}. Verify via GET /api/contacts.
4.1 Direction 1 CLI -> Android:
  curl -s -X POST http://127.0.0.1:9876/api/send -H "Content-Type: application/json" -d "{\"recipient\":\"android\",\"message\":\"WAVE2-D1-cli-to-android-<date>\"}"
success:true proves ACCEPTANCE only. Recipient evidence (agy): logcat
"Message from <senderId>: <messageId>" (MeshRepository.kt:1748). Receipt
proof (orchestrator): findstr /C:"Delivered:" cli.log -> "[OK][OK]
Delivered: <8-char-id>" (cli/src/main.rs:2081; appears only when CLI
decodes Android's DELIVERED receipt). Absent within 60s -> triage 6C.
4.2 Direction 2 Android -> CLI (agy composes+sends
"WAVE2-D2-android-to-cli-<date>" in the cli-node chat, waits 45s, captures
logcat, screenshots the delivery state). Sender-observed DELIVERED:
"[RECEIPT-RX] Received from core: msg=<id> status=delivered"
(MeshRepository.kt:2152) + "[RECEIPT-RX] Emitted MessageEvent.Delivered:
msg=<id>" (:2325). Receiver accepts only delivered/read (:2158) -- any
other status = receipt regression, not a pass. Recipient evidence:
  curl -s -X POST http://127.0.0.1:9876/api/history -H "Content-Type: application/json" -d "{\"limit\":50}" -o history_cli.json
Expect direction":"received" entry containing the marker text.
4.3 Artifacts (tmp/work_files/2026-07-28_wave2/): cli.log, logcat dumps,
netstat.txt, cli_identity.json, android_identity.txt, contacts_cli.json,
peers_cli.json, history_cli.json, health.txt, provenance.txt (HEAD SHA +
both Core Provenance lines), agy screenshots. Orchestrator writes a
TRACKED verdict at HANDOFF/review/V040_S4_DELIVERY_VERDICT.md indexing
artifacts against pass criteria.

## 5. Pass/fail criteria (all required)

1. Provenance match: <HEAD_SHA> == CLI Core Provenance hash == Android
   core provenance. Mismatch = artifact skew = rebuild both, not a pass.
2. ConnectionEstablished BOTH sides: "Connected to ... via
   /ip4/100.56.248.69/tcp/9001 (promiscuous mode ...)" in cli.log AND
   android_logcat_seed.txt (stronger: second Connected to the counterparty
   peer on each side).
3. Cloud-node socket: Established TCP row to 100.56.248.69:9001 in netstat.txt.
4. CLI->Android: "Message from ..." in Android logcat AND "Delivered: <id>"
   in cli.log for the same send.
5. Android->CLI: direction":"received" marker entry in history_cli.json AND
   "[RECEIPT-RX] ... status=delivered" + "Emitted MessageEvent.Delivered"
   in android_logcat_send.txt.
DISQUALIFIED as sole evidence: "Dialed ... via SwarmBridge", "Dialed seed
relay from ledger" without a following core Connected line, /api/send
success:true alone, any dial-queue log. success:true with no receipt = FAIL.

## 6. Failure triage (ordered)

A (no connection): A1 cloud node health+9001 (down -> infra, retry x2, fallback
or ABORT). A2 CLI not on the cloud node: grep "Outgoing connection error" (needs
RUST_LOG debug), confirm SC_BOOTSTRAP_NODES in process env, check firewall
outbound 9001. A3 Android ledger empty: "No known relay in ledger yet" ->
re-run seed, confirm 64-hex public_key (else silently rejected at
ContactsViewModel.kt:495-504), confirm "Dialed seed relay from ledger"
after re-import. A4 dialed but no Connected: "Dial timed out after 10s"
(sweep, swarm.rs:2836-2851); confirm emulator egress (ping 8.8.8.8).
B (cloud-node-connected, no mutual delivery): B1 outbox grep "Flushing" /
"deferring outbox flush" (main.rs:1878-1884 = peer not transport-reachable
yet). B2 peers_cli.json must contain <ANDROID_PEER_ID>; contacts_cli.json
must hold the Android contact; read /api/send error field. B3 agy confirms
cli-node contact + cloud node visible in getBootstrapNodesForSettings (proves
seed promoted to success_count>0). B4 cloud-node ss corroboration. B5 transport
selection: SmartTransportRouter must choose INTERNET/swarm, not BLE.
C (message arrives, no sender DELIVERED): C1 did Android send the ACK
(encodeReceipt MeshRepository.kt:2448-2460)? CLI inline decode debug
"Delivery ACK received from" (main.rs:2084). C2 CLI sends ACK at
main.rs:2066 ("Sending delivery ACK for <id>"); no ACK = CLI regression;
ACK sent but no [RECEIPT-RX] = Android receipt regression. C3 status
other than delivered/read = rejected at :2158 = real defect. C4
provenance skew -> rebuild both from <HEAD_SHA> (most common historical
cause, P1-04 class). C5 receipts genuinely failing at this HEAD = NEW
regression: ticket + full logs, never a relaxed criterion.

## 7. Role split

Orchestrator OWNS: pre-flight, build queue (serial), all builds, all
host-side curl/netstat, artifacts + TRACKED VERDICT.md, pass/fail decision.
agy OWNS (never builds/edits/decides): emulator boot, adb install,
permission grant, app launch, UI seed import, reading Android identity,
Android->CLI compose+send, logcat captures, screenshots. Canonical agy
shape: agy.exe -p --model "gemini-3.6-flash-high" --add-dir <repo>
--dangerously-skip-permissions --print-timeout 1800s "<scoped task>" --
model suffix IS the effort; never pass --effort with a suffixed model.

## 8. Time budget / retry policy

~1.5-2.5h wall (builds dominate): pre-flight+boot ~10m; CLI cold ~15-25m;
APK ~20-40m (serial); seed+connect ~10m; delivery+receipts ~15-20m;
triage buffer ~30-45m. Per phase max 2 retries with triage between; 3rd
failure escalates (no loops). Receipt failure at correct provenance:
freeze logs, ticket, report FAIL -- never retry into a pass.

## Appendix A -- LAN-DIRECT fallback (cloud node down; labelled LAN evidence)

CLI without SC_BOOTSTRAP_NODES; Android seed listeners = CLI's
host-reachable address from the emulator (host = 10.0.2.2; CLI binds P2P
on ws_port+1, main.rs:1440 -- verify actual "Listening on ..." line,
swarm.rs:4502-4503). Same criteria except criterion 3 becomes Established
row to the CLI LAN address; relay evidence recorded N/A-with-reason.

## Residual UNKNOWNs (resolve empirically during the run, never invent)

1. Whether import's connectToPeer appends /p2p/<CLI_PEER_ID> to the relay
   listener and still establishes (expected YES, promiscuous dial).
2. Exact Android identity screen navigation for <ANDROID_PK_HEX> +
   <ANDROID_PEER_ID> -- agy resolves from running UI, reports verbatim.
3. Whether CLI ledger gossips its relay entry via /sc/ledger-exchange/1.0.0
   (protocol exists addr_filter.rs:363; propagation unproven) -- relevant
   only if relying on auto-discovery instead of explicit import.
4. Cloud-node-side ss output is operator-assisted corroboration, never a gate.
