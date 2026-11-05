# Vendored libp2p-swarm 0.48.0 -- delta against upstream

Base: crates.io `libp2p-swarm` 0.48.0 (registry source, unmodified). Wired in via
`[patch.crates-io]` in the workspace `Cargo.toml`. Reason: stock 0.48.0 panics with
`unreachable!()` in the `Either` handler/behaviour event routers (D9; observed live,
tokio worker task died right after identify). 0.48.0 is the version pinned by libp2p
0.57 in `Cargo.lock`; it still contains the `unreachable!()` sites, so the patch is
still required. Re-check on every libp2p bump: `diff -ru <registry>/libp2p-swarm-X
vendor/libp2p-swarm-0.48.0` should show only the items below; drop the vendor copy if
upstream removes the `unreachable!()` sites.

Complete list of differences (anything not listed here is byte-identical to upstream):

1. `src/handler/either.rs` (functional): `FullyNegotiatedInbound::transpose` and
   `ListenUpgradeError::transpose` return `Option`; every `_ => unreachable!()` arm
   (on_behaviour_event, FullyNegotiatedInbound/Outbound, DialUpgradeError,
   ListenUpgradeError) becomes a `tracing::warn!("D9-DEGRADE: ...")` drop. Adds a
   `#[cfg(test)] mod tests` with three D9 regression tests.
2. `src/behaviour/either.rs` (functional): `on_connection_handler_event`
   `_ => unreachable!()` becomes a `warn!` drop.
3. `src/lib.rs`: adds `#![allow(clippy::disallowed_methods)]` because the workspace
   `.clippy.toml` bans `Option/Result::unwrap`. Upstream `unwrap()` calls (including
   `pool.rs` `extract` and two in `lib.rs`) are left UNCHANGED, not rewritten to
   `expect()`.
4. Upstream unit-test modules gated `#[cfg(all(test, scm_upstream_tests))]` in:
   `behaviour/{external,listen,peer}_addresses.rs`, `connection.rs` (2 sites),
   `connection/pool/dial_ranker.rs`, `connection/supported_protocols.rs`,
   `handler.rs`, `handler/one_shot.rs`, `lib.rs` (`mod test` and `mod tests`).
   Reason: they need upstream dev-deps (libp2p-plaintext, libp2p-yamux, quickcheck,
   libp2p-swarm-test, ...) that are not vendored. `scm_upstream_tests` is a cfg
   declared via `[lints.rust] unexpected_cfgs`, deliberately not a feature, so
   `--all-features` cannot turn it on.
5. `Cargo.toml`: header note; `[[test]]`/`[[bench]]` sections and ALL
   `[dev-dependencies.*]` removed (keeps `Cargo.lock` free of dev-only packages);
   `doctest = false`; `check-cfg` entry for `scm_upstream_tests`.
6. Removed files/dirs not needed for the build: `tests/`, `benches/`,
   `Cargo.toml.orig`, `Cargo.lock`, `.cargo-ok`, `.cargo_vcs_info.json`.
7. Added: this file.
