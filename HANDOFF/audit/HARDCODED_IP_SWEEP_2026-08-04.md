# Hardcoded IP Address Sweep -- 2026-08-04

Status: Audit complete, no files modified
Scope: All 2,736 git-tracked files in the repository
Author: analysis pass (read-only); no build tools invoked, no tracked file edited

## Method

A dotted-quad IPv4 regex (`(?<![0-9A-Za-z.])((?:\d{1,3}\.){3}\d{1,3})(?![0-9A-Za-z.])`)
was run over every git-tracked file, skipping binaries and archives. Each literal was
parsed with Python `ipaddress` and bucketed by RFC class. An IPv6 pass
(`/ip6/` + hex-group regex) was run separately.

The first regex attempt excluded a leading hyphen in the lookbehind and silently
missed every `${VAR:-<ip>}` shell default. That cost four real findings
(`scripts/live-smoke.sh`, `scripts/preflight.sh`, `scripts/test_gcp_node.sh`,
`docs/SCRIPT_IMPLEMENTATION_PLAN.md`). The counts below are from the corrected pass.

[WARNING] Nothing in this report asserts that any address is live, correct, or
reachable. No network probe was performed. Every address named here should be
treated as unverified and ephemeral, which is the entire point of the audit.

## Executive summary

| Classification | Hits | Distinct files |
|---|---|---|
| STALE-CRITICAL | 91 | 21 |
| STALE-DOC | 13 | 10 |
| HISTORICAL | 869 | 54 |
| BENIGN | 3,225 | -- |
| FALSE-POSITIVE (filtered) | 7 | -- |
| **Total literals matched** | **4,205** | **~300** |

Ten distinct routable addresses account for every STALE finding:

| Address | Total hits | Role as described in repo |
|---|---|---|
| `34.135.34.73` | 432 | GCP bootstrap relay (us-central1) |
| `104.28.216.43` | 383 | secondary "Cloudflare" relay |
| `100.56.248.69` | 99 | AWS alpha relay / cloud node |
| `32.197.246.78` | 38 | AWS farm-sim instance `i-00e068c0837ac0857` |
| `34.168.102.7` | 6 | older GCP node |
| `54.242.56.150` | 5 | AWS node `i-078cb870316683e79` |
| `13.220.17.4` | 4 | AWS Android-emulator node |
| `74.244.37.79` | 3 | observed WAN address in RCA docs |
| `147.81.41.188` | 1 | operator home fiber public IP |
| `136.117.121.95` | 1 | example value in a compose comment |

### Two contradictions found in the repo itself

1. **The relay is recorded as terminated while runbooks still dial it.**
   `infra/ec2/alpha-relay-state.json` records
   `"state": "TERMINATED"`, `"terminated_at": "2026-07-30"`,
   `"former_public_ip": "100.56.248.69"`. Meanwhile
   `HANDOFF/todo/AWS_CLOUD_NODE_PRELAUNCH_CHECKLIST.md`,
   `HANDOFF/todo/QWEN_RUN2_CLOUD_NODE_VERIFICATION.md`,
   `HANDOFF/review/V040_S4_DELIVERY_PROOF_RUNBOOK.md` and
   `HANDOFF/review/V040_S5_JOSH_WAN_RUNBOOK.md` all instruct an operator to SSH,
   curl and dial that same literal. The state file is the only artifact in the repo
   that got it right, and nothing reads it.

2. **A live task file describes code that no longer exists.**
   `HANDOFF/todo/V1_INSTALL_ARTIFACT_FOR_ALPHA_TESTERS.md:63` says
   `getBootstrapNodesForSettings()` returns `listOf("/ip4/100.56.248.69/tcp/9001")`
   at `MeshRepository.kt:87`. The real
   `getBootstrapNodesForSettings()` is at
   `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:5558-5564`
   and returns `ledgerManager?.getPreferredRelays(MAX_SETTINGS_RELAYS)`. The
   hardcoded fallback was removed; only a doctrine comment at line 78 mentions it.
   The Android client is already de-hardcoded. The docs are not.

## STALE-CRITICAL

Routable address used as a live endpoint: a bootstrap default, a health-check
target, an SSH/SCP target, or a dial target in an active checklist or runbook.
These actively mislead an operator or a program.

### Executable code and scripts (13 sites)

| File:line | Address | Context | Note |
|---|---|---|---|
| `headless/main.js:9` | `34.135.34.73` | `"/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETat..."` | entry in `DEFAULT_BOOTSTRAP_NODES`, no override path |
| `headless/main.js:10` | `104.28.216.43` | `"/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmu..."` | same array |
| `ui/app.js:6` | `34.135.34.73` | `"/ip4/34.135.34.73/tcp/9001/ws/p2p/12D3KooWETat..."` | `DEFAULT_BOOTSTRAP`, seeds `localStorage` key `scm.desktop.bootstrap.v1` |
| `ui/app.js:7` | `104.28.216.43` | `"/ip4/104.28.216.43/tcp/9010/ws/p2p/12D3KooWHpmu..."` | same array |
| `scripts/test_all_bootstrap_nodes.sh:20` | `34.135.34.73` | `"34.135.34.73:9001"       # GCP primary (us-central1)` | fallback when `SCMESSENGER_BOOTSTRAP_NODES` unset |
| `scripts/test_all_bootstrap_nodes.sh:21` | `104.28.216.43` | `"104.28.216.43:443"       # Cloudflare relay` | same fallback array |
| `scripts/live-smoke.sh:19` | `34.135.34.73` | `GCP_RELAY_IP="${GCP_RELAY_IP:-34.135.34.73}"` | env override exists, literal is the default |
| `scripts/preflight.sh:124` | `34.135.34.73` | `GCP_IP="${GCP_RELAY_IP:-34.135.34.73}"` | feeds `nc -z -w 3 "$GCP_IP" "$GCP_PORT"` |
| `scripts/test_gcp_node.sh:9` | `34.135.34.73` | `GCP_IP="${1:-34.135.34.73}"` | positional override only |
| `docs/SCRIPT_IMPLEMENTATION_PLAN.md:137` | `34.135.34.73` | `GCP_IP="${GCP_RELAY_IP:-34.135.34.73}"` | script template that will be copied verbatim |
| `docs/CURRENT_STATE.md:1147` | `34.135.34.73` | `nc -zv 34.135.34.73 9001` | copy-paste block titled "Commands to Run" |
| `docs/CURRENT_STATE.md:1151` | `34.135.34.73` | `print("relay_health_check host=34.135.34.73 port=9001")` | same block |
| `docker-compose.yml:17` | `136.117.121.95` | `# Example: SC_BOOTSTRAP_NODES=/ip4/136.117.121.95/tcp/9001/p2p/12D3KooW...` | see STALE-DOC; listed here because it is a real routable address used as an example |

### Active checklists and runbooks -- `100.56.248.69` (57 sites)

| File | Lines | Representative context |
|---|---|---|
| `HANDOFF/todo/AWS_CLOUD_NODE_PRELAUNCH_CHECKLIST.md` | 1, 16, 20, 28, 54, 67, 82 | `- [ ] Can SSH in: ssh -i <key> ec2-user@100.56.248.69 "hostname"`; `curl -s http://100.56.248.69:9876/health` |
| `HANDOFF/todo/QWEN_RUN2_CLOUD_NODE_VERIFICATION.md` | 12, 25, 65 | `ssh -i ~/.ssh/aws_key ubuntu@100.56.248.69` |
| `HANDOFF/review/V040_S5_JOSH_WAN_RUNBOOK.md` | 43, 59, 60, 94, 108, 111, 124, 140, 142, 180, 224, 233, 234, 236, 251 | `curl -fsS http://100.56.248.69:9876/health`; `set SC_BOOTSTRAP_NODES=/ip4/100.56.248.69/tcp/9001` |
| `HANDOFF/review/V040_S4_DELIVERY_PROOF_RUNBOOK.md` | 10, 46, 47, 48, 98, 107, 111, 116, 117, 127, 131, 170, 173 | `Test-NetConnection 100.56.248.69 -Port 9001`; `Cloud node facts: /ip4/100.56.248.69/tcp/9001` |
| `HANDOFF/ALPHA_TEST_LUCAS_JOSH_SETUP.md` | 44, 54, 65, 89, 119, 140, 158, 182 | `scm config bootstrap add /ip4/100.56.248.69/tcp/9001` (given to two external alpha testers) |
| `HANDOFF/plans/V040_ORCHESTRATION_PLAN.md` | 21, 32, 257 | `- Alpha relay: LIVE at 100.56.248.69:9001, containerized, restart policy` |
| `HANDOFF/plans/FIVE_NODE_RUN_2_PLAN.md` | 17, 106 | `| 5. Cloud Node | AWS (100.56.248.69) | Qwen / AWS | Needs verification |` |
| `HANDOFF/todo/V1_INSTALL_ARTIFACT_FOR_ALPHA_TESTERS.md` | 63 | claims Kotlin returns `listOf("/ip4/100.56.248.69/tcp/9001")` -- no longer true |
| `HANDOFF/gpt/GPT_TAKEOVER_2026-08-01_WINDOWS_WINDDOWN.md` | 141, 171 | narrative reference to the old node address |

The `V040_ORCHESTRATION_PLAN.md:32` line asserting the relay is "LIVE" is the most
dangerous single line in this set: it is a bare claim of health with no date and no
verification method attached.

### Active checklists -- other addresses (21 sites)

| File | Lines | Address | Representative context |
|---|---|---|---|
| `HANDOFF/todo/EXECUTE_PHASE_2_3_ON_INSTANCE.md` | 5, 20, 33, 65, 121, 134, 160, 185, 219, 255, 299, 325 | `32.197.246.78` | `ssh -i scmessenger-farm-sim-key.pem ubuntu@32.197.246.78 << 'EOF'` (11 separate SSH/SCP invocations) |
| `HANDOFF/IN_PROGRESS/FARM_SIM_PHASE_2_3_COMPREHENSIVE_TESTING.md` | 4, 6, 16, 319 | `32.197.246.78` | `Instance: i-00e068c0837ac0857 at 32.197.246.78 (key: ./scmessenger-farm-sim-key.pem)` |
| `HANDOFF/todo/FARM_SIM_PHASE_2_3_FINDINGS.md` | 4, 392 | `32.197.246.78` | `- SSH: ec2-user@32.197.246.78` |
| `HANDOFF/gpt/GPT_TAKEOVER_2026-08-01_WINDOWS_WINDDOWN.md` | 130, 132 | `54.242.56.150` | `SSH: ssh -i ~/.ssh/scm-node-key.pem ubuntu@54.242.56.150` |
| `HANDOFF/IMMEDIATE_NEXT_STEPS.md` | 119 | `34.135.34.73` | `ping 34.135.34.73` under a numbered "Steps" heading |

## STALE-DOC

Prose or comments describing infrastructure. Misleading to a reader, not executed.

| File:line | Address | Context |
|---|---|---|
| `AGENTS.md:22` | `100.56.248.69` | `not a role. The AWS instance at 100.56.248.69 is a CLOUD NODE: a full node` |
| `HANDOFF/FINAL_HANDOFF_2026-08-04.md:42` | `100.56.248.69` | `\| Cloud node verification (100.56.248.69) \| Qwen \| [BLOCKED] SSH auth via IAM` |
| `HANDOFF/FINAL_HANDOFF_2026-08-04.md:63` | `100.56.248.69` | `### 2. Cloud Node (100.56.248.69)` |
| `HANDOFF/ORCHESTRATOR_HANDOFF_2026-08-04.md:39` | `100.56.248.69` | `**Cloud node SSH access** -- Need IAM user auth for 100.56.248.69` |
| `HANDOFF/ORCHESTRATOR_HANDOFF_2026-08-04.md:69` | `100.56.248.69` | `- [ ] SSH to 100.56.248.69 using IAM auth` |
| `HANDOFF/ORCHESTRATOR_HANDOFF_2026-08-04.md:170` | `100.56.248.69` | `**Cloud node SSH access** -- 100.56.248.69 needs IAM auth` |
| `HANDOFF/LAUNCH_NEXT.md:24` | `100.56.248.69` | `### 2. Cloud Node (100.56.248.69)` |
| `HANDOFF/NEXT_ORCHESTRATOR_PROMPT.md:18` | `100.56.248.69` | `### 2. Cloud Node -- 100.56.248.69` |
| `HANDOFF/gpt/GPT_IOS_LANE_KICKOFF.md:59` | `100.56.248.69` | `"/ip4/100.56.248.69/tcp/9001"` -- DELETE. Source bootstrap from the ...` |
| `HANDOFF/gpt/GPT_PLANNING_040_050.md:26` | `100.56.248.69` | `Pennsylvania over the live AWS relay (100.56.248.69:9001, runs` |
| `docs/orchestration/LUCAS_JOSH_AND_FARM_SIM_REMAINING_TASKS.md:17` | `100.56.248.69` | `... reliably across the real internet through the AWS relay (100.56.248.69:9001)` |
| `android/.../data/MeshRepository.kt:78` | `100.56.248.69` | `// "/ip4/100.56.248.69/tcp/9001" fallback and the dead BootstrapSource /` |
| `docker-compose.yml:17` | `136.117.121.95` | `# Example: SC_BOOTSTRAP_NODES=/ip4/136.117.121.95/tcp/9001/p2p/12D3KooW...` |

`MeshRepository.kt:78` is the one entry here that is correct as written -- it documents
the removal of the literal and is useful context. It only needs the ephemerality
warning, not a rewrite. `docker-compose.yml:17` should use a documentation-range
address (`192.0.2.x`, RFC 5737) rather than a real routable one.

## HISTORICAL

869 hits across 54 files. Frozen records of a past moment. **These should not be
rewritten.** Their addresses were accurate when captured and rewriting them destroys
the evidentiary value of the record. They need a one-line ephemerality banner, nothing
more.

| File | Hits | Addresses |
|---|---|---|
| `android/android_logcat_4-23-26.md` | 546 | `34.135.34.73`, `104.28.216.43` |
| `android/android_logcat_4-22-26.md` | 156 | `34.135.34.73`, `104.28.216.43` |
| `docs/historical/ADB_SESSION_AUDIT_2026-03-18.md` | 21 | `34.135.34.73`, `104.28.216.43` |
| `HANDOFF/SESSION_HANDOFF_2026-07-20_LUCAS_JOSH_ALPHA.md` | 9 | `100.56.248.69`, `13.220.17.4`, `147.81.41.188` |
| `docs/CURRENT_STATE.md` (dated snapshot, lines 2223-2296) | 9 | `34.135.34.73`, `104.28.216.43` |
| `HANDOFF/audit/redaction_scan.md` | 8 | four relay IPs, already flagged for redaction |
| `docs/historical/audits/*`, `docs/historical/plans/*` | ~40 | `34.135.34.73`, `104.28.216.43`, `34.168.102.7`, `74.244.37.79` |
| `HANDOFF/done/**`, `HANDOFF/STATE/**`, `HANDOFF/results/**` | ~35 | mixed |
| `.claude/plans/*` | 8 | `34.135.34.73`, `104.28.216.43` (already described there as "dead/hallucinated") |
| remainder (44 files) | ~37 | mixed |

Two entries deserve separate handling:

- `infra/ec2/alpha-relay-state.json:7` -- `"former_public_ip": "100.56.248.69"`.
  This is the correct pattern and should be the model for everything else: the field
  is named `former_`, it sits next to `"state": "TERMINATED"` and a date. Leave it.
- `HANDOFF/SESSION_HANDOFF_2026-07-20_LUCAS_JOSH_ALPHA.md:21` -- `147.81.41.188`
  is described in-line as the operator's home fiber public IP. That is personal
  network data in a public repository, and `HANDOFF/audit/redaction_scan.md` already
  tracks it. Out of scope for this sweep but it should not be forgotten.

## BENIGN (counts only)

| Category | Hits | Note |
|---|---|---|
| RFC1918 private (`10/8`, `172.16-31/12`, `192.168/16`) | 1,934 | Docker compose networks, LAN test fixtures, `169.254.169.254` AWS IMDS |
| Loopback (`127.0.0.0/8`) | 834 | listen defaults, unit tests |
| Unspecified / bind-all (`0.0.0.0`) | 235 | `--listen /ip4/0.0.0.0/tcp/9001`, security-group CIDRs |
| Documentation and example literals | 207 | `1.2.3.4` (116), `5.6.7.8`, `6.6.6.6`, `8.8.8.x`, `1.1.1.1`, `9.9.9.9`, plus `100.64/10`, `192.0.x`, `198.x`, `223.x` fixtures in `addr_filter.rs` range tests |
| Multicast (`224.0.0.251` mDNS, `239.255.255.250` SSDP) | 15 | protocol constants |
| **False positives filtered** | **7** | `17.0.0.14` x2 (JDK 17 version path), `124.0.0.0` x2 (Chrome UA string), `999.999.999.999` and `010.1.2.3` x2 (deliberate invalid-input test vectors in `DeepLinkValidator`) |

IPv6: no routable IPv6 literal is used as an endpoint anywhere. The only IPv6
addresses present are `::1`, `::`, `fe80::`, `2001:db8::` and four Teredo/6to4
fixtures in `core/src/transport/addr_filter.rs:1088-1103` used to test address
filtering. Nothing to action.

Two BENIGN items worth noting even though they are not stale:

- `android/.../network/NetworkDiagnostics.kt:99,150` and
  `NetworkTypeDetector.kt:56` connect to `8.8.8.8` on ports 9001/9010 to detect port
  blocking. `8.8.8.8` does not drift, so this is not a staleness problem, but it is a
  hardcoded third-party network dependency in shipping Android code, which sits
  awkwardly against the "no third-party network dependencies" line in
  `HANDOFF/review/V040_S4_DELIVERY_PROOF_RUNBOOK.md:46`.
- `docker/docker-compose.test.yml:271,300,330` uses subnet `172.32.0.0/24`.
  `172.32/16` is **outside** RFC1918 (which stops at `172.31`) and is publicly
  allocated space. Harmless inside a Docker bridge, but it is not the private range
  the file appears to intend.

## De-hardcoding recommendation

### What already exists (do not invent a new mechanism)

The repository already has a complete, layered configuration path. Nothing new is
needed; the STALE-CRITICAL sites simply bypass it.

| Layer | Location | Behavior |
|---|---|---|
| Runtime env var | `cli/src/bootstrap.rs:36-46` | `SC_BOOTSTRAP_NODES`, comma-separated multiaddrs, checked first |
| Build-time env var | `cli/src/bootstrap.rs:49-65` | `option_env!("SC_BOOTSTRAP_NODES")` |
| Compiled default | `cli/src/bootstrap.rs:26` | `pub const DEFAULT_BOOTSTRAP_NODES: &[&str] = &[];` -- deliberately empty |
| Persisted user config | `cli/src/config.rs:45, 288, 304` | `bootstrap_nodes: Vec<String>` in `config.json`, edited via `scm config bootstrap add/remove`; default is `Vec::new()` with the comment "No hardcoded bootstrap nodes (community ledger)" |
| Android runtime source | `MeshRepository.kt:5558-5564` | `getPreferredRelays()` off the sled-backed ledger; no literal |
| Ledger seeding | `core/src/relay/invite.rs` (`seed_ledger`) | invite/QR token carries relay addresses into a fresh node's ledger |
| Container config | `docker-compose.yml:18` | `SC_BOOTSTRAP_NODES=${SC_BOOTSTRAP_NODES:-}` -- already correct |
| Address discovery at provision time | `infra/aws/provision-relay.sh:67`, `provision-farm-sim.sh:62` | `MY_IP="$(curl -s https://checkip.amazonaws.com \|\| echo '0.0.0.0')"` |
| Node self-discovery | `scripts/get-node-info.sh:92` | `PUBLIC_IP=$(ip route get 1.1.1.1 ... \| grep -oP 'src \K\S+')` |
| Instance state of record | `infra/ec2/alpha-relay-state.json` | JSON with `state`, `terminated_at`, `former_public_ip` |

The Rust CLI, the Android client and the infra provisioning scripts are already
correct. **The gap is entirely in the JS clients, the four helper shell scripts, and
the operator-facing markdown.**

### Exact replacements for each STALE-CRITICAL site

**`headless/main.js:8-11`** -- replace the literal array with an empty default plus an
override, matching `cli/src/bootstrap.rs:26`:

```js
// Bootstrap addresses are supplied at runtime. There is no compiled-in relay.
// Sources, in priority order: ?bootstrap= query param, localStorage, empty.
const DEFAULT_BOOTSTRAP_NODES = [];
```

**`ui/app.js:5-8`** -- same change. This file already persists user-entered bootstrap
addresses under `localStorage["scm.desktop.bootstrap.v1"]` (`ui/app.js:9`), so the
override path exists; only the seed array needs to go:

```js
const DEFAULT_BOOTSTRAP = [];
```

**`scripts/test_all_bootstrap_nodes.sh:18-24`** -- the env override
(`SCMESSENGER_BOOTSTRAP_NODES`, line 14) already works. Make it required rather than
optional: replace the hardcoded `BOOTSTRAP_NODES=(...)` fallback with an error exit
that names the env var, or read the list from the CLI's own config
(`scm config get bootstrap_nodes`).

**`scripts/live-smoke.sh:19`**, **`scripts/preflight.sh:124`**,
**`scripts/test_gcp_node.sh:9`**, **`docs/SCRIPT_IMPLEMENTATION_PLAN.md:137`** --
drop the literal from the parameter expansion and skip the check when unset:

```sh
GCP_RELAY_IP="${GCP_RELAY_IP:-}"
[ -n "$GCP_RELAY_IP" ] || { echo "[SKIP] GCP relay check: GCP_RELAY_IP unset"; }
```

`scripts/preflight.sh:124` already emits `check_warn` on failure, so skipping is
strictly better than probing a dead literal and warning about it.

**`docs/CURRENT_STATE.md:1147,1151`** -- replace the literal with `$SC_RELAY_HOST`
and add a line pointing at the state-of-record file:

```bash
nc -zv "$SC_RELAY_HOST" 9001   # SC_RELAY_HOST: see infra/ec2/alpha-relay-state.json
```

**`docker-compose.yml:17`** -- change the example to a documentation-range address:
`# Example: SC_BOOTSTRAP_NODES=/ip4/192.0.2.10/tcp/9001/p2p/12D3KooW...`

**Every operator-facing checklist and runbook** (`HANDOFF/todo/*`,
`HANDOFF/review/V040_S4/S5_*`, `HANDOFF/plans/*`, `HANDOFF/ALPHA_TEST_*`) -- replace
the literal with a placeholder that cannot be pasted by accident and a pointer to the
single source of truth:

```
ssh -i <key> ubuntu@$SC_RELAY_HOST      # resolve via infra/ec2/alpha-relay-state.json
curl -fsS "http://$SC_RELAY_HOST:9876/health"
```

Add a step 0 to each runbook: "Resolve `$SC_RELAY_HOST` from
`infra/ec2/alpha-relay-state.json` and confirm `state == RUNNING`. If the file says
TERMINATED, stop -- there is no relay."

**`HANDOFF/todo/V1_INSTALL_ARTIFACT_FOR_ALPHA_TESTERS.md:63`** -- delete the claim
outright. The described code does not exist; the correct statement is that Android
sources bootstrap addresses from the ledger via `getPreferredRelays()`.

### Canonical single source of truth

**Short term (do this first, costs nothing):** `infra/ec2/alpha-relay-state.json`
already is the state of record and already has the right shape. Promote it formally:

- Add `"current_public_ip"` and `"last_verified_utc"` alongside the existing
  `state` / `former_public_ip` fields.
- Have `infra/ec2/launch-alpha-relay.sh` write both fields at launch. The script
  already retrieves the address from the AWS API, so this is a write, not a lookup.
- Make every doc reference the **file path**, never the value. A doc that says
  "see `infra/ec2/alpha-relay-state.json`" can never go stale; a doc that says
  `100.56.248.69` goes stale the moment the instance restarts.

**Correct long term: Elastic IP plus a DNS name.** The root cause named by the
operator is precisely that no Elastic IP was attached, so the address changes on
every stop/start. An Elastic IP alone fixes the drift; an Elastic IP plus a DNS A
record additionally lets docs reference a stable hostname that never needs editing.

The repo is already primed for this and the pieces are inconsistent:

- `docs/TEST_PLAN_KMP_DESKTOP.md:61` references `wss://relay.scmsg.org`.
- `HANDOFF/done/P1_ANDROID_013_Bootstrap_Reliability.md:15` references
  `/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooW...`.
- `HANDOFF/done/P0_ANDROID_007_NETWORK_DIAGNOSTICS.md:107` references
  `relay.scmessenger.net`.
- `.claude/plans/snazzy-spinning-dongarra.md:10` describes
  `bootstrap.scmessenger.net` and the two GCP/Cloudflare IPs as
  "hardcoded/hallucinated addresses".

[WARNING] I did not resolve any of these hostnames and make no claim about whether
any are registered or point anywhere. Before adopting one, someone must confirm the
zone is actually controlled by the project. Do not assume a hostname is real because
it appears in a doc -- that is the same failure mode as the IP.

Three constraints from the repo bear on the DNS choice and must be weighed:

1. `HANDOFF/done/ESC_ANDROID_DNS_RESOLVER_FIX.md:39` records that Android nodes were
   at one point restricted to IP-form multiaddrs and could not dial `/dns4/` forms.
   Verify current Android behavior before making a DNS name the only path.
2. `HANDOFF/review/LEDGER_SEEDING_ADVERSARIAL_REVIEW_2026-07-25.md:13` records finding
   F3/NEW-1 as **NOT CLOSED**: "DNS multiaddrs bypass every IP check ... IP-form
   addresses are filtered; `/dns4/...` is not." Introducing `/dns4/` into the seed
   ledger path without closing that finding widens an SSRF surface. Route the
   hostname through `core/src/transport/addr_filter.rs` first.
3. `HANDOFF/review/V040_S4_DELIVERY_PROOF_RUNBOOK.md:46` states the project
   philosophy as "no third-party network dependencies", and the same line notes the
   AWS node is "a test rendezvous, not a production relay dependency". The DNS name
   should therefore be scoped as **test infrastructure**, not shipped as a compiled
   default. `DEFAULT_BOOTSTRAP_NODES` must stay empty.

Recommended end state: Elastic IP attached; `infra/ec2/alpha-relay-state.json` holds
the current address and a `dns_name` field; a project-controlled A record points at
the Elastic IP; all docs cite the hostname or the file path; no client ships any
address compiled in.

## Proposed policy blurb

Suitable for `.claude/rules/` or `CONTRIBUTING.md`.

```
## Hardcoded IP addresses are ephemeral

Never trust an IP address written in a document. Any routable IPv4 or IPv6
literal in this repository is a snapshot of a moment that has probably passed.
Cloud instances without an Elastic IP change address on every stop/start, and a
doc that recorded the old address will not tell you it is wrong -- it will tell
you the relay is healthy while nothing is listening.

Rules:

1. Do not hardcode a routable address in code, scripts, CI, or compose files.
   Read it from config: SC_BOOTSTRAP_NODES for the CLI and containers,
   `scm config bootstrap add` for persisted user config, the sled-backed ledger
   for Android and iOS. DEFAULT_BOOTSTRAP_NODES stays empty.
2. Docs reference the source of truth by path, never by value. Write
   "see infra/ec2/alpha-relay-state.json", not the address itself.
3. Examples use documentation ranges only: 192.0.2.0/24, 198.51.100.0/24,
   203.0.113.0/24 (RFC 5737), or 1.2.3.4 / 5.6.7.8 as already used in the Rust
   tests. Never a real routable address.
4. Historical records -- docs/historical/, HANDOFF/done/, dated audits, captured
   logs -- keep their addresses. They are evidence. Do not rewrite them. Add the
   banner below instead.
5. Before any distributed test, resolve the current address from the source of
   truth and probe it. A checklist that says the relay is up is not evidence
   that it is up. Record the probe output with a UTC timestamp.
6. Never write "verified", "healthy", or "LIVE" about an endpoint you did not
   just probe yourself, and always attach the timestamp and the command used.

Banner for historical documents containing addresses:

    [WARNING] Addresses in this document are a historical record from <date>.
    They are ephemeral and are almost certainly no longer valid. Do not dial
    them. Resolve the current endpoint from infra/ec2/alpha-relay-state.json.
```

## Suggested order of work

1. Correct the two contradictions above -- the terminated-instance runbooks and the
   false code reference in `V1_INSTALL_ARTIFACT_FOR_ALPHA_TESTERS.md:63`. These are
   the ones that burn operator time.
2. De-hardcode the 13 executable sites. Small, mechanical, no behavior change once
   the env vars are set.
3. Add `current_public_ip` / `last_verified_utc` to
   `infra/ec2/alpha-relay-state.json` and have `launch-alpha-relay.sh` write them.
4. Placeholder-ise the 57 runbook and checklist sites plus the 13 STALE-DOC sites.
5. Add the ephemerality banner to HISTORICAL files. Do not touch their addresses.
6. Attach an Elastic IP and decide the DNS question, gating on the `/dns4/` filter
   finding (NEW-1) and current Android resolver behavior.
7. Land the policy blurb so this does not recur.

## Attestation

Every `file:line` in this report was produced by a scripted regex sweep over
git-tracked files and spot-checked by reading the surrounding source. No finding was
inferred, extrapolated, or invented. No address was probed for liveness and no claim
of reachability, health, or currency is made anywhere in this document. No tracked
file was modified by this audit; this report is the only file created.
