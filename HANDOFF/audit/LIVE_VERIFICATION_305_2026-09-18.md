# Live verification of #305's two riskiest changes

Date: 2026-09-18 (UTC). Scope: exercise PR #305's TRN-07 relay-admission ladder and
its AND-06 strict public-key decoding under real traffic on the live mesh, using
candidate binaries on the Windows node and the AWS container. The Pixel was not
touched.

Every number below came from a command run in this session. Raw captures and the
scratch scripts live in `tmp/v040-live-verify-20260917/` (`RAW-OBSERVATIONS.md`,
`key-corpus-raw.json`, `aws-container-config.txt`, `win-node-run1-0ca9da53.out`).

## 1. Method: how each thing was driven

Both fixes are only observable through behaviour, so nothing here rests on the
in-process tests:

* **Candidate binaries.** Windows: the CI artifact `windows-cli-0ca9da53...`
  (sha256 `20d1b481...`), staged in `tmp/radio-candidates/0ca9da53/` **outside
  any `target/`** so a reclaim cannot delete a running node's own binary
  (rule 17). AWS: `testbotz/scmessenger:sha-aab7e2a`, produced by dispatching
  `docker-publish.yml` on the branch (itself checked as additive first: the tag
  did not exist, HTTP 404, and `latest` is main-only).
* **The two candidate commits are the same tree.** The Windows artifact is built
  at the PR merge commit `0ca9da53`; `git diff --stat aab7e2a8 0ca9da53` is
  empty, so the merge commit contains exactly the branch head. The deployed fix
  code is therefore identical on both nodes.
* **Traffic.** The real relay path is reached by drift fallback: a node that
  cannot dial a destination commits the envelope to a connected carrier, which
  receives a `RelayRequest` and runs the ladder. Driving it required a
  destination nobody runs, so a **phantom identity** was generated with
  `openssl genpkey -algorithm ED25519` and its libp2p `PeerId` derived as
  `base58(identity-multihash(protobuf(pubkey)))`. That derivation was validated
  by re-deriving all three live mesh peer ids from their real public keys
  (3/3 match). The phantom exists so that volume could be applied without
  spamming a real user's chat.
* **Config fidelity.** The AWS container was recreated with the *same* name,
  network mode, bind mount and every environment variable as before (captured
  with `docker inspect` into `aws-container-config.txt`); the Windows node was
  restarted with its original command line and `RUST_LOG=debug`.

## 2. Identity preservation (a hard requirement)

| | before | after |
|---|---|---|
| Windows | `985a25f9...`, pk `30d0fa67...`, peer `12D3KooWD6vZ...` | unchanged |
| AWS | `37eb7561...`, pk `69805e17...`, peer `12D3KooWGvCW...` | unchanged |

Both nodes also kept their `device_id` and `seniority_timestamp`. Rollback is
available: the previous Windows binary
(`tmp/radio-candidates/c2ce2f64/`, sha256 `aa814b93...`) and the previous AWS
image (`testbotz/scmessenger:sha-c2ce2f6`, still present on the host) with the
unchanged data volume `/opt/scm-relay-data`.

## 3. TRN-07: the ladder, with real per-peer counters

Neither node logged a per-peer accounting line before this run, because the
dimension did not exist.

**Direction A - AWS node requests, Windows node relays** (70 sends):

| Windows node (candidate) | value |
|---|---|
| relay requests received | 113 |
| per-peer refusals | **62** |
| custody writes accepted | 40 |
| first refusal | `2026-09-18T00:29:52.110416Z` |
| refusal text | `Relay per-peer share (50/50 this hour) reached for 12D3KooWGvCW... - refusing <msg>` |
| global-budget refusals | **0** |

**Direction B - Windows node requests, AWS node relays** (70 sends):

| AWS node (candidate) | value |
|---|---|
| relay requests received from Windows | 112 |
| per-peer refusals | **62** |
| custody writes accepted | 38 |
| first refusal | `2026-09-18T00:39:25.694264Z` |
| requester-side errors | **62 x `relay_peer_budget_exhausted`** |
| requester-side successes | 40 x `Message relayed successfully` |
| global-budget refusals | **0** |

Both directions produced the same shape: ~50 requests admitted, then refusal at
exactly the default share `max(200/4, 25) = 50`, with the node-global 200/hr
budget untouched. The refusal is wire-visible with its own error string, and the
requester's own accepted/refused counts match the relayer's (40/62 and 38/62).

The refusal path takes the `PerPeerBudgetExhausted` branch *before* the
spam/token-bucket branch, and that branch emits no abuse signal — read from
`core/src/transport/swarm.rs:5104-5131` and confirmed by the log text: the
refusal is reported as a budget state, not as `RateLimited`.

### 3a. Does the cap bite in production? The node's own history says no

From the hourly logs of the node that ran before the swap (same workload, main
binary), inbound relay requests per hour, all from the Pixel peer:

```
17:00 -> 15    19:00 -> 7     21:00 -> 12    23:00 -> 0
18:00 -> 4     20:00 -> 10    22:00 -> 3     00:00 -> 8 (7 Pixel + 1 probe)
```

Peak observed: **15/hour**. Default per-peer share: **50/hour**, i.e. 3.3x the
observed peak. The cap is a safety net, not a throttle on normal traffic.

## 4. AND-06: the strict decoder against what the nodes actually store

New regression test `core/tests/test_and06_live_key_corpus.rs` (committed with
this document), corpus captured from the live nodes' control APIs:

```
cargo test -p scmessenger-core --test test_and06_live_key_corpus
  -> 5 passed; 0 failed
```

* All **3** public keys the live mesh stores are **accepted** (Windows, AWS, and
  the third peer), with their identities and provenance in the fixtures.
* `identity_id_from_public_key_hex` still derives each live `identity_id` from
  its live public key - the identity-resolution path this change could have
  broken.
* **4096** deterministically generated Ed25519 keys: all accepted, all deriving
  their documented identity_id.
* Monotonicity over 8000+ values (live corpus + generated keys + generated
  hashes + boundary encodings): the strict decoder **never** accepts a value the
  lenient dalek decode rejects, so the change can only remove acceptances.
* Measured on the 6 real non-key 64-hex identifiers the nodes store:
  `strict accepts 2, lenient accepts 2` (33%). That rate is inherent to Ed25519
  decompression - roughly half of all 32-byte strings decode to some point - and
  is unchanged by this fix; the monotonicity test is what pins that down.

### 4a. The same thing live, on the deployed binaries

The decoder is live-reachable: `GET /api/peer-resolve?input=<64-hex>` calls
`PeerIdTriad::resolve`, which calls `is_valid_public_key`
(`cli/src/api.rs:1642` route, `api.rs:1070` handler). Probed on **both**
candidate nodes:

```
real public keys (windows pk, aws pk, third peer pk, phantom pk)
  -> 4/4 success:true, input_kind=public_key, self_certifying=true   (both nodes)
real identity_ids
  985a25f9.. (windows) -> identity_id,  self_certifying=false   correct
  37eb7561.. (aws)     -> public_key,   self_certifying=true    AMBIGUOUS
  f83ab163.. (peer)    -> public_key,   self_certifying=true    AMBIGUOUS
```

So the condition this run was told to treat as a defect - strictness rejecting a
value a node legitimately produced or holds - **does not occur**: every real
public key is accepted, on both deployed binaries.

The two ambiguous identity_ids are **not** caused by this change. Measured
offline on the same three values: `strict=false/true/true`,
`lenient=false/true/true`. Identical under both decoders, so the ambiguity is
pre-existing and merely narrowed (not closed) here - it cannot be closed
mathematically, since roughly half of all 32-byte strings decompress to a point.

### 4b. Where the strict decoder is actually reachable

`is_valid_public_key` is referenced only by `identity_id_from_public_key_hex`,
`identify_key_type`, and `PeerIdTriad::resolve` inside `identity/keys.rs`, plus
the UniFFI export. Nothing in the transport, store, or custody paths calls it,
so this change's blast radius on the running mesh is exactly those resolvers -
and the Android/iOS adapters it was written for, which are not wired yet
(`PeerKeyUtils.isValidPublicKey` still runs the pure-Kotlin copy in 4 places in
`MeshRepository.kt`).

## 5. Custody

78 custody records were written across the two nodes during the drives (40
Windows, 38 AWS) plus their audit transitions, all addressed to the phantom. The
new retention bound applies to exactly these records, so they expire; a
`custody_expired` transition is written when they do.

## 6. What could NOT be exercised (stated plainly)

1. **Window rollover with non-zero counters.** The window is a hard-coded 3600 s
   from process start, so the boundary is `start + 3600`: AWS `01:33:40Z`,
   Windows `01:37:44Z`. Capture is appended to section 8 below.
2. **Live custody *expiry*.** The 5-minute prune tick only logs when it purges
   something (`[CUSTODY] Retention sweep expired N of M`). Neither node produced
   that line, and neither produced the failure line either - so the sweep did
   not error, but a no-op sweep is indistinguishable in the log stream from a
   sweep that never ran. The expiry logic itself is verified in-process against a
   persistent sled store (3 open/reopen cycles) in
   `test_trn04_custody_lifecycle.rs`. **No live record older than 7 days could
   be constructed**, so live expiry is unproven.
3. **Multi-peer fairness live.** The property "one peer's exhaustion does not
   stop another peer" was exercised in-process
   (`a_greedy_peer_is_throttled_while_others_still_relay`), but not live: after
   the refusals, no relay request arrived from any peer other than the refused
   one (Windows: 0 inbound in that window; AWS: 112, all from Windows). The
   Pixel's natural rate in those minutes was zero requests, so its absence is
   not evidence of a block - it is simply an untested live case.
4. **The Pixel.** Deliberately untouched, so no on-device Android behaviour was
   observed on this run.

## 7. Incidental findings (not fixed here)

* **N-06: loopback addresses propagate through ledger sharing and cause
  self-dials.** After the AWS container restart, the Windows node dialed its own
  loopback for the AWS peer and got `WrongPeerId { obtained: 12D3KooWD6vZ...
  }` (i.e. itself) for `/ip6/::1/tcp/9090/p2p/12D3KooWGvCW...`, then applied
  backoff to the **valid** public bootstrap address as a consequence.
* **N-07: the seed-dial gate can hide a lost hub.** While the Pixel was still
  connected, the Windows node logged `[SEED-DIAL] peers=1 -- connected; re-check
  in 120s` and stopped re-dialing seeds, so the Windows<->AWS link stayed down
  until the node was restarted. Mesh partition lasted ~4 minutes
  (00:33:40Z -> 00:37:44Z) and healed on restart; no operator action was needed.
* **N-08: an identity_id that happens to decompress is reported as a
  self-certifying public key, with a fabricated peer id.** Live, verbatim, on the
  candidate Windows node:

  ```
  GET /api/peer-resolve?input=37eb75612c179d57a83801ba71f3545ffdf3d009adbf73c652dae37536b1d006
  {"success":true,
   "input":"37eb75612c179d57a83801ba71f3545ffdf3d009adbf73c652dae37536b1d006",
   "triad":{"libp2p_peer_id":"12D3KooWDaeqG6cLWKfp33sUfPMNNRQLtV6AF8EUjZVWhicqsQNR",
            "public_key_hex":"37eb75612c179d57a83801ba71f3545ffdf3d009adbf73c652dae37536b1d006",
            "identity_id":"4382ce624614c55561b942bc322a90892ee3b345b9f779bc9dbdfe8279c01479",
            "input_kind":"public_key","self_certifying":true}}
  ```

  The input is the AWS node's *identity_id*. The reply asserts it is a
  self-certifying public key, invents a `libp2p_peer_id` that no node owns, and
  returns an `identity_id` that is blake3 of a hash - the double-hashing that
  `identity_id_from_public_key_hex`'s own doc comment says "must not be
  allowed". This is pre-existing (strict and lenient agree on all three real
  identity_ids) and **out of scope** for #305, but it is live-reachable and a
  caller that trusts `self_certifying` could route to a peer id nobody owns. It
  needs an operator decision - the shape is either an explicit `input_kind` on
  the caller side, or refusing to derive a peer id from a value that was never
  shown to be a key. Escalated, not silently changed (rule 9).

## 8. Rollover capture

Pending the boundaries noted in 6.1; appended when observed.

## 9. State changes this run made (and how to undo them)

* Windows node: restarted twice (graceful, via `POST /api/shutdown`); identity and
  store preserved. Binary staged outside `target/`.
* AWS node: container `scm-node` recreated with identical config; data volume
  untouched. Old image retained for rollback.
* One contact named `verify-phantom-20260917` added to each node. Both are
  reversible with `scmessenger-cli contacts remove 12D3KooWJaS3om...`.
* ~78 custody records addressed to the phantom, subject to the new retention.

## 10. Defect this run exposed in the fixes, and its fix

CI on the head (`aab7e2a8`) failed the **FFI Surface Contract** job:
`isValidPublicKey` appears in the generated Kotlin and Swift bindings, and the
checked-in snapshots were not updated with it (`195a196`, `389a390`). That is a
defect in the AND-06 half of this change, not a pre-existing one. Fixed by
adding exactly the line each snapshot needed, in sorted position, and committed
as `c8725b49`, which restarted CI on the corrected head.
