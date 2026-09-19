# V040-T14 (P0) -- PR #270 open (ephemeral NAT port never advertised again)

Status: PR FILED -- awaiting adversarial review

## Root cause (confirmed, was "to confirm" in the ticket)

The `AddressObserver` consensus accepted ANY observed SocketAddr from Identify
observed_addr / address reflection. The observed address of an OUTBOUND flow
carries our NAT-mapped SOURCE port (ephemeral, never dialable). Once it won the
consensus vote, two `swarm.add_external_address()` sites (Identify ~5170,
reflection ~4061) promoted it into the advertised-external set, and the same
consensus fed diagnostics + mobile hints directly.

Verified libp2p is not the promoter: libp2p-swarm 0.47 has only the manual
`add_external_address` (no add_foreign_address path); libp2p-autonat 0.15 has
zero external-address management. Our two sites are the only promotions.

## Fix (at the source, per campaign doctrine)

`AddressObserver` gained a listen-port allowlist (`set_listen_ports`, maintained
on NewListenAddr). Observations whose port is not a listen port are dropped at
record time and excluded from the consensus -- they can never become
primary_external_address() nor enter external_addresses(), diagnostics, relay
addr construction, or mobile hints. Defense-in-depth: both add_external_address
sites refuse non-listen ports with a warning. wasm keeps accept-all (no
listeners). DCUtR/relay paths untouched -- no regression surface.

## Verification (worktree scm-t14-ephemeral-port at 67d19d3c)

- fmt clean; clippy (documented gate) 0 errors; core+cli --all-targets clean
- wasm32 proof OK (scmessenger-wasm)
- core lib 1403/0 (4 new regression tests, incl. "ephemeral port loses the
  vote to the listen port even when more common"); cli lib 82/0
- Diff vs origin/main: exactly observation.rs + swarm.rs (+178/-17)

## Filed

- PR: https://github.com/Sovereign-Communication/SCMessenger/pull/270
- Commit: 53a9e8ff on `freebuff/v040-t14-ephemeral-port`
- Rule-8: adversarial review required (core/src/transport/).

Lane: #267/#268/#269/#270 all open, each awaiting non-author adversarial
review. #264 (T12) open -- blocks T9. T8 pending premise check vs #260.