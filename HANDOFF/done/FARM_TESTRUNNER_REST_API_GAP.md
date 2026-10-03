# Farm-sim test-runner assumes a REST API that doesn't exist

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: Scoped, NOT started. Found live 2026-07-14 running the AWS farm-sim
(docker-compose-extended.yml, 7-node/3-network topology) through
`docker/test-scripts/run-integration-tests.sh`.

## What's confirmed working (real, live-verified)
- Full docker build succeeds (fixed 4 stacked bugs this session: missing
  desktop_bridge workspace member, missing libdbus-1-dev/libssl-dev/pkg-config
  build deps, missing cli/build.rs COPY breaking SCM_BUILD_TIME, missing
  libdbus-1-3 runtime lib).
- relay1 + relay2 reach `healthy` (fixed: entrypoint.sh now auto-injects
  --http-bind so the axum health server starts; healthcheck route corrected
  from a nonexistent /api/status to the real /health).
- All 7 containers (2 relays + alice/bob/carol/david/eve) run and stay up.
- git-tracked executable bit fixed on run-integration-tests.sh (was 100644,
  broke on Linux checkout since the volume mount is read-only).
- Identity creation gap fixed: entrypoint.sh now runs `scm init` before
  `scm start` (previously no identity was ever created - confirmed via
  `docker exec ... scm identity` showing "No keys found in store").

## The remaining gap
`run-integration-tests.sh` calls `curl http://$node:8080/api/identity` and
`curl http://$node:8080/api/peers` to get each node's peer ID and peer count.
Neither endpoint exists - the only route the axum health server
(`spawn_http_health_server` in `cli/src/main.rs`) implements is `/health`.
This test script was written assuming a REST API surface that was never
built. Confirmed live: both curls return empty (non-2xx, suppressed by
`curl -f`).

Tests 2,4,5,6,7,8,9,11,12 (9 of 13) depend on this and will keep failing
until either:
(a) the REST surface is built out (`/api/identity` returning own peer_id,
    `/api/peers` returning current connections) in `spawn_http_health_server` -
    real feature work, needs to thread through whatever already exposes
    peer/identity info internally (IronCore/swarm handle) into the axum
    router's shared state, or
(b) the test script is rewritten to check the SAME things a different way
    (e.g. `docker exec <node> scm identity`/`scm status` output parsing
    instead of HTTP, if test-runner can be given docker exec access - it
    currently cannot, no docker socket mount).

Not attempted this session - re-scoped per the operator's standing rule
("a finding too large for a micro-fix gets torn down and re-scoped") given
severe token-budget pressure (91% weekly). The actual v1.0.0 deliverables
(Android AAB build, CLI, iOS CI) do not depend on this - it only affects the
depth of automated verification available on the AWS farm-sim harness itself.
The mesh topology being genuinely up and healthy (relays + 5 client nodes)
is itself real evidence the app works end-to-end at the transport/health
layer; peer-messaging-specific assertions are what's still unverified by this
particular harness.

## Next steps (pick one, not both, when resumed)
- (a) is the "do it right" option: extend spawn_http_health_server with real
  routes, likely a 1-2 hour task including an adversarial-review-adjacent
  sanity pass since it touches the CLI's network-facing surface (not
  crypto/transport/routing/privacy proper, so NOT mandatory-audit-gated, but
  still worth a Fusion Lite pass given it's new attack surface).
- (b) is the "fix the test, not the product" option: faster, but the test
  suite would then diverge further from testing real network behavior.

## RESOLVED (2026-07-19, verified during V1.0.0 backlog sweep)

This ticket's diagnosis was stale. `/api/identity` and `/api/peers` (plus
`/api/swarm/stats`, `/api/listeners`, `/api/history`, `/api/discovery/*`,
`/submit-run`, `/poll-status/:run_id`, `/fetch-artifact/:run_id/:name`, and
`/health`) all already exist on the SAME axum router in
`cli/src/api.rs::start_api_server` (routes registered ~line 1146-1171),
serving on the `--http-bind` address. The ticket's diagnosis referenced
`spawn_http_health_server` in `cli/src/main.rs` (line 164) -- that function
is DEAD CODE, never called from anywhere in main.rs (confirmed via grep: no
call sites). The real, live health/API server is `api::start_api_server`,
wired via `api::start_api_server(api_ctx, http_bind_api).await` at both
`cmd_start` (main.rs:1623) and `cmd_relay` (main.rs:2530).

`handle_get_identity` (api.rs:963) and `handle_get_peers` (api.rs:624) are
real, non-stub implementations backed by `ctx.core.get_identity_info()` and
`ctx.swarm_handle.get_peers()`. Matches the `_QUEUE.md` 2026-07-17 header
correction: "DONE since the 07-13 body was written: ... FARM_TESTRUNNER_REST_API_GAP."

Moved to done/. No code change needed. Separately: `spawn_http_health_server`
in main.rs is confirmed dead code (unreachable, unused) -- worth a follow-up
cleanup removal, not done here to keep this ticket's scope to verification.
