# V0.4.0 comprehensive implementation plan â€” identity / transport / WiFi delivery

Status: Active â€” **authoritative for implementing models** (no guesswork)
Date: 2026-09-21
Owner: CTO/orchestrator
Authority stack:
1. THIS file (implementation truth + WP gates)
2. `HANDOFF/V040_CTO_MASTER_PLAN_2026-09-20.md` (0.4.0 checklist + merge train)
3. `HANDOFF/V040_WORKING_FIRST_PATH_2026-09-20.md` (operator working-first)
4. Source tickets in `HANDOFF/todo/` (evidence history â€” **not** status authority)
5. Code on `origin/main` (verified 2026-09-21 commands below)

Audit inputs (product only â€” no other-lane work):
- `HANDOFF/V040_CTO_HANDOFF_SCMESSENGER_IDENTITY_TRANSPORT_WIFI_2026-09-21.md`
- `HANDOFF/todo/P0_SCMESSENGER_WIFI_DELIVERY_IDENTITY_TRANSPORT_CANONICAL_2026-09-21.md`

JEV: `sovereign-harness` at `<repo>/vendor/sovereign-harness (scripts/update_local_harness.py)`
(origin/main worktree preferred; `git show origin/main:harness/jev.py` if dirty).
JEV key: `~/.config/harness/jev.env` via `harness.config.resolve_jev_key()`.
**Full integration:** `HANDOFF/V040_JEV_HARNESS_INTEGRATION_2026-09-21.md`.
**Repo insight packs:** `scripts/jev_packs.py` + `scripts/jev_repo_insights.py`
(batched; run after merge trains and before paste waves).

---

## 0. Canonical model (non-negotiable â€” implement to this)

| Concept | ONE definition |
|---|---|
| Contact / addressing identity | **Ed25519 public-key hex** (64 hex chars), store + UI + ledger write format |
| Network peer id | Derived via documented extract/parse helpers â€” **never** a second contact-address flavor |
| Routing feed | `IronCore::routing_peer_seen(peer_id_hex, transport)` on **every** data-link establishment (swarm + BLE + WiFiAware + WiFiDirect + any platform path) |
| Send addressing | Accept hex **and** legacy base58; resolve to hex then to PeerId only as needed for libp2p |
| Inbound identity | Delegate gets **authenticated** `sender_public_key_hex` + canonical derived id â€” never raw payload `sender_id` |
| Outbox keys | Canonical hex at enqueue; flush/drain must resolve both spellings (`resolve_queue_key` / dual-drain) |

---

## 1. Verification matrix â€” ticket vs `origin/main` (2026-09-21)

Commands run this session against `origin/main` @ `95b81b5b` (code tip lineage `51edac4b` + docs).

| WP / finding | Ticket Status | **Code truth on main** | Implementing-model action |
|---|---|---|---|
| **CRYPTO-01** spoofable `sender_id` to delegate | OPEN (P1 2026-09-16) | **MOSTLY CLOSED** â€” `iron_core` builds `sender_public_key_hex` from `sender_pubkey`, sets `sender_id: canonical_peer_id` | **Verify + tests only** â€” do not re-implement; write/extend unit test that spoofed payload sender â‰  delegate identity; close ticket with cite |
| **TRN-03 / WASM own-topic** | OPEN same ticket | **CLOSED on main** â€” `own_peer_key_hex` + `is_ghost_peer_topic` in `swarm.rs` (both loops) | **Verify WASM test / residual ghost-guard**; no new subscribe code unless test proves gap |
| **CLI send hex vs peer id** | "Fixed (pending CI)" | **CLOSED on main** â€” `peer_id_from_contact_identifier` accepts hex + base58; comments match the filed defect | **Regression test** CLI path (unit/integration); live WP5 proof |
| **Contact recovery peer_id as pubkey** | Active | **PARTIAL** â€” `placeholder_or_derived_contact` derives key via `public_key_hex_from_libp2p_peer_id`; placeholder when missing + notes | **CLOSE residual**: never write PeerId into `public_key`; placeholder must fail closed for **send encrypt** (no fake encrypt to empty key); tests |
| **Swarm routing feed** | OPEN routing ticket | **CLOSED for swarm** â€” production `core_arc.routing_peer_seen` in `swarm.rs` ConnectionEstablished | Field proof in WP5; do not re-wire |
| **Non-swarm routing feed** (BLE/WiFiAware/WiFiDirect) | OPEN P2 | **OPEN** â€” `mobile_bridge.rs` has **no** `routing_peer_seen` | **WP2 implement** â€” exact feed + fail-closed block gate |
| **Ghost-guard own-topic loss** | FIXED pending review | Code present on main (`is_ghost_peer_topic`) | **Rule-8 verify** if not already on file for that change; tests |
| **Outbox dual-key flush** (CO-B-001) | Audit HIGH | **CLOSED** PR **#339** `51edac4b` dual-drain + unit test | None |
| **CLI-03 / CORE-02 outbox** | P1 2026-09-16 | **CLOSED** (#297 + #322 lineage) | None â€” residual was CO-B-001 |
| **Receipts do not converge live** | OPEN | **OPEN** | **WP4** |
| **Crypto fail vs claimed delivered** | OPEN | **OPEN** | **WP4** â€” honest status: transport ACK â‰  app delivery |
| **Windows silent wedge** | OPEN | Watchdog landed historically; N-03 positive test still DISPATCHABLE | **WP4** + freebuff watchdog ticket |
| **conn-limit multi-port** | Wave-1 ticket | **OPEN** (live WARNs) | Freebuff `V040_T_CONN_LIMITS_MULTIPORT.md` (Rule-8) â€” **feeds WP5** |

**Audit checks (operator brief):**

| # | Check | Result |
|---|---|---|
| 1 | Handoff + P0 = SCMessenger product only | **PASS** â€” no cross-lane items in those two files |
| 2 | P0 points at existing OPEN todos; no duplicate plan | **PASS** after this doc â€” implement from **this** matrix + P0 WP order; do not open another root-cause plan |
| 3 | WiFi not claimed fixed without WP5 3-node logs | **PASS** â€” WP5 mandatory before any "WiFi fixed" / 0.4.0 working-bar claim |
| 4 | Canonical model = ONE identity + ONE routing feed entry | **PASS** â€” Â§0 table |

---

## 2. Work packages â€” exact remaining work (no guesswork)

Order: **WP1 â†’ WP2 â†’ WP3 â†’ WP4 â†’ WP5**. One PR per WP (or smaller). Isolated worktrees.

### WP1 â€” Identity unification (close the remaining write-side holes)

**Already on main:** hex as contact key; CLI send resolver; CO-B-001 flush dual-drain; CRYPTO-01 delegate binding (verify only).

**Must implement:**

| ID | Task | Files (anchors â€” re-grep before edit) | Acceptance |
|---|---|---|---|
| WP1.1 | Contact recovery: derive real pubkey or **refuse encrypt path** | `core/src/contacts_bridge.rs` `placeholder_or_derived_contact` / `emergency_recover` | Test: recovery with unknown peer â†’ contact either has derived hex **or** cannot be used as encrypt recipient (no PeerId-as-pubkey) |
| WP1.2 | Single write helper for contact identity | `core/src/store/contacts.rs`, `contacts_bridge.rs` | All production `Contact::new` call sites use hex pubkey; grep `Contact::new(peer_id, peer_id)` = 0 |
| WP1.3 | CLI/UI/core/Android/WASM send all resolve hexâ†’PeerId via **one** helper | `cli/src/main.rs` `peer_id_from_contact_identifier`, `core` export if needed | Unit tests: hex, base58, nameâ†’hexâ†’send |
| WP1.4 | CRYPTO-01 / WASM **regression tests** | `core/src/iron_core.rs` / tests, wasm tests | Spoofed payload id â‰  delegate identity; wasm own-topic subscribe asserted |

**Refs:** P1_CLI_SEND (verify+tests), P1_CONTACT_RECOVERY, P1_CORE_IDENTITY_SPOOF (verify only).

**Freebuff paste:** `HANDOFF/freebuff/queue/V050_WP1_IDENTITY_UNIFICATION_2026-09-21.md`

### WP2 â€” Transport â†’ routing feed

| ID | Task | Files | Acceptance |
|---|---|---|---|
| WP2.1 | Call `IronCore::routing_peer_seen(hex, transport)` from **every** platform data-link establish path | `core/src/mobile_bridge.rs` + Android/Kotlin bridge if it owns link-up events; **same** core entry as swarm | Grep: `routing_peer_seen` reachable from BLE/WiFiAware/WiFiDirect establish; **not** discovery-only |
| WP2.2 | Fail-closed | same as swarm block-check | Unknown/blocked peer â†’ **no** feed; test |
| WP2.3 | WiFi LAN path uses the feed | WP5 + unit | "Connected but unroutable" cannot occur when link is up |

**Refs:** P2_NON_SWARM_TRANSPORT_ROUTING_FEED, P1_ROUTING_ENGINE_NEVER_LEARNS_PEERS (swarm half already done).

**Gate:** **Rule-8** if `core/src/transport` or routing-sensitive store paths change. Harness/JEV canonical check before DONE.

**Freebuff paste:** `V050_WP2_ROUTING_FEED_ALL_TRANSPORTS_2026-09-21.md`

### WP3 â€” Inbound completeness

| ID | Task | Files | Acceptance |
|---|---|---|---|
| WP3.1 | Confirm WASM own-topic on current main; fill test gaps | `swarm.rs` wasm loop | Test fails if subscribe removed |
| WP3.2 | Ghost-guard own-topic residual | `swarm.rs` `is_ghost_peer_topic` | Own topic never classified ghost when key known |
| WP3.3 | Authenticated sender to delegate â€” **all** receive paths | `iron_core.rs` `receive_message` | No path passes payload-only `sender_id` as `sender_public_key_hex` |

**Gate:** Rule-8 for transport/ghost-guard if still in gated dirs; JEV canonical before DONE.

### WP4 â€” Delivery truth / receipts / wedge

| ID | Task | Files / tickets | Acceptance |
|---|---|---|---|
| WP4.1 | Receipts converge live | P1_ASYNC_DELIVERY_RECEIPTS_* | 3-node: sender UI receipt matches receiver durable history |
| WP4.2 | Honest crypto-send vs delivered | P0_SEND_CRYPTO_FAILS_VS_DELIVERED | Transport ACK never marks `delivered=true` without recipient app verification |
| WP4.3 | Windows silent wedge + N-03 positive test | P1_WINDOWS_NODE_SILENT_WEDGE, freebuff watchdog ticket | Quiet healthy node does not exit; wedge events logged/bounded |

**Gate:** tests + live WP5; JEV on evidence claims.

### WP5 â€” 3-node log + live WiFi proof (exit gate for "WiFi fixed")

**Mandatory evidence attached to P0 umbrella ticket** â€” not a PR body claim.

| Step | Command / action |
|---|---|
| 1 | Same-SHA fleet (or documented delta) |
| 2 | Capture **failed** send window logs all 3 nodes (commands in P0 WP5) |
| 3 | After WP1â€“4: capture **success** window Aâ†”Bâ†”C WiFi (no cellular) |
| 4 | Trace: enqueue â†’ identity resolve â†’ transport â†’ route â†’ send â†’ ack â†’ UI |
| 5 | Attach paths under P0 ticket; mark UNVERIFIED legs explicitly |

**Do not claim WiFi fixed without step 3â€“4 evidence.**

---

## 3. Harness + JEV verification protocol (completion gate)

Implementing models **must** run this before marking any WP "DONE" or opening a merge request that claims canonical compliance.

### 3.1 Toolchain

```
# Prefer harness origin/main (local clone may lag)
cd C:\Users\SCM\Documents\GitHub\Harness
git fetch origin
# Use a clean worktree if main is dirty:
git worktree add <repo>/vendor/sovereign-harness (scripts/update_local_harness.py) origin/main

# Key (if live JEV): ~/.config/harness/jev.env  (JEV_API_KEY / jev_api_key)
# Without a key: local structural fallback only â€” record is_fallback=true
```

JEV API (harness `origin/main` `harness/jev.py`):
- `JevEvaluator(state, questions).evaluate(...)` â†’ `JevEvaluationResult`
- `result.is_passing(min_confidence=0.70)` â€” **canonical completion** requires `True` **and** command evidence
- Cost: `jev_cost(input_tokens)` = input_tokens * 42 / 1e6 (do not invent usage.cost)
- Default questions: `diff_question_pack()` â€” for WP completion use **custom pack** in Â§3.3
- Code-owned facts (paths, tests, greps) are **not** re-judged by JEV â€” JEV answers only semantic canonical compliance

### 3.2 Required mechanical gates (always â€” code-owned)

| Gate | Command |
|---|---|
| Rules | `python scripts/rules_check.py` |
| Wiring | `python scripts/check_wiring.py` (android/UI touched) |
| Tests | `cargo test -p scmessenger-core --lib <wp_test>` + CI full lanes |
| Grep canon | WP-specific greps in Â§2 tables |
| Queue status | `python scripts/check_queue_status.py` |
| PR scope | `scripts/pr_scope.sh <pr>` before merge |
| Rule-8 | non-author APPROVE on file for gated dirs |

### 3.3 JEV canonical question pack (WP completion)

Paste state as JSON (or markdown) with: **WP id, instruction, files touched,
acceptance list, command outputs, canon Â§0 rows this WP must uphold**.

```python
# Pseudocode â€” implement via harness JevEvaluator from origin/main
questions = {
  "canon_identity": {
    "type": "noul",
    "instructions": "Does the change keep ONE contact identity flavor (public-key hex) as the addressing key, without introducing a second contact-address flavor?",
    "criteria": {"true": "Hex remains the contact/store addressing key; peer id is only derived for libp2p.", "false": "A second contact-address flavor is introduced or peers are addressed primarily by base58 in storage/UI."}
  },
  "canon_routing_feed": {
    "type": "noul",
    "instructions": "Does the change keep ONE routing feed entry point (IronCore::routing_peer_seen) for all data-link transports, without a parallel bespoke feed?",
    "criteria": {"true": "All data-link establishes call the same routing_peer_seen entry (or WP is unrelated to routing).", "false": "A second routing-presence API is added or a transport is connected without that entry."}
  },
  "instruction_matches": {
    "type": "noul",
    "instructions": "Does the implementation and evidence satisfy the WP instruction and acceptance rows?",
    "criteria": {"true": "Acceptance rows are met with command/test evidence cited.", "false": "Acceptance rows are unmet, contradicted, or only asserted without evidence."}
  },
}
```

**DONE rule:** mechanical gates green **AND** `result.is_passing(0.70)` **AND**
evidence file paths recorded on the PR / P0 ticket. If JEV is unavailable
(`is_fallback` or no key), mark verification `UNVERIFIED-JEV` and **do not**
claim canonical complete until a keyed pass is recorded â€” mechanical gates
still required.

### 3.4 Clarification during implementation

When the implementing model is <99% confident:
1. Prefer **harness verify** (panel) on the specific claim/diff.
2. Prefer **JEV typed question** over free-text debate.
3. If still unclear, write `HANDOFF/freebuff/inbox/` PREMISE-WRONG / QUESTION â€” do not invent a new root cause.
4. Do **not** open a parallel architecture plan.

### 3.5 Helper script

`scripts/jev_canonical_check.py` on SCMessenger main (lands with this plan)
loads `harness.jev` from `HARNESS_REPO` (default `C:\Users\SCM\Documents\GitHub\Harness`
or a `Harness-jev-use` worktree of origin/main) and evaluates the Â§3.3 pack.

```text
python scripts/jev_canonical_check.py --wp WP2 --state-file tmp/wp2_state.json
```

Exit 0 = JEV `is_passing`. Without a key or with `--allow-fallback`, may print
`UNVERIFIED-JEV` â€” do not treat that as canonical DONE.

---

## 4. Dispatch order for implementing model (Freebuff or orchestrator)

| Step | Paste / action | Gate |
|---|---|---|
| 0 | Read this file + P0 umbrella + master plan | â€” |
| 1 | WP1 ticket (`V050_WP1_...`) | tests + JEV Â§3.3 |
| 2 | WP2 ticket (`V050_WP2_...`) | Rule-8 + JEV |
| 3 | WP3 ticket (`V050_WP3_...`) | Rule-8 if gated + JEV |
| 4 | WP4 ticket (`V050_WP4_...`) | tests + JEV |
| 5 | WP5 live proof on P0 ticket | **operator** phone + 3-node logs |
| 6 | Parallel Wave-1 leftovers: conn-limits, AND-06, watchdog, IP-churn | per ticket gates |
| 7 | Orchestrator merge train + fleet redeploy + final 0.4.0 checklist | master plan Â§4 |

**Do not paste:** keystore/tag chores as freebuff impl; CODE ON MAIN tickets; completed review dispatches.

---

## 5. Relationship to 0.4.0 tag checklist

| Master plan item | Wifi/identity plan effect |
|---|---|
| Working mesh day-to-day | WP1â€“4 code + WP5 proof |
| Audit HIGHs | CO-B-001 already merged; WP items cover identity/transport HIGHs still open |
| conn-limits | Required for WP5 reliability |
| AND-06 / SEC-03 | Still parallel (Wave-1 / operator) |
| D4/D6/D7 / tag | After WP5 + operator session; **not** before |

---

## 6. Audit record

- Handoff + P0: **product-only** â€” PASS
- No duplicate root-cause plan â€” THIS doc is the implementation authority
- WiFi claim requires WP5 evidence â€” enforced
- Canonical model single identity + single routing feed â€” Â§0

*End of comprehensive plan.*
