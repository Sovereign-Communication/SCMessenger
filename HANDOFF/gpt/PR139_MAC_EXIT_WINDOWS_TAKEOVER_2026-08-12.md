# PR139 Mac Exit and Windows Takeover - 2026-08-12

Status: **HARD NO-GO for responder cutover and field qualification; GO for control-plane ownership handoff only**
Last updated: 2026-08-12T10:10:40Z
Evidence snapshot: 2026-08-12T09:58:26.183504Z

## Executive verdict

This packet transfers administrative orchestration ownership; it does not transfer technical readiness.

- **Responder cutover: HARD NO-GO.** The accepted responder is an isolated synthetic artifact. Exact sender mapping, production execution/completion, node-enforced send idempotency, rollback, and receiver-backed proof remain open.
- **Ownership handoff: GO (control-plane only).** Windows may take over coordination while preserving every prohibition and evidence boundary in this packet.
- **Five-node gate: INCOMPLETE.** The official current-candidate score is **0/12 directional flows**, Matrix Pass 1 is not started, Matrix Pass 2 is not started, and the continuous five-node soak is 0/60 minutes.

The 0/12 score does not erase field work. Live and historical evidence is preserved for Windows, Android, macOS, and iOS at its own timestamp and provenance. It cannot be promoted to a current-candidate PASS until all five nodes are reconciled to one frozen source, each runtime/artifact is separately anchored, all 12 flows have receiver evidence, G1-G6 pass twice, and the full fleet completes one reset-free 60-minute soak.

No deployment, responder activation, contact/device mutation, message send/reply, delivery claim, Matrix Pass 1, soak, PR merge, or objective-complete claim is authorized by this handoff.

## 1. Ownership after Mac exit

| Owner | Authorized scope | Boundary |
|---|---|---|
| Mac lane | iOS signing/build/install, physical-iPhone evidence, macOS CLI/platform evidence, authoritative `xcodebuild`, and Mac-side adversarial review | iOS/macOS only. Do not take over Rust/Core integration, Android, the AWS cloud node, shared-PR merge, release tags, or Windows gates. |
| Windows orchestrator | Windows CLI, Rust/Core integration and authoritative non-iOS gates, Android build/sign/install/physical-Pixel evidence, AWS cloud-node artifact/deployment/custody evidence, branch integration, CI invocation, and final merge coordination | Do not fabricate or simulate Mac/iPhone proof. Core security-sensitive changes require independent adversarial review. |
| Human operator | H1-H4 below, security/privacy and API-contract choices, signing/account decisions, deployment authorization, destructive action, release timing/versioning | These decisions cannot be inferred or delegated away. |

Product doctrine remains: all deployments are full nodes and every node relays through store-and-forward custody behavior. The AWS deployment is an **AWS cloud node**, not a standalone relay. For this field matrix only, it supplies an infrastructure test function and is excluded from ordinary user-chat pair counting; its identity, reachability, route, and custody behavior remain mandatory evidence.

## 2. Provenance planes - never collapse them

| Plane | Immutable snapshot | Meaning and limit |
|---|---|---|
| PR source | Git commit `090b134041ee9f486bd1dd0c774ad715fd1746ad` on remote `tracking/pre-v040-tag-work` | Audited remote PR head, not a frozen or deployed candidate. |
| PR base | Git commit `ef431acc0dc6c5112cac16d40e77414a092dbdc0` on `origin/main` | Audited base and base of the clean publication worktree. |
| Publication source | Branch `gpt/pr139-mac-exit-handoff-20260812` at base Git commit `ef431acc0dc6c5112cac16d40e77414a092dbdc0` before these docs | Documentation publication plane only; it is not PR #139 runtime source. |
| Mac checkout source | Git commit `a29e53f384e038c1e35ee4e4f18972a008af5436` on local `gpt/ios-macos-launch-debug-20260810` | Source checked out in the dirty Mac working tree; not the running binary or PR head. |
| Mac runtime-reported source | Version `0.4.0`, Git commit `e7ac25c4f683431df3c4fdbcd6c3937d49a670fc`, build time `2026-08-11T19:53:17Z`, reported at `2026-08-12T09:54:04Z` | Embedded runtime metadata from the running Mac binary. Its binary digest was not captured; it does not prove checkout or PR equivalence. |
| Receiver evidence | Metadata report at `2026-08-12T09:47:24Z`: 200 history rows, including 166 inbound and 34 outbound; zero exact history `peer_id` matches to either uniquely pinned contact | Exact sender routing is unavailable and fails closed. This is not delivery, ACK, wake, or authenticated responder evidence. |
| Older restart anchors | Source anchor Git commit `ab4f448635ae7bca0592bf3f615fa818eeb765fc`; runtime/artifact anchor `9f54b1078ad512c895b68029c9e79a1870d7f286` | Historical anchors retained byte-for-byte in the restart packet; neither replaces the fresher PR, checkout, runtime, or receiver snapshots above. |

Immediate consequence: PR `090b1340...`, Mac checkout `a29e53f3...`, and Mac runtime report `e7ac25c4...` differ. G6 is blocked before a message flow begins. Each future node manifest must record source commit, artifact/image digest, runtime build stamp/version, stable identity continuity, UTC interval, and receiver evidence in separate fields.

## 3. PR and branch truth at the audited snapshot

- PR #139: OPEN, not draft, `MERGEABLE`, merge state `UNSTABLE`.
- Checks: 32 total; 31 `SUCCESS`; one `FAILURE`, `Repository Hygiene`.
- Review decision: none recorded.
- PR source/base: `090b134041ee9f486bd1dd0c774ad715fd1746ad` / `ef431acc0dc6c5112cac16d40e77414a092dbdc0`.
- The PR had 182 comments. The newest scoped comment metadata preceded later PR metadata and checks, so comments are historical context rather than current head/check truth.

This is a time-bounded, read-only snapshot. Windows must revalidate the active writer, PR head/base/checks, remote refs, and open PRs before acting. A changed head or another active writer is a stop condition, not permission to improvise.

## 4. Exact Mac monitor answer

| Plane | Verdict | Preserved evidence | Gate boundary |
|---|---|---|---|
| launchd supervision | **WORKING** | The Mac CLI daemon and wake watcher jobs were running at `2026-08-12T09:54:04Z`; process presence and restart supervision were observed. | Supervision is not transport health, responder execution, or delivery. |
| Mac CLI daemon | **PARTIAL diagnostic only** | The daemon had been up about 13h54m; `/version` was reachable; the newest 2 MiB log slice had no panic/swarm-death marker. | Runtime source differs from PR and checkout. The newest slice contained 755 ERROR and 282 WARN entries, and historical stderr retains request-response assertion and CoreBluetooth panic signatures. This is not five-node stability proof. |
| wake watcher/bridge | **PARTIAL and lossy/ambiguous admission** | Fresh inventory: 12 accepted ledger records, 11 `queued`, 1 `delivered`; 8 history outages and 442 bridge-unavailable/active-writer warnings were retained. The older restart snapshot counted 11 accepted, 10 queued, 1 delivered; the difference is preserved as snapshot/counter drift, not silently reconciled. | API polling is not a native event stream. The log stopped changing hours before the final snapshot. `queued`/`delivered` here are bridge admission statuses, not turn completion, SCM delivery, receiver ACK, or sender convergence. |
| responder | **NOT DEPLOYED and BLOCKED** | Synthetic responder and execution models passed bounded tests. No responder launchd job and no separate live responder process were observed. | Exact pinned-sender mapping is zero; durable live claim/running/pinned-send/completed semantics and a node-reconciled idempotency boundary are absent. Keep the live bridge unchanged. |

Combined answer: launchd supervision is working; daemon diagnostics are partial; wake admission is partial and lossy; the responder is not deployed and remains blocked.

## 5. Preserved four-client-lane field evidence

These are the strongest scoped live/historical observations. They remain evidence at their recorded candidate, timestamp, direction, and proof class; they are not rewritten as current-candidate matrix results.

| Client lane | Strongest preserved evidence | Current qualification boundary |
|---|---|---|
| Windows CLI | The historical 2026-08-05/06 plan records a Windows CLI build at then-main Git commit `6b2573fa...`; cross-lane ACK adjuncts remain preserved. Historical receipt roundtrip tests reported 8/8 and CLI BLE tests 7/7 at `2026-08-11T06:02:15Z`. | No fresh Windows manifest tied to PR source `090b1340...` and no fresh full receiver E1-E5 chain. Official current-candidate flows involving Windows remain unscored. |
| Android Pixel | Physical Pixel 6a plus iPhone live bidirectional Android/iOS messaging was confirmed on `2026-08-05`. The historical plan records an in-place Pixel install at then-main `6b2573fa...`; later signing-lineage evidence showed the matching local Android debug key can preserve identity/history on update. | That proof predates this candidate. Isolated lifecycle/identity drafts are undeployed and have no authoritative Gradle/APK/device result. |
| macOS CLI | Fresh live diagnostics show launchd-supervised runtime availability at `2026-08-12T09:54:04Z`, plus preserved partial Mac ingestion/ACK adjuncts. A focused macOS btleplug containment run reported 3 passed, 0 failed at `2026-08-11T17:52:18Z`. | The running source report is `e7ac25c4...`, not PR `090b1340...`; scan containment does not prove GATT/BLE parity or a receiver-backed pair. |
| Physical iPhone | The `2026-08-05` physical Android/iOS field run proved bidirectional messaging on the then-tested artifacts. | The evidence is historical and remains valuable, but current signed-build provenance, identity continuity, and current receiver E1-E5 evidence are unresolved. |

**Official current-candidate score: 0/12. Preserved historical four-lane field evidence: present and retained.** The distinction is candidate qualification, not deletion of observed field behavior.

## 6. High-value isolated artifacts and exact results

| Artifact | Exact anchor/result | What Windows may reuse | Boundary |
|---|---|---|---|
| Restart control packet | `HANDOFF/gpt/PR139_PRIME_ORCHESTRATE_RESTART_2026-08-12.md`; SHA-256 `fa2f7828df965f2c022cc1d1dc43a31fc10f83029e3c42e3b868e58e81ae6b1e` | Byte-identical fresh-session prohibitions and initial worker contracts | The publication copy must remain byte-identical. |
| Sol completion gate | SHA-256 `00d3096893368b67f9b76d1fb9aeac572c8fc158ebd107dab8025965f9fab71e` | Objective/evidence matrix | Static HARD NO-GO evidence. |
| Sol identity parity gate | SHA-256 `bd06803290dfb07e4637d3037b56000b1930932ee3962413f65c4b440298b3c6` | Core/Android writer and authority inventory | Requires approved contract, security review, and Windows/device gates. |
| Sol post-reconciliation gate | `tmp/sol-pr139-postreconcile-gate-20260812.md`; HARD NO-GO at `2026-08-12T05:03:33Z` | High-level missing-proof list | Its digest was not supplied by the scoped inventory; do not invent one. |
| Android drafts | `tmp/android-boot-fix-20260812` and `tmp/terra-android-pr139-20260812`; scoped diff-check reported clean | Lifecycle, motion, and identity starting diffs | No Gradle, APK, deployment, or device verification. Review/rebase on Windows. |
| CLI stable-ID/cursor drafts | `tmp/cli-history-id-contract-20260812`; `tmp/terra-history-cursor-pr139-20260812`; `tmp/terra-history-cursor-impl-20260812`; both Terra cursor investigations BLOCKED | Stable event-ID starting diff and proof that snapshot/revision semantics are prerequisite | Serialization, cursor, storage snapshot, reconnect, and gap-free semantics remain open. |
| Earlier responder ledger | `tmp/terra-responder-hardening-20260812`; **9/9 Python 3.9 tests passed** | Regression cases | Undeployed and superseded as final MVP acceptance anchor. |
| MVP responder | `tmp/pr139-mvp-responder-20260812` and `tmp/pr139-mvp-acceptance-20260812/FINAL_ACCEPTANCE_REPORT.md`; final implementation hash `3c8c62719ab0099ab433d1043e8bd9171efb7084126dcb8d9b8ff80ade72179e`; **13/13 artifact tests plus 12/12 independent tests = 25/25 Python 3.9 tests passed**; `py_compile` and scoped diff-check passed; peak RSS `109520 KiB` | Durable claim/lease, ambiguous-send fencing, fail-closed two-sender model, reconciliation oracle | Synthetic artifact only; no live API, Prime, SCM, receiver, deploy, or rollback proof. |
| Sender mapping | `tmp/pr139-history-sender-mapping-20260812/MAPPING_REPORT.md`; final report SHA-256 `44fb0ee9985fb685932300124297e884636be9497fe0425c75cf7d93a05eb3b6`; 200 rows; zero exact matches to either pinned contact | Fail-closed identity evidence | Never infer principal from nickname, content, platform, or device metadata. |
| Prime execution contract | `tmp/pr139-prime-execution-contract-20260812/FINAL_CONTRACT_GATE.md`; final report SHA-256 `dae8bf2d0bb1f27f97121ce239e5ff01efb8317495239ef934609b6e73a3aba4`; **15/15 tests passed**: 11 positive model cases and 4 live-interface counterexamples | Exact acceptance oracle for `claim -> running -> pinned send -> completed` and crash windows | Overall verdict BLOCKED because production adapters lack the modeled primitives. |
| Dual-sender watcher design | `tmp/pr139-dual-sender-watcher-design-20260812.md`; BLOCKED | Event/principal separation and cutover checklist | Authentication and shared-cursor aliasing unresolved. |
| Platform readiness matrix | `tmp/pr139-platform-lanes-readiness-20260812.md`; BLOCKED / NO-GO | Cross-platform missing-evidence inventory | Artifact/static audit only. |
| Smart-monitor snapshots/fixtures | `tmp/pr139-smart-monitor-pre-restart-20260811T192019Z.txt`, `tmp/pr139-smart-monitor-post-restart-20260811T192019Z.txt`, and synthetic event records | Diagnostic regression fixtures | Synthetic wake was not persisted inbound evidence and triggered no SCM send. |
| Request-response admission patch | Branch `gpt/pr139-libp2p-admission-fix-20260811`, Git commit `860f5ed561c625cfa2e5fa0ea70f664cc15dd70b` | Candidate behavior-ordering fix | Isolated formatting and diff-check passed; the first focused Cargo test did not finish. Current Core transport review and Windows-authoritative gates required. |
| macOS BLE containment | Focused `ble_mesh` run: **3 passed, 0 failed** | Safe scan/discovery containment | No macOS GATT/peripheral parity and no G2 receiver evidence. |
| Historical receipt/BLE suites | Receipt roundtrip **8/8**; CLI BLE **7/7** | Regression leads | Historical candidate only; rerun on the frozen source. |

### 6.1 Exact MVP acceptance hashes

- Final acceptance report: `a50284196a8af621e012e6c5a7ec7c00562561b9494a09ae244c17e066925fab`
- Checkpoint: `287b18e4620af283b362505e926e4b09ddd6d7d89f751473711769869858c8dc`
- Prior compatibility report: `d8e321e4e10809005ed50f31d077c49fe4529600fe1476f0332bae4c1527aa36`
- Responder implementation: `3c8c62719ab0099ab433d1043e8bd9171efb7084126dcb8d9b8ff80ade72179e`
- MVP test suite: `ffc2929cdf6600413c198184ad6c5a00c283c97359ea2a73ea93eea20cee117d`
- Synthetic fixture: `cbe114279fec07c90c13e02a10c634791b1b68e8258234db822d9b5cc11fb8f6`
- Deployment manifest: `847905437b5066f8fa108cbf96fe1a8786b081d4d8d7127dedb3adfe34cff0d1`
- Fix result: `37682bfab21f8f646f361a0183014ea74725fd52e0246f1e715b2f40202d56d3`
- Independent acceptance test: `62cb248428e1aba09221157a8f6e6140b4867f5ebe7e9fc0f40f87b2490910d8`

### 6.2 Exact Prime contract hashes

- Final contract gate: `dae8bf2d0bb1f27f97121ce239e5ff01efb8317495239ef934609b6e73a3aba4`
- Contract model: `1f9bf2f90c53348f4da4177a7bc344a75cf07fabecaea8cb1bf6cfbf68e0be13`
- Contract tests: `15f7d53d41162f425c0def9897523c76ed52eaeb4d32816e57919d6681adcd0d`
- Test evidence: `293de0ce839860bb4743f04d2f772723e16f3a48d520c5ad2df362e721ddba61`
- Contract: `2f5618e1359c11d6ce0ed3a2b01c2e60485a45eff00e0338a735b87002d79232`
- Contract review: `ba921a1cd670778a6b899cffad081eda543fc435f247bb5c4fc59a5cb7980208`

All test counts above are artifact-local. None is a field delivery or responder-cutover PASS.

## 7. Dirty-state preservation

The shared Mac checkout was dirty before publication work began:

- Modified: `HANDOFF/gpt/MAC_WINDOWS_BLE_PARITY_QUEUE_2026-08-11.md`
- Modified: `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift`
- Untracked source restart packet: `HANDOFF/gpt/PR139_PRIME_ORCHESTRATE_RESTART_2026-08-12.md`
- Untracked: four files under `HANDOFF_AUDIT/turbofieldfare-audit/iron-core-muse-iq2-xs-comparison/`
- Untracked: `scripts/run_triplepass_turbofieldfare.py`
- Numerous repo-local worktrees and old prunable registrations also exist.

Preserve all of it. Do not restore, stash, reset, clean, prune, delete, rebase, or use `git commit -a`. Do not edit the shared restart source. Unknown changes are foreign in-progress work, not cleanup candidates.

## 8. Branch and publication strategy

### 8.1 Clean documentation publication

1. Use only the clean worktree `tmp/publish-pr139-handoff-20260812` on `gpt/pr139-mac-exit-handoff-20260812`, based on `origin/main` Git commit `ef431acc0dc6c5112cac16d40e77414a092dbdc0`.
2. Publish exactly the byte-identical restart packet and this takeover packet as documentation-only changes.
3. Require independent review before commit/push and before posting the PR comment. Use a normal fast-forward push of the new `gpt/*` branch; never force-push.
4. This documentation branch does not change PR #139 source, deploy a runtime, or close any gate.

### 8.2 Existing Mac code branch - independent publication verdict BLOCKED

Independent publication review verdict: **BLOCKED. Do not push** `gpt/ios-macos-launch-debug-20260810` at Git commit `a29e53f384e038c1e35ee4e4f18972a008af5436`. That commit contains `core/src/iron_core.rs` and `core/tests/integration_ironcore_roundtrip.rs`; Core Rust is reserved to the Windows orchestrator and its AUDIT-GATE. The branch tip also lacks an exact-tip authoritative gate/artifact match: the preserved Mac build/runtime evidence names earlier or different sources, no Mac-authoritative `xcodebuild` result is tied to the exact tip tree, and the receipt/outbox change lacks exact-tip Windows Core gates and the reserved review. Leave the branch local until those blockers are closed and a fresh independent audit accepts the exact source.

The docs-only branch proceeds independently. It is based on Git commit `ef431acc0dc6c5112cac16d40e77414a092dbdc0`, contains only the byte-identical restart packet and this takeover packet, and does not contain `a29e53f...`, Core Rust, or code-branch history. Its publication still requires its own exact two-file review and normal non-force gate.

### 8.3 Safe Windows integration branch

1. Re-read current rules and confirm no orchestrator is actively writing.
2. Fetch the remote PR head without changing the Mac checkout; revalidate PR head/base/checks.
3. If `090b134041ee9f486bd1dd0c774ad715fd1746ad` is still the PR head, create a **new** Windows-owned branch, suggested `windows/pr139-takeover-20260812`, from that freshly fetched commit.
4. Never push local `tracking/pre-v040-tag-work` at `c57e167382e976a59e5630128656657bebff5d08` to the shared PR branch; it is divergent. Never overwrite, rebase, force-push, or delete the shared branch.
5. Import only independently accepted diffs as new normal commits. Prefer reviewable stacked work; advance shared refs only through the authorized, gated, non-force path.

## 9. Five-node rows and 12 official flows

| Row | Node and owner | Current status | Required next evidence |
|---:|---|---|---|
| 1 | Windows CLI - Windows | BLOCKED; current runtime/artifact not anchored by this Mac audit | Source, binary digest, runtime stamp, identity continuity, listeners/log start, and both-direction receiver E1-E5 evidence. |
| 2 | Android Pixel - Windows | BLOCKED; historical field evidence exists, isolated drafts are unverified | Review/rebase; authoritative Android gates; matching signing lineage; in-place install; lifecycle stress; source/artifact/runtime/identity manifest; receiver E1-E5. |
| 3 | AWS cloud node - Windows | BLOCKED; immutable current artifact and custody route are not tied to the PR source | Operator-authorized immutable deployment, image digest, runtime/source stamp, stable identity/listeners, forced custody with recipient offline and later receiver convergence. |
| 4 | macOS CLI - Mac | BLOCKED; live runtime is proven present but mismatched | Rebuild from frozen source without data wipe; binary digest/runtime stamp; identity continuity; Mac-authoritative evidence and every required receiver chain. |
| 5 | Physical iPhone - Mac | BLOCKED; historical field evidence exists, current signed provenance is unresolved | Operator-approved signing, state-preserving install, source/build/identity manifest, same-lane/cross-lane/roaming receiver evidence. |

The 12 directional flows, each requiring E1-E5, are:

1. Windows -> Android
2. Android -> Windows
3. Windows -> macOS
4. macOS -> Windows
5. Windows -> iPhone
6. iPhone -> Windows
7. Android -> macOS
8. macOS -> Android
9. Android -> iPhone
10. iPhone -> Android
11. macOS -> iPhone
12. iPhone -> macOS

Receiver E1-E5 means: **E1** receiver ingest; **E2** decrypt/application acceptance; **E3** exactly one clean durable history item; **E4** authenticated application receipt; **E5** sender convergence with no continued transmission. Sender acceptance, local history, watcher detection, bridge admission, transport ACK, custody alone, or a `delivered` flag cannot substitute.

## 10. G1-G6, two passes, and soak

| Gate | Status now | Objective closure |
|---|---|---|
| G1 pairwise bidirectional | BLOCKED, 0/12 official | All six endpoint pairs, both directions, with E1-E5 for every flow. |
| G2 transport coverage | BLOCKED | Classified LAN/Wi-Fi, applicable BLE, and forced AWS cloud-node custody/routes with receiver outcome; no route inferred from candidate presence. |
| G3 delivery truth/durability | BLOCKED | No false delivered/failure state; accepted work survives offline/backoff without finite-attempt abandonment; delivery occurs on opportunity; valid receipt stops transmission. |
| G4 fleet convergence | BLOCKED | All messaging endpoints learn the expected fleet; cloud node remains reachable; restart reconverges without wipe or re-pair. |
| G5 liveness | BLOCKED | Full multi-peer fleet survives controlled disruption/reconnect without panic, process/swarm death, app restart, unbounded storms, or evidence loss. |
| G6 provenance | BLOCKED | One frozen source for all nodes, with each platform artifact/runtime digest recorded separately and exactly; any mismatch invalidates the run. |

- Matrix Pass 1: NOT STARTED.
- Matrix Pass 2: NOT STARTED and must be a fresh complete repetition on the same frozen runtime anchor.
- Five-node soak: NOT STARTED, 0/60 minutes. It begins only after both matrices pass and must retain the full fleet, periodic low-rate flows, one controlled transition, continuous collectors, and start/midpoint/end provenance snapshots.

A reduced-fleet or one-peer uptime window never counts as G5 or soak.

## 11. Human decision queue - H1 through H4

| ID | Operator decision required | Evidence needed before closure |
|---|---|---|
| H1 | Core-owned identity-scoped nickname migration/version/conflict contract and authenticated-envelope sender boundary | Written architecture/API decision and security classification; no platform-local cache becomes authority by inference. |
| H2 | Canonical history sender identity, stable event ID/cursor semantics, SCM node-enforced send idempotency/reconciliation, and Prime terminal-completion semantics | Approved request/response and state-transition contract covering timeout/crash ambiguity, conflicting key reuse, prior-result lookup, and admission vs. completion. |
| H3 | Whether Lucas has a distinct authenticated identity in scope or is removed; generic Android must not be aliased to Lucas | Signed scope decision and, if retained, metadata-only authenticated identity that exactly matches one pinned contact. |
| H4 | Trust-scoped LAN disclosure and its security/privacy trade-off | Recorded policy decision and independent security disposition on the exact candidate. |

Release timing/tag flavor remains human-only after, not before, a complete five-node gate.

## 12. Smallest ordered Windows dispatch checklist

This ordering is intentionally simple for DeepSeek V4 Flash. The controller gives one worker one bounded prompt and expects one evidence file. Keep dependent work sequential; never exceed three active workers. Every prompt states one hypothesis, exact paths/API shape, canonical anchor, objective acceptance test, stop rule, tool limit, and 384 MiB RSS ceiling. Required worker response fields: `RESULT`, `ANCHORS`, `CHANGES`, `TESTS`, `BLOCKERS`, `NEXT`.

- [ ] **W0 - accept control only.** Record the three Sol verdicts, evidence hashes, dirty-state prohibition, and 0/12 status. Acceptance: a read-only acknowledgement. Stop on an active writer or changed evidence.
- [ ] **W1 - truth preflight.** Revalidate PR head/base/checks, remote refs, current Windows checkout, worktrees/dirt, and node availability without deployment. Acceptance: one timestamped source/runtime-separated snapshot. Stop on any head mismatch, conflict, or unknown writer.
- [ ] **H - obtain H1-H4.** Do not ask an implementation worker to decide them. Acceptance: written operator decisions. Stop all dependent identity/history/idempotency/disclosure work until received.
- [ ] **W2 - patch and security triage.** Independently accept/reject receipt/outbox lineage, request-response admission, Android lifecycle/identity, CLI stable-ID/cursor, and responder/Prime-contract work; resolve Repository Hygiene. Acceptance: exact-diff dispositions, required tests, and exact-candidate adversarial review for Core-sensitive changes. No deployment.
- [ ] **W3 - freeze one source.** Run authoritative Windows gates after accepted fixes/reviews. Acceptance: declared Git commit plus separately recorded build artifact digests. Any runtime change returns to W2.
- [ ] **W4 - build the Windows-owned three rows.** Windows CLI, state-preserving Android, then operator-authorized AWS cloud node. Acceptance: three complete source/artifact/runtime/identity/listener manifests. Stop on source mismatch, identity loss, signing mismatch, unstable process, or custody failure.
- [ ] **W5 - request the Mac two-row packet.** Mac supplies macOS CLI and signed iPhone manifests and authoritative Mac results from the same frozen source. Windows correlates but does not edit/simulate. Stop on any mismatch or destructive install requirement.
- [ ] **W6 - receiver preflight.** Prove collectors, route classification, exact pinned metadata, production-adapter dry compatibility, rollback manifest, and at least one fresh E1-E5 chain in each lane. Admission-only or synthetic evidence fails. Responder staging still requires explicit operator authorization.
- [ ] **W7 - official gate.** Run Matrix Pass 1, fresh Matrix Pass 2, then the uninterrupted 60-minute full-fleet soak. Merge consideration begins only after every evidence row passes and independent review accepts the bundle.

## 13. Exact HARD NO-GO and rollback/reset rules

### 13.1 HARD NO-GO

Until every prerequisite is independently evidenced:

- Do not deploy, install, reload, or activate a responder.
- Do not mutate contacts/devices, send/reply, or claim execution, completion, delivery, ACK, or convergence.
- Do not merge PR #139, begin Matrix Pass 1, or begin the soak.
- Do not infer receiver proof from PR checks/comments, sender acceptance, local history, bridge `queued`/`delivered`, custody, peer tables, or route attempts.
- Do not collapse source, runtime/artifact, and receiver evidence.
- Do not wipe, re-pair, or destructively reinstall to align identity.
- Do not force-push, overwrite, rebase, delete, prune, clean, stash, restore, or reset shared work.
- Do not publish credentials, identity material, device identifiers, PINs, message text, tokens, or raw API responses.
- Do not accept Core crypto/transport/routing/privacy changes without the required independent adversarial review.

### 13.2 Reversible staging prerequisites

Before any later operator-authorized staging, record: prior and proposed artifact digests; service/job definitions; configuration schema; data/ledger locations; health checks; exact disable/revert sequence; evidence preservation steps; and the operator who authorized it. Rollback must preserve data and evidence and must return to the known no-responder state. Do not improvise destructive rollback.

### 13.3 Reset and rollback triggers

- Exact sender mismatch/ambiguity, lost ownership, unknown send outcome, duplicate visible send, or missing completion: stop responder activity, preserve ledger/log evidence, return to no-responder state, and reopen H2/P1/P2. Do not retry blindly.
- Runtime/source or artifact mismatch: invalidate qualification, correct all manifests, and restart from freeze/deployment.
- Runtime code change: new source anchor, redeploy all five nodes, restart Matrix Pass 1.
- Unexpected identity change: stop, preserve state, diagnose persistence without wipe, then restart the current matrix.
- Panic, process crash, or swarm event-loop death: immediate FAIL; fix and re-anchor, then restart Matrix Pass 1 if runtime changed.
- Missing receiver/receipt evidence: flow is unproven; rerun only after collector health is restored.
- Collector failure: affected evidence is invalid; silence from a dead collector is not a PASS.
- Delivered message resumes transmission: immediate G3 FAIL.
- Accepted message becomes permanently abandoned due to attempt count: immediate G3 FAIL.
- Required AWS cloud-node custody path unavailable when direct transport is removed: G2/G5 FAIL.
- Any reset condition during soak resets the 60-minute clock; both passed matrices remain usable only if source, runtime artifacts, identities, and their evidence remain unchanged and the failure does not invalidate their gate result.

## 14. Evidence index

| Evidence | SHA-256 / anchor | Authority boundary |
|---|---|---|
| Exit evidence inventory | `e13bc388a55c0efc83037fbdb3bba0db3bd8b7dedf4c79a4bcb79d07615b640d` | Audited inventory snapshot at `2026-08-12T09:58:26.183504Z`; not live mutation authority. |
| Independent Sol final handoff gate | `f935d67e176828481a2f7176916c4589663fb8ec0fd094138e9d8871eab37f74` | `responder cutover HARD NO-GO`; `ownership handoff GO (control-plane only)`; `five-node INCOMPLETE`. |
| Independent Mac code-branch publication gate | `6e57619e9aa118a8fb4647288480ebd173bbb66d717f272270a94c47d3d2fd40` | `gpt/ios-macos-launch-debug-20260810` at `a29e53f...` is BLOCKED: reserved Core Rust is present and exact-tip authoritative gate/artifact evidence is absent. The docs-only branch is evaluated independently. |
| Restart packet | `fa2f7828df965f2c022cc1d1dc43a31fc10f83029e3c42e3b868e58e81ae6b1e` | Byte-identical control packet; publication copy is canonical for restart. |
| MVP final acceptance | `a50284196a8af621e012e6c5a7ec7c00562561b9494a09ae244c17e066925fab` | Synthetic PASS, 25/25; no cutover authority. |
| Metadata sender mapping | `44fb0ee9985fb685932300124297e884636be9497fe0425c75cf7d93a05eb3b6` | BLOCKED, exact mapping zero. |
| Prime final contract gate | `dae8bf2d0bb1f27f97121ce239e5ff01efb8317495239ef934609b6e73a3aba4` | 15/15 model/counterexample tests; production verdict BLOCKED. |
| Canonical five-node field reference | `acb4a17e491a5e049b610f6176fc67d5276e7c935f9237aed0cd736534806b8a` | Topology, 12 flows, matrices, reset rules, soak, and master checklist. Its old candidate facts are historical. |
| Compact five-node plan | `f6c092aa645a9ba984737db930fdb0a02e5f340e52b7e4e86b576fc2da762d82` | G1-G6 and preserved `2026-08-05` Android/iOS field proof. Its resume candidate is stale. |
| Repository rules | `AGENTS.md`, SHA-256 `4642e0a3d71c6bd369e49651eb9cff9eece88a5c3ab06d76065e95db6a911d68` | Current universal capability and safety contract. |

The Mac-local `tmp/` reports may disappear after publication. Their decisive verdicts, counts, hashes, provenance boundaries, branch rules, dirty-state warnings, and next actions are therefore embedded in this committed packet rather than required for restart.
