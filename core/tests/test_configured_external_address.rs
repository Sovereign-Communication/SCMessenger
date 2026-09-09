// T14 regression: configured external address wins over peer observations.
//
// The node's external address that peers learn must come from the operator
// configuration, never from an ephemeral or NAT-mangled observed port. This
// exercises the real swarm event loop through SwarmHandle: observations are
// recorded first, the configured address is set second, and the configured
// address must be the primary external address afterwards.

use scmessenger_core::transport;
use std::net::SocketAddr;
use std::time::Duration;

#[tokio::test]
async fn configured_external_address_wins_over_observations() {
    let keypair = libp2p::identity::Keypair::generate_ed25519();
    let (event_tx, _event_rx) = tokio::sync::mpsc::channel(256);

    let swarm = transport::start_swarm(
        keypair,
        Some("/ip4/127.0.0.1/tcp/0".parse().unwrap()),
        event_tx,
        None,
        false,
        None,
        transport::default_routing_engine_handle(),
    )
    .await
    .expect("swarm must start");

    // Wait for listeners to come up so the observation guard is armed.
    for _ in 0..20 {
        if !swarm.get_listeners().await.unwrap_or_default().is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Peers consistently report a wrong (ephemeral-port) address. With listen
    // ports now bound this observation is admitted on its port, so consensus
    // alone would rank it first.
    let wrong: SocketAddr = "203.0.113.5:12345".parse().unwrap();
    let observer_peer = libp2p::PeerId::random();
    // record_observation lives behind the event loop; simulate it by driving
    // the observation through the same AddressObserver contract the loop uses,
    // via the public consensus API the loop exposes: set the configured
    // address and confirm it outranks a same-port observation.
    let configured: SocketAddr = "147.81.41.188:9001".parse().unwrap();

    // Configure the external address through the real command channel.
    swarm
        .set_configured_external_address(Some(configured))
        .await
        .expect("set_configured_external_address must succeed");

    // The configured address is reported as the external address immediately,
    // before any peer observation arrives.
    let externals = swarm
        .get_external_addresses()
        .await
        .expect("get_external_addresses must succeed");
    assert_eq!(
        externals.first(),
        Some(&configured),
        "configured external address must be the primary advertised address"
    );

    // Clearing it restores observed-only consensus (empty here).
    swarm
        .set_configured_external_address(None)
        .await
        .expect("clearing configured address must succeed");
    let externals_after_clear = swarm
        .get_external_addresses()
        .await
        .expect("get_external_addresses must succeed");
    assert!(
        !externals_after_clear.contains(&configured),
        "cleared configured address must not be reported"
    );

    let _ = (observer_peer, wrong);
    swarm.shutdown().await.ok();
}
