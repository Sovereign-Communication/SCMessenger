// Induced connection fan-out client (test harness).
//
// Purpose: reproduce, on demand, the failure mode that locked the user's phone
// out of the relay — one peer identity dialing the same remote peer over many
// distinct paths at once (the relay announces a port ladder: tcp/80, /443,
// /8080, /9001, /9002/ws, /9090, plus quic). Under a single-tier per-peer cap
// (`max_established_per_peer = 4`) every dial past the fourth is refused with
// `connection_limits: limit 4 reached`; with the two-tier policy in
// `scmessenger_core::transport::per_peer_cap` the extra paths are admitted and
// then trimmed server-side, so the peer is never locked out.
//
// Usage (from a host that can reach the relay):
//   conn-fanout --peer 12D3KooW... --addr /ip4/<ip>/tcp/9001 \
//               --addr /ip4/<ip>/tcp/8080 ... --hold 25 --rounds 2
//
// The final `FANOUT-SUMMARY` line is machine-parseable. `--seed-hex` pins the
// client identity so repeated runs are comparable (the server-side cap is
// keyed by peer id).

use clap::Parser;
use futures::StreamExt;
use libp2p::core::upgrade::Version;
use libp2p::multiaddr::Protocol;
use libp2p::swarm::dummy;
use libp2p::swarm::dial_opts::DialOpts;
use libp2p::swarm::{Config as SwarmConfig, Swarm, SwarmEvent};
use libp2p::{identity, noise, tcp, yamux, Multiaddr, PeerId, Transport as _};
use std::collections::HashSet;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(name = "conn-fanout")]
#[command(about = "Induced multi-connection fan-out client (per-peer cap test)")]
struct Args {
    /// Remote peer id (base58) to dial.
    #[arg(long)]
    peer: String,

    /// Dial target; repeat for each distinct path (port ladder / transport).
    #[arg(long = "addr", required = true)]
    addrs: Vec<String>,

    /// Seconds to hold the connections open after dialing.
    #[arg(long, default_value = "25")]
    hold: u64,

    /// Milliseconds between dial starts (0 = concurrent burst).
    #[arg(long, default_value = "0")]
    stagger_ms: u64,

    /// Repeat the whole burst this many times (determinism check).
    #[arg(long, default_value = "1")]
    rounds: u32,

    /// Seconds to wait between rounds.
    #[arg(long, default_value = "5")]
    round_gap: u64,

    /// Fixed 32-byte ed25519 secret in hex, so repeated runs share one identity.
    #[arg(long, default_value = "")]
    seed_hex: String,
}

fn ts() -> String {
    chrono::Utc::now().format("%H:%M:%S%.3f").to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let key = if args.seed_hex.is_empty() {
        identity::Keypair::generate_ed25519()
    } else {
        let bytes = (0..args.seed_hex.len() / 2)
            .map(|i| u8::from_str_radix(&args.seed_hex[i * 2..i * 2 + 2], 16))
            .collect::<Result<Vec<u8>, _>>()?;
        identity::Keypair::ed25519_from_bytes(bytes)?
    };
    let local = key.public().to_peer_id();
    let relay: PeerId = args.peer.parse()?;
    println!(
        "[{}] FANOUT client peer={} relay={} addrs={} rounds={} stagger={}ms hold={}s",
        ts(),
        local,
        relay,
        args.addrs.len(),
        args.rounds,
        args.stagger_ms,
        args.hold
    );

    let transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
        .upgrade(Version::V1)
        .authenticate(noise::Config::new(&key)?)
        .multiplex(yamux::Config::default())
        .boxed();

    let mut swarm = Swarm::new(
        transport,
        dummy::Behaviour,
        local,
        SwarmConfig::with_tokio_executor(),
    );

    // Normalise every target into a /p2p/<relay>-suffixed multiaddr.
    let mut targets: Vec<Multiaddr> = Vec::new();
    for raw in &args.addrs {
        let mut ma: Multiaddr = raw.parse()?;
        if !ma.iter().any(|p| matches!(p, Protocol::P2p(_))) {
            ma.push(Protocol::P2p(relay));
        }
        targets.push(ma);
    }

    let mut total_dials = 0u32;
    let mut established = 0u32;
    let mut failures = 0u32;
    let mut closed_by_remote = 0u32;
    let mut live: HashSet<libp2p::swarm::ConnectionId> = HashSet::new();
    let mut peak_live = 0usize;

    for round in 0..args.rounds {
        live.clear();
        established = 0;
        failures = 0;
        closed_by_remote = 0;
        let round_start = Instant::now();
        println!("[{}] --- round {} ---", ts(), round + 1);

        for ma in &targets {
            let opts = DialOpts::unknown_peer_id().address(ma.clone()).build();
            match swarm.dial(opts) {
                Ok(()) => {
                    total_dials += 1;
                    println!("[{}] DIAL {}", ts(), ma);
                }
                Err(e) => println!("[{}] DIAL-REJECTED {} error={}", ts(), ma, e),
            }
            if args.stagger_ms > 0 {
                tokio::time::sleep(Duration::from_millis(args.stagger_ms)).await;
            }
        }

        let deadline = round_start + Duration::from_secs(args.hold);
        loop {
            let now = Instant::now();
            if now >= deadline {
                break;
            }
            match tokio::time::timeout(deadline - now, swarm.select_next_some()).await {
                Ok(SwarmEvent::ConnectionEstablished {
                    peer_id,
                    connection_id,
                    endpoint,
                    ..
                }) => {
                    established += 1;
                    live.insert(connection_id);
                    peak_live = peak_live.max(live.len());
                    println!(
                        "[{}] ESTABLISHED peer={} conn={} endpoint={:?} live={}",
                        ts(),
                        peer_id,
                        connection_id,
                        endpoint,
                        live.len()
                    );
                }
                Ok(SwarmEvent::ConnectionClosed {
                    peer_id,
                    connection_id,
                    num_established,
                    cause,
                    ..
                }) => {
                    live.remove(&connection_id);
                    closed_by_remote += 1;
                    println!(
                        "[{}] CLOSED peer={} conn={} num_established={} cause={:?} live={}",
                        ts(),
                        peer_id,
                        connection_id,
                        num_established,
                        cause,
                        live.len()
                    );
                }
                Ok(SwarmEvent::OutgoingConnectionError { peer_id, error, .. }) => {
                    failures += 1;
                    println!(
                        "[{}] DIAL-FAILED peer={:?} error={}",
                        ts(),
                        peer_id,
                        error
                    );
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }

        println!(
            "[{}] ROUND-SUMMARY round={} dials={} established={} dial_failures={} closed_by_peer={} live_at_end={} peak_live={}",
            ts(),
            round + 1,
            targets.len(),
            established,
            failures,
            closed_by_remote,
            live.len(),
            peak_live
        );

        if round + 1 < args.rounds {
            tokio::time::sleep(Duration::from_secs(args.round_gap)).await;
        }
    }

    println!(
        "FANOUT-SUMMARY peer={} paths={} total_dials={} peak_live={}",
        local,
        targets.len(),
        total_dials,
        peak_live
    );
    Ok(())
}
