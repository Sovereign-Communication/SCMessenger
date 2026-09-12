# DONE -- PR #274: kill the 5-minute recycle (dial dispatch self/connected guards)

TYPE: task
STATUS: DONE (PR open, Rule-8 APPROVED -- merge authority is the CEO seat)

## What shipped

PR https://github.com/Sovereign-Communication/SCMessenger/pull/274 on branch
freebuff/v040-nimble-peer (base cto/v040-candidate-2026-09-02, i.e. on top of
#273 which is MERGED into that line). 5 files: core/src/transport/swarm.rs
(+~450), core/Cargo.toml + Cargo.lock (if-addrs native-only), cli/src/ledger.rs
(complete_dial_skipped), cli/src/main.rs (skipped:-prefix handling).

Root cause (established from 3-node logs, not theory): every ~300s each node's
periodic re-dial loop (CLI 120s sweep x ledger BACKOFF_LADDER 300s cap) dialed
poisoned ledger entries attributing OUR OWN listeners to other peers. The
dials landed on ourselves (127.0.0.1/::1 + own ports): negotiation failed, the
sockets aborted (yamux 10053 closes at :50), and AutoNAT probe dials (300s
retry_interval) dead-marked the peer at :15. #273 fixed the dead-mark side;
#274 kills the self-dial + connected-peer redial at the dispatch point.

Fix: one dispatch owner (dial_skip_reason) in both SwarmCommand::Dial arms +
seed path: skip target-is-self, skip peer-already-connected (respond over the
existing link -- the user's directive), skip own-socket addresses
(OwnSockets: normalized IPs, TCP/UDP port separation, interface IPs, circuit
forms excluded, trusted Wi-Fi Aware proxy dials exempt). Skipped dials reply
Err("skipped:") and the CLI ledger releases claims neutrally.

## Gates (Windows host, head 6764e2b0)

- cargo check -p scmessenger-core --all-targets: clean
- cargo test -p scmessenger-core --lib: 1403 passed / 0 failed
- cargo test -p scmessenger-cli --lib: 83 passed / 0 failed
- cargo check -p scmessenger-wasm --target wasm32-unknown-unknown: PASS
- cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments: clean
- cargo fmt --check: clean

## Rule-8

Four qwen-lane review rounds (qwen3.8-2.4t-a95b): R1 A1-A10 -> R2 C1-C8 ->
R3 E1-E9 -> **R4 APPROVE** at 6764e2b0. Findings log + dispositions in the PR
body and HANDOFF/review/V040_NIMBLE_RECYCLE_REVIEW_QWEN_2026-09-03.md.

## Next decision (CEO seat)

Merge #274 into cto/v040-candidate-2026-09-02 (post-#273), then re-run the
3-node live window to observe the recycle gone (no :50 closes, no :15
dead-marks, custody drains while peers are connected). Not merged, not tagged.
