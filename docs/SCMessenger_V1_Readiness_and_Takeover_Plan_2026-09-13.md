### SCMessenger V1.0.0 — Readiness Audit and Takeover Plan

**Audit date:** 13 September 2026 (UTC), beginning 17:21:02Z. **Scope:** application-source/GitHub audit and bounded Linux baseline, followed by an explicitly authorized docs-only publication attempt (remote push blocked). This is not application implementation, a penetration test, or an independent cryptographic audit.

**Audited repository:** [Sovereign-Communication/SCMessenger](https://github.com/Sovereign-Communication/SCMessenger). **Branch:** `main`. **Exact SHA:** `b5a70bd54d14f93b63463dd39df556ae5fabbc4c` — PR #282, committed 2026-09-13T17:14:07Z. The repository was shallow-cloned with `--depth=50`; history outside that window was consulted selectively through GitHub. No application code, existing PR, issue, release, runner configuration, cloud node, or user device was changed. The user subsequently authorized saving this report through a new docs-only branch and PR. A separate local worktree/branch was created, but GitHub rejected the push preflight with HTTP 403 (permission denied to the connected account). **Remote publication is blocked: no remote branch or PR was created.** The report is preserved locally for publication once access is corrected. No remote workflow was manually dispatched and no external runner or cloud resource was provisioned.

**Priority update incorporated:** Linux is the first stable-backbone target, with **Linux ↔ Android parity as the first paired product gate**. Cloud, desktop, and Raspberry Pi Linux are distinct deployment profiles; x86-64 Linux results do not establish ARM or radio-device readiness. The six requested targets remain in scope.

### 1. Executive verdict

**Not ready to declare V1.0.0 complete. There is a substantial working implementation, but neither six-platform release completeness nor backbone-grade durability is established.** The repository itself calls the product pre-release and unsuitable for users relying on privacy against a capable adversary. Its Cargo version is `0.4.0`, not `1.0.0`.

The strongest takeover point is **the audited merged `main`, not an older open integration PR**. PR #281 already incorporated the three-node candidate and superseded #272/#279; #282 subsequently added Android stability fixes. Several old queue entries describe problems already addressed on main. Reimplementing those would waste effort and risk regressions.

The Linux-first priority is well justified, but stable location is not the same as reliable custody. A stationary node still faces disk exhaustion, process termination, power loss, NAT/public-address changes, stale peer state, hostile traffic, and upgrades. Source inspection found a particularly important remaining gap: **the custody store has in-memory fallbacks on persistent-store open failure**, separately from the merged fail-loud `IronCore` fix. Control-plane exposure and shutdown/recovery behavior also need focused regression coverage.

**Honest capability boundary:** I can own Linux source development, local automated tests and network experiments, Linux packaging, Android build/test tooling and emulator integration, cross-platform source changes, and evidence-based PR preparation. I cannot promise perfection, certify the cryptography, independently approve my own security-sensitive changes, exercise physical handset radios through an emulator, or sign/publish native releases without the relevant owner-controlled credentials and native validation. This report proposes work; it does not mark those capabilities as already executed.

### 2. What actually exists

The architecture is shared Rust plus platform adapters, **not six independent finished native apps**:

- **Core:** `core/src/iron_core.rs` is the composition root; `core/src/store/` owns identity/message/history/contact/ledger state; `core/src/transport/swarm.rs` wires libp2p; routing policy lives under `core/src/routing/`. `core/src/mobile_bridge.rs` and UniFFI expose platform integration. This follows [the current architecture boundary document](https://github.com/Sovereign-Communication/SCMessenger/blob/b5a70bd54d14f93b63463dd39df556ae5fabbc4c/docs/ARCHITECTURE_SCOPE_V040.md).
- **Android:** a real Kotlin/Compose application, foreground-service and platform-transport code, generated UniFFI bindings, unit tests, debug APK and signed release Gradle paths exist. This is substantially more than a scaffold.
- **iOS:** a real Swift application and Xcode project, generated Swift bindings/XCFramework build, platform adapters and XCTest target exist. Historical simulator CI is meaningful evidence, but is not physical iPhone transport, backgrounding, or distribution proof.
- **Linux/Windows/macOS:** the shared Rust **CLI node** is the established desktop deliverable. It includes a local WebSocket/HTTP bridge and browser UI assets. A separate Rust `desktop_bridge` crate exists, but the Compose JVM desktop entry point only prints a greeting: `shared/src/desktopMain/kotlin/com/scmessenger/shared/Main.kt:3–6`; the shared UI file only defines `greet()`. Do not label that a completed native desktop messenger. `Platform.kt` even returns the constant `"Linux"`.
- **Browser:** `wasm/` is a real workspace member and browser/daemon bridge implementation; `ui/` has browser assets. These are relevant to desktop UX, but compiling a WASM crate does not prove that a downloaded CLI bundles and serves a complete usable UI. `cli/src/server.rs:266–270` serves relative `ui/` and `wasm/` directories, so packaging and working-directory behavior matter.
- **Nodes and custody:** the intended doctrine is that every node can relay sealed messages; the Linux backbone must not become a new trusted central messaging server. Cloud is a deployment profile, not a separate anonymous relay product. The CLI retains a command named `relay`; identifiers do not redefine the architecture contract.

**Build inventory measured:** `cargo metadata --locked --no-deps --format-version 1` reports five workspace/default members, all version `0.4.0`, and 63 Cargo targets. The root manifest simultaneously lists and excludes `wasm`; actual metadata includes it. The README instruction to run the workspace tests is therefore broader than a core-only test.

**Transport claim to reconcile:** native swarm construction at `core/src/transport/swarm.rs:3356–3375` wires TCP, WebSocket and circuit connectivity. It does not attach a QUIC transport there, despite a QUIC listener attempt at lines 3479–3484 and QUIC-enabled Cargo dependencies. Treat QUIC as an unfulfilled/unverified runtime claim for this builder, not a completed path merely because types, features or state-machine tests exist.

### 3. Current GitHub reality, not stale handoff status

#### Merged work and takeover baseline

| Item | Verified disposition | Takeover consequence |
|---|---|---|
| [#282](https://github.com/Sovereign-Communication/SCMessenger/pull/282) | Merged 13 September at audited SHA; Android logging concurrency/error handling, motion-event dirty checks, subnet-probe fallback gating, plus governance tooling | Do not propose these stability fixes as new implementation; add regression/soak evidence around them |
| [#281](https://github.com/Sovereign-Communication/SCMessenger/pull/281) | Merged 12 September at `956ec3713…`; unified three-node parity candidate | The candidate is already on main; it is not a pending merge |
| [#272](https://github.com/Sovereign-Communication/SCMessenger/pull/272) | Closed without merge; [closing comment](https://github.com/Sovereign-Communication/SCMessenger/pull/272#issuecomment-5645770318) explicitly says superseded by #281 | Preserve the resulting address-admission/local-ordering work on main rather than revive the branch |
| #279 | Closed without merge; unified transport work is represented by #281 | No independent takeover branch to adopt blindly |
| [#221](https://github.com/Sovereign-Communication/SCMessenger/pull/221) | Merged 24 August; authenticated ingress/root-key changes, suite `0x03`, ratchet bypass removal | Preserve and exercise forgery/downgrade regression tests; do not resurrect the bypass |
| [#222](https://github.com/Sovereign-Communication/SCMessenger/pull/222) | Merged 24 August; persistent `IronCore` storage fails loudly | Do not repeat the old core RAM-fallback fix; investigate the distinct custody-store fallback |
| [#224](https://github.com/Sovereign-Communication/SCMessenger/pull/224) | Historical closed checkpoint about then-blocked #221/#222 | Its failure status is not today's main status |
| #262/#263/#266/#267/#268/#269/#270/#271/#275 | Recent history contains peer-ledger unification, connection-established routing feed, bounded seed dialing, verified-peer DHT gating/timestamp clamp, stable routing hints, address-admission fixes, restored diagnostics test and cargo-deny installation pin | Reuse these foundations; do not restart the old T1/T2/T4/T8 backlog from its unamended descriptions |

Recent catch-up sampled the latest 25 local commits, latest 20 closed PRs, latest 50 main workflow runs, latest five runs of selected release/desktop workflows, all returned open PRs/issues, and all returned releases/tags. This is a **bounded recent-history review**, not an exhaustive review of every historical commit or CI run. Relevant API pagination was recorded; the historical samples have further pages. The open-PR query had no next page; the open-issues response contained fewer than its requested 100 entries.

#### Open/competing work

All **22 open PRs** at discovery are accounted for below; an open PR is not proof that its advertised fix is absent from main.

| PRs | Status / overlap and recommended disposition |
|---|---|
| #216, #220 | Competing Android reachability/passphrase branches. #216 is draft and has an explicit adversarial BLOCK; #220 splits out reachability and retracts unsafe migration. Main now passes its wiring scanner and calls `SecurityUtils` from passphrase resolution. Compare semantic deltas and recovery behavior before retaining either; do not merge both |
| #215 | Draft routing-feed PR with an explicit adversarial BLOCK. Main now has `routing_peer_seen` calls on connection establishment and related regressions. Review remaining unique intent only; do not reintroduce its old unauthenticated/hint-mismatch path |
| #227 | Draft Android degraded-storage follow-up. Main already checks degradation in `mobile_bridge.rs` and surfaces state in Android. Establish any remaining unique test/UI delta before adoption |
| #228 | Draft ignored-security-parameter/forgery gate. Its prerequisite #221 has merged, so the old prerequisite is stale; remaining policy changes and exclusions still require review. Do not assume draft CI hardening is installed |
| #209 | Identity-ID unification branch overlaps the later canonical identity/history work. Inventory residual diffs and cross-platform compatibility; no blanket merge |
| #178, #207, #208 | Apple work/continuity/parity branches; #178 has an unfilled PR template, not adequate acceptance evidence. Request/prepare a scoped residual-change inventory rather than treating these as validated iOS/macOS completion |
| #156 | Proposes making Docker integration non-blocking. Newer main Docker suite has succeeded; do not adopt an old waiver as a substitute for diagnosing today's failures |
| #218 | Draft self-circuit guard work; inspect its residual delta against merged address/self-dial admission before adoption. The audit socket failures make guard-preserving regression coverage especially relevant |
| #170 | Orchestration tooling; not the Linux–Android product critical path |
| #103, #106, #107, #108, #141, #210, #211, #212, #213, #214 | Dependency/Actions updates. Batch triage by compatibility and security need, then verify scoped PRs; not an automatic bulk merge |

The open issues response contained **one non-PR issue**, [#155](https://github.com/Sovereign-Communication/SCMessenger/issues/155), about Docker/UniFFI binding generation. Recent Android Docker/unit successes weaken the premise that the old failure is still universal; close or update it only after matching the exact build path and artifact, not from an issue title.

#### CI and releases: evidence has different strength

- Previous merged main `956ec3713…` has successful [CI](https://github.com/Sovereign-Communication/SCMessenger/actions/runs/34692698327), [Mobile](https://github.com/Sovereign-Communication/SCMessenger/actions/runs/34692698340), [iOS Build & Test](https://github.com/Sovereign-Communication/SCMessenger/actions/runs/34692698278), and [Docker Integration Suite](https://github.com/Sovereign-Communication/SCMessenger/actions/runs/34692698268) runs on 12 September. Job-level review confirmed Linux/macOS/Windows Rust tests, Windows CLI artifact, FFI, Android JVM tests/APK, and iOS builds rather than merely trusting workflow titles.
- The iOS simulator log at that previous main records **53 tests, zero failures**. This is historical CI, not local/native execution in this audit. The historical Android JVM job completed successfully but explicitly skipped three listed tests, including two UniFFI integration tests; a green job is not proof of those skipped integrations.
- PR #282 head `d35d3883e…` had successful Rust three-OS, Android APK/JVM and Apple jobs before merge. That is PR evidence, not automatically proof of its merged SHA. Audited-main final snapshot is recorded with the baseline results below.
- [Security Scan 34735024303](https://github.com/Sovereign-Communication/SCMessenger/actions/runs/34735024303) failed on previous main: dependency audit and license jobs succeeded, **Gitleaks reported one leak and exited 1**. This was not a billing/licensing failure. The candidate secret was deliberately not printed or copied into this report. Its validity, location/history disposition and revocation status remain unverified; responsible private triage is required.
- [Docker run 34686880221](https://github.com/Sovereign-Communication/SCMessenger/actions/runs/34686880221) failed earlier at image/cache export: `error writing layer blob: not_found`. That is the observed failure, not a reproduced Rust test defect. Later Docker success must be kept distinct from that historical infrastructure error.
- The latest observed release workflow, [32817839477](https://github.com/Sovereign-Communication/SCMessenger/actions/runs/32817839477) at tag `v0.4.0-rc.1`, failed at Android `:app:packageRelease`: **no key with the configured alias was found in the supplied keystore**. The earlier “signing not configured” line belongs to an earlier phase; the final signed phase did load env-driven signing. Windows, Linux x86-64, both macOS CLI builds and WASM succeeded, but GitHub Release creation was skipped. Do not diagnose this as “signing code does not exist.”
- Four GitHub Release objects were returned: `v0.2.1`, `v0.1.9`, `v0.1.1`, `v0.1.0`, with desktop CLI binaries; **no APK asset and no V1 release**. `v0.2.1` is the highest released version, while `v0.1.9` has a slightly later publication timestamp on 19 March. Tags also include `v0.3.5` and `v0.4.0-rc.1`; a tag is not a downloadable Release object. No Linux ARM release asset was listed.
- Desktop CI's recent successful run [32624303152](https://github.com/Sovereign-Communication/SCMessenger/actions/runs/32624303152) is on the identity-unification PR branch, not current main; it builds bindings and compiles JVM sources. It does not exercise a desktop messaging UI. The separate dispatch-only cross-platform-test workflow returned **zero runs**; its YAML is not test evidence.

### 4. Baseline execution and readiness matrix

**Observed result: ordinary workspace tests passed, but the separately enabled real-socket baseline failed before message delivery. Android native checks were not executable with the installed tooling.** These are Linux VM observations, advisory under the repository's REMOTE SANDBOX rules; they do not close the operator's Windows/physical-Pixel acceptance gates.

#### Environment and exact execution

Ubuntu 24.04, x86-64 Linux, two vCPUs, approximately 7.8 GiB RAM and no swap; initial free space approximately 35 GB on root and 51 GB on the home filesystem. Python 3.12.3, Node 22.23.2 and Docker 29.1.3 were present. Docker images were not built/run in this audit. Rust and Java were initially absent. The audit installed Rust stable using rustup, plus `libdbus-1-dev` and `protobuf-compiler`; resulting versions were rustc 1.98.1, cargo 1.98.1, libprotoc 3.21.12 and D-Bus 1.14.10. Only `x86_64-unknown-linux-gnu` was installed as a Rust target. No Java/Android SDK/NDK/AVD or native Apple/Windows toolchain was installed.

The full workspace ran **once**, from the audited checkout, with the following environment and command. Optimization/debug overrides bounded this cold build; this is not release-mode or performance evidence.

```bash
export PATH=/tmp/.cargo/bin:$PATH
export TMPDIR="$PWD/tmp/audit"
export CARGO_BUILD_JOBS=2
export CARGO_PROFILE_DEV_OPT_LEVEL=0 CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_TEST_OPT_LEVEL=0 CARGO_PROFILE_TEST_DEBUG=0
timeout 1800 cargo test --locked --workspace --features test-utils \
  --no-fail-fast -- --test-threads=1
```

`/tmp/.cargo` was the environment's preconfigured Cargo home; audit test scratch state was isolated under repository-local `tmp/audit`. The only test data used were synthetic/generated audit fixtures. No production node, message store, signing key or device was used.

| Command / measured check | Exit and measured outcome | Elapsed / qualification |
|---|---|---|
| Workspace command above | **0; 1,905 passed, 0 failed, 25 ignored** across 59 result summaries, including doc-tests | **756 seconds**, including cold dependency download/build; Cargo reported 11m25s build time |
| `cargo fmt --all -- --check` | 0; formatting check passed | 2 seconds; no runtime guarantee |
| `python3 scripts/test_check_wiring.py` | 0; 7 tests passed | 0.72 seconds |
| `python3 scripts/check_wiring.py` | 0; no findings | 0.69 seconds; scanner coverage only |
| `python3 scripts/test_verify_apk_native_libs.py` | 0; 7 tests passed | 0.05 seconds; fixture tests of the checker, **not inspection of an actual APK** |
| `bash android/gradlew --version` | **1; Java/JAVA_HOME unavailable** | 0.00 seconds as measured; Android Gradle build, JVM tests, lint and instrumentation were **not run locally** |
| `target/debug/scmessenger-cli --version` / `--help` | 0 / 0 | 0.02 / 0.02 seconds |
| `target/debug/scmessenger-cli init --name AuditLocal` | 0 | 0.04 seconds; isolated fresh config/data |
| `target/debug/scmessenger-cli identity`, twice in separate processes | 0 / 0; displayed identity fields identical after removing timestamped log lines | 0.04 / 0.03 seconds; verifies reload, not a running-node restart or crash recovery |
| Existing ignored two-node address-reflection binary, command below | **101; 0 passed, 1 failed, 0 ignored, 4 filtered out** | 1.02 seconds; dial rejected before reflection |
| Existing ignored offline-recipient custody binary, command below | **101; 0 passed, 1 failed, 0 ignored** | 0.03 seconds wall clock; harness reports 0.02 seconds; failed before third node/delivery |

The socket checks executed the **same test executables produced by the successful workspace build**, avoiding a second full suite or different feature-unification build. Each had a 180-second cap and isolated `XDG_DATA_HOME`/`TMPDIR` under `tmp/audit`. Exact local repros:

```bash
timeout 180 target/debug/deps/integration_nat_reflection-a26e382eab31bedf \
  test_two_node_address_reflection --exact --ignored --test-threads=1
# exit 101; core/tests/integration_nat_reflection.rs:75:10
# Failed to dial: skipped: address is our own listener/external/interface addr -- self-dial

timeout 180 target/debug/deps/integration_relay_custody-fc2776125cb067dd \
  --ignored --test-threads=1
# exit 101; core/tests/integration_relay_custody.rs:119:10
# sender failed to dial relay: skipped: address is our own listener/external/interface addr -- self-dial
```

Executable hashes are build-specific. A fresh-checkout equivalent is `cargo test --locked -p scmessenger-core --features test-utils --test integration_nat_reflection test_two_node_address_reflection -- --exact --ignored --test-threads=1`, and the corresponding `--test integration_relay_custody -- --ignored --test-threads=1` command. Those equivalent Cargo invocations were **not separately rerun**.

**Failure interpretation:** the immediate observed mechanism is the self-address admission guard (`core/src/transport/swarm.rs:725–744,806–807`), not denied Linux socket permissions. The fixtures create local swarms with default listener arguments and then dial their observed listener addresses. A same-host/default-listener collision is a hypothesis to test with explicit distinct sockets or isolated namespaces; this audit did not establish whether the needed correction is fixture configuration, transport behavior, or both. Do not disable the anti-self-dial guard merely to green these tests. The custody test never reached its recipient reconnect or delivery assertion, and even success there would assert a transport envelope, not endpoint decrypt/history/receipt. **No successful real multi-node message path was demonstrated here.**

The 25 ignored workspace tests remain the recorded first-run count. Two of those were subsequently enabled and failed; **23 remained unexecuted**. Do not count their first-run ignored entries as additional successful coverage. These include other socket/NAT/registration/ledger cases and a doc-test. No long-duration soak, real WAN, network fault-injection, process-supervisor lifecycle, Pi power-loss or published-artifact installation test ran.

Selected passing coverage within the workspace: core library 1,444 tests; CLI library 82 and CLI integration target 19; core roundtrip 9, persistent ratchet 6, retry lifecycle 6, fail-loud storage 5, backup 15, backup continuity 2, property tests 29; explicit legacy/hybrid forgery regression targets passed. `desktop_bridge` library had 16 passes plus 6 XDG integration passes, mobile library 4, and WASM crate 43 **host** tests. Names can overstate integration: the CLI five-node mesh acceptance target uses fixtures, local-mesh tests use mock platforms, and the inspected IronCore roundtrip cases deliver in-process. They are useful but not radio, WAN or multiprocess acceptance.

#### Six-target readiness matrix

Legend: **Observed** means executed on this Linux VM at the audited SHA; **historical CI** means a retrieved run on the stated earlier/PR SHA; **inspection** means source/config only; **unknown** means no adequate execution evidence. None is an overall V1 approval.

| Target | Observed on this VM | Historical CI / inspection | Readiness and missing proof |
|---|---|---|---|
| **Core** | Workspace build/tests passed; two separately enabled socket tests failed | Rust three-OS tests green on previous main and PR #282 head; shared storage/crypto/transport implementation inspected | Strong unit baseline, **network acceptance blocked**; custody fault/restart, production bundle-ingress validation and independent security review remain |
| **Android** | Wiring scanner and checker unit tests passed; Gradle preflight failed without Java; **no APK, JVM/lint/instrumented run or emulator boot here** | Previous-main and PR-head Android build/JVM CI green; three JVM tests explicitly skipped in reviewed historical log | Substantial app, not locally qualified; set up emulator, prove Linux pairing, migration/Keystore handling and physical radio/lifecycle behavior |
| **iOS** | **No execution** | Previous-main simulator log: 53 tests, zero failures; PR-head Apple jobs green; Swift app/Xcode/FFI inspected | Simulator evidence only; physical background/radio/IP parity, signing, distribution and current-candidate testing remain |
| **Windows** | **No execution** | Previous-main/PR-head Windows Rust jobs and CLI artifact green; release workflow built Windows CLI | CLI build evidence, not user-machine execution; packaged UI/native desktop scope and Windows service/device/install behavior remain |
| **macOS / OSX** | **No execution** | Previous-main macOS Rust and PR-head Apple jobs green; release workflow built both Mac CLI architectures | CLI/source baseline, not completed GUI or signed/notarized release/install/transport proof |
| **Linux** | Full x86-64 workspace plus CLI init/reload passed; real-socket checks failed before reflection/delivery | Linux CI and amd64 release build evidence; CLI/container/UI/service documentation inspected | First hardening target, **not backbone-qualified**. No supervised node soak, disk-full/kill recovery, actual Linux–Android delivery, aarch64 build/native Pi test, or finished Compose GUI proof |

#### Audited-main CI snapshot

At **2026-09-13T17:40:52Z**, GitHub returned nine runs for the exact audited SHA, with no next page: CI [34770973707](https://github.com/Sovereign-Communication/SCMessenger/actions/runs/34770973707), Mobile [34770973775](https://github.com/Sovereign-Communication/SCMessenger/actions/runs/34770973775), iOS Build & Test [34770973826](https://github.com/Sovereign-Communication/SCMessenger/actions/runs/34770973826), and Docker Integration Suite [34770973760](https://github.com/Sovereign-Communication/SCMessenger/actions/runs/34770973760) were **in progress**, not green. Repository Hygiene 34770973772, Lint 34770973868, Docker Publish 34770973747, Cross 34770973721 and Push on main 34770973338 had completed successfully. This is the final audit snapshot, not a prediction about their eventual outcomes. Existing CI was read only; no workflow-dispatch request was made.

Evidence is retained outside tracked repository files under the audit evidence directory: workspace log/counts/exit, static results, CLI smoke/identity comparison, socket logs/results, and selected GitHub metadata and safely filtered failing-job logs. Raw logs and generated identities are deliberately omitted from the user-facing deliverable and docs PR.

### 5. Highest-risk blockers and evidence-driven Linux hardening

| Priority | Finding and evidence | Required closure |
|---|---|---|
| High — reproduced baseline blocker | Real-socket tests fail at `integration_nat_reflection.rs:75` and `integration_relay_custody.rs:119`, exit 101, on the live self-address rejection path | First separate fixture/default-listener collision from a production transport defect using distinct addresses/ports or namespaces. Preserve self-dial protection and obtain non-vacuous reflection and endpoint message evidence |
| P0 candidate — durability | `core/src/store/relay_custody.rs:405–432,436–470` silently substitutes `MemoryStorage` plus `NoopStoragePressureProbe` when persistent stores fail to open. `swarm.rs:3623–3644` constructs and installs this custody store on the live node path. This is source-confirmed; actual loss under failure has not yet been reproduced | Fault-inject unavailable/locked/corrupt storage; prohibit successful durable custody acceptance in degraded mode; prove acknowledged ciphertext/registration/audit survival across restart and abrupt termination. Preserve legitimate explicitly ephemeral test use |
| P0 candidate — control surface | `cli/src/api.rs:1599–1684` accepts an arbitrary bind address, permits any CORS origin, and registers send/history/contacts/shutdown endpoints without a client-authentication layer in the assembled router. `main.rs:195–199` describes `--http-bind` as a health server, but the live start path passes it to the control API. Local WebSocket route `server.rs:249–264` also lacks an Origin/auth check | Separate public health from privileged control, refuse unsafe remote binding by default, authenticate control access, enforce Origin policy and limits. Prove unauthorized send/history/reset/shutdown rejection. Never expose the current control API publicly for testing |
| Release blocker — secret hygiene | Current reviewed Security Scan reports one leak | Privately establish true/false positive and exposure history; rotate/revoke if real before any history cleanup; add precise prevention. No broad scanner suppression |
| High — unattended recovery | CLI `start` and `relay` loops handle `ctrl_c` (`main.rs:2503,3945`) but no SIGTERM handler was found in CLI source. Config writes are direct replacement (`config.rs:180–184`); headless identity recovery can rotate after decode failure (`main.rs:126–178`). A watchdog already exits if the swarm dies (`main.rs:2480–2493`) | Linux SIGTERM/systemd-stop/restart and kill/recovery tests; atomic config/identity updates, explicit corrupt-identity recovery, no silent identity churn. Prove service readiness means durable storage and live swarm, not just `/health` returning a constant |
| High — device storage/identity continuity | Main now encrypts the Android backup passphrase, but `SecurityUtils.kt:45–71` quarantines then deletes/recreates preferences on error; `data_extraction_rules.xml:30–42` does not exclude the hardware-key-bound secure preferences. `MeshRepository.kt:4042–4087` restores with that passphrase, then legacy empty fallback, then returns failure | Reproduce migration, failed commit, Keystore invalidation and D2D restore in Android tests; prove recoverable failure without silently replacing identity. Quarantine is not cryptographic recovery of a lost hardware key. This is a source-backed risk, not an observed handset failure here |
| High — delivery evidence | `SHIP_PLAN.md:29–35` requires receiver decrypt + durable history + receipt, not a transport ACK. Latest candidate/device notes are not a same-SHA, released-artifact, all-cell acceptance result | Score same-SHA Linux–Android direct, node-assisted/offline custody, reconnect, duplicate/reorder and unavailable-first-transport scenarios on receiver evidence. Repeat on real hardware for radio/cellular cells |
| High — crypto/privacy closure | README explicitly disclaims independent audit; #221 follow-ups include bundle-ingress verification/session recovery. `verify_bundle` exists, but the inspected core search found definition/tests/re-export rather than a production ingestion call | Trace every shipped bundle/QR/import ingress; pin suite downgrade and sender-auth rejection, replay and ratchet restart tests; preserve compatibility policy. Independent expert review before production privacy assurances |
| High — Linux ARM/backbone qualification | Current Cross workflow covers Android ABIs, Apple and WASM, not Linux ARM. Published Linux binaries are amd64. Generic ARM mentions in `docs/CLI_LINUX.md` are explicitly “Needs Revalidation” | Build/package/test aarch64 Linux and validate on representative Raspberry Pi hardware; decide whether ARMv7 is supported. Benchmark rather than repeat unverified minimum-RAM/throughput claims |
| High — desktop scope gap | Compose entry point is a console greeting; bridge and AppImage script are not a finished app. AppImage script hardcodes x86-64 and calls a non-present `shared/gradlew` fallback (`scripts/build_appimage.sh:29,44–49`) | Distinguish CLI + packaged local browser UI from native GUI acceptance. Keep the repo's intended desktop scope visible; do not quietly mark a greeting as “desktop complete” or silently waive the native GUI |
| Release blocker — distribution | No current public APK; historical release failed on signing alias. Native Apple/Windows/Linux release-install verification and provenance remain incomplete | Validate signing material privately; produce reviewed artifacts from one approved SHA, verify checksums/provenance/install/upgrade, and exercise the published artifacts. Owner controls signing/store publication |

**Important positives to preserve:** authenticated envelope ingress (`iron_core.rs:3547–3607`), core fail-loud persistent storage, Android degradation UI, a shared ledger, verified-address admission, seed-dial backoff, deterministic routing feeds, custody flush calls/pressure machinery, and the swarm-death watchdog already exist. Hardening should extend their real call paths rather than create competing stores, schedulers, identity formats or transport authorities.

### 6. Acceptance contract: existing requirements versus proposed V1 gates

**Repository-established:** `SHIP_PLAN.md` supersedes the old execution queues until the current alpha is shipped. D1–D7 require green main, downloadable signed APK, accurate install documentation, actual endpoint message/receipt proof, trunk consolidation, working fallback, and offline proximity. Its later section 6.4 explicitly allows functional demos using a throwaway-signed **release-configured** APK before production signing is resolved; the final published-artifact check still needs the production release. Do not unnecessarily block Linux–Android functional work on the production keystore.

The long-horizon `HANDOFF/V1_0_0_EXECUTION_PLAN.md` remains a scope reference, but section 0A supersedes its frozen July findings. It records Phase 1 exit with waivers, CI becoming available, Wi-Fi Aware's orphan concern closed, ratchet wiring completed and the bypass subsequently removed. The current user's Linux–Android-first instruction should be recorded as the next approved priority amendment, not used to erase remaining platform obligations.

**Proposed additional V1 acceptance gates — not already satisfied, and not falsely attributed to the repository:**

1. **Exact release identity:** one candidate SHA, locked dependency graph and recorded toolchains; reproducible packaging/provenance for each supported CPU/OS. Test both the core and each shipped frontend; generated FFI snapshots match the compiled library.
2. **Durable Linux backbone:** successful custody means persisted ciphertext and metadata before acceptance. Restart, SIGTERM, abrupt kill, lock contention, disk-full/read-only storage, migration and recovery preserve identity and message state or fail clearly without false delivery. No silent RAM substitution for a persistent node.
3. **Linux–Android protocol parity:** bilateral sends and receipts, sender authenticity, deduplication, replay rejection, blocked-contact enforcement, unread/history consistency, persistence and restart; direct TCP/WS plus node-assisted delivery with the receiver initially offline. Tests must assert the recipient's decrypted content and durable history and the sender's final receipt state, not merely queue/API acceptance.
4. **Network resilience:** no manual contact/route repair after supported restart/address-change cases; cold seed/invite join, stale endpoints, IPv4/IPv6, NAT, network loss, timeouts, loss/reorder, unavailable preferred path and bounded recovery. Explicitly identify which paths need real LAN/multicast/radio hardware. Fix or narrow advertised QUIC support through an approved scope decision.
5. **Resource/abuse limits:** bounded peers, pending requests, message/frame size, custody quota/retention, retry work, WebSocket queues, logs and disk growth under honest and hostile traffic. Observe CPU/RSS/FDs and queue age during representative sustained load; set thresholds from measurements, not invented capacity numbers. Cloud node operation cannot depend on BLE/desktop D-Bus availability.
6. **Secure local/remote operations:** privileged control is not public health; unauthorized API/WS access is denied, secrets/message bodies are excluded from routine logs, identity files are private, service account is least-privileged, and upgrade/rollback preserves the stores. No arbitrary unauthenticated debug/farm endpoints on a public interface.
7. **Native acceptance:** Windows/macOS CLI or agreed desktop UI release installation and upgrade; Android emulator **and** physical device lifecycle/radio tests; iOS simulator **and** physical iPhone transport/background tests. Raspberry Pi power-loss/storage/radio behavior is a separate hardware gate. Unsupported cells require explicit scope waivers, never a green label.
8. **Security/release approval:** triaged secret-scanner finding, dependency review, independent security-sensitive PR review under Rule 8, external cryptographic/protocol review appropriate to production claims, documented residual risks, validated signing and user-approved release. This audit is not that review.

### 7. Dependency-aware takeover work packages

No dates, effort estimates or delivery guarantees are assigned before these gates produce evidence. Work proceeds through small reviewed PRs; no automatic merges.

| Order / dependency | Work package and target subsystems | Required tests and definition of done |
|---|---|---|
| **L0 — first** | Baseline reconciliation and test ownership: `SHIP_PLAN.md`, current release/readiness docs, Linux test scripts, CI definitions. Record this Linux–Android priority; map older PR deltas to current main | Baseline failures classified as product/test/environment; no “already done” tasks reintroduced; ignored socket/hardware tests explicitly assigned; latest-main versus PR evidence recorded. Do not rewrite the historical archive |
| **L1 — after L0, highest implementation priority** | Persistent custody fail-closed semantics: `core/src/store/relay_custody.rs`, storage backend, swarm/IronCore construction and acceptance boundaries | Targeted lock/open/permission/disk failure tests, positive persistent restart test, receipt/custody acceptance invariant. Regression fails against the pre-fix behavior; no acknowledged durable work exists only in RAM. Independent review for any transport/crypto/routing/privacy changes |
| **L2 — after L0; coordinate shared CLI files with L1** | Linux service/control hardening: `cli/src/{main,api,server,config}.rs`, maintained Docker/service packaging | CLI no-stdin start, SIGTERM/SIGINT, kill/restart, readiness failure, port collision, no-BLE/no-D-Bus, private file permissions, API/WS auth/origin and message-size/queue tests. Single-instance identity safe; no public privileged API; no false healthy state |
| **P1 — after L1/L2 fixes relevant to the path** | Hermetic Linux network acceptance harness: `core/tests/`, `cli/tests/`, maintained local Docker/netns test scripts | Three independent node processes/stores; sender and receiver communicate through custody node; restart each at critical handoffs; duplicates/reordering/recipient-offline cases. Assert decrypt + persisted history + final receipt and exact node SHA. No production cloud endpoints or credentials |
| **A1 — paired priority with Linux, after tooling and baseline** | Local Android lab: Android SDK/JDK/NDK/versioned toolchain, x86-64 emulator, `android/app/src/test`, instrumented tests, `MeshRepository`, foreground service/platform bridge | APK/native-library and UniFFI match; JVM tests, instrumented smoke, clean install/upgrade, foreground/background/service restart, storage/backup recovery, UI send/history/receipt. Pair emulator with isolated Linux nodes using explicit emulator-host routes. AVD success is not BLE/cellular proof |
| **P2 — after P1/A1** | Linux–Android parity under failure: routing, discovery, transport, custody, receipt convergence; build on merged seed/ledger/address fixes | Direct and node-assisted bilateral delivery, receiver offline/reconnect, process death, stale route, first-choice path unavailable, authentication/blocked contact/dedup. Automate what IP emulation can cover; preserve separate physical D4/D6/D7 evidence |
| **L3 — after service/protocol contract stabilizes** | Linux amd64/aarch64 packaging and backbone qualification: release workflow, canonical Dockerfile, CLI/UI bundle, Raspberry Pi build path, operational guide | Clean Ubuntu/Debian installs and upgrades; ARM build and native Pi run; no source-tree CWD requirement; non-root container/service, durable volume, health/readiness, graceful stop and rollback. Sustained queue/disk/CPU/RSS/FD observations and power-loss qualification. ARMv7 only if explicitly supported |
| **N1 — after common protocol/FFI stabilization** | Remaining six-target product scope: iOS app/Swift bindings, Windows/macOS CLI and agreed desktop UI, `desktop_bridge`, `shared/`, packaging | Inspect/reconcile Apple PR residuals, regenerate bindings, native builds/tests and install/upgrade. Complete intended desktop UX or obtain an explicit CLI/browser-only waiver. Physical transports/signing remain native-owner gates |
| **R1 — after product gates, security work starts earlier** | Independent security review, signing/release provenance, compatibility and recovery documentation, final risk register | Review fixes closed; published candidate artifacts re-tested; no unresolved critical data-loss/authentication/secret-exposure findings; platform evidence and waivers visible; owner approves version and distribution. Only then consider V1 status |

**Recommended first implementation tranche:** L0 first reproduces and classifies the two observed socket failures, using distinct listener sockets or isolated namespaces without weakening self-dial protection. Establish a non-vacuous network harness before relying on it to validate L1. Then implement the smallest reproducible L1 custody correction and L2's privileged-control/shutdown regressions, while preparing the A1 emulator toolchain without competing builds. This directly serves the user's Linux backbone and Linux–Android priority. It is not a rewrite, a new networking stack, a new orchestration system, or a broad resurrection of old PRs. If the local baseline reveals a more immediate blocking product failure, reproduce and close that first; do not conceal it behind the hardening worklist.

### 8. Practical PR-based ownership

- Start each work package from a fresh main-based worktree/feature branch after checking for concurrent operator changes. Keep one accountable owner per load-bearing subsystem; serialize Cargo/Gradle writes in a worktree.
- Before adopting an open PR, enumerate its complete changed-file list and semantic delta against current main, including any inherited crypto/transport changes. Mark it **superseded**, **unique residual**, or **blocked** only with evidence. Preserve useful tests even when the implementation was superseded.
- A PR carries exact SHA/toolchain, commands/exit status, non-vacuous regression proof, native gaps, migration/rollback notes and independent review where required. Generated bindings are regenerated, not hand-edited.
- Do not trigger paid/external runners, deploy to existing cloud nodes, rotate production credentials, create tags, publish artifacts or merge without the appropriate authorization. No manual workflow dispatch, merge, tag, release, cloud deployment or production credential change occurred in this audit. The user-authorized docs-only push preflight was denied with HTTP 403, so publication remains blocked and no new PR CI was intentionally started.
- Security-sensitive discoveries belong in the project's private vulnerability channel, not public issue dumps. Do not commit logs with tokens, identity keys, private message bodies, live peer dossiers or deployment credentials.
- Close work by acceptance evidence, not by file count, an agent verdict, a build alone, or a passing scanner that excludes the real path.

### 9. Capability and remaining owner dependencies

**Autonomous on this Linux VM:** Rust implementation and bounded test execution, loopback/IP fault harnesses, Linux CLI and service/container development, resource measurement, Android toolchain/emulator setup where supported, JVM/instrumented testing after setup, Android–Linux IP parity, cross-platform source/FFI inspection and PR preparation. KVM API version 12 and `KVM_CREATE_VM` were successfully probed using local elevated access; the current user lacks direct device access, so least-privilege emulator device access still needs local setup. No Android emulator has yet been booted in this audit.

**Not established by this VM's results:** Windows/macOS/iOS execution; Raspberry Pi native performance, SD-card/power-loss behavior; Bluetooth LE, Wi-Fi Aware/Direct and physical cellular transitions; Apple background execution; Android OEM/Doze behavior; store acceptance, signing-key custody, external security approval. Emulator battery/network controls model scenarios but do not reproduce a handset radio or OEM scheduler.

**Minimal remaining decisions/dependencies (not requests for a new broad specification):**

1. Preserve Linux–Android as the first pair. For Linux ARM, use **aarch64 Raspberry Pi OS/Ubuntu as the proposed first ARM target**; decide separately whether ARMv7 is a V1 requirement. The x86-64 VM cannot establish Pi runtime readiness.
2. Confirm the desktop product boundary when updating the canonical plan: the repository's long-horizon scope includes desktop work, but today's actual end-user deliverable is CLI + browser assets, not a finished Compose app. Do not silently narrow “fully complete” to CLI-only.
3. Provide owner-controlled native acceptance access/evidence for Windows, Mac/iPhone, Raspberry Pi and radio-capable Android devices when those gates are reached. No new paid runner/resource is assumed authorized by this report.
4. Resolve production Android keystore/alias mapping privately, Apple distribution/signing arrangements, release approval ownership and independent security review. The Android lab and protocol tests need not wait for the production key.
5. Correct GitHub App/account write authorization before publishing the prepared docs branch: the authenticated dry-run push returned HTTP 403 despite repository metadata advertising push access. Read access succeeded; write access did not. This is a publication dependency, not a reason to rerun the audit.

### 10. Conclusion

**Publication limitation:** the requested GitHub push/PR could not be completed because the authenticated push preflight was denied (HTTP 403). The standalone report and local docs-only branch preserve the work; no application code was changed or merged.

The appropriate commitment is to **own a measured, PR-based Linux–Android-first completion program**, with Linux treated as a durable, resource-bounded mesh backbone and the other platforms retained as explicit release gates—not to promise a perfect or already V1-ready messenger.

Repository access advisory: private-repository access can be granted through the [Abacus GitHub App installation selector](https://github.com/apps/abacusai/installations/select_target). This grants **access only**, not repository creation. The audited repository was public and accessible in this session.

**What really ran:** one Linux workspace run passed 1,905 tests with 25 ignored; formatting, seven wiring-checker tests, the wiring scan, seven APK-checker fixture tests and CLI initialization/identity reload passed. Two then-enabled real-socket tests failed before reflection/offline delivery. **What remains unverified:** successful real multi-node messaging, Android builds/emulator/device execution, native Windows/macOS/iOS/Raspberry Pi behavior, sustained backbone reliability, signing/distribution and independent security assurance. The report and proposed takeover program are complete; the software is not declared V1-ready. Evidence logs remain separate and are not the default user-facing deliverable.
