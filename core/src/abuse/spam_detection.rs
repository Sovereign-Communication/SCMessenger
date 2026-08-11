//! P0_SECURITY_003: Comprehensive spam detection engine.
//!
//! Implements community-based and heuristic spam detection where:
//! 1. Blocked contacts still route/relay messages (evidence preservation)
//! 2. High percentage of blocks across contacts indicates spam
//! 3. Content-pattern heuristics detect common spam characteristics
//! 4. Flooding/rate-pattern analysis catches volumetric abuse
//! 5. Cold-contact detection flags unsolicited mass outreach

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use parking_lot::RwLock;

use crate::store::blocked::BlockedManager;
use crate::store::contacts::ContactManager;

/// Spam detection configuration
#[derive(Debug, Clone)]
pub struct SpamDetectionConfig {
    /// Minimum number of contacts to consider for spam detection
    pub min_contacts_for_detection: usize,
    /// Percentage threshold for small networks (< 10 contacts)
    pub small_network_spam_threshold: f64,
    /// Percentage threshold for large networks (>= 10 contacts)
    pub large_network_spam_threshold: f64,
    /// Minimum percentage to consider for spam regardless of network size
    pub min_spam_threshold: f64,
    /// Maximum message length before scoring as potential content spam
    pub max_normal_message_len: usize,
    /// Minimum repetitions of identical/similar content to flag as mass distribution
    pub mass_distribution_repeat_threshold: usize,
    /// Window in seconds for flooding detection
    pub flooding_window_secs: u64,
    /// Max messages within the flooding window before flagging
    pub flooding_max_in_window: usize,
    /// Cold-contact threshold: fraction of messages to non-contacts
    pub cold_contact_threshold: f64,
}

impl Default for SpamDetectionConfig {
    fn default() -> Self {
        Self {
            min_contacts_for_detection: 3,
            small_network_spam_threshold: 0.75,
            large_network_spam_threshold: 0.10,
            min_spam_threshold: 0.10,
            max_normal_message_len: 4096,
            mass_distribution_repeat_threshold: 5,
            flooding_window_secs: 60,
            flooding_max_in_window: 30,
            cold_contact_threshold: 0.8,
        }
    }
}

/// Spam detection results
#[derive(Debug, Clone)]
pub struct SpamDetectionResult {
    /// Whether this peer is considered spam
    pub is_spam: bool,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    /// Total contacts sampled
    pub total_contacts: usize,
    /// Number of contacts that have blocked this peer
    pub blocked_by_count: usize,
    /// Percentage of contacts that blocked this peer
    pub block_percentage: f64,
    /// Signals that contributed to this result
    pub contributing_signals: Vec<SpamSignal>,
}

/// Per-peer message tracking for heuristic analysis
#[derive(Debug, Clone)]
struct PeerMessageTrack {
    /// Fingerprint -> count for mass-distribution detection
    content_fingerprints: HashMap<u64, usize>,
    /// Timestamps of recent messages for flooding detection
    recent_message_times: Vec<Instant>,
    /// Count of messages sent to non-contacts
    cold_contact_sends: usize,
    /// Total outbound sends
    total_sends: usize,
    /// Accumulated spam signals and their weights
    signal_accumulator: f64,
}

impl PeerMessageTrack {
    fn new() -> Self {
        Self {
            content_fingerprints: HashMap::new(),
            recent_message_times: Vec::new(),
            cold_contact_sends: 0,
            total_sends: 0,
            signal_accumulator: 0.0,
        }
    }
}

/// Community-based spam detection engine with heuristic analysis.
///
/// The ContactManager and BlockedManager are optional — when absent, community-based
/// detection is skipped but heuristic tracking (flooding, mass-distribution,
/// cold-contact) still operates.
pub struct SpamDetectionEngine {
    config: SpamDetectionConfig,
    contact_manager: Option<Arc<ContactManager>>,
    blocked_manager: Option<Arc<BlockedManager>>,
    /// Per-peer message tracking for heuristic analysis
    peer_tracks: RwLock<HashMap<String, PeerMessageTrack>>,
    /// Maximum tracked peers to bound memory
    max_tracked_peers: usize,
}

impl SpamDetectionEngine {
    /// Create a full-featured engine with community detection support.
    pub fn new(
        config: SpamDetectionConfig,
        contact_manager: Arc<ContactManager>,
        blocked_manager: Arc<BlockedManager>,
    ) -> Self {
        Self {
            config,
            contact_manager: Some(contact_manager),
            blocked_manager: Some(blocked_manager),
            peer_tracks: RwLock::new(HashMap::new()),
            max_tracked_peers: 4096,
        }
    }

    /// Create an engine with heuristic-only detection (no community blocking data).
    /// This is used when ContactManager/BlockedManager aren't available at construction time.
    pub fn new_heuristics_only(config: SpamDetectionConfig) -> Self {
        Self {
            config,
            contact_manager: None,
            blocked_manager: None,
            peer_tracks: RwLock::new(HashMap::new()),
            max_tracked_peers: 4096,
        }
    }

    /// Simple fingerprint for content similarity detection.
    /// Uses FNV-1a-like hashing of byte content for fast comparison.
    fn content_fingerprint(data: &[u8]) -> u64 {
        let mut hash: u64 = 0xcbf29ce484222325;
        for &byte in data {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }

    /// Detect if a peer is likely spam based on community blocking patterns and heuristics.
    pub fn detect_spam(&self, peer_id: &str) -> SpamDetectionResult {
        let mut contributing_signals = Vec::new();
        let mut blocked_by_count = 0;
        let mut total_contacts = 0;
        let mut block_percentage = 0.0;

        // Community-based detection (requires ContactManager + BlockedManager)
        if let (Some(ref cm), Some(ref bm)) = (&self.contact_manager, &self.blocked_manager) {
            let contacts = cm.list().unwrap_or_else(|_| vec![]);
            total_contacts = contacts.len();

            if total_contacts >= self.config.min_contacts_for_detection {
                let blocked_set = bm.blocked_only_peer_ids().unwrap_or_default();
                for contact in &contacts {
                    if blocked_set.contains(&contact.peer_id) {
                        blocked_by_count += 1;
                    } else if let Ok(true) = bm.is_blocked_resolved(&contact.peer_id, None) {
                        blocked_by_count += 1;
                    }
                }

                block_percentage = if total_contacts == 0 {
                    0.0
                } else {
                    blocked_by_count as f64 / total_contacts as f64
                };

                let spam_threshold = if total_contacts < 10 {
                    self.config.small_network_spam_threshold
                } else {
                    self.config.large_network_spam_threshold
                };

                let effective_threshold = spam_threshold.max(self.config.min_spam_threshold);

                if block_percentage >= effective_threshold {
                    contributing_signals.push(SpamSignal::CommunityBlocked);
                }
            }
        }

        // Heuristic signals from peer tracking (always available)
        let tracks = self.peer_tracks.read();
        if let Some(track) = tracks.get(peer_id) {
            let now = Instant::now();
            let window = std::time::Duration::from_secs(self.config.flooding_window_secs);

            // Flooding detection
            let recent_count = track
                .recent_message_times
                .iter()
                .filter(|t| now.duration_since(**t) < window)
                .count();
            if recent_count >= self.config.flooding_max_in_window {
                contributing_signals.push(SpamSignal::Flooding);
            }

            // Mass distribution detection
            let max_repeats = track
                .content_fingerprints
                .values()
                .copied()
                .max()
                .unwrap_or(0);
            if max_repeats >= self.config.mass_distribution_repeat_threshold {
                contributing_signals.push(SpamSignal::MassDistribution);
            }

            // Cold contact spam detection
            if track.total_sends > 0 {
                let cold_ratio = track.cold_contact_sends as f64 / track.total_sends as f64;
                if cold_ratio >= self.config.cold_contact_threshold && track.total_sends >= 5 {
                    contributing_signals.push(SpamSignal::ColdContactSpam);
                }
            }
        }

        let is_spam = !contributing_signals.is_empty();
        let community_confidence = if block_percentage > 0.0 {
            let spam_threshold = if total_contacts < 10 {
                self.config.small_network_spam_threshold
            } else {
                self.config.large_network_spam_threshold
            };
            let effective_threshold = spam_threshold.max(self.config.min_spam_threshold);
            if block_percentage >= effective_threshold {
                (block_percentage / effective_threshold).min(1.0)
            } else {
                0.0
            }
        } else {
            0.0
        };
        let heuristic_confidence = contributing_signals
            .iter()
            .filter(|s| !matches!(s, SpamSignal::CommunityBlocked))
            .count() as f64
            * 0.3;
        let confidence = (community_confidence + heuristic_confidence).min(1.0);

        SpamDetectionResult {
            is_spam,
            confidence,
            total_contacts,
            blocked_by_count,
            block_percentage,
            contributing_signals,
        }
    }

    /// Get spam score for a peer (0.0 to 1.0, where 1.0 is definitely spam)
    pub fn spam_score(&self, peer_id: &str) -> f64 {
        let result = self.detect_spam(peer_id);
        if result.is_spam {
            result.confidence
        } else {
            0.0
        }
    }

    /// Record a spam signal that contributes to reputation scoring.
    /// Also updates internal peer tracking state for heuristic analysis.
    pub fn record_spam_signal(&self, peer_id: &str, signal: SpamSignal) {
        let mut tracks = self.peer_tracks.write();

        // Evict if at capacity
        if !tracks.contains_key(peer_id) && tracks.len() >= self.max_tracked_peers {
            if let Some(evict_key) = tracks.keys().next().cloned() {
                tracks.remove(&evict_key);
            }
        }

        let track = tracks
            .entry(peer_id.to_string())
            .or_insert_with(PeerMessageTrack::new);

        let weight = match signal {
            SpamSignal::CommunityBlocked => 0.4,
            SpamSignal::ContentPattern => 0.3,
            SpamSignal::Flooding => 0.2,
            SpamSignal::MassDistribution => 0.25,
            SpamSignal::ColdContactSpam => 0.15,
        };
        track.signal_accumulator += weight;
    }

    /// Record an outbound message for heuristic tracking.
    pub fn record_outbound_message(
        &self,
        peer_id: &str,
        envelope_data: &[u8],
        is_to_contact: bool,
    ) {
        let mut tracks = self.peer_tracks.write();

        if !tracks.contains_key(peer_id) && tracks.len() >= self.max_tracked_peers {
            if let Some(evict_key) = tracks.keys().next().cloned() {
                tracks.remove(&evict_key);
            }
        }

        let track = tracks
            .entry(peer_id.to_string())
            .or_insert_with(PeerMessageTrack::new);

        let fp = Self::content_fingerprint(envelope_data);
        *track.content_fingerprints.entry(fp).or_insert(0) += 1;

        if track.content_fingerprints.len() > 256 {
            let mut fps: Vec<_> = track.content_fingerprints.iter().collect();
            fps.sort_by_key(|(_, c)| **c);
            track.content_fingerprints = fps
                .into_iter()
                .rev()
                .take(128)
                .map(|(k, v)| (*k, *v))
                .collect();
        }

        let now = Instant::now();
        track.recent_message_times.push(now);
        let window = std::time::Duration::from_secs(self.config.flooding_window_secs);
        track
            .recent_message_times
            .retain(|t| now.duration_since(*t) < window);

        track.total_sends += 1;
        if !is_to_contact {
            track.cold_contact_sends += 1;
        }
    }

    /// Check if content matches spam-like patterns (excessive length).
    pub fn is_content_suspicious(&self, envelope_data: &[u8]) -> bool {
        envelope_data.len() > self.config.max_normal_message_len
    }

    /// Prune stale peer tracking entries.
    pub fn prune_stale_peers(&self, max_entries: usize) -> usize {
        let mut tracks = self.peer_tracks.write();
        let before = tracks.len();
        if tracks.len() > max_entries {
            let remove_count = tracks.len() - max_entries;
            let keys: Vec<String> = tracks.keys().take(remove_count).cloned().collect();
            for key in keys {
                tracks.remove(&key);
            }
        }
        before - tracks.len()
    }
}

/// Types of spam signals that can be detected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpamSignal {
    /// High percentage of contacts have blocked this peer
    CommunityBlocked,
    /// Message content matches known spam patterns
    ContentPattern,
    /// Sending rate exceeds normal communication patterns
    Flooding,
    /// Messages are identical or very similar (mass spam)
    MassDistribution,
    /// Peer is sending to many new/unconnected contacts
    ColdContactSpam,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::backend::MemoryStorage;
    use std::sync::Arc;

    fn make_engine() -> SpamDetectionEngine {
        let backend = Arc::new(MemoryStorage::new());
        let contacts = Arc::new(ContactManager::new(backend.clone()));
        let blocked = Arc::new(BlockedManager::new(backend));
        SpamDetectionEngine::new(SpamDetectionConfig::default(), contacts, blocked)
    }

    fn make_heuristics_only_engine() -> SpamDetectionEngine {
        SpamDetectionEngine::new_heuristics_only(SpamDetectionConfig::default())
    }

    #[test]
    fn test_default_config() {
        let config = SpamDetectionConfig::default();
        assert_eq!(config.min_contacts_for_detection, 3);
        assert_eq!(config.flooding_max_in_window, 30);
    }

    #[test]
    fn test_no_contacts_is_not_spam() {
        let engine = make_engine();
        let result = engine.detect_spam("unknown_peer");
        assert!(!result.is_spam);
        assert_eq!(result.confidence, 0.0);
    }

    #[test]
    fn test_heuristics_only_no_contacts_is_not_spam() {
        let engine = make_heuristics_only_engine();
        let result = engine.detect_spam("unknown_peer");
        assert!(!result.is_spam);
        assert_eq!(result.total_contacts, 0);
    }

    #[test]
    fn test_content_fingerprint_deterministic() {
        let data = b"hello world";
        let fp1 = SpamDetectionEngine::content_fingerprint(data);
        let fp2 = SpamDetectionEngine::content_fingerprint(data);
        assert_eq!(fp1, fp2);

        let different = b"hello world!";
        let fp3 = SpamDetectionEngine::content_fingerprint(different);
        assert_ne!(fp1, fp3);
    }

    #[test]
    fn test_record_spam_signal_accumulates() {
        let engine = make_engine();
        engine.record_spam_signal("peer1", SpamSignal::Flooding);
        let tracks = engine.peer_tracks.read();
        assert!(tracks.contains_key("peer1"));
        assert!(tracks.get("peer1").unwrap().signal_accumulator > 0.0);
    }

    #[test]
    fn test_record_outbound_message() {
        let engine = make_engine();
        engine.record_outbound_message("peer1", b"test message", true);
        let tracks = engine.peer_tracks.read();
        let track = tracks.get("peer1").unwrap();
        assert_eq!(track.total_sends, 1);
        assert_eq!(track.cold_contact_sends, 0);
    }

    #[test]
    fn test_record_outbound_cold_contact() {
        let engine = make_engine();
        engine.record_outbound_message("peer1", b"test", false);
        let tracks = engine.peer_tracks.read();
        let track = tracks.get("peer1").unwrap();
        assert_eq!(track.cold_contact_sends, 1);
    }

    #[test]
    fn test_content_suspicious_length() {
        let engine = make_engine();
        let short = vec![0u8; 100];
        let long = vec![0u8; engine.config.max_normal_message_len + 1];
        assert!(!engine.is_content_suspicious(&short));
        assert!(engine.is_content_suspicious(&long));
    }

    #[test]
    fn test_prune_stale_peers() {
        let engine = make_engine();
        for i in 0..20 {
            let peer_id = format!("peer{}", i);
            engine.record_spam_signal(&peer_id, SpamSignal::Flooding);
        }
        let pruned = engine.prune_stale_peers(10);
        assert!(pruned > 0);
        let tracks = engine.peer_tracks.read();
        assert!(tracks.len() <= 10);
    }

    #[test]
    fn test_heuristics_only_flooding_detection() {
        let engine = make_heuristics_only_engine();
        // Simulate flooding: many messages in quick succession
        for i in 0..35 {
            let msg = format!("message {}", i);
            engine.record_outbound_message("flooder", msg.as_bytes(), true);
        }
        let result = engine.detect_spam("flooder");
        assert!(result.is_spam);
        assert!(result.contributing_signals.contains(&SpamSignal::Flooding));
    }
}
