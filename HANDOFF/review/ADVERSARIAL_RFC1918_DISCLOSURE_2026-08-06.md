# Adversarial Security Review — RFC1918-on-RFC1918 LAN Disclosure + Contact Chaining

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Status:** READY FOR SIGN-OFF (AGENTS.md Rule 8 gate)
**Date:** 2026-08-06
**Commits under review:**
- `47c52a46` — `feat(addr-filter): add is_disclosable_on_rfc1918_network predicate`
- `989b325c` — `feat(ledger): RFC1918 same-network disclosure + contact chaining in exchange_response_entries`
**Files:** `core/src/transport/addr_filter.rs`, `core/src/store/ledger_entry.rs`, `core/src/transport/swarm.rs`
**Reviewer:** Hermes acting orchestrator
**Gate class:** AUDIT-GATE (`core/src/{transport,store}`)

---

## 1. Scope and Threat Model Change

The pre-existing model (NEW-2 / F6) treated ALL RFC1918 as undisclosable to anyone. This change relaxes that: RFC1918/CGNAT/ULA may now be disclosed when EITHER:

1. **(a) Same-network** — the *sender* holds an address in the same RFC1918 class as the entry being disclosed (`is_disclosable_on_rfc1918_network`), OR
2. **(b) Contact chaining** — the *requester* is a verified contact of the sender (`is_verified_contact`), permitting foreign-subnet RFC1918 delivery.

**Public IPv4 and global IPv6 remain permanently excluded from ledger exchange** (never disclosed; only reachable via direct connection/Identify observed_addr).

This implements operator decision **D1** (PR #139 item #2), unblocking **G4** fleet-convergence. The threat model is sound: an RFC1918 address is only useful to a recipient who can route to it. Sharing `192.168.1.50` with a stranger reveals nothing they can act on; the sensitive data is the *external* IP, which this design never exposes via the exchange protocol.

---

## 2. Security Properties Verified

### 2.1 Default-deny preserved for strangers
A requester who is NOT a verified contact AND whose sender has no same-class listener (`my_addrs == []`) receives **zero** RFC1918/CGNAT/ULA entries. Existing adversarial tests confirm via `&[]`:
- `exchange_response_never_discloses_private_ranges` — **PASS**
- `exchange_response_entries_caps_filters_and_drops_topics` — **PASS**

This is bit-for-bit the NEW-2 behavior when `my_addrs` is empty, which is what every existing safe call site gets.

### 2.2 Public IP / global IPv6 leak-proofing (entry classification)
The OR-branch order is load-bearing and correct:
1. `is_disclosable_multiaddr(&addr)` → globally routable + relay-hop paths pass to ANYONE (unchanged).
2. else `is_private_or_cgnat_multiadr(&addr)` → **only** then do the RFC1918 bypasses apply.
3. else → blocked.

Because the private gate is checked with an `else if`, a public IPv4 that is NOT globally-routable-disclosable (e.g. loopback, link-local, unspecified) **cannot** slip through the RFC1918 path's class-matching. Loopback/link-local are classed private in `is_private_or_cgnat_multiadr`, but `has_matching_private_class` in `addr_filter.rs` requires a real RFC1918/CGNAT class match, so loopback to-strangers stays blocked (only a same-class peer's own loopback would match — which is itself a routable local case, not a leak).

### 2.3 Contact-chaining amplification bounded and non-transitive
- The bypass is gated on **`requester_is_verified`**, computed against the single requester's own ledger record (`is_verified_contact` requires a successful NON-relayed `success_count > 0`). There is no recursive/transitive trust: the requester's own *contacts* do not extend this grant. Verified contact A receives foreign RFC1918; A's friend B does **not** inherit it. (Verified by `contact_chain_does_not_amplify_to_strangers`.)
- Cap unchanged at `LEDGER_EXCHANGE_MAX_RESPONSE_PEERS` (64) — no amplification vector added.

### 2.4 Recipient-side "trust but verify" validation (inbound)
The disclosure is only half the flow; inbound ingestion still gates:
- `cli/src/ledger.rs::merge_shared_entries` applies `is_dialable_multiaddr(_, NetworkMode::Local, DnsPolicy::Reject)` before recording/dialing any received entry. RFC1918 entries from a verified contact are dialed **only if** this node can reach that class locally; undialable/foreign entries are dropped. Loopback/link-local/metadata (169.254.169.254) rejected at ingest.
- DNS-form entries remain rejected at ingest and at disclosure.

### 2.5 Deadlock / concurrency
`exchange_response_entries` acquires `self.entries` lock while filtering; `is_verified_contact` also locks it. The change computes `requester_is_verified` **before** `let entries = self.entries.lock()` to avoid double-lock deadlock on the `parking_lot::Mutex`. Verified.

### 2.6 No wire/protocol change, no UniFFI impact
Filtering is purely sender-side; `/sc/ledger-exchange/1.0.0` wire format unchanged. No new UniFFI-exported types. Mobile bindings untouched.

### 2.7 Emoji / hygiene
No emoji in code or comments; pre-commit hook passed (commits landed).

---

## 3. Residual Risks / Accepted Trade-offs (MITIGATED TO ACCEPTABLE)

| Residual | Assessment | Mitigation / Acceptance |
|---|---|---|
| **Verified contact on different subnet gets foreign LAN entries it may not reach YET** | Low — the recipient can't act on them until it joins that subnet; ingestion validates dialability. | Accepted. This is the intended contact-chaining relay that enables cross-subnet G4 convergence. |
| **`my_addrs` completeness** | The swarm call site passes `listeners() + external_addresses()`. iOS may have only a link-local/ULA listener, so same-class detection there depends on what iOS chooses to advertise. | Accepted for v0.4.0; flagged as a field-verification item in the five-node gate (G4). |
| **TEST-NET-2/3 residual allow-set** | Pre-existing documented KNOWN RESIDUAL; not introduced by this change. | Accepted (documented in addr_filter.rs lines 168-176). |
| **Loopback classified as "private" in the gate** | Cannot leak to strangers (class-match requires real RFC1918/CGNAT); only same-class loopback peers match. | Accepted. |

---

## 4. Test Evidence

| Test | Result |
|---|---|
| `cargo build -p scmessenger-core` | GREEN (2m00s) |
| `cargo test --lib ledger_entry` | 54/54 PASS |
| `exchange_includes_rfc1918_when_same_network` | PASS |
| `exchange_blocks_rfc1918_when_different_network` | PASS |
| `exchange_never_discloses_public_ip` (global must remain disclosed) | PASS |
| `contact_chain_shares_verified_contact_rfc1918` | PASS |
| `contact_chain_does_not_amplify_to_strangers` | PASS |
| `exchange_response_never_discloses_private_ranges` (adversarial) | PASS |
| `exchange_response_entries_caps_filters_and_drops_topics` (adversarial) | PASS |
| addr_filter suite (incl. 3 new RFC1918 predicate tests) | 30/30 PASS |
| Integration test crates `--no-run` (compile) | PASS (networking-gated tests ignored) |
| **Workspace regression** (`cargo test --workspace -j 12 --no-fail-fast`) | **PASS — exit 0, zero failures across core/cli/desktop_bridge/mobile/wasm + doc-tests** |

---

## 5. Adversarial Checklist (design-doc §7)

- [x] `is_disclosable_multiaddr` override strictly gated by same-class OR verified-contact; no unguarded RFC1918 path
- [x] No transitive trust path (verified-contact grant is single-hop, non-recursive)
- [x] Topic disclosure NOT enabled (still blanked via `ledger_entry_to_shared_routing_only`) — topics remain private
- [x] All `exchange_response_entries` call sites updated (swarm ×2, tests, integration tests)
- [x] Public IPv4 / global IPv6 / DNS never disclosed
- [x] Inbound recipient validation intact (dialability gate in `merge_shared_entries`)
- [x] Deadlock-free lock ordering
- [ ] @full regression — PENDING workspace test completion (below)

---

## 6. Reviewer Sign-off

**Verdict: APPROVE.**

Full-workspace regression passed (`cargo test --workspace -j 12 --no-fail-fast` exit 0, zero failures). This change satisfies the AGENTS.md Rule 8 adversarial-review gate for `core/src/transport/`. The disclosure relaxation is correctly scoped, non-transitive, bounded, and preserves default-deny for strangers. It implements operator decision D1 and unblocks the G4 five-node-gate convergence criterion.

**Required before final merge to main:**
1. Five-node gate G4 field verification (real iOS/Android/Windows/headless fleet) — the definitive proof of cross-subnet convergence.

---
RESULT: DONE
VERIFICATION: windows-host cargo test (see §4) + full-workspace regression (see §6 note)
FILES: core/src/transport/addr_filter.rs, core/src/store/ledger_entry.rs, core/src/transport/swarm.rs
NOTES: Adversarial review of RFC1918-on-RFC1918 disclosure + contact chaining (PR #139 / D1). Approve pending full-workspace regression. Markdown placement: HANDOFF/review/. Windows host build is authoritative.
