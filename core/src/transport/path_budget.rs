--- core/src/transport/path_budget.rs
+++ core/src/transport/path_basket.rs
@@
     /// Enforce the resource-derived bound across ALL peers by EVICTING the
     /// lowest-value paths (never by refusing a connection). Value, lowest first:
     ///
     /// 1. peer without authenticated history (not a saved contact, no positive
     ///    reputation): authenticated peers go last
     /// 2. no VALIDATED protocol traffic seen (paths that carry traffic go last)
     /// 3. still inside grace (an honest new path gets time to prove itself, but
     ///    only ahead of paths that already carry traffic)
     /// 4. redundant: the peer holds other live paths (sole paths go later)
     /// 5. lowest local reputation (unknown peers are lowest)
     /// 6. youngest first, so Sybil churn evicts itself
     ///
     /// Ranking is a hint. The gate is live: the last remaining path of a peer
     /// is evicted only if it is unauthenticated, carries no traffic, is past
     /// grace, and the peer did not start this call with several paths. Victim
     /// selection tracks each peer's remaining live paths as it evicts, so one
     /// call can never take an authenticated peer's last path, nor drop a peer
     /// that had several paths to zero.
     ///
     /// `keep` (the connection that triggered the check) is never chosen.
     ///
     /// Victims are found by partial selection (`select_nth_unstable_by`) in
     /// growing windows, so the cost is O(paths) plus O(k log k) for the k
     /// victims, not a full O(paths log paths) sort per call.
     pub fn evict_over_total(
         &mut self,
         limits: TotalLimits,
         now: Instant,
         keep: Option<I>,
     ) -> Vec<Eviction<C, I>> {
         let (live_total, live_fd) = self.occupancy();
         let mut excess_total = live_total.saturating_sub(limits.total);
         let mut excess_fd = live_fd.saturating_sub(limits.fd);
         if excess_total == 0 && excess_fd == 0 {
             return Vec::new();
         }
         #[derive(Clone, Copy)]
         struct Candidate<C, I> {
             peer: C,
             id: I,
             authenticated: bool,
             traffic: bool,
             protected: bool,
             sole: bool,
             /// Live paths the peer held when this call started.
             peer_live: usize,
             reputation: f64,
             established_at: Instant,
             uses_fd: bool,
         }
         fn rank<C, I>(a: &Candidate<C, I>, b: &Candidate<C, I>) -> std::cmp::Ordering {
             a.authenticated
                 .cmp(&b.authenticated)
                 .then(a.traffic.cmp(&b.traffic))
                 .then(a.protected.cmp(&b.protected))
                 .then(a.sole.cmp(&b.sole))
                 .then(a.reputation.total_cmp(&b.reputation))
                 .then(b.established_at.cmp(&a.established_at))
         }
         let mut candidates: Vec<Candidate<C, I>> = Vec::with_capacity(live_total);
         // Live paths per peer, decremented as victims are chosen in this call.
         let mut remaining: HashMap<C, usize> = HashMap::with_capacity(self.peers.len());
         for (peer, paths) in &self.peers {
             let live = paths.iter().filter(|p| p.closing.is_none()).count();
             remaining.insert(*peer, live);
             let trust = self.trust.get(peer).copied().unwrap_or_default();
             for path in paths.iter().filter(|p| p.closing.is_none()) {
                 if keep == Some(path.id) {
                     continue;
                 }
                 candidates.push(Candidate {
                     peer: *peer,
                     id: path.id,
                     authenticated: trust.authenticated,
                     traffic: path.traffic,
                     protected: now < path.established_at + path.grace,
                     sole: live <= 1,
                     peer_live: live,
                     reputation: trust.reputation,
                     established_at: path.established_at,
                     uses_fd: path.meta.uses_fd,
                 });
             }
         }
         let mut out = Vec::new();
         let mut rest: &mut [Candidate<C, I>] = &mut candidates;
         // First window: exactly the number of victims the excess asks for. If
         /// fd-only excess forces skipping non-fd paths the window doubles.
         let mut window = excess_total.saturating_add(excess_fd).max(1);
         while (excess_total > 0 || excess_fd > 0) && !rest.is_empty() {
             let k = window.min(rest.len());
             if k < rest.len() {
                 rest.select_nth_unstable_by(k - 1, rank);
             }
             rest[..k].sort_by(rank);
             let (head, tail) = std::mem::take(&mut rest).split_at_mut(k);
             for c in head.iter() {
                 if excess_total == 0 && excess_fd == 0 {
                     break;
                 }
                 let helps_fd = c.uses_fd && excess_fd > 0;
                 if excess_total == 0 && !helps_fd {
                     continue;
                 }
                 // Re-checked against live state, not the snapshot taken above:
                 // an earlier victim in this call may have been this peer's
                 // other path.
                 let left = remaining.get(&c.peer).copied().unwrap_or(0);
                 if left <= 1 {
                     continue;
                 }
                 let Some(path) = self
                     .peers
                     .get_mut(&c.peer)
                     .and_then(|paths| paths.iter_mut().find(|p| p.id == c.id))
                 else {
                     continue;
                 };
                 let due = mark_closing(&mut self.counts, path, now);
                 let class = path.class;
                 if let Some(n) = remaining.get_mut(&c.peer) {
                     *n = n.saturating_sub(1);
                 }
                 out.push(Eviction {
                     peer: c.peer,
                     id: c.id,
                     reason: EvictReason::OverTotal,
                     class,
                 });
                 self.hint_deadline(due, now);
                 excess_total = excess_total.saturating_sub(1);
                 if c.uses_fd {
                     excess_fd = excess_fd.saturating_sub(1);
                 }
             }
             rest = tail;
             window = window.saturating_mul(2);
         }
         out
     }
@@
                     Op::OverTotal(limit) => {
                         over_total = true;
                         let limit = usize::from(limit);
                         let before = ledger.occupancy().0;
                         let peer_before = ledger.live_paths(&PEER);
                         evictions = ledger.evict_over_total(
                             TotalLimits { total: limit, fd: usize::MAX },
                             now,
                             None,
                         );
                         // Never evicts past the excess; the soft bound may stay exceeded.
                         let after = ledger.occupancy().0
                         if before <= limit {
                             prop_assert_eq!(after, before);
                         } else {
                             prop_assert!(after >= limit && after <= before);
                         }
                         // The total bound never takes a multi-path peer to zero,
                         // and never takes an authenticated peer's last path.
                         let peer_after = ledger.live_paths(&PEER);
                         if peer_before >= 2 {
                             prop_assert!(peer_after >= 1, "multi-path peer emptied");
                         }
                         if authenticated && peer_before >= 1 {
                             prop_assert!(peer_after >= 1, "authenticated peer lost its last path");
                         }
                     }
                 }
                 let live_after = ledger.live_paths(&PEER);
                 // An authenticated peer never loses its last path to eviction,
                 // under any bound. Its own close is not an eviction. Unauthenticated,
                 // trafficless sole paths past grace may still be evicted by the
                 // total bound, by design.
                 if authenticated && live_before >= 1 && !is_close {
                     prop_assert!(live_after >= 1, "authenticated peer left with no live path");
                 }
                 // Per-peer evictions alone never take a peer from connected to
                 // zero paths (the total bound may: it evicts, by value).
                 if !evictions.is_empty() && !over_total {
                     prop_assert!(live_after >= 1, "peer left with no live path");
                     prop_assert!(live_before >= 1);
                 }
                 // The triggering connection is never its own victim.
                 if let Some(new_id) = established {
                     prop_assert!(evictions.iter().all(|e| e.id != new_id));
                 }
                 // Incremental counters always match a full recount.
                 prop_assert!(ledger.counters_consistent());
                 // Budget bookkeeping stays consistent.
                 let stats = ledger.stats(now);
                 prop_assert!(stats.budget <= stats.used);
                 prop_assert!(stats.budget >= stats.peers);
                 prop_assert!(stats.used >= stats.peers);
             }
         }
     }
+    #[test]
+    fn total_bound_eviction_respects_per_peer_floor() {
+        let base = Instant::now();
+        let mut ledger = Ledger::new();
+        let n_peers = 5;
+        // Establish one path per peer, all unauthenticated, no traffic, and make them past grace.
+        let mut peer_ids = Vec::new();
+        for i in 0..n_peers {
+            let peer = i as u32 + 100; // avoid collision with PEER in other tests
+            peer_ids.push(peer);
+            // Handshake duration: 1 second -> grace = 4 seconds.
+            let handshake = Duration::from_secs(1);
+            let now = at(base, 0);
+            ledger.on_established(peer, i, PathClass::WanV4, handshake, now);
+        }
+        // Advance clock to be past grace: 5 seconds after establishment.
+        let past_grace = at(base, 5_000); // 5 seconds in milliseconds
+        // Now, set a total bound that is one less than the number of peers.
+        let limits = TotalLimits { total: n_peers - 1, fd: usize::MAX };
+        let ev = ledger.evict_over_total(limits, past_grace, None);
+        // We expect no evictions because each peer has only one path and we cannot evict the last one.
+        assert_eq!(ev.len(), 0, "no evictions should occur: {ev:?}");
+        // Each peer should still have exactly one live path.
+        for &peer in &peer_ids {
+            assert_eq!(ledger.live_paths(&peer), 1, "peer {peer} should have one live path");
+        }
+        // Total occupancy should be n_peers (which is above the total bound of n_peers-1).
+        assert_eq!(ledger.occupancy().0, n_peers);
+    }