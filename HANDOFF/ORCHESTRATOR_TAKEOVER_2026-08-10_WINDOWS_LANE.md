# Orchestrator takeover -- 2026-08-10, Windows lane

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Active
Written: 2026-08-10 ~05:05Z
Successor: next `/orchestrate` session -- **READ THIS FIRST, then `HANDOFF/todo/_QUEUE.md`**

Everything below is file-backed. No memory of the prior session is required.

---

## 0. Sixty-second orientation

- **Anchor we are running: `68fcc3f1`.** Do NOT re-anchor without a named SHA
  whose CI has finished. The branch head moves every ~15 minutes.
- **Windows node**: running from `C:\Users\SCM\Documents\GitHub\scm-winlane`,
  launched with `SC_BOOTSTRAP_NODES` pointing at the AWS relay.
- **PR #139** is the coordination channel with the GPT-MAC lane. It is the
  PRIMARY channel. SCM CLI messaging works but is intermittent.
- **Five-node run is HELD.** Blockers in Section 4.
- **GPT-MAC pushed `4083e59b`** implementing fixes for three of our findings.
  Reviewing it is the top open task.

---

## 1. Node inventory

| # | Node | Owner | State | Identity |
|---|---|---|---|---|
| 1 | Windows CLI | us | on `68fcc3f1` | `12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw` -- **stable across 6 restarts, 4 SHAs** |
| 2 | Android (Pixel 6a, "Lucaso") | us | on `68fcc3f1` | `12D3KooWNnPi9wqUJ7Jypj6g4jHmW2PUTmynUs9sJY1h6SQbjLrG` -- survived in-place upgrade |
| 3 | AWS relay | us | image `sha-68fcc3f`, healthy | `12D3KooWPJK6KgKsafefLWeGs4kVbj7wBnU67yKe88ni3FHZ3Hr2` |
| 4 | macOS CLI | GPT-MAC | on `68fcc3f1` | `12D3KooWNC5rEK...` -- **has presented 3 identities today** |
| 5 | iOS ("ChristyLove") | GPT-MAC | roaming | `12D3KooWJUJ1koSWwSEAX32z6SGaepikyqpJawpojoy6gvQ8k688` |

### Access

- **AWS relay: `ssh -i ~/.ssh/scm-node-key.pem ec2-user@54.226.67.101`**.
  The user is `ec2-user`, NOT `ubuntu` -- the box is Amazon Linux 2023. Getting
  this wrong wastes an hour and produces a false "no access" conclusion.
  Docker commands need `sudo`.
- **Android**: wireless adb. Serial is
  `adb-26261JEGR01896-6pHTac._adb-tls-connect._tcp`. Two entries sometimes
  appear and one contains a space -- parsing `adb devices` with `awk '{print $1}'`
  truncates the serial. Use the full string with `-s`.

---

## 2. TRANSPORT CONTRACT -- get this wrong and everything looks dead

Each node resolves the dual-bind race **differently**. There is no single
convention. Verified by probe (connect, then attempt an RFC 6455 upgrade):

| Node | Plain TCP | WebSocket |
|---|---|---|
| Windows | 80, 443, 8080, 9001, 9090 | 9002 only |
| Android | **80 only** | 443, 8080, 9001, 9002, 9090 |
| AWS relay | none reachable | **9001 only** |

**AWS relay multiaddr (note the `/ws`):**

```
/ip4/54.226.67.101/tcp/9001/ws/p2p/12D3KooWPJK6KgKsafefLWeGs4kVbj7wBnU67yKe88ni3FHZ3Hr2
```

A plain-TCP dial to that same port **times out and looks exactly like a dead
host**. Only 9001 and 9876 are open in the security group.

**Re-probe before assuming.** Addresses churn constantly -- the Pixel was
`.141 -> .107 -> .131 -> .111` in one session.

---

## 3. What we fixed and proved today

1. **AWS relay in the routing path for the first time.** Its only bootstrap was
   `/ip4/127.0.0.1/tcp/19001` -- loopback, nothing serving. Fixed at runtime via
   `SC_BOOTSTRAP_NODES` (no rebuild). The compiled default is an EMPTY list.
2. **Relay container had no persistent volume** (`"Mounts": []`), so every
   upgrade silently discarded its state. Now mounted at `/opt/scm-relay-data`.
3. **Android upgraded in place**, identity preserved, and the relay is in its
   ledger **verified on-device** -- 52 PeerId refs, 5 IP refs in
   `files/ledger.json`.
4. **Bidirectional CLI coordination** with GPT-MAC, receipt-backed both ways.
5. **Local contact repair**: auto-created contacts store the **PeerId in the
   `public_key` field**, which blocks all outbound sends to that peer while
   inbound and ACKs keep working. Fix: derive the real key with
   `scripts/peerid_to_pubkey.py <PeerId>` and re-POST `/api/contacts`.
6. **PR #144 merged** -- Android mDNS parity + address-level dial dedup.

---

## 4. Blockers holding the five-node run

1. **P0 request-response panic.** Reproduced ON the anchor WITH our dial-dedup
   fix: 13m23s, six simultaneous connections to one peer. Address-level dedup is
   insufficient -- the contended resource is the PEER, not the address.
   GPT-MAC's `4083e59b` claims a per-peer connection cap; **verify it**.
2. **Receipt loop open.** `core/src/iron_core.rs:3423-3444` decodes a receipt,
   fires the delegate callback, never calls `mark_message_sent`
   (`iron_core.rs:1008`). `4083e59b` claims to fix this; **verify it**.
3. **Relay not used as fallback for unreachable peers.** Two-platform evidence:
   Windows builds 29 circuit dials but anchors them on the peer's carrier/stale
   addresses rather than the relay it is connected to; Android holds 52 relay
   ledger entries and mentions the relay **once** in 4000 log lines with
   `peersDiscovered=0`. Codebase-wide, not platform-specific.
4. **macOS identity instability.** Three PeerIds in one day. Unanswered whether
   its data directory survives rebuilds.

---

## 5. Open tickets filed today (all on `main`)

- `P0_DUAL_BIND_TCP_AND_WS_ON_SAME_PORT_2026-08-10.md`
- `P0_NO_MOBILE_BOOTSTRAP_MEANS_NO_OFF_LAN_RENDEZVOUS_2026-08-10.md`
- `P0_NO_RELAY_FALLBACK_FOR_ROAMING_PEERS_2026-08-10.md` (includes a correction)
- `P1_ASYNC_DELIVERY_RECEIPTS_DO_NOT_CONVERGE_2026-08-10.md` (root cause found)
- `P1_NESTED_CIRCUIT_ADDRESSES_STILL_FORMED_2026-08-10.md`
- `P1_STALE_BUILD_PROVENANCE_INVALIDATES_SHA_CLAIMS_2026-08-09.md` (fixed upstream)
- `P1_TEST_FIXTURE_ADDRESSES_LEAKED_INTO_LIVE_LEDGER_2026-08-09.md`
- Plans: `FIVE_NODE_UNIFIED_TEST_PLAN_2026-08-09.md`,
  `ITERATION_2_NAT_TRAVERSAL_TEST_2026-08-10.md`,
  `COORDINATION_FALLBACK_PROTOCOL_2026-08-10.md`

---

## 6. GOTCHAS -- every one of these produced a wrong conclusion

### 6.1 The big one: absence from a blind measurement

**Three findings were WRONG today for the same reason.** Before reporting that
something is absent, confirm the measurement is CAPABLE of showing it.

| Wrong conclusion | Actual cause |
|---|---|
| "port 9001 is not advertised" | read `/api/listeners` through `head -c 400`; the response was truncated |
| "mobile-to-CLI messaging is broken" | only `history_sync`/`identity_sync` envelopes had been sent; those legitimately carry empty text |
| "zero relay circuit attempts" | regex terminated at `/tcp/<port>` and discarded the `/p2p-circuit` suffix |

The repo's own rule names it: *"a check that did not run looks identical to a
check that found nothing."*

### 6.2 Monitors and hooks

- **A subagent's background children outlive the agent.** A CLI supervisor
  spawned three watchers, ended its turn, and orphaned them for 9+ hours.
  Long-lived watches belong on a Monitor the orchestrator owns.
- **`tail -F` on a fixed log path goes deaf when the node restarts** onto a new
  log file. Resolve the newest log dynamically. This cost an hour of missed
  operator messages.
- **A watch that exits on panic loses the fallback channel exactly when it is
  needed.** Report and keep watching.

### 6.3 Windows / shell

- `tasklist /FO CSV /NH` returns NOTHING under Git Bash -- it mangles `/FO`.
  Use `tasklist | grep -i <name>`.
- **Never read `$?` after a pipe** -- it reports the last command, so a piped
  gate can never fail.
- `./gradlew` does not exist; use the **absolute path** to `gradlew.bat`. It is
  not on PATH under `build_lock.py`.
- **A running node holds its own `.exe`**, so cargo cannot relink. STOP THE NODE
  before any rebuild or you get `Access is denied (os error 5)`.
- `git show <rev>:<path>` gets mangled by MSYS -- prefix `MSYS_NO_PATHCONV=1`.
- **Backticks in a double-quoted bash string get executed.** Posting a PR
  comment with `gh pr comment --body "...`code`..."` blanks the code references.
  Use `--body-file`.

### 6.4 Builds and disk

- Disk hits 100% easily. An Android build needs ~12 GB for four ABIs -- build
  **arm64-v8a only** with `-PabiFilters=arm64-v8a` (the Pixel is arm64).
- `cargo clean --target <triple>` wipes ALL of `target/`. Use
  `scripts/clean_target.sh`.
- Provenance: GPT-MAC's `build.rs` fix works -- `--version` reports the correct
  SHA without env vars. **Verified.** The old `SCM_GIT_HASH` workaround is
  retired.

### 6.5 Android

- **Doze blocks inbound connections.** The app can be running and listening on
  six ports while every probe times out. `adb shell input keyevent KEYCODE_WAKEUP`
  then `dumpsys deviceidle whitelist +com.scmessenger.android`.
- **`dumpsys activity services` reports NO ServiceRecord** for a mesh service
  that is demonstrably running. Do not use it as a readiness check.
- **`adb install -r` preserves identity** only with the local debug keystore. A
  CI-built APK has a different signature and forces a wipe.
- **`adb shell logcat -G 16M`** before any run. The 256 KiB default already
  destroyed one field test's evidence.

### 6.6 Scoring rules

- **Sender-side delivery status is unreliable.** Score on receiver-side
  `inbox_receive` plus ACK. A non-200 from `/api/send` is not proof of
  non-delivery; a 200 is not proof of delivery.
- **A node that died mid-window invalidates every delivery attributed to it.**
- `[auto-reply]` echoes inflate count-based metrics. Score distinct message IDs
  plus direction.
- **Release builds do not panic on the `debug_assert`** -- they accumulate the
  drift silently. A green release run is NOT evidence of a fix.

---

## 7. Live infrastructure (running now)

- **Windows node**: `scm-winlane/target/debug/scmessenger-cli.exe start`, logs at
  `tmp/logs/win_68fcc3f1_run5_std{out,err}.log`. Launch with the full debug
  `RUST_LOG` filter and **stdout/stderr to SEPARATE files** -- the panic never
  reaches the rolling `scm.log`.
- **Monitors**: an inbound-CLI-message wake hook and a PR#139-comments + node
  health watch. Both survive node restarts.
- **Artifacts**: `tmp/fieldtest_20260810T042428Z/` -- 12 MB, Windows (6 runs),
  Android (logcat + ledger), AWS relay. `INDEX.md` has the analysis targets.
  macOS and iOS halves still outstanding from GPT-MAC.
- **Known-good rollback binary**: `tmp/known_good/scmessenger-cli-d48558a8.exe`
  (9h32m proven uptime) and a cold identity backup at
  `tmp/known_good/identity_cold_20260810T013925Z/`.

---

## 8. Next actions, in order

1. **Review GPT-MAC's `4083e59b`.** It claims fixes for three of our findings:
   receipt-loop close, per-peer connection cap, bounded auto-reply. **Verify
   each against the tickets rather than accepting the claim** -- especially the
   connection cap, since our own dial-dedup fix looked right and did not stop
   the panic.
2. **Answer or chase the five open questions** in the PR escalation comment.
   Two block the run: which macOS PeerId is canonical, and is the relay in the
   iOS ledger verified on-device.
3. **Re-anchor once** to a named SHA with finished CI, rebuild all three of our
   nodes, capture provenance.
4. **Then** attempt the five-node run, or the iteration-2 roaming test.

---

## 9. Working agreement with the GPT-MAC lane

- PR #139 comments are PRIMARY. A decision that exists only in a CLI message
  does not exist.
- Both lanes state the build stamp of the **running process**, not the SHA they
  intended to build.
- Re-anchor deliberately, once, on a named SHA -- never chase.
- Report what was observed. Never infer delivery, never infer propagation.
  Inference about Android's ledger was wrong tonight; reading the device settled
  it in one command.
