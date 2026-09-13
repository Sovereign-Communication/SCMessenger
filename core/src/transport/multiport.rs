// Multi-Port Adaptive Listening
//
// Enables nodes to listen on multiple ports simultaneously to maximize
// connectivity in restrictive network environments. Common ports (443, 80)
// are prioritized for firewall traversal.

use libp2p::Multiaddr;
// use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
// use tracing::{info, warn};

/// Common ports to try for maximum connectivity
pub const COMMON_PORTS: &[u16] = &[
    443,  // HTTPS (most likely to be allowed outbound)
    80,   // HTTP (widely allowed)
    8080, // HTTP alternate (common proxy port)
    9090, // Common alternative
];

/// Ports excluded from adaptive swarm listening (control/health API ports, dedicated listeners)
pub const EXCLUDED_PORTS: &[u16] = &[
    9876, // Control API & Relay Health server port
    9002, // WASM bridge WebSocket listener (swarm.rs) -- must not be
          // double-bound as TCP by the multiport generator
];

/// Configuration for multi-port listening
#[derive(Debug, Clone)]
pub struct MultiPortConfig {
    /// Whether to try binding to common ports
    pub enable_common_ports: bool,
    /// Whether to bind to a random port (always recommended)
    pub enable_random_port: bool,
    /// Specific ports to try (in addition to common ports)
    pub additional_ports: Vec<u16>,
    /// IP version preferences
    pub enable_ipv4: bool,
    pub enable_ipv6: bool,
    /// Preferred port to try first (before common ports)
    pub preferred_port: Option<u16>,
}

impl Default for MultiPortConfig {
    fn default() -> Self {
        Self {
            enable_common_ports: true,
            enable_random_port: true,
            additional_ports: Vec::new(),
            enable_ipv4: true,
            enable_ipv6: true,
            preferred_port: None,
        }
    }
}

/// Result of attempting to bind to a port
#[derive(Debug, Clone)]
pub enum BindResult {
    /// Successfully bound
    Success { addr: Multiaddr, port: u16 },
    /// Failed to bind (permission denied, port in use, etc.)
    Failed { port: u16, error: String },
    /// Skipped (not attempted)
    Skipped { port: u16, reason: String },
}

/// Generate list of addresses to attempt binding to.
///
/// Deterministic transport assignment:
/// For every candidate port, at most ONE transport is generated and bound:
/// plain TCP (`/ip4/0.0.0.0/tcp/{port}` for IPv4, `/ip6/::/tcp/{port}` for IPv6).
/// WebSocket (`/ws`) dual-binding on the same port is eliminated so that inbound
/// connections never collide with mismatched transport protocol negotiation.
/// Ports are deduplicated across preferred_port, common_ports, additional_ports,
/// and the ephemeral random port (0).
pub fn generate_listen_addresses(config: &MultiPortConfig) -> Vec<(Multiaddr, u16)> {
    let mut addresses = Vec::new();
    let mut seen_ports = std::collections::HashSet::new();

    // Helper to add IPv4 and IPv6 plain TCP addresses for a port (exactly one transport per port)
    let mut add_port = |port: u16| {
        if EXCLUDED_PORTS.contains(&port) {
            return;
        }
        if !seen_ports.insert(port) {
            return;
        }
        if config.enable_ipv4 {
            let tcp_addr: Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", port)
                .parse()
                .expect("valid IPv4 TCP multiaddr format");
            addresses.push((tcp_addr, port));
        }
        if config.enable_ipv6 {
            let tcp_addr: Multiaddr = format!("/ip6/::/tcp/{}", port)
                .parse()
                .expect("valid IPv6 TCP multiaddr format");
            addresses.push((tcp_addr, port));
        }
    };

    // Add preferred port first (most prioritized)
    if let Some(port) = config.preferred_port {
        add_port(port);
    }

    // Add common ports next
    if config.enable_common_ports {
        for &port in COMMON_PORTS {
            add_port(port);
        }
    }

    // Add additional user-specified ports
    for &port in &config.additional_ports {
        add_port(port);
    }

    // Add random port last (0 = OS assigns)
    if config.enable_random_port {
        add_port(0);
    }

    addresses
}

/// Check if a port requires elevated privileges (ports < 1024 on Unix)
// PERIMETER-ALLOW-UNDERSCORE: `_port` IS read below under cfg(unix); on
// non-unix targets that arm compiles away and the parameter is genuinely
// unused there, so the underscore avoids a per-platform warning without
// needing two differently-named signatures.
pub fn requires_elevated_privileges(_port: u16) -> bool {
    #[cfg(unix)]
    {
        _port > 0 && _port < 1024
    }
    #[cfg(not(unix))]
    {
        false
    }
}

/// Analyze bind results and provide recommendations
pub fn analyze_bind_results(results: &[BindResult]) -> BindAnalysis {
    let mut successful = Vec::new();
    let mut failed_permission = Vec::new();
    let mut failed_in_use = Vec::new();
    let mut failed_other = Vec::new();

    for result in results {
        match result {
            BindResult::Success { addr, port } => {
                successful.push((addr.clone(), *port));
            }
            BindResult::Failed { port, error } => {
                if error.contains("permission") || error.contains("Permission") {
                    failed_permission.push(*port);
                } else if error.contains("in use") || error.contains("address already in use") {
                    failed_in_use.push(*port);
                } else {
                    failed_other.push((*port, error.clone()));
                }
            }
            BindResult::Skipped { .. } => {}
        }
    }

    // Categorize success level
    let has_common_port = successful
        .iter()
        .any(|(_, port)| COMMON_PORTS.contains(port));
    let has_random_port = successful
        .iter()
        .any(|(_, port)| *port == 0 || *port > 10000);

    let status = match (has_common_port, has_random_port, successful.len()) {
        (true, true, _) => ConnectivityStatus::Excellent,
        (true, false, _) => ConnectivityStatus::Good,
        (false, true, n) if n > 0 => ConnectivityStatus::Moderate,
        (false, false, n) if n > 0 => ConnectivityStatus::Limited,
        _ => ConnectivityStatus::None,
    };

    BindAnalysis {
        successful,
        failed_permission,
        failed_in_use,
        failed_other,
        status,
    }
}

/// Analysis of bind attempt results
#[derive(Debug)]
pub struct BindAnalysis {
    /// Successfully bound addresses
    pub successful: Vec<(Multiaddr, u16)>,
    /// Ports that failed due to permission issues
    pub failed_permission: Vec<u16>,
    /// Ports that failed because they're already in use
    pub failed_in_use: Vec<u16>,
    /// Ports that failed for other reasons
    pub failed_other: Vec<(u16, String)>,
    /// Overall connectivity assessment
    pub status: ConnectivityStatus,
}

impl BindAnalysis {
    /// Generate user-friendly report
    pub fn report(&self) -> String {
        let mut report = String::new();

        report.push_str(&format!(
            "[OK] Listening on {} address(es)\n",
            self.successful.len()
        ));

        for (addr, port) in &self.successful {
            let priority = if COMMON_PORTS.contains(port) {
                "priority"
            } else if *port == 0 || *port > 10000 {
                "random"
            } else {
                "custom"
            };
            report.push_str(&format!("  {} ({})\n", addr, priority));
        }

        if !self.failed_permission.is_empty() {
            report.push_str(&format!(
                "\n[WARNING] {} port(s) need elevated privileges: {:?}\n",
                self.failed_permission.len(),
                self.failed_permission
            ));
            report.push_str("  Tip: Run with sudo/admin rights, or use ports > 1024\n");
        }

        if !self.failed_in_use.is_empty() {
            report.push_str(&format!(
                "\n[WARNING] {} port(s) already in use: {:?}\n",
                self.failed_in_use.len(),
                self.failed_in_use
            ));
        }

        if !self.failed_other.is_empty() {
            report.push_str(&format!(
                "\n[WARNING] {} port(s) failed for other reasons\n",
                self.failed_other.len()
            ));
        }

        report.push_str(&format!("\nConnectivity: {:?}", self.status));

        report
    }
}

/// Overall connectivity assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectivityStatus {
    /// Listening on both common ports and random port
    Excellent,
    /// Listening on common ports only
    Good,
    /// Listening on non-common ports
    Moderate,
    /// Very limited listening capabilities
    Limited,
    /// Not listening on any port
    None,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_listen_addresses_default() {
        let config = MultiPortConfig::default();
        let addresses = generate_listen_addresses(&config);

        // Should have common ports * 2 (IPv4 + IPv6) + random port * 2
        assert!(
            addresses.len() >= COMMON_PORTS.len() * 2,
            "Should generate addresses for common ports"
        );
    }

    #[test]
    fn test_generate_listen_addresses_ipv4_only() {
        let config = MultiPortConfig {
            enable_common_ports: true,
            enable_random_port: true,
            additional_ports: vec![],
            enable_ipv4: true,
            enable_ipv6: false,
            preferred_port: None,
        };
        let addresses = generate_listen_addresses(&config);

        // All addresses should be IPv4
        for (addr, _) in &addresses {
            assert!(
                addr.to_string().contains("/ip4/"),
                "Should only have IPv4 addresses"
            );
        }
    }

    #[test]
    fn test_generate_listen_addresses_custom_ports() {
        let config = MultiPortConfig {
            enable_common_ports: false,
            enable_random_port: false,
            additional_ports: vec![9999, 8888],
            enable_ipv4: true,
            enable_ipv6: false,
            preferred_port: None,
        };
        let addresses = generate_listen_addresses(&config);

        assert_eq!(
            addresses.len(),
            2,
            "Should have 2 custom addresses (plain TCP)"
        );
        assert!(addresses.iter().any(|(_, port)| *port == 9999));
        assert!(addresses.iter().any(|(_, port)| *port == 8888));
    }

    #[test]
    fn test_requires_elevated_privileges() {
        #[cfg(unix)]
        {
            assert!(requires_elevated_privileges(80));
            assert!(requires_elevated_privileges(443));
            assert!(!requires_elevated_privileges(8080));
            assert!(!requires_elevated_privileges(9999));
        }
    }

    #[test]
    fn test_analyze_bind_results() {
        let results = vec![
            BindResult::Success {
                addr: "/ip4/0.0.0.0/tcp/443".parse().unwrap(),
                port: 443,
            },
            BindResult::Success {
                addr: "/ip4/0.0.0.0/tcp/12345".parse().unwrap(),
                port: 12345,
            },
            BindResult::Failed {
                port: 80,
                error: "Permission denied".to_string(),
            },
            BindResult::Failed {
                port: 8080,
                error: "Address already in use".to_string(),
            },
        ];

        let analysis = analyze_bind_results(&results);

        assert_eq!(analysis.successful.len(), 2);
        assert_eq!(analysis.failed_permission.len(), 1);
        assert_eq!(analysis.failed_in_use.len(), 1);
        assert_eq!(analysis.status, ConnectivityStatus::Excellent);
    }

    #[test]
    fn test_bind_analysis_report() {
        let results = vec![BindResult::Success {
            addr: "/ip4/0.0.0.0/tcp/443".parse().unwrap(),
            port: 443,
        }];

        let analysis = analyze_bind_results(&results);
        let report = analysis.report();

        assert!(report.contains("Listening on 1 address"));
        assert!(report.contains("443"));
        assert!(report.contains("priority"));
    }

    #[test]
    fn test_excluded_ports_filtered() {
        let config = MultiPortConfig {
            additional_ports: vec![9876, 9000],
            preferred_port: Some(9876),
            ..Default::default()
        };
        let addrs = generate_listen_addresses(&config);
        for (_, port) in addrs {
            assert_ne!(
                port, 9876,
                "Control API port 9876 must be excluded from listener addresses"
            );
        }
    }

    #[test]
    fn test_excluded_port_9002_wasm_bridge_never_emitted() {
        let ip_permutations = [
            (true, true),   // Dual-stack (IPv4 + IPv6)
            (true, false),  // IPv4 only
            (false, true),  // IPv6 only
            (false, false), // Neither
        ];

        for (enable_ipv4, enable_ipv6) in ip_permutations {
            let config = MultiPortConfig {
                enable_common_ports: true,
                enable_random_port: true,
                additional_ports: vec![9002, 9000, 8080],
                enable_ipv4,
                enable_ipv6,
                preferred_port: Some(9002),
            };

            let addresses = generate_listen_addresses(&config);

            for (addr, port) in &addresses {
                assert_ne!(
                    *port, 9002,
                    "WASM bridge WebSocket port 9002 must never be emitted by multiport generator (ipv4={}, ipv6={})",
                    enable_ipv4, enable_ipv6
                );
                assert!(
                    !addr.to_string().contains("/tcp/9002"),
                    "Address {} must not contain port 9002 (ipv4={}, ipv6={})",
                    addr,
                    enable_ipv4,
                    enable_ipv6
                );
            }

            assert!(
                !addresses.iter().any(|(_, port)| *port == 9002),
                "generate_listen_addresses must not return port 9002 when explicitly requested (ipv4={}, ipv6={})",
                enable_ipv4, enable_ipv6
            );
        }
    }

    #[test]
    fn test_no_two_emitted_addresses_share_ip_port_pair() {
        let configs = vec![
            MultiPortConfig::default(),
            MultiPortConfig {
                enable_common_ports: true,
                enable_random_port: true,
                additional_ports: vec![443, 8080, 9999, 12345],
                enable_ipv4: true,
                enable_ipv6: true,
                preferred_port: Some(443),
            },
            MultiPortConfig {
                enable_common_ports: true,
                enable_random_port: false,
                additional_ports: vec![9090, 80],
                enable_ipv4: true,
                enable_ipv6: false,
                preferred_port: Some(80),
            },
        ];

        for config in configs {
            let addresses = generate_listen_addresses(&config);
            let mut seen_ip_ports = std::collections::HashSet::new();

            for (addr, port) in &addresses {
                let ip_family = if addr.to_string().starts_with("/ip4/") {
                    "ip4"
                } else if addr.to_string().starts_with("/ip6/") {
                    "ip6"
                } else {
                    panic!("unexpected address format: {}", addr);
                };

                let key = (ip_family, *port);
                assert!(
                    seen_ip_ports.insert(key),
                    "Duplicate (ip, port) detected: {:?} with addr {}",
                    key,
                    addr
                );
            }
        }
    }

    #[test]
    fn test_ipv4_and_ipv6_enabled_each_ip_port_appears_once() {
        let config = MultiPortConfig {
            enable_common_ports: true,
            enable_random_port: true,
            additional_ports: vec![7000, 8000],
            enable_ipv4: true,
            enable_ipv6: true,
            preferred_port: Some(9000),
        };

        let addresses = generate_listen_addresses(&config);

        let mut counts = std::collections::HashMap::new();
        for (addr, port) in &addresses {
            let is_ipv4 = addr.to_string().starts_with("/ip4/");
            let is_ipv6 = addr.to_string().starts_with("/ip6/");
            assert!(is_ipv4 || is_ipv6, "Address must be IPv4 or IPv6: {}", addr);

            let key = (is_ipv4, *port);
            *counts.entry(key).or_insert(0) += 1;
        }

        for (key, count) in &counts {
            assert_eq!(
                *count, 1,
                "Expected (is_ipv4={}, port={}) to appear exactly once, appeared {} times",
                key.0, key.1, count
            );
        }

        let unique_ports: std::collections::HashSet<u16> =
            addresses.iter().map(|(_, port)| *port).collect();
        for port in unique_ports {
            let ipv4_count = addresses
                .iter()
                .filter(|(a, p)| *p == port && a.to_string().starts_with("/ip4/"))
                .count();
            let ipv6_count = addresses
                .iter()
                .filter(|(a, p)| *p == port && a.to_string().starts_with("/ip6/"))
                .count();

            assert_eq!(
                ipv4_count, 1,
                "Port {} should have exactly 1 IPv4 address",
                port
            );
            assert_eq!(
                ipv6_count, 1,
                "Port {} should have exactly 1 IPv6 address",
                port
            );
        }
    }

    #[test]
    fn test_failed_or_skipped_bind_not_advertised() {
        let successful_addr: Multiaddr = "/ip4/0.0.0.0/tcp/443".parse().unwrap();
        let failed_addr_port = 80;
        let skipped_addr_port = 8080;

        let results = vec![
            BindResult::Success {
                addr: successful_addr.clone(),
                port: 443,
            },
            BindResult::Failed {
                port: failed_addr_port,
                error: "Address already in use".to_string(),
            },
            BindResult::Skipped {
                port: skipped_addr_port,
                reason: "Not attempted".to_string(),
            },
        ];

        let analysis = analyze_bind_results(&results);

        assert_eq!(analysis.successful.len(), 1);
        assert_eq!(analysis.successful[0], (successful_addr, 443));

        assert!(
            !analysis
                .successful
                .iter()
                .any(|(_, p)| *p == failed_addr_port),
            "Failed port {} must not be present in successful addresses",
            failed_addr_port
        );
        assert!(
            !analysis
                .successful
                .iter()
                .any(|(_, p)| *p == skipped_addr_port),
            "Skipped port {} must not be present in successful addresses",
            skipped_addr_port
        );

        let report = analysis.report();
        assert!(report.contains("Listening on 1 address"));
        assert!(report.contains("443"));
        assert!(!report.contains("/tcp/80 ("));
        assert!(!report.contains("/tcp/8080 ("));
    }
}
