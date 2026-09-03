//! Multi-path delivery (Phase 2 API)
//!
//! Tracks multiple delivery paths per recipient hint, scores them by
//! success rate and latency, and prunes underperforming routes.

use std::collections::HashMap;

/// Represents a delivery path for multi-path message routing
#[derive(Debug, Clone)]
pub struct DeliveryPath {
    /// Unique path identifier
    pub path_id: u64,
    /// Target peer ID (32-byte Ed25519 public key hash)
    pub peer_id: [u8; 32],
    /// Latency estimate in milliseconds (moving average)
    pub estimated_latency_ms: u64,
    /// Whether this path is currently active
    pub active: bool,
    /// Performance score (0.0–100.0), higher is better
    pub score: f64,
    /// Total delivery attempts through this path
    pub attempt_count: u64,
    /// Successful deliveries through this path
    pub success_count: u64,
}

impl DeliveryPath {
    /// Recalculate the performance score based on current statistics.
    ///
    /// Formula:
    ///   success_ratio = success_count / max(attempt_count, 1)
    ///   latency_penalty = min(estimated_latency_ms, 2000) / 100.0
    ///   score = clamp((success_ratio * 80.0) + 20.0 - latency_penalty, 0.0, 100.0)
    ///
    /// New paths with 0 attempts start at 75.0 (neutral-positive).
    pub fn recalculate_score(&mut self) {
        if self.attempt_count == 0 {
            self.score = 75.0;
            return;
        }
        let success_ratio = self.success_count as f64 / self.attempt_count.max(1) as f64;
        let latency_penalty = (self.estimated_latency_ms.min(2000) as f64) / 100.0;
        self.score = ((success_ratio * 80.0) + 20.0 - latency_penalty).clamp(0.0, 100.0);
    }

    /// Record a successful delivery, updating the moving average latency.
    pub fn record_success(&mut self, latency_ms: u64) {
        self.attempt_count += 1;
        self.success_count += 1;
        // Moving average: new_avg = (old_avg + sample) / 2
        self.estimated_latency_ms = (self.estimated_latency_ms + latency_ms) / 2;
        self.recalculate_score();
    }

    /// Record a failed delivery attempt.
    pub fn record_failure(&mut self) {
        self.attempt_count += 1;
        self.recalculate_score();
    }
}

/// Manages multi-path message delivery across redundant routes.
///
/// Paths are indexed by recipient hint (`[u8; 8]`) for O(1) lookup
/// from the routing engine's decision path.
#[derive(Debug, Clone)]
pub struct MultiPathDelivery {
    paths: HashMap<[u8; 8], Vec<DeliveryPath>>,
    max_paths_per_hint: usize,
}

impl Default for MultiPathDelivery {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiPathDelivery {
    /// Create a new multi-path delivery manager
    pub fn new() -> Self {
        Self {
            paths: HashMap::new(),
            max_paths_per_hint: 3,
        }
    }

    /// Register a delivery path for a recipient hint.
    ///
    /// If a path with the same `path_id` already exists under this hint,
    /// it is replaced. Otherwise the path is appended if under the limit.
    pub fn register_path(&mut self, hint: [u8; 8], path: DeliveryPath) {
        let paths = self.paths.entry(hint).or_default();
        // Update existing path with same path_id
        if let Some(existing) = paths.iter_mut().find(|p| p.path_id == path.path_id) {
            *existing = path;
            return;
        }
        if paths.len() < self.max_paths_per_hint {
            paths.push(path);
        }
    }

    /// Get all active paths for a recipient hint, sorted by score descending.
    pub fn active_paths(&self, hint: &[u8; 8]) -> Vec<&DeliveryPath> {
        let mut result: Vec<&DeliveryPath> = self
            .paths
            .get(hint)
            .map(|paths| paths.iter().filter(|p| p.active).collect())
            .unwrap_or_default();
        result.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        result
    }

    /// Mark a path as inactive (failed).
    pub fn mark_path_failed(&mut self, path_id: u64) {
        for paths in self.paths.values_mut() {
            if let Some(path) = paths.iter_mut().find(|p| p.path_id == path_id) {
                path.active = false;
                path.record_failure();
                break;
            }
        }
    }

    /// Prune paths whose score falls below the given threshold.
    ///
    /// Paths below threshold are deactivated (`active = false`) but
    /// retained in storage so their history is preserved for scoring.
    pub fn prune_below(&mut self, threshold: f64) {
        for paths in self.paths.values_mut() {
            for path in paths.iter_mut() {
                if path.score < threshold {
                    path.active = false;
                }
            }
        }
    }

    /// Get the number of recipient hints with registered paths
    pub fn len(&self) -> usize {
        self.paths.len()
    }

    /// Check if the delivery manager is empty
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }
}
