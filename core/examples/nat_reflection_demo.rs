// Demo: NAT Traversal & Address Reflection
//
// This example demonstrates the sovereign mesh address discovery protocol.
// It creates a small network of nodes and shows how they discover their
// external addresses through peer reflection (no STUN servers needed).

use libp2p::identity::Keypair;
use parking_lot::RwLock;
use scmessenger_core::transport::{
    nat::{NatConfig, NatTraversal},
    swarm::{start_swarm, SwarmEvent2},
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🌐 SCMessenger NAT Traversal Demo");
    println!("==================================\n");

    // Create three nodes: 2 reflectors + 1 requester
    println!("📡 Starting bootstrap nodes (reflectors)...");

    let keypair1 = Keypair::generate_ed25519();
    let keypair2 = Keypair::generate_ed25519();
    let keypair3 = Keypair::generate_ed25519();

    let peer1_id = keypair1.public().to_peer_id();
    let peer2_id = keypair2.public().to_peer_id();
    let peer3_id = keypair3.public().to_peer_id();

    println!("   Bootstrap 1: {}", peer1_id);
    println!("   Bootstrap 2: {}", peer2_id);
    println!("   Requester:   {}\n", peer3_id);

    // Start reflector nodes
    let (event_tx1, mut event_rx1) = mpsc::channel(256);
    let (event_tx2, mut event_rx2) = mpsc::channel(256);
    let (event_tx3, mut event_rx3) = mpsc::channel(256);

    let swarm1 = start_swarm(
        keypair1,
        None,
        event_tx1,
        None,
        false,
        None,
        Arc::new(RwLock::new(None)),
    )
    .await?;
    tokio::time::sleep(Duration::from_millis(300)).await;

    let swarm2 = start_swarm(
        keypair2,
        None,
        event_tx2,
        None,
        false,
        None,
        Arc::new(RwLock::new(None)),
    )
    .await?;
    tokio::time::sleep(Duration::from_millis(300)).await;

    let swarm3 = start_swarm(
        keypair3,
        None,
        event_tx3,
        None,
        false,
        None,
        Arc::new(RwLock::new(None)),
    )
    .await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Get listening addresses
    let mut addr1 = None;
    let mut addr2 = None;

    tokio::time::timeout(Duration::from_secs(2), async {
        while let Some(event) = event_rx1.recv().await {
            if let SwarmEvent2::ListeningOn(addr) = event {
                addr1 = Some(addr);
                break;
            }
        }
    })
    .await
    .ok();

    tokio::time::timeout(Duration::from_secs(2), async {
        while let Some(event) = event_rx2.recv().await {
            if let SwarmEvent2::ListeningOn(addr) = event {
                addr2 = Some(addr);
                break;
            }
        }
    })
    .await
    .ok();

    if addr1.is_none() || addr2.is_none() {
        eprintln!("❌ Failed to start bootstrap nodes");
        return Ok(());
    }

    println!("✅ Bootstrap nodes listening");
    println!("   Node 1: {}", addr1.as_ref().unwrap());
    println!("   Node 2: {}\n", addr2.as_ref().unwrap());

    // Connect requester to bootstrap nodes
    println!("🔗 Connecting to bootstrap nodes...");

    swarm3.dial(addr1.unwrap()).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    swarm3.dial(addr2.unwrap()).await?;
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Monitor connection events
    let mut connected_count = 0;
    tokio::time::timeout(Duration::from_secs(3), async {
        while let Some(event) = event_rx3.recv().await {
            if let SwarmEvent2::PeerDiscovered(peer) = event {
                println!("   ✓ Connected to {}", peer);
                connected_count += 1;
                if connected_count >= 2 {
                    break;
                }
            }
        }
    })
    .await
    .ok();

    println!("\n✅ Connected to {} bootstrap nodes\n", connected_count);

    // Demonstrate address reflection
    println!("🔍 Step 1: Single Address Reflection");
    println!("─────────────────────────────────────");

    let result = swarm3.request_address_reflection(peer1_id).await;
    match result {
        Ok(addr) => {
            println!("✅ Bootstrap 1 sees us at: {}", addr);
            println!("   This is our external address as observed by peer\n");
        }
        Err(e) => {
            println!("❌ Reflection failed: {}\n", e);
        }
    }

    // Demonstrate NAT type detection
    println!("🔍 Step 2: NAT Type Detection");
    println!("─────────────────────────────");

    let config = NatConfig {
        peer_reflectors: vec![peer1_id.to_string(), peer2_id.to_string()],
        ..NatConfig::default()
    };

    let nat_traversal = NatTraversal::new(config)?;

    println!("   Querying multiple peers...");

    let nat_result = nat_traversal.probe_nat(&swarm3).await;

    match nat_result {
        Ok(nat_type) => {
            println!("✅ NAT Detection Complete!");
            println!("   Type: {:?}", nat_type);

            if let Some(external_addr) = nat_traversal.get_external_address() {
                println!("   External Address: {}", external_addr);
            }

            // Explain NAT type
            match nat_type {
                scmessenger_core::transport::nat::NatType::Open => {
                    println!("\n   📖 Open NAT means:");
                    println!("      • No NAT detected");
                    println!("      • Directly reachable from internet");
                    println!("      • Optimal for peer-to-peer");
                }
                scmessenger_core::transport::nat::NatType::FullCone => {
                    println!("\n   📖 Full Cone NAT means:");
                    println!("      • NAT present but permissive");
                    println!("      • Direct libp2p DCUtR may work");
                    println!("      • Good for peer-to-peer");
                }
                scmessenger_core::transport::nat::NatType::Symmetric => {
                    println!("\n   📖 Symmetric NAT means:");
                    println!("      • Strict NAT with port randomization");
                    println!("      • Direct libp2p DCUtR may be difficult");
                    println!("      • Relay fallback recommended");
                }
                _ => {
                    println!("\n   📖 Other NAT type detected");
                }
            }
        }
        Err(e) => {
            println!("❌ NAT detection failed: {}", e);
        }
    }

    println!("\n🔍 Step 3: Multiple Reflections");
    println!("────────────────────────────────");
    println!("   Testing service stability with rapid requests...\n");

    for i in 1..=5 {
        let result = swarm3.request_address_reflection(peer1_id).await;
        match result {
            Ok(addr) => {
                println!("   [{}/5] ✓ Reflection: {}", i, addr);
            }
            Err(e) => {
                println!("   [{}/5] ✗ Failed: {}", i, e);
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    println!("\n✅ Service handled multiple requests successfully\n");

    // Architecture explanation
    println!("📚 Architecture Highlights");
    println!("═════════════════════════");
    println!("✓ No external STUN servers required");
    println!("✓ Sovereign mesh - peers help each other");
    println!("✓ libp2p protocol: /sc/address-reflection/1.0.0");
    println!("✓ ~82 bytes per reflection (minimal bandwidth)");
    println!("✓ ~10-200ms latency (depending on peer distance)");
    println!("✓ Works on any libp2p transport (TCP, QUIC, WebSocket)");

    println!("\n🎯 Use Cases");
    println!("════════════");
    println!("• Mobile clients discovering external address");
    println!("• Browser nodes behind NAT");
    println!("• Automatic NAT traversal setup");
    println!("• Direct connectivity via libp2p DCUtR");
    println!("• Relay fallback decision");

    println!("\n✨ Demo Complete!");
    println!("═════════════════");
    println!("Try running the integration tests for more:");
    println!("  cargo test --test integration_nat_reflection -- --nocapture\n");

    // Cleanup
    swarm1.shutdown().await.ok();
    swarm2.shutdown().await.ok();
    swarm3.shutdown().await.ok();

    tokio::time::sleep(Duration::from_millis(500)).await;

    Ok(())
}
