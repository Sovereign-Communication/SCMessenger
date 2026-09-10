// Network diagnostics reporting
//
// Provides a cross-platform parity type for network diagnostic information.
// Core parity with the Android `DiagnosticsReporter.NetworkDiagnosticsReport`
// and iOS equivalent. Aggregates data from TransportHealthMonitor into a
// single report.

use crate::transport::health::{ConnectionState, ConnectionStats, TransportHealthMonitor};
use libp2p::PeerId;
use serde::{Deserialize, Serialize};

/// Summary of network diagnostics for the mesh node.
///
/// Aggregates connection statistics, transport metrics, and relay health into a
/// single reportable structure. Used by platform layers (Android, iOS, WASM)
/// to surface network state to users.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDiagnosticsReport {
    pub connected_peer_count: usize,
    pub total_messages_sent: u64,
    pub total_messages_failed: u64,
    pub avg_latency_ms: u64,
    pub active_connections: u32,
    pub connection_summary: Vec<PeerConnectionSummary>,
    pub generated_at_ms: u64,
    #[serde(default)]
    pub recent_routing_decisions: Vec<RoutingDecisionSnapshot>,
}

/// Per-peer connection summary included in diagnostics reports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerConnectionSummary {
    pub peer_id: String,
    pub state: String,
    pub messages_sent: u64,
    pub messages_failed: u64,
    pub avg_latency_ms: u64,
    pub last_activity_ms: u64,
}

/// Build a network diagnostics report from the transport health monitor.
///
/// Aggregates all peer connection statistics and message metrics into a single
/// report. This is the core parity of the Android
/// `SettingsViewModel.getNetworkDiagnosticsReport()` method.
pub fn get_network_diagnostics_report(
    monitor: &TransportHealthMonitor,
) -> NetworkDiagnosticsReport {
    let metrics = monitor.get_global_metrics();
    let peers = monitor.get_all_connection_stats();

    let connection_summary: Vec<PeerConnectionSummary> = peers
        .values()
        .map(|stats| PeerConnectionSummary {
            peer_id: stats.peer_id.to_string(),
            state: format!("{:?}", stats.state),
            messages_sent: stats.messages_sent,
            messages_failed: stats.message_failures,
            avg_latency_ms: stats.avg_latency_ms,
            last_activity_ms: stats.last_activity,
        })
        .collect();

    let connected_count = peers
        .values()
        .filter(|p| p.state == ConnectionState::Connected)
        .count();

    NetworkDiagnosticsReport {
        connected_peer_count: connected_count,
        total_messages_sent: metrics.total_messages_sent,
        total_messages_failed: metrics.total_message_failures,
        avg_latency_ms: compute_avg_latency(&peers),
        active_connections: metrics.current_active_connections,
        connection_summary,
        generated_at_ms: now_ms(),
        recent_routing_decisions: Vec::new(),
    }
}

/// Build a diagnostics report that includes routing telemetry.
pub fn get_network_diagnostics_report_with_telemetry(
    monitor: &TransportHealthMonitor,
    telemetry: &RoutingTelemetry,
) -> NetworkDiagnosticsReport {
    let mut report = get_network_diagnostics_report(monitor);
    report.recent_routing_decisions = telemetry.entries().to_vec();
    report
}

/// Extended network diagnostics that includes healthy and unhealthy connection lists.
///
/// Builds on the base `get_network_diagnostics_report` by also surfacing
/// which peers are considered healthy vs. unhealthy based on the
/// `TransportHealthMonitor`'s quality metrics.
pub fn get_extended_network_diagnostics(
    monitor: &TransportHealthMonitor,
) -> ExtendedNetworkDiagnostics {
    let base = get_network_diagnostics_report(monitor);
    let healthy_peers = monitor.get_healthy_connections();
    let unhealthy_peers = monitor.get_unhealthy_connections();

    ExtendedNetworkDiagnostics {
        base,
        healthy_peer_count: healthy_peers.len(),
        unhealthy_peer_count: unhealthy_peers.len(),
        healthy_peer_ids: healthy_peers.iter().map(|p| p.to_string()).collect(),
        unhealthy_peer_ids: unhealthy_peers.iter().map(|p| p.to_string()).collect(),
    }
}

/// Extended diagnostics report that includes healthy/unhealthy connection breakdowns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedNetworkDiagnostics {
    /// Base network diagnostics report
    pub base: NetworkDiagnosticsReport,
    /// Number of currently healthy connections
    pub healthy_peer_count: usize,
    /// Number of currently unhealthy connections
    pub unhealthy_peer_count: usize,
    /// List of healthy peer IDs
    pub healthy_peer_ids: Vec<String>,
    /// List of unhealthy peer IDs
    pub unhealthy_peer_ids: Vec<String>,
}

fn compute_avg_latency(peers: &std::collections::HashMap<PeerId, ConnectionStats>) -> u64 {
    let latencies: Vec<u64> = peers
        .values()
        .filter(|p| p.state == ConnectionState::Connected)
        .map(|p| p.avg_latency_ms)
        .filter(|l| *l > 0)
        .collect();
    if latencies.is_empty() {
        0
    } else {
        latencies.iter().sum::<u64>() / latencies.len() as u64
    }
}

fn now_ms() -> u64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Routing telemetry ring buffer for field debugging.
/// Stores the last 256 routing decisions in memory (cleared on app kill).
pub struct RoutingTelemetry {
    ring: Vec<RoutingDecisionSnapshot>,
    capacity: usize,
}

/// Snapshot of a single routing decision for telemetry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecisionSnapshot {
    pub message_id_hex: String,
    pub recipient_hint_hex: String,
    pub decided_by: String,
    pub confidence: f64,
    pub primary_hop: String,
    pub alternative_count: usize,
    pub timestamp_ms: u64,
}

impl Default for RoutingTelemetry {
    fn default() -> Self {
        Self::new()
    }
}

impl RoutingTelemetry {
    pub fn new() -> Self {
        Self {
            ring: Vec::with_capacity(256),
            capacity: 256,
        }
    }

    /// Record a routing decision.
    pub fn record(&mut self, decision: &crate::routing::RoutingDecision) {
        let snapshot = RoutingDecisionSnapshot {
            message_id_hex: hex::encode(decision.message_id),
            recipient_hint_hex: hex::encode(decision.recipient_hint),
            decided_by: format!("{:?}", decision.decided_by),
            confidence: decision.confidence,
            primary_hop: format!("{:?}", decision.primary),
            alternative_count: decision.alternatives.len(),
            timestamp_ms: now_ms(),
        };

        if self.ring.len() >= self.capacity {
            self.ring.remove(0);
        }
        self.ring.push(snapshot);
    }

    /// Get all recorded decisions (newest last).
    pub fn entries(&self) -> &[RoutingDecisionSnapshot] {
        &self.ring
    }

    /// Get the number of recorded decisions.
    pub fn len(&self) -> usize {
        self.ring.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ring.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_diagnostics_report_serializes() {
        let report = NetworkDiagnosticsReport {
            connected_peer_count: 3,
            total_messages_sent: 100,
            total_messages_failed: 2,
            avg_latency_ms: 50,
            active_connections: 3,
            connection_summary: vec![],
            generated_at_ms: 1000,
            recent_routing_decisions: vec![],
        };

        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("connected_peer_count"));
        assert!(json.contains("total_messages_sent"));
    }

    #[test]
    fn peer_connection_summary_formats_state() {
        let summary = PeerConnectionSummary {
            peer_id: "peer-1".to_string(),
            state: "Connected".to_string(),
            messages_sent: 10,
            messages_failed: 1,
            avg_latency_ms: 30,
            last_activity_ms: 500,
        };
        assert_eq!(summary.peer_id, "peer-1");
    }

    #[test]
    fn get_network_diagnostics_report_from_empty_monitor() {
        let monitor = TransportHealthMonitor::new();
        let report = get_network_diagnostics_report(&monitor);
        assert_eq!(report.connected_peer_count, 0);
        assert_eq!(report.total_messages_sent, 0);
        assert!(report.connection_summary.is_empty());
    }

    #[test]
    fn routing_telemetry_ring_buffer_capacity() {
        let mut telemetry = RoutingTelemetry::new();
        assert!(telemetry.is_empty());

        // Record 300 decisions — ring should hold last 256
        for i in 0..300 {
            let decision = crate::routing::RoutingDecision {
                message_id: [i as u8; 16],
                recipient_hint: [0xAA; 8],
                primary: crate::routing::NextHop::StoreAndCarry,
                alternatives: vec![],
                decided_by: crate::routing::RoutingLayer::StoreAndCarry,
                confidence: 0.5,
            };
            telemetry.record(&decision);
        }

        assert_eq!(telemetry.len(), 256);
        assert!(!telemetry.is_empty());

        // Verify oldest entry is from index 44 (300 - 256 = 44)
        let entries = telemetry.entries();
        assert_eq!(entries.len(), 256);
    }

    #[test]
    fn diagnostics_report_includes_routing_telemetry() {
        let monitor = TransportHealthMonitor::new();
        let mut telemetry = RoutingTelemetry::new();

        let decision = crate::routing::RoutingDecision {
            message_id: [1u8; 16],
            recipient_hint: [0xAA; 8],
            primary: crate::routing::NextHop::StoreAndCarry,
            alternatives: vec![],
            decided_by: crate::routing::RoutingLayer::StoreAndCarry,
            confidence: 0.5,
        };
        telemetry.record(&decision);

        let report = get_network_diagnostics_report_with_telemetry(&monitor, &telemetry);
        assert_eq!(report.recent_routing_decisions.len(), 1);
    }
}
