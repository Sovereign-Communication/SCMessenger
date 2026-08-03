# 040-S5 JOSH WAN PROOF RUNBOOK (workflow prep agent draft)

# 040-S5 RUNBOOK (DRAFT) — Josh WAN proof: Hawaii <-> Pennsylvania E2E messaging through the cloud node

Status: DRAFT for operator review. Read-only task; nothing written to the repo. All source anchors re-verified this session at HEAD `909edf4c` unless noted as carried from the S4 runbook (verified at `645c36ec`; two wave-1b commits since touched `core/src/transport/swarm.rs`, so its swarm.rs line numbers are stale — this draft cites the HEAD numbers).

Authority chain:
- Gate definition: `HANDOFF/review/V040_S4_DELIVERY_PROOF_RUNBOOK.md:5-8` — local lab (S4) proves fresh CLI<->emulator delivery at current HEAD; the cross-internet Hawaii<->PA custody cell is 040-S5 (operator + Josh/Lucas, infra-gated) and stays separate.
- Gate authority per `HANDOFF/review/V040_BASELINE_FREEZE.md:5-6` is the GPT planning verdict PR #115, `GPT_PLANNING_040_050_VERDICT.md`. Honesty note: that file is NOT committed to the repo (absent from `HANDOFF/gpt/` and everywhere else; grep-confirmed). The gate and tag checklist in Section 0 below are therefore reconstructed from repo-authoritative sources: `_QUEUE.md:30-34` (wave 2 = fresh E2E proof with ConnectionEstablished evidence, not dial-queue logs; wave 3 = gates/docs/operator tag), `V040_BASELINE_FREEZE.md:44-53`, `V1_0_0_EXECUTION_PLAN.md:48-50`, and `.github/workflows/release.yml`.

Evidence standards inherited from S4 (non-negotiable, `V040_S4_DELIVERY_PROOF_RUNBOOK.md:16-35,165-181`):
- The AWS box is the "cloud node", not "relay" (terminology scrub, commit `645c36ec`).
- PASS requires `ConnectionEstablished` text evidence on BOTH sides: `Connected to <peer> via <addr> (promiscuous mode — any PeerID accepted)` (HEAD: `core/src/transport/swarm.rs:4568`, INFO level).
- DISQUALIFIED as sole evidence: any dial-queue log, `Dialed seed relay from ledger` without a following core `Connected to` line, `/api/send success:true` alone (acceptance, not delivery).
- Receipt failure at correct provenance = FAIL, never a relaxed criterion; never retry into a pass (`V040_S4_DELIVERY_PROOF_RUNBOOK.md:221-227`).

---

## 0. Gate 040-S5 definition + v0.4.0-alpha.1 tag checklist

040-S5 PASSES when ALL hold, evidenced by artifacts (not claims):
1. Provenance match: tag commit SHA == operator CLI `Core Provenance:` hash == Josh APK `Settings` Core hash.
2. ConnectionEstablished to the cloud node on BOTH endpoints (operator CLI log + Josh Android logcat), from TWO independent real networks (PA home fiber egress; HI cellular and HI home WiFi — both legs, per `HANDOFF/ALPHA_TEST_LUCAS_JOSH_SETUP.md:158-159`).
3. Delivered message both directions, each with the sender-side receipt (`Delivered: <id>` CLI / `[RECEIPT-RX] ... status=delivered` Android).
4. Restart-persistence arm (Section 7) and disconnect/reconnect queued-delivery arm (Section 8) pass.
5. Lucas port-forward verification recorded OR the AWS-only waiver (Section 9) signed by the operator.

Tag checklist (operator-executed, terminal):
- [ ] Terminal verdict disposes findings F2/F3/F6/F7/F10/F12/F13/F16/NEW-6 (`V040_BASELINE_FREEZE.md:52-53` -> `HANDOFF/review/V040_FINDING_DISPOSITIONS.md`).
- [ ] Version bump 0.3.5 -> 0.4.0 (currently `0.3.5`, `Cargo.toml:9`) lands ONLY in the terminal release PR with the `auto-tag-release.yml` auto-trigger removed; operator creates `v0.4.0-alpha.1` manually (`V040_BASELINE_FREEZE.md:46-51`).
- [ ] `git tag v0.4.0-alpha.1 && git push origin v0.4.0-alpha.1` — tag push triggers `release.yml` (`_QUEUE.md:71-74`, `V1_0_0_EXECUTION_PLAN.md:48-50`).
- [ ] Release page live with `scm-windows-amd64.exe` + `.sha256`, debug APK, `SHA256SUMS.txt`; prerelease flag true (alpha in tag, `release.yml:323-337`). No signed release APK exists until the four keystore secrets are set (`release.yml:112-121`) — Josh sideloads the DEBUG APK; that is acceptable (`HANDOFF/plans/V040_ORCHESTRATION_PLAN.md:254-258`).
- [ ] 040-S5 proof run executes against the tag artifacts; tracked verdict written at `HANDOFF/review/V040_S5_WAN_PROOF_VERDICT.md` (new file, operator-tracked) indexing Section 10's manifest against artifacts.

---

## 1. Participants and endpoints

| Role | Who | Endpoint | Network | Notes |
|---|---|---|---|---|
| Operator node | Operator (Lucas, PA) | Windows CLI `scmessenger-cli.exe` (release asset `scm-windows-amd64.exe`) | PA home fiber, NATed, outbound only | Binary name is `scmessenger-cli`, not `scm` (`cli/Cargo.toml`, per S4 runbook:17). No inbound needed — dials outbound to cloud node. |
| Josh node | Josh (HI) | Physical Android phone, DEBUG APK from the tag | HI cellular (his usual) + HI home WiFi, both tested | AWS-hosted "Josh emulator" path is ABANDONED — corrupt system image, two crash-loop classes, operator decision 2026-07-20 (`HANDOFF/SESSION_HANDOFF_2026-07-20_LUCAS_JOSH_ALPHA.md:209-258`; memory `project_josh_emulator_abandoned`). Never retry that AVD. |
| Cloud node | infra (AWS t3.micro) | `/ip4/100.56.248.69/tcp/9001`; health `http://100.56.248.69:9876/health` (port 9876, NOT 8080/9000) | public internet | Image `testbotz/scmessenger`, container `scm-alpha-relay`, `--network host --restart unless-stopped` (`SESSION_HANDOFF_2026-07-20...:11-18`). SG `sg-0f195044b0dc7a800` opens 22/9001tcp/9001udp/9000/9876. Do NOT pull the prebuilt-image rule (memory `project_alpha_relay_prebuilt_image`). |

Cloud node peer id: dials are promiscuous — any PeerID accepted. Newer alpha doc records `12D3KooW<redacted>`; the 2026-07-20 proof recorded `12D3KooW<redacted>`. Record both, trust the live node's identify output (`V040_S4_DELIVERY_PROOF_RUNBOOK.md:12-14`).

Topology: both endpoints dial OUTBOUND to the cloud node; the cloud node bridges them. Neither participant needs inbound reachability — this is why the AWS-only test needs no port forwards (Section 9).

---

## 2. Pre-flight (operator, all green before starting)

2.1 Tag SHA baseline (record as `<TAG_SHA>`):
```
git rev-parse v0.4.0-alpha.1
```
2.2 Cloud node health + port (operator egress; retry x2 @60s on fail, then ABORT — never fake evidence, per S4:46-50):
```
curl -fsS http://100.56.248.69:9876/health
powershell -NoProfile -Command "Test-NetConnection 100.56.248.69 -Port 9001 -InformationLevel Quiet"
```
Expect `{"status":"healthy"}` and `True`.
2.3 Release asset integrity: download `scm-windows-amd64.exe`, `scm-windows-amd64.exe.sha256`, the APK, and `SHA256SUMS.txt` from the release page; verify:
```
powershell -NoProfile -Command "(Get-FileHash scm-windows-amd64.exe -Algorithm SHA256).Hash"
certutil -hashfile app-debug.apk SHA256
```
Both must match `SHA256SUMS.txt` lines.
2.4 OPERATOR -> Josh: transmit APK + its SHA256 line over a side channel; Josh verifies before install (physical step).
2.5 Josh device readiness (OPERATOR -> Josh, physical): Android >= 8 (minSdk 26); enable Developer Options + USB debugging only if Josh can run adb for logcat capture; otherwise plan on UI screenshots as Josh-side evidence (Section 6 notes which evidence is screenshot-acceptable).

---

## 3. Artifact install

3.1 Operator CLI (Windows):
```
mkdir %USERPROFILE%\bin 2>NUL & copy scm-windows-amd64.exe %USERPROFILE%\bin\scmessenger-cli.exe
```
(`$HOME\bin` on PATH or use full path. `--http-bind` is a GLOBAL flag — it precedes the subcommand, `cli/src/main.rs:154-158` per S4:18.)
3.2 OPERATOR -> Josh (physical): sideload the APK (Settings -> install unknown app -> file manager -> APK), grant notifications when prompted.
3.3 Provenance stamp check (BOTH sides, gate criterion 1):
- Operator: start per 4.1; expect in `cli.log`: `Core Provenance: 0.4.0 (<TAG_SHA>)` (`cli/src/main.rs:649-652`; hash embedded at build time, `cli/build.rs:15`).
- Josh (OPERATOR -> Josh, physical): app -> Settings -> version row shows `<VERSION_NAME> (Core: <TAG_SHA>)` (`SettingsScreen.kt:883`, backed by `BuildConfig.SCM_GIT_HASH`, `android/app/build.gradle:93`). Josh photographs/sends the row.
Mismatch on either side = artifact skew = FAIL, not a pass (P1-04 class; S4:167-168).

---

## 4. Identity exchange + ledger seeding on Josh's device (the crux)

4.1 Start the operator CLI node (also the proof node):
```
set CARGO_INCREMENTAL=0
set SC_BOOTSTRAP_NODES=/ip4/100.56.248.69/tcp/9001
set RUST_LOG=info,scmessenger_core=debug
scmessenger-cli.exe --http-bind 127.0.0.1:9876 start --port 9100 > tmp\work_files\040-s5\cli.log 2>&1
```
Initial-node seeding is config/env (`SC_BOOTSTRAP_NODES` / `bootstrap_nodes`, `cli/src/bootstrap.rs:36-46` per S4:19-20), NOT the ledger and NOT `scm config bootstrap add` (older doc; superseded).
4.2 Capture operator identity (send these two values to Josh out-of-band):
```
curl -s http://127.0.0.1:9876/api/identity -o tmp\work_files\040-s5\cli_identity.json
```
Fields: `public_key_hex` (64 hex) and `libp2p_peer_id` (`cli/src/api.rs:1025,1030`). Record `<CLI_PK_HEX>`, `<CLI_PEER_ID>`.
4.3 Operator asserts own ConnectionEstablished to cloud node (first half of gate criterion 2):
```
findstr /C:"Core Provenance" /C:"Connected to" tmp\work_files\040-s5\cli.log
```
Expect `Core Provenance: 0.4.0 (<TAG_SHA>)` and `Connected to 12D3Koo... via /ip4/100.56.248.69/tcp/9001 (promiscuous mode — any PeerID accepted)`.
4.4 Seed payload (OPERATOR -> Josh, exact JSON — this single import does double duty: registers the operator contact AND writes the cloud node to the ledger seed tier AND dials it):
```json
{"public_key":"<CLI_PK_HEX>","peer_id":"<CLI_PEER_ID>","nickname":"lucas-cli","listeners":["/ip4/100.56.248.69/tcp/9001"]}
```
This is the ONLY live seed path at HEAD (mechanism inventory, S4:77-94): the Settings cloud-node entry is a DEAD write path; deep links cannot carry a cloud-node multiaddr; signed-invite import is dead on Android; there is no discovery HTTP endpoint. The import writes via `annotateIdentityInLedger` and dials via `connectToPeer` (`MainViewModel.kt:218` — `listeners` parsed :236-243, `connectToPeer` :261-262; second parser `ContactsViewModel.kt:733`).
4.5 OPERATOR -> Josh (physical): in the app, Add Contact / Import; paste the JSON verbatim; confirm `lucas-cli` appears in contacts.
4.6 Constraint that silently kills the seed if violated: `public_key` must be EXACTLY 64 hex characters or import is rejected — `ContactsViewModel.kt:495-496` (`Public key must be exactly 64 hex characters (got N)`), filter repeated at :537. Verify `<CLI_PK_HEX>` length before sending:
```
powershell -NoProfile -Command "(Get-Content tmp\work_files\040-s5\cli_identity.json | ConvertFrom-Json).public_key_hex.Length"
```
Expect 64.
4.7 Josh-side seed evidence (Josh captures `adb logcat -d` if able — filter by TEXT, the core TAG is unknown, S4:33; else screenshots + operator infers from CLI-side discovery in Section 5):
```
adb logcat -d | findstr /C:"Dialed seed relay from ledger" /C:"Connected to" /C:"Dial timed out" /C:"No known relay in ledger"
```
Expect `Dialed seed relay from ledger: /ip4/100.56.248.69/tcp/9001` (`MeshRepository.kt:4953`) FOLLOWED BY a core `Connected to ... via /ip4/100.56.248.69/tcp/9001` line. At HEAD `Dialed seed relay...` means CONNECTED, not queued (SwarmBridge.dial resolves Ok only on ConnectionEstablished, Err on error/10s timeout — S4:28-30). `No known relay in ledger yet` (`MeshRepository.kt:4948`) = import did not land -> Section 11 tree B.
4.8 Josh reports verbatim (OPERATOR -> Josh, physical; Settings/identity screen): his device's 64-hex public key `<JOSH_PK_HEX>` and `12D3Koo...` peer id `<JOSH_PEER_ID>`.
4.9 Operator registers Josh as a CLI contact (`/api/send` matches contact by peer_id OR nickname; both `peer_id` and `public_key` fields hold the Ed25519 PUBKEY HEX, NOT the 12D3Koo id — `cli/src/api.rs:560-595`, comment at :572-574):
```
curl -s -X POST http://127.0.0.1:9876/api/contacts -H "Content-Type: application/json" -d "{\"peer_id\":\"<JOSH_PK_HEX>\",\"public_key\":\"<JOSH_PK_HEX>\",\"name\":\"josh\"}"
curl -s http://127.0.0.1:9876/api/contacts -o tmp\work_files\040-s5\contacts_cli.json
```
Expect success and `josh` listed (GET route exists since commit adding `handle_get_contacts`, per `SESSION_HANDOFF_2026-07-20...:290-294`).

---

## 5. Connection phase — both sides on the cloud node + mutual discovery

5.1 Signatures: CLI logs a second `Connected to <JOSH_PEER_ID> via ...` once discovered through the cloud node; Josh's logcat shows `Connected to <CLI_PEER_ID> via ...` (`swarm.rs:4568`).
5.2 Socket evidence (operator host):
```
powershell -NoProfile -Command "Get-NetTCPConnection -RemoteAddress 100.56.248.69 -RemotePort 9001 -State Established | Format-Table -AutoSize" > tmp\work_files\040-s5\netstat_operator.txt
```
Expect >=1 Established row (the CLI leg; Josh's leg is on his phone, evidenced by his logcat Connected line). Cloud-node-side `ss -tn state established :9001` showing BOTH public IPs (PA fiber IP + HI IP) is OPERATOR-ASSISTED corroboration (SSH to 100.56.248.69) — request it, never block on it, never fabricate (S4:126-131; the 2026-07-20 Lucas proof was exactly this `ss` evidence, `SESSION_HANDOFF_2026-07-20...:19-23`).
5.3 Mutual peer knowledge:
```
curl -s http://127.0.0.1:9876/api/peers -o tmp\work_files\040-s5\peers_cli.json
```
Expect `<JOSH_PEER_ID>` listed (routes verified at HEAD `cli/src/api.rs:1208-1217`). Absent -> Section 11 tree C.

---

## 6. Delivery phase — both directions + receipt round trip

6.1 Direction 1, CLI -> Android (Josh on CELLULAR first — his usual connection, `ALPHA_TEST_LUCAS_JOSH_SETUP.md:165-169`):
```
curl -s -X POST http://127.0.0.1:9876/api/send -H "Content-Type: application/json" -d "{\"recipient\":\"josh\",\"message\":\"S5-D1-cli-to-josh-cell-<date>\"}"
```
`success:true` proves ACCEPTANCE ONLY. Josh-side evidence (OPERATOR -> Josh): the message text visible in the `lucas-cli` chat (screenshot acceptable) and/or logcat `Message from <senderId>: <messageId>` (`MeshRepository.kt:1748`). Operator-side receipt proof:
```
findstr /C:"Delivered:" tmp\work_files\040-s5\cli.log
```
Expect `[OK][OK] Delivered: <8-char-id>` (`cli/src/main.rs:2081` — emitted ONLY when the CLI decodes Josh's DELIVERED receipt). Absent within 60s -> Section 11 tree D.
6.2 Direction 2, Android -> CLI (OPERATOR -> Josh, physical): in the `lucas-cli` chat compose and send `S5-D2-josh-to-cli-cell-<date>`; wait 45s; capture logcat + screenshot the delivery state. Josh-side sender receipt:
```
adb logcat -d | findstr /C:"RECEIPT-RX"
```
Expect `[RECEIPT-RX] Received from core: msg=<id> status=delivered` (`MeshRepository.kt:2152`). Status normalization is strict: unknown statuses are logged `[RECEIPT-RX] IGNORING: Unknown delivery status` (:2160) — any status other than delivered/read is a receipt regression, NOT a pass (S4:154-155).
6.3 Operator-side recipient evidence:
```
curl -s -X POST http://127.0.0.1:9876/api/history -H "Content-Type: application/json" -d "{\"limit\":50}" -o tmp\work_files\040-s5\history_cli.json
```
Expect a `direction":"received"` entry containing `S5-D2-josh-to-cli-cell-<date>`.
6.4 WiFi leg: OPERATOR -> Josh: switch phone to home WiFi, wait for reconnect (Josh-side `Connected to` again), repeat 6.1-6.3 with markers `S5-D1-...-wifi-<date>` / `S5-D2-...-wifi-<date>`. Carrier-filter failure on cellular but success on WiFi is itself a real alpha finding — document, don't treat as setup error (`ALPHA_TEST_LUCAS_JOSH_SETUP.md:190-194`) — but gate criterion 3 still requires at least one network leg to pass both directions with receipts.
6.5 Artifacts (tmp/work_files/040-s5/): cli.log, cli_identity.json, contacts_cli.json, peers_cli.json, history_cli.json, netstat_operator.txt, health.txt, provenance.txt (TAG_SHA + both provenance stamps), Josh logcat dumps + screenshots.

---

## 7. Restart-persistence arm (ledger seed survives, no re-import)

7.1 OPERATOR -> Josh (physical): force-stop the app (or reboot phone), relaunch. No re-import of the JSON.
7.2 Josh-side expectation: startup path re-dials from the sled-backed ledger WITHOUT a fresh import — `ensureBootstrapRelayConnected` reads `getPreferredRelays(1u)` and dials (`MeshRepository.kt:4941-4957`); logcat must again show `Dialed seed relay from ledger: /ip4/100.56.248.69/tcp/9001` (:4953) followed by core `Connected to ... via /ip4/100.56.248.69/tcp/9001`. `No known relay in ledger yet -- skipping proactive NAT dial` (:4948) = persistence FAIL (seed tier did not survive -> Section 11 tree B).
7.3 Proof of end-to-end persistence: operator sends `S5-PERSIST-cli-to-josh-<date>` (command per 6.1); expect Josh UI/logcat receipt AND CLI `Delivered: <id>`.
7.4 CLI side needs no arm: its cloud-node seed is env/config, not ledger; a CLI restart re-dials via `SC_BOOTSTRAP_NODES` (4.1).

---

## 8. Disconnect/reconnect queued-delivery arm (custody)

8.1 OPERATOR -> Josh (physical): force-stop the app (airplane mode also acceptable). Operator confirms Josh gone:
```
curl -s http://127.0.0.1:9876/api/peers -o tmp\work_files\040-s5\peers_cli_offline.json
```
Expect `<JOSH_PEER_ID>` absent (or the connection visibly dropped in cli.log).
8.2 Operator sends while Josh offline:
```
curl -s -X POST http://127.0.0.1:9876/api/send -H "Content-Type: application/json" -d "{\"recipient\":\"josh\",\"message\":\"S5-QUEUED-cli-to-josh-<date>\"}"
```
Expect `success:true` (queued to outbox — acceptance only; no `Delivered:` yet; a `Delivered:` here would mean Josh is still connected and the arm is invalid).
8.3 OPERATOR -> Josh (physical): relaunch app; capture logcat for the next 90s.
8.4 Operator-side flush + receipt evidence:
```
findstr /C:"Flushing" /C:"Delivered:" tmp\work_files\040-s5\cli.log
```
Expect `Flushing 1 queued message(s) to peer <JOSH_PEER_ID>` (`cli/src/main.rs:2476`) once Josh reconnects, THEN `Delivered: <8-char-id>` for the queued send. Josh side: `Message from` (MeshRepository.kt:1748) / UI screenshot showing `S5-QUEUED-cli-to-josh-<date>`. Outbox never drops (retry with backoff, `HANDOFF/V040_ORCHESTRATION_COMPLETE.md:96-107`); a silent non-delivery with no `Flushing` line = outbox-flush regression -> Section 11 tree D.

---

## 9. Lucas port-forward verification + AWS-only waiver

Context: the v1.0.0 plan's open decision 2 asks for one public endpoint that is NOT AWS — home-router port-forward to a Lucas CLI relay, or a WAN-live waiver (`V1_0_0_EXECUTION_PLAN.md:325`). The 040-S5 test as designed uses the AWS cloud node as the rendezvous and needs NO inbound path on either side, so port forwards are optional corroboration for this gate.

9.1 OPTIONAL ARM — if the operator stands up Lucas-home port forwards to the CLI node (OPERATOR, physical router config): verify from OUTSIDE the PA LAN (Josh's side, or the cloud node via SSH):
```
# From Josh/any external host — TCP ladder ports per multiport.rs fallback [443, 80, 8080, ...]
# (SESSION_HANDOFF_2026-07-25.md:88; V1_0_0_EXECUTION_PLAN.md:101) and QUIC UDP:
powershell -NoProfile -Command "Test-NetConnection <LUCAS_DDNS_HOST> -Port 443 -InformationLevel Quiet"   # TCP 443
powershell -NoProfile -Command "Test-NetConnection <LUCAS_DDNS_HOST> -Port 80  -InformationLevel Quiet"   # TCP 80
# UDP 443 (QUIC) needs a UDP probe, e.g. from the cloud node:
nc -vzu <LUCAS_DDNS_HOST> 443
# DDNS: the hostname must resolve to the current PA fiber public IP:
nslookup <LUCAS_DDNS_HOST>
```
Record all four results (TCP 443, TCP 80, UDP 443, DDNS resolution) in the manifest as PASS/FAIL/N-A with the responder's identity. Note: CLI default single-port listen is `/ip4/0.0.0.0/tcp/9001` and WS is `0.0.0.0:9002/ws` (`V1_0_0_EXECUTION_PLAN.md:101`) — the 443/80 ladder only answers if the CLI runs in multi-port listen mode; record the actual `Listening on ...` lines rather than assuming.
9.2 AWS-ONLY WAIVER TEXT (use verbatim in the verdict if the optional arm is not run):
> 040-S5 WAN-live waiver: this proof run establishes cross-internet end-to-end delivery between Hawaii and Pennsylvania through the cloud node at 100.56.248.69:9001, with ConnectionEstablished evidence from both endpoints on independent real networks (HI cellular + HI WiFi; PA fiber) and receipt-confirmed delivery both directions. A non-AWS public endpoint (Lucas home-router port-forward to a self-hosted CLI relay, TCP 443/TCP 80/UDP 443/DDNS) was NOT exercised; that arm belongs to P1-18 verification debt (V1_0_0_EXECUTION_PLAN.md:31-32,325) and remains open. The AWS cloud node is a test rendezvous, not a production relay dependency. Operator sign-off: ____ Date: ____

---

## 10. Pass/fail manifest template

| # | Criterion | Evidence artifact | Expected string/value | Verdict (PASS/FAIL/N-A) |
|---|---|---|---|---|
| 1 | Provenance match | provenance.txt | TAG_SHA == CLI `Core Provenance: 0.4.0 (<TAG_SHA>)` == Josh Settings `(Core: <TAG_SHA>)` | |
| 2a | ConnectionEstablished, operator side | cli.log | `Connected to 12D3Koo... via /ip4/100.56.248.69/tcp/9001 (promiscuous mode — any PeerID accepted)` | |
| 2b | ConnectionEstablished, Josh side (cellular) | josh_logcat_cell.txt | same signature, `via /ip4/100.56.248.69/tcp/9001` | |
| 2c | ConnectionEstablished, Josh side (WiFi) | josh_logcat_wifi.txt | same | |
| 3 | Cloud-node socket (corroboration) | netstat_operator.txt (+ optional ss output) | >=1 Established row to 100.56.248.69:9001 | |
| 4 | D1 CLI->Android delivered + receipted | cli.log + josh screenshot/logcat | `Message from ...` (or UI) AND `[OK][OK] Delivered: <id>` for `S5-D1-...` | |
| 5 | D2 Android->CLI delivered + receipted | history_cli.json + josh_logcat_send.txt | `direction":"received"` marker AND `[RECEIPT-RX] ... status=delivered` | |
| 6 | Restart-persistence arm | josh_logcat_restart.txt + cli.log | `Dialed seed relay from ledger` + `Connected to` without re-import; then `S5-PERSIST-...` delivered+receipted | |
| 7 | Queued-delivery arm | cli.log + josh evidence | `Flushing 1 queued message(s) to peer <JOSH_PEER_ID>` then `Delivered:` for `S5-QUEUED-...` | |
| 8 | Port-forward arm OR waiver | manifest row / verdict waiver block | four probe results, or signed AWS-only waiver text (9.2) | |
| DISQ | None of the sole evidence is: dial-queue logs, `Dialed seed relay` without core Connected, `success:true` alone | — | — | any criterion resting on these = FAIL |

Overall: PASS only if rows 1-7 PASS and row 8 is PASS-or-waived. Verdict written TRACKED at `HANDOFF/review/V040_S5_WAN_PROOF_VERDICT.md` (S4 pattern, `V040_S4_DELIVERY_PROOF_RUNBOOK.md:161-163`).

---

## 11. Failure escalation tree (max 2 retries per phase with triage between; 3rd failure escalates — no loops, S4:221-227)

Tree A — cloud node down (health/9001 fail at 2.2 or mid-run):
- A1 retry health x2 @60s. Still down: check container remotely if SSH available (`docker ps` on 100.56.248.69; restart policy is `unless-stopped`, `SESSION_HANDOFF_2026-07-20...:11-15`).
- A2 still down: ABORT to operator as INFRA failure. 040-S5 cannot proceed and cannot be faked — there is no LAN fallback for a Hawaii<->PA cell (Appendix A of S4 is local-lab only). Record health output, reschedule. Never fabricate cloud-node evidence (S4:50).

Tree B — seeding failed on Josh's device:
- B1 import visibly rejected: `Public key must be exactly 64 hex characters (got N)` (`ContactsViewModel.kt:495-496`) -> re-check `<CLI_PK_HEX>` (step 4.6), re-send JSON. Most common cause: truncation/copy damage in transit.
- B2 import accepted but `No known relay in ledger yet` (`MeshRepository.kt:4948`): the `listeners` array did not land -> confirm the JSON was pasted into the Import path that parses `listeners` (`MainViewModel.kt:236-243`), confirm `lucas-cli` contact exists, re-import.
- B3 `Dial timed out after 10s...` (`swarm.rs:2880`) with no Connected: Josh's egress problem — OPERATOR -> Josh: verify internet on that network leg; retry on the other leg (WiFi vs cellular). Dial failures log at DEBUG with `[WARNING]` text (`swarm.rs:4850,4852,6706`) — Josh-side logcat capture needs verbose core logging; the operator CLI side is already at `scmessenger_core=debug`.
- B4 third failure: escalate with full logcat — do not loosen the seed criterion.

Tree C — cloud-node-connected both sides, no mutual delivery:
- C1 operator side: `peers_cli.json` must contain `<JOSH_PEER_ID>` (else discovery through the cloud node failed — check ledger exchange in cli.log); `contacts_cli.json` must hold `josh`; read `/api/send`'s error field (`Contact not found` = 4.9 wrong or skipped).
- C2 outbox grep: `Flushing` / `deferring outbox flush` in cli.log (deferral = peer not transport-reachable yet).
- C3 transport selection: Josh's SmartTransportRouter must choose INTERNET/swarm, not BLE (S4 triage B5).
- C4 cloud-node `ss` corroboration requested from operator SSH (never a gate).

Tree D — message arrives but no sender DELIVERED receipt:
- D1 operator direction: check `Sending delivery ACK for` / `Delivery ACK received from` in cli.log (S4:201-204, main.rs:2066,2084). Josh direction: `[RECEIPT-RX]` trace on Josh logcat — ACK sent by CLI but no `[RECEIPT-RX]` = Android receipt regression; status other than delivered/read = rejected at the status gate (`MeshRepository.kt:2156-2160`) = real defect.
- D2 TERMINAL RULE: receipts genuinely failing at correct provenance = NEW regression. Freeze both-side logs, file a ticket, report FAIL — never retry into a pass, never relax the criterion (S4:205-208,225-227). Provenance skew discovered mid-triage -> rebuild BOTH from `<TAG_SHA>` and restart (most common historical cause, P1-04 class).

---

## 12. Role split + time budget

Operator OWNS: tag + release, asset checksums, CLI run, all host curl/netstat commands, cloud-node health and optional SSH corroboration, manifest + tracked verdict, pass/fail decision, waiver sign-off.
Josh OWNS (all physical, relayed as OPERATOR->Josh steps): APK install + checksum check, identity-screen readout, JSON import, message compose/send for D2, network switching (cellular/WiFi), force-stop/restart for arms 7-8, logcat capture or screenshots.
Budget: ~2-3h wall across two humans: pre-flight+artifacts ~15m; identity exchange+seed+connect ~20m; both directions x2 network legs ~40m; persistence + queued arms ~30m; triage buffer ~45m. Schedule with Josh in one sitting; the cloud node is stable but not immortal.

## Residual UNKNOWNs (resolve empirically during the run; never invent)

1. Josh's ability to run adb on his phone — if unavailable, Josh-side logcat criteria fall back to UI screenshots + operator-side receipts (weaker but still gateable; rows 2b/2c then rest on the second `Connected to <JOSH_PEER_ID>` line in cli.log + cloud-node `ss` showing the HI IP).
2. Whether the DEBUG APK's release-tree log suppression matters: CI builds via `assembleDebug` (`release.yml:99-110`), so INFO Timber lines survive — but confirm Josh's `Dialed seed relay` / `RECEIPT-RX` lines actually appear in his first capture before relying on them.
3. Exact cloud node peer id in service (two recorded; promiscuous dial accepts either — S4:12-14).
4. Whether carrier-grade NAT on Josh's cellular leg keeps the libp2p connection alive across idle — if the cellular leg drops silently, the WiFi leg still satisfies the gate, and the cellular behavior is recorded as an alpha finding, not a setup error (`ALPHA_TEST_LUCAS_JOSH_SETUP.md:190-194`).
