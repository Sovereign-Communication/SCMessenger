# Four-Node Gate Execution Plan -- Claude CTO/CAO prep

Status: Active
Created: 2026-08-23
Owner: Interim CTO/CAO (Claude seat)
Supersedes: nothing. Executes `HANDOFF/CTO_STATE.md` section 0-2026-08-23b,
amended by this file's VALIDATED corrections. Apple-lane contract remains
`HANDOFF/gpt/CTO_TO_CAO_2026-08-23_APPLE_LANE_ACTIVE.md` (AW-BILAT-0003),
restaffed per section 8 below.
Companion: `HANDOFF/CTO_CAO_PRETAG_VALIDATION_AND_UNIFICATION_2026-08-23.md`
(claim-by-claim evidence ledger).

---

## 0. Mission and definition of done

Run the four-node field gate against tagged draft release `v0.4.0-rc.1`,
proving D4/D6/D7 of `SHIP_PLAN.md` on receiver-side evidence only, with every
node reporting the same frozen git hash. Publishing the release additionally
requires the four publish conditions in CTO_STATE 0-2026-08-23d (forgery tests
on main shown failing-on-revert, one adversarial review APPROVE of the merged
tree, external audit COMMISSIONED, honest release body).

## 1. Topology (validated 2026-08-23)

| Node | Platform | Artifact | Source | Live state at validation |
|---|---|---|---|---|
| N1 | Pixel 6a (Android) | signed release APK | release assets on tag | last known install: b4ccd30a-era; reinstall from tag |
| N2 | Second Android handset | signed release APK | same asset, same file | operator to confirm device availability |
| N3 | Windows CLI | `scm-windows-amd64.exe` | release assets on tag | RUNNING (PID 16156, up since 08-22), listening multiport 9001/9002/9090/8080/80/443, ESTABLISHED to 54.226.67.101:9001 |
| N4 | AWS headless relay | prebuilt image at tag SHA | Docker Publish output | LIVE and healthy (`HTTP 200 {"status":"healthy"}` at 54.226.67.101:9876/health) but image is STALE: built at 6b2573fa per `HANDOFF/gpt/AWS_RELAY_CURRENT_ADDRESS.md`; must be rebuilt/redeployed at the tag SHA |

Apple (macOS CLI + iOS app) joins as N5 only through the AW-BILAT-0003 CI-green
join point. It is not a blocker.

## 2. Gate order (do not reorder)

1. **Gate A** -- code lands on main (items below).
2. **Gate B** -- tag `v0.4.0-rc.1`, cut DRAFT release; machinery verified.
3. **Deploy** -- every node rebuilt from the tag; manifest captured.
4. **Gate C** -- D4, D6, D7 proofs using the G1-G6 matrix adapted to four nodes
   (section 5).
5. **Publish decision** -- per CTO_STATE 0-2026-08-23d.

Freeze rule (operator-locked): one exact SHA on every node; any runtime fix
creates a new anchor and restarts qualification.

## 3. Gate A status board -- corrected and validated 2026-08-23

Evidence source for each line is in the companion validation file. States that
CHANGED vs CTO_STATE stand-down are marked **[DRIFT]**.

| # | Item | PR | Validated state 2026-08-23 | Blocking action |
|---|---|---|---|---|
| A1 | V1+V2 sender auth | #221 | OPEN/DRAFT, MERGEABLE, BEHIND main | Close X25519 coverage-gap test; merge main in; clippy; FRESH adversarial review of merged tree |
| A2 | Storage fail-loud | #222 | OPEN/DRAFT, MERGEABLE, BEHIND main | Merge main in; land first so A3 stacks cleanly |
| A3 | Android degraded-storage wiring | #227 | OPEN/DRAFT, UNSTABLE -- **[DRIFT] `Android JVM Unit Tests` FAILING**: `MeshRepositoryTest > isStorageDegraded initial state is false` (ClassCastException ConnectivityManager in JVM test); run 32670592900, 2026-08-23T22:47Z. CTO_STATE "verified green" is STALE | Fix the JVM test harness (Robolectric shadow or guard), re-run green, then merge after A2 |
| A4 | Android reachability | #220 | OPEN, `Android Wiring Gate` fail = the 2 accepted findings | Operator records written ACCEPTANCE (accept, do not fix); then merge or close per ruling |
| A5 | CLI identity persistence | #219 | OPEN/DRAFT, BEHIND; **RED confirmed today**: `Lint` + `Rust Linting` failing (runs 32617007302/32617007321) | First check whether A2/#222 alone stops the churn (root cause at iron_core.rs:402). If yes: close #219 with reason. If no: fix lint failures and join Gate A. N3 cannot run the gate with churning identity |
| A6 | POST_TAG_QUEUE re-entry | done | 2 BLOCKS remain open (see section 4) | BLOCK-1 needs operator ruling + stderr capture wired; BLOCK-2 needs the ladder check during D4/D6 |
| A7 | Clippy deny + negative-test CI | worktree `_scm_wt/cihard` branch `cto/ci-hardening-2026-08-23` | Work INTACT and uncommitted (verified: 8 modified files + 2 untracked, based on e5ff72cf). Script policy sound; allowances reviewed -- all legitimate (test mocks, cfg(unix) dual-use param, documented deliberate no-op). Zero-test loud-failure implemented via `forg` count | Do NOT re-dispatch blind. Commit, prove the lint fires (deliberate violation -> paste failure -> revert), push, let CI green, merge. Making the new workflow a REQUIRED check is an admin follow-up |

## 4. The two A6 BLOCKS -- gate-day handling

### BLOCK 1 -- desktop request-response panic (VALIDATED at source)

libp2p-request-response-0.29.0/src/lib.rs carries unconditional `.expect()`
panics at lines 670 and 676 (fire in ALL profiles); the `debug_assert_eq!`
at :678 is compiled out of release. Our swarm-level cap protects application
state only, not crate internals. Most likely during D6 transport-failover churn.

Minimum handling:
1. N3 stderr captured to its own file from process start (`... 2> n3_stderr.log`),
   separate from the tracing log -- the panic never reaches the rolling log.
2. Operator ruling REQUIRED before the gate: (a) ACCEPT with workaround
   "restart N3 promptly; mid-gate panic is a known-possible event, not a new
   regression", recorded in the run manifest; or (b) extend the peer-level cap
   into core/src/transport dial paths (merge-blocked perimeter, needs auditor).
   This is an operator judgement call.

### BLOCK 2 -- relay fallback for roamed peers (ladder check)

Fixed parts verified (self-dial filter, stale reaping). UNPROVEN: that a
circuit through the live AWS relay is constructed and prioritised for a peer
reachable only via relay. Cheap targeted check (minutes): during the D4/D6
cross-NAT leg with `RUST_LOG=debug` on N3, grep the ladder line

    "Dialing candidate ladder for {}: {:?}"   (core/src/transport/swarm.rs:6095)

and confirm by eye that a `/p2p-circuit` entry through the AWS peerId both
APPEARS and IS DIALLED. Capture FULL multiaddrs including `/p2p-circuit`
suffixes (the historical regex dropped them and produced a false root cause --
see field-gate reference section 2.2C). If no circuit is reached, D4/D6 block.

## 5. Gate C -- what the run must prove (four-node adaptation)

Scoring is unchanged and non-negotiable. Delivered = receiver-side decrypt AND
durable history surviving app restart AND receipt returned to sender. Transport
ACKs, UI counters, BLE local acceptance, and "log says sent" count for nothing.

Adapt the G1-G6 matrix from
`HANDOFF/plans/PR139_FIVE_NODE_FIELD_GATE_REFERENCE.md` (still authoritative
for harness/evidence semantics) to endpoints {N1, N2, N3} with N4 as
infrastructure-only:

| Criterion | Four-node form |
|---|---|
| D4 | N1 <-> N2 message + receipt, cross-network (one cellular, one WiFi) on the RELEASED APK |
| G1 pairwise | N1<->N2, N1<->N3, N2<->N3, both directions each (6 flows) |
| G2 transports | LAN/WiFi direct, BLE (D7 leg), internet path via N4 custody |
| G3 delivery truth | offline queue -> restore -> opportunistic delivery; no resend after receipt |
| G4 convergence | restart any node; reconverge without re-pair |
| G5 liveness | transport disruption recovers without app restart; fleet growth to 3 endpoints + infra does not kill N3 |
| G6 provenance | all four nodes report the tag hash; N4 image digest immutable |
| D6 | delivery when first-choice transport unavailable (failover proof; ladder check from section 4 doubles as route evidence) |
| D7 | N1 <-> N2 with NO internet; name which transport carried it |

Qualification bar stays as locked: two complete matrix passes + one 60-minute
soak on the frozen anchor (operator decision 2026-08-10, never rescinded).
Soak clock-reset conditions inherit from field-gate reference section 10.3.

## 6. Log capture and live analysis (wire BEFORE deploy)

| Node | Capture | Notes |
|---|---|---|
| N1/N2 | `adb logcat -b crash -v time > n_crash.log` AND `-b main` to second file; pull `files/logs/scmessenger-mesh.log` after each matrix | crash buffer FIRST (main buffer hides crashes); watch ring eviction during BLE storms |
| N3 | stdout+tracing to `n3.log`, stderr SEPARATELY to `n3_stderr.log` from launch; `RUST_LOG=debug` for the ladder window | panic visibility lives only in stderr |
| N4 | `docker logs -f` persisted host-side; record image digest in manifest | collector must survive container restart |
| Fleet | per-run dir layout per field-gate reference section 11 (manifest.json, matrix-N/results.json, per-node logs, messages/<test-id>.json, summary.md) | correlation IDs minted before send; raw captures kept unfiltered |

Live-analysis rules that have already paid off once each:
- absence in a collector is not evidence of absence; prove collectors can see
  what they score;
- score sustained absence, not transient errors (BLE L2CAP now backs off);
- classify "unknown" routes as unproven, never PASS.

## 7. Pre-deploy checklist (Gate B -> Deploy)

1. All Gate A rows closed or operator-accepted in writing.
2. `bash scripts/verify_versions.sh` passes (VALIDATED green locally on
   2026-08-23: Cargo/Android/Desktop/WASM 0.4.0, iOS 0.4.0, build numbers above
   baseline). Re-run at the release commit.
3. Tag `v0.4.0-rc.1` on a green main SHA; confirm the draft release carries:
   signed AAB+APK, `scm-windows-amd64.exe`, checksums, provenance.
4. Rebuild/redeploy N4 at the tag-SHA image (current image predates #139).
   Confirm SSH-key situation first -- recorded 2026-08-05 that none exists;
   teardown+rebuild path must be proven before tag day, not on it.
5. Update `HANDOFF/gpt/AWS_RELAY_CURRENT_ADDRESS.md` immediately after any N4
   rebuild (ephemeral-IP policy).
6. Install the SAME APK file on N1 and N2 (in-place on the Pixel to preserve
   identity; note the CI-signature in-place-upgrade caveat in
   `HANDOFF/todo/ANDROID_CI_APK_SIGNATURE_BLOCKS_INPLACE_UPGRADE_2026-08-09.md`
   if relevant to the chosen handsets).
7. Capture the pre-flight manifest: per-node git hash, identities/PeerIds,
   listeners, image digest, clocks, log-capture status.
8. Verify every node prints the tag hash before Matrix Pass 1.

## 8. Staffing (changed 2026-08-23, operator directive)

- GPT subscription lapsed. The CTO/CAO seat is CLAUDE (this seat).
- Antigravity (Gemini 3.7 Flash / 3.1 Pro) may orchestrate OSX/iOS DEPLOYMENT
  on the MacBook -- read from the handoff docs, build/install/report. It does
  NOT edit code; implementation stays behind PRs and CI. Deployment evidence it
  returns must be pasted command output, never prose (L3 rule applies).
- The AW-BILAT-0003 contract stands mechanically: iOS/macOS code completes
  locally, pushes to a branch, ALL iOS/macOS CI lanes green, THEN deploy via
  antigravity, THEN the node joins as N5 reporting the tag hash.
- Worker lanes keep: no merges, no HANDOFF moves, report format per AGENTS.md.

## 9. Known unknowns carried into the gate (honest list)

Full detail in the companion file. Headlines:
1. Four Aug-10 tickets (finite-retry abandonment PF-1, Android self-ratchet
   reset, inbound CryptoError, async receipt non-convergence) are undispositioned
   against current main -- verify-or-disposition BEFORE the tag.
2. GitHub signing-secret presence is corroborated by record (set 2026-08-15)
   but not independently verifiable without admin access; first tag attempt
   proves it either way.
3. N2 handset availability is operator logistics, unresolved.
4. Whether #222 alone fixes N3 identity churn (decides A5 disposition).
