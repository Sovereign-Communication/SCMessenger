# V040 CTO handoff -- architecture verification and three-node validation

Status: ACTIVE -- operator/CEO handoff to CTO
From: CEO seat
To: CTO seat
Date: 2026-09-02

## Objective

Continue from the current shared checkout without broad restructuring. First verify the focused architecture pass. Then coordinate a three-node validation run using one exact candidate SHA across Windows CLI, AWS cloud node, and Android Pixel.

The release goal remains: last required change landed and reviewed, all three nodes rebuilt from the same target, then functional validation before any v0.4.0 tag decision.

## Files to read first

1. `docs/ARCHITECTURE_SCOPE_V040.md` -- resulting ownership and data-flow boundaries.
2. `HANDOFF/freebuff/inbox/V040_CEO_HANDOFF_2026-09-01.md` -- current lane status and review dispositions.
3. `HANDOFF/freebuff/queue/V040_CTO_HANDOFF_2026-09-01.md` -- prior CEO/CTO release alignment.
4. `HANDOFF/freebuff/README.md` -- queue order and lane rules.
5. `AGENTS.md` and `CLAUDE.md` -- shared-checkout, build, security-review, and node terminology rules.

## Current architecture-pass changes

The shared checkout currently contains these focused product changes:

- `core/src/transport/observation.rs`: `AddressObserver` owns listen-port admission, fails closed with an empty listener set, removes observations that become ineligible, and orders consensus deterministically.
- `core/src/routing/local.rs`: hint-specific and all-active peer selection share one reliability comparator with peer-ID tie-breaking.
- `core/src/routing/optimized_engine.rs`: removed duplicated `local_id` and `local_hint`; `RoutingEngine`/`LocalCell` remain the owner.
- `core/src/iron_core.rs`: records the composition-root and state-flow rule.
- `docs/ARCHITECTURE_SCOPE_V040.md`: durable architecture record.

Do not rework Android navigation, `IronCore` decomposition, or repository-wide scaffolding during this validation pass. Those are separate architecture projects and would conflict with the release gate.

## Verified locally in this session

Commands and observed results:

```text
cargo fmt --check
[OK]

cargo test -p scmessenger-core --lib observation -- --nocapture
6 passed; 0 failed

cargo test -p scmessenger-core --lib optimized_engine -- --nocapture
7 passed; 0 failed

cargo test -p scmessenger-core --lib local -- --nocapture
34 passed; 0 failed

git diff --check
[OK]
```

The required workspace compile command was also run:

```text
cargo test --workspace --no-run
[FAILED]
```

It failed in the shared target state with incompatible/missing dependency artifacts (`match_lookup` staticlib instead of rlib, missing `scmessenger_core`, and cascading compiler errors). Do not call this a source-level regression without reproducing from a clean, safe build state. Do not use destructive `cargo clean`; follow `scripts/clean_target.sh` rules and coordinate if target repair is needed.

A pre-existing warning remains at `core/src/transport/swarm.rs:8263` for unused `try_envelope_hint_dial`.

## CTO work order

### 1. Acknowledge and isolate

- Confirm the actual candidate base with `git rev-parse HEAD` and `git rev-parse origin/main`.
- Confirm no other build is running before launching a build.
- Do not modify or clean unrelated shared files.
- Keep the architecture-pass files explicit; do not stage broad paths.

### 2. Verify the architecture pass

Run, capturing raw output:

```text
cargo fmt --check
cargo test -p scmessenger-core --lib observation -- --nocapture
cargo test -p scmessenger-core --lib local -- --nocapture
cargo test -p scmessenger-core --lib optimized_engine -- --nocapture
cargo test --workspace --no-run
```

If the workspace compile gate hits the same artifact corruption, repair only through the approved safe target procedure and rerun. The authoritative Windows result is required.

### 3. Run the three-node validation

Use the exact same candidate SHA for all nodes:

- Windows CLI: operator-driven build/start, diagnostics, and log capture.
- AWS: CTO-driven dynamic deployment; never hardcode the AWS address. Discover it through the existing deployment tooling.
- Android Pixel: install the matching artifact and capture service/log diagnostics.

The CTO owns AWS deployment and collection of AWS evidence. The operator owns the Windows driver. Android is verification-only during this run; no code authoring on the handset.

### 4. Required evidence

Create one timestamped run record under the approved local evidence area, with links to raw artifacts. It must record:

- exact candidate SHA and build provenance for each node;
- node identity and dynamically discovered address;
- startup and coordinated-restart timestamps;
- Windows `[SEED-DIAL]` output;
- peer/ledger propagation across nodes;
- diagnostics `external_addrs` with no non-listen/ephemeral source-port entry;
- successful inbound reachability where the environment permits it;
- AWS redeploy/IP churn and unaided re-mesh result;
- Android install/start/join evidence;
- every command used and its exit status captured before any pipe/filter;
- explicit PASS, FAIL, or UNVERIFIED for each gate.

A claim in a handoff is not evidence unless the raw command output or CI/run URL is attached.

## Conflict-control rules

- Do not move, rename, delete, or mass-edit `HANDOFF/`, `.codebuff_deploy/`, or `scratch/driver/`.
- Do not touch another agent's uncommitted product files.
- Do not merge or tag from this handoff without the applicable review and release decision.
- Changes under `core/src/{crypto,transport,routing,privacy}` still require a non-author adversarial APPROVE on file. This architecture pass touches transport and routing; do not treat local tests as the Rule-8 gate.
- If the candidate SHA, fleet state, signing posture, or persisted-state assumption is uncertain, stop and report the blocker rather than improvising.

## Return format

Write a completion note to `HANDOFF/freebuff/inbox/` beginning with:

```text
Task: V040_CTO_ARCHITECTURE_AND_3NODE_HANDOFF_2026-09-02.md
Type: DONE | BLOCKED
```

Include exact commands/output references, the candidate SHA, each node's observed state, failures, and the next decision required from the CEO seat.
