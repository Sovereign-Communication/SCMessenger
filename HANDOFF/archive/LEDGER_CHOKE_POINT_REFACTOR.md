# Ledger choke-point refactor -- stop patching call sites

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: TODO (operator-approved 2026-07-26)
Created: 2026-07-26
Scope: prerequisite for closing the ledger findings; blocks 0.4.0 sign-off
Priority: HIGH

## Why this exists

Three adversarial review rounds, three BLOCK verdicts. Every round failed the
**same way**: a fix applied to one instance and not its equivalent sibling.

| Round | Failure |
|---|---|
| 1 | Address filter added, then run in `NetworkMode::Local` -- the mode that disables the RFC1918 check |
| 1 | Recency pruner added that reintroduced the exact event-loop DoS F4 was filed for |
| 2 | DNS gate added to `cli/src/main.rs:2996` (`cmd_relay`) but NOT to the identical handler at `:2034` (`cmd_start`) |
| 2 | Disclosure filter added to the ledger-exchange RESPONSE but not the REQUEST (`to_shared_entries`) |

Round 2's miss is the clearest illustration: `cmd_relay` even carries a comment
citing "re-review NEW-1". The author fixed the site they were pointed at and
never grepped for siblings.

**Conclusion: the defect is architectural, not clerical.** Security predicates
are enforced at N call sites that must all agree, with nothing forcing
agreement. Patching the named site will keep missing the unnamed one. Fix the
class.

## The refactor

### 1. ONE ingestion choke point

Move the address gate INSIDE `LedgerStore::record_connection` /
`LedgerManager::record_connection` and make the policy a required parameter, so
a caller cannot record an unvalidated address without saying so explicitly.
Today the gate lives in callers; `cli/src/main.rs:2034` proves that does not
hold.

Consequence: the invariant asserted at `cli/src/ledger.rs:352-357` and `:710-713`
("a DNS-form address may enter the CLI ledger only through `add_bootstrap`")
becomes true by construction. `dialable_addresses`' use of
`AllowLocallyConfigured` currently DEPENDS on that invariant and is unsound
until this lands.

### 2. ONE disclosure choke point

DELETE `LedgerStore::to_shared_entries()` (`cli/src/ledger.rs:463-480`). It has
no address filter, no cap, no `success_count > 0` filter, and copies
`known_topics` verbatim -- the exact field the core response path deliberately
blanks. It fires on every peer connection from three sites
(`cli/src/main.rs:1864`, `:2040`, `:2892`).

Have `SwarmCommand::ShareLedger` build its payload from the same function the
response path uses (`exchange_response_entries`), so there is one door instead
of two. Both directions of the protocol then share one predicate.

### 3. Make the predicate total, not per-family

`is_disclosable_multiaddr` is correct and unweakenable for the families it
covers, but the address space has more encodings than it enumerates:

- **NAT64 `64:ff9b::/96`** -- `/ip6/64:ff9b::a9fe:a9fe/tcp/80` IS
  `169.254.169.254` and passes BOTH filters. Mandatory-support territory on iOS
  carriers and default on several US carriers. Also `64:ff9b:1::/48`, 6to4
  `2002::/16`, Teredo `2001::/32`.
- **CGNAT `100.64.0.0/10`**, `192.0.2.0/24`, `198.18.0.0/15`, `240.0.0.0/4` --
  `Ipv4Addr::is_private()` does not cover these, so they are disclosable.

Use a "globally routable" predicate for disclosure rather than `!is_private()`,
and unwrap every embedded-IPv4 encoding before deciding.

### 4. Add a LocalMesh audience to the exchange reply

`export_seed_entries` got a `SeedExportAudience`; `exchange_response_entries`
did not. That asymmetry means peer-mediated LAN introduction is impossible: node
C cannot tell A about B's `192.168.x.y`. This matters because **mDNS is
`cfg`'d out entirely on Android** (`not(target_os = "android")` throughout
`swarm.rs`/`behaviour.rs`) -- Android uses `NsdManager` instead -- and on
Windows the `Expired` handler is excluded (`swarm.rs:4246`). Android and Windows
are the Phase-1 parity targets.

Drive the audience from whether the requester's own connection is on a private
address; that signal is available at the handler via `endpoint`/`remote_addr`.

Note also `exportSeedEntries` is UniFFI-exported and hardcodes `Untrusted`,
while `export_seed_entries_for` is not exported -- so mobile currently cannot
produce a LAN cold-start invite at all.

## Test debt that must be fixed with it

The review found the existing tests cannot detect these regressions:

1. **The NEW-4 regression test is sized so it cannot fail.**
   `future_dated_flood_from_bounded_identities_cannot_evict_honest_routes` uses
   8 identities x 64 quota = 512 slots against a 4096 ceiling -- it never
   crosses the threshold it claims to test. The real attack needs **64**
   identities (64 x 64 = 4096). Re-run it with 64+.
2. **Every wire-level test for round 2 is `#[ignore]`d**
   (`integration_ledger_seeding_hardening.rs:89, 184, 452, 572` --
   "requires real networking"). The only non-ignored NEW-5 test
   (`swarm.rs:7462`) asserts two constants are equal and would still pass if
   the rate-limit check were moved back below the expensive work. Extract the
   handler into a testable function and cover the ORDERING without networking.
3. **No CLI test covers the `cmd_start` handler at all** -- which is why the
   round-2 miss survived. Add one asserting a `PeerIdentified` carrying a DNS
   `listen_addr` does not reach `dialable_addresses()`.

## Also open (do not lose)

- **NEW-5 residual:** the 4 MiB CBOR request is decoded into ~80 000 structs by
  the codec inside `ConnectionHandler::poll` -- on the same `select!` task --
  BEFORE the token bucket is consulted. Gating a 64-iteration loop while
  leaving the 80k-allocation decode ungated. Needs a dedicated codec size for
  `/sc/ledger-exchange/1.0.0`, or disconnect on repeated over-quota.
- **NEW-7 residual:** `build_seed_ledger` (`core/src/relay/invite.rs:134-164`)
  inserts the inviter's own address at index 0 with NO filter and with the naive
  `find("/p2p/")` strip that F8 was filed against. Latent only because F2 is
  open. Must be fixed before invite acceptance is wired.
- **F2:** `InviteToken::verify()` still has zero production callers. F1 built
  the lock; it is not fitted.
- **F13:** pending-dial resolution (`swarm.rs:4522-4532`) has no
  `endpoint.is_dialer()` check while `record_connection` 40 lines below at
  `:4569` does. The asymmetry makes the fix trivial.
- F7, F10, F16, NEW-6, NEW-8, NEW-9, NEW-10.

## Gates

Machine is shared. `tasklist | grep -iE "cargo|rustc|gradle|java|ndk"` before
ANY build; `-j6`, `CARGO_INCREMENTAL=0`, never `-j8`+; `df -h /c` first; never
`cargo clean` (wipes all of `target/`).

```
cargo fmt --all -- --check
cargo clippy --workspace --all-features -j6 -- -D warnings
cargo clippy --workspace -j6 -- -D warnings -A clippy::empty_line_after_doc_comments
cargo test --workspace --no-run -j6
cargo build --target wasm32-unknown-unknown -p scmessenger-wasm --release -j6
cd android && ./gradlew assembleDebug -x lint --quiet
```

A fourth adversarial review is required after this lands. When dispatching it,
tell the reviewer explicitly that the previous three rounds all failed by
partial application, and ask it to grep for siblings of every fix rather than
checking only the named site.
