# V040 CTO three-node checkpoint - T14 configured-external-address fix (local)

## Metadata

- Stage: `T14_EXTERNAL_ADDR_FIX_LOCAL` (code-level; NOT a three-node completion stage)
- UTC timestamp: `2026-09-08T23:20:00Z`
- Operator/session: Freebuff `/cto`; operator rulings in force: all work locally,
  no dispatch, no worker worktrees; do not touch android/; do not modify Windows
  firewall settings or the running node's state
- Branch: `cto/t2-disk-ruling-2026-08-31`
- HEAD at implementation: `ba474a7a1cd1b5e7ef87cdbc1c4a7069e091784b`
- Defect (T14, P0): the node advertises an ephemeral/NAT-mangled observed port as
  its external address, so remote peers (the Pixel) cannot dial it and
  store-and-forward delivery to offline/other-network peers fails.

## Fix (local implementation, shared checkout)

Configured-external-address primacy. The external address peers learn comes from
a new config knob `external_addr` (host:port); when set it wins over every peer
observation. No `start_swarm_with_config` signature change (8 call sites
untouched); plumbing is a new `SwarmCommand::SetConfiguredExternalAddress` +
`SwarmHandle::set_configured_external_address()`.

Files changed:

- `core/src/transport/observation.rs` (+52/-1): `AddressObserver` gains
  `configured_external: Option<SocketAddr>` + `set_configured_external()`;
  `recalculate_consensus()` pins the configured address first and deduplicates
  it, so `primary_external_address()` - the value both promotion sites
  (AddressReflection consensus, Identify observed_addr) and the diagnostics
  API read - is always the configured endpoint. Unit test
  `configured_external_wins_over_consensus` added.
- `core/src/transport/swarm.rs` (+28): new `SwarmCommand` variant handled in
  the native loop (registers the configured address via
  `swarm.add_external_address` immediately) and the wasm loop (observer state);
  new `SwarmHandle::set_configured_external_address()`.
- `cli/src/config.rs` (+51): `external_addr: Option<String>` config knob
  (serde default None), `set`/`get`/`list` support, malformed values fail
  closed; roundtrip/validation unit test added.
- `cli/src/main.rs` (+35): both production startup paths (`cmd_start` and
  `cmd_relay`) parse `config.external_addr` and push it into the swarm right
  after `start_swarm_with_config`, before any reflection traffic.
- `core/tests/test_configured_external_address.rs` (new, 74 lines): swarm-level
  regression through the real command channel - configured address is the
  primary external address immediately, and clearing it removes it.

Diff stat: 4 files changed, 165 insertions(+), 1 deletion(-), plus the new test file.

SHA256 (on-disk, post-edit):

- `core/src/transport/observation.rs`
  `defe8b9218c2e4b82a24e11f737dbd110360d1960e16978c6ce7a37ba17f25c7`
- `core/src/transport/swarm.rs`
  `9bd53eac4243ead8aa40d70e550c28a941d7243d654af0ba760308551a3cd61e`
- `cli/src/config.rs`
  `480db519710cf67c0e28b5e1d1396e6f09d64703f6b2db0c11468490c0a51f9c`
- `cli/src/main.rs`
  `3a26355d84a880ac0eb0d6b20e2e152939d88d8eafa88aa45086a386d871a888`
- `core/tests/test_configured_external_address.rs`
  `1478be51ac5bcb00130bbc7eae5a0fc552cb0ba5c533fce0bbc38cce09272f77`

## Gates (authoritative Windows host, under scripts/build_lock.py, holder `cto-t14`)

| Gate | Command | Result |
| --- | --- | --- |
| 1. Observer unit tests | `cargo test -p scmessenger-core --lib transport::observation -- --nocapture` | PASS: 6/6 ok, 0 failed (1390 filtered) - `tmp/cto/T14_GATE_1_OBSERVER_CONFIG_20260908T230550Z.log` |
| 2. Swarm-level T14 regression | `cargo test -p scmessenger-core --test test_configured_external_address -- --nocapture` | PASS: 1/1 ok - `tmp/cto/T14_GATE_2_SWARM_CONFIG_20260908T230838Z.log` |
| 3. CLI config tests | `cargo test -p scmessenger-cli --lib config:: -- --nocapture` | PASS: 4/4 ok (incl. new roundtrip/validation test) - `tmp/cto/T14_GATE_3_CLI_CONFIG_20260908T230838Z.log` |
| 4. Wasm compile check | `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown` | PASS (RC 0; pre-existing warnings only) - `tmp/cto/T14_GATE_4_WASM_CHECK_*.log` |

Only compile warning in Gate 1 is pre-existing (`try_envelope_hint_dial` unused
import, swarm.rs:8296) and untouched by this diff.

## Verdicts (explicit)

- T14 code-level fix (configured external address wins over observations):
  **PASS** on the Windows host gates above.
- Regression test proving the advertised/registered address keeps the
  configured external port: **PASS** (Gate 2 + observer unit test in Gate 1).
- Live node delivery of store-and-forward to offline/other-network peers:
  **UNVERIFIED** - requires rebuild + redeploy of the Windows node binary and a
  live three-node delivery run. Nothing is claimed deployed.
- Wasm behavior (not just compile): **UNVERIFIED** - wasm loop records the
  configured address for observer state only (parity with the existing
  diagnostics-only wasm design).

## Rule-8 gate (adversarial review)

This diff touches `core/src/transport/` (observation.rs, swarm.rs). Per
`docs/rules/FREEBUFF.md` and AGENTS.md rule 8, a merge to main requires a
recorded adversarial review by a reviewer who did not author the change. This
session authored the change and CANNOT satisfy that gate itself. Required
before merge: independent reviewer APPROVE, review verdict filed under
`HANDOFF/review/`. Suggested review focus: (1) configured address must not
bypass the listen-port allowlist semantics (it intentionally may differ from a
bound port - that is the point of port-forwarded NAT - but reviewers should
confirm that is acceptable); (2) clearing the knob retracts primacy but does
not remove the address from libp2p's registry; (3) no new unbounded state.

## Node relaunch configuration (so the fix is live)

The running Windows node still carries the pre-fix binary. To activate:

1. Rebuild: `python scripts/build_lock.py --run "cargo build --release -p scmessenger-cli" --holder cto-t14`
2. Set the config knob (file `%APPDATA%\scmessenger\config.json`):
   `"external_addr": "147.81.41.188:9001"` - the current public IP and the
   listen port from live diagnostics (`147.81.41.188:9001` was reported by the
   running node's external-address observation). Or via CLI once supported:
   `scmessenger config set external_addr 147.81.41.188:9001`.
3. Relaunch the Windows relay node with the same listen/API ports as the
   current live node (P2P 9001/9002 ownership per earlier re-derivation) and
   confirm `/api/diagnostics` `external_addrs` leads with `147.81.41.188:9001`.
4. The AWS cloud node needs the same treatment only if its observed address
   differs from its configured public endpoint; current live diagnostics
   report it healthy - re-derive before relaunch.

Operator note: relaunching the Windows node is an operator-approved action
(the running node's state was explicitly declared off-limits this session), so
relaunch is queued, not executed here.

## Provenance and coordination

- Implementation: this CTO session, local shared checkout, per operator ruling
  (supersedes the earlier dispatch flow; that evidence is preserved at
  `tmp/cto/BLE01_DISPATCH_EVIDENCE/`).
- CEO coordination: check-in appended to `HANDOFF/CEO_STATE.md` referencing
  this checkpoint.
- Next steps toward the 3-node test: operator relaunch of the Windows node
  with `external_addr` set; rebuild/deploy the BLE-01 scanner fix APK (prior
  checkpoint `..._20260908T224800Z_BLE01_SCANNER_FIX_LOCAL.md`); then the
  package's live BLE and offline-custody delivery gates.
