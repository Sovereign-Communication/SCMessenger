// Transport module — libp2p swarm and networking

pub mod abstraction;
pub mod addr_filter;
pub mod behaviour;
pub mod ble;
pub mod bootstrap;
pub mod capability;
pub mod circuit_breaker;
pub mod diagnostics;
pub mod dial_policy;
pub mod discovery;
pub mod escalation;
pub mod health;
pub mod internet;
pub mod manager;
pub mod mesh_routing;
pub mod multiport;
pub mod nat;
pub mod observation;
pub mod peer_broadcast;
pub mod reflection;
pub mod relay_health;
pub mod reputation;
pub mod routing;
pub mod swarm;
#[cfg(not(target_arch = "wasm32"))]
pub mod websocket;
pub mod wifi_aware;
pub mod wifi_direct;

pub use crate::store::ledger_entry::{
    LedgerExchangeRequest, LedgerExchangeResponse, SharedPeerEntry,
};
pub use addr_filter::{
    is_acceptable_peer_address, is_dialable_multiaddr, is_dialable_multiaddr_parsed,
    is_dialable_trusted_local_proxy_parsed, is_disclosable_multiaddr,
    is_disclosable_multiaddr_parsed, is_self_address, strip_peer_id, strip_peer_id_multiaddr,
    DnsPolicy, NetworkMode,
};
pub use behaviour::{
    DeregistrationPayload, DeregistrationRequest, IronCoreBehaviour, Libp2pMessageRequest,
    Libp2pMessageResponse, RegistrationMessage, RegistrationPayload, RegistrationRequest,
    RegistrationResponse, RelayRequest, RelayResponse,
};
pub use bootstrap::{BootstrapConfig, BootstrapManager, BootstrapState};
pub use capability::can_forward_for_wasm;
pub use circuit_breaker::{
    CircuitBreakerConfig, CircuitBreakerManager, CircuitBreakerStats, CircuitState,
};
pub use diagnostics::{
    get_network_diagnostics_report, NetworkDiagnosticsReport, PeerConnectionSummary,
};
pub use dial_policy::{
    multiaddr_to_key, CircuitRelayLadder, DialPolicyManager, PerPeerBackoffState,
};
pub use discovery::{DiscoveryConfig, DiscoveryMode};
pub use health::{
    ConnectionState, ConnectionStats, GlobalTransportMetrics, TransportHealthMonitor,
};
pub use mesh_routing::{
    BootstrapCapability, DeliveryAttempt, MultiPathDelivery, RelayReputation, RelayStats,
    ReputationTracker, RetryStrategy, ROUTE_REASON_DIRECT_FIRST,
    ROUTE_REASON_RELAY_RECENCY_SUCCESS, ROUTE_REASON_RELAY_SUCCESS_SCORE,
    ROUTE_REASON_RELAY_TIEBREAK_LAST_SUCCESS, ROUTE_REASON_RELAY_TIEBREAK_PEER_ID,
};
pub use multiport::{BindAnalysis, BindResult, ConnectivityStatus, MultiPortConfig};
pub use observation::{AddressObservation, AddressObserver, ConnectionEndpoint, ConnectionTracker};
pub use peer_broadcast::PeerBroadcaster;
pub use reflection::{
    AddressReflectionRequest, AddressReflectionResponse, AddressReflectionService,
};
pub use relay_health::{RelayDiscovery, RelayFallback, RelayMetrics};
pub use reputation::{AbuseReputationManager, AbuseSignal, PeerAbuseStats, ReputationScore};
pub use routing::EnhancedReputationScore;
pub use routing::{
    adaptive_ttl::{ActivityHistory, AdaptiveTTLManager},
    engine::{
        NextHop, RoutingDecision, RoutingEngine, RoutingLayer, RoutingMaintenance, RoutingSummary,
    },
    global::{GlobalRoutes, RouteAdvertisement, RouteRequest},
    local::{CellSummary, LocalCell, PeerEvent, PeerId, PeerInfo, PeerStatus, TransportType},
    negative_cache::{NegativeCache, NegativeCacheStats},
    neighborhood::{GatewayInfo, NeighborhoodGossip, NeighborhoodSummary, NeighborhoodTable},
    optimized_engine::{OptimizedRoutingEngine, OptimizedRoutingMaintenance},
    resume_prefetch::{
        FrequentPeer, PrefetchConfig, PrefetchStats, PrefetchStatus, PrefetchedRoute,
    },
    smart_retry::{calculate_next_attempt, BackoffStrategy, DeliveryTrigger},
    timeout_budget::{BudgetSummary, DiscoveryPhase, TimeoutBudget},
};
pub use swarm::{
    default_routing_engine_handle, start_swarm, start_swarm_with_config, SwarmCommand,
    SwarmEvent2 as SwarmEvent, SwarmHandle,
};
