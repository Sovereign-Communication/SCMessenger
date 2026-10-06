//! Bloom Filter Negative Cache for DHT Peer Discovery
//!
//! Quickly identifies peers that are DEFINITELY unreachable without
//! performing expensive DHT walks. This reduces wasted lookups for
//! sleeping or offline peers.
//!
//! # Design Principles
//!
//! 1. **Fast negative answers**: Bloom filter check is O(1) vs O(log n) DHT walk
//! 2. **Bounded false positives**: Accept some false positives (treat reachable as unreachable)
//!    in exchange for fast negative answers. False positives trigger fallback to DHT.
//! 3. **Time-based expiry**: Negative results expire after TTL to allow recovery
//! 4. **Privacy-preserving**: Local-only cache, no network queries
//! 5. **Reputation-aware**: Use overall_score from abuse reputation to avoid caching trusted peers

use std::collections::HashMap;
use web_time::{Duration, Instant};

/// A simple bloom filter implementation for peer unreachability
///
/// Uses a bit vector with k hash functions. False positives are possible
/// (may treat a reachable peer as unreachable) but false negatives are not
/// (will never treat an unreachable peer as reachable).
#[derive(Debug, Clone)]
pub struct BloomFilter {
    /// Bit vector
    bits: Vec<bool>,
    /// Number of bits
    size: usize,
    /// Number of hash functions
    k: usize,
    /// Number of items inserted (for capacity estimation)
    count: usize,
}

impl BloomFilter {
    /// Create a new bloom filter with given capacity and false positive rate
    ///
    /// # Arguments
    /// * `expected_items` - Expected number of items to store
    /// * `false_positive_rate` - Desired false positive rate (e.g., 0.01 for 1%)
    pub fn new(expected_items: usize, false_positive_rate: f64) -> Self {
        // Calculate optimal size: m = -(n * ln(p)) / (ln(2)^2)
        let m = (-(expected_items as f64) * false_positive_rate.ln()) / (2.0_f64.ln().powi(2));
        let size = m.ceil() as usize;

        // Calculate optimal k: k = (m/n) * ln(2)
        let k = ((size as f64 / expected_items as f64) * 2.0_f64.ln()).ceil() as usize;

        BloomFilter {
            bits: vec![false; size],
            size,
            k,
            count: 0,
        }
    }

    /// Create a bloom filter optimized for 1000 peers at 1% false positive rate
    pub fn for_peer_cache() -> Self {
        Self::new(1000, 0.01)
    }

    /// Hash a peer ID to get k bit positions
    fn hash_positions(&self, peer_id: &str) -> Vec<usize> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut positions = Vec::with_capacity(self.k);

        // Use double hashing technique: h(i) = h1 + i * h2
        let mut hasher1 = DefaultHasher::new();
        peer_id.hash(&mut hasher1);
        let h1 = hasher1.finish() as usize;

        let mut hasher2 = DefaultHasher::new();
        // Add salt to get a different hash
        (peer_id, "salt").hash(&mut hasher2);
        let h2 = hasher2.finish() as usize;

        for i in 0..self.k {
            let pos = (h1.wrapping_add(i.wrapping_mul(h2))) % self.size;
            positions.push(pos);
        }

        positions
    }

    /// Insert an item into the bloom filter
    pub fn insert(&mut self, peer_id: &str) {
        for pos in self.hash_positions(peer_id) {
            self.bits[pos] = true;
        }
        self.count += 1;
    }

    /// Check if an item MIGHT be in the filter
    ///
    /// Returns true if the item might be in the filter (false positive possible).
    /// Returns false if the item is DEFINITELY not in the filter (no false negatives).
    pub fn contains(&self, peer_id: &str) -> bool {
        for pos in self.hash_positions(peer_id) {
            if !self.bits[pos] {
                return false; // Definitely not in filter
            }
        }
        true // Might be in filter (could be false positive)
    }

    /// Get the current false positive rate estimate
    pub fn false_positive_rate(&self) -> f64 {
        if self.count == 0 {
            return 0.0;
        }
        // p ≈ (1 - e^(-k*m/n))^k
        let n = self.count as f64;
        let m = self.size as f64;
        let k = self.k as f64;
        (1.0 - (-k * n / m).exp()).powf(k)
    }

    /// Get the number of items inserted
    pub fn count(&self) -> usize {
        self.count
    }

    /// Clear the filter (reset all bits)
    pub fn clear(&mut self) {
        self.bits.fill(false);
        self.count = 0;
    }
}

/// Information about a peer in the negative cache
#[derive(Debug, Clone)]
struct NegativeCacheEntry {
    /// When we confirmed the peer was unreachable
    confirmed_at: Instant,
    /// How long to trust this negative result
    ttl: Duration,
    /// Number of times we've confirmed unreachability
    confirmation_count: u32,
}

impl NegativeCacheEntry {
    fn new(ttl: Duration) -> Self {
        NegativeCacheEntry {
            confirmed_at: Instant::now(),
            ttl,
            confirmation_count: 1,
        }
    }

    fn is_expired(&self) -> bool {
        self.confirmed_at.elapsed() > self.ttl
    }

    fn refresh(&mut self) {
        self.confirmed_at = Instant::now();
        self.confirmation_count += 1;
    }
}

/// Negative cache for peer unreachability
///
/// Combines a bloom filter for fast negative checks with a HashMap
/// for TTL tracking. The bloom filter provides O(1) negative answers,
/// while the HashMap ensures entries expire correctly.
#[derive(Debug, Clone)]
pub struct NegativeCache {
    /// Bloom filter for fast negative checks
    bloom: BloomFilter,
    /// Detailed entries with TTL tracking
    entries: HashMap<String, NegativeCacheEntry>,
    /// Default TTL for negative results
    default_ttl: Duration,
    /// Maximum number of entries before forced expiry
    max_entries: usize,
    /// Statistics
    stats: NegativeCacheStats,
}

/// Statistics for the negative cache
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct NegativeCacheStats {
    /// Total number of negative checks performed
    pub negative_checks: u64,
    /// Number of times bloom filter said "definitely unreachable"
    pub bloom_hits: u64,
    /// Number of times bloom filter said "might be reachable"
    pub bloom_misses: u64,
    /// Number of entries currently in cache
    pub entry_count: usize,
    /// Number of entries expired during cleanup
    pub expired_count: u64,
}

impl NegativeCache {
    /// Create a new negative cache
    pub fn new(default_ttl: Duration, max_entries: usize) -> Self {
        NegativeCache {
            bloom: BloomFilter::for_peer_cache(),
            entries: HashMap::new(),
            default_ttl,
            max_entries,
            stats: NegativeCacheStats::default(),
        }
    }

    /// Create with default settings (10 minute TTL, 10000 max entries)
    pub fn with_defaults() -> Self {
        Self::new(Duration::from_secs(600), 10000)
    }

    /// Check if we recently confirmed this peer is unreachable
    ///
    /// Returns true if we're CONFIDENT the peer is unreachable (fast path).
    /// Returns false if we DON'T KNOW or the peer might be reachable (slow path).
    pub fn is_definitely_unreachable(&mut self, peer_id: &str) -> bool {
        self.stats.negative_checks += 1;

        // First check bloom filter (fast)
        if !self.bloom.contains(peer_id) {
            self.stats.bloom_misses += 1;
            return false; // Definitely not in filter, so might be reachable
        }

        // Bloom says "might be unreachable", check detailed entries
        if let Some(entry) = self.entries.get(peer_id) {
            if !entry.is_expired() {
                self.stats.bloom_hits += 1;
                return true; // Confirmed unreachable and not expired
            }
            // Entry expired, will be cleaned up later
        }

        self.stats.bloom_misses += 1;
        false // Bloom false positive, entry expired or doesn't exist
    }

    /// Record that we confirmed a peer is unreachable
    pub fn record_unreachable(&mut self, peer_id: String) {
        // Insert into bloom filter
        self.bloom.insert(&peer_id);

        // Add or refresh detailed entry
        if let Some(entry) = self.entries.get_mut(&peer_id) {
            entry.refresh();
        } else {
            // Check capacity before adding
            if self.entries.len() >= self.max_entries {
                self.evict_oldest();
            }
            self.entries
                .insert(peer_id, NegativeCacheEntry::new(self.default_ttl));
        }
    }

    /// Record that a peer became reachable (clear negative)
    pub fn clear_unreachable(&mut self, peer_id: &str) {
        // Remove from detailed entries
        self.entries.remove(peer_id);

        // Note: Can't truly delete from bloom filter, but the entry will
        // expire naturally. We could rebuild the filter periodically.
    }

    /// Clean up expired entries
    ///
    /// Returns the number of entries removed.
    pub fn cleanup_expired(&mut self) -> usize {
        let before = self.entries.len();
        self.entries.retain(|_, entry| !entry.is_expired());
        let removed = before - self.entries.len();
        self.stats.expired_count += removed as u64;
        removed
    }

    /// Evict the oldest entry when at capacity
    fn evict_oldest(&mut self) {
        if let Some(oldest_key) = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.confirmed_at)
            .map(|(key, _)| key.clone())
        {
            self.entries.remove(&oldest_key);
        }
    }

    /// Get current statistics
    pub fn stats(&self) -> NegativeCacheStats {
        NegativeCacheStats {
            negative_checks: self.stats.negative_checks,
            bloom_hits: self.stats.bloom_hits,
            bloom_misses: self.stats.bloom_misses,
            entry_count: self.entries.len(),
            expired_count: self.stats.expired_count,
        }
    }

    /// Get the current false positive rate
    pub fn false_positive_rate(&self) -> f64 {
        self.bloom.false_positive_rate()
    }

    /// Get the number of entries in the cache
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.bloom.clear();
        self.entries.clear();
    }

    /// Prune entries whose confirmation count falls below the given threshold.
    /// Entries with fewer confirmations than `min_confirmations` are removed,
    /// treating low-confirmation entries as less reliable.
    pub fn prune_below_confidence(&mut self, min_confirmations: f64) -> usize {
        let threshold = min_confirmations.ceil() as u32;
        let before = self.entries.len();
        self.entries
            .retain(|_, entry| entry.confirmation_count >= threshold);
        let removed = before - self.entries.len();
        self.stats.expired_count += removed as u64;
        removed
    }

    /// Check if a peer should be exempted from negative caching based on reputation.
    /// High-reputation peers are less likely to be accurately marked as unreachable.
    ///
    /// # Arguments
    /// * `peer_id` - The peer ID to check
    /// * `reputation_score` - The overall reputation score from abuse detection
    ///   (0.0 = abusive, 1.0 = highly trusted)
    ///
    /// # Returns
    /// `true` if the peer should be exempted from negative cache (trusted),
    /// `false` if normal caching rules apply.
    pub fn should_exempt_from_negative_cache(&self, peer_id: &str, reputation_score: f64) -> bool {
        // A peer already negatively cached with repeated confirmations is a
        // recurring (not transient) failure: reputation does not mask it.
        if let Some(entry) = self.entries.get(peer_id) {
            if entry.confirmation_count >= 2 {
                return false;
            }
        }
        // Peers with overall_score >= 0.5 are considered trusted and exempted
        // This prevents marking good peers as unreachable due to transient issues
        reputation_score >= 0.5
    }

    /// Record unreachability with reputation-based exemption.
    /// High-reputation peers are exempted from negative caching.
    ///
    /// # Arguments
    /// * `peer_id` - The peer ID that was confirmed unreachable
    /// * `reputation_score` - The overall reputation score from abuse detection
    ///   (0.0 = abusive, 1.0 = highly trusted)
    ///
    /// # Returns
    /// `true` if the peer was added to the cache, `false` if exempted.
    pub fn record_unreachable_with_reputation(
        &mut self,
        peer_id: String,
        reputation_score: f64,
    ) -> bool {
        // Exempt high-reputation peers from negative caching
        if self.should_exempt_from_negative_cache(&peer_id, reputation_score) {
            return false;
        }
        self.record_unreachable(peer_id);
        true
    }
}

impl Default for NegativeCache {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bloom_filter_basic() {
        let mut bloom = BloomFilter::new(100, 0.01);

        // Insert some items
        bloom.insert("peer1");
        bloom.insert("peer2");
        bloom.insert("peer3");

        // Check they're in the filter
        assert!(bloom.contains("peer1"));
        assert!(bloom.contains("peer2"));
        assert!(bloom.contains("peer3"));

        // Check an item not in the filter
        // (might be false positive, but unlikely with small filter)
        assert!(!bloom.contains("peer999"));
    }

    #[test]
    fn test_bloom_filter_false_positive_rate() {
        let mut bloom = BloomFilter::new(100, 0.01);

        // Insert 100 items
        for i in 0..100 {
            bloom.insert(&format!("peer{}", i));
        }

        // Check false positive rate is reasonable
        let fp_rate = bloom.false_positive_rate();
        assert!(fp_rate < 0.05, "False positive rate too high: {}", fp_rate);
    }

    #[test]
    fn test_negative_cache_basic() {
        let mut cache = NegativeCache::new(Duration::from_secs(60), 100);

        // Initially, peer is not in cache
        assert!(!cache.is_definitely_unreachable("peer1"));

        // Record as unreachable
        cache.record_unreachable("peer1".to_string());

        // Now should be definitely unreachable
        assert!(cache.is_definitely_unreachable("peer1"));

        // Other peers should not be affected
        assert!(!cache.is_definitely_unreachable("peer2"));
    }

    #[test]
    fn test_negative_cache_expiry() {
        let mut cache = NegativeCache::new(Duration::from_millis(100), 100);

        cache.record_unreachable("peer1".to_string());
        assert!(cache.is_definitely_unreachable("peer1"));

        // Wait for expiry
        std::thread::sleep(Duration::from_millis(150));

        // Should no longer be definitely unreachable
        assert!(!cache.is_definitely_unreachable("peer1"));
    }

    #[test]
    fn test_negative_cache_clear() {
        let mut cache = NegativeCache::new(Duration::from_secs(60), 100);

        cache.record_unreachable("peer1".to_string());
        assert!(cache.is_definitely_unreachable("peer1"));

        cache.clear_unreachable("peer1");
        assert!(!cache.is_definitely_unreachable("peer1"));
    }

    #[test]
    fn test_negative_cache_cleanup() {
        let mut cache = NegativeCache::new(Duration::from_millis(50), 100);

        cache.record_unreachable("peer1".to_string());
        cache.record_unreachable("peer2".to_string());
        assert_eq!(cache.len(), 2);

        std::thread::sleep(Duration::from_millis(100));

        let removed = cache.cleanup_expired();
        assert_eq!(removed, 2);
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_negative_cache_capacity() {
        let mut cache = NegativeCache::new(Duration::from_secs(60), 3);

        cache.record_unreachable("peer1".to_string());
        cache.record_unreachable("peer2".to_string());
        cache.record_unreachable("peer3".to_string());
        assert_eq!(cache.len(), 3);

        // Adding a 4th should evict the oldest
        cache.record_unreachable("peer4".to_string());
        assert_eq!(cache.len(), 3);
    }

    #[test]
    fn test_negative_cache_stats() {
        let mut cache = NegativeCache::with_defaults();

        cache.record_unreachable("peer1".to_string());
        cache.record_unreachable("peer2".to_string());

        let _ = cache.is_definitely_unreachable("peer1");
        let _ = cache.is_definitely_unreachable("peer999");

        let stats = cache.stats();
        assert_eq!(stats.negative_checks, 2);
        assert_eq!(stats.entry_count, 2);
    }

    #[test]
    fn test_reputation_based_exemption() {
        let mut cache = NegativeCache::with_defaults();

        // High reputation peer (score = 0.8) should be exempted
        let exempted = cache.record_unreachable_with_reputation("good_peer".to_string(), 0.8);
        assert!(!exempted, "High reputation peer should be exempted");

        // Low reputation peer (score = 0.2) should be cached
        let cached = cache.record_unreachable_with_reputation("bad_peer".to_string(), 0.2);
        assert!(cached, "Low reputation peer should be cached");

        // Verify the bad_peer is in cache but good_peer is not
        assert!(cache.is_definitely_unreachable("bad_peer"));
        assert!(!cache.is_definitely_unreachable("good_peer"));
    }

    #[test]
    fn test_reputation_exemption_boundary() {
        let mut cache = NegativeCache::with_defaults();

        // Exactly at the boundary (0.5) should be exempted
        let exempted = cache.record_unreachable_with_reputation("borderline_peer".to_string(), 0.5);
        assert!(!exempted, "Peer at 0.5 threshold should be exempted");

        // Just below the boundary (0.49) should not be exempted
        let cached = cache.record_unreachable_with_reputation("just_below".to_string(), 0.49);
        assert!(cached, "Peer below 0.5 should be cached");
    }

    #[test]
    fn test_reputation_exemption_recurring_failures() {
        let mut cache = NegativeCache::with_defaults();

        // Two consecutive low-reputation failures create a cache entry
        // with confirmation_count == 2 (recurring confirmed unreachability)
        assert!(cache.record_unreachable_with_reputation("flaky_peer".to_string(), 0.4));
        assert!(cache.record_unreachable_with_reputation("flaky_peer".to_string(), 0.4));

        // Reputation later recovers, but recurring confirmations stand:
        // a peer with >= 2 confirmed failures is not masked by reputation.
        assert!(
            cache.record_unreachable_with_reputation("flaky_peer".to_string(), 0.9),
            "recurring confirmations must override the reputation exemption"
        );
        assert!(cache.is_definitely_unreachable("flaky_peer"));
    }
}
