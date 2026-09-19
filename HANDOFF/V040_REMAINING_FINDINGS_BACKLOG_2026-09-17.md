# V040 remaining-findings backlog + v0.4.0 tag gate status (2026-09-17)

Author: CTO seat (Freebuff lane), post-merge-train reconciliation
Baseline audited: `HANDOFF/audit/SHADOW_AUDIT_V040_V050_ADVERSARIAL_REVIEW_2026-09-16.md`
(audit evidence baseline was commit `ddca1340`)
Live main at time of writing: `eb55756957e2d01f558321b374258a3c05750181`
Method: every status below comes from a command run on 2026-09-17 against
`origin/main` (the exact command family is named per row). Audit line numbers
have drifted; do not treat the audit's file:line as current.

## 1. What today's train actually landed

Merged into main 2026-09-17 (each verified green before merge):

| PR | Content | Merge evidence |
|---|---|---|
| #288 | V040 multi-transport store-and-forward carrier (absorbs #290/#292/#293/#294/#296/#297) | 34/34 checks green, Rule-8 reviews on file (09-14 APPROVE bod-9ee86618 + 09-17 #296/#297 verdicts) |
| #289 | Android Ed25519 curve-check consolidation + CI SDK/RUSTSEC fixes | all lanes green, harness Rule-8 verdict on file (`HANDOFF/review/RULE8_PR289_VERDICT_2026-09-17.md`) |
| #283 | V1.0.0 readiness audit + Linux-first takeover plan | docs-only, all lanes green |

Still in final CI at time of writing: **#295** (AND-01/02/03). Its Android JVM
lane — the one that was red — has passed on the fixed head; Apple/matrix lanes
were pending. #295 must not be counted as landed until merged.

Carrier fix commits authored on the train (all CI-verified): hygiene
whitespace, `cargo fmt`, and a wasm-cfg split of `is_ghost_peer_topic`
(native ledger consultation unchanged; wasm falls back to pre-#296 allow).

## 2. Audit findings — verified status against live main

| ID | Severity | Status on `origin/main` | Evidence obtained 2026-09-17 |
|---|---|---|---|
| CLI-01 docker control API | CRITICAL | **FIXED** | `git show origin/main:docker/entrypoint.sh` -> `--http-bind "${SCM_HTTP_BIND:-127.0.0.1:9876}"` (localhost default) |
| SEC-01 unsigned debug APK fallback | CRITICAL | **FIXED** | `release.yml:131-135` — "Assert release keystore configured for version tag" step: `if startsWith(github.ref,'refs/tags/v') && HAS_KEYSTORE != 'true'` then `exit 1` ("Failing closed") |
| CRYPTO-01 identity spoofing | CRITICAL | **FIXED (merged, Rule-8 reviewed)** | PR #296 merged; verdict `HANDOFF/review/RULE8_PR296_VERDICT_2026-09-17.md` |
| TRN-03 WASM own-topic | CRITICAL | **FIXED (merged, Rule-8 reviewed)** | PR #296 merged (verdict above); wasm cfg split verified by the WASM CI lane |
| CLI-03 outbox key mismatch | CRITICAL | **FIXED (merged, Rule-8 reviewed)** | PR #297 merged; verdict `HANDOFF/review/RULE8_PR297_VERDICT_2026-09-17.md` |
| CORE-02 IronCore outbox split-brain | CRITICAL | **FIXED (merged, Rule-8 reviewed)** | PR #297 merged (`Outbox::persistent` unification) |
| TRN-01 swarm/CLI deadlock | CRITICAL | **ADDRESSED BY MERGED PR #292** (runtime efficacy not re-proven this session) | PR #292 merged today |
| AND-01/02/03 Android cold-start + scaffold + SubnetProbe | CRITICAL/HIGH | **IN FINAL CI (PR #295, not yet merged)** | JVM lane green on fixed head; see `HANDOFF/review` note |
| TRN-08 handover starvation (4-conn cap) | HIGH | **NO LONGER REPRODUCES at the audited site** | `behaviour.rs` current limits: `max_pending_outgoing 32`, `max_established_outgoing 128`, `max_established_incoming 64` — not a 4-connection cap |
| TRN-07 global relay budget DoS | HIGH | **OPEN** | `swarm.rs` still initialises `let mut relay_budget: u32 = 200;` and gates at `relay_count_this_hour >= relay_budget` |
| AND-06 Kotlin BigInteger Ed25519 | HIGH | **OPEN** | `PeerIdValidator.kt` on main still contains 15 `BigInteger` references (consolidation in #289 reduced divergence, not the math) |
| SEC-03 sled unmaintained + waivers | HIGH | **OPEN** | `deny.toml` still waives RUSTSEC-2025-0141 / 2025-0057 / 2026-0118 / 2026-0119 / (parking_lot chain) as transitive-via-`sled` |
| TRN-04 unauthenticated custody + no TTL | CRITICAL | **PARTIAL** | `relay_custody.rs::accept_custody` now enforces payload bounds (1..=65536), message-id length, 64-hex recipient identity, device-id format; tests include `accept_custody_accepts_unregistered_identity_in_cooperative_mesh` (by-design) and `accept_custody_rejects_invalid_unregistered_identity_or_payload`. **Still absent: sender authentication and any TTL/retention bound.** |
| TRN-02 Windows pressure-probe bypass | HIGH | NOT RE-VERIFIED (defined-line window moved; needs re-baseline) | — |
| TRN-05 false-positive ghost classification | MAJOR | NOT RE-VERIFIED — site now carries "Fail closed on ghost shape when we cannot consult the ledger"; disposition needs a read | — |
| TRN-06 O(N) custody scan per write | MAJOR | NOT RE-VERIFIED | — |
| CLI-02 watchdog false termination | MAJOR | NOT RE-VERIFIED (watchdog block now keys off `latest_mtime: Option<SystemTime>`) | — |
| AND-04 mDNS dial injection | HIGH | NOT RE-VERIFIED | — |
| AND-05 sync FFI on UI thread | HIGH | NOT RE-VERIFIED | — |
| SEC-02 keystore alias mismatch | HIGH | NOT RE-VERIFIED | — |
| GOV-01 governance TOCTOU lock | HIGH | NOT RE-VERIFIED | — |

Verification-quality note: one early attempt to read `release.yml` returned an
empty result because MSYS mangled the `origin/main:` revision and stderr was
discarded — a false "no match". It was re-run with `MSYS_NO_PATHCONV=1` and the
real content is recorded above. Any "NOT RE-VERIFIED" row above is explicitly
not a claim either way.

## 3. v0.4.0 tag gate

**NOT READY.** The 2026-09-16 HALT verdict is only partly discharged:

- Closed by merged work: CLI-01, SEC-01, CRYPTO-01, TRN-03, CLI-03, CORE-02,
  TRN-01 (PR #292), TRN-08 (site no longer matches).
- Still blocking: **TRN-04** (auth + retention), **TRN-07** (global budget),
  **AND-06** (Kotlin BigInteger), **SEC-03** (sled waivers) — plus the
  unverified set (TRN-02, TRN-05, TRN-06, CLI-02, AND-04, AND-05, SEC-02,
  GOV-01), which must be re-baselined before a tag can be argued.
- `v0.4.0` still does not exist as a tag (latest: `v0.4.0-rc.1`).

Recommended next wave (dispatch order, each as its own PR onto main with the
Rule-8 gate for `core/src/{crypto,transport,routing,privacy}`):

1. **Wave A (gate re-baseline, cheap, no code)**: re-verify the eight
   NOT-RE-VERIFIED findings against current main and rewrite the audit's
   file:line references. Gate: a tracked re-baseline doc.
2. **Wave B (custody hardening)**: TRN-04 sender authentication + TTL/quota
   bound, TRN-02 pressure-probe bypass, TRN-06 scan cost. One PR, Rule-8.
3. **Wave C (liveness/DoS)**: TRN-07 budget policy (per-peer + global),
   TRN-05 ghost-classification precision.
4. **Wave D (Android)**: AND-06 move curve validation to Rust/UniFFI,
   AND-04 mDNS dial validation, AND-05 off-main FFI.
5. **Wave E (supply chain/CI)**: SEC-03 sled replacement or documented
   acceptance, SEC-02 alias fix, GOV-01 lock ordering.

## 4. Fleet readiness (cross-reference)

See `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260917T195444Z_PREFLIGHT.md`:
all three nodes are reachable and a live Pixel → cloud-node delivery was
observed today (265 ms), but the fleet is **not same-candidate** — Windows
core `597e2c72`, AWS core `6acaa231`, Pixel APK of 2026-09-16 — none at main's
tip. A redeploy of all three is required for any tag-certification run.
