# Next /orchestrate Kickoff -- v0.4.0 + v0.5.0 + 5-Node Test

Status: Ready to dispatch
Last updated: 2026-08-04

Paste the prompt in section 1 to start the next phase. Sections 2-4 are the
supporting detail that prompt refers to.

---

## 1. The kickoff prompt

```
/orchestrate lanes

Read HANDOFF/SESSION_HANDOFF_2026-08-04_IDENTITY_AND_RUN2.md first -- it has the
current state, the open security debt, and the lane economics you must follow.

FREE LANES ONLY unless a task is a genuine audit gate or judgment call. The last
session exhausted the Anthropic quota by fanning out Claude subagents for bulk
code reading. Bulk reading, mechanical edits, and analysis go to qwenpaid, Groq,
or OpenRouter/FusionLite. Claude native is for verdicts.

Work these in order. Do not start a phase until the prior one is verified green.

PHASE 0 -- UNBLOCK PR #136 (critical path, nothing else moves until this lands)
  a. Verify all four tests pass:
     cargo test -j6 -p scmessenger-cli --test integration_message_requests
  b. ADVERSARIAL REVIEW of the message-request gating fix in cli/src/server.rs.
     REQUIRED before merge per .claude/rules/security.md -- it is a block/allow
     gate. Probe specifically: can a blocked peer reappear as a pending request
     under a different identifier flavor, and do accept/reject still agree on
     which flavor they surface versus consume.
  c. cargo fmt --all -- --check, scoped clippy, commit, push, CI to green, merge.

PHASE 1 -- IDENTIFIER UNIFICATION AUDIT (do this BEFORE the 5-node test)
  We have hit the identifier-confusion bug class twice in one PR. public_key_hex
  and identity_id are both 64-char lowercase hex and both bare Strings, so
  neither the compiler nor a reviewer can catch a mixup. Assume more sites exist.
  A ready-made workflow script is saved and re-runnable -- see section 3 below.
  Run it on free lanes. Deliver a confirmed-findings list plus a structural fix
  recommendation (newtype wrappers vs a single resolution choke point vs wire
  prefixes vs a CI lint).
  Fix every CRITICAL and HIGH finding before the 5-node test, or the test results
  cannot be trusted.

PHASE 2 -- 5-NODE RUN 2
  Nodes: Windows CLI, Android Pixel 6a, iOS, macOS CLI, AWS relay.
  All five must be freshly installed from post-merge main with wiped identity
  state. GPT owns iOS + macOS and is already armed -- see
  HANDOFF/gpt/GPT_WAIT_FOR_PR136_BEFORE_FINAL_IOS_MACOS_BUILD_2026-08-04.md.
  Relay is live at the address recorded in
  HANDOFF/audit/AWS_RELAY_REBUILD_2026-08-04.md -- read it fresh, never copy an
  IP out of a doc, there is no Elastic IP and the address drifts.
  Success = real bidirectional delivery with verified receipts across all five,
  evidenced by ConnectionEstablished plus receipt logs, not by dial-queue logs.

PHASE 3 -- v0.4.0 CLOSE-OUT
  Remaining blockers from HANDOFF/plans/MILESTONE_RELEASE_PLAN.md lines 76-172.
  See section 2 below for the list and which are already satisfied.

PHASE 4 -- v0.5.0 FARM SIMULATION
  Scope at MILESTONE_RELEASE_PLAN.md lines 173-253. Do not start until v0.4.0
  is tagged.

Standing rules: delegate everything implementable; commit after each verified
task; never push unless asked; treat every IP in every doc as ephemeral; run the
finalize-checklist skill before declaring done.
```

---

## 2. v0.4.0 remaining blockers (canonical list at MILESTONE_RELEASE_PLAN.md:81)

| # | Item | Status entering next session |
|---|---|---|
| 1 | Outbox Site-1 flush on reconnect | Reported DONE at HEAD (f521f142, 4 call sites) -- VERIFY by grep before dispatching |
| 2 | Receipt round-trip fix | Reported DONE (8f866bfc + iron_core.rs classify path) -- VERIFY |
| 3 | Android retry suppression | Kotlin side, confirm state |
| 4 | AWS relay live proof | **Relay is LIVE and verified 2026-08-04.** WAN end-to-end proof still owed |
| 5 | DNS-name-first hardening (IP-flip mandate) | OPEN, and now urgent -- the relay has no Elastic IP, so IP flip is a live failure mode. Note the unclosed `/dns4/` filter-bypass finding from the IP sweep |
| 6 | Bootstrap topology wiring | OPEN. Related: the JS clients still hardcode dead bootstrap IPs |
| 7 | Ledger convergence test + fix | Reported DONE 2026-07-23 -- VERIFY |
| 8 | Graceful dial policy | Reported COMPLETED per _QUEUE.md 2026-07-21 header -- VERIFY |

Human gates: H-04 (AWS relay activate) is now SATISFIED. Lucas port forwards
(tcp/443, tcp/80, udp/443) plus DDNS record still outstanding.

Do not trust a "DONE" marker without a grep-level confirmation at HEAD --
`_QUEUE.md` explicitly warns that several doc-claimed completions were stale.

---

## 3. Re-running the identifier audit

A complete workflow script already exists and is re-runnable:

```
C:\Users\SCM\.claude\projects\C--Users-SCM-Documents-GitHub-SCMessenger\09cba5c5-aa56-4e2b-b944-47e4c73a6010\workflows\scripts\identifier-unification-audit-wf_0634f975-9c1.js
```

It fans out five surface inventories (core identity/crypto, core store and
iron_core, core transport/routing/privacy, CLI plus WASM rpc, Android/iOS/UniFFI),
builds a producer-to-consumer matrix, adversarially refutes every candidate before
confirming it, then designs the structural fix.

**Its prompts are lane-agnostic and can be lifted wholesale into
`scripts/delegate_task.py` task files.** Do that rather than re-running it as a
Claude workflow -- that is what exhausted the quota. Each surface inventory is an
independent task file; run them against qwenpaid or OpenRouter, collect the five
outputs, then do the cross-reference and the confirm/refute pass.

Seed every dispatch with the central hazard, because it is the whole reason the
bug recurs:

> `public_key_hex` (the Ed25519 public key) and `identity_id`
> (`hex(blake3(public_key_bytes))`) are BOTH 64-character lowercase hex strings.
> Indistinguishable by length, regex, or eyeball. Both are bare `String`. The
> compiler cannot catch a mixup; it appears at runtime as a silent lookup miss or
> a "wrong key" decryption failure. Never assume a value's flavor from its
> variable name -- trace the actual value.

---

## 4. Known traps for the next orchestrator

- **Qwen diff headers are malformed.** Qwen emits `@@ def function_name(` instead
  of `@@ -1341,7 +1341,12 @@`. Every apply method rejects it as "only garbage was
  found in the patch input." The code is fine; the framing is not. Post-process
  the header or transcribe the body.
- **Do not read `$?` after a pipe.** `cargo fmt --check | head; echo $?` reports
  the status of `head`, always 0, so the gate silently cannot fail.
- **One build tool at a time on Windows.** Multiple agent sessions share this
  repo; Gradle can spawn cargo-ndk upstream. Check `tasklist` for
  cargo/rustc/gradle/java before any build.
- **Cargo is RAM-bound here:** 16 cores, ~11.8 GB RAM. Use `-j6`.
- **Disk was at 93% (18 GB free).** The build rules want 25 GB before a full
  five-gate sweep. Prefer scoped builds. Never `cargo clean --target <triple>` --
  despite the name it wipes all of `target/`.
- **Repository Hygiene CI fails on trailing whitespace,** and CRLF line endings
  present as trailing whitespace. Two CI cycles were burned on this.
- **No emojis anywhere** -- hook-enforced, and it scans the whole file, not just
  the changed lines.
