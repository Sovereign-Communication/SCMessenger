# TASK: Onion routing FFI/RPC surface bypasses the onion_routing_enabled gate

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: TODO, LOWERED PRIORITY - verify-first step done 2026-07-13. Grepped
android/app/src/main, iOS/SCMessenger/SCMessenger (excluding Generated/), and
log-visualizer/browser for any UI callers of prepare_onion_message/
peel_onion_layer (or their camelCase FFI names): zero hits outside the
auto-generated UniFFI bindings themselves (iOS Generated/api.swift, apiFFI.h -
which every FFI method appears in regardless of whether anything calls it).
No Android Kotlin, iOS Swift app code, or WASM/browser JS actually invokes these
entry points today. This is exposed-but-unused surface, not a live exploit path -
defensive gating (per the fix direction below) is still worth doing before any
future UI wires it up, but this is not an active farm-blocking risk.

Status: TODO. Found 2026-07-13 while writing the AD-8 seam-freeze test
(`core/tests/seam_freeze_onion.rs`, FARM_FINAL_PLAN.md WS-FARM-H1).

## Finding

AD-8 states onion routing has "ZERO call sites from the live send path" beyond
one documented, config-gated wiring point: `core/src/iron_core.rs`'s
`prepare_message_internal` calls `self.prepare_onion_message(...)` only when
`self.privacy_config().onion_routing_enabled` is true (default `false`, see
`core/src/privacy/mod.rs`).

However, `IronCore::prepare_onion_message` / `peel_onion_layer` are `pub fn` on
`IronCore` itself, and TWO other surfaces call them directly, bypassing that
gate entirely:
- `core/src/mobile_bridge.rs:1590,1598,1603,1611` - UniFFI-exposed methods that
  call `core.prepare_onion_message(...)` / `core.peel_onion_layer(...)`
  unconditionally, reachable from Android/iOS via the generated bindings.
- `core/src/wasm_support/rpc.rs:347,373` - JSON-RPC method dispatch (`"prepare_onion_message"`,
  `"peel_onion_layer"` match arms) reachable from the browser/WASM client over
  the `/ws` JSON-RPC bridge.

Neither surface checks `onion_routing_enabled` before invoking onion
construction/peeling. This is NOT the same concern as AD-8's "live send path"
(the automatic wiring during normal message sending) - this is a manually-
invokable API surface. Unknown today: does the actual Android/iOS/WASM UI code
ever call these FFI/RPC entry points? If not called by any UI path, this is
unused surface, not a live exploit - but it means a compromised or malicious
platform client could invoke onion routing directly regardless of the
farm-safety default.

## What's needed

1. Verify-first: grep the Android Kotlin / iOS Swift / WASM JS client code for
   actual callers of these FFI/RPC entry points. If none exist, this is exposed-
   but-unused surface (lower priority, still worth gating defensively).
2. Decide: should these FFI/RPC entry points also check
   `self.privacy_config().onion_routing_enabled` and return an error/no-op when
   disabled (consistent with the internal gate), or are they intentionally a
   manual/debug API surface? [OPUS+/THINK - policy decision, not just a wiring fix]
3. If gated: add the same `onion_routing_enabled` check to both call sites,
   with a clear error message distinguishing "onion routing disabled" from
   other failure modes.

## Gate

Touches `core/src/privacy/` adjacency and mobile/wasm bridges - standard review
for correctness, not a crypto-primitive change. Low urgency (not farm-gating
per AD-8's actual scope) but should not be forgotten - tracked here instead of
silently expanding the seam-freeze test's whitelist.
