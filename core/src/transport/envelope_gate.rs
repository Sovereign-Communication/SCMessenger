//! Trust gate for contact-signed envelopes.
//!
//! Layout, one owner per concern:
//! - `swarm::inbound_message_is_validated` decides who may be probed. Blocked
//!   peers are refused, peers with authenticated history pass, and strangers
//!   need a probe allowance from admission.
//! - This module decides whether a verified envelope signed by a saved contact
//!   is addressed to this identity, fresh, and not a replay, so that it earns
//!   the contact-signed traffic bit.
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
/// whole freshness window. At the cap, contact-signed traffic awards pause
/// until every envelope observed without a replay record has expired.
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
    /// Last fresh second among IDs observed while the replay record was full.
    /// No unrecorded ID may earn a traffic bit until this second has passed.
    saturated_until: Option<u32>,
}

impl RecentEnvelopeIds {
    /// Records `id` and returns true when it was not already in its window.
    /// An envelope stays fresh through `created_at + ENVELOPE_MAX_AGE_SECS`,
    /// subject to the exclusive `ttl_expiry`. Remember its ID through the last
    /// second it can pass. If the record is full, the ID cannot be recorded;
    /// all new IDs are then refused until the freshness windows of every
    /// unrecorded ID observed during saturation have passed.
    fn first_seen(&mut self, id: [u8; 16], created_at: u32, ttl_expiry: u32, now: u32) -> bool {
        self.prune(now);
        if self.until.contains_key(&id) {
            return false;
        }
        let until = last_fresh_second(created_at, ttl_expiry);
        if let Some(saturated_until) = self.saturated_until.filter(|until| *until >= now) {
            self.saturated_until = Some(saturated_until.max(until));
            return false;
        }
        if self.until.len() >= RECENT_ENVELOPE_IDS_CAP {
            self.saturated_until = Some(until);
            return false;
        }
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
        if self.saturated_until.is_some_and(|until| until < now) {
            self.saturated_until = None;
        }
    }
}

/// Last unix second at which an envelope can pass `envelope_is_fresh`.
/// `ttl_expiry` is exclusive, while the age limit is inclusive.
fn last_fresh_second(created_at: u32, ttl_expiry: u32) -> u32 {
    let age_expiry = created_at.saturating_add(ENVELOPE_MAX_AGE_SECS);
    if ttl_expiry == 0 {
        age_expiry
    } else {
        age_expiry.min(ttl_expiry.saturating_sub(1))
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
/// the signer must be a saved contact, the envelope must target this identity
/// and be fresh, and its message ID must not have earned the bit recently. The
/// replay record is written last, so only envelopes that pass every other check
/// fill it.
fn contact_envelope_earns_traffic(
    message_id: [u8; 16],
    created_at: u32,
    ttl_expiry: u32,
    signer_is_contact: bool,
    recipient_matches: bool,
    now: u32,
    recent: &mut RecentEnvelopeIds,
) -> bool {
    signer_is_contact
        && recipient_matches
        && envelope_is_fresh(created_at, ttl_expiry, now)
        && recent.first_seen(message_id, created_at, ttl_expiry, now)
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
/// contact, the envelope targets this identity and is fresh, and it is not a
/// replay of one that already earned the bit. A self-signed envelope from a
/// stranger is not "a known identity".
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
    let Some(local_public_key) = core
        .identity
        .read()
        .keys()
        .map(|keys| keys.signing_key.verifying_key().to_bytes())
    else {
        return false;
    };
    let recipient_matches = envelope.recipient_hint
        == crate::drift::DriftEnvelope::hint_from_public_key(&local_public_key);
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
        recipient_matches,
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
        assert!(recent.first_seen(id(0), NOW, 0, NOW));
        assert!(!recent.first_seen(id(0), NOW, 0, NOW + 1));
    }

    #[test]
    fn replay_rejected_after_thousands_of_other_ids_while_fresh() {
        let mut recent = RecentEnvelopeIds::default();
        assert!(contact_envelope_earns_traffic(
            id(0),
            NOW,
            0,
            true,
            true,
            NOW,
            &mut recent
        ));
        // Far more than the old 4,096-entry LRU could hold, all still in window.
        for n in 1..=5_000u64 {
            assert!(recent.first_seen(id(n), NOW, 0, NOW + 1));
        }
        // The first envelope is still fresh, so its replay must not earn the bit.
        assert!(envelope_is_fresh(NOW, 0, NOW + 2));
        assert!(!contact_envelope_earns_traffic(
            id(0),
            NOW,
            0,
            true,
            true,
            NOW + 2,
            &mut recent
        ));
    }

    #[test]
    fn replay_stays_rejected_through_the_last_fresh_second() {
        let mut recent = RecentEnvelopeIds::default();
        assert!(recent.first_seen(id(1), NOW, 0, NOW));
        let last = NOW + ENVELOPE_MAX_AGE_SECS;
        assert!(envelope_is_fresh(NOW, 0, last));
        assert!(!recent.first_seen(id(1), NOW, 0, last));
        // One second later the envelope is stale, and its ID is forgotten.
        assert!(!envelope_is_fresh(NOW, 0, last + 1));
        recent.prune(last + 1);
        assert!(recent.until.is_empty());
        assert!(recent.by_expiry.is_empty());
    }

    #[test]
    fn saturation_quarantines_unrecorded_ids_until_their_freshness_windows_end() {
        let mut recent = RecentEnvelopeIds::default();
        for n in 0..RECENT_ENVELOPE_IDS_CAP as u64 {
            assert!(recent.first_seen(id(n), NOW, NOW + 2, NOW));
        }
        assert_eq!(recent.until.len(), RECENT_ENVELOPE_IDS_CAP);
        let unrecorded_id = id(RECENT_ENVELOPE_IDS_CAP as u64);
        // Full: refuse this still-fresh envelope and remember its last-fresh
        // second in the bounded overflow quarantine.
        assert!(!contact_envelope_earns_traffic(
            unrecorded_id,
            NOW,
            NOW + 100,
            true,
            true,
            NOW,
            &mut recent
        ));
        assert_eq!(recent.saturated_until, Some(NOW + 99));

        // The recorded IDs expire and free the map, but a replay of the
        // unrecorded envelope cannot earn during its original freshness window.
        assert!(!contact_envelope_earns_traffic(
            unrecorded_id,
            NOW,
            NOW + 100,
            true,
            true,
            NOW + 2,
            &mut recent
        ));
        assert!(recent.until.is_empty());
        assert_eq!(recent.saturated_until, Some(NOW + 99));

        // Unrecorded IDs observed during quarantine extend it to cover their
        // own freshness windows, including bounded future clock skew.
        let later_created_at = NOW + ENVELOPE_MAX_FUTURE_SKEW_SECS;
        let later_last_fresh = last_fresh_second(later_created_at, 0);
        assert!(!contact_envelope_earns_traffic(
            id(RECENT_ENVELOPE_IDS_CAP as u64 + 1),
            later_created_at,
            0,
            true,
            true,
            NOW + 2,
            &mut recent
        ));
        assert_eq!(recent.saturated_until, Some(later_last_fresh));
        recent.prune(later_last_fresh);
        assert_eq!(recent.saturated_until, Some(later_last_fresh));
        recent.prune(later_last_fresh + 1);
        assert_eq!(recent.saturated_until, None);

        // Once every unrecorded ID is stale, fresh IDs can earn again.
        assert!(recent.first_seen(
            id(RECENT_ENVELOPE_IDS_CAP as u64 + 2),
            later_last_fresh + 1,
            0,
            later_last_fresh + 1
        ));
        assert_eq!(recent.until.len(), 1);
    }

    #[test]
    fn recent_ids_are_pruned_when_their_earlier_ttl_expires() {
        let mut recent = RecentEnvelopeIds::default();
        assert!(recent.first_seen(id(1), NOW, NOW + 2, NOW));
        assert_eq!(recent.until.get(&id(1)), Some(&(NOW + 1)));

        // TTL is exclusive, so the envelope is no longer fresh at NOW + 2.
        recent.prune(NOW + 2);
        assert!(recent.until.is_empty());
        assert!(recent.by_expiry.is_empty());
        assert!(recent.first_seen(id(2), NOW + 2, NOW + 3, NOW + 2));
    }

    #[test]
    fn replayed_contact_envelope_earns_traffic_once() {
        let mut recent = RecentEnvelopeIds::default();
        assert!(contact_envelope_earns_traffic(
            id(7),
            NOW,
            0,
            true,
            true,
            NOW,
            &mut recent
        ));
        assert!(!contact_envelope_earns_traffic(
            id(7),
            NOW,
            0,
            true,
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
            true,
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
            true,
            NOW,
            &mut recent
        ));
    }

    #[test]
    fn wrong_recipient_hint_does_not_earn_or_consume_replay_id() {
        let mut recent = RecentEnvelopeIds::default();
        assert!(!contact_envelope_earns_traffic(
            id(10),
            NOW,
            0,
            true,
            false,
            NOW,
            &mut recent
        ));
        assert!(recent.until.is_empty());
        assert!(contact_envelope_earns_traffic(
            id(10),
            NOW,
            0,
            true,
            true,
            NOW,
            &mut recent
        ));
    }
}
