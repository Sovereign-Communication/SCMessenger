# Canonical Outlier Audit -- iteration 1 (DIM-A: doctrine noun/role outliers)
Task: HANDOFF/freebuff/queue/AUDIT_CANONICAL_OUTLIERS_V040_V050_V100_2026-09-19.md
Date: 2026-09-19
Branch: glm/canonical-outlier-audit
HEAD: 1acb63531aaa1e7c22071bb7430c72b5a9753210 (+ iter0 commit 89a067a0)
Model: Buffy (Freebuff)
Mode: REPORT-ONLY

## Method and evidence commands

Ran via the code_search tool (ripgrep-backed; `rg` itself is NOT on this shell's
PATH -- `rg: command not found`, exit 127, so shell-level rg commands were
adapted to the tool and to grep):

1. Repo-wide: pattern `\b(relay node|relay nodes|dedicated relay|bootstrap relay|relay server|the relay)\b`, case-insensitive.
   [WARNING] This search hit the tool's output-size cap ("Stopped early after 144
   match(es)") -- the repo-wide listing is NOT exhaustive. The scoped searches
   below ran to completion and are the classification basis for code.
2. Repo-wide: pattern `every node is a full relay|no standalone relay|nodes, not relays` -- 23 matches, complete list obtained (see ALREADY-CORRECT section).
3. Scoped live code: same noun pattern in `core/src` (102 matches), `cli/src`
   (19), `android/app/src/main` (16) -- all complete, no cap hit.
4. Prior-inventory membership checks: `grep -n "<file>" doctrine_violation_inventory.md`
   per file (command output in transcript this session).
5. Spot reads: `sed -n` on swarm.rs:2688-2696, peer_broadcast.rs:1-4, internet.rs:165-178,
   docs/RELAY_OPERATOR_GUIDE.md:1-60, README.md grep.

Exception registry applied: `HANDOFF/audit/doctrine_violation_inventory.md` section 1
(§1A libp2p circuit-relay identifiers, §1B Rust module declarations, §1C generated
bindings, §1D test hostname literals) -- re-read this session at lines 17-50.

## Counts

- NEW findings: 5 (CO-A-001..CO-A-005)
- STILL-OPEN prior-inventory rows re-verified at current file:line: 12 locations
- Reclassified prior rows (prior classification wrong): 3
- Target spread: 0.4.0 x4, 1.0.0 x1, process x1 (CO-A-005 dual-tagged)

## NEW findings

### CO-A-001 -- Stray tracked tree `AgentSwarmCline/` uses relay-node participant nouns in code strings
- Dimension: DIM-A
- Severity: MED
- Target: 1.0.0
- Location: `AgentSwarmCline/scmessenger_swarm/observability.rs:5,20,24,42,73,81`;
  `AgentSwarmCline/scmessenger_swarm/observability_docs.md:10,13,35,39,43`;
  `AgentSwarmCline/scmessenger_swarm/observability_tests.rs:423,451`
- Authority contradicted: AGENTS.md "Architecture doctrine: nodes, not relays"
- Evidence: code_search pattern above; `git ls-files AgentSwarmCline | head -5`
  lists the files as tracked; `grep -n members Cargo.toml` ->
  `members = ["core", "cli", "wasm", "mobile", "desktop_bridge"]` --
  AgentSwarmCline is NOT a workspace member (never built by cargo). String
  example: `observability.rs:81` error text "Relay node hash must be a
  64-character hexadecimal string".
- Why it is an outlier: participant-noun "relay nodes" in code+docs of a
  git-tracked but build-foreign tree; unification debt and repo-hygiene debt at
  1.0.0 (decide: adopt, archive, or delete the tree).
- Suggested remediation class: inventory-ticket
- Status: OPEN

### CO-A-002 -- Operator-facing diagnostics strings call relay servers a failure mode by role name
- Dimension: DIM-A
- Severity: MED
- Target: 0.4.0
- Location: `android/app/src/main/java/com/scmessenger/android/network/DiagnosticsReporter.kt:167` ("Connection refused -- relay servers may be down, try again later") and `:171` ("All relay servers unreachable -- check firewall or try a different network")
- Authority contradicted: AGENTS.md doctrine; task mission row "doctrine
  violations that ship wrong operator-facing language" (0.4.0 column)
- Evidence: code_search `core/...android main` scoped output (complete), quoted above.
- Why it is an outlier: shipped user-facing troubleshooting text implies
  dedicated "relay servers" are a product component whose downness is
  diagnosable, which the doctrine forbids as a mental model.
- Suggested remediation class: docs-correct (string copy change; code edit
  deferred to follow-on ticket)
- Status: OPEN

### CO-A-003 -- Android network security config comment names "public mesh-relay servers"
- Dimension: DIM-A
- Severity: LOW
- Target: 0.4.0
- Location: `android/app/src/main/res/xml/network_security_config.xml:24`
- Authority contradicted: AGENTS.md doctrine
- Evidence: scoped code_search output, quoted: "over the public mesh-relay
  servers via TLS / libp2p noise."
- Why it is an outlier: comment frames relay servers as an infrastructure tier.
- Suggested remediation class: docs-correct
- Status: OPEN

### CO-A-004 -- README.md describes "an internet relay" / "relays carry" as participant nouns
- Dimension: DIM-A
- Severity: LOW
- Target: 0.4.0
- Location: `README.md:6` ("network, or an internet relay -- and the transports race"), `README.md:41` ("peers exchange messages directly, and relays carry")
- Authority contradicted: AGENTS.md doctrine (relay as noun only as verb/identifier)
- Evidence: `grep -n -i -E "relay" README.md` output above (complete for README).
- Why it is an outlier: README is the D3 operator-facing entrypoint; describing
  relays as agents implies a role. Borderline: the sentences describe the
  relayed-traffic behavior, which exists; wording can keep the behavior and drop
  the role noun.
- Suggested remediation class: docs-correct
- Status: OPEN

### CO-A-005 -- Canonical doc filename `docs/RELAY_OPERATOR_GUIDE.md` encodes a relay role the doc's own body denies
- Dimension: DIM-A
- Severity: LOW (content is correct; name is the outlier) -- PROCESS touch
- Target: 0.4.0
- Location: `docs/RELAY_OPERATOR_GUIDE.md` (filename; body header line 1 says "Node Operator Guide")
- Authority contradicted: AGENTS.md doctrine
- Evidence: read this session lines 1-17: title "Node Operator Guide", body:
  "SCMessenger has no dedicated relays and no bootstrap node tier. There are
  only nodes, and **every node is a full relay**"; `relay` subcommand called
  "not a distinct role".
- Why it is an outlier: the file is indexed and discoverable by a name that
  contradicts its own first sentence; a stranger searching "relay" lands on a
  doc whose URL asserts the role exists.
- Suggested remediation class: docs-correct (rename + link updates; needs an
  operator nod because inbound links/docs-sync may reference the path)
- Status: NEEDS-HUMAN (rename touches inbound links)

## STILL-OPEN prior-inventory rows (re-verified at CURRENT file:line this session)

Prior inventory: `HANDOFF/audit/doctrine_violation_inventory.md` (~950 estimated
violations across 117 files at filing time). Rows below were re-verified by the
scoped searches and spot reads; the inventory file-section membership grep
output is in this session's transcript. Full inventory rows not re-verified one
by one are not claimed either way (Rule 15: no silent truncation -- the method
limit is stated).

| # | Prior inventory anchor | Current file:line | Current text (from this session's search output) |
|---|---|---|---|
| 1 | inventory:540 peer_broadcast.rs section (4 violations) | `core/src/transport/peer_broadcast.rs:3-4` | "This module implements active relay functionality where relay nodes broadcast peer join/leave events" |
| 2 | inventory:155 internet.rs section (~85 violations) | `core/src/transport/internet.rs:176` | "Connect to a known relay node" (also :235 "Dial a relay node", :248-249 param docs) |
| 3 | inventory:474 peer_exchange.rs (2) | `core/src/relay/peer_exchange.rs:1` | "Peer Exchange -- learn about new relay nodes from connected peers" |
| 4 | inventory:397 onion.rs (15) | `core/src/privacy/onion.rs:410` | "Called by a relay node to:" |
| 5 | inventory:336 drift/relay.rs (26) | `core/src/drift/relay.rs:106` | "The relay engine -- heart of the mesh" |
| 6 | inventory:462 relay/server.rs (8) | `core/src/relay/server.rs:1` | "Relay Server -- accepts connections and stores messages for offline peers" |
| 7 | inventory:186 mesh_routing.rs (70) | `core/src/transport/mesh_routing.rs:457` | "Register a potential relay node" |
| 8 | inventory:530 relay_health.rs (25) | `core/src/transport/relay_health.rs:18` | "Relay node stability metrics for priority calculation" |
| 9 | inventory:620 wasm_support/transport.rs (18) | `core/src/wasm_support/transport.rs:3,22,55` | "Manages connections to relay servers", "Relay server URLs" |
| 10 | inventory:682,686,677,1033 cli main.rs rows | `cli/src/main.rs:278` ("Run headless relay node"), `:3814` ("Operates as a relay node: forwards all mesh traffic"), `:4137` ("Relay node is running."), `:4422,4431` ("Shutting down relay node...", "Relay node stopped.") | operator-visible display strings |
| 11 | inventory:729 cli bootstrap.rs row | `cli/src/bootstrap.rs:21` | "Strategy: Multiple public relay nodes with varying availability" |
| 12 | inventory:824 Kotlin MeshRepository rows | `android/.../data/MeshRepository.kt:2223` ("Ignoring payload attributed to bootstrap relay peer"), `:4638` ("is a bootstrap relay (infrastructure, not a user contact)"), `:249` ("NODE-RELAY-LABEL-002: detect infrastructure relay nodes") | NOTE: MeshRepository.kt is DIRTY in the shared checkout (another session's in-progress edit); lines cited as read this session from the working tree |

Aggregate: the prior inventory's core/cli rows remain present at current HEAD
in the files checked; none of the 12 spot-checked rows has been remediated.

## Reclassifications (prior inventory row was wrong; NOT violations)

| Prior row | Current location | Reading |
|---|---|---|
| inventory:76 ("every node is a full relay" listed as doc prose) + swarm.rs "1724 self-contradiction" | `core/src/transport/swarm.rs:2690-2692` | Full sentence: "There is no such thing as a dedicated bootstrap relay in this mesh -- every node is a full relay -- so the command is named for what it does: dial a seed peer." This STATES the doctrine; classified ALREADY-CORRECT-EXPLAINS-DOCTRINE, not a violation. |
| -- | `core/src/relay/mod.rs:3` | "Every node with internet connectivity is a relay server." -- universal-behavior statement, doctrine-consistent. ALREADY-CORRECT-EXPLAINS-DOCTRINE. |
| -- | `core/src/transport/behaviour.rs:520` | "Relay server - all nodes act as relays for NAT traversal" -- verb/behavior usage applied to all nodes. EXCEPTION-adjacent (technical identifier + universal behavior). Not filed. |

## ALREADY-CORRECT-EXPLAINS-DOCTRINE (complete list from search 2, 23 matches)

`AGENTS.md:18`, `DOCUMENTATION.md:97`, `cli/README.md:109`,
`docs/BOOTSTRAP_GOVERNANCE.md:8`, `docs/BOOTSTRAP.md:16`,
`docs/ANDROID_QUICKSTART_WINDOWS.md:69`, `docs/INSTALL.md:196`,
`docs/RELAY_OPERATOR_GUIDE.md:12`, `docs/platform/DOCKER_QUICKSTART.md:15`,
`docs/ops/GCP_DEPLOY_GUIDE.md:13`, `core/src/transport/swarm.rs:2692`,
`core/src/relay/protocol.rs:31`, `MeshRepository.kt:132,6060,10827`,
`scripts/test_bod_governance.py:101`, `HANDOFF/BOD_STATE.md:33,121`, plus
historical archive rows (docs/historical x2, doctrine inventory itself x2,
this task file x1). Count stated in full; no "and N more".

## Not done / UNVERIFIED

- Repo-wide noun search capped by tool output size (disclosed above); scoped
  searches used instead for classification.
- `HANDOFF/audit/doctrine_violation_inventory.md` is 1070 lines; 120 read
  directly + targeted membership greps for the 12 rows above. NOT re-verified:
  the remaining ~938 estimated rows row-by-row (their aggregate status is
  reported as "spot-check says still present", not as a complete recount).

## Next iteration aim

DIM-B: canonical identifier outliers (outbox keying, contact keys, peer_id
usage; re-verify IDENTIFIER_PARITY_AUDIT_2026-09-15 claims and the SHADOW
audit CLI-03 contradiction).
