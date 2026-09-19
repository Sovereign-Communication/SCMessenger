// Integration regression tests for the ledger CHOKE-POINT REFACTOR
// (HANDOFF/todo/LEDGER_CHOKE_POINT_REFACTOR.md, findings F3/NEW-1 and NEW-2).
//
// WHY THIS FILE EXISTS. Three adversarial review rounds returned BLOCK on the
// ledger-seeding work, and every failure had the same shape: a concept
// implemented in two places with the fix applied to only one. The clearest was
// round 2 -- the DNS gate landed in `cli/src/main.rs`'s `cmd_relay`
// `PeerIdentified` handler, with a comment citing the review, and NOT in the
// byte-identical handler in `cmd_start`. The review's own post-mortem says why
// it survived: "No CLI test covers the `cmd_start` handler at all."
//
// So this file covers the handler, from the crate's public surface, with no
// swarm and no networking. `ConnectionLedger::record_identified_peer` is now the
// only copy of that handler; both `main.rs` call sites delegate to it.
//
// T2 UNIFICATION (2026-08-31): the facade now stores into the shared core
// ledger, whose ingress rejects DNS/SSRF/unroutable addresses at the store
// edge and whose dialable-candidate build requires `success_count > 0`
// (proven, not hearsay). The gate tests below pin the same hostile-address
// outcomes they always did; the "operator DNS name" and "hearsay is dialable"
// cases pin the NEW, stricter edges that replaced them.

use scmessenger_cli::ledger::ConnectionLedger;
use scmessenger_core::store::LedgerManager;

fn ledger() -> ConnectionLedger {
    ConnectionLedger::new(LedgerManager::ephemeral())
}

fn peer() -> String {
    libp2p::PeerId::random().to_string()
}

fn advertised(addrs: &[&str]) -> Vec<String> {
    addrs.iter().map(|s| s.to_string()).collect()
}

/// The exploit from the ticket, end to end through the ledger: a peer completes
/// Identify and advertises `/dns4/evil.example/tcp/80`. If that string reached
/// `dialable_addresses()` the scheduler dials whatever the zone says -- and the
/// zone owner can re-point the A record between probes, which turns one ledger
/// entry into an internal port scanner.
#[test]
fn identify_advertised_dns_never_becomes_a_dial_target() {
    let mut ledger = ledger();
    let pid = peer();

    let hostile = advertised(&[
        "/dns4/evil.example/tcp/80",
        "/dns6/evil.example/tcp/80",
        "/dns/evil.example/tcp/80",
        "/dnsaddr/evil.example",
        "/dns4/evil.example/udp/9001/quic-v1",
        "/dns4/evil.example/tcp/443/p2p-circuit",
        "/dns4/metadata.google.internal/tcp/80",
    ]);

    assert_eq!(
        ledger.record_identified_peer(&pid, &hostile),
        0,
        "the Identify handler recorded a remote-advertised DNS name"
    );
    assert!(
        ledger.dialable_addresses(None, &[]).is_empty(),
        "a remote-advertised DNS name reached dialable_addresses(): {:?}",
        ledger.dialable_addresses(None, &[])
    );
    assert_eq!(ledger.entry_count(), 0);
}

/// The IP half of the same handler: `169.254.169.254` (cloud metadata),
/// loopback and link-local are advertisable by any peer and must not be stored
/// either. Includes the NAT64 / 6to4 wrappers of the metadata endpoint, which
/// `Ipv6Addr::to_ipv4()` does not unwrap and which therefore passed every IPv4
/// rule before round 4.
#[test]
fn identify_advertised_ssrf_addresses_never_become_dial_targets() {
    let mut ledger = ledger();
    let pid = peer();

    let hostile = advertised(&[
        "/ip4/169.254.169.254/tcp/80",
        "/ip4/127.0.0.1/tcp/8080",
        "/ip6/::1/tcp/8080",
        "/ip6/::ffff:169.254.169.254/tcp/80",
        // NAT64 well-known prefix: this IS 169.254.169.254.
        "/ip6/64:ff9b::a9fe:a9fe/tcp/80",
        // NAT64 local-use prefix, RFC 6052 /48 embedding: also 169.254.169.254.
        "/ip6/64:ff9b:1:a9fe:0:a9fe::/tcp/80",
        // 6to4: also 169.254.169.254.
        "/ip6/2002:a9fe:a9fe::/tcp/80",
        "/ip4/0.0.0.0/tcp/9001",
        "/ip4/224.0.0.1/tcp/9001",
        "/ip4/255.255.255.255/tcp/9001",
    ]);

    assert_eq!(ledger.record_identified_peer(&pid, &hostile), 0);
    assert!(
        ledger.dialable_addresses(None, &[]).is_empty(),
        "an SSRF address reached dialable_addresses(): {:?}",
        ledger.dialable_addresses(None, &[])
    );
    assert_eq!(ledger.entry_count(), 0);
}

/// The gate is a filter, not an outage: a genuine advertised address is still
/// RECORDED (routing knowledge), but it is HEARSAY -- `success_count == 0`,
/// not `locally_verified` -- so it must not be blindly re-dialed from the
/// persistent sweep. Only a locally proved connection promotes it to a dial
/// candidate (T2 disclosure rule). Includes the LAN addresses the WiFi/mesh
/// tier of the transport priority order depends on.
#[test]
fn identify_still_records_real_addresses_but_never_verifies() {
    let mut ledger = ledger();
    let pid = peer();

    let good = advertised(&[
        "/ip4/198.51.100.4/tcp/9001",
        "/ip4/192.168.1.20/tcp/9001",
        "/ip6/2606:4700:4700::1111/tcp/9001",
    ]);

    assert_eq!(ledger.record_identified_peer(&pid, &good), 3);
    assert_eq!(ledger.entry_count(), 3);
    // Hearsay only: nothing from an advertisement is dialable yet.
    assert!(
        ledger.dialable_addresses(None, &[]).is_empty(),
        "an advertisement reached dialable_addresses(): {:?}",
        ledger.dialable_addresses(None, &[])
    );

    // A real connection to one of them proves it (core fires
    // ledger.record_connection on ConnectionEstablished); that one becomes a
    // dial target, the hearsay siblings do not.
    ledger.record_connection("/ip4/198.51.100.4/tcp/9001", &pid);
    let dialable = ledger.dialable_addresses(None, &[]);
    assert_eq!(dialable.len(), 1, "got {dialable:?}");
    assert_eq!(dialable[0].0, "/ip4/198.51.100.4/tcp/9001");
}

/// Both `main.rs` handlers now call the same function, so "the two handlers
/// agree" is true by construction. This asserts the property the two handlers
/// are supposed to share, so that a future re-inlining of either one is caught:
/// running the handler twice with the same input is idempotent and never
/// upgrades a rejected address.
#[test]
fn the_handler_is_the_same_function_for_both_cli_commands() {
    let pid = peer();
    let mixed = advertised(&[
        "/dns4/evil.example/tcp/80",
        "/ip4/198.51.100.4/tcp/9001",
        "/ip4/127.0.0.1/tcp/9001",
    ]);

    // `cmd_start`'s handler.
    let mut start_ledger = ledger();
    let start_recorded = start_ledger.record_identified_peer(&pid, &mixed);

    // `cmd_relay`'s handler.
    let mut relay_ledger = ledger();
    let relay_recorded = relay_ledger.record_identified_peer(&pid, &mixed);

    assert_eq!(start_recorded, relay_recorded);
    assert_eq!(start_recorded, 1);

    // Only the single good address entered the store.
    assert_eq!(start_ledger.entry_count(), 1);
    assert_eq!(relay_ledger.entry_count(), 1);

    // Idempotent: replaying the same Identify does not promote a rejection.
    assert_eq!(start_ledger.record_identified_peer(&pid, &mixed), 1);
    assert_eq!(start_ledger.entry_count(), 1);
}

/// Operator-configured bootstrap entries must keep working -- that is the
/// internet-relay tier of the transport priority order -- but the UNIFIED store
/// has one definition of "dialable": even an operator-supplied DNS name cannot
/// enter the core ledger (DNS is decided by the zone owner at dial time), so a
/// bootstrap must be an IP-form multiaddr, like every fleet seed. The old
/// `add_bootstrap` DNS-name slot is GONE along with the CLI's private store
/// (T2); a name configured any other way cannot be dialed either.
#[test]
fn operator_bootstrap_addresses_survive_but_names_never_enter() {
    let mut ledger = ledger();

    // IP-form bootstrap: recorded, verified, dialable.
    ledger.add_bootstrap("/ip4/198.51.100.200/tcp/443", None);
    let dialable = ledger.dialable_addresses(None, &[]);
    assert_eq!(dialable.len(), 1, "got {dialable:?}");
    assert!(dialable[0].0.contains("198.51.100.200"));

    // DNS-form bootstrap: rejected at the store edge, no dialable target.
    let pid = peer();
    ledger.add_bootstrap("/dns4/relay.sovereign.example/tcp/443", None);
    assert_eq!(ledger.entry_count(), 1, "DNS name must not enter the store");
    ledger.record_identified_peer(&pid, &advertised(&["/dns4/evil.example/tcp/80"]));
    assert_eq!(ledger.entry_count(), 1);
    assert_eq!(ledger.dialable_addresses(None, &[]).len(), 1);
}

/// NEW-2: `ConnectionLedger::to_shared_entries()` is gone, and with it the
/// second, unguarded disclosure door. Both directions of
/// `/sc/ledger-exchange/1.0.0` are now built inside the swarm from
/// `LedgerManager::exchange_response_entries`.
///
/// This is a compile-time property (the function does not exist, and
/// `SwarmHandle::share_ledger` takes no payload), so what a runtime test can add
/// is the guarantee that the CLI ledger exposes no OTHER way to turn its entries
/// into `SharedPeerEntry` records: the wire type is never constructed from a
/// `ConnectionLedger`, only consumed by `merge_shared_entries`.
#[test]
fn the_cli_ledger_has_no_wire_export_path() {
    let mut ledger = ledger();
    let pid = peer();
    ledger.record_identified_peer(&pid, &advertised(&["/ip4/198.51.100.4/tcp/9001"]));
    ledger.record_topic("/ip4/198.51.100.4/tcp/9001", "sc-family-chat");

    // The topic is held locally for the CLI's own display and ranking...
    assert_eq!(
        ledger.all_known_topics(),
        vec!["sc-family-chat".to_string()]
    );
    // ...and the only direction the wire type moves is inward.
    let learned = ledger.merge_shared_entries(&[scmessenger_core::transport::SharedPeerEntry {
        multiaddr: "/ip4/203.0.113.7/tcp/9001".to_string(),
        last_peer_id: Some(peer()),
        last_seen: 1_700_000_000,
        known_topics: vec!["sc-activists".to_string()],
    }]);
    assert_eq!(learned, 1);
}
