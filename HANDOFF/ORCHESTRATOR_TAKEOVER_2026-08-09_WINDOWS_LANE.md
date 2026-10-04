# Windows lane takeover packet -- 2026-08-09, PR #139

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Active
Written: 2026-08-09 ~12:45 UTC
Lane: Windows / ADB-Pixel / AWSUbuntu (macOS + iOS belong to GPT-MAC)
Candidate at handoff: `bfc7cac9bedd85ea5a86c9a327ed84d75dbabfad`

Read this, then `HANDOFF/todo/_QUEUE.md`. Everything below is file-backed; no
memory of the prior session is required.

---

## 1. Where PR #139 actually stands

- Full GitHub matrix **8/8 green** on `bfc7cac9` (CI, Desktop CI, Cross, Mobile,
  iOS Build & Test, Lint, Repository Hygiene, Auto Label).
- Windows/Claude adversarial review verdict: **PASS** on `6cb7033a..bfc7cac9`.
  Recorded in `docs/security/PR139_REVIEW_15dbcde0_2026-08-09.md` and extended in
  PR comments for `33c16712` and `bfc7cac9`.
- **The PR is nonetheless not runnable as a five-node test.** Three defects found
  by running it, not by reading it, are described below. None were introduced by
  the recent security commits.
- No candidate release artifact and no candidate Docker image exist. Every
  candidate `release.yml` and `docker-publish.yml` run so far was cancelled.
  GPT-MAC reserved those dispatches.

## 2. Blockers, in priority order

### P0-1 `libp2p-request-response` drift kills the desktop node when the mesh grows
`HANDOFF/todo/P0_REQUEST_RESPONSE_PANIC_KILLS_DESKTOP_ON_MESH_GROWTH_2026-08-09.md`

Root cause is **identified and verified from the crate source**, not inferred.
`libp2p-request-response-0.29.0/src/lib.rs:678`:

```rust
debug_assert_eq!(connections.is_empty(), remaining_established == 0);
```

`left: false, right: true` means the behaviour still tracks connections for a peer
the swarm says has none -- its `connected` map drifted and holds a stale entry
that never received a `ConnectionClosed`.

Two things the next session must not get wrong:

1. **It is a `debug_assert`.** Debug builds panic; release builds do not, and
   instead accumulate the drift silently. Running the fleet on release binaries
   would make the symptom disappear without fixing anything. **That is not a
   pass** and must not be recorded as one.
2. **0.29.0 is the latest published version.** There is no upstream bump, unlike
   the `libp2p-upnp` path. Any fix is ours.

Investigation direction: how does a connection get registered in
request-response and never produce a matching `ConnectionClosed`? Ranked
candidates are in the ticket. The existing `ConnectionClosed { num_established > 0 }`
guard is present, correct, and insufficient.

### P1-1 Promiscuous dial sweep wastes ~60% of its budget
`HANDOFF/todo/P1_PROMISCUOUS_DIAL_WASTES_BUDGET_ON_SELF_AND_CELLULAR_2026-08-09.md`

Measured: 700 dials in 4 minutes, zero connections. 88 to this node's own IP, 331
to unreachable carrier IPv6, 17 to the reachable macOS peer.

Root cause is subtler than "no filter". `is_dialable_for_this_node` IS applied at
`cli/src/main.rs:1853-1870`, but `is_self_address`
(`core/src/transport/addr_filter.rs:700`) compares the **whole multiaddr including
port**, so this node's own IP on a different port is not recognised as self.

**A fix diff exists and is unapplied** -- see section 4.

### P1-2 Three defects in the CLI send path
`HANDOFF/todo/P1_CLI_CANNOT_REPLY_TO_UNSAVED_PEER_2026-08-09.md`

All in `handle_send_message`, `cli/src/api.rs:561-632`:

- **A** cannot reply to a peer not already in contacts, even one whose message was
  just received and decrypted with a full identity block.
- **B** `/api/send` returns 500 for messages the retry machinery then delivers
  (observed: delivered 28 ms after the API reported failure).
- **C** outbound messages are never written to durable history -- only
  `direction: "received"` rows exist.

B and C **corrupt run scoring**, so they matter even if unfixed: do not treat a
non-200 from `/api/send` as proof of non-delivery, and do not expect sender-side
history rows on desktop nodes.

**A fix diff exists and is unapplied** -- see section 4.

### P1-3 Android has no route candidates
Not yet ticketed; evidence in
`docs/fieldtest/PR139_WINDOWS_LANE_CORRELATION_2026-08-09.md` and the logs below.
Every outbox retry fails `reason=no_route_candidates`, and
`Bootstrap all-failed (consecutive=94)`. Likely downstream of P0-1 and P1-1;
confirm before treating as independent.

### P2 items
- `HANDOFF/todo/P2_RESTORE_UPNP_ON_0_7_0_2026-08-09.md` -- blocked upstream.
  **The fix is in 0.7.0, NOT 0.6.0**, and 0.7.0 is unpublished. Restoring on
  0.6.0 reintroduces the crash.
- `docs/FEATURE_PARITY.md` matrix is stale (internal date 2026-03-27) and now
  carries a warning block listing five open parity gaps.

## 3. Live machine state at handoff

| Thing | State |
|---|---|
| Windows node | **running**, PID 3060, started 12:24:09Z, build `33c16712`, `RUST_LOG` debug filter on |
| Windows node peers | **0** -- has not reconnected since restart, consistent with P1-1 |
| Node binary | `C:\Users\SCM\Documents\GitHub\scm-review-8621a4b5\target\debug\scmessenger-cli.exe` |
| Node logs | stdout `scratchpad\soak2_stdout.log`, stderr `scratchpad\soak2_stderr.log` |
| Pixel | **updated** to a local build of `bfc7cac9`, identity preserved |
| macOS node | `12D3KooWNC5rEKFhuxDNDNsJ6Q58Ca75LnxfjUqspGzGRdYRUWyt` @ `192.168.0.136` |
| AWS node | untouched, old image, `/version` 404 |
| Build toolchain | idle; no cargo/gradle/java running |

**Isolated worktree** at `C:\Users\SCM\Documents\GitHub\scm-review-8621a4b5`,
detached at `bfc7cac9`. The shared checkout was never disturbed.

### Node identities

| Node | Peer id | Address |
|---|---|---|
| Windows (`Claude-Windows-Driver`) | `12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw` | `192.168.0.121` |
| Pixel (`Lucaso`) | `12D3KooWNnPi9wqUJ7Jypj6g4jHmW2PUTmynUs9sJY1h6SQbjLrG` | `192.168.0.141` |
| macOS (GPT-MAC) | `12D3KooWNC5rEKFhuxDNDNsJ6Q58Ca75LnxfjUqspGzGRdYRUWyt` | `192.168.0.136` |
| iPhone (`ChristyLove`, NOT a comms endpoint) | `12D3KooWJUJ1koSWwSEAX32z6SGaepikyqpJawpojoy6gvQ8k688` | `192.168.0.142` |

Operator lane split: **macOS is the GPT-driven agentic node and the coordination
endpoint. The iPhone is a personal device -- a run participant whose logs we need,
but no agent should message it for coordination.**

## 4. Unapplied worker diffs -- pick these up first

Three dispatches completed. **None applied, none gated.** The Gradle build held
the toolchain until the end of the session.

| File | Diff | Footer |
|---|---|---|
| `cli/src/ledger.rs` (self-dial fix) | `tmp/fix_self_dial_response.md` | RESULT: DONE |
| `cli/src/api.rs` (send-path A/B/C) | `tmp/fix_cli_send_path_response.md` | RESULT: DONE |
| analysis only, no diff | `tmp/analyze_reqresp_panic_response.md` | **unreliable, see below** |

Prompts are at `tmp/fix_self_dial.prompt.md`, `tmp/fix_cli_send_path.prompt.md`,
`tmp/analyze_reqresp_panic.prompt.md`.

**Review notes on the self-dial diff, from reading it:** it is substantively
correct -- circuit check moved before the new IP-level self-check, RFC1918 logic
preserved, five tests added. Two cleanups needed on apply: it leaves an orphaned
comment where the old circuit check was, and it declares a private `enum IpAddr`
that shadows `std::net::IpAddr`.

**The send-path diff has not been reviewed at all.** Verify especially that the
blocked-peer check is not bypassed by any new resolution fallback.

Next actions: apply, clean up, run `cargo check --workspace` plus focused tests
under `scripts/build_lock.py`, then restart the Windows node on the fixed binary
and see whether it finds the macOS node. That is the fastest route to a working
Windows-macOS CLI link.

## 5. Traps that cost time today -- do not re-pay these

1. **The panic never reaches `scm.log`.** The rolling tracing file shows only
   `swarm_event_loop_died`. Both P0 root causes were only visible because stderr
   was captured separately. Always redirect stderr when running a node.
2. **A single-peer soak proves almost nothing.** 93 minutes clean with one peer,
   dead in under 2 minutes at four peers. Any soak must have three or more live
   peers with relay circuits.
3. **`RUST_LOG` already works on the CLI** -- no code change needed. The dial-budget
   defect was invisible at `info` for hours and obvious within 4 minutes at debug:
   `RUST_LOG="info,scmessenger_core::transport=debug,scmessenger_core::store::outbox=debug,scmessenger_core::store::inbox=debug,scmessenger_core::relay=debug,scmessenger_cli=debug"`
   Android debug builds already default to that filter. iOS cannot use `RUST_LOG`
   at all and needs a debug-profile build or it is a recorded evidence gap.
4. **CI APKs cannot install over a locally-built app.** Signing keys differ (CI
   `314e6538...`, this machine `1cdef09c...`, release a third). `adb install -r`
   fails non-destructively; only a wipe would let a CI artifact on. Local build +
   `install -r` preserves identity and is proven to work.
5. **`:latest` moved three times today.** Every docs push to `main` triggers
   `docker-publish`, which tags `latest` on the default branch. Rebuild AWS from
   an image **digest** only.
6. **An Ed25519 `12D3Koo...` peer id contains its public key.** Identity multihash,
   not hashed. Helper written and verified against three pairs:
   `scratchpad/peerid_to_pubkey.py`. This removes the need to wait for an
   `identity_sync` before adding a contact.
7. **Verify delegated analysis against source.** The THINK-tier worker returned a
   confident analysis with the wrong assertion, fabricated line numbers
   (`ConnectionClosed` at 2682-2724; the real arms are 4915 and 4930), and a false
   claim that no path drops an inbound channel without responding (`swarm.rs:3540`
   and `:6731` both do). The crate source was on the machine the whole time.
8. **Do not read `$?` after a pipe**, and prefer PowerShell `Get-Process` over
   `tasklist //FI` -- the latter silently returned empty for a live process.

## 6. What was filed today (all on `main`)

| Commit | Contents |
|---|---|
| `59ed9611` | security review of the candidate + first log-evidence doc |
| `d05f1b21` | Windows/Android/AWS correlation; corrects the earlier "window unrecoverable" claim |
| `8b7adc21` | UPnP restoration P2 + CLI reply-resolution P1 |
| `03286060` | `FEATURE_PARITY.md` stale-matrix warning and gap list |
| `625c3846` | send-path defects B and C added to the P1 |
| `523cb087` | P0 request-response panic |
| `727ced40` | P1 promiscuous dial budget |
| `4ae1480a` | P0 root cause from crate source |

PR comments carry the same content for GPT-MAC. The last one is
`issuecomment-5231584606`.

## 7. Open questions for the operator / GPT-MAC

1. Does the iPhone ship a debug-profile build for the run, or do we record iOS
   transport detail as a known evidence gap?
2. The AWS node reports `initialized: false` with no peer identity. Relay-only or
   participant? There is currently nothing to put on the provenance line.
3. `LEDGER_SHARING_ANDROID_NODE_VISIBILITY_2026-08-05.md` is blocked "awaiting
   operator decision on disclosure policy" -- but this PR has since *made*
   disclosure-policy decisions. A peer joining over a relay circuit now receives
   no private entries by design. Does that resolve, supersede, or reproduce that
   ticket's symptom?
4. Who dispatches `release.yml` (`artifacts_only=true`) and `docker-publish.yml`?
   GPT-MAC reserved it; the matrix gate they set is now met.

## 8. Standing constraints

- Two independent flawless five-node PASS runs are required before merge.
- `core/src/{crypto,transport,routing,privacy}/` is merge-blocked pending
  adversarial review.
- An agent that authored or specified a fix cannot give the rule 8 sign-off for
  it. This lane proposed the W1 fix shape and both unapplied diffs above, so it
  cannot be the signatory for those.
- Never push unless the operator asks.
