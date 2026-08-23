# CTO state — live handoff

Status: Active
Last updated: 2026-08-22 (Antigravity session audit; two dispatches validated)
Entry point: `/CTO`. This file is the whole context load.

## 0-2026-08-22d. BOTH FIXES LAND AND VERIFY -- main is GREEN

### main is green on `b538f3ba`. D1 SATISFIED.

All nine lanes success: CI, Lint, Mobile, Cross, iOS Build & Test, Docker
Integration Suite, Docker Publish, Repository Hygiene, Push on main. The 30+
minute wait earlier in the session was runner backlog from the dependabot
batch, not failure. **This is the first green main this plan has recorded.**

### Packet J -- V2 sender auth. VERIFIED BY THE CTO, not by the worker.

Branch `cto/v2-sender-auth-2026-08-22`, commit `2aadf489`, pushed.

**The worker's falsifiability answer was PROSE, not command output.** It
described what would happen on revert instead of running it. That is the exact
failure mode the packet was written to prevent, and it is why the packet
demanded pasted output. Do not accept the worker's section 4 as evidence.

**The CTO check that does count.** I extracted the ORIGINAL forgery test from
`bab533e0` -- byte-identical, written by a different worker that never saw this
fix and could not have tuned it -- dropped it into the fixed tree, and ran it:

```
test test_v2_hybrid_envelope_forgeable_without_sender_key ... FAILED
Direct WireEnvelope::V2 primitive decryption failed:
  "decryption failed: aead::Error"
```

The attack that worked three hours ago no longer works. Then the pair that
proves it is not a blanket denial of service:

```
test test_v2_hybrid_envelope_forgery_is_rejected ... ok
test test_v2_hybrid_honest_sender_succeeds ... ok
```

Local three-part gate, run by me: `cargo fmt --all --check` exit 0,
`cargo check --workspace` exit 0 with zero errors and zero warnings,
`cargo check -p scmessenger-wasm --target wasm32-unknown-unknown` exit 0.

**Note the original test dies at Leg 1**, so that run does NOT independently
exercise Leg 2. Ingress verification is covered only by the worker's own test.
State that honestly when this goes to review.

### My own adversarial read of J -- three findings for the auditor

**1. `signing_hash()` is a pure extraction. Leg 2 is backward compatible.**
The hash body is unchanged from the old private `sign()`, so `verify()` accepts
signatures from unfixed peers. **The wire break comes ENTIRELY from Leg 1's KDF
context change.** Leg 2 could ship independently as non-breaking hardening if we
ever need to decouple them.

**2. The static-DH derivation is asymmetric, and this is the real review
question.** Sender computes `DH(ed25519_to_x25519_secret(our_signing_key),
their_bundle.x25519_public)`. Receiver computes `DH(our_x25519_secret,
ed25519_public_to_x25519(sender_bundle.ed25519_public))`. These agree **only if
`our_x25519_secret` corresponds to `bundle.x25519_public`**. The honest-path
test proves it holds in that configuration. Given this repo just ran an identity
unification across flavors, **if any flavor populates `x25519_public`
independently of the Ed25519 key, sessions fail silently.** The bundle already
carries a dedicated X25519 key -- ask the auditor why the sender side derives
from Ed25519 instead of using it.

**3. `verify()` returns `DriftError::IoError` for signature failure.** Minor,
but a forgery attempt will read as an I/O problem in field logs during the very
gate where we are hunting for exactly that. Rename before the run.

**A concern I raised and then withdrew, recorded so nobody re-raises it.** The
diff deletes the Drift encoding path from `encode_envelope`, which the header
described as the primary compact LZ4 format -- it looked like a wire regression
that would show up on BLE as fragmentation noise. It is not. `encode_envelope`
has exactly ONE caller, a test in its own file. The real send path is
`iron_core.rs:834` and `:884`, which build the Drift envelope and call
`to_bytes()` directly with the real signing key. Dead API, no runtime change.
I checked before reporting rather than after.

### Packet K -- storage fail-loud. Committed, CTO verification in flight.

Branch `cto/storage-fail-loud-2026-08-22`, commit `70a00e9d`, pushed.

Unlike J, **this worker actually executed its revert check** -- the run appears
as executed steps in the dispatch log, not as narrative -- and labelled its own
report `VERIFICATION: NONE (verifier must re-run)`. Honest.

Design: `Err(_) => MemoryStorage` replaced with a `DegradedStorage` that fails
every operation loudly, logs `[ERROR]`, and sets `storage_degraded`. Lock
contention (OS 32/33, WouldBlock, sharing violations) is distinguished from
corruption. `initialize_identity()` refuses to mint into RAM. Block checks fail
closed. New accessors: `is_storage_degraded()`, `storage_error()`,
`is_storage_healthy()`, `try_with_storage()`.

### Disk -- the binding constraint all session, and how it was handled

Three reclaims, all of build artifacts only, all after confirming the commits
were durable in the shared object store and nothing tracked lived under
`target/`:

- `_scm_wt/v2forge/target` 4.2 GB, after the forgery proof was pushed
- `_scm_wt/wiring-split/target` 10.2 GB, PR #220 already pushed -- this one was
  an emergency, J's build had taken free space to **2.8 GB**
- `_scm_wt/storefix/target` 6.7 GB, after K was committed and pushed

**Commit and push worker output BEFORE reclaiming anything.** That ordering is
what made all three of these safe rather than a repeat of 2026-08-08.

### Next, in order

1. Adversarial crypto review of J. `core/src/crypto/` is merge-blocked; finding
   2 above is the question that matters most.
2. Operator sign-off on J. Not mine to give.
3. Merge K (not in the blocked perimeter), then J.
4. Tag `v0.4.0-rc.1`. Release machinery is verified ready -- signed AAB + APK
   with the apksigner gate, plus `scm-windows-amd64.exe`.
5. Five-node rollout per `HANDOFF/gpt/CTO_TO_CAO_2026-08-22_FIVE_NODE_ROLLOUT.md`.

## 0-2026-08-22c. P0 CONFIRMED BY EXECUTION -- rollout freeze held

Operator asked to be pushed to the next five-node test, on Windows/Android,
after 0.4.0 and 0.5.0 scope was complete. This section is the honest answer.

### The forgery is real. I ran it myself.

Dispatch I returned. `core/tests/test_v2_hybrid_forgery.rs`
(`cto/v2-forgery-proof-2026-08-22`, commit `bab533e0`) passes. I re-ran it from
a **clean 13m02s rebuild**, so it is not a stale artifact:

```
test test_v2_hybrid_envelope_forgeable_without_sender_key ... ok
test result: ok. 1 passed; 0 failed
```

I read all 334 lines before trusting it. It is NOT one of this project's
self-certifying tests: Alice's private keys are dropped before the attack
begins, the signature is literally `[0u8; 64]`, and the forged envelope goes
through the real `IronCore::receive_message` -- not a helper written beside the
test. Bob accepts it and files it under Alice.

**The P0 ticket's analysis was correct in every particular.** Ticket status
updated from "NOT YET PROVEN" to PROVEN.

### Second P0 promoted from PR #219's diagnosis

`core/src/iron_core.rs:402` `Err(_) => MemoryStorage::new()` is one line with
three consequences: CLI identity churn (proven in the field), the desktop
request-response panic chain, and the fail-CLOSED block checks at `:1179` and
`:3395` silently inverting. PR #219 says so itself and deliberately scopes to
the CLI boundary. The arm itself is now dispatched.

### Dispatched -- `tmp/dispatch/launch7.sh`, strictly sequential

| Packet | Branch / worktree | Scope |
|---|---|---|
| **J** | `cto/v2-sender-auth-2026-08-22` / `_scm_wt/v2auth` | Bind sender static X25519 into the root KDF, new derive_key context, verify signature at ingress. Both legs. |
| **K** | `cto/storage-fail-loud-2026-08-22` / `_scm_wt/storefix` | Stop the silent RAM degradation; prove block checks cannot invert. |

Both packets carry the falsifiability gate explicitly: **"does this test fail if
you revert the fix?" -- run it both ways and paste the output.** Both are DRAFT
PR only. J is inside the merge-blocked crypto perimeter.

Sequential, not parallel: 8.3 GB free and each build costs ~4 GB. The launcher
waits for a free build slot, refuses below 5 GB, and reclaims J's target between
runs only if free drops under 7 GB.

### Why the tag is NOT being cut today

`HANDOFF/plans/PR139_FIVE_NODE_FIELD_GATE_REFERENCE.md` section 1.1, operator
locked: *"any runtime fix creates a new anchor and restarts qualification."*

The J fix changes the handshake KDF context string -- deliberately wire-breaking
so version skew fails closed. Rolling out today therefore guarantees a second
full five-node qualification round. **One round, after the freeze, is cheaper
than two.** That is the recommendation, not a decision I have taken alone; it is
in front of the operator.

### The rollout machinery itself is READY -- verified, and this is good news

`.github/workflows/release.yml` on `main` is complete and correct. A pushed tag
produces:

- signed release **AAB + APK** (all four signing secrets exist in the repo,
  set 2026-08-15), with native-lib verification AND the `apksigner` signature
  gate that catches a `CN=Android Debug` build
- `scm-windows-amd64.exe`, plus linux/macos-amd64/macos-arm64 CLIs
- SHA256 checksums and build provenance per artifact

The gates `daab8a2b` deleted are present on `origin/main`. Nothing needs
building for D2 -- it needs a tag on a SHA we trust.

### Gap list against SHIP_PLAN D1-D7, as of now

| ID | State | Blocker |
|---|---|---|
| D1 main green | **UNKNOWN** | `CI` and `Lint` still QUEUED on `b538f3ba` 26+ min. Runner backlog, not a failure. Re-check. |
| D2 signed APK | Machinery ready | Needs a tag. No `v0.4.0*` tag exists; latest release is v0.1.9 from March. |
| D3 README | Not re-verified this session | |
| D4/D6/D7 field proofs | Blocked | Need the frozen build. |
| D5 no long-lived branch | **DONE** | #139 merged. |

**v0.5.0 scope is not late -- it has not started, by policy.** `SHIP_PLAN.md`
section 4 defers iOS parity until after the 0.4.0 tag. The Apple lane also still
owes written CR1-CR3 answers and has never once supplied an iOS/macOS log across
six operator requests. A five-node gate whose fifth node cannot produce evidence
is a four-node gate described dishonestly. Said plainly to the CAO.

### Written this session

`HANDOFF/gpt/CTO_TO_CAO_2026-08-22_FIVE_NODE_ROLLOUT.md` (`AW-BILAT-0002`) --
tells the Apple lane to build from the tag `v0.4.0-rc.1` and nothing else, why
the freeze is held, the wire-skew warning, the receiver-side-only scoring rules,
and the three things their lane owes. `CTO_TO_CAO.md` deliberately untouched;
it is contested and holds another session's uncommitted edits.

### Operational note

`CARGO_INCREMENTAL=0` on a worktree cargo already built with default settings
invalidates every fingerprint and forces a full rebuild -- 13 minutes and 4.2 GB
for one test. That cost was mine and avoidable. Match the flags the prior build
used, or accept the rebuild knowingly.

## 0-2026-08-22b. STAND-DOWN HANDOFF — read this first

### Landed

**PR #217 MERGED.** `main` is now `b538f3ba`. The CRLF recurrence is fixed at its
root: `.gitattributes` extended 10 -> 91 lines with explicit `binary` rules, and
the one-time `git add --renormalize .` that had never been run. **546 CRLF files
-> 0.** A fresh `git worktree` no longer starts dirty with hundreds of phantom
modifications. 33 CI checks passed, zero failures.

**Set this locally, and tell every contributor to:**

```
git config merge.renormalize true
```

Without it, branches predating `b538f3ba` conflict across the sweep. With it the
cost is zero. Measured overlap before the merge was 6 file-touches across 5 PRs.

### In flight when I stood down

**Dispatch I** (`cto/v2-forgery-proof-2026-08-22`, worktree `_scm_wt/v2forge`)
is running the forgery test for the P0 below. It had not committed. Check
`tmp/dispatch/launch6.log` and the worktree for a commit. **Its result decides
whether the P0 is real.**

### THE IMPORTANT ONE — P0, unproven, needs your judgement

`HANDOFF/todo/P0_V2_HYBRID_HANDSHAKE_HAS_NO_SENDER_AUTHENTICATION_2026-08-22.md`

The V2 hybrid handshake appears to provide **no sender authentication**, on the
default negotiated suite. `init_as_receiver_hybrid` (`core/src/crypto/ratchet.rs:508`)
takes `_our_signing_key` and `_sender_bundle` and reads NEITHER; the root key is
`blake3(ss_hybrid || transcript_hash)`, and no term needs the sender's private
key. `verify_envelope_v2` has only test callers and `core/src/drift/envelope.rs`
has no verify function at all.

If it holds: anyone with two published bundles can send a message that decrypts
cleanly and is attributed to a contact of their choosing.

**This was derived by reading, not execution. Do not act on it until dispatch I
reports.** If the forgery fails, the ticket is wrong and should be corrected
loudly, not quietly deleted.

### PR board

| PR | State | Note |
|---|---|---|
| #217 | **MERGED** | CRLF root fix |
| #220 | Open | Reachability split out of #216. Wiring gate will report **2** findings, not 0 — the passphrase revert re-orphans SecurityUtils. That is correct. |
| #215 | Draft, BLOCKED | Superseded by `cto/routing-peer-seen-v2-2026-08-22`, which is ALSO blocked on the P0 above |
| #216 | Draft, BLOCKED | Passphrase half. Do not re-land without fixing the destroy-on-error recovery path AND the `<device-transfer>` exclusion |
| #219 | Draft, BLOCKED | `verify-signature` fails open; `identity import` blocked on the machine you'd run it on; root cause unfixed |
| #218 | Draft by design | Dead code; the panic analysis is the value |

### Three tests today certified nothing. Assume this failure mode.

- **#215** derived its own expected hint, so it passed while the feature was
  provably broken (`blake3(raw)` vs `blake3(hex)` never matched).
- **#219**'s test calls only `core/` APIs the PR does not touch and never invokes
  the CLI binary. **It passes on `origin/main` unmodified.**
- **#216**'s tests were genuinely good — the exception, worth noting.

Put "does this test fail if you revert the fix?" in every packet. I now do.

### Reviewing a function is not reviewing its dependencies

I read `migrateOrGeneratePassphrase`, confirmed its write -> commit -> read-back
-> verify -> delete ordering, and pronounced it fail-safe. It is, internally. I
never checked that `getEncryptedSharedPreferences` — which supplies the store it
writes into — calls `deleteSharedPreferences` on error. That made the "fix" a
data-loss regression on device transfer. The adversarial review caught it.

### Operational traps that cost real time today

- **`cargo fmt --all` silently no-ops in a git worktree.** Exits 0, writes
  nothing, and a following `--check` reports the same diffs. Use
  `cargo fmt -p <package>`. Cost three round-trips before I spotted it.
- **`cargo test --workspace --no-run` does not fit this disk** (drives
  `target/debug/deps` to 20 GB). Local gate is:
  `cargo fmt --all --check` + `cargo check --workspace` +
  `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown`. Full gate
  belongs on CI.
- **`cargo check --workspace` builds HOST only.** It missed a wasm32 break I
  introduced (`ledger_manager` is `cfg(not(wasm32))`).
- **Disk hit 2.9 GB with a build running.** My watchdog fired 9 times and
  reclaimed nothing, because I had already deleted every PDB and it knew no other
  trick, then it exited silently. Stale worktree `target/` dirs are the big win:
  `_scm_wt/wiring/target` alone was 9.3 GB. A watchdog whose reclaim does nothing
  must escalate, not keep logging.
- **MSYS mangles `/`-leading args to native exes.** `relay -l /ip4/0.0.0.0/tcp/9001`
  became `C:/Program Files/Git/ip4/...`. Prefix `MSYS_NO_PATHCONV=1`.

### Windows relay node

Owned by the CTO seat, running PID from this session, 0 panics, stable identity
`12D3KooWD6vZ...` (unchanged since 2026-08-09). Start it with:

```
MSYS_NO_PATHCONV=1 ./target/release/scmessenger-cli.exe relay -l /ip4/0.0.0.0/tcp/9001
```

It logs self-referential circuits (SELF -> AWS -> SELF) and
`Periodic re-dial: 2555 addresses attempted` — the ghost-ledger accumulation.

### UNCOMMITTED WORK IN THE SHARED CHECKOUT THAT IS NOT MINE — do not discard

At stand-down the shared checkout holds work I did not author and deliberately
left alone. Recording it because uncommitted foreign work is exactly what was
destroyed here on 2026-08-08.

- **`android/.../transport/ble/BleGattClient.kt`** (+7/-4, real change). Someone
  is replacing the hardcoded `WRITE_TYPE_DEFAULT` with a negotiation off the
  advertised characteristic properties (`PROPERTY_WRITE` /
  `PROPERTY_WRITE_NO_RESPONSE`). Looks deliberate and sensible. Not mine, not
  committed, not staged.
- **`HANDOFF/gpt/CTO_TO_CAO.md`** (+20/-133). A concurrent session overwrote the
  working copy to reassert the consensus claim I retracted. **The committed and
  pushed version (`e37b7afd`) carries the retraction and is the CTO position**;
  the superseding record is
  `HANDOFF/gpt/CTO_TO_CAO_2026-08-22_IDENTITY_CHURN.md`. I left their working-tree
  edit untouched rather than undo it.
- `.claude/hooks/preflight_guard.py` (+26) is MY hook fix, already merged to main
  via #217. It shows as modified only because this branch has not merged main
  yet. Redundant, harmless, will vanish on the next merge.

### Not done, deliberately

- **`orchestrator_guard` CONTROLLER scope was NOT widened.** A CTO widening the
  role that constrains the CTO is the pattern that control exists to resist.
  Operator decision.
- Worktrees under `_scm_wt/` are left in place; several hold uncommitted CR-noise
  that is not real work. `_scm_wt/wiring/target` was deleted to reclaim disk.

## 0-2026-08-22. SESSION RECORD — audit of the dead Antigravity session

A 13-hour Antigravity session (id `2224b573`, 2,569 steps, 46 operator requests)
died on a dropped stream, not a rate limit, three seconds after a green
`assembleDebug`. Its last claims are therefore unverified, not merely unfinished.

### Retraction — there is no evidenced bilateral consensus

That session wrote a formal `[OK-PLAN-ACK]` into `HANDOFF/gpt/CTO_TO_CAO.md`
citing a reference document. All three elements fail verification: commit
`0dc1f357` is not a valid object; `FIVENODE_CONSENSUS_PLAN_2026-08-21.md` exists
in no ref at any point in history; PR #208 is real but is the Apple lane's 4-node
parity status doc. The operator cleared both lanes to auto-proceed on the
strength of that claim. **That file has been rewritten to retract it.** Do not
treat the prior ack as authority.

### Defect status, verified against `daab8a2b` — the B1-B5 worklist was stale

| ID | Status | Evidence |
|---|---|---|
| B1 mDNS self-peer guard | `[OK]` committed | `fd7655fa`; `MdnsServiceDiscovery.kt:211-212` |
| B2 outbox attempt cap | `[WARNING]` working tree only | `pendingOutboxMaxAttempts` still has 4 refs in HEAD |
| B3 receipt convergence | `[OK]` committed | `4083e59b`; `iron_core.rs:3460`, gated `Delivered\|Read` |
| B4 `routing_peer_seen` | `[OK]` fixed 2026-08-22, unmerged | branch `cto/routing-peer-seen-2026-08-22` |
| B5 ledger → Android cellular | `[WARNING]` working tree only | `iron_core.rs` +15, `MeshRepository.kt` +225 |

W1/W2/W3 from the 2026-08-21 plan were already complete when it was written.

### RELEASE BLOCKER — `daab8a2b` silently deleted three CI gates

That commit's whole message is one line about identity unification, no body. It
also removed: **Verify release APK signature** (`release.yml`) — the check that
fails a build signed `CN=Android Debug`; **Android Wiring Gate** (`mobile.yml`),
while `scripts/check_wiring.py` still exists; and **Windows CLI Artifact**
(`ci.yml`), reverting PR #203 which had merged minutes earlier.

`origin/main` retains all three. **Resolve the PR #209 workflow conflicts by
taking main's side** — that restores every gate without reverting anyone's work.
Conflicts are confined to `mobile.yml` and `release.yml`; no source conflicts.

### The request-response panic — my earlier framing was WRONG

`HANDOFF/archive/P0_REQUEST_RESPONSE_PANIC_KILLS_DESKTOP_ON_MESH_GROWTH_2026-08-09.md`
opens by describing a 4-node-convergence panic. **Its own later updates
contradict that** and I initially propagated the wrong version. Corrected:

- Reproduced **3/3 with a single peer** (iPhone), fastest in **62 s**. It needs
  one chatty peer, not a fleet.
- Signature is 4-6 simultaneous connections to ONE peer, several to the
  byte-identical multiaddr within 30 ms.
- Trigger chain: identity churn → ghost ledger entries → dial dedup misses →
  concurrent connections to one endpoint → connection-map drift → assertion.
- **The PR #144 address-level dial dedup did NOT fix it** — reproduced with the
  fix in, and the connection count rose from 4 to 6. Negative result, recorded.

**Most actionable finding:** the panic is `debug_assert_eq!` at
`libp2p-request-response-0.29.0/src/lib.rs:678`. This workspace's
`[profile.release]` does not set `debug-assertions`, so it defaults OFF and the
assertion is **compiled out of release builds**. Every panicking run in the
ticket was a debug build. Running the field test on release binaries very likely
removes this specific process death.

Caveat, do not overstate: two `.expect()` calls at lines ~670 and ~676 panic in
BOTH profiles on a genuinely inconsistent connection map, and suppressing the
assert does not repair the drift. Release changes the odds; it is not a fix.

### Dispatches 2026-08-22 (free agy Gemini lane, isolated worktrees, unpushed)

**B — `cto/routing-peer-seen-2026-08-22` — ACCEPT (pending audit gate).**
Nine real production call sites wired in `iron_core.rs` (TCP, BLE, relay
ingress) plus a falsifiability test that establishes the StoreAndCarry/0.0
baseline, then proves Direct/BLE, switch to TCP with BLE preserved in
`alternatives`, and failover back. Touches merge-blocked `core/src/routing/` —
needs `crypto-security-auditor` before merge.

**A — `cto/panic-self-circuit-guard-2026-08-22` — REJECT as incomplete.**
The self-circuit guard in `addr_filter.rs` is correct and well-tested but has
**zero callers** — dead code that changes no runtime behaviour, the exact B4
pathology. My packet caused this by scoping the writable path to
`addr_filter.rs` alone when the call sites live elsewhere. The worker's written
analysis was excellent and honest (correctly labelled it a mitigation, and it
was the source that caught my 4-node error). Re-dispatch with the wiring path
in scope — and note the guard does not address the real trigger anyway.

### Environment

**`cargo test --workspace --no-run` DOES NOT FIT ON THIS MACHINE.** It filled
C: twice in one session. The second run went from 15 GB free to **7.3 MB
(100% used)** because `target/debug/deps` alone reaches **20 GB** — many
integration-test binaries, each linked at `opt-level = 3` with debuginfo.

Both failures were disk, and both disguised themselves as code errors:
- run 1: `can't find crate for scmessenger_core`, `found invalid metadata files
  for crate serde`
- run 2: `error: linking with 'link.exe' failed: exit code: 1318` (also 1140,
  1180) — no "no space" text anywhere

`cargo check --workspace` passed clean (0 errors, 0 warnings) on the same tree
both times. **Use `cargo check --workspace` plus a scoped
`cargo test -p <crate> --lib --no-run` as the local gate, and let CI run the
full workspace gate on a clean runner.** Wrap any long local build in a
watchdog that kills cargo below ~3 GB free; a 100%-full C: destabilises Windows.

`scripts/clean_target.sh --deps` reclaimed 19.6 GB cleanly and preserved built
binaries and `core/target/generated-sources`. A cheaper first move exists and
was skipped this session: `find target/debug -name "*.pdb" -delete` frees
comparable space without dirtying cargo fingerprints, so nothing recompiles.

Current: 20 GB free / 92%. The four Android target triples were reclaimed
earlier via `--triples` and will be rebuilt by cargo-ndk on the next Gradle
run (~15-20 min).

Git Bash `date` on this machine is ~7.5 h off Windows local time. Use PowerShell
`Get-Date` for anything time-sensitive.

### ADVERSARIAL REVIEW BLOCKED #215 AND #216 — read before touching either

The mandated review returned **BLOCK** on both. Three critical findings were
re-verified by the CTO afterwards and all three held.

**#215 does not do what it claims.** The routing hint preimages do not match:

| site | preimage |
|---|---|
| `optimized_engine.rs:390` (`peer_seen`) | `blake3(&peer_id)` — raw `[u8; 32]` |
| `iron_core.rs:938` (send path) | `blake3(recipient_id.as_bytes())` — 64-char **hex string** |

`blake3(raw) != blake3(ascii_hex)`. The hint registered on sighting can never be
found by the send path, so transport failover is NOT restored on the IronCore
path. Only `swarm.rs:5690` uses a matching derivation. The test passes because it
derives its hint the same way `peer_seen` does — it certifies the code it was
written beside, not the code that ships. **The PR description asserting this
fixed the operator's failover symptom was wrong and has been retracted.**

Also CRITICAL: the sighting is keyed on `sender_public_key_hex`, which is NOT
authenticated on the V1 non-ratcheted branch — `decrypt_message` uses it only as
AAD, which binds it to the ciphertext without proving key possession, and branch
selection is made from attacker-controlled fields. Reachable from raw BLE GATT
bytes (`mobile_bridge.rs:1443`) and any libp2p peer (`swarm.rs:3596`). Attack
surface shipped without the benefit.

**#216's security fix is a DATA-LOSS REGRESSION.** This one is a CTO validation
error worth recording precisely: I read `migrateOrGeneratePassphrase`, confirmed
its write -> commit -> read-back -> verify -> delete ordering, and called it
fail-safe. It is — *internally*. **I never checked the durability of the store it
writes into.**

`SecurityUtils.getEncryptedSharedPreferences()` uses
`context.deleteSharedPreferences(...)` as its KeyStore recovery path, and
`scmessenger_secure_prefs.xml` is NOT excluded from `<device-transfer>` in
`data_extraction_rules.xml` (verified: 0 matches). So: user transfers to a new
phone -> prefs arrive, hardware master key does not -> decrypt throws -> store is
DELETED -> fresh passphrase generated -> identity backup permanently orphaned.
**The current plaintext build survives that transfer; the "fix" breaks it.**

**Lesson: validating a function is not validating its dependencies.** The
migration logic was sound and the surrounding store was not.

**`iron_core.rs:402` is worse than previously recorded.** The
`Err(_) => MemoryStorage::new()` arm shares its backend with `blocked_manager`,
so the deliberately fail-CLOSED block checks at `:1179` and `:3395` silently
become fail-OPEN whenever storage degrades. Separate P0; not a merge blocker on
its own, but it must NOT be cited to wave through #216, which adds a second sled
open on the same path.

### Open PRs from this session — nothing is only-local any more

| PR | Branch | State |
|---|---|---|
| [#215](https://github.com/Sovereign-Communication/SCMessenger/pull/215) | `cto/routing-peer-seen-2026-08-22` | B4 fixed and CTO-verified. **Needs `crypto-security-auditor`** — touches `core/src/routing/`. |
| [#216](https://github.com/Sovereign-Communication/SCMessenger/pull/216) | `cto/android-wiring-restore-2026-08-22` | Wiring 32 -> 9. Stacked on `feat/identity-id-unification`. Packet F is closing the last 9. |
| [#217](https://github.com/Sovereign-Communication/SCMessenger/pull/217) | `cto/crlf-root-fix-2026-08-22` | `.gitattributes` rules only. **The `git add --renormalize .` sweep is deliberately NOT included** — 550 files, and the emoji hook blocks it on 21 pre-existing files without `--no-verify`. Operator decision. |
| [#218](https://github.com/Sovereign-Communication/SCMessenger/pull/218) | `cto/panic-self-circuit-guard-2026-08-22` | DRAFT. Code is dead (zero callers) — opened to preserve the panic analysis, which is the valuable part. Do not merge until wired. |
| [#209](https://github.com/Sovereign-Communication/SCMessenger/pull/209) | `feat/identity-id-unification` | MERGEABLE. Was CONFLICTING. |

### THE LOCAL GATE IS NOT `cargo check` — it missed two real breaks today

Both from my own commit `c8a758d5`, both caught only by CI:

- `cargo fmt --check` — over-long iterator chain (fixed `d665979b`)
- **wasm32** — `ledger_manager` is `cfg(not(wasm32))`; the new lookup used it
  unconditionally and broke the WASM lane (fixed `bf6ca5f0`)

`cargo check --workspace` builds the HOST target only. Use all three:

```
cargo fmt --all --check
cargo check --workspace
cargo check -p scmessenger-wasm --target wasm32-unknown-unknown
```

### Restoring the gates keeps surfacing real debt

Every gate restored in `b4ba61f3` went red on pre-existing problems, because
PR #209 had been BLOCKED as CONFLICTING and the suite had never run:
Repository Hygiene on 1,098 untouched lines, `cargo fmt` on 3 files I never
edited, and the Wiring Gate on 32 findings covering nine unreachable features.
None of that was new breakage. It had simply stopped being reported.

### Still open

- Phase 0 working-tree commit (B2/B5, router timeout 100ms→500ms revert, blank
  BLE target filtering) — gate re-run then commit. Stage explicit paths only.
- PR #209 rebase, taking main's side on the two workflow files.
- Apple/CAO lane owes written CR1-CR3 answers and has never supplied iOS/macOS
  logs despite six operator requests across the dead session.
- Five of the six mis-filed P0 tickets in `POST_TAG_QUEUE.md` §2 are still
  sitting in `HANDOFF/archive/` undispositioned.

> **2026-08-16 — READ `HANDOFF/CTO_DISPATCH_PLAN_2026-08-16.md` FIRST.**
> #167, #168, #169 and #165 are **merged to tracking**. The lane picture in §3
> below **inverted** since it was written: `Mobile`/KSP UniFFI is now GREEN and
> `Test` went RED on two transport tests. The dispatch plan carries the
> re-derived table, the verified merge mechanics, and the routing plan.
> Sections §1, §4, §5, §6, §7 and §8 of this file remain accurate.

## 0. STANDING RULE — keep this file current

**Update this file at the END of every session, and immediately on any
important change.** Operator directive, 2026-08-16, standing.

"Important" means: a merge or close, a gate result, a decision made or reversed,
a blocker found or cleared, a claim in here proven wrong. Do not batch these to
the end — a session that dies mid-run leaves the next one reading fiction.

When a section here is overtaken by events, **mark it superseded and say what
replaced it. Do not delete it.** The history of a wrong call is how the next
session avoids re-making it; every §8 lesson exists because someone deleted the
context instead of the conclusion.

## 0a. HANDOFF -- 2026-08-19. Read this first, then sections 0b/0c.

**D1 and D5 are DONE.** PR #139 merged to `main` at `6e70a3db`; `tracking` fully
absorbed (`git rev-list --left-right --count origin/main...origin/tracking` -> `1 0`).
All main lanes green. `main` is now `b4ccd30a`. Docker Publish fired: image
**`sha-6e70a3d`** exists, which unblocks the D4 node rebuild.

### Open PRs -- merge order

| PR | What | Gate |
|---|---|---|
| **#180** | DUAL_BIND fix (operator-approved: advertise only what bound) | **needs a fresh CRITICAL_VALIDATOR** -- touches `core/src/transport/`, rule 8 |
| **#179** | field evidence, AGENTS.md **rule 16**, wiring audit, `check_wiring.py` (5/5 tests, verified by CTO), zai lane | ready |
| **#177** | P0 dispositions | **NEEDS CORRECTION -- see below** |
| #178, #170, #156, #154 | Apple API limit, free lanes, docker non-blocking, APK signing verify | #154 must merge before the tag |

### TAG-BLOCKING work not yet started

**Nine Android features are wired out** -- implementation present, call sites
absent (`ebf5411b` restored files but not their callers). Full list in
`docs/fieldtest/ANDROID_WIRING_AUDIT_2026-08-18.md`. Worst three:

1. **Diagnostics/logs viewer** -- `Screen.Diagnostics` is DEFINED (`MeshApp.kt:397`)
   and NAVIGATED TO (`:287`) with **no `composable()` registration**. Lands nowhere.
2. **QR APK sharing** -- `ApkShareDialog.kt:36`, zero callers. Damages **D2**.
3. **QR join-mesh** -- `JoinMeshScreen.kt:49`, never in the NavHost.

A restoration PR is still to be written. Verify each with
`python scripts/check_wiring.py` (exit 1 = findings), **not by eye**.

### CTO error to correct in #177

`NO_MOBILE_BOOTSTRAP` was dispositioned to S4 **because** `JoinMeshScreen`
supplied a working QR join path. **It is orphaned.** That reasoning is void, so
re-open the ruling. Do not let the wrong justification stand.

### Operator rulings, 2026-08-19

- **WS deferred** to unblock Android; returns **before v1.0.0**. #180 emits TCP
  only. Cost: browser/WASM peers have no transport. Recorded in the field doc.
- **zai `glm-4.7-flash` is the primary free lane.** MANDATORY quirk: send
  `"thinking":{"type":"disabled"}` or it returns `content:""` -- a silent
  vacuous success. Qwen paid remains **off limits**.

### Field state -- rollout on real hardware

Windows node + Pixel 6a both run `b4ccd30a`. **#176 verified live**:
`pm query-activities -a VIEW -d scmessenger://invite` resolves to MainActivity.
**Messaging does NOT work between them**: 14,496 x `Failed to negotiate transport
protocol(s)`, 13 peers marked dead, 0 peers discovered. That is DUAL_BIND, and
#180 is the fix awaiting review. The in-app message to the operator is still
**queued, never delivered** -- do not claim otherwise.

### Traps that cost time this session

- `adb logcat` main buffer hides crashes. Use **`adb logcat -b crash`** first.
  Diagnosing without it produced a confident wrong answer (blamed memory).
- Do **not** set `CARGO_TARGET_DIR` for the Android gradle build. jniLibs come
  from `core/target/android-libs`; overriding it ships an APK with **no**
  `libscmessenger_core.so` and gradle still says BUILD SUCCESSFUL.
- `git show <rev>:.dotted/path` needs `MSYS_NO_PATHCONV=1` -- fails as
  plausible emptiness.
- agy has a **~90s per-tool timeout** separate from `--print-timeout`. Never ask
  it to run a cold full build.

## 0b. OPERATOR APPROVAL GATE — standing, 2026-08-16

### Who may merge — role-bound, not negotiable

| Role | May merge? |
|---|---|
| **CTO** (this seat) | **YES** — under the confidence test below. Merging is a CTO decision |
| ORCHESTRATOR / CONTROLLER | **NO.** Coordinates, dispatches, integrates verified output. Never merges |
| Worker lanes — SCANNER, IMPLEMENTER, VALIDATOR, agy, any HTTP lane | **NEVER.** They open PRs and report. A worker holding merge rights defeats every gate above it |
| OPERATOR | Always, and overrides this table |

A worker asking the CTO to merge is a dispatch event, not permission. A green
gate is not approval either: `pr_scope.sh` exiting 0 means no reason was
*found*.

### The confidence test — deterministic, run it in order

Before any **irrevocable or potentially destructive** action:

1. **Am I at 100%?** All five must hold, or the answer is no:
   - every gate green, or every blocker named out loud with evidence answering it
   - every required review exists as a **durable artifact**, not a recollection
   - **zero UNKNOWNs.** Undetermined is never treated as safe
   - the blast radius is bounded and I can state it
   - I verified the load-bearing claim myself, with a command, this session
2. **At 100%** -> execute. Do not ask. Sequencing is the CTO's to own.
3. **Below 100%** -> confer with the CEO session. Reach 100% or consensus, then
   execute.
4. **CEO and CTO cannot both reach 100%** -> escalate to the operator with both
   positions stated. Do not split the difference.

"I think it's fine" is below 100%. "The worker said so" is below 100%. "It
passed CI" alone is below 100% when a review artifact is also required.

### Blast radius — only as big as it needs to be

**Keep the blast radius only as big as it needs to be, within the constraints
currently available.**

Both halves bind. Minimise scope, sequence so a failure is small and
attributable, prefer several small merges to one large one, and never inflate a
change already in flight with unrelated work. But minimise **within what is
actually achievable now** — #139 is 204 commits because `tracking` is the
long-lived integration branch and collapsing that is not available today. The
rule asks for the smallest radius reachable, not an impossible one.

Worked example, 2026-08-16: #174 (required for D1) merged alone, while #171 and
#173 — tooling with no bearing on D1 — were held until after the trunk merge.
Batching them would have saved two ~50 minute CI cycles and inflated the largest
PR in the repo. Wall clock was the cheaper thing to spend.

---

**The test for gating is DESTRUCTIVENESS, not whether it writes.** Operator
directive, standing, refined 2026-08-16.

The operator's reasoning, which is the rule: *opening a PR "isn't destructive,
and really only helps to safely preserve data, as it offers a place to track the
changes."* Gating work-preservation strands work in worktrees — which is how
this repo has lost things. We are moving to **small, frequent PRs**; a
200-commit PR is what made per-merge buyoff necessary, and that is going away.

| Operator approval FIRST (irreversible, or outside CTO authority) | CTO executes at 100% confidence | Proceed freely (preserves work, or read-only) |
|---|---|---|
| Tags, releases, branch protection | **Merging** (see the confidence test) | Reading anything; read-only git (`log`/`diff`/`merge-tree`/`rev-list`) |
| Force-push, history rewrite | Closing/reopening a PR | **Opening a PR. PR comments. PR body/title updates.** |
| Deleting a branch or worktree registration | Reclaiming `target/` in a SAFE worktree | Committing and pushing to **your own** branch |
| Anything touching the shared checkout's working tree | Pushing to a shared branch you own | Writing files in **your own** worktree |
| Deleting files or worktrees outside a SAFE `target/` | Dispatching an IMPLEMENTER into an isolated worktree | `pr_scope.sh`, `gh pr checks`, CI logs |
| | | Compile/test verification (deconflict builds first) |
| | | Dispatching read-only SCANNER / VALIDATOR |
| | | `tmp/` scratch; reporting findings and recommendations |

Investigation is not a change. Verification is not a change. Preserving work in
a tracked place is not a change. **Destroying, discarding, or releasing is.**

Two calls the CTO made by inference rather than instruction — correct them if
wrong: **closing** a PR is treated as gated (it discards rather than preserves,
even though it is reopenable), and **IMPLEMENTER dispatch is free when
isolated**, because an isolated writer produces a branch and a PR, which is
preservation. An implementer that would touch the shared checkout or a shared
branch is gated.

A green gate is still not approval: `pr_scope.sh` exiting 0 means no reason was
*found*, not that the operator said yes.

Present the evidence, state the recommendation, then wait. A green gate is not
approval; `pr_scope.sh` exiting 0 means no reason was *found*, not that the
operator said yes.

## 0c. The verification loop — keep this shape

This is the loop that caught a CRITICAL-adjacent defect on 2026-08-16 after the
CTO had already talked himself into "it looks fixed". Do not shorten it.

1. **The controller never self-certifies.** Reading the code and concluding it
   is fine is a *claim*, not a review. `docs/ORCHESTRATION.md` forbids the
   controller from making that call and AGENTS.md rule 8 requires an
   independent sign-off. The CTO read `swarm.rs`, saw the guardrail call, and
   declared W1 fixed. An independent validator found the cooldown was erased by
   `forget_peer` on full disconnect. **The gate exists for the person running
   it, not just for other people.**
2. **Frame the packet adversarially.** Hand the worker your reading as *a claim
   to falsify*, in those words: "If you merely agree with it, this review has no
   value." A packet that asks for confirmation gets confirmation.
3. **Spot-check what comes back.** A delegated verification is still a claim.
   Verify the load-bearing assertion with your own command — not the whole
   report, just the one thing the verdict rests on.
4. **Expect corrections in both directions.** On 2026-08-16 workers corrected
   the CTO twice (the #164/#169 renormalization claim; W1), and the CTO
   corrected workers twice (a `git diff -w` cited as empty when blank lines
   survive it; a "270 occurrences" census that counted argv unpacking). Neither
   side is the authority. The command output is.
5. **Prefer UNCERTAIN to a clean answer.** Tell workers so explicitly. This gate
   already produced one false "[OK] clear" while six gated files were invisible.
6. **Artifacts, not chat.** Verdicts go to `docs/security/` or the PR. A review
   that exists only in a session transcript did not happen — and untracked work
   in this shared checkout has been destroyed before, so commit it.

7. **A REPEAT MISTAKE IS A PROCESS DEFECT, NOT A MEMORY LAPSE.** Operator
   directive, standing 2026-08-16. The second occurrence of any mistake stops
   being about the mistake and becomes about the process that failed to catch
   it. When one happens: run an RCA, then **change the mechanism** — do not
   write a better reminder.

   **The governing finding: a lesson stored as prose gets re-learned; a lesson
   stored as a gate does not.**

   Evidence, from this repo. Rule 14 has an executable form (`pr_scope.sh`) and
   has held. Rule 13 has none. The trailing-whitespace lesson had none — it sat
   in §8 as prose, the CTO **quoted it earlier the same day**, and then committed
   a worker-produced artifact verbatim and turned `Repository Hygiene` red on the
   trunk merge for the second time, in the same lane, on the same PR (#174).
   Re-reading was never going to be the fix.

   So the RCA question is never "why did I forget?" It is **"what gate was
   missing, and can I build it?"** If a gate genuinely cannot exist, say so
   explicitly and accept the residual risk in writing — that is a decision, not
   an oversight.

   Corollary for delegation: **worker-produced files are not exempt from the
   repo's gates.** Worker *code* gets compiled and reviewed; worker *artifacts*
   — markdown, reports, docs — were being committed on sight. Run
   `git diff --cached --check` (and the emoji check) against anything a worker
   generated, before committing it.

Mechanics that keep dispatch healthy: isolated worktree per writer, never the
shared checkout; deconflict builds (`tasklist` for cargo/gradle/java) before any
dispatch that builds; a distinct log-dir per concurrent `agy_run.sh` or two runs
on the same model and SHA silently overwrite each other; `--add-dir` and an
exact `--model` always; 30m+ timeouts, and a transient `error_message` mid-run
is not a failure — check whether it recovered before re-dispatching.

### Seat status

**2026-08-16: this is the ONLY live CTO session.** The other sessions listed by
`mcp__ccd_session_mgmt__list_sessions` as `isRunning: true` — "Cto resume v040",
"Scm cto 1000 hst" — are **stale processes, not active seats** (operator
confirmed). The §8 "one CTO seat" caution stands for the future, but it is
resolved for now: no need to re-establish the seat before merging.

### Session log — 2026-08-16

| Change | Evidence |
|---|---|
| #167, #168, #169, #165 merged to tracking | `manager.rs:470` carries `saturating_sub`; `.gitattributes` carries `*.kt`/`*.kts`/`*.md eol=lf` |
| `Repository Hygiene` and `Rust Linting` went GREEN on #139 | 11s / 4m32s, confirmed from the check list |
| **#152 CLOSED** | Audited (`CTO-152-AUDIT`), then verified independently: whitespace + blank-line movement only; conflicts on `MeshApplication.kt`, which tracking superseded via `17216e1a`/`149d3725`. Nothing lost. Evidence on the PR |
| **#171 opened, HELD** | `pr_scope.sh` no-truncation + AGENTS.md rule 15. Independently validated (`CTO-171-VALIDATE`): APPROVE, R3 fails-closed verified. Held until #139 lands so it does not restart the trunk merge's CI |
| AGENTS.md **rule 15** added | No renumbering: all existing citations (rules 1,2,5,8,9,11,12,13,14) still resolve. Coherence audit dispatched as `CTO-AGENTS15-COHERENCE` |
| CEO escalation sent | README honest-first framing; dependency-deferral trigger |
| **The #139 crypto gate was found NOT satisfied** | Last recorded verdict was BLOCK. See the correction banner on §4 |
| **W1 found still live**, refuting the CTO's own reading | `CTO-139-CRYPTO-REVERIFY` -> `docs/security/PR139_REVERIFY_2026-08-16.md`. F1 FIXED, everything else CLEAN, W1 NOT FIXED |
| **W1 fixed** -> PR #172 | `forget_peer` removed with both call sites (native `:5397`, WASM `:7736`). Independently validated: `docs/security/W1_FIX_VALIDATION_2026-08-16.md` -- APPROVE_WITH_FINDINGS, W1 CLOSED, REGRESSION RISK NONE |
| W1 fix gates pass | target test 1 passed/0 failed; `cargo test --workspace --no-run` **CARGO_EXIT=0**, zero errors |
| **19 GB reclaimed** | 76 PDB files in `.scm-shared-target/debug/deps` held 19 GB of 27 GB. Disk 100% -> 92% |
| Operator approved the merge sequence | #172 -> tracking, then #139 -> main. Stop before branch protection and the tag |

### Open findings — not yet fixed, safe to dispatch

1. **Preflight guard false positive.** It blocks read-only commands whose *file
   path* merely contains `agy` — no dispatch involved. #167 fixed this class for
   git commands; still open for others. Do NOT reach for
   `SCM_SKIP_DISPATCH_CHECK=1` to read a log; use Read/Grep instead.
2. **`agy_run.sh` log collision.** `RAW="$LOGDIR/agy_${MODEL}_${STAMP}.jsonl"` is
   model + HEAD SHA, so two concurrent dispatches on the same model write the
   same file. Pass a distinct 4th arg (log-dir) per dispatch. Same class as the
   known `delegate_task.py` collision.
3. **145 `.md` files pending renormalization** — see §9 of the dispatch plan.
   Held; collides with #139, which touches 91 `.md` files.

4. **Rule 15 propagation backlog — the REAL list.** `CTO-AGENTS15-COHERENCE`
   returned "270 occurrences across 74 files". **That number is wrong; do not
   dispatch against it.** It pattern-matched `head`/`tail`/`[:N]`
   indiscriminately and counted argv unpacking (`sys.argv[1:4]`), display
   formatting (`pid[:22]`), deliberate single-value extraction (`adb version |
   head -1`), and even the comment lines in `pr_scope.sh` that *describe* the
   fix. It also under-reported, because the packet scoped the search to
   `scripts/` and `.claude/hooks/` and therefore missed `.codex/`.

   The genuine defect is narrow: **a list of violations, errors, or findings
   shown to a decision-maker, silently cut short.** Verified sites:

   | File:line | Truncation | Hides |
   |---|---|---|
   | `scripts/verify_all_builds.sh:24,31,38` | `tail -5` | clippy / gradle / iOS **build failure output** |
   | `scripts/verify_incremental_gate.py` (x5) | `stderr[:1000]` | compiler errors |
   | `scripts/verify_delivery_state_monotonicity.sh:64` | `regressions[:10]` | delivery-state regressions |
   | `scripts/verify_swift_violations.py:40` | `bad[:15]` | Swift violations |
   | `.claude/hooks/preflight_guard.py:571` | `risky[:4]` | risky ops shown before a destructive command |
   | `.claude/hooks/check_no_emoji.py:45` | `matches[:10]` | emoji violations |
   | `.codex/hooks/check_no_emoji.py:45` | `matches[:10]` | same file, second copy |
   | `scripts/rules_check.py:78` | `hits[:8]` | rule violations |
   | `scripts/repo_audit.sh:27` | `head -n 200` | audit hits |
   | `scripts/triage_lane.sh:72` | `tail -25` | `git diff --stat` between pass and fail |
   | `scripts/apply_branch_protection.sh:77,80` | `head -40` / `head -20` | branch-protection API state |

   `verify_all_builds.sh` is the worst of these: a script whose entire job is to
   prove the build is good shows only the last 5 lines when it is not.

   **Lesson for future dispatches:** a grep-shaped question returns
   grep-shaped answers. Ask for the *semantic* defect and require the worker to
   justify each hit, or budget for the CTO to sort the census by hand.

5. **W1 regression protection is thin** (from `W1_FIX_VALIDATION_2026-08-16.md`
   V5, non-blocking). With `forget_peer` gone the unit test cannot exercise the
   disconnect path, so it simulates the scenario in a comment. A future refactor
   could reintroduce a map-clearing call in `start_swarm_with_config` and the
   test would still pass. Closing it needs an integration test driving
   `SwarmEvent::ConnectionEstablished`/`ConnectionClosed` and asserting no
   redundant `LedgerExchangeRequest`. Does not affect current correctness.

6. ~~**`git merge-base --is-ancestor` disagreed with reality.**~~
   **DIAGNOSED AND FIXED 2026-08-16 (PR #173).**

   Root cause: `--is-ancestor` returns **0 = merged, 1 = NOT merged, 128 = REF
   ERROR**. The check collapsed non-zero to "not merged". PR #165 merged at
   16:53Z, GitHub deleted the branch, `git fetch --prune` dropped the local ref,
   and the check then returned **128** -- reported as "not merged".

   That is a **permanent** false negative: the ref never returns, so a
   merged-and-pruned worktree could never be reclaimed. It failed safe, but a
   gate that can never open is still broken.

   Fixed in #173 via `scripts/reclaim_safe.py`: 128 yields **UNKNOWN**, never
   "no" and never SAFE, with a PR-state fallback when the ref is gone. Verdict
   requires all three of clean + zero unpushed + merged. `reap_worktrees.sh`
   carried the same bug and is updated.

   **Verified reclaim survey (24 worktrees): 14 SAFE, 8 HOLD, 1 PATH-GONE,
   0 unpushed anywhere.** PR #165 confirmed MERGED (`81a4bbd2`). 5 GB reclaimed
   from `scm-android-gate` and `scm-fix-transport-defects`; source trees intact.

   Still open from this: **`e01c-pq-mixing` is registered in `git worktree list`
   but absent from disk**, so that list is not a trustworthy inventory. Needs
   `git worktree prune` -- not run, it deletes.

7. **`LNK1318: Unexpected PDB error; LIMIT` is disk exhaustion**, not
   corruption. Observed at 963 MB free / 100% full mid-link. Add it to the
   CLAUDE.md list beside `STATUS_STACK_BUFFER_OVERRUN` and "can't find crate".
   The reclaim that fixes it: delete `*.pdb` in
   `$CARGO_TARGET_DIR/debug/deps` -- 76 files held 19 GB. Debug symbols only,
   regenerated on link. **Never** touch `core/target/generated-sources/`.

8. **`scripts/clean_target.sh --dry-run` is not a real flag.** CLAUDE.md's
   routing table instructs running it before deleting artifacts; the script does
   not implement it and just prints usage. The documented safety step does not
   do what the doc says. Real modes are `--triples` and `--deps`.

9. **agy has a ~90s PER-TOOL timeout, separate from `--print-timeout`.** A
   worker with `--print-timeout 45m` still died polling a cold `cargo` build in
   90-second slices for 20 minutes -- and the build had actually SUCCEEDED. Do
   not ask an agy worker to run a cold full build: have it make the change and
   commit, then run the compile gate separately.

Everything below has a command next to it. **Re-derive before acting** — this
file ages, the repo does not.

---

## 1. The goal

Ship **v0.4.0 as an Android beta** the operator can hand to friends and family.
Then v0.5.0 iOS. `SHIP_PLAN.md` D1-D5 is the definition of done and the only
execution queue until the tag. **Nothing in v0.5.0/v1.0.0 scope starts before the
0.4.0 tag.**

Latest thing a stranger can download is **v0.1.9, from 2026-03-19.** That number
is the whole problem.

### Exit criteria status

| | Criterion | State |
|---|---|---|
| **D1** | `main` is green | **Blocked on #139.** Every red lane is explained; see §3 |
| **D2** | Signed APK downloadable | **Unblocked** — all four signing secrets set 2026-08-15 17:08Z. Needs the tag |
| **D3** | README explains the product | **DONE** |
| **D4** | Two-device message + receipt | Blocked on D2 and a node rebuild; see §5 |
| **D5** | No long-lived integration branch | **Blocked on #139** (merging it satisfies this) |

---

## 2. Where the merge train stands

**Merged to `tracking` this sprint (11):** #149 UniFFI build fix + 7 restored
Android sources + orchestration tooling; #150 lane routing; #153 backlog amnesty
87→8; #157 circuit-relay transport prefix; #158 pr_scope truncation fix; #159 D4
runbook; #160 release notes; #161 dead-IP retirement; #162 five-layer integration
suites; #163 shared cargo cache docs; #164 hygiene; #146 Android durable delivery.

**Closed:** #147, with proof — `git log --oneline 7538e4e9 --not origin/tracking`
returned zero commits. It was a branch cut from `tracking` but aimed at `main`,
so GitHub rendered the whole tracking-vs-main delta as its own 102-file diff.
**Read the ancestry, not the diff stat.**

### Open right now

```
gh pr list --limit 20
gh pr checks 139 ; gh pr checks 165
bash scripts/pr_scope.sh 139        # the REPAIRED gate -- see §6
```

| PR | Base | What it is |
|---|---|---|
| **#139** | main ← tracking | **THE TRUNK MERGE. D1 + D5 together.** Checks re-triggered 2026-08-16 after the four merges below |
| #152 | main | Hygiene whitespace. **CONFLICTS with tracking** and is probably obsolete after #164/#169. Verify after #139; do not close blind |
| #154 | main | `apksigner verify` guard. **Merge this before tagging** — see §5 |
| #156 | main | Docker Integration Suite non-blocking + issue #155 |
| #170 | main | Free-lane orchestration tooling. Its red `Lint` is `core/src/lib.rs:159` **inherited from main** — it self-clears when #139 lands |
| 13 dependabot | main | **DEFER all, close none.** They are the post-tag S4 queue. GitHub reports 7 vulnerabilities on the default branch, 3 high — real, but not before the tag |

**Merged to tracking 2026-08-16 (4):** #167 dispatch-guard false positives; #168
stale-checkout gate + dispatch timeout floor; #169 `.gitattributes` eol=lf +
whitespace/rustfmt (clears `Lint`, `Rust Linting`, `Repository Hygiene`); #165
transport saturating latency score + zero-duration bandwidth bypass (clears
`Test` ×3 and `macOS Native Tests`). #165 carried a full adversarial APPROVE,
zero findings, `CRYPTO_TOUCHED: NO`.

---

## 3. Critical path to the tag

1. ~~**#165 green → merge to tracking.**~~ **DONE 2026-08-16**, together with
   #167, #168 and #169. All four fixes verified present on `tracking`:
   `manager.rs:470` now reads `100u64.saturating_sub(...)`, and `.gitattributes`
   declares `*.kt`/`*.kts`/`*.md` as `eol=lf`. #139's checks re-triggered.
2. **#139 → main.** This is D1 + D5. The repaired `pr_scope.sh` will raise five
   blockers; four are resolved and must be named explicitly rather than silently
   overridden:
   - *"100 commits, is this based on the branch you are merging into?"* —
     intentional here. `tracking` IS the long-lived integration branch, and
     merging it is precisely what D5 asks for.
   - *"touches merge-blocked directories"* — true, six files. **The
     crypto-security-auditor verdict exists** (§4). Its one HIGH finding is fixed
     by #157, which is merged into tracking, so #139 now carries the fix.
   - *"checks still running"* — must actually be green. Do not merge on a
     pending check.
   - *"no conflicts"* — clean.
3. `bash scripts/apply_branch_protection.sh --apply` (operator approved;
   `enforce_admins` true, **0** required approvals — raising it to 1 locks a
   single-operator repo out, GitHub forbids self-approval). **Do NOT list
   `Docker Integration Suite` as a required check** — see §6.
4. **Merge #154**, then tag `v0.4.0-alpha.1`.
5. Verify the published APK is genuinely release-signed (§5). **D2 + D3.**
6. Rebuild the AWS node to the tagged SHA, then run D4 (§5).

### Why every red lane on `main` is red

> **Superseded 2026-08-16.** This table described the state on 2026-08-15 and has
> since inverted — `Mobile`/KSP is GREEN and `Test` went RED. The current table,
> re-derived from the logs, is §3 of
> `HANDOFF/CTO_DISPATCH_PLAN_2026-08-16.md`. Kept here for history.

Verified from the literal CI logs, not inferred:

| Lane | Real cause | Fix |
|---|---|---|
| `CI` | its only failing job was `Lint` → `cargo fmt` on `core/src/lib.rs:159` | #139 |
| `Lint` | the same single fmt diff | #139 |
| `Mobile` | KSP `error.NonExistentClass` — the UniFFI bug | #139 (carries #149) |
| `Repository Hygiene` | trailing whitespace | #164 (merged) |
| `Docker Integration Suite` | UniFFI metadata stripping in the container | #156, non-blocking |

`CI`'s other five jobs (Test on windows/ubuntu/macos, FFI Surface Contract, Docs)
all PASSED on `main`. The lane was red on one formatting diff.

---

## 4. Security review of #139 — verdict on record

> **CORRECTED 2026-08-16. THIS SECTION WAS WRONG ABOUT THE GATE.**
>
> It records the verdict as "NEEDS FIXES. No CRITICAL hole" and reads as though
> the crypto gate is satisfied. **It is not.** The actual artifacts in
> `docs/security/` are a three-link chain that ends unresolved:
>
> | Artifact | Commit | Verdict |
> |---|---|---|
> | `PR139_ADVERSARIAL_REVIEW_2026-08-08.md` | `6cb7033a` | **BLOCK** — F1 CRITICAL (RFC1918 ledger disclosure gate never checked the requester; internal subnet map + peer-id-to-private-address binding to any unauthenticated remote) plus F2–F5 HIGH |
> | `PR139_REMEDIATION_2026-08-08.md` | — | remediation claimed |
> | `PR139_REVIEW_15dbcde0_2026-08-09.md` | `15dbcde0` | **BLOCK** — supersedes the first for that range; everything else clean; new **W1**: failover re-exchange is an unrated outbound amplifier |
>
> **The last recorded verdict on this PR is BLOCK, and no artifact clears W1.**
> The section below never mentions F1 or W1 at all. Whoever wrote it was
> describing a different, later review of a narrower diff — the §8 lesson
> ("your own past statements are claims") applied to this file itself.
>
> CTO code reading on 2026-08-16 indicates both are fixed at the current head —
> W1 gated behind `allow_failover_reexchange` on native (`swarm.rs:5343`) and
> WASM (`:7708`) with tests at `:8764-8768`; F1 now requires the requester's
> observed address, fails closed on `None`, rejects `P2pCircuit` on both sides,
> excludes CGNAT, and narrows to /24 or /64 (`addr_filter.rs:470`).
>
> **That reading is NOT a review.** AGENTS.md rule 8 requires an adversarial
> sign-off and `docs/ORCHESTRATION.md` forbids the controller from making that
> call. `CTO-139-CRYPTO-REVERIFY` is dispatched to produce the missing artifact
> at `docs/security/PR139_REVERIFY_2026-08-16.md`.
>
> **Do not merge #139 until that verdict exists and says APPROVE.**

A `crypto-security-auditor` pass ran against the six merge-blocked files #139
touches (`core/src/crypto/backup.rs`, and `addr_filter/behaviour/dial_policy/
observation/swarm` under `core/src/transport/`; +1,645/-154).

**Verdict: NEEDS FIXES.** No CRITICAL hole; X25519 and XChaCha20-Poly1305
untouched. Nothing was found that admits, dials, or discloses to a peer that
should be rejected — the diff actually *tightens* several gaps (stale-connection
disclosure, fail-open block checks, nested relay circuits).

- **HIGH — `dial_policy.rs` `build_relay_addresses`. FIXED by #157 (merged).**
  The loop set `has_ip`/`has_port` for the Ip4/Ip6 and Tcp/Udp match arms but
  never pushed those components, so circuit-relay addresses lost their transport
  prefix entirely. libp2p's relay transport requires a concrete address and fails
  with `MissingRelayAddr` before any I/O, at debug log level only. Relay NAT
  traversal was broken mesh-wide, in the same change set that removed UPnP.
- **LOW** — blocked-peer status is a distinguishable oracle: Registration and
  Relay answer with an explicit `"blocked"` error while AddressReflection and
  LedgerExchange stay silent. Post-tag.
- **LOW/INFO** — `mdns_dial_attempted` is unbounded under LAN mDNS spoofing.
  Requires L2 adjacency. Post-tag.
- **INFO** — backup salt moved `OsRng` → `rand::random()`. Cryptographically
  equivalent; no action.

**Two further defects, found by the integration suites in #162, fixed in #165:**

- `transport/manager.rs` — `std::cmp::max(0, 100 - latency_ms as u64)`. The
  subtraction evaluates first and u64 has nothing below zero, so any link over
  100 ms panicked in debug and **wrapped to near `u64::MAX` in release, inverting
  transport selection so the worst path scored highest.** Every cellular/WAN link
  is over 100 ms, and D4 is a cross-network test.
- `transport/internet.rs` — timestamps are whole seconds, so a relay in the same
  second as registration gave `conn_duration == 0` and the `if conn_duration > 0`
  guard skipped the bandwidth limit entirely.

Both were **pre-existing** (present on `main`, untouched by #139).

---

## 5. D2 and D4 — what is actually required

### D2 — signing is unblocked, but not proven

All four secrets are set (verified by name only; no agent has ever seen a value):
`SCMESSENGER_KEYSTORE_BASE64`, `_KEYSTORE_PASSWORD`, `_KEY_ALIAS`, `_KEY_PASSWORD`.

**Secrets existing is not proof signing works.** The base64 went through a
PowerShell pipe, and `release.yml`'s signed steps are conditional on
`HAS_KEYSTORE` — a malformed secret still yields a green job and a **debug-signed
APK**. Merge **#154** before tagging; it adds `apksigner verify --print-certs` and
fails the job on an unsigned or debug-signed artifact. If tagging first, check by
hand — `CN=Android Debug` means it did not sign.

The keystore lives at `%USERPROFILE%\kiee\` on the operator's machine. Never in
the repo, never read/copied/printed by any agent, permanently operator-only.

### D4 — Pixel 6a ↔ the AWS node

Operator decision: D4 runs **Android ↔ the AWS node**. There are no "relays" in
this architecture — every node relays — so this is node-to-node and
**cross-platform**, which is *stronger* evidence than two Android handsets.

Verified live 2026-08-15 via the EC2 API as `user/scmessenger-relay-orchestrator`:

```
i-006b14491d421bd0d  running  t3.micro  us-east-1  tag scm-always-on-node
curl http://54.226.67.101:9876/health   -> {"status":"healthy"}  (256 ms)
```

- **The node is Amazon Linux 2023, NOT Ubuntu.** `ssh ubuntu@` gives
  `Permission denied`; **`ssh ec2-user@` works**. At least 8 repo docs still say
  `ubuntu@` and every one of them fails.
- **`HANDOFF/gpt/AWS_RELAY_CURRENT_ADDRESS.md` is the canonical address pointer
  and it is correct.** The IP is dynamic — the account holds zero Elastic IPs and
  `ec2:AllocateAddress` is an **explicit deny** in `SCMessengerRelayFreeTierOnly`.
  Do not try to route around that; it is a deliberate cost guardrail, and the
  product does not need a stable address (v0.4.0 removed hardcoded bootstrap
  addresses; discovery is invite/QR ledger seeding).
- **The node runs code from a closed branch** — `/version` reports `9f54b107` on
  `gpt/pr139-receipt-filter-20260811` (PR #147, closed). It must be rebuilt.
- **NEVER build on the t3.micro.** A previous attempt ran 16 hours and OOMed.
  Pull the CI-prebuilt image.
- **Ordering is forced:** `docker-publish.yml` only fires on push to `main`,
  publishing `sha-<7char>`. So **#139 → main → CI publishes the image → rebuild
  node → run D4.** There is no way to prove D4 before the trunk merge.
- Runbook: `HANDOFF/D4_NODE_REBUILD_RUNBOOK.md` (merged, #159). Identity baseline
  to prove a rebuild did not orphan the ledger:
  `libp2p_peer_id 12D3KooWKMUXfjvWeodBUJbSwBuRXBU3d6XSbP1AJXL9WhaS3yKy`.

Scoring is unchanged: **receiver-side decrypt + durable history + receipt.** Not
transport ACKs, not UI counters, not BLE local acceptance.

---

## 6. Tooling and infrastructure

| Script | Purpose |
|---|---|
| `scripts/pr_scope.sh` | executable "unless there's a reason not to?"; **fails closed** |
| `scripts/triage_lane.sh` | first moves on a red lane — history before hypothesis |
| `scripts/agy_run.sh` | dispatch with per-step progress + stall detection |
| `scripts/reap_worktrees.sh` | reap abandoned worktrees; refuses DIRTY ones |
| `scripts/clean_target.sh` | scoped artifact reclamation; never calls `cargo clean` |
| `scripts/apply_branch_protection.sh` | branch protection, dry-run verified |

**`pr_scope.sh` failed open on 2026-08-15 and has been repaired (#158).** It read
its file list from `gh pr view --json files`, which caps at **100 files**. #139
changes 253. The first 100 held none of the merge-blocked paths, so it printed
`[OK] clear of core/src/{crypto,transport,routing,privacy}` while six gated files
were invisible — on the largest PR in the repo, on the exact check it exists for.
It now derives from `git diff --name-only origin/<base>...origin/<head>`,
announces loudly if it ever falls back to the API, and fails closed when the API
returns exactly 100 files. **Any PR reporting exactly 100 changed files should be
assumed truncated.**

**Docker Integration Suite is non-blocking (#156)** via `continue-on-error: true`
on the single failing step — narrowly scoped, so other Docker breakage still
fails the job. It therefore reports green while that step is broken. **D1 is to be
evaluated with this lane explicitly excluded**, and it must NOT be listed as a
required check in branch protection. Issue #155 tracks the real fix.

**Shared cargo cache (#163).** Every dispatched worker used to build its own
`target/`; one reached 16 GB and filled a 237 GB disk to 99%, at which point rustc
failed with `no space on device` and the compile gate could not run at all. Use:

```
export CARGO_TARGET_DIR=C:/Users/SCM/Documents/GitHub/.scm-shared-target
export CARGO_INCREMENTAL=0
```

Concurrent builds then block on the cargo lock — which enforces the existing
"never two build tools at once" rule rather than fighting it. Documented in
`docs/rules/BUILD_AND_CI.md`. Disk was 4.2 GB free at worst, 34 GB after cleanup.

---

## 7. OPEN — do not guess

1. **Was `ebf5411b`'s deletion of 7 Android sources intentional?** Restored on
   #149 on the CTO's read that APK sharing is active work. If it was a deliberate
   strip-down, revert the restore. Note the Josh fork independently deleted the
   *tests* for two of them, preserved on `josh-fork/local-worktree-state-2026-08-15`.
2. **Josh single-transport build** — operator ruled it is NOT the v0.4.0 default;
   ships as **v0.3.9** if at all. The transport quarantine described in an earlier
   session summary is **not implemented**; `d0e3258a` is 4 files, +23/-5.
3. **README framing** — the CEO was asked to bless the honest-first tone before
   the tag. No reply as of handoff.
4. **Dependency debt** — 7 vulnerabilities on the default branch, 3 high. Deferred
   to post-tag S4, which is right for shipping but should not stay deferred long
   on a security product.

---

## 8. Standing lessons

**Open the file.** Repeatedly this project has classified an artifact without
reading it and been wrong: `GEMINI.md` was already correct; two "duplicate pairs"
were prefix collisions; #147 looked like 102 files of unique work and had zero
unique commits; the repo already had a maintained canonical node-address file
that ~99 documents ignored. **The repo is consistently more coherent than its
directory listing suggests.** `AGENTS.md` rules 13 and 14 exist because of this.

**Verify the mechanism before quantifying.** "AWS node is down" came from a
5-second curl cap against an address that had simply changed. The node was
healthy in 256 ms the whole time.

**Commit before you clear.** A disk cleanup nearly discarded four untracked
integration-test files. Committing them to a branch first is what surfaced two
shipping transport defects on their very first compile. Two other worktrees held
work that existed on **no** remote: `wip-w1-ledger` (20 commits + 73 uncommitted
lines, unpushed entirely) and the JoshFork clone's 5 working-tree changes. Both
are now on origin. **Survey for unpushed work before reclaiming anything.**

**Committing a file as-is is not the same as it passing CI.** Preserving four
untracked files faithfully introduced 9 lines of trailing whitespace and turned
`Repository Hygiene` red on the trunk merge. Run the repo's own checks against
anything you commit, even when you are only preserving someone else's content.

**Dispatch budgets: 90 minutes, not 45.** Three "capability failures" on this
project were too-short timeouts. A task ending in `cargo test --workspace` needs
90m; the relay-ladder fix died at 36 minutes mid-compile with the work complete
but unpushed. Its worktree survived, so nothing was lost — check the worktree
before re-dispatching.

**One CTO seat.** Two sessions ran concurrently on 2026-08-15 and both were
editing this file and capable of merging. They did not collide, but only by luck.
If you find evidence of another active session, establish who holds the seat
before merging anything.

**Destructive history:** `git checkout <ref> -- .` destroyed four files of another
session's uncommitted work. `cargo clean --target <triple>` wiped 44.7 GB. The
preflight hook now blocks both and prints the working form — if it fires, read
it; it is there because someone already paid for that lesson.
