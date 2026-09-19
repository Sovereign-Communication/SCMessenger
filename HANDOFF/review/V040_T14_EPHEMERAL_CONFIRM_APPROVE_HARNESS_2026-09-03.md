# V040 T14-EPHEMERAL CONFIRM-APPROVE -- HARNESS free lane -- 2026-09-03

Target: PR #270 freebuff/v040-t14-ephemeral-port
Reviewed head: 6fd0230b (re-verified live on origin at filing time)
Lane: harness free lane (panel-based, deterministic convergence)
Closes: FLAG-2 of V040_POST_VALIDATION_MERGE_PLAN_2026-09-03.md
Verdict: APPROVE (all confirm claims found FALSE = no real defects at head)

## Panel

- google/gemma-4-31b-it:free
- minimax/minimax-m3:free
- inclusionai/ling-3.0-flash-fin:free (truncated pre-JSON; reasoning
  leans false -- excluded from the deterministic tally per protocol)
- judge: cohere/north-mini-code:free (convergence specialist step)
- Cost: $0.000000 total (free tier, ceiling enforced)

## Claims manifest (defect propositions) and per-claim tally

Manifest + verbatim source window: tmp/harness/w270-claims.json +
tmp/harness/w270-window.txt (83 lines, sections E-G from swarm.rs at
6fd0230b: consensus guard 4051-4082, Identify guard 5183-5213, cfg
branches of start_swarm_with_config at 2786 [native] and 6881 [wasm]).

| claim | defect proposition | gemma | minimax | ling | tally |
|---|---|---|---|---|---|
| c1 | promotion sites execute in wasm32 builds | false (0.9) | false (0.85) | (truncated) | 2/2 false |
| c2 | guard advertises on empty listen_ports | false (1.0) | false (0.95) | (truncated) | 2/2 false |
| c3 | wasm32 branch has an add_external_address path | false (1.0) | false (0.85) | (truncated) | 2/2 false |

Deterministic convergence: agreement=high, confidence=1.0, defer=false.
Judge synthesis: "Wasm nodes cannot promote addresses because the
promotion guards are outside the wasm32 cfg branches and the wasm32 code
lacks the necessary add_external_address calls." Full raw output:
tmp/harness/w270-verdict.json.

## Evidence commands (run at filing)

- git grep -c 'add_external_address' 6fd0230b -- core/src/transport/swarm.rs
  -> 2 occurrences ONLY: :4075 (consensus guard) and :5203 (Identify
  guard); both inside the #[cfg(not(target_arch = "wasm32"))] branch of
  start_swarm_with_config (native block opens :2786; wasm twin :6881).
- Both guards: advertise ONLY when listen_ports.contains(&primary.port());
  on an empty listen set contains() is false, so nothing is advertised --
  fail-closed. The else-if only warns and only when the set is non-empty.
- The wasm branch (6881+) contains no add_external_address call; its
  event loop has diagnostics-only parity (Identify :7956, address
  reflection :7584).

## Human double-check (lane operator, independent of panel)

The prior Critical finding ("empty listen_ports on wasm never advertises")
is confirmed FALSE at 6fd0230b: the two promotion sites are inside the
native cfg block; file-wide add_external_address count = 2, both native;
wasm has zero promotion calls; the guard fails closed on an empty listen
set. Matches the CTO-side disposition (including the corrected addendum).

## Autonomy ledger

- Chain verified: true (harness ledger verify)
- Entries: task_id scmessenger-270-confirm (panel + judge), chain head
  hash 18ef25607932f43b08a986b8a3dda3578e22ec2b4dec8e6dcde2c5cb11459d9b
  (seq 172, ledger verify independently checkable)

Verdict: APPROVE for PR #270 @ 6fd0230b.