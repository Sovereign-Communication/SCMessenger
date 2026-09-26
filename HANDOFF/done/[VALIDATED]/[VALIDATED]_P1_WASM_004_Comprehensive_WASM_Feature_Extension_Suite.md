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

# P1_WASM_004_Comprehensive_WASM_Feature_Extension_Suite

**Status:** VERIFIED REMAINING WORK (per user request 2026-06-04: "test WASM as full extension to ensure all features work")
**Agent:** implementer
**Budget:** 1800s (MIXED tier)
**Phase:** v0.2.1 P1  WASM full-feature coverage
**Source:** `wasm/src/lib.rs` exported API + `wasm/README.md` list
**Depends on:** P0_BUILD_001, P1_WASM_003

---

## Verified Gap

The `wasm/src/lib.rs` exports (per `wasm/README.md` and `wasm/pkg/scmessenger_wasm.d.ts`):

- `IronCore` (constructor, withStorage, start, stop)
- `startSwarm(bootstrapAddrs)`
- `stopSwarm()`
- `sendPreparedEnvelope(peerId, envelopeBytes)`
- `getPeers()`
- `getConnectionPathState()`
- `exportDiagnostics()`
- `drainReceivedMessages()`

There is no Node-based test driver for these. `wasm/test/notification_test.js` and
`wasm/test/verification_test.js` exist but neither covers the libp2p-swarm-backed
functions. The user requested: "test WASM as full extension to ensure all features
work as intended."

## Scope (~250 LoC across 3 files)

### Part A: Headless Node test driver (LOC: ~120)

Create `wasm/test/full_feature_test.js`. Use a fake `WebSocket` and `WebWorker` polyfill
so the WASM can run in Node for the parts that don't need actual networking. Stub out
`getRandomValues` to return deterministic bytes. Then exercise:

```js
import init, {
  IronCore,
  startSwarm, stopSwarm,
  sendPreparedEnvelope,
  getPeers, getConnectionPathState, exportDiagnostics, drainReceivedMessages,
} from '../pkg/scmessenger_wasm.js';

await init();

// IronCore lifecycle
const core = IronCore.new('test-storage');
assert(core !== null, 'IronCore constructed');
core.start();
const state = core.exportDiagnostics();
assert(state.running === true, 'IronCore started');

// Swarm (will fail to dial any real peer, but must not panic)
try {
  await startSwarm([]);   // empty bootstrap, no panic
} catch (e) { /* expected: no bootstrap */ }
try { stopSwarm(); } catch (e) {}

// Prepared envelope round-trip
const env = core.prepareMessage('00'.repeat(32), 'hello wasm', 0, null);
assert(env && env.envelope_data && env.message_id, 'prepared envelope has fields');

// Diagnostics shape
const diag = core.exportDiagnostics();
for (const key of ['connection_path_state', 'listeners', 'external_addrs', 'peers', 'history_stats']) {
  assert(key in diag, `diagnostics has ${key}`);
}

console.log('PASS: WASM full-feature smoke test');
```

### Part B: Deterministic WebRTC adapter test (LOC: ~80)

`wasm/src/transport.rs` declares WebRTC support (look for `RtcPeerConnection` in
`web-sys` features). Add `wasm/test/transport_stub_test.js` that verifies the transport
module's public surface compiles in a Node environment (the actual WebRTC connection
can't be tested without a browser; just verify the module is reachable and accepts
the correct multiaddrs):

```js
import init, { startSwarm } from '../pkg/scmessenger_wasm.js';
await init();

const CASES = [
  ['/ip4/127.0.0.1/tcp/9000/ws',           'websocket'],
  ['/ip4/192.168.0.230/tcp/9101/p2p/12D3K', 'tcp'],
  ['/ip6/::1/tcp/9101',                     'tcp'],
];

for (const [addr, expectedProto] of CASES) {
  try {
    await startSwarm([addr]);
    console.log(`OK: accepted ${addr} (${expectedProto})`);
  } catch (e) {
    // Connection will fail, but parse error must be the only failure
    assert(/dial|connection|timeout/i.test(e.message),
      `unexpected error for ${addr}: ${e.message}`);
    console.log(`OK: ${addr} parse-accepted (dial failed as expected)`);
  }
}
```

### Part C: Notification permission flow (LOC: ~50)

Add `wasm/test/notification_full_test.js` covering the `Notification` API path that
`notification_manager.rs` exposes:

```js
// Verify the WASM module imports the Notification web-sys feature
// and that the export names match what notification_manager expects
import { readFile } from 'fs/promises';
const wasmBytes = await readFile(new URL('../pkg/scmessenger_wasm_bg.wasm', import.meta.url));
const text = new TextDecoder('utf-8').decode(wasmBytes);
assert(text.includes('Notification'), 'WASM imports Notification web-sys feature');
assert(text.includes('requestPermission') || text.includes('permission'),
  'WASM has permission-state import');
console.log('PASS: WASM notification feature wired');
```

## File Targets

- `wasm/test/full_feature_test.js` [CREATE]
- `wasm/test/transport_stub_test.js` [CREATE]
- `wasm/test/notification_full_test.js` [CREATE]
- `wasm/package.json` [EDIT  add to `scripts.test`: `test:full` runs all of the above]

## Build Verification Commands

```bash
cd wasm
wasm-pack build --target web --release
npm install --save-dev mocha  # or use the existing test runner
npm run test:full
```

## Acceptance Gates

1. `wasm/test/full_feature_test.js` exits 0  IronCore lifecycle, prepared envelope, diagnostics shape all work
2. `wasm/test/transport_stub_test.js` exits 0  multiaddr parsing accepts tcp/ws/ip6
3. `wasm/test/notification_full_test.js` exits 0  Notification web-sys feature is in the .wasm
4. `npm run test:full` runs all 3 in sequence and exits 0
5. Tests are committed in `wasm/test/` and referenced from `wasm/package.json`

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: JS] [REQUIRES: QWEN_CODER_NEXT] [DEPENDS_ON: P1_WASM_003]
