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

use std::collections::{BTreeSet, HashMap};

/// Model parameter: largest future skew accepted on an envelope's `created_at`,
/// in seconds. Bounds honest clock disagreement between two devices; it is a
/// tolerance, not a capacity, so it has no derivation from resource state.
const ENVELOPE_MAX_FUTURE_SKEW_SECS: u32 = 300;
/// Oldest `created_at` that still earns a traffic bit: the default envelope
/// TTL, so anything older has expired on its own terms.
const ENVELOPE_MAX_AGE_SECS: u32 = crate::drift::envelope::DEFAULT_TTL_SECS;
/// Model parameter: most message IDs the replay record holds at once. At 16 bytes
/// of ID plus index overhead this is a few tens of MiB at worst. When full, no
/// new ID is recorded and none earns a traffic bit until expired entries are
/// pruned; a still-fresh recorded ID is never evicted to make room, because
/// evicting it would let a captured envelope be replayed to earn the bit again.
const RECENT_ENVELOPE_IDS_CAP: usize = 262_144;
/// Model parameter: the record is shared by at most this many full-quota
/// signers, so one saved contact can hold only `1 / SIGNER_QUOTA_DIVISOR` of it.
const SIGNER_QUOTA_DIVISOR: usize = 64;
/// Most replay IDs a single signer may hold at once, derived from the cap.
const SIGNER_IDS_CAP: usize = RECENT_ENVELOPE_IDS_CAP / SIGNER_QUOTA_DIVISOR;

/// Model parameter: envelope parses plus signature verifications one
/// authenticated peer may trigger on the swarm event loop per second. Past it
/// the request is still admitted (authenticated standing is unchanged) but is
/// not parsed, so it cannot earn or record anything at the gate.
const AUTHENTICATED_PARSES_PER_SEC: u32 = 8;
/// Model parameter: most peers the parse-budget table tracks. Entries older than
/// the current second are dropped when it fills; peers beyond it are not parsed.
const PARSE_BUDGET_PEERS_CAP: usize = 1_024;

type SignerKey = [u8; 32];

/// Message IDs that earned the contact-signed traffic bit, each kept until its
/// envelope falls out of the freshness window. A captured envelope replayed
/// while still fresh can never earn the bit again: only expiry removes an entry.
/// Capacity is bounded globally and per signer; at either bound new IDs are
/// refused rather than evicting a fresh one.
#[derive(Default)]
pub(super) struct RecentEnvelopeIds {
    /// Message ID -> (last unix second at which it is still fresh, signer).
    until: HashMap<[u8; 16], (u32, SignerKey)>,
    /// (Expiry second, ID) in time order, so pruning and eviction walk oldest first.
    by_expiry: BTreeSet<(u32, [u8; 16])>,
    /// Per-signer (expiry second, ID) index enforcing the per-signer quota.
    by_signer: HashMap<SignerKey, BTreeSet<(u32, [u8; 16])>>,
    /// Peer -> (unix second of the current window, parses used in it).
    parse_budget: HashMap<libp2p::PeerId, (u32, u32)>,
}

impl RecentEnvelopeIds {
    /// Records `id` from `signer` and returns true when it was not already in its
    /// window. An envelope stays fresh through `created_at + ENVELOPE_MAX_AGE_SECS`,
    /// subject to the exclusive `ttl_expiry`. Remember its ID through the last
    /// second it can pass. Returns false, recording nothing, when the signer is at
    /// its quota (that signer is quarantined until its oldest entry expires) or
    /// the record is at the global cap. A recorded, unexpired ID is never evicted.
    fn first_seen(
        &mut self,
        id: [u8; 16],
        signer: SignerKey,
        created_at: u32,
        ttl_expiry: u32,
        now: u32,
    ) -> bool {
        self.prune(now);
        if self.until.contains_key(&id) {
            return false;
        }
        let until = last_fresh_second(created_at, ttl_expiry);
        let signer_full = self
            .by_signer
            .get(&signer)
            .is_some_and(|ids| ids.len() >= SIGNER_IDS_CAP);
        if signer_full || self.until.len() >= RECENT_ENVELOPE_IDS_CAP {
            return false;
        }
        self.until.insert(id, (until, signer));
        self.by_expiry.insert((until, id));
        self.by_signer
            .entry(signer)
            .or_default()
            .insert((until, id));
        true
    }

    /// Charges one envelope parse to `peer` for second `now`; false when the
    /// peer has used its per-second budget or the table is full of live peers.
    fn charge_parse(&mut self, peer: libp2p::PeerId, now: u32) -> bool {
        if self.parse_budget.len() >= PARSE_BUDGET_PEERS_CAP {
            self.parse_budget.retain(|_, (window, _)| *window == now);
        }
        if self.parse_budget.len() >= PARSE_BUDGET_PEERS_CAP
            && !self.parse_budget.contains_key(&peer)
        {
            return false;
        }
        let entry = self.parse_budget.entry(peer).or_insert((now, 0));
        if entry.0 != now {
            *entry = (now, 0);
        }
        if entry.1 >= AUTHENTICATED_PARSES_PER_SEC {
            return false;
        }
        entry.1 += 1;
        true
    }

    /// Drops one ID from every index.
    fn remove(&mut self, id: &[u8; 16]) {
        let Some((until, signer)) = self.until.remove(id) else {
            return;
        };
        self.by_expiry.remove(&(until, *id));
        if let Some(ids) = self.by_signer.get_mut(&signer) {
            ids.remove(&(until, *id));
            if ids.is_empty() {
                self.by_signer.remove(&signer);
            }
        }
    }

    /// Forgets IDs whose envelopes can no longer pass the freshness check, which
    /// fails only once `now` is past `until`.
    fn prune(&mut self, now: u32) {
        while let Some((until, id)) = self.by_expiry.first().copied() {
            if until >= now {
                break;
            }
            self.remove(&id);
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
#[allow(clippy::too_many_arguments)]
fn contact_envelope_earns_traffic(
    message_id: [u8; 16],
    signer: SignerKey,
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
        && recent.first_seen(message_id, signer, created_at, ttl_expiry, now)
}

fn unix_now_secs() -> u32 {
    // A clock before the epoch yields 0, which fails the skew check closed.
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .ok()
        .and_then(|d| u32::try_from(d.as_secs()).ok())
        .unwrap_or(0)
}

/// Whether an authenticated peer may have this message parsed and verified now.
/// Rate-limits the event-loop cost an authenticated peer can impose per second.
pub(super) fn authenticated_parse_allowed(
    recent: &mut RecentEnvelopeIds,
    peer: libp2p::PeerId,
) -> bool {
    recent.charge_parse(peer, unix_now_secs())
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
        envelope.sender_public_key,
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
    const SIGNER: SignerKey = [1u8; 32];

    fn signer(n: u64) -> SignerKey {
        let mut out = [0u8; 32];
        out[..8].copy_from_slice(&n.to_be_bytes());
        out
    }

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
        assert!(recent.first_seen(id(0), SIGNER, NOW, 0, NOW));
        assert!(!recent.first_seen(id(0), SIGNER, NOW, 0, NOW + 1));
    }

    #[test]
    fn replay_rejected_after_thousands_of_other_ids_while_fresh() {
        let mut recent = RecentEnvelopeIds::default();
        assert!(contact_envelope_earns_traffic(
            id(0),
            SIGNER,
            NOW,
            0,
            true,
            true,
            NOW,
            &mut recent
        ));
        // Far more than the old 4,096-entry LRU could hold, all still in window.
        for n in 1..=5_000u64 {
            assert!(recent.first_seen(id(n), signer(100 + n % 8), NOW, 0, NOW + 1));
        }
        // The first envelope is still fresh, so its replay must not earn the bit.
        assert!(envelope_is_fresh(NOW, 0, NOW + 2));
        assert!(!contact_envelope_earns_traffic(
            id(0),
            SIGNER,
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
        assert!(recent.first_seen(id(1), SIGNER, NOW, 0, NOW));
        let last = NOW + ENVELOPE_MAX_AGE_SECS;
        assert!(envelope_is_fresh(NOW, 0, last));
        assert!(!recent.first_seen(id(1), SIGNER, NOW, 0, last));
        // One second later the envelope is stale, and its ID is forgotten.
        assert!(!envelope_is_fresh(NOW, 0, last + 1));
        recent.prune(last + 1);
        assert!(recent.until.is_empty());
        assert!(recent.by_expiry.is_empty());
        assert!(recent.by_signer.is_empty());
    }

    #[test]
    fn full_record_refuses_new_ids_and_never_evicts_a_fresh_one() {
        let mut recent = RecentEnvelopeIds::default();
        // Fill the record from many signers, each under its own quota.
        for n in 0..RECENT_ENVELOPE_IDS_CAP as u64 {
            assert!(recent.first_seen(id(n), signer(n % 128), NOW, NOW + 100, NOW));
        }
        assert_eq!(recent.until.len(), RECENT_ENVELOPE_IDS_CAP);
        // A new ID is refused while every recorded one is fresh.
        let new_id = id(RECENT_ENVELOPE_IDS_CAP as u64);
        assert!(!contact_envelope_earns_traffic(
            new_id,
            signer(9_999),
            NOW,
            NOW + 100,
            true,
            true,
            NOW,
            &mut recent
        ));
        assert_eq!(recent.until.len(), RECENT_ENVELOPE_IDS_CAP);
        assert!(!recent.until.contains_key(&new_id));
        // Every previously recorded ID is still recorded: no replay by eviction.
        for n in 0..RECENT_ENVELOPE_IDS_CAP as u64 {
            assert!(recent.until.contains_key(&id(n)));
        }
        assert!(!recent.first_seen(id(0), signer(0), NOW, NOW + 100, NOW));
        // Once the recorded entries expire the space is reclaimed.
        assert!(recent.first_seen(new_id, signer(9_999), NOW + 100, 0, NOW + 100));
        assert_eq!(recent.until.len(), 1);
    }

    #[test]
    fn alternating_replays_cannot_cycle_ids_through_a_full_signer_quota() {
        let mut recent = RecentEnvelopeIds::default();
        let hostile = signer(7);
        for n in 0..SIGNER_IDS_CAP as u64 {
            assert!(recent.first_seen(id(n), hostile, NOW, 0, NOW));
        }
        // E_a was never recorded; replaying it earns nothing and evicts nothing.
        let e_a = id(u64::MAX);
        assert!(!recent.first_seen(e_a, hostile, NOW, 0, NOW + 1));
        assert!(!recent.first_seen(id(0), hostile, NOW, 0, NOW + 1));
        assert!(!recent.first_seen(e_a, hostile, NOW, 0, NOW + 2));
        assert!(!recent.first_seen(id(0), hostile, NOW, 0, NOW + 2));
        assert_eq!(recent.until.len(), SIGNER_IDS_CAP);
    }

    #[test]
    fn one_signer_is_quarantined_at_its_quota_without_affecting_others() {
        let mut recent = RecentEnvelopeIds::default();
        let hostile = signer(7);
        for n in 0..SIGNER_IDS_CAP as u64 {
            assert!(recent.first_seen(id(n), hostile, NOW, 0, NOW));
        }
        assert!(!recent.first_seen(id(u64::MAX), hostile, NOW, 0, NOW));
        assert_eq!(
            recent.by_signer.get(&hostile).map(|ids| ids.len()),
            Some(SIGNER_IDS_CAP)
        );
        assert_eq!(recent.until.len(), SIGNER_IDS_CAP);
        // Another signer is unaffected.
        assert!(recent.first_seen(id(u64::MAX - 1), signer(8), NOW, 0, NOW));
        assert_eq!(recent.until.len(), SIGNER_IDS_CAP + 1);
        // After the hostile signer's entries expire it earns bits again.
        let later = NOW + ENVELOPE_MAX_AGE_SECS + 1;
        assert!(recent.first_seen(id(u64::MAX), hostile, later, 0, later));
    }

    #[test]
    fn parse_budget_limits_each_peer_per_second_and_resets_next_second() {
        let mut recent = RecentEnvelopeIds::default();
        let peer = libp2p::PeerId::random();
        let other = libp2p::PeerId::random();
        for _ in 0..AUTHENTICATED_PARSES_PER_SEC {
            assert!(recent.charge_parse(peer, NOW));
        }
        assert!(!recent.charge_parse(peer, NOW));
        assert!(recent.charge_parse(other, NOW));
        assert!(recent.charge_parse(peer, NOW + 1));
    }

    #[test]
    fn recent_ids_are_pruned_when_their_earlier_ttl_expires() {
        let mut recent = RecentEnvelopeIds::default();
        assert!(recent.first_seen(id(1), SIGNER, NOW, NOW + 2, NOW));
        assert_eq!(recent.until.get(&id(1)), Some(&(NOW + 1)));

        // TTL is exclusive, so the envelope is no longer fresh at NOW + 2.
        recent.prune(NOW + 2);
        assert!(recent.until.is_empty());
        assert!(recent.by_expiry.is_empty());
        assert!(recent.by_signer.is_empty());
        assert!(recent.first_seen(id(2), SIGNER, NOW + 2, NOW + 3, NOW + 2));
    }

    #[test]
    fn replayed_contact_envelope_earns_traffic_once() {
        let mut recent = RecentEnvelopeIds::default();
        assert!(contact_envelope_earns_traffic(
            id(7),
            SIGNER,
            NOW,
            0,
            true,
            true,
            NOW,
            &mut recent
        ));
        assert!(!contact_envelope_earns_traffic(
            id(7),
            SIGNER,
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
            SIGNER,
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
            SIGNER,
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
            SIGNER,
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
            SIGNER,
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
            SIGNER,
            NOW,
            0,
            true,
            true,
            NOW,
            &mut recent
        ));
    }
}
