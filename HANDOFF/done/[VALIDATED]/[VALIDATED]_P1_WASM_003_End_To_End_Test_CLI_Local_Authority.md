<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Triage Decision -- 2026-06-08

**Status:** ready
**Bucket:** pending-dispatch
**Cross-ref:** see `HANDOFF/STATE/2026-06-08_UNIFIED_BACKLOG.md`
**Decided by:** Claude Code (Overseer) sweep

**Rationale:** Ticket is a real remaining work item with no shipped code on the
integration branch. No blocker identified. Ready for `/orchestrate` dispatch on
the next cloud slot allocation. Per Lucas directive 2026-06-08 "I want it all
fixed," this is part of the ~30-ticket remaining backlog.

---
# MODEL: qwen3-coder-next:cloud
# BUDGET: 1800
# token_budget: 18000

# P1_WASM_003_End_To_End_Test_CLI_Local_Authority

**Status:** VERIFIED REMAINING WORK (per user request 2026-06-04 + wasm_plan.md Phase 3 "Lobotomy & Bridge")
**Agent:** implementer
**Budget:** 1800s (MIXED tier)
**Phase:** v0.2.1 P1  Thin-Client UI integration test
**Source:** `wasm/wasm_plan.md` "WASM Lobotomy & Bridge Implementation" + user explicit request
**Depends on:** P0_BUILD_001

---

## Verified Gap

The WASM build artifact exists at `wasm/pkg/scmessenger_wasm_bg.wasm` and the source at
`wasm/src/` declares an `IronCore` wrapper with `startSwarm`, `sendPreparedEnvelope`,
`getPeers`, `drainReceivedMessages`, etc. Per `wasm/README.md` these are "browser-friendly
wasm-bindgen API" and `startSwarm(bootstrapAddrs)` is the entry point.

**However:** the `wasm_plan.md` "WASM Lobotomy" mandate is to STRIP all browser-side
identity/crypto and route everything through the CLI's local 127.0.0.1 WebSocket RPC.
Today the WASM module still imports `scmessenger-core` directly (look at
`wasm/Cargo.toml` line 15: `scmessenger-core = { path = "../core", features = ["wasm"] }`)
and re-exports the full API surface. Nothing in the test directory actually exercises
the local-authority contract: that the WASM code never generates or holds an Ed25519
identity.

The user explicitly requested: "add WASM as well, and test that the WASM respects the
CLI local authority".

## Scope (~150 LoC across 2 files)

### Part A: Add `wasm/test/authority_test.js` (LOC: ~100)

Verify the WASM bundle has NO identity-generation calls reachable from the JS surface:

```js
import { readFile } from 'fs/promises';

// Read the compiled .wasm and search for telltale strings
const wasmBytes = await readFile(new URL('../pkg/scmessenger_wasm_bg.wasm', import.meta.url));
const text = new TextDecoder('utf-8', { fatal: false }).decode(wasmBytes);

const FORBIDDEN = [
  'generate_ed25519',     // libp2p identity generation
  'x25519',                // key-agreement primitive  must be CLI-only
  'chacha20',              // symmetric cipher  must be CLI-only
  'random_secret',         // cryptographic RNG
  'ed25519_dalek::SigningKey::generate', // explicit
];

let failed = [];
for (const needle of FORBIDDEN) {
  if (text.includes(needle)) failed.push(needle);
}

if (failed.length) {
  console.error('FAIL: WASM bundle contains forbidden identity/crypto symbols:', failed);
  process.exit(1);
}
console.log('PASS: WASM bundle has no identity/crypto symbols  local authority is preserved');
```

### Part B: Wire a smoke test that hits the CLI RPC (LOC: ~50)

Add `wasm/test/rpc_smoke_test.js`:

```js
// This test must be run AFTER starting the CLI locally (port 9000).
// It exercises the RPC contract from the WASM side to prove the
// round-trip goes through the CLI's local authority, not browser crypto.

const CLI_RPC_URL = process.env.SCM_CLI_WS || 'ws://127.0.0.1:9000/ws';

async function rpc(ws, method, params) {
  return new Promise((resolve, reject) => {
    const id = Math.random().toString(36).slice(2);
    const handler = (evt) => {
      const msg = JSON.parse(evt.data);
      if (msg.id === id) {
        ws.removeEventListener('message', handler);
        msg.error ? reject(new Error(msg.error.message)) : resolve(msg.result);
      }
    };
    ws.addEventListener('message', handler);
    ws.send(JSON.stringify({ jsonrpc: '2.0', id, method, params }));
  });
}

const ws = new WebSocket(CLI_RPC_URL);
await new Promise((r) => ws.addEventListener('open', r));

const identity = await rpc(ws, 'GetIdentity', {});  // must come from CLI, not browser
assert(identity.peer_id && /12D3Koo/.test(identity.peer_id));

const peers = await rpc(ws, 'ScanPeers', {});     // must hit the CLI's mDNS/BLE
assert(Array.isArray(peers));

const topology = await rpc(ws, 'GetTopology', {}); // must come from CLI's mesh state
assert(topology.nodes || topology.peers);

console.log('PASS: WASM  CLI RPC round-trip works; local authority confirmed');
```

Wire it in `wasm/package.json` as a `test:authority` script that runs after
`wasm-pack build --target web`.

## File Targets

- `wasm/test/authority_test.js` [CREATE]
- `wasm/test/rpc_smoke_test.js` [CREATE]
- `wasm/package.json` [EDIT  add test scripts]
- `wasm/Cargo.toml` [VERIFY  features flag for local-authority-only mode is feasible; add a
  `local-authority` feature that strips `startSwarm` etc.]

## Build Verification Commands

```bash
# Pre-req: build WASM
cd wasm
wasm-pack build --target web --release

# Run the authority test
node test/authority_test.js

# Optionally run the smoke test (requires CLI on :9000)
node test/rpc_smoke_test.js
```

## Acceptance Gates

1. `node test/authority_test.js` exits 0  the compiled `.wasm` has NO identity/crypto strings
2. `node test/rpc_smoke_test.js` exits 0 against a running `scmessenger-cli.exe start`  round-trip
   through `ws://127.0.0.1:9000/ws` returns real peer/topology from the CLI's persistent state
3. The new tests are listed in `wasm/package.json` `scripts.test`
4. `wasm/Cargo.toml` `local-authority` feature compiles cleanly (currently stubbed)

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: JS] [REQUIRES: QWEN_CODER_NEXT] [DEPENDS_ON: P0_BUILD_001] [BUILDS_ON: wasm_plan.md]
