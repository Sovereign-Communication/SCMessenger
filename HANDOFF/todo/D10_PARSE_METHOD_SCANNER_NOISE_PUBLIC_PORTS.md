# D10 — Windows node binds public 0.0.0.0:80/443/8080/9002/9090; internet scanners hitting the HTTP server produce recurring hyper::Error(Parse(Method)) noise

Status: Todo
Priority: MEDIUM (cosmetic log noise plus unintended public exposure; no crash)
Filed: 2026-09-23, Buffy (Freebuff lane)
Found by: live Windows node observation during JEV dogfood, 2026-09-23

## Evidence (all commands run this session, 2026-09-23)

1. Old node (v0.4.0 `56d66f7`) stdout: 96 occurrences of
   `ERROR warp::server::run: server connection error: hyper::Error(Parse(Method))`
   over ~18h (`grep -c "Parse(Method)" tmp/radio-candidates/56d66f7/node-stdout.log`
   -> 96), i.e. roughly 5/hour, NOT the ~10s cadence an earlier compressed
   window suggested. Each hit often pairs with a `yamux` os error 10053 abort.
2. New node (v0.4.0 `1ec0c24`, D9-degrade build) listens on:
   `netstat -ano | grep 8324 | grep LISTEN` -> 0.0.0.0:80, 0.0.0.0:443,
   0.0.0.0:8080, 0.0.0.0:9002, 0.0.0.0:9090, 0.0.0.0:59004 (ephemeral),
   127.0.0.1:9001 (Warp HTTP+WS), 127.0.0.1:9876 (Control API).
   Log confirms: "Starting multi-port adaptive listening" with
   /ip4/192.168.0.121/tcp/443, /tcp/80, /tcp/9002/ws among the 10 libp2p
   listen addresses.
3. Established outbound connections during observation were the cloud nodes
   (18.234.62.247:9001 x2, 13.217.204.112:9090) — normal mesh, not the prober.
4. After restart, the fresh node produced zero Parse(Method) hits in the first
   minutes; the old log's cadence is consistent with periodic internet/LAN
   scanning of the public ports (TLS ClientHello bytes fed to a plain-HTTP
   parser is the classic Parse(Method) trigger).

## Defect

The multi-port adaptive listener binds well-known ports (80, 443, 8080) on all
interfaces. Any scanner connecting to them speaks TLS or garbage; the node's
HTTP server logs an ERROR per attempt. Two problems:

- Log noise: recurring ERROR lines that look alarming and pollute JEV bucket
  sorting (sorted `unmatched` honestly, but each occurrence costs a sort).
- Exposure: a mesh node advertising 0.0.0.0:443 invites unsolicited inbound
  from the entire LAN/internet scope. libp2p connection limits held (0 DENIED
  so far on the new build), but the surface is larger than a LAN-first design
  needs.

## Acceptance criteria

1. Identify the intended scope of the adaptive port set (LAN-first vs
   internet-reachable) with the operator — this is a design decision, not a
   code fix.
2. If ports are meant to be public: classify Parse(Method) noise down to a
   rate-limited WARN/DEBUG (or a dedicated probe-counter) so genuine errors
   stay visible.
3. If not: bind the well-known ports to the LAN interface only, or make the
   adaptive set configurable, and document the chosen surface in NODE_MODEL
   terms (all nodes identical; exposure is a node configuration, not a role).
4. A regression check that the chosen binding policy is what the code does.

## Gates

Touches `core/src/transport/` binding policy -> Rule-8 adversarial review if
implemented in that tree.

## References

- JEV dogfood run record extension 3 (2026-09-23):
  `HANDOFF/harness/JEV_DOGFOOD_RUN_2026-09-22.md` (live sort of this line:
  honest `unmatched`, keyed model privately chose `capacity` @ 0.38 and was
  vetoed by the code-owned layer — the pack has no API-noise bucket).
- Node model: `docs/rules/NODE_MODEL.md` (no roles; exposure is config).
- Old-node evidence preserved at `tmp/radio-candidates/56d66f7/node-stdout.log`.
