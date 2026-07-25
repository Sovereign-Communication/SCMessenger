# Remaining Tasks: Lucas/Josh Alpha Test vs. Farm-Sim (V1.0.0)

Compiled 2026-07-20 from HANDOFF docs (through `SESSION_HANDOFF_2026-07-20_LUCAS_JOSH_ALPHA.md`, the most recently updated file in `HANDOFF/`) and live repo state (`git log`, `git status` checked directly, not just doc claims).

## Bottom line

The dial-establishment bug that was silently reporting fake "connected" status is fixed, verified, and pushed to `origin/main`. But **no real two-person connection has ever been demonstrated.** Every success so far is Lucas's own two clients (Windows CLI + his local Android emulator) talking to the relay. Josh's side — the entire point of the test — has never actually connected, exchanged contacts, or received a message. That's the real gap before you send him a link.

Farm-Sim is a separate, larger V1.0.0 workstream (the 12-node/28-acre farm deployment validator) and is **not** on the critical path for sending Josh a link. It's included below because you asked for both.

Repo check performed just now: `git log` head is `2bbea431`, and local `main` matches `origin/main` exactly (0 ahead / 0 behind) — everything described as "done" below is actually pushed, not just committed locally. (Note: `git status` initially showed ~967 files as "modified" — verified this is CRLF/LF line-ending noise from viewing the Windows checkout through a Linux shell, not real uncommitted work; no untracked/added/deleted files exist.)

---

## Part 1: Lucas/Josh Alpha Test

Goal: Lucas (fiber) and Josh (cellular/WiFi, his phone) message each other reliably across the real internet through the AWS node (`100.56.248.69:9001`).

### Verified working now

- Alpha node live and healthy, running the dial-fix image, restart policy set.
- **Core dial bug fixed** (commit `f2831458`): `SwarmCommand::Dial` now waits for a real `ConnectionEstablished` event before reporting success. Previously it reported "connected" as soon as libp2p merely *queued* the dial — which is why prior sessions saw "Bootstrap connected" in the logs while the node showed zero real TCP connections.
- Lucas's Windows CLI → node: real connection, confirmed via `ss -tn state established` on the node itself. Ledger exchange completed (48 entries).
- Lucas's local Android emulator → relay: confirmed via app logs (`Connected(peerId=..., transport=INTERNET)`).
- Graceful-AF dial policy (self-dial prevention + RFC1918 private-range awareness) implemented in `cli/src/ledger.rs`, adversarial-reviewed (caught and fixed a real bug in the circuit-relay exemption before merge), committed.
- E-00 (ratchet/PQ subsystem wiring) — **done 2026-07-17**, separately from this session: real messages now get actual forward secrecy + PQ protection. Before this fix, every message sent by the app had zero forward secrecy regardless of how correct the underlying crypto code was. Kill switch: env `SCM_RATCHET_DISABLE`.
- All of the above is committed and pushed to `origin/main` (commits `f2831458`, `1950c374`, `2eaad174`, `efd164de`, `2bbea431`).

### Still open — what's actually blocking the Josh test

1. **Josh's device has never connected.** The AWS-emulator stand-in for Josh hit two separate, unfixed Android system-image crash loops (missing `libstatspull.so`, then missing `libnetd_updatable.so` — looks like a corrupted/incomplete system image, not a config problem). Abandoned 2026-07-20; the operator accepted Lucas's own solo verified connection as sufficient proof the *dial fix* works, but that is not the same as proving Josh's real phone can connect.
2. Confirm both Lucas and a second real endpoint appear **simultaneously** in the node's connection list.
3. Provision Lucas and Josh as contacts (public-key exchange via the `scmessenger://?public_key=...` deep link or in-app Add Contact) — not yet done with any real second party.
4. Send an actual message Lucas → Josh (or back) and confirm delivery + receipt. Never done, for anyone.
5. Filed but not fixed: after connecting, the CLI still promiscuously dials every address it learns from the ledger (own LAN IP, emulator-internal junk) with only partial filtering. Per-peer backoff/concurrent-dial cap and "prefer circuit-relay routing over direct dial" are designed but not implemented (`HANDOFF/todo/GRACEFUL_AF_DIAL_POLICY.md`, items 3–4).
6. Small fix not yet filed as a ticket: the swarm's adaptive port listener should exclude the control-API port (9876) from its own port range so the earlier port-collision class of bug can't recur.
7. Design question, not yet resolved: operator directive says "bootstrap deprecated in favor of nodes," but the code still reads `config.json bootstrap_nodes` as the seed. Needs a decision on whether/how to reframe this and whether the app should proactively probe for nodes.
8. A-04 (Android receipt-encoding unification) — dispatched today, the dispatch produced no output (empty log), ticket still open in `HANDOFF/IN_PROGRESS/`.
9. Practical note: the node address is currently hardcoded in `MeshRepository.kt` — fine for this specific test, but there's no general "add any node" flow yet if you want Josh to use a build that isn't pinned to your test node.
10. What you'd actually send Josh: a debug APK exists locally (`android/app/build/outputs/apk/debug/app-debug.apk`, built with the dial fix) or he builds from source per `HANDOFF/ALPHA_TEST_LUCAS_JOSH_SETUP.md`. There is no signed/distributable release build yet — this is a debug artifact, not a shareable download link in the normal sense.

---

## Part 2: Farm-Sim

Goal: validate the mesh at the scale of your actual 28-acre farm deployment (patchy/no cellular, farm WiFi, BLE-carried "sneakernet" messages, ~12 dispersed users, roughly half iPhone). Currently modeled as a 7-node Docker topology on AWS EC2 (alice/bob/carol/david/eve + 2 relays), planned to grow to 12 nodes / 3 network groups.

### Verified working

- 7-node topology deploys cleanly; bootstrap/ledger convergence PASSes well above target (42–127 peers per node vs. 6+ target).
- Basic resilience PASSes: 50ms latency injection, 5% packet loss, container crash+recovery all handled cleanly.
- Root cause of "zero messages ever deliver" found: every container starts with **zero contacts** (no provisioning mechanism existed), so `/api/send` always returned 404.
- Fix implemented and committed: `docker/bootstrap-topology.sh` now fetches real identities via `/api/identity` and provisions contacts via `/api/contacts`; a missing `GET /api/contacts` route was added (it was POST-only); the stress-test binary is now built into the Docker image; healthcheck blocks added to all client-node compose services.

### Still open

1. **The contact-provisioning fix has not been re-validated end-to-end.** No report yet confirms Phase 2 (progressive load, 10→100 msg/sec) or Phase 3 (failure injection) actually pass with real message delivery post-fix. This retest hasn't happened.
2. Multi-hour (6+ hr) stability run under sustained load — not started (checking for memory leaks / connection-pool exhaustion).
3. Cross-variant test — confirming real Android/iOS app builds (not just the CLI container) can join this same topology — not done.
4. A-09 (relay-discovery dial amplification / unauthenticated peer injection — HIGH-severity DoS): partially mitigated twice, but the core fixes are still open: installing `libp2p::connection_limits`, authenticating relay-discovery messages, deduping dial targets, and an explicit input-size guard in `RelayMessage::from_bytes`. Requires mandatory crypto-security-auditor sign-off before it can close.
5. C-05 (hostile-network / lossy-NAT test): not implemented. Real netem/`tc`-based infra already exists (`docker/docker-compose.network-test.yml`) but is orphaned — not wired into any test runner, no pass/fail assertion. Needs a design decision before it's safe to hand off.
6. Scaling from 7 nodes to the full 12-node, 3-group (farmhouse / far-field-cellular / dead-zone) topology — not started.
7. Physical two-device field trials (WiFi Aware, BLE "tractor route" sneakernet test) — human/hardware-only (Wave H, item H-02), not done. Scheduled for the farm pilot phase.
8. iOS parity for the farm's iPhone-carrying users is in-scope and farm-gating, but CI validation is still blocked on GitHub Actions billing (H-01).

---

## Cross-cutting: operator-only gates (Wave H)

These need you specifically, not an agent, and are documented in `HANDOFF/todo/WAVE_H_HUMAN_GATES.md` (dated 2026-07-17 — worth a quick sanity-check since some items like adaptive-port sign-off may already be superseded by work completed since):

- **H-01** GitHub Actions billing — blocks CI + the whole iOS lane.
- **H-02** Physical two-device field trials (WiFi Aware, BLE) — needs real hardware in hand.
- **H-04** AWS node resume decision — this is about the separate Farm-Sim cloud-mesh infra (`infra/aws/`), not the alpha-node you're already using with Josh.
- **H-05** Final v1.0.0 sign-off — the actual release-tag gate: needs all of the above closed, farm drills logged, PQC audits on file, and an Apple Developer account decision ($99/yr, needed before iOS TestFlight).

---

## Sources

- `HANDOFF/SESSION_HANDOFF_2026-07-20_LUCAS_JOSH_ALPHA.md` (today's session, most current)
- `HANDOFF/ALPHA_TEST_SESSION_FINDINGS_2026-07-19.md`
- `HANDOFF/ALPHA_TEST_LUCAS_JOSH_SETUP.md`
- `HANDOFF/todo/GRACEFUL_AF_DIAL_POLICY.md`, `CONTACT_PROVISIONING_FIX.md`, `VERIFY_LEDGER_EXCHANGE.md`, `A-09_RELAY_DISCOVERY_DIAL_AMPLIFICATION.md`, `C-05_P1_14_hostile_network_test_lo.md`, `WAVE_H_HUMAN_GATES.md`
- `HANDOFF/FARM_SIM_EXECUTION_STATUS_2026-07-18.md`, `HANDOFF/results/FARM_SIM_PHASE_2_3_TEST_REPORT.md`, `HANDOFF/todo/FARM_SIM_PHASE_2_3_FINDINGS.md`, `HANDOFF/todo/ORCHESTRATE_FARM_SIM_FIX_AND_RETEST.md`
- `HANDOFF/done/CRITICAL_RATCHET_SUBSYSTEM_NOT_WIRED_INTO_IRONCORE.md`
- `REMAINING_WORK_TRACKING.md`, `HANDOFF/todo/_QUEUE.md`, `docs/V1_KNOWN_LIMITATIONS.md`
- Live `git log` / `git status` on the repo (checked 2026-07-20)
