# v0.4.0-alpha.1 FINDING DISPOSITIONS (040-S1b live table)

Status: LIVE -- updated per remediation commit and review round
Authority: operator mandate "fix ALL open findings before tagging" +
GPT plan 040-S1b ("implicit deferral is forbidden -- every finding is
FIXED or has an operator-signed release decision").
Verdict source of record: HANDOFF/review/LEDGER_SEEDING_ADVERSARIAL_REVIEW_2026-07-25.md

| Finding | Status at HEAD | Disposition | Owner packet / evidence |
|---|---|---|---|
| F1 invite signatures unverified | CLOSED (30181941) | FIXED -- prior | verify_with_policy + ML-DSA-65 in core/src/relay/invite.rs |
| F2 signed import path dead | OPEN (no product accept path) | DOCUMENTED + CANARY -- residual risk is future-wiring only; acceptance ticket for invite-accept MUST call verify_with_policy before import_seed_entries; CI grep canary: no non-test callers of import_seed_entries without verify | canary pending (post-v2b) |
| F3 DNS/SSRF address admission | CLOSED (22b921ca + annotate guards) | FIXED -- verify in terminal verdict | is_recordable_multiaddr + is_dialable_multiaddr + DnsPolicy::Reject on all insert paths |
| F4 unbounded candidate build | CLOSED | FIXED -- prior | caps + HashSet dedupe + bounded seed_addresses |
| F5 startup deadlock | CLOSED | FIXED -- prior | seed dial via tokio::spawn |
| F6 topology disclosure | CLOSED (filter rebuilt; NEW-2 addressed at addr_filter.rs:365) | FIXED -- verify bucket-ordering residual (NEW-5) in terminal verdict | exchange_response_entries proven-tier-only + lazy take(limit) |
| F7 dial-policy bypass / no record_failure | PARTIAL | FIX IN PROGRESS | threshold (3) + proven-tier filter LANDED (v2a-2); (a) register gate + (b) record_failure wiring LANDED (packet 2, 5b66f896); GPT V2A_PRECHECK: dialable_addresses + exchange_response_entries threshold fix pending v2b |
| F8 circuit collapse | CLOSED | FIXED -- prior | protocol-iterating strip |
| F9 empty multiaddr parse | CLOSED | FIXED -- prior | empty + no-transport rejected |
| F10 unbounded ledger growth / disk DoS | CLOSED pending terminal verdict | FIXED | load cap (v2a-1); byte bounds at ingest AND load (v2a-2 + v2c-2); save serialization + atomic durable writes (v2c-1); batch caller (1c); durable shrink-persist at load (v2c-2); corrupt-file recovery (v2c-3). RESIDUAL (post-alpha, bounded): streaming parse of multi-MB legacy files -- growth path closed by write-side caps, 16MB warn guard in place |
| F11 ledger never loaded | CLOSED | FIXED -- prior | load() in constructors |
| F12 ranking poison / floor placement | CLOSED (wire-boundary recorder mesh_routing.rs:516-528; bounded + pruned) | FIXED -- verify in terminal verdict; deterministic ordering hardening in v2b | record_recipient_seen_via_relay_from_wire |
| F13 pending-dial inbound false resolution | LANDED (5b66f896) | FIXED pending terminal verdict | is_dialer() gate; DESIGN DECISION: gate kept -- evidence integrity over the rare collapsed-simultaneous-open edge (self-heals via 10s sweep, connection stays up, spurious failure +1 vs threshold 3) |
| F16 mobile bindings drift | CLOSED -- verified NO DRIFT (workflow F16 check) | FIXED | all seed symbols in source exports + FFI snapshot aligned (ffi_surface.sh check exit 0); Kotlin bindings are build-time generated (gradle preBuild) -- nothing committed to drift; Swift regenerated in PR #112; making the CI drift step blocking = V050-I1 (0.5.0) |
| NEW-5 bucket checked after expensive work | CLOSED (consume-first at swarm.rs:3757-3781) | FIXED -- verify in terminal verdict | guardrail consume before request.peers touched |
| NEW-6 per-peer bucket Sybil bypass | LANDED (5b66f896) | FIXED pending terminal verdict | global TokenBucketState alongside per-peer; burst tuned 20->10 for alpha topologies, refill 2/s; revisit at farm scale |
| NEW (1b review) lost-update race + corruption | CLOSED pending terminal verdict | FIXED | save_lock + atomic durable writes (21095127); batch callers (d2497460); load sanitization + durable shrink + 5 persistence/concurrency tests (02efea70, Fusion: lock ordering/tmp race/shrink all SOUND); corrupt-JSON recovery + peer_id parity (v2c-3) -- legacy corrupt files quarantine instead of bricking startup |
| RESIDUAL sustained-burst anchor aging | DOCUMENTED | POST-ALPHA (invite-acceptance design; anchors earn success_count via real dials -> proven tier) | operator-accepted in GPT_SEEDING_REVIEW_RESPONSE_STAGE_1A.md |
| RESIDUAL cross-instance mobile LedgerManager on shared path | DOCUMENTED | POST-ALPHA (single shared manager or file locking; mobile-architecture work) | atomic rename bounds corruption to never; lost updates possible across instances |

Terminal requirement: every row is FIXED with evidence or carries an
explicit operator-signed decision before 040-S6.
