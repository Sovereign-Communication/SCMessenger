//! Rule-8 review of PR #305, c8: the hourly relay budget and the per-peer share
//! are charged only when the node actually carries a message.
//!
//! WHY THIS TEST IS STRUCTURAL. The confirmed defect was the *position* of the
//! accounting inside `swarm.rs`: both loops incremented `relay_count_this_hour`
//! and `relay_counts_this_hour` before the destination was even parsed, so a peer
//! could spend its share -- and the node's whole hourly ceiling -- on requests
//! the node then refused (bad destination, custody-enforcement rejection,
//! duplicate). Behaviourally that path needs a live swarm and a peer that sends
//! admissible-but-undeliverable requests, which this crate's tests cannot stand
//! up; the honest regression guard for a positional defect is an assertion about
//! the compiled source. `include_str!` is that source, so re-introducing the
//! pre-parse increment (or deleting a commit point) fails here.
//!
//! The behavioural companion for the same fix, the per-peer share and the map
//! bound, lives in `core/src/transport/swarm.rs` (`relay_per_peer_budget_tests`).

/// The transport source as compiled, not as read from disk afterwards.
const SWARM_SOURCE: &str = include_str!("../src/transport/swarm.rs");

#[test]
fn hourly_budget_is_charged_at_exactly_the_two_commit_points() {
    // One marker per loop: native and wasm. A wasm-only regression would have
    // been the TRN-03 shape -- the native path fixed, the wasm path not.
    assert_eq!(
        SWARM_SOURCE.matches("C8-CHARGE-POINT").count(),
        2,
        "the hourly budget must be charged at exactly two commit points: the \
         native relay loop and the wasm relay loop"
    );
}

#[test]
fn budget_accounting_is_no_longer_incremented_from_the_old_pre_parse_site() {
    // The old shape, present in both loops before the fix: a raw increment of the
    // per-peer map by custody-of-request rather than custody-of-delivery.
    assert!(
        !SWARM_SOURCE.contains("or_insert(0) += 1"),
        "the per-peer budget counter must be updated through \
         note_peer_relay_admitted at a commit point, not by a raw increment from \
         the request path"
    );
}
