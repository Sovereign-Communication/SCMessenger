//! Trust gate for contact-signed envelopes.
//!
//! Layout, one owner per concern:
//! - `swarm::inbound_message_is_validated` decides who may be probed. Blocked
//!   peers are refused, peers with authenticated history pass, and strangers
//!   need a probe allowance from admission.
//! - This module decides whether a verified envelope signed by a saved contact
//!   is fresh and not a replay, so that it earns the contact-signed traffic bit.
//! - `RecentEnvelopeIds` is the replay record. Each swarm event loop owns one,
//!   created with `Default` and passed by `&mut` into the gate.
//! - The age window is `drift::envelope::DEFAULT_TTL_SECS`, the single source for
//!   the envelope lifetime.

use std::collections::HashMap;

/// Largest future skew accepted on an envelope's `created_at`, in seconds.
const ENVELOPE_MAX_FUTURE_SKEW_SECS: u32 = 300;
/// Oldest `created_at` that still earns a traffic bit: the default envelope
/// TTL, so anything older has expired on its own terms.
const ENVELOPE_MAX_AGE_SECS: u32 = crate::drift::envelope::DEFAULT_TTL_SECS;
/// Most message IDs the replay record holds at once. Each ID is kept for its
/// whole freshness window, so this bounds memory without forgetting an ID that
/// can still pass. At the cap a new envelope is refused the traffic bit.
const RECENT_ENVELOPE_IDS_CAP: usize = 262_144;

/// Message IDs that earned the contact-signed traffic bit, each kept until its
/// envelope falls out of the freshness window. A captured envelope replayed
/// while still fresh cannot earn the bit again, however many other envelopes
/// have arrived in between.
#[derive(Default)]
pub(super) struct RecentEnvelopeIds {
    /// Message ID -> last unix second at which it is still fresh.
    until: HashMap<[u8; 16], u32>,
    /// Expiry second -> IDs expiring then, so pruning walks in time order.
    by_expiry: std::collections::BTreeMap<u32, Vec<[u8; 16]>>,
}

impl RecentEnvelopeIds {
    /// Records `id` and returns true when it was not already in its window.
    /// An envelope stays fresh through `created_at + ENVELOPE_MAX_AGE_SECS`, so
    /// its ID is remembered until then. Returns false, without recording, when
    /// the record is full.
    fn first_seen(&mut self, id: [u8; 16], created_at: u32, now: u32) -> bool {
        self.prune(now);
        if self.until.contains_key(&id) || self.until.len() >= RECENT_ENVELOPE_IDS_CAP {
            return false;
        }
        let until = created_at.saturating_add(ENVELOPE_MAX_AGE_SECS);
        self.until.insert(id, until);
        self.by_expiry.entry(until).or_default().push(id);
        true
    }

    /// Forgets IDs whose envelopes can no longer pass the freshness check, which
    /// fails only once `now` is past `until`.
    fn prune(&mut self, now: u32) {
        while let Some(entry) = self.by_expiry.first_entry() {
            if *entry.key() >= now {
                break;
            }
            for id in entry.remove() {
                self.until.remove(&id);
            }
        }
    }
}

/// Whether an envelope's timestamps are usable at `now` (unix seconds): not
/// dated beyond the skew window, not older than the TTL window, and not past
/// its own expiry (`ttl_expiry == 0` never expires).
fn envelope_is_fresh(created_at: u32, ttl_expiry: u32, now: u32) -> bool {
    if created_at > now.saturating_add(ENVELOPE_MAX_FUTURE_SKEW_SECS) {
        return false;
    }
    if now.saturating_sub(created_at) > ENVELOPE_MAX_AGE_SECS {
        return false;
    }
    ttl_expiry == 0 || ttl_expiry > now
}

/// Whether a signature-verified envelope earns the contact-signed traffic bit:
/// the signer must be a saved contact, the envelope must be fresh, and its
/// message ID must not have earned the bit recently. The replay record is
/// written last, so only envelopes that pass every other check fill it.
fn contact_envelope_earns_traffic(
    message_id: [u8; 16],
    created_at: u32,
    ttl_expiry: u32,
    signer_is_contact: bool,
    now: u32,
    recent: &mut RecentEnvelopeIds,
) -> bool {
    signer_is_contact
        && envelope_is_fresh(created_at, ttl_expiry, now)
        && recent.first_seen(message_id, created_at, now)
}

fn unix_now_secs() -> u32 {
    // A clock before the epoch yields 0, which fails the skew check closed.
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .ok()
        .and_then(|d| u32::try_from(d.as_secs()).ok())
        .unwrap_or(0)
}

/// True when the signer of a Drift envelope (signature verified) is a saved
/// contact, the envelope is fresh, and it is not a replay of one that already
/// earned the bit. A self-signed envelope from a stranger is not "a known
/// identity".
pub(super) fn contact_signed_envelope_earns_traffic(
    core: &crate::IronCore,
    envelope_data: &[u8],
    recent: &mut RecentEnvelopeIds,
) -> bool {
    let Ok(envelope) = crate::drift::DriftEnvelope::from_bytes(envelope_data) else {
        return false;
    };
    if envelope.verify().is_err() {
        return false;
    }
    let signer_is_contact = core
        .contact_manager
        .read()
        .get_by_public_key(&hex::encode(envelope.sender_public_key))
        .ok()
        .flatten()
        .is_some();
    contact_envelope_earns_traffic(
        envelope.message_id,
        envelope.created_at,
        envelope.ttl_expiry,
        signer_is_contact,
        unix_now_secs(),
        recent,
    )
}

#[cfg(test)]
mod contact_envelope_gate_tests {
    use super::*;

    const NOW: u32 = 1_800_000_000;

    fn id(n: u64) -> [u8; 16] {
        let mut out = [0u8; 16];
        out[..8].copy_from_slice(&n.to_be_bytes());
        out
    }

    #[test]
    fn freshness_rejects_future_skew_beyond_window() {
        assert!(envelope_is_fresh(
            NOW + ENVELOPE_MAX_FUTURE_SKEW_SECS,
            0,
            NOW
        ));
        assert!(!envelope_is_fresh(
            NOW + ENVELOPE_MAX_FUTURE_SKEW_SECS + 1,
            0,
            NOW
        ));
    }

    #[test]
    fn freshness_rejects_created_at_older_than_ttl_window() {
        assert!(envelope_is_fresh(NOW - ENVELOPE_MAX_AGE_SECS, 0, NOW));
        assert!(!envelope_is_fresh(NOW - ENVELOPE_MAX_AGE_SECS - 1, 0, NOW));
    }

    #[test]
    fn freshness_honours_own_expiry() {
        // ttl_expiry == 0 means no expiry; a future expiry is still live.
        assert!(envelope_is_fresh(NOW, 0, NOW));
        assert!(envelope_is_fresh(NOW, NOW + 1, NOW));
        // An expiry at or before now is expired.
        assert!(!envelope_is_fresh(NOW, NOW, NOW));
        assert!(!envelope_is_fresh(NOW - 10, NOW - 1, NOW));
    }

    #[test]
    fn recent_ids_reject_repeat_within_window() {
        let mut recent = RecentEnvelopeIds::default();
        assert!(recent.first_seen(id(0), NOW, NOW));
        assert!(!recent.first_seen(id(0), NOW, NOW + 1));
    }

    #[test]
    fn replay_rejected_after_thousands_of_other_ids_while_fresh() {
        let mut recent = RecentEnvelopeIds::default();
        assert!(contact_envelope_earns_traffic(
            id(0),
            NOW,
            0,
            true,
            NOW,
            &mut recent
        ));
        // Far more than the old 4,096-entry LRU could hold, all still in window.
        for n in 1..=5_000u64 {
            assert!(recent.first_seen(id(n), NOW, NOW + 1));
        }
        // The first envelope is still fresh, so its replay must not earn the bit.
        assert!(envelope_is_fresh(NOW, 0, NOW + 2));
        assert!(!contact_envelope_earns_traffic(
            id(0),
            NOW,
            0,
            true,
            NOW + 2,
            &mut recent
        ));
    }

    #[test]
    fn replay_stays_rejected_through_the_last_fresh_second() {
        let mut recent = RecentEnvelopeIds::default();
        assert!(recent.first_seen(id(1), NOW, NOW));
        let last = NOW + ENVELOPE_MAX_AGE_SECS;
        assert!(envelope_is_fresh(NOW, 0, last));
        assert!(!recent.first_seen(id(1), NOW, last));
        // One second later the envelope is stale, and its ID is forgotten.
        assert!(!envelope_is_fresh(NOW, 0, last + 1));
        recent.prune(last + 1);
        assert!(recent.until.is_empty());
        assert!(recent.by_expiry.is_empty());
    }

    #[test]
    fn full_record_refuses_new_ids_instead_of_forgetting() {
        let mut recent = RecentEnvelopeIds::default();
        for n in 0..RECENT_ENVELOPE_IDS_CAP as u64 {
            assert!(recent.first_seen(id(n), NOW, NOW));
        }
        assert_eq!(recent.until.len(), RECENT_ENVELOPE_IDS_CAP);
        // Full: a new envelope is refused, and an existing ID is still remembered.
        assert!(!recent.first_seen(id(RECENT_ENVELOPE_IDS_CAP as u64), NOW, NOW));
        assert!(!recent.first_seen(id(0), NOW, NOW));
        assert_eq!(recent.until.len(), RECENT_ENVELOPE_IDS_CAP);
    }

    #[test]
    fn replayed_contact_envelope_earns_traffic_once() {
        let mut recent = RecentEnvelopeIds::default();
        assert!(contact_envelope_earns_traffic(
            id(7),
            NOW,
            0,
            true,
            NOW,
            &mut recent
        ));
        assert!(!contact_envelope_earns_traffic(
            id(7),
            NOW,
            0,
            true,
            NOW,
            &mut recent
        ));
    }

    #[test]
    fn non_contact_or_stale_envelopes_do_not_fill_replay_record() {
        let mut recent = RecentEnvelopeIds::default();
        // A stranger-signed envelope earns nothing and is not recorded.
        assert!(!contact_envelope_earns_traffic(
            id(9),
            NOW,
            0,
            false,
            NOW,
            &mut recent
        ));
        assert!(recent.until.is_empty());
        // A stale envelope earns nothing and is not recorded.
        assert!(!contact_envelope_earns_traffic(
            id(9),
            NOW - ENVELOPE_MAX_AGE_SECS - 1,
            0,
            true,
            NOW,
            &mut recent
        ));
        assert!(recent.until.is_empty());
        // The same ID, once fresh and from a contact, still earns the bit.
        assert!(contact_envelope_earns_traffic(
            id(9),
            NOW,
            0,
            true,
            NOW,
            &mut recent
        ));
    }
}
