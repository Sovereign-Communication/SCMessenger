# Windows/Android/AWS Node Parity & 5-Node Gate Readiness

**Status:** Active — orchestrator session, 2026-08-12
**Authority:** Windows orchestrator (control-plane owner per Mac exit handoff)
**Session constraint:** No builds/tests/cargo (Antigravity consuming compute). Read-only verification only.
**PR:** #139, branch `tracking/pre-v040-tag-work`, local HEAD `9f280e19`, origin `ab9c34f7`

---

## 1. Live Node Parity Table (verified 2026-08-12T15:10Z)

| # | Node | SHA | Branch | PeerId | Status | Provenance Source |
|---|------|-----|--------|--------|--------|-------------------|
| 1 | Windows CLI | `9f54b107` | `gpt/pr139-receipt-filter-20260811` | `12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw` | RUNNING (PID 22004, port 9876) | `/version` HTTP endpoint |
| 2 | Android (Pixel 6a) | v0.4.0/vCode14 | unknown (no git_hash in logs) | `12D3KooWG3qdZPvnsRZ6RJwa87bin7GG8zSaoaMEbMzee3vCSTFL` | RUNNING | `dumpsys package` + mesh log |
| 3 | AWS headless | `9f54b107` | `gpt/pr139-receipt-filter-20260811` | `12D3KooWKMUXfjvWeodBUJbSwBuRXBU3d6XSbP1AJXL9WhaS3yKy` | RUNNING (healthy, 2 peers) | `/version` HTTP endpoint |
| 4 | macOS CLI | `e7ac25c4` | local checkout `a29e53f3` | `12D3KooWNC5rEKFhuxDNDNsJ6Q58Ca75LnxfjUqspGzGRdYRUWyt` | OFFLINE (Mac lane exited) | PR #139 comment 2026-08-12T10:35Z |
| 5 | iOS (iPhone) | 0.4.0/build 9 | unknown | `12D3KooWNnPi9wqUJ7Jypj6g4jHmW2PUTmynUs9sJY1h6SQbjLrG` | OFFLINE | PR #139 comment history |

**Key finding:** Windows and AWS are on the **same SHA** (`9f54b107`) — but it's from `gpt/pr139-receipt-filter-20260811`, NOT from `tracking/pre-v040-tag-work`. This is an older GPT-MAC side branch.

**Android gap:** The APK was installed from a CI artifact but has **no embedded git_hash** — commit `816422fc` (which adds `SCM_GIT_HASH` to the APK build) is on our branch but was NOT in the build that produced the installed APK. Android provenance cannot be verified at the SHA level.

---

## 2. Critical Branch Divergence

The two lanes diverged at merge-base `6e50963d`. Each has fixes the other needs:

### Our branch (`tracking/pre-v040-tag-work`, HEAD `9f280e19`)
Unique commits not in the GPT branch:
- `c242fb53` — **fix(cli): per-peer connection cap and port-stripped self-dial guard** (PF-3 partial)
- `816422fc` — **ci(android): export SCM_GIT_HASH in the APK build step** (PF-6 partial)
- `ab9c34f7` — style(cli): cargo fmt the new P1 test assertions
- `9f280e19` — docs(handoff): ticket identity-backup UI integration (docs only)
- `11710cf3` — fix(bridge): classify bare delivery receipts as housekeeping to stop ACK storm
- Plus several docs/handoff commits

### Their branch (`gpt/pr139-receipt-fix-20260812`, tip `7538e4e9`)
Unique commits not in our branch:
- `860f5ed5` — **fix(core): admit connections before request-response state** (THE P0 PANIC FIX)
- `4d445899` — fix: classify bare delivery receipts as housekeeping
- `41410513` — fix: suppress delivery receipts from chat UI paths
- `e7ac25c4` — fix: classify sent receipts as housekeeping
- `9f54b107` — fix CI provenance and SwiftLint gate
- `ab4f4486` — test: serialize socket activation environment checks
- `73444f89` — fix(core): keep delivery receipts out of user history
- `7538e4e9` — style: format receipt regression assertions

**Merge test:** `git merge-tree --write-tree HEAD origin/gpt/pr139-receipt-fix-20260812` → **clean, no conflicts**. Both branches touch the same files but in non-overlapping regions.

---

## 3. PF Scope Item Status (from subagent audit)

| ID | Item | Status | Evidence |
|----|------|--------|----------|
| **PF-1** | Finite-attempt abandonment | **P0 BLOCKER — STILL OPEN** | `outbox.rs:66` `MAX_DELIVERY_ATTEMPTS=12`; `relay_custody.rs:746` same. Android log shows attempt 49 on AWS-bound messages. Philosophy violation active. |
| **PF-2** | Receipt/outbox convergence | **FIXED** | `iron_core.rs:3460` calls `mark_message_sent()` on Delivered/Read receipts. |
| **PF-3** | Request-response stability | **PARTIALLY FIXED** | `c242fb53` (per-peer cap) suppresses trigger. `860f5ed5` (admission ordering) fixes root cause. **BOTH needed** — currently split across branches. Debug_assert in libp2p 0.29.0 remains; release builds skip it. |
| **PF-4** | Headless relay fallback | **FIELD-GATE ONLY** | No code blocker; needs physical proof. |
| **PF-5** | Identity stability | **FIELD-GATE ONLY** | Data dir `%LOCALAPPDATA%\scmessenger\` survives rebuilds. |
| **PF-6** | Exact provenance | **PARTIALLY FIXED** | `816422fc` adds SCM_GIT_HASH to APK CI (not in installed APK). `7e527df0` adds Android provenance. Running nodes report stale `9f54b107`. |
| **PF-7** | Harness parity | **NOT STARTED** | `scripts/run5.sh` outdated; no shared scorer/lane drivers. |
| **PF-8** | Signing lineage | Post-gate (deferred) | — |
| **PF-9** | Security BLOCK reconciliation | **PARTIALLY REMEDIATED** | F1-F5 addressed in commits, but review was against `6cb7033a`. Needs fresh reconciliation against the final candidate. |
| **PF-10** | Candidate ordering / self-dial | **PARTIALLY FIXED** | `c242fb53` adds port-stripped self-dial guard. Full candidate-ladder proof needed. |
| **PF-11** | BLE liveness | **FIELD-GATE ONLY** | macOS BLE fixes landed (`e80d658b`, `30f4ee9d`); Android BLE storm unverified at current SHA. |
| **PF-12** | Capacity semantics | **P1 — OPEN** | Tied to PF-1. No audit of queue/retention vs age/attempt abandonment. |

---

## 4. Blockers Before Freeze (ordered by priority)

1. **PF-1 (P0):** `MAX_DELIVERY_ATTEMPTS=12` in `core/src/store/outbox.rs:66` and `core/src/store/relay_custody.rs:746` silently abandons accepted undelivered messages. This violates the delivery philosophy canon. **Android is actively hitting this** — attempt 49 observed on AWS-bound messages. Requires `core/src/store/` change → Rule 8 adversarial review.

2. **Branch merge:** `gpt/pr139-receipt-fix-20260812` must be merged into `tracking/pre-v040-tag-work` to land `860f5ed5` (the P0 panic fix). Clean merge confirmed — no conflicts. The `860f5ed5` commit touches `core/src/transport/behaviour.rs` (Rule 8 path) — it's a 5-line field reorder (moves `connection_limits` before request-response in the derive macro), not new logic, but still needs review.

3. **PF-9 fresh security reconciliation:** The adversarial review was against `6cb7033a`. F1-F5 remediation must be re-verified against the final merged SHA.

4. **PF-3 operator exception decision:** The per-peer cap (`c242fb53`) + admission ordering (`860f5ed5`) suppress the trigger, but the `debug_assert` in libp2p-request-response 0.29.0 remains. Release builds skip debug_assert. Operator must decide: accept release-only for gate (explicit OPERATOR EXCEPTION) or implement a workaround.

5. **PF-7 harness:** No shared scorer/lane drivers exist. Cannot run the matrices without them.

---

## 5. Re-Pin Procedure (when a SHA is ready to freeze)

### Windows CLI
```
1. STOP: taskkill /PID <pid> /F  (or graceful stop if supported)
2. RECORD: PeerId, identity_id, data dir path
3. CHECKOUT: git checkout <FROZEN_SHA>
4. BUILD (when compute available): cargo build -p scmessenger-cli --release
5. VERIFY: scmessenger-cli.exe --version → must report <FROZEN_SHA>
6. VERIFY IDENTITY: start node, check /api/identity → PeerId must match
7. RECONNECT: verify 3 peers re-establish, ledger convergence
```

### AWS headless (teardown + rebuild — no SSH key locally)
```
1. TERMINATE: aws ec2 terminate-instances --instance-ids <id> (by tag Name=scm-always-on-node)
2. REUSE: SG sg-02288078fa0b39b92, key pair scm-node-key
3. LAUNCH: t3.micro, AL2023, user-data:
   - dnf install -y docker && systemctl enable --now docker
   - docker pull testbotz/scmessenger@sha256:<digest>  # MUST use digest, NOT :latest
   - docker run ... scm start --port 9000 --http-bind 0.0.0.0:9876
4. IP WILL DRIFT — update AWS_RELAY_CURRENT_ADDRESS.md and all node configs
5. VERIFY: /health 200, /version reports <FROZEN_SHA>, /api/identity initialized:true
6. PROPAGATE: update bootstrap multiaddr on every other node's config.json
```

### Android (in-place APK install)
```
1. DOWNLOAD: CI artifact for <FROZEN_SHA> (Mobile workflow android-debug-apk)
2. INSTALL: adb install -r <apk>  (preserves identity/contacts/history)
3. VERIFY: check mesh log for git_hash/provenance line
4. VERIFY IDENTITY: PeerId unchanged
```

---

## 6. Live Fleet Connectivity (verified now)

- Windows ↔ AWS: **CONNECTED** (both see each other in /api/peers)
- Windows ↔ Android: **CONNECTED** (Android mesh log shows connection to Windows at 192.168.0.121:9001)
- Android ↔ AWS: **FAILING** — Android marks AWS peer as dead after 3 failed dial attempts, attempt 42-49 observed. Root cause: bootstrap PeerId mismatch (nodes reference old `12D3KooWPJK6...` instead of current `12D3KooWKMUXf...`)
- macOS ↔ anything: **OFFLINE** (Mac lane exited)
- iOS ↔ anything: **OFFLINE**

---

## 7. Dispatch Plan (when compute frees up)

| # | Task | Model/Lane | Priority | Rule 8? |
|---|------|-----------|----------|---------|
| 1 | Merge `gpt/pr139-receipt-fix-20260812` into `tracking/pre-v040-tag-work` | Windows orchestrator | P0 | `860f5ed5` touches `core/src/transport/` — needs review |
| 2 | PF-1: Remove `MAX_DELIVERY_ATTEMPTS` hard cap, implement durable outstanding-delivery state | qwenpaid (qwen3.8-max or glm-5.2) | P0 | Yes (`core/src/store/`) |
| 3 | Rule 8 adversarial review of merged candidate | Cross-model (different from implementer) | P0 | — |
| 4 | Freeze the merged + reviewed SHA | Operator decision | P0 | — |
| 5 | Rebuild + re-pin Windows CLI from frozen SHA | Windows orchestrator | P1 | No |
| 6 | Rebuild + re-pin AWS from Docker image of frozen SHA | Windows orchestrator (aws CLI) | P1 | No |
| 7 | Rebuild + install Android APK from CI artifact of frozen SHA | Windows orchestrator (adb) | P1 | No |
| 8 | Build PF-7 harness (shared scorer + lane drivers) | qwenpaid | P1 | No |
| 9 | Run 5-node gate (2 matrix passes + 60-min soak) | Operator + synchronized | P2 | No |

---

## 8. Provider Availability (verified 2026-08-12T15:10Z)

| Provider | Status | Models |
|----------|--------|--------|
| qwenpaid (alibaba coding-plan) | [OK] LIVE (quota reset 04:44 UTC) | qwen3.8-max, qwen3.7-max, qwen3.7-plus, glm-5.2, deepseek-v4-pro, deepseek-v4-flash-0731 |
| OpenRouter (free tier) | [OK] Available | gpt-oss-20b:free, nvidia/nemotron-3-ultra, google/gemma-4, cohere/north-mini-code |
| AGY (Antigravity) | [WARNING] Heavy compute in use — avoid | claude-sonnet-4-6, claude-opus-4-6-thinking, gemini-3.6-flash |
| Ollama | [FAIL] Not installed | — |
| openai-oauth proxy | [FAIL] Not responding (127.0.0.1:10531) | — |

---

## Resume point

A fresh session should:
1. Read this file
2. Read `HANDOFF/plans/PR139_FIVE_NODE_FIELD_GATE_REFERENCE.md` (the gate authority)
3. Read `HANDOFF/todo/_QUEUE.md` (dispatch order)
4. Check if Antigravity compute is still heavy before any build
5. Start from dispatch plan item 1 (branch merge)
