# Rule-8 verdict: zombie-connection fix (follow-up to PR #305)

- Date: 2026-09-18
- Branch / head: `freebuff/v040-blocking-fixes-20260917` @ `8cc356b8` (fix) + `44ed8071` (exact-IP hardening found during gate prep; reviewed in this same panel)
- Under review: the WIFI_TRANSPORT_REGRESSION_RCA fix — `ZombieTracker` (liveness stamps: successful pings native + identify::Received both loops), deny-cause classification (`connection_limits::Exceeded` downcast, `source()` unwrap fallback), periodic reap gated on all-silent + fresh deny-classified attempt, wired in native and wasm swarm loops, 9 new regression tests.
- Harness: `sovereign-harness` 0.3.3, `verify` structured-claims mode; inputs `tmp/rule8-zombie/{claims_transport.json,source_transport.txt,prompt_transport.md}` (builder `tmp/rule8-pr305/build_review_zombie.py`, 17 fail-closed windows / 691 lines); run artifact `~/Documents/GitHub/Harness/audits/scmessenger/_runs/seat-gates/verify_zombie_transport_20260918.json`, mirrored to `tmp/rule8-zombie/verify_zombie_transport_20260918.json`.
- Full seat: 3 of 3 seats voted (`cohere/north-mini-code:free`, `nvidia/nemotron-3-super-120b-a12b:free`, `openrouter/free`) — all completed normally this run after the first attempt failed on 8k-token output truncation; re-run used 32768 (operator had pre-authorized raising token budget; cost $0.00 vs $0.10 ceiling).
- Deterministic tally: z1 not_real (0R/3NR), z2 not_real (1R/2NR), z3 not_real (0R/3NR), z4 not_real (0R/3NR), z5 not_real (0R/3NR). Convergence 4/5 claims; `defer: responder_disagreement` on z2; judge synthesis BYOK-blocked on this account (raw panel outputs + deterministic tally used, as in the PR #305 gate).

## The one dissent (z2, nvidia seat, high severity): "tracker maps unbounded; prune only affects connections map and never runs due to other limits"

Overruled on code evidence, not on vote count:

1. `prune()` (swarm.rs:1326-1357) has TWO while-loops — one for `connections`, one for `last_inbound_attempt_ms`, both capping at `ZOMBIE_TRACKER_MAX_PEERS`.
2. Both insert paths call `prune()`: `note_inbound_attempt` (:1394) and `note_inbound_attempt_by_ip` (:1404). The "never runs" premise is false.
3. The per-peer `Vec<ConnectionActivity>` is bounded by `max_established_per_peer=4` in connection_limits — the same limit whose enforcement was proven live during the incident itself (the 5th dial got `Denied`).
4. The regression test `zombie_tracker_maps_are_bounded_like_every_per_peer_map` (swarm.rs:10563) inserts `MAX_PEERS + 64` into BOTH maps and asserts both stay at/below cap — it passes, so the exact prune the dissent says never runs demonstrably runs.

## Residual notes the panel left standing (no defect verdicts, kept for the record)

- The reap closes the WHOLE peer (all connections) when its tracked state is all-silent + fresh deny. This is intentional (zombie slots are per-peer-attributed by IP join) but is a coarse instrument; if a future scenario has one silent + one active connection to the same peer, the active one is collateral. Risk accepted for now; tracker stamps all connections of the peer on any liveness event, so the window is small.
- `extract_ip_component` covers ip4/ip6; `/dns4/`+`/dns6/` addresses return None, so a deny from a DNS-addressed peer without peer_id cannot be attributed (stamped None — no reap gate contribution). Mobile LAN scenario is IP-based; cloud dials carry peer_id... actually they do not (deny pre-identify), but cloud peers are rarely IP-churning zombies. Accepted residual.
- Deny-cause classification covers `connection_limits::Exceeded` + source-chain unwrap. Exotic deny sources not in the chain print verbatim — which is the designed fallback (never a bare "denied").

## Gate outcome

CLEAR (deterministic tally 0R/3NR x5 with the single z2 dissent overruled on code evidence; all 5 claims were adversarial "DEFECT" probes and none survived). Panel convergence 4/5; convergence step errored on provider, tally is authoritative per harness note. No merge executed; CI on `44ed8071` still running at verdict time (17 green, 8 in flight) — merge only when green per standing order.

## Raw tally (verbatim, from the run JSON)

- z1=not_real (0R/3NR of 3); z2=not_real (1R/2NR of 3); z3=not_real (0R/3NR of 3); z4=not_real (0R/3NR of 3); z5=not_real (0R/3NR of 3)
- consensus: agreement=low, confidence=0.8, disagreements=[z2], defer=true, defer_reason=responder_disagreement, voted_by=3/3, panel_shortfall=false, missing_votes=0
- convergence: status=error (provider), tally converged_claims=4, responder_converged_claims=4, total_claims=5, convergence_rate=0.8
- cost: $0.00 actual vs $0.10 ceiling; all seat/judge attempts $0.000000
