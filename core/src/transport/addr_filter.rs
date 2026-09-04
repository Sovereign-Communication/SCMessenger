//! Shared multiaddr validation: "is this address worth dialing, and is it
//! safe to dial or to disclose?"
//!
//! WHY THIS MODULE EXISTS (adversarial review F3, 2026-07-25): the CLI had
//! `is_dialable_multiaddr` / `is_self_address` in `cli/src/ledger.rs`, but core
//! had no equivalent. Core's ledger-seed import, its seed-dial candidate build
//! and its ledger-exchange response all accepted any string that merely
//! *parsed* as a `Multiaddr`. An attacker could therefore push
//! `/ip4/169.254.169.254/tcp/80` (cloud metadata), `/ip4/127.0.0.1/tcp/8080`
//! or arbitrary RFC1918 host:port pairs into a victim's dial set and read the
//! result off the dial-outcome timing (refused resolves in milliseconds,
//! filtered hangs to the sweep timeout) -- an SSRF/internal-port-scan oracle.
//!
//! The CLI now re-exports these functions so there is exactly ONE definition
//! of "dialable" in the workspace.
//!
//! This module is deliberately free of any I/O, any lock and any platform
//! `cfg` so it compiles identically on wasm32, Android and desktop.

use libp2p::multiaddr::Protocol;
use libp2p::{Multiaddr, PeerId};
use std::net::{Ipv4Addr, Ipv6Addr};

/// Network context for address filtering.
///
/// `Local` (WiFi/LAN/mesh) keeps private/LAN ranges dialable for local mesh
/// discovery; `Public` (cellular / public-only) additionally drops private
/// ranges since a public-only node cannot reach anyone's LAN.
///
/// Defaults to the conservative-for-connectivity `Local`, matching the CLI's
/// pre-existing behaviour. Do NOT hardcode `Public`: the entire BLE/WiFi-first
/// transport priority order depends on RFC1918 peers staying dialable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum NetworkMode {
    #[default]
    Local,
    Public,
}

/// Whether a DNS-form multiaddr (`/dns/`, `/dns4/`, `/dns6/`, `/dnsaddr/`) may
/// be accepted.
///
/// WHY THIS EXISTS (re-review NEW-1, 2026-07-25): the first remediation pass
/// validated every `Ip4`/`Ip6` component and then set `has_transport = true`
/// for DNS components while validating NOTHING about them. A name resolves to
/// whatever its owner's zone says, so `/dns4/evil.example/tcp/80` skipped every
/// rule below: publish `A evil.example -> 169.254.169.254`, put that string in
/// a ledger-exchange entry or an invite `seed_ledger`, and the desktop swarm --
/// which wires a real resolver -- resolves and dials it. That restores the full
/// SSRF/internal-port-scan oracle F3 was filed for, and it is re-pointable per
/// probe (change the zone between dials) so it scans, not just hits one host.
///
/// Resolve-then-validate is NOT implemented here on purpose: this module is
/// I/O-free and `cfg`-free by contract (it compiles identically on wasm32,
/// Android and desktop), and a resolve-then-validate gate is a DNS-rebinding
/// TOCTOU anyway -- libp2p re-resolves at dial time and on every reconnect, so
/// a validated answer is not the answer that gets connected to.
///
/// So the rule is provenance-based: a name is only as trustworthy as whoever
/// supplied it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DnsPolicy {
    /// The address came from a remote peer (ledger exchange, invite
    /// `seed_ledger`, Identify `listen_addrs`, gossip) or is about to be
    /// disclosed to one. DNS forms are REJECTED.
    ///
    /// This is the [`Default`] so that a future call site which forgets to
    /// think about provenance fails closed.
    #[default]
    Reject,
    /// The address came from local configuration: an operator-supplied
    /// bootstrap list, a CLI flag, or an address this node itself connected to.
    /// DNS forms are allowed, because the operator chose the name.
    AllowLocallyConfigured,
}

/// Which question a call site is asking of an address.
///
/// There are exactly two, they have different answers, and conflating them is
/// what produced NEW-2. Keeping them as one enum means there is ONE traversal of
/// the multiaddr and ONE place where each protocol component is interpreted --
/// see [`check_multiaddr`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Audience {
    /// "Can I reach it?" -- contextual, so it is parameterised by
    /// [`NetworkMode`].
    Dial(NetworkMode),
    /// "May I hand it to a stranger?" -- absolute, so it takes no parameter.
    Disclose,
    /// "Can I reach this proxy THIS PROCESS just created?"
    ///
    /// Exists for exactly one caller: the Wi-Fi Aware confirmed-data-path dial
    /// (`mobile_bridge.rs`, after `create_data_path` resolves).
    /// `WifiAwareTransport.startLoopbackProxy()` binds a TCP proxy on 127.0.0.1
    /// and reports THAT address, deliberately -- a Wi-Fi Aware peer is only
    /// reachable at a link-local IPv6 address, which needs an interface
    /// scope-id (`/ip6/<addr>%<scope>/tcp/<port>`) on a multi-interface device,
    /// and libp2p's Multiaddr parser cannot represent a scope-id at all. So
    /// rejecting loopback on that path does not harden anything; it silently
    /// disables the whole Wi-Fi Aware transport.
    ///
    /// This is a SEPARATE audience rather than a `NetworkMode` variant on
    /// purpose. `NetworkMode` is public and threaded through many call sites; a
    /// variant there could be reached by an untrusted caller, and relaxing
    /// `is_unconditionally_routable_ipv4` itself would leak into
    /// `is_globally_routable_ipv4` -- and therefore into `Disclose`, letting us
    /// ADVERTISE a loopback address. Keeping it here means the relaxation is
    /// unreachable except through the one function below, and every existing
    /// `Dial(Local)` / `Dial(Public)` / `Disclose` verdict is bit-for-bit
    /// unchanged.
    ///
    /// It relaxes IPv4 loopback ONLY. `::1` stays rejected, matching the
    /// Kotlin side, which binds `127.0.0.1` explicitly rather than
    /// `InetAddress.getLoopbackAddress()` (which resolves to `::1` on
    /// IPv6-preferring devices, where a dial to 127.0.0.1 would find nobody).
    DialTrustedLocalProxy,
}

/// Rejects the IPv4 addresses that are never a peer, in any context:
///
/// - loopback (127/8) -- SSRF into our own host
/// - unspecified (0.0.0.0) and the rest of 0/8 ("this network")
/// - link-local 169.254/16 -- includes the 169.254.169.254 cloud metadata
///   endpoint, the single highest-value SSRF target in existence
/// - multicast 224/4 and broadcast 255.255.255.255 -- not unicast peers;
///   dialing them is a local-segment amplification primitive
/// - 192.0.0.0/24 (IETF protocol assignments) -- mirrors the same carve-out
///   `swarm::is_discoverable_multiaddr` already makes for mobile/VPN internal
///   NAT addresses
fn is_unconditionally_routable_ipv4(ip: &Ipv4Addr) -> bool {
    let o = ip.octets();
    if ip.is_loopback()
        || ip.is_unspecified()
        || ip.is_link_local()
        || ip.is_multicast()
        || ip.is_broadcast()
    {
        return false;
    }
    // 0.0.0.0/8 "this network" -- only 0.0.0.0 itself is `is_unspecified`.
    if o[0] == 0 {
        return false;
    }
    // 192.0.0.0/24 IETF protocol assignments.
    if o[0] == 192 && o[1] == 0 && o[2] == 0 {
        return false;
    }
    true
}

/// Returns true iff `ip` is an IPv4 address that a peer somewhere else on the
/// internet could actually route a packet to.
///
/// WHY THIS IS NOT `!is_private()` (re-review round 4). `Ipv4Addr::is_private()`
/// covers RFC1918 and nothing else, so every one of these was disclosable:
///
/// - **`100.64.0.0/10` (RFC 6598 CGNAT / "shared address space")** -- the
///   important one. On a carrier-grade-NAT mobile network this is a REAL,
///   live internal host range; the phone's own neighbours sit in it. Telling a
///   stranger `100.64.x.y:port` is the same class of disclosure as telling them
///   `192.168.x.y:port`, and dialing one is the same internal probe.
/// - **`198.18.0.0/15` (RFC 2544 benchmarking)** -- routed inside lab and
///   appliance networks.
/// - **`240.0.0.0/4` (RFC 1112 reserved)** -- never routable; also subsumes
///   `255.255.255.255`.
/// - **`192.0.2.0/24` (RFC 5737 TEST-NET-1)** -- documentation only.
///
/// KNOWN RESIDUAL, stated rather than hidden: the other two RFC 5737
/// documentation prefixes, `198.51.100.0/24` (TEST-NET-2) and `203.0.113.0/24`
/// (TEST-NET-3), are NOT rejected here. They are this workspace's canonical
/// "globally routable peer" test fixtures (~50 occurrences across `core/src`,
/// `core/tests` and `cli`), so rejecting them would make most of the disclosure
/// suite vacuous rather than stricter. They are unassigned documentation space:
/// disclosing one is useless to an attacker and reveals nothing about us, so the
/// residual is cosmetic, not a leak. Closing it means migrating those fixtures
/// to a genuinely routable prefix first.
pub fn is_globally_routable_ipv4(ip: &Ipv4Addr) -> bool {
    if !is_unconditionally_routable_ipv4(ip) {
        return false;
    }
    let o = ip.octets();
    if ip.is_private() {
        return false;
    }
    // 100.64.0.0/10 -- RFC 6598 shared address space (CGNAT).
    if o[0] == 100 && (64..=127).contains(&o[1]) {
        return false;
    }
    // 192.0.2.0/24 -- RFC 5737 TEST-NET-1.
    if o[0] == 192 && o[1] == 0 && o[2] == 2 {
        return false;
    }
    // 198.18.0.0/15 -- RFC 2544 benchmarking.
    if o[0] == 198 && (o[1] == 18 || o[1] == 19) {
        return false;
    }
    // 240.0.0.0/4 -- RFC 1112 reserved (includes 255.255.255.255).
    if o[0] >= 240 {
        return false;
    }
    true
}

/// The IPv4 address embedded in an IPv6 address, for every encoding that makes
/// an IPv4 destination reachable through an IPv6 literal.
///
/// WHY THIS IS NOT `Ipv6Addr::to_ipv4()` (re-review round 4). `to_ipv4()`
/// unwraps only `::/96` (IPv4-compatible) and `::ffff:0:0/96` (IPv4-mapped).
/// It does NOT unwrap RFC 6052 NAT64, so
/// `/ip6/64:ff9b::a9fe:a9fe/tcp/80` **is** `169.254.169.254` -- the cloud
/// metadata endpoint -- and it passed every IPv4 rule in both the dial and the
/// disclosure predicate as "some global IPv6 address". NAT64 is not exotic: it
/// is mandatory-support territory for iOS apps and the default on several US
/// carriers, so a phone genuinely resolves and connects to these.
///
/// Handled here:
/// - `::/96` and `::ffff:0:0/96` (via `to_ipv4`)
/// - `64:ff9b::/96` -- RFC 6052 well-known NAT64 prefix
/// - `64:ff9b:1::/48` -- RFC 8215 local-use NAT64 prefix, using the RFC 6052
///   /48 embedding (IPv4 octets at bits 48..64 and 72..88, with the `u` byte at
///   bits 64..72 skipped)
/// - `2002::/16` -- 6to4, IPv4 at bits 16..48
///
/// Teredo (`2001::/32`) is deliberately NOT handled here because it embeds TWO
/// IPv4 addresses (server and obfuscated client) and both have to clear the
/// rules; see [`teredo_ipv4s`].
pub fn embedded_ipv4(ip: &Ipv6Addr) -> Option<Ipv4Addr> {
    if let Some(v4) = ip.to_ipv4() {
        return Some(v4);
    }
    let o = ip.octets();
    let seg = ip.segments();

    // 64:ff9b::/96 -- well-known NAT64 prefix (RFC 6052 s2.1).
    if seg[0] == 0x0064
        && seg[1] == 0xff9b
        && seg[2] == 0
        && seg[3] == 0
        && seg[4] == 0
        && seg[5] == 0
    {
        return Some(Ipv4Addr::new(o[12], o[13], o[14], o[15]));
    }
    // 64:ff9b:1::/48 -- local-use NAT64 prefix (RFC 8215), /48 embedding.
    if seg[0] == 0x0064 && seg[1] == 0xff9b && seg[2] == 0x0001 {
        return Some(Ipv4Addr::new(o[6], o[7], o[9], o[10]));
    }
    // 2002::/16 -- 6to4 (RFC 3056).
    if seg[0] == 0x2002 {
        return Some(Ipv4Addr::new(o[2], o[3], o[4], o[5]));
    }
    None
}

/// The two IPv4 addresses a Teredo address (`2001::/32`, RFC 4380) embeds: the
/// Teredo server, and the client's own public IPv4 stored obfuscated (bitwise
/// complement).
///
/// Both are real destinations implied by the address, so both have to clear the
/// rules. Note `2001:db8::/32` and the rest of `2001::/16` are ordinary global
/// unicast -- only `2001:0000::/32` is Teredo.
pub fn teredo_ipv4s(ip: &Ipv6Addr) -> Option<(Ipv4Addr, Ipv4Addr)> {
    let seg = ip.segments();
    if seg[0] != 0x2001 || seg[1] != 0x0000 {
        return None;
    }
    let o = ip.octets();
    let server = Ipv4Addr::new(o[4], o[5], o[6], o[7]);
    let client = Ipv4Addr::new(!o[12], !o[13], !o[14], !o[15]);
    Some((server, client))
}

/// IPv4 verdict for one [`Audience`].
fn ipv4_permitted(ip: &Ipv4Addr, audience: Audience) -> bool {
    match audience {
        Audience::Dial(NetworkMode::Local) => is_unconditionally_routable_ipv4(ip),
        Audience::Dial(NetworkMode::Public) => {
            is_unconditionally_routable_ipv4(ip) && !ip.is_private()
        }
        Audience::Disclose => is_globally_routable_ipv4(ip),
        // Loopback permitted here and ONLY here -- see `DialTrustedLocalProxy`.
        // Everything else still has to clear the normal Local bar.
        Audience::DialTrustedLocalProxy => ip.is_loopback() || is_unconditionally_routable_ipv4(ip),
    }
}

/// IPv6 verdict for one [`Audience`].
///
/// Rejects loopback (`::1`), unspecified (`::`), multicast (`ff00::/8`),
/// link-local (`fe80::/10`) and site-local (`fec0::/10`) unconditionally;
/// unique-local (`fc00::/7`) is the IPv6 analogue of RFC1918 and is therefore
/// dropped for `Dial(Public)` and for `Disclose`.
///
/// Every embedded-IPv4 encoding is unwrapped and re-checked as IPv4 FIRST --
/// otherwise `::ffff:127.0.0.1`, `64:ff9b::a9fe:a9fe`, `2002:c0a8:0101::` and
/// friends are each a one-line bypass of the entire IPv4 rule set.
fn ipv6_permitted(ip: &Ipv6Addr, audience: Audience) -> bool {
    if let Some(v4) = embedded_ipv4(ip) {
        // `::` and `::1` unwrap to 0.0.0.0 / 0.0.0.1, both of which the 0/8
        // rule rejects -- which is the answer we want anyway.
        return ipv4_permitted(&v4, audience);
    }
    if let Some((server, client)) = teredo_ipv4s(ip) {
        return ipv4_permitted(&server, audience) && ipv4_permitted(&client, audience);
    }
    if ip.is_loopback() || ip.is_unspecified() || ip.is_multicast() {
        return false;
    }
    // std lacks stable helpers for these on the pinned toolchain, so check the
    // top bits of the first 16-bit segment directly.
    let seg0 = ip.segments()[0];
    if seg0 & 0xffc0 == 0xfe80 || seg0 & 0xffc0 == 0xfec0 {
        return false;
    }
    let unique_local = (seg0 & 0xfe00) == 0xfc00;
    match audience {
        Audience::Dial(NetworkMode::Local) => true,
        // TrustedLocalProxy is like Local for IPv6 — but ::1 still falls
        // through to the loopback early-return above and stays rejected.
        Audience::DialTrustedLocalProxy => true,
        Audience::Dial(NetworkMode::Public) | Audience::Disclose => !unique_local,
    }
}

/// Returns true iff `addr` is worth dialing / safe to disclose.
///
/// Ordering matters and is load-bearing: components are examined in wire
/// order, so for a circuit address such as
/// `/ip4/R/tcp/443/p2p/QmRelay/p2p-circuit/p2p/QmTarget` the RELAY hop
/// (`/ip4/R/tcp/443`) is fully validated, and everything after the
/// `/p2p-circuit` marker is accepted unconditionally -- a relayed peer's own
/// address is not something we dial and not something we can reason about.
/// This reproduces the CLI's short-circuit semantics exactly while closing the
/// "relay hop is loopback" hole implicitly.
///
/// An address with no transport component at all (`""`, `/p2p/QmX`,
/// `/p2p-circuit`) is REJECTED. Note `"".parse::<Multiaddr>()` returns
/// `Ok(<empty>)`, so "it parsed" is not evidence of anything (review F9).
///
/// DNS-form components are governed by `dns`; see [`DnsPolicy`] for why a name
/// supplied by a remote peer is not validatable at all. Note the DNS check runs
/// BEFORE the `P2pCircuit` short-circuit can fire, so
/// `/dns4/evil.example/tcp/80/p2p-circuit` -- whose relay hop we would really
/// dial -- is rejected too.
pub fn is_dialable_multiaddr_parsed(addr: &Multiaddr, mode: NetworkMode, dns: DnsPolicy) -> bool {
    check_multiaddr(addr, Audience::Dial(mode), dns)
}

/// Dial verdict for an address THIS PROCESS created (the Wi-Fi Aware
/// loopback proxy). See `Audience::DialTrustedLocalProxy`. Never call this
/// with an address that came from a remote peer or from user input.
pub fn is_dialable_trusted_local_proxy_parsed(addr: &Multiaddr, dns: DnsPolicy) -> bool {
    check_multiaddr(addr, Audience::DialTrustedLocalProxy, dns)
}

/// The ONE multiaddr traversal. Both public predicates delegate here so a
/// protocol component can never be interpreted two different ways.
fn check_multiaddr(addr: &Multiaddr, audience: Audience, dns: DnsPolicy) -> bool {
    let mut has_transport = false;
    let mut has_circuit = false;

    for proto in addr.iter() {
        // After the first relay marker, the remaining components describe the
        // target peer and are not independently dialed. Still reject a second
        // marker: libp2p does not support nested relay circuits, and retaining
        // one in the ledger creates a retry storm against an invalid path.
        if has_circuit {
            if matches!(proto, Protocol::P2pCircuit) {
                return false;
            }
            continue;
        }

        match proto {
            // Everything beyond the relay hop belongs to the relayed peer.
            Protocol::P2pCircuit => has_circuit = true,
            Protocol::Ip4(ip) => {
                has_transport = true;
                if !ipv4_permitted(&ip, audience) {
                    return false;
                }
            }
            Protocol::Ip6(ip) => {
                has_transport = true;
                if !ipv6_permitted(&ip, audience) {
                    return false;
                }
            }
            Protocol::Dns(_) | Protocol::Dns4(_) | Protocol::Dns6(_) | Protocol::Dnsaddr(_) => {
                if dns == DnsPolicy::Reject {
                    return false;
                }
                has_transport = true;
            }
            _ => {}
        }
    }

    has_transport
}

/// String convenience wrapper over [`is_dialable_multiaddr_parsed`].
///
/// An unparseable string is not dialable. (The CLI's previous implementation
/// split on `/` and returned `true` for garbage that happened to contain no
/// recognised IP component; that is now rejected.)
pub fn is_dialable_multiaddr(multiaddr: &str, mode: NetworkMode, dns: DnsPolicy) -> bool {
    match multiaddr.parse::<Multiaddr>() {
        Ok(addr) => is_dialable_multiaddr_parsed(&addr, mode, dns),
        Err(_) => false,
    }
}

/// Returns true iff `addr` is safe to HAND TO SOMEONE ELSE -- in a
/// `/sc/ledger-exchange/1.0.0` reply, or baked into an invite QR.
///
/// DISCLOSURE IS NOT DIALABILITY (re-review NEW-2). The first remediation pass
/// reused the dial predicate for the exchange reply and passed
/// `NetworkMode::Local` at the call site, because `Local` is what keeps the
/// LAN/mesh transport priority order working. But `Local` deliberately skips
/// the `is_private()` check, and `record_connection` is deliberately unfiltered,
/// so every LAN peer we ever dialed became a *proven, disclosable* record:
/// internal subnet, live host:port, and each neighbour's `last_peer_id`. That
/// is an internal network map handed to any peer that completed a Noise
/// handshake.
///
/// The two predicates answer different questions and must not share a mode:
/// - "can I reach it?" is contextual -- an RFC1918 peer on my own LAN is
///   perfectly reachable, which is why the dial path keeps `Local`;
/// - "may I tell a stranger about it?" is not -- an address the recipient
///   cannot route to is, by construction, only useful to them as
///   reconnaissance about us.
///
/// So this function takes NO `NetworkMode` parameter. There is no argument a
/// call site can pass to weaken it, which is the point: the previous bug was
/// exactly a call site passing the wrong mode.
///
/// DNS is rejected for a second, independent reason: a name like
/// `/dns4/nas.corp.internal/tcp/443` leaks internal naming even when it does
/// not resolve for the recipient.
///
/// GLOBALLY ROUTABLE, NOT `!is_private()` (re-review round 4). This used to be
/// literally `is_dialable_multiaddr_parsed(addr, Public, Reject)`, i.e. the dial
/// predicate with `is_private()` switched on. `is_private()` is RFC1918 and
/// nothing else, so CGNAT `100.64.0.0/10` -- a live internal host range on every
/// carrier-grade-NAT mobile network -- plus `198.18.0.0/15` and `240.0.0.0/4`
/// were all disclosable. See [`is_globally_routable_ipv4`].
pub fn is_disclosable_multiaddr_parsed(addr: &Multiaddr) -> bool {
    check_multiaddr(addr, Audience::Disclose, DnsPolicy::Reject)
}

/// String convenience wrapper over [`is_disclosable_multiaddr_parsed`].
pub fn is_disclosable_multiaddr(multiaddr: &str) -> bool {
    match multiaddr.parse::<Multiaddr>() {
        Ok(addr) => is_disclosable_multiaddr_parsed(&addr),
        Err(_) => false,
    }
}

/// Returns true if a private `multiaddr` may be disclosed to the observed
/// requester on the same local subnet.
///
/// The requester address is required evidence. A local listener list alone is
/// not sufficient: it describes this node, not the peer receiving the data.
/// Missing or relayed requester addresses fail closed. We use /24 for private
/// IPv4 and /64 for ULA IPv6, which is deliberately narrower than the RFC1918
/// address classes. CGNAT is accepted as a dialable private class elsewhere,
/// but it is not evidence of local adjacency here because unrelated carrier
/// subscribers can collide within the same /24.
pub fn is_disclosable_on_rfc1918_network(
    multiaddr: &str,
    requester_addr: Option<&str>,
    my_addrs: &[String],
) -> bool {
    let Ok(addr) = multiaddr.parse::<Multiaddr>() else {
        return false;
    };

    if addr
        .iter()
        .any(|proto| matches!(proto, Protocol::P2pCircuit))
    {
        return false;
    }

    let Some(requester_addr) = requester_addr else {
        return false;
    };
    let Ok(requester_multiaddr) = requester_addr.parse::<Multiaddr>() else {
        return false;
    };
    // A circuit address identifies the relay hop, not the peer's local
    // network. Treating that hop as observed requester adjacency would allow
    // a LAN-resident relay to authorize private-ledger disclosure to a remote
    // circuit peer.
    if requester_multiaddr
        .iter()
        .any(|proto| matches!(proto, Protocol::P2pCircuit))
    {
        return false;
    }
    let Some(requester_ip) = first_ip_component(requester_addr) else {
        return false;
    };
    // CGNAT is shared address space, not proof that the requester is on our
    // local network. It may remain dialable in Local mode, but it must not
    // authorize private-ledger disclosure.
    if !is_private_adjacency_ip(&requester_ip) {
        return false;
    }

    let Some(entry_ip) = first_ip_component(multiaddr) else {
        return false;
    };
    if !is_private_adjacency_ip(&entry_ip) || !same_private_subnet(&entry_ip, &requester_ip) {
        return false;
    }

    // Require concrete listener evidence as well as an observed direct
    // requester. Wildcard-only listeners, DNS-only listeners, and the startup
    // window before a concrete listen address is reported carry no usable
    // local-subnet evidence, so they must fail closed.
    let concrete_local_ips: Vec<_> = my_addrs
        .iter()
        .filter_map(|a| first_ip_component(a))
        .collect();
    !concrete_local_ips.is_empty()
        && concrete_local_ips
            .iter()
            .any(|local_ip| same_private_subnet(local_ip, &requester_ip))
}

fn first_ip_component(multiaddr: &str) -> Option<std::net::IpAddr> {
    let addr = multiaddr.parse::<Multiaddr>().ok()?;
    addr.iter().find_map(|proto| match proto {
        Protocol::Ip4(ip) => Some(std::net::IpAddr::V4(ip)),
        Protocol::Ip6(ip) => Some(std::net::IpAddr::V6(ip)),
        _ => None,
    })
}

fn is_safe_private_ip(ip: &std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(ip) => {
            !ip.is_loopback()
                && !ip.is_unspecified()
                && !ip.is_link_local()
                && !ip.is_multicast()
                && !ip.is_broadcast()
                && (ip.is_private() || is_cgnat(ip))
        }
        std::net::IpAddr::V6(ip) => {
            !ip.is_loopback()
                && !ip.is_unspecified()
                && !ip.is_multicast()
                && !ip.is_unicast_link_local()
                && ((ip.segments()[0] & 0xfe00) == 0xfc00
                    || embedded_ipv4(ip).is_some_and(|v4| {
                        !v4.is_loopback()
                            && !v4.is_unspecified()
                            && !v4.is_link_local()
                            && !v4.is_multicast()
                            && !v4.is_broadcast()
                            && (v4.is_private() || is_cgnat(&v4))
                    }))
        }
    }
}

/// Returns true only for an address class that can provide local-adjacency
/// evidence. Unlike [`is_safe_private_ip`], this deliberately excludes CGNAT:
/// the same carrier /24 can contain unrelated subscribers.
fn is_private_adjacency_ip(ip: &std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(ip) => {
            let ip_addr = std::net::IpAddr::V4(*ip);
            is_safe_private_ip(&ip_addr) && ip.is_private()
        }
        std::net::IpAddr::V6(ip) => {
            let ip_addr = std::net::IpAddr::V6(*ip);
            if !is_safe_private_ip(&ip_addr) {
                return false;
            }
            if (ip.segments()[0] & 0xfe00) == 0xfc00 {
                return true;
            }
            embedded_ipv4(ip).is_some_and(|v4| {
                let ip_addr = std::net::IpAddr::V4(v4);
                is_safe_private_ip(&ip_addr) && v4.is_private()
            })
        }
    }
}

fn same_private_subnet(left: &std::net::IpAddr, right: &std::net::IpAddr) -> bool {
    match (left, right) {
        (std::net::IpAddr::V4(left), std::net::IpAddr::V4(right)) => {
            let left = left.octets();
            let right = right.octets();
            left[..3] == right[..3]
        }
        (std::net::IpAddr::V6(left), std::net::IpAddr::V6(right)) => {
            if (left.segments()[0] & 0xfe00) != 0xfc00 || (right.segments()[0] & 0xfe00) != 0xfc00 {
                return false;
            }
            left.segments()[..4] == right.segments()[..4]
        }
        (std::net::IpAddr::V6(left), std::net::IpAddr::V4(right)) => embedded_ipv4(left)
            .is_some_and(|left| {
                same_private_subnet(&std::net::IpAddr::V4(left), &std::net::IpAddr::V4(*right))
            }),
        (std::net::IpAddr::V4(left), std::net::IpAddr::V6(right)) => embedded_ipv4(right)
            .is_some_and(|right| {
                same_private_subnet(&std::net::IpAddr::V4(*left), &std::net::IpAddr::V4(right))
            }),
    }
}

fn is_cgnat(ip: &Ipv4Addr) -> bool {
    let o = ip.octets();
    o[0] == 100 && (64..=127).contains(&o[1])
}

/// Returns true iff `addr` may be WRITTEN INTO A LEDGER as an address this node
/// actually reached.
///
/// This is deliberately the weakest of the three predicates, and the reason it
/// exists as its own named function rather than as a `bool` argument to one of
/// the others: "we connected here" is a different claim from "we could dial
/// here" and from "a stranger could route here", and each deserves a predicate
/// that cannot be reconfigured into another.
///
/// It requires an IP transport component and rejects DNS forms outright. It does
/// NOT reject loopback or RFC1918: an address a socket just came off
/// demonstrably works for us, and filtering it at ingestion would erase the
/// evidence that the re-dial and disclosure gates are supposed to filter later.
/// See [`crate::store::ledger_entry::LedgerManager::record_connection`].
pub fn is_recordable_multiaddr(multiaddr: &str) -> bool {
    let Ok(addr) = multiaddr.parse::<Multiaddr>() else {
        return false;
    };
    let mut has_ip_transport = false;
    let mut has_circuit = false;
    for proto in addr.iter() {
        match proto {
            // Everything past the relay hop belongs to the relayed peer; the
            // hop itself has already been seen by this point. A second relay
            // marker is malformed and cannot be dialed by libp2p.
            Protocol::P2pCircuit => {
                if has_circuit {
                    return false;
                }
                has_circuit = true;
            }
            Protocol::Ip4(_) | Protocol::Ip6(_) => has_ip_transport = true,
            Protocol::Dns(_) | Protocol::Dns4(_) | Protocol::Dns6(_) | Protocol::Dnsaddr(_)
                if !has_circuit =>
            {
                return false;
            }
            _ => {}
        }
    }
    has_ip_transport
}

/// Remove the peer-id component(s) that identify the *endpoint* of a
/// multiaddr, leaving the transport path.
///
/// CIRCUIT CORRECTNESS (review F8): a naive `find("/p2p/")` truncation turns
/// `/ip4/A/tcp/443/p2p/QmRelay/p2p-circuit/p2p/QmTarget` into
/// `/ip4/A/tcp/443` -- the RELAY's address. Paired with a ledger entry whose
/// `last_peer_id` is still QmTarget, that produces a wire record asserting
/// "QmTarget is directly reachable at the relay's IP:port", which recipients
/// feed straight into `kademlia.add_address()`. That is DHT poisoning plus a
/// distributed dial amplifier aimed at an arbitrary host, and it happens with
/// no attacker present, from honest circuit entries.
///
/// So: keep everything up to and including the LAST `/p2p-circuit`, and strip
/// `/p2p/` components only after it (or everywhere, if there is no circuit).
/// The relay's own peer id is part of the *address* -- it is required to dial
/// the circuit -- and must survive.
pub fn strip_peer_id_multiaddr(addr: &Multiaddr) -> Multiaddr {
    let protocols: Vec<Protocol> = addr.iter().collect();
    let last_circuit = protocols
        .iter()
        .rposition(|p| matches!(p, Protocol::P2pCircuit));

    let mut out = Multiaddr::empty();
    for (idx, proto) in protocols.into_iter().enumerate() {
        let after_circuit = match last_circuit {
            Some(circuit_idx) => idx > circuit_idx,
            None => true,
        };
        if after_circuit && matches!(proto, Protocol::P2p(_)) {
            continue;
        }
        out.push(proto);
    }
    out
}

/// String convenience wrapper over [`strip_peer_id_multiaddr`].
///
/// Unparseable input is returned unchanged: this function's job is
/// normalisation, not validation. Callers must still run
/// [`is_dialable_multiaddr`] afterwards, which rejects it.
pub fn strip_peer_id(multiaddr: &str) -> String {
    match multiaddr.parse::<Multiaddr>() {
        Ok(addr) => strip_peer_id_multiaddr(&addr).to_string(),
        Err(_) => multiaddr.to_string(),
    }
}

/// Returns true iff `candidate` is one of this node's own known addresses
/// (listen or external) -- i.e. dialing it would be a self-dial.
///
/// Compares the transport address only (peer-id components stripped on both
/// sides), since the same node can be observed with or without its own peer id
/// attached depending on which ledger entry produced it.
pub fn is_self_address(candidate: &str, my_addrs: &[String]) -> bool {
    let stripped_candidate = strip_peer_id(candidate);
    if stripped_candidate.is_empty() {
        return false;
    }
    my_addrs
        .iter()
        .any(|a| strip_peer_id(a) == stripped_candidate)
}

/// Combined gate used by every core call site that turns remote-supplied
/// address data into a dial candidate, a stored ledger entry, or a disclosed
/// wire record: syntactically valid, routable under `mode`, and not us.
pub fn is_acceptable_peer_address(
    candidate: &str,
    mode: NetworkMode,
    dns: DnsPolicy,
    my_addrs: &[String],
) -> bool {
    is_dialable_multiaddr(candidate, mode, dns) && !is_self_address(candidate, my_addrs)
}

/// Returns true iff `addr` is a self-referential or self-destination circuit
/// address — i.e. any circuit multiaddr whose final destination equals
/// `local_peer_id`, or which routes through `local_peer_id` at any relay hop.
///
/// WHY THIS EXISTS (P0 mesh growth panic, 2026-08-09): when the mesh grows
/// beyond two nodes and all nodes advertise as relays, circuit topologies
/// multiply quickly. A node can receive or construct circuit listener/dial
/// addresses that route through a peer and back to this same node:
///
///   `/ip6/::1/tcp/8080/p2p/<SELF>/p2p-circuit/p2p/<PEER>/p2p-circuit/p2p/<SELF>`
///   `/ip6/::1/tcp/9001/p2p/<MAC>/p2p-circuit/p2p/<PEER>/p2p-circuit/p2p/<SELF>`
///   `/ip4/1.2.3.4/tcp/443/p2p/<RELAY>/p2p-circuit/p2p/<SELF>`
///
/// A circuit whose final destination is this node, or which routes through
/// this node to reach an endpoint, is pathological. Maintaining or dialing such
/// addresses produces concurrent self-referential connection loops that cause
/// `libp2p-request-response`'s internal connection map to desynchronise from the
/// swarm's connection accounting, triggering
/// `debug_assert_eq!(connections.is_empty(), remaining_established == 0)` at
/// line 678 in debug builds.
///
/// This filter returns `true` (self-loop detected) if:
/// 1. The final `/p2p/<peer-id>` destination in `addr` equals `local_peer_id`.
/// 2. `addr` contains at least one `/p2p-circuit` hop and `local_peer_id`
///    appears at any position (relay hop, intermediate hop, or destination).
///
/// Plain non-circuit addresses to other peers and legitimate circuits to
/// different peers return `false` (unaffected).
pub fn is_self_circuit_multiaddr_parsed(addr: &Multiaddr, local_peer_id: &PeerId) -> bool {
    let mut has_circuit = false;
    let mut has_self_peer = false;
    let mut last_peer_id: Option<PeerId> = None;

    for proto in addr.iter() {
        match proto {
            Protocol::P2pCircuit => {
                has_circuit = true;
            }
            Protocol::P2p(pid) => {
                if pid == *local_peer_id {
                    has_self_peer = true;
                }
                last_peer_id = Some(pid);
            }
            _ => {}
        }
    }

    // 1. Final destination equals local peer ID.
    if last_peer_id == Some(*local_peer_id) {
        return true;
    }

    // 2. Any circuit address containing local peer ID at any position.
    if has_circuit && has_self_peer {
        return true;
    }

    false
}

/// String convenience wrapper over [`is_self_circuit_multiaddr_parsed`].
///
/// Unparseable input returns `false` (normalisation/syntax validation is handled
/// by [`is_dialable_multiaddr`]).
pub fn is_self_circuit(multiaddr: &str, local_peer_id: &PeerId) -> bool {
    match multiaddr.parse::<Multiaddr>() {
        Ok(addr) => is_self_circuit_multiaddr_parsed(&addr, local_peer_id),
        Err(_) => false,
    }
}

/// String convenience wrapper over [`is_self_circuit_multiaddr_parsed`].
pub fn is_self_circuit_multiaddr(multiaddr: &str, local_peer_id: &PeerId) -> bool {
    is_self_circuit(multiaddr, local_peer_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOCAL: NetworkMode = NetworkMode::Local;
    const PUBLIC: NetworkMode = NetworkMode::Public;
    /// Provenance of every address in the tests below unless stated otherwise:
    /// a peer told us. That is the case that matters.
    const REMOTE: DnsPolicy = DnsPolicy::Reject;
    const CONFIGURED: DnsPolicy = DnsPolicy::AllowLocallyConfigured;

    #[test]
    fn rejects_non_routable_ipv4_in_every_mode() {
        for mode in [LOCAL, PUBLIC] {
            assert!(!is_dialable_multiaddr(
                "/ip4/127.0.0.1/tcp/8080",
                mode,
                REMOTE
            ));
            assert!(!is_dialable_multiaddr(
                "/ip4/0.0.0.0/tcp/9001",
                mode,
                REMOTE
            ));
            assert!(!is_dialable_multiaddr(
                "/ip4/0.1.2.3/tcp/9001",
                mode,
                REMOTE
            ));
            // Cloud metadata service -- the marquee SSRF target.
            assert!(!is_dialable_multiaddr(
                "/ip4/169.254.169.254/tcp/80",
                mode,
                REMOTE
            ));
            assert!(!is_dialable_multiaddr(
                "/ip4/224.0.0.1/udp/9001/quic-v1",
                mode,
                REMOTE
            ));
            assert!(!is_dialable_multiaddr(
                "/ip4/255.255.255.255/tcp/9001",
                mode,
                REMOTE
            ));
            assert!(!is_dialable_multiaddr(
                "/ip4/192.0.0.8/tcp/9001",
                mode,
                REMOTE
            ));
        }
    }

    #[test]
    fn rejects_non_routable_ipv6_in_every_mode() {
        for mode in [LOCAL, PUBLIC] {
            assert!(!is_dialable_multiaddr("/ip6/::1/tcp/9001", mode, REMOTE));
            assert!(!is_dialable_multiaddr("/ip6/::/tcp/9001", mode, REMOTE));
            assert!(!is_dialable_multiaddr(
                "/ip6/fe80::1897:a8ff:fec5:3d16/tcp/443",
                mode,
                REMOTE
            ));
            assert!(!is_dialable_multiaddr(
                "/ip6/fec0::1/tcp/9001",
                mode,
                REMOTE
            ));
            assert!(!is_dialable_multiaddr(
                "/ip6/ff02::1/tcp/9001",
                mode,
                REMOTE
            ));
        }
    }

    #[test]
    fn ipv4_mapped_ipv6_cannot_bypass_the_ipv4_rules() {
        // Without the to_ipv4() unwrap these all sail through as "some global
        // v6 address".
        assert!(!is_dialable_multiaddr(
            "/ip6/::ffff:127.0.0.1/tcp/8080",
            LOCAL,
            REMOTE
        ));
        assert!(!is_dialable_multiaddr(
            "/ip6/::ffff:169.254.169.254/tcp/80",
            LOCAL,
            REMOTE
        ));
        assert!(!is_dialable_multiaddr(
            "/ip6/::ffff:192.168.1.1/tcp/443",
            PUBLIC,
            REMOTE
        ));
    }

    #[test]
    fn private_ranges_follow_network_mode() {
        assert!(is_dialable_multiaddr(
            "/ip4/10.0.2.16/tcp/9001",
            LOCAL,
            REMOTE
        ));
        assert!(is_dialable_multiaddr(
            "/ip4/192.168.1.5/tcp/9001",
            LOCAL,
            REMOTE
        ));
        assert!(is_dialable_multiaddr(
            "/ip4/172.16.4.4/tcp/9001",
            LOCAL,
            REMOTE
        ));
        assert!(!is_dialable_multiaddr(
            "/ip4/10.0.2.16/tcp/9001",
            PUBLIC,
            REMOTE
        ));
        assert!(!is_dialable_multiaddr(
            "/ip4/192.168.1.5/tcp/9001",
            PUBLIC,
            REMOTE
        ));
        assert!(!is_dialable_multiaddr(
            "/ip4/172.16.4.4/tcp/9001",
            PUBLIC,
            REMOTE
        ));
        // IPv6 unique-local is the RFC1918 analogue.
        assert!(is_dialable_multiaddr(
            "/ip6/fd00::1/tcp/9001",
            LOCAL,
            REMOTE
        ));
        assert!(!is_dialable_multiaddr(
            "/ip6/fd00::1/tcp/9001",
            PUBLIC,
            REMOTE
        ));
    }

    #[test]
    fn accepts_globally_routable_addresses() {
        assert!(is_dialable_multiaddr(
            "/ip4/1.2.3.4/tcp/9001",
            LOCAL,
            REMOTE
        ));
        assert!(is_dialable_multiaddr(
            "/ip4/198.51.100.11/tcp/9000",
            PUBLIC,
            REMOTE
        ));
        assert!(is_dialable_multiaddr(
            "/ip6/2606:4700:4700::1111/tcp/9001",
            LOCAL,
            REMOTE
        ));
        // A name is fine when the OPERATOR chose it.
        assert!(is_dialable_multiaddr(
            "/dns4/relay.example/tcp/443",
            PUBLIC,
            CONFIGURED
        ));
    }

    // ------------------------------------------------------------------
    // NEW-1 -- DNS bypasses all address validation
    // ------------------------------------------------------------------

    /// The module previously had exactly ONE DNS assertion and it was positive,
    /// which is precisely why the bypass survived a full adversarial review.
    ///
    /// Every one of these strings sets `has_transport = true` and validates
    /// nothing under the old code, so every IPv4/IPv6 rule above is skipped and
    /// the desktop resolver dials whatever the zone says.
    #[test]
    fn remote_supplied_dns_is_rejected_in_every_form_and_mode() {
        let dns_forms = [
            "/dns4/evil.example/tcp/80",
            "/dns6/evil.example/tcp/80",
            "/dns/evil.example/tcp/80",
            "/dnsaddr/evil.example",
            "/dnsaddr/evil.example/tcp/443",
            "/dns4/evil.example/udp/9001/quic-v1",
            "/dns4/metadata.google.internal/tcp/80",
        ];
        for mode in [LOCAL, PUBLIC] {
            for addr in dns_forms {
                assert!(
                    !is_dialable_multiaddr(addr, mode, REMOTE),
                    "{addr} was accepted from a remote peer in {mode:?}: \
                     `A evil.example -> 169.254.169.254` is now a dial target"
                );
            }
        }
    }

    /// A DNS relay hop must not be laundered through the `/p2p-circuit`
    /// short-circuit: the hop is the part we actually connect a socket to.
    #[test]
    fn remote_supplied_dns_cannot_hide_behind_a_circuit_marker() {
        assert!(!is_dialable_multiaddr(
            "/dns4/evil.example/tcp/443/p2p-circuit",
            LOCAL,
            REMOTE
        ));
        assert!(!is_dialable_multiaddr(
            "/dns4/evil.example/tcp/443/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN/p2p-circuit/p2p/12D3KooWSHj3RRbBjD15g6wekV8y3mm57Pobmps2g2WJm6F67Lay",
            LOCAL,
            REMOTE
        ));
        // ...and the mixed form, where a legitimate-looking IP hop is followed
        // by a name.
        assert!(!is_dialable_multiaddr(
            "/ip4/1.2.3.4/tcp/443/dns4/evil.example/tcp/80",
            LOCAL,
            REMOTE
        ));
    }

    /// The rejection is provenance-based, not a blanket ban: an operator's own
    /// bootstrap relay name still works, which is what keeps the internet-relay
    /// tier of the transport priority order alive.
    #[test]
    fn locally_configured_dns_still_works() {
        assert!(is_dialable_multiaddr(
            "/dns4/relay.example/tcp/443",
            LOCAL,
            CONFIGURED
        ));
        assert!(is_dialable_multiaddr(
            "/dnsaddr/bootstrap.example",
            PUBLIC,
            CONFIGURED
        ));
        assert!(is_dialable_multiaddr(
            "/dns4/relay.example/tcp/443/p2p-circuit",
            PUBLIC,
            CONFIGURED
        ));
    }

    /// Fail-closed: a call site that forgets to think about provenance gets the
    /// strict answer.
    #[test]
    fn dns_policy_defaults_to_reject() {
        assert_eq!(DnsPolicy::default(), DnsPolicy::Reject);
        assert!(!is_dialable_multiaddr(
            "/dns4/evil.example/tcp/80",
            LOCAL,
            DnsPolicy::default()
        ));
    }

    // ------------------------------------------------------------------
    // NEW-2 -- disclosure is not dialability
    // ------------------------------------------------------------------

    /// The exchange reply used to run the dial predicate in
    /// `NetworkMode::Local`, which skips `is_private()` entirely. Every LAN peer
    /// we had ever dialed was therefore a disclosable record: internal subnet,
    /// live host:port, neighbour peer id.
    #[test]
    fn disclosure_always_drops_private_ranges() {
        for addr in [
            "/ip4/192.168.1.5/tcp/9001",
            "/ip4/10.0.2.16/tcp/9001",
            "/ip4/172.16.4.4/tcp/9001",
            "/ip6/fd00::1/tcp/9001",
            "/ip6/::ffff:192.168.1.1/tcp/443",
        ] {
            assert!(
                !is_disclosable_multiaddr(addr),
                "{addr} would be handed to any peer that completed a handshake"
            );
            // ...even though it is legitimately DIALABLE on our own LAN. This
            // pair of assertions is the whole point of the two predicates.
            assert!(is_dialable_multiaddr(addr, LOCAL, REMOTE));
        }
    }

    /// Everything the dial predicate rejects unconditionally is also
    /// undisclosable, and an internal hostname is not disclosable either.
    #[test]
    fn disclosure_drops_loopback_link_local_and_dns() {
        for addr in [
            "/ip4/127.0.0.1/tcp/8080",
            "/ip6/::1/tcp/8080",
            "/ip4/169.254.169.254/tcp/80",
            "/ip4/0.0.0.0/tcp/9001",
            "/ip4/224.0.0.1/tcp/9001",
            "/ip4/255.255.255.255/tcp/9001",
            "/dns4/nas.corp.internal/tcp/443",
            "/dnsaddr/vpn.corp.internal",
            "",
            "/p2p-circuit",
            "not-a-multiaddr",
        ] {
            assert!(!is_disclosable_multiaddr(addr), "{addr} was disclosable");
        }
    }

    #[test]
    fn disclosure_keeps_globally_routable_addresses() {
        assert!(is_disclosable_multiaddr("/ip4/198.51.100.11/tcp/9001"));
        assert!(is_disclosable_multiaddr(
            "/ip6/2606:4700:4700::1111/tcp/443"
        ));
        assert!(is_disclosable_multiaddr(
            "/ip4/203.0.113.9/tcp/443/p2p-circuit"
        ));
    }

    #[test]
    fn circuit_validates_the_relay_hop_and_allows_the_target() {
        // Routable relay hop -- allowed (CLI parity).
        assert!(is_dialable_multiaddr(
            "/ip4/1.2.3.4/tcp/9001/p2p-circuit",
            LOCAL,
            REMOTE
        ));
        assert!(is_dialable_multiaddr(
            "/ip4/1.2.3.4/tcp/443/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN/p2p-circuit/p2p/12D3KooWSHj3RRbBjD15g6wekV8y3mm57Pobmps2g2WJm6F67Lay",
            LOCAL,
            REMOTE
        ));
        // Loopback relay hop -- rejected, because the hop is validated before
        // the circuit marker short-circuits.
        assert!(!is_dialable_multiaddr(
            "/ip4/127.0.0.1/tcp/443/p2p-circuit/p2p/12D3KooWSHj3RRbBjD15g6wekV8y3mm57Pobmps2g2WJm6F67Lay",
            LOCAL,
            REMOTE
        ));
    }

    #[test]
    fn rejects_nested_relay_circuits() {
        let nested = "/ip4/203.0.113.9/tcp/443/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN/p2p-circuit/p2p/12D3KooWSHj3RRbBjD15g6wekV8y3mm57Pobmps2g2WJm6F67Lay/p2p-circuit/p2p/12D3KooWJ7t4Qz4VwzYbP4uU6z4aGz2gZ4nM6mX2sJ6j7V8b9c1d";
        assert!(!is_dialable_multiaddr(nested, LOCAL, REMOTE));
        assert!(!is_recordable_multiaddr(nested));
    }

    #[test]
    fn rejects_addresses_with_no_transport_component() {
        // F9: "" parses as Ok(<empty>).
        assert!("".parse::<Multiaddr>().is_ok());
        assert!(!is_dialable_multiaddr("", LOCAL, REMOTE));
        assert!(!is_dialable_multiaddr(
            "/p2p/12D3KooWSHj3RRbBjD15g6wekV8y3mm57Pobmps2g2WJm6F67Lay",
            LOCAL,
            REMOTE
        ));
        assert!(!is_dialable_multiaddr("/p2p-circuit", LOCAL, REMOTE));
        assert!(!is_dialable_multiaddr("not-a-multiaddr", LOCAL, REMOTE));
    }

    #[test]
    fn strip_peer_id_keeps_plain_address() {
        assert_eq!(
            strip_peer_id(
                "/ip4/1.2.3.4/tcp/9001/p2p/12D3KooWSHj3RRbBjD15g6wekV8y3mm57Pobmps2g2WJm6F67Lay"
            ),
            "/ip4/1.2.3.4/tcp/9001"
        );
        assert_eq!(
            strip_peer_id("/ip4/1.2.3.4/tcp/9001"),
            "/ip4/1.2.3.4/tcp/9001"
        );
    }

    #[test]
    fn strip_peer_id_does_not_collapse_circuit_to_relay_address() {
        // F8 regression: the naive find("/p2p/") implementation returned
        // "/ip4/1.2.3.4/tcp/443" here.
        let circuit = "/ip4/1.2.3.4/tcp/443/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN/p2p-circuit/p2p/12D3KooWSHj3RRbBjD15g6wekV8y3mm57Pobmps2g2WJm6F67Lay";
        let stripped = strip_peer_id(circuit);
        assert_eq!(
            stripped,
            "/ip4/1.2.3.4/tcp/443/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN/p2p-circuit"
        );
        assert!(
            stripped.contains("/p2p-circuit"),
            "circuit marker must survive stripping"
        );
        assert!(
            !stripped.contains("12D3KooWSHj3RRbBjD15g6wekV8y3mm57Pobmps2g2WJm6F67Lay"),
            "target peer id must not survive stripping"
        );
    }

    #[test]
    fn strip_peer_id_of_bare_p2p_is_empty() {
        assert_eq!(
            strip_peer_id("/p2p/12D3KooWSHj3RRbBjD15g6wekV8y3mm57Pobmps2g2WJm6F67Lay"),
            ""
        );
    }

    #[test]
    fn self_address_matches_regardless_of_peer_id_placement() {
        let my_addrs = vec![
            "/ip4/192.168.0.121/tcp/9001".to_string(),
            "/ip4/1.2.3.4/tcp/9001/p2p/12D3KooWSHj3RRbBjD15g6wekV8y3mm57Pobmps2g2WJm6F67Lay"
                .to_string(),
        ];
        assert!(is_self_address("/ip4/192.168.0.121/tcp/9001", &my_addrs));
        assert!(is_self_address(
            "/ip4/192.168.0.121/tcp/9001/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
            &my_addrs
        ));
        assert!(is_self_address("/ip4/1.2.3.4/tcp/9001", &my_addrs));
        assert!(!is_self_address("/ip4/10.0.2.16/tcp/9001", &my_addrs));
        // An empty candidate must never "match" an empty own-address entry.
        assert!(!is_self_address("", &["".to_string()]));
    }

    // ------------------------------------------------------------------
    // Round 4 -- embedded-IPv4 encodings other than ::ffff:
    // ------------------------------------------------------------------

    /// `Ipv6Addr::to_ipv4()` unwraps `::/96` and `::ffff:0:0/96` and NOTHING
    /// else, so every one of these was "some global IPv6 address" and passed
    /// both predicates. `/ip6/64:ff9b::a9fe:a9fe/tcp/80` IS
    /// `169.254.169.254:80` -- the cloud metadata endpoint -- reached through a
    /// NAT64 gateway that iOS and several US carriers run by default.
    #[test]
    fn nat64_wellknown_prefix_cannot_launder_an_internal_ipv4() {
        // 0xa9fe_a9fe == 169.254.169.254
        for addr in [
            "/ip6/64:ff9b::a9fe:a9fe/tcp/80",
            "/ip6/64:ff9b::7f00:1/tcp/8080",    // 127.0.0.1
            "/ip6/64:ff9b::c0a8:101/tcp/443",   // 192.168.1.1 -- Public only
            "/ip6/64:ff9b::/tcp/9001",          // 0.0.0.0
            "/ip6/64:ff9b::ffff:ffff/tcp/9001", // 255.255.255.255
        ] {
            assert!(
                !is_disclosable_multiaddr(addr),
                "{addr} unwraps to a non-routable IPv4 and was disclosable"
            );
        }
        for addr in [
            "/ip6/64:ff9b::a9fe:a9fe/tcp/80",
            "/ip6/64:ff9b::7f00:1/tcp/8080",
            "/ip6/64:ff9b::/tcp/9001",
            "/ip6/64:ff9b::ffff:ffff/tcp/9001",
        ] {
            assert!(
                !is_dialable_multiaddr(addr, LOCAL, REMOTE),
                "{addr} unwraps to a non-routable IPv4 and was dialable"
            );
        }
        // RFC1918 through NAT64 follows the same mode rule as bare RFC1918.
        assert!(is_dialable_multiaddr(
            "/ip6/64:ff9b::c0a8:101/tcp/443",
            LOCAL,
            REMOTE
        ));
        assert!(!is_dialable_multiaddr(
            "/ip6/64:ff9b::c0a8:101/tcp/443",
            PUBLIC,
            REMOTE
        ));
        // A genuinely routable IPv4 behind NAT64 must still work -- this is a
        // real connectivity path on IPv6-only carriers, not a thing to ban.
        // 0xcb00:7109 == 203.0.113.9
        assert!(is_dialable_multiaddr(
            "/ip6/64:ff9b::cb00:7109/tcp/443",
            LOCAL,
            REMOTE
        ));
        assert!(is_disclosable_multiaddr("/ip6/64:ff9b::cb00:7109/tcp/443"));
    }

    /// RFC 8215 local-use NAT64 prefix with the RFC 6052 /48 embedding: the
    /// IPv4 octets straddle the `u` byte, so a naive "last 32 bits" unwrap gets
    /// the wrong answer and lets the address through.
    #[test]
    fn nat64_local_use_prefix_uses_the_rfc6052_48_embedding() {
        // 64:ff9b:1:a9fe:0:a9fe::  ->  169.254 . 169.254
        //   octets[6..8] = a9 fe, octets[8] = u = 00, octets[9..11] = a9 fe
        assert!(!is_dialable_multiaddr(
            "/ip6/64:ff9b:1:a9fe:0:a9fe::/tcp/80",
            LOCAL,
            REMOTE
        ));
        assert!(!is_disclosable_multiaddr(
            "/ip6/64:ff9b:1:a9fe:0:a9fe::/tcp/80"
        ));
        // 64:ff9b:1:7f00:0:0100::  ->  127.0.0.1
        assert!(!is_dialable_multiaddr(
            "/ip6/64:ff9b:1:7f00:0:100::/tcp/8080",
            LOCAL,
            REMOTE
        ));
        // 64:ff9b:1:cb00:0:7109::  ->  203.0.113.9, a real destination.
        assert!(is_dialable_multiaddr(
            "/ip6/64:ff9b:1:cb00:0:7109::/tcp/443",
            LOCAL,
            REMOTE
        ));
    }

    /// 6to4 embeds the IPv4 in bits 16..48. `2002:a9fe:a9fe::` is the metadata
    /// endpoint again.
    #[test]
    fn sixtofour_prefix_cannot_launder_an_internal_ipv4() {
        assert!(!is_dialable_multiaddr(
            "/ip6/2002:a9fe:a9fe::/tcp/80",
            LOCAL,
            REMOTE
        ));
        assert!(!is_disclosable_multiaddr("/ip6/2002:a9fe:a9fe::/tcp/80"));
        assert!(!is_dialable_multiaddr(
            "/ip6/2002:7f00:1::/tcp/8080",
            LOCAL,
            REMOTE
        ));
        // 2002:c000:0201:: -> 192.0.2.1 (TEST-NET-1): not disclosable, and the
        // 6to4 wrapper must not change that.
        assert!(!is_disclosable_multiaddr("/ip6/2002:c000:201::/tcp/443"));
        // 2002:cb00:7109:: -> 203.0.113.9, routable.
        assert!(is_dialable_multiaddr(
            "/ip6/2002:cb00:7109::/tcp/443",
            LOCAL,
            REMOTE
        ));
    }

    /// Teredo carries the server IPv4 in bits 32..64 and the client's own IPv4
    /// in bits 96..128, bitwise-complemented. Both are real destinations, so
    /// both are checked.
    #[test]
    fn teredo_checks_both_the_server_and_the_obfuscated_client_ipv4() {
        // server 203.0.113.9, client ~(127.0.0.1) = 0x80ff:fffe
        assert!(!is_dialable_multiaddr(
            "/ip6/2001:0:cb00:7109:0:0:80ff:fffe/tcp/443",
            LOCAL,
            REMOTE
        ));
        // server 169.254.169.254 (metadata), client 203.0.113.9 -> ~ = 34ff:8ef6
        assert!(!is_dialable_multiaddr(
            "/ip6/2001:0:a9fe:a9fe:0:0:34ff:8ef6/tcp/80",
            LOCAL,
            REMOTE
        ));
        assert!(!is_disclosable_multiaddr(
            "/ip6/2001:0:a9fe:a9fe:0:0:34ff:8ef6/tcp/80"
        ));
        // Both halves routable -> allowed.
        assert!(is_dialable_multiaddr(
            "/ip6/2001:0:cb00:7109:0:0:34ff:8ef6/tcp/443",
            LOCAL,
            REMOTE
        ));
        // `2001::/16` outside `2001:0000::/32` is ordinary global unicast and
        // must NOT be reinterpreted as Teredo.
        assert!(is_dialable_multiaddr(
            "/ip6/2001:4860:4860::8888/tcp/443",
            LOCAL,
            REMOTE
        ));
        assert!(is_disclosable_multiaddr(
            "/ip6/2001:4860:4860::8888/tcp/443"
        ));
    }

    // ------------------------------------------------------------------
    // Round 4 -- disclosure needs "globally routable", not "!is_private()"
    // ------------------------------------------------------------------

    /// `Ipv4Addr::is_private()` is RFC1918 and nothing else. CGNAT is the one
    /// that matters: on a carrier-grade-NAT mobile network `100.64.x.y` is a
    /// live internal host, so disclosing it is the same class of leak as
    /// disclosing `192.168.x.y`, and dialing it is the same internal probe.
    #[test]
    fn disclosure_drops_cgnat_benchmark_reserved_and_test_net_1() {
        for addr in [
            "/ip4/100.64.0.1/tcp/9001",
            "/ip4/100.100.50.7/tcp/9001",
            "/ip4/100.127.255.254/tcp/9001",
            "/ip4/192.0.2.5/tcp/9001",
            "/ip4/198.18.0.1/tcp/9001",
            "/ip4/198.19.255.254/tcp/9001",
            "/ip4/240.0.0.1/tcp/9001",
            "/ip4/250.1.2.3/tcp/9001",
        ] {
            assert!(
                !is_disclosable_multiaddr(addr),
                "{addr} is not globally routable but was disclosable"
            );
        }
        // The addresses immediately outside each range must still be accepted,
        // so the masks are not accidentally too wide.
        for addr in [
            "/ip4/100.63.255.255/tcp/9001",
            "/ip4/100.128.0.1/tcp/9001",
            "/ip4/192.0.1.1/tcp/9001",
            "/ip4/192.0.3.1/tcp/9001",
            "/ip4/198.17.255.255/tcp/9001",
            "/ip4/198.20.0.1/tcp/9001",
            "/ip4/223.255.255.254/tcp/9001",
        ] {
            assert!(
                is_disclosable_multiaddr(addr),
                "{addr} is globally routable but was rejected -- the mask is too wide"
            );
        }
    }

    /// CGNAT stays DIALABLE -- a phone really can reach its CGNAT neighbours.
    /// The two predicates must diverge here, exactly as they do for RFC1918.
    #[test]
    fn cgnat_is_dialable_locally_but_never_disclosable() {
        assert!(is_dialable_multiaddr(
            "/ip4/100.64.0.1/tcp/9001",
            LOCAL,
            REMOTE
        ));
        assert!(!is_disclosable_multiaddr("/ip4/100.64.0.1/tcp/9001"));
    }

    #[test]
    fn acceptable_peer_address_combines_both_gates() {
        let my_addrs = vec!["/ip4/1.2.3.4/tcp/9001".to_string()];
        assert!(!is_acceptable_peer_address(
            "/ip4/1.2.3.4/tcp/9001",
            LOCAL,
            REMOTE,
            &my_addrs
        ));
        assert!(!is_acceptable_peer_address(
            "/ip4/127.0.0.1/tcp/9001",
            LOCAL,
            REMOTE,
            &my_addrs
        ));
        assert!(!is_acceptable_peer_address(
            "/dns4/evil.example/tcp/80",
            LOCAL,
            REMOTE,
            &my_addrs
        ));
        assert!(is_acceptable_peer_address(
            "/ip4/5.6.7.8/tcp/9001",
            LOCAL,
            REMOTE,
            &my_addrs
        ));
    }

    // ------------------------------------------------------------------
    // TrustedLocalProxy audience -- Wi-Fi Aware loopback proxy gating
    // ------------------------------------------------------------------

    #[test]
    fn wifi_aware_loopback_proxy_is_dialable_only_via_trusted_audience() {
        // The trusted audience accepts our own loopback proxy address.
        assert!(is_dialable_trusted_local_proxy_parsed(
            &"/ip4/127.0.0.1/tcp/9001".parse::<Multiaddr>().unwrap(),
            REMOTE
        ));
        // But the regular Local dial predicate still rejects it.
        assert!(!is_dialable_multiaddr(
            "/ip4/127.0.0.1/tcp/9001",
            LOCAL,
            REMOTE
        ));
    }

    #[test]
    fn trusted_proxy_audience_still_rejects_metadata_and_ipv6_loopback() {
        // Cloud metadata must stay rejected even through the trusted path.
        assert!(!is_dialable_trusted_local_proxy_parsed(
            &"/ip4/169.254.169.254/tcp/80".parse::<Multiaddr>().unwrap(),
            REMOTE
        ));
        // IPv6 loopback stays rejected unconditionally (early-return).
        assert!(!is_dialable_trusted_local_proxy_parsed(
            &"/ip6/::1/tcp/9001".parse::<Multiaddr>().unwrap(),
            REMOTE
        ));
    }

    #[test]
    fn rfc1918_disclosable_when_local_network_matches() {
        let local_addrs = vec!["/ip4/192.168.1.50/tcp/9001".to_string()];
        assert!(
            is_disclosable_on_rfc1918_network(
                "/ip4/192.168.1.100/tcp/9001",
                Some("/ip4/192.168.1.20/tcp/9001"),
                &local_addrs,
            ),
            "same /24 should be disclosable"
        );
    }

    #[test]
    fn rfc1918_not_disclosable_when_different_private_class() {
        let local_addrs = vec!["/ip4/10.0.0.5/tcp/9001".to_string()];
        assert!(
            !is_disclosable_on_rfc1918_network(
                "/ip4/192.168.1.100/tcp/9001",
                Some("/ip4/192.168.1.20/tcp/9001"),
                &local_addrs,
            ),
            "different RFC1918 class should NOT be disclosable"
        );
    }

    #[test]
    fn public_ipv4_never_disclosable_on_network() {
        let local_addrs = vec!["/ip4/192.168.1.50/tcp/9001".to_string()];
        assert!(
            !is_disclosable_on_rfc1918_network(
                "/ip4/203.0.113.45/tcp/9001",
                Some("/ip4/192.168.1.20/tcp/9001"),
                &local_addrs,
            ),
            "public IPv4 must never be disclosed via network check"
        );
    }

    #[test]
    fn rfc1918_disclosure_requires_observed_same_subnet_requester() {
        let local_addrs = vec!["/ip4/10.0.0.5/tcp/9001".to_string()];
        assert!(!is_disclosable_on_rfc1918_network(
            "/ip4/10.99.0.42/tcp/9001",
            Some("/ip4/10.0.0.20/tcp/9001"),
            &local_addrs,
        ));
        assert!(!is_disclosable_on_rfc1918_network(
            "/ip4/10.0.0.42/tcp/9001",
            Some("/ip4/198.51.100.20/tcp/9001"),
            &local_addrs,
        ));
        assert!(!is_disclosable_on_rfc1918_network(
            "/ip4/10.0.0.42/tcp/9001",
            None,
            &local_addrs,
        ));
    }

    #[test]
    fn rfc1918_disclosure_rejects_cgnat_requester_same_subnet_collision() {
        // Carrier-grade NAT /24s can contain unrelated subscribers. A shared
        // 100.64/24 is therefore not evidence that the requester is our LAN
        // neighbour, even when the entry and listener are in that /24.
        let local_addrs = vec!["/ip4/100.64.0.10/tcp/9001".to_string()];
        assert!(!is_disclosable_on_rfc1918_network(
            "/ip4/100.64.0.12/tcp/9001",
            Some("/ip4/100.64.0.11/tcp/9001"),
            &local_addrs,
        ));
    }

    #[test]
    fn rfc1918_disclosure_rejects_nat64_embedded_cgnat_adjacency() {
        // The same shared-space rule applies when CGNAT is embedded in an
        // IPv6 NAT64 address; neither side may authorize private disclosure.
        let local_addrs = vec!["/ip6/64:ff9b::6440:1a/tcp/9001".to_string()];
        assert!(!is_disclosable_on_rfc1918_network(
            "/ip6/64:ff9b::6440:1c/tcp/9001",
            Some("/ip6/64:ff9b::6440:1b/tcp/9001"),
            &local_addrs,
        ));
    }

    #[test]
    fn rfc1918_disclosure_requires_concrete_local_listener_evidence() {
        // An observed requester and matching entry are insufficient while the
        // listener list is empty (startup, wildcard-only, or DNS-only state).
        assert!(!is_disclosable_on_rfc1918_network(
            "/ip4/192.168.1.100/tcp/9001",
            Some("/ip4/192.168.1.20/tcp/9001"),
            &[],
        ));
    }

    #[test]
    fn rfc1918_disclosure_rejects_relay_loopback_and_link_local() {
        let local_addrs = vec!["/ip4/192.168.1.5/tcp/9001".to_string()];
        for entry in [
            "/ip4/127.0.0.1/tcp/9001",
            "/ip4/169.254.1.2/tcp/9001",
            "/p2p-circuit/p2p/12D3KooWJ7t4Qz4VwzYbP4uU6z4aGz2gZ4nM6mX2sJ6j7V8b9c1d",
        ] {
            assert!(
                !is_disclosable_on_rfc1918_network(
                    entry,
                    Some("/ip4/192.168.1.20/tcp/9001"),
                    &local_addrs,
                ),
                "unsafe entry disclosed: {entry}"
            );
        }
        assert!(!is_disclosable_on_rfc1918_network(
            "/ip4/192.168.1.42/tcp/9001",
            Some("/ip4/127.0.0.1/tcp/9001"),
            &local_addrs,
        ));
        assert!(!is_disclosable_on_rfc1918_network(
            "/ip4/192.168.1.42/tcp/9001",
            Some("/ip4/192.168.1.50/tcp/443/p2p-circuit"),
            &local_addrs,
        ));
        assert!(!is_disclosable_on_rfc1918_network(
            "/ip4/192.168.1.42/tcp/9001",
            Some("/p2p-circuit"),
            &local_addrs,
        ));
        assert!(!is_disclosable_on_rfc1918_network(
            "/ip4/192.168.1.42/tcp/9001",
            Some("not-a-multiaddr"),
            &local_addrs,
        ));
    }

    // ------------------------------------------------------------------
    // P0 mesh growth self-circuit guard (2026-08-22)
    // ------------------------------------------------------------------

    #[test]
    fn rejects_self_destination_single_hop_circuit() {
        let self_peer = PeerId::random();
        let relay_peer = PeerId::random();

        // Single-hop circuit with explicit relay peer ID ending in self.
        let self_circuit_with_relay_id = format!(
            "/ip4/198.51.100.1/tcp/443/p2p/{relay_peer}/p2p-circuit/p2p/{self_peer}"
        );
        assert!(
            is_self_circuit(&self_circuit_with_relay_id, &self_peer),
            "single-hop circuit ending in self must be rejected"
        );

        // Single-hop circuit without relay peer ID ending in self.
        let self_circuit_no_relay_id =
            format!("/ip4/198.51.100.1/tcp/443/p2p-circuit/p2p/{self_peer}");
        assert!(
            is_self_circuit(&self_circuit_no_relay_id, &self_peer),
            "single-hop circuit ending in self (no relay pid) must be rejected"
        );

        // Single-hop circuit where relay hop is self.
        let self_as_relay = format!(
            "/ip4/198.51.100.1/tcp/443/p2p/{self_peer}/p2p-circuit/p2p/{relay_peer}"
        );
        assert!(
            is_self_circuit(&self_as_relay, &self_peer),
            "single-hop circuit with self as relay must be rejected"
        );
    }

    #[test]
    fn rejects_two_hop_self_referential_circuits() {
        let self_peer = PeerId::random();
        let peer = PeerId::random();
        let mac_peer = PeerId::random();

        // Exact shape 1 from P0 ticket:
        // /ip6/::1/tcp/8080/p2p/<SELF>/p2p-circuit/p2p/<PEER>/p2p-circuit/p2p/<SELF>
        let shape1 = format!(
            "/ip6/::1/tcp/8080/p2p/{self_peer}/p2p-circuit/p2p/{peer}/p2p-circuit/p2p/{self_peer}"
        );
        assert!(
            is_self_circuit(&shape1, &self_peer),
            "two-hop self-referential circuit (self -> peer -> self) must be rejected"
        );

        // Exact shape 2 from P0 ticket:
        // /ip6/::1/tcp/9001/p2p/<MAC>/p2p-circuit/p2p/<PEER>/p2p-circuit/p2p/<SELF>
        let shape2 = format!(
            "/ip6/::1/tcp/9001/p2p/{mac_peer}/p2p-circuit/p2p/{peer}/p2p-circuit/p2p/{self_peer}"
        );
        assert!(
            is_self_circuit(&shape2, &self_peer),
            "two-hop circuit ending in self (mac -> peer -> self) must be rejected"
        );

        // Shape with self as intermediate hop:
        // /ip4/198.51.100.1/tcp/443/p2p/<MAC>/p2p-circuit/p2p/<SELF>/p2p-circuit/p2p/<PEER>
        let intermediate_self = format!(
            "/ip4/198.51.100.1/tcp/443/p2p/{mac_peer}/p2p-circuit/p2p/{self_peer}/p2p-circuit/p2p/{peer}"
        );
        assert!(
            is_self_circuit(&intermediate_self, &self_peer),
            "two-hop circuit routing through self as intermediate hop must be rejected"
        );
    }

    #[test]
    fn accepts_legitimate_circuit_to_different_peer() {
        let self_peer = PeerId::random();
        let relay_peer = PeerId::random();
        let target_peer = PeerId::random();

        let legitimate_circuit = format!(
            "/ip4/198.51.100.1/tcp/443/p2p/{relay_peer}/p2p-circuit/p2p/{target_peer}"
        );
        assert!(
            !is_self_circuit(&legitimate_circuit, &self_peer),
            "legitimate circuit to a different peer must be accepted"
        );

        let parsed: Multiaddr = legitimate_circuit
            .parse()
            .expect("legitimate circuit multiaddr must parse");
        assert!(
            !is_self_circuit_multiaddr_parsed(&parsed, &self_peer),
            "parsed legitimate circuit to a different peer must be accepted"
        );
    }

    #[test]
    fn plain_non_circuit_address_is_unaffected() {
        let self_peer = PeerId::random();
        let other_peer = PeerId::random();

        // Plain IP:port without peer ID.
        assert!(
            !is_self_circuit("/ip4/198.51.100.1/tcp/9001", &self_peer),
            "plain IP:port address must be unaffected"
        );

        // Plain IP:port with other peer ID.
        let plain_with_peer = format!("/ip4/198.51.100.1/tcp/9001/p2p/{other_peer}");
        assert!(
            !is_self_circuit(&plain_with_peer, &self_peer),
            "plain address with other peer ID must be unaffected"
        );

        // Non-circuit address ending with self peer ID.
        let plain_with_self = format!("/ip4/198.51.100.1/tcp/9001/p2p/{self_peer}");
        assert!(
            is_self_circuit(&plain_with_self, &self_peer),
            "address whose final destination is self must be rejected"
        );

        // Invalid multiaddr string.
        assert!(
            !is_self_circuit("not-a-multiaddr", &self_peer),
            "unparseable address must return false from is_self_circuit"
        );
    }
}
