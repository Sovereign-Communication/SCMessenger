//! Log-silence heartbeat watchdog -- the exit decision, as a value.
//!
//! The decision used to exist only inside an async task in `main.rs` whose only
//! observable effect was `std::process::exit`. There was therefore no way to
//! assert the case the 2026-09-17 multi-dimensional audit flagged as N-03: a
//! node that is quiet but healthy must not be killed. Ticket
//! `HANDOFF/freebuff/queue/V040_T_WATCHDOG_POSITIVE_TEST.md` requires that
//! positive test, and instructs (item 3) that if the decision is not
//! unit-testable the pure predicate comes out and gets tested instead. This
//! module is that extraction.
//!
//! Behaviour is UNCHANGED from the inline version, deliberately:
//!
//! - a fresh log reading at or under the threshold clears the streak;
//! - an unreadable/missing heartbeat (`None`) clears the streak -- a missing log
//!   directory only exists pre-init, so it is not evidence of a wedge;
//! - ONE silence reading above the threshold warns and records a diagnostic;
//! - TWO CONSECUTIVE silence readings above the threshold exit the process.
//!
//! The two-reading rule is load-bearing and predates this extraction: a single
//! bad measurement must never kill a healthy node. Observed live 2026-09-15,
//! when enumeration-cached metadata made the first version read 659s of
//! "silence" against logs written one second earlier. Tuning the threshold is an
//! operator decision (`SCM_LOG_SILENCE_TIMEOUT_SECS`), not a test knob.

use std::time::Duration;

/// Smallest poll interval the watchdog will use, whatever the threshold is.
pub const MIN_POLL_SECS: u64 = 5;

/// Consecutive silence readings required before the node exits.
pub const SILENCE_READINGS_BEFORE_EXIT: u32 = 2;

/// What one heartbeat reading means for the node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogSilenceVerdict {
    /// Heartbeat fresh (or unreadable): nothing to do, streak cleared.
    Healthy,
    /// One silence reading above the threshold: warn, require a second.
    Warning { age_secs: u64, consecutive: u32 },
    /// Enough consecutive silence readings: exit so a restart can recover.
    Exit { age_secs: u64, consecutive: u32 },
}

/// Poll cadence: one tenth of the threshold, floored at [`MIN_POLL_SECS`], so
/// detection latency stays proportional to the configured timeout.
pub fn poll_interval(threshold_secs: u64) -> Duration {
    Duration::from_secs((threshold_secs / 10).max(MIN_POLL_SECS))
}

/// The watchdog's state: a threshold plus the current consecutive-silence run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogSilenceWatchdog {
    threshold_secs: u64,
    consecutive_silence: u32,
}

impl LogSilenceWatchdog {
    /// Build a watchdog that exits after [`SILENCE_READINGS_BEFORE_EXIT`]
    /// consecutive readings above `threshold_secs`.
    pub fn new(threshold_secs: u64) -> Self {
        Self {
            threshold_secs,
            consecutive_silence: 0,
        }
    }

    /// The configured silence threshold, in seconds.
    pub fn threshold_secs(&self) -> u64 {
        self.threshold_secs
    }

    /// How many consecutive silence readings have been seen so far.
    pub fn consecutive_silence(&self) -> u32 {
        self.consecutive_silence
    }

    /// Feed one heartbeat reading; `None` means it could not be read.
    pub fn observe(&mut self, age_secs: Option<u64>) -> LogSilenceVerdict {
        let Some(age) = age_secs else {
            self.consecutive_silence = 0;
            return LogSilenceVerdict::Healthy;
        };
        if age <= self.threshold_secs {
            self.consecutive_silence = 0;
            return LogSilenceVerdict::Healthy;
        }
        self.consecutive_silence = self.consecutive_silence.saturating_add(1);
        if self.consecutive_silence >= SILENCE_READINGS_BEFORE_EXIT {
            LogSilenceVerdict::Exit {
                age_secs: age,
                consecutive: self.consecutive_silence,
            }
        } else {
            LogSilenceVerdict::Warning {
                age_secs: age,
                consecutive: self.consecutive_silence,
            }
        }
    }
}

impl LogSilenceVerdict {
    /// Whether this verdict means the process should exit.
    ///
    /// Named so the watchdog loop reads as intent rather than pattern matching,
    /// and so tests can assert "did not exit" without matching every field.
    pub fn is_exit(self) -> bool {
        matches!(self, LogSilenceVerdict::Exit { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const THRESHOLD: u64 = 600;

    /// The positive case N-03 asked for: a node that keeps emitting its periodic
    /// liveness line is never killed, however long it runs and however little
    /// else it has to say. The relay-custody audit alone logs every 60s even on
    /// an idle node, so this is what "quiet but healthy" looks like on the wire.
    #[test]
    fn wp4_quiet_but_logging_node_is_never_killed() {
        let mut watchdog = LogSilenceWatchdog::new(THRESHOLD);
        let poll = poll_interval(THRESHOLD).as_secs();
        let mut age = 0u64;
        for reading in 0..600 {
            // One liveness line per poll interval: the age stays at or under the
            // threshold forever.
            age = (age + poll).min(THRESHOLD);
            assert_eq!(
                watchdog.observe(Some(age)),
                LogSilenceVerdict::Healthy,
                "reading {reading} at age {age}s must not arm an exit"
            );
        }
        assert_eq!(watchdog.consecutive_silence(), 0);
    }

    #[test]
    fn wp4_fresh_heartbeat_clears_a_warning_streak() {
        let mut watchdog = LogSilenceWatchdog::new(THRESHOLD);
        assert!(matches!(
            watchdog.observe(Some(THRESHOLD + 1)),
            LogSilenceVerdict::Warning { consecutive: 1, .. }
        ));
        // A single fresh line must disarm the pending exit.
        assert_eq!(watchdog.observe(Some(1)), LogSilenceVerdict::Healthy);
        assert_eq!(watchdog.consecutive_silence(), 0);
        // So the next silence reading is a first reading again, not a second.
        assert!(matches!(
            watchdog.observe(Some(THRESHOLD + 5)),
            LogSilenceVerdict::Warning { .. }
        ));
    }

    #[test]
    fn wp4_unreadable_heartbeat_is_not_a_wedge() {
        let mut watchdog = LogSilenceWatchdog::new(THRESHOLD);
        assert_eq!(watchdog.observe(None), LogSilenceVerdict::Healthy);
        assert_eq!(watchdog.consecutive_silence(), 0);
        // Even after a warning, an unreadable then readable-quiet sequence must
        // not accumulate towards exit.
        assert!(matches!(
            watchdog.observe(Some(THRESHOLD + 1)),
            LogSilenceVerdict::Warning { .. }
        ));
        assert_eq!(watchdog.observe(None), LogSilenceVerdict::Healthy);
        assert_eq!(
            watchdog.observe(Some(THRESHOLD)),
            LogSilenceVerdict::Healthy
        );
    }

    #[test]
    fn wp4_age_exactly_at_threshold_is_healthy() {
        let mut watchdog = LogSilenceWatchdog::new(THRESHOLD);
        assert_eq!(
            watchdog.observe(Some(THRESHOLD)),
            LogSilenceVerdict::Healthy,
            "the threshold is exclusive: age == threshold is not yet silence"
        );
        assert_eq!(watchdog.observe(Some(THRESHOLD + 1)).is_exit(), false);
    }

    #[test]
    fn wp4_one_reading_never_exits_and_two_consecutive_do() {
        let mut watchdog = LogSilenceWatchdog::new(THRESHOLD);
        let first = watchdog.observe(Some(THRESHOLD + 400));
        assert_eq!(
            first,
            LogSilenceVerdict::Warning {
                age_secs: THRESHOLD + 400,
                consecutive: 1
            },
            "a single silence reading must only warn"
        );
        let second = watchdog.observe(Some(THRESHOLD + 460));
        assert_eq!(
            second,
            LogSilenceVerdict::Exit {
                age_secs: THRESHOLD + 460,
                consecutive: 2
            },
            "the second consecutive silence reading exits"
        );
    }

    #[test]
    fn wp4_poll_interval_tracks_the_threshold() {
        assert_eq!(poll_interval(600).as_secs(), 60);
        assert_eq!(poll_interval(120).as_secs(), 12);
        // Floored, so a tiny test threshold cannot spin.
        assert_eq!(poll_interval(10).as_secs(), MIN_POLL_SECS);
        assert_eq!(poll_interval(0).as_secs(), MIN_POLL_SECS);
    }
}
