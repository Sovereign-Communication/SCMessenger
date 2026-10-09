//! Resource-derived total connection bound (no static caps).
//!
//! The per-peer budget in [`super::path_budget`] says how many paths ONE
//! identity needs. It cannot stop N identities from holding one path each, so
//! the total across all peers is bounded by what the machine can actually
//! carry, derived from live measurements:
//!
//! ```text
//! B_fd    = share_fd  x RLIMIT_NOFILE (soft)              -- paths that own a descriptor
//! pool    = available_memory + used x per_connection_RSS  -- what we could hold if we held none
//! B_mem   = max(active, share_mem x pool / per_connection_RSS)
//! total   = min(B_mem, B_fd + descriptor-free paths) x platform_scale
//! ```
//!
//! * The memory pool adds our own connections' footprint back to the free
//!   memory, so admitting a connection (which lowers free memory) does not
//!   lower the bound by itself; the bound only falls when something ELSE takes
//!   memory. The memory-derived bound never goes below the number of paths that
//!   carry traffic (`active`): a predicted shortage idles the idle paths first
//!   and never cuts a conversation. Actual allocation failures (pressure
//!   feedback below) are the hard signal and are not floored.
//! * `per_connection_RSS` is MEASURED: the process RSS delta observed while the
//!   live connection count grew, smoothed by an EWMA. It starts from a
//!   conservative prior so a cold node never over-admits.
//! * QUIC multiplexes every connection over one UDP socket and a relay circuit
//!   rides a substream, so only the memory bound applies to them.
//! * The platform scale shrinks the bound on battery (see
//!   [`set_platform_power`]): every kept-alive path costs radio wakeups.
//! * Pressure feedback (EMFILE/ENFILE/ENOMEM/ENOBUFS from dial, accept or
//!   listener errors) shrinks the bound at once, but per EPISODE (one tick),
//!   not per error event: a raw-socket flood raises hundreds of errors per
//!   poll and must not translate into hundreds of evictions. Each episode
//!   retains [`PRESSURE_RETAIN_NUM`]/[`PRESSURE_RETAIN_DEN`] of the live paths
//!   (multiplicative shrink) and never goes below the paths worth protecting
//!   (authenticated peers and paths with validated traffic): established
//!   authenticated peers are never evicted to make room for unauthenticated
//!   sockets. The next periodic sample restores the derived figure. When the
//!   probe fails under descriptor exhaustion the last good snapshot is kept.
//!
//! Above the bound the ledger EVICTS the lowest-value paths; it never refuses
//! a connection. Where no input is available (a platform without a memory
//! probe and without a descriptor limit) no bound can be derived, so none is
//! enforced and only pressure feedback applies.
//!
//! # Platform notes
//!
//! * Linux / Android: `getrlimit`, `/proc/meminfo` (`MemAvailable`, clamped by
//!   the cgroup v2 headroom when one is set), `/proc/self/statm`.
//! * Windows has no per-process descriptor soft limit. Sockets are kernel
//!   handles bounded by the 2^24-entry handle table; that figure is used as
//!   the descriptor limit, so in practice the memory bound
//!   (`GlobalMemoryStatusEx`, working set via `K32GetProcessMemoryInfo`)
//!   governs.
//! * macOS / iOS: `getrlimit` only (256 by default on iOS, which is exactly
//!   the scarce resource there); memory probes are not implemented, so the
//!   descriptor bound is used as the total bound as well.

use super::path_budget::TotalLimits;
use std::sync::atomic::{AtomicU32, Ordering};

/// Fraction of the soft descriptor limit connections may use. The rest is
/// needed by sled, log files, listeners, DNS and the host application.
pub const FD_SHARE: f64 = 0.5;

/// Fraction of AVAILABLE memory connections may use. The rest is headroom for
/// the application and the OS page cache pressure it already causes.
pub const MEMORY_SHARE: f64 = 0.5;

/// Conservative prior for the resident cost of one connection, used until
/// RSS deltas have been measured. The review estimated 50-150 KB with about a
/// dozen handlers; 256 KiB is above that range (also covering the Noise,
/// yamux and per-stream buffers), so a cold node errs towards a smaller bound.
pub const PRIOR_CONNECTION_BYTES: f64 = 256.0 * 1024.0;

/// One RSS observation may not claim more than this many priors per
/// connection, so a one-off allocation (a sled flush, a message burst) that
/// happens to coincide with a new connection cannot collapse the bound.
pub const OBSERVATION_CEILING_PRIORS: f64 = 4.0;

/// The estimate never drops below this fraction of the prior: allocator reuse
/// can make a connection look free in a delta, but it is never free.
pub const ESTIMATE_FLOOR_FRACTION: f64 = 0.125;

/// EWMA weight of a new per-connection RSS observation. A model parameter,
/// not a bound: noisy deltas (other allocations ride along) get a quarter
/// weight so the estimate follows sustained change but not single samples.
pub const RSS_SMOOTHING: f64 = 0.25;

/// Fraction of the live paths one pressure episode retains (3/4). A model
/// parameter, not a cap: repeated episodes compound, so sustained exhaustion
/// converges quickly on what the machine can carry, while one burst of errors
/// (however many events it holds) costs at most a quarter of the paths, and
/// never those worth protecting.
pub const PRESSURE_RETAIN_NUM: usize = 3;
/// Denominator of [`PRESSURE_RETAIN_NUM`].
pub const PRESSURE_RETAIN_DEN: usize = 4;

/// Connection-count growths with zero measured RSS growth tolerated before the
/// per-connection estimate starts decaying towards its floor. Allocator reuse
/// can hide one or two connections' cost; a sustained run of free
/// connections is evidence the estimate is too high (for example after a
/// one-off spike inflated it).
pub const ZERO_GROWTH_PATIENCE: u32 = 3;

/// Windows handle-table maximum, used as its descriptor limit (see module docs).
pub const WINDOWS_HANDLE_LIMIT: u64 = 1 << 24;

/// How pressure was detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressureKind {
    /// Out of file descriptors / handles (EMFILE, ENFILE).
    Fd,
    /// Out of memory or socket buffers (ENOMEM, ENOBUFS).
    Mem,
}

/// Which input currently binds the total.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetSource {
    Fd,
    Mem,
    Platform,
}

impl BudgetSource {
    /// Marker spelling.
    pub fn as_str(&self) -> &'static str {
        match self {
            BudgetSource::Fd => "fd",
            BudgetSource::Mem => "mem",
            BudgetSource::Platform => "platform",
        }
    }
}

/// `[CONN] total_budget=<B> used=<n> source=<fd|mem|platform>`
pub fn format_total_marker(total: usize, used: usize, source: BudgetSource) -> String {
    format!(
        "[CONN] total_budget={} used={} source={}",
        total,
        used,
        source.as_str()
    )
}

/// One reading of the machine. Every field is optional: a platform that cannot
/// supply a figure leaves it `None` and the bound is derived from the rest.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResourceSnapshot {
    /// Soft limit on open descriptors (RLIMIT_NOFILE).
    pub fd_soft_limit: Option<u64>,
    /// Memory that can be committed without pushing the machine into reclaim.
    pub mem_available: Option<u64>,
    /// Resident set size of this process.
    pub rss: Option<u64>,
}

impl ResourceSnapshot {
    /// This reading, with every field the probe failed to supply taken from
    /// `last`. Under descriptor exhaustion the `/proc` reads themselves fail
    /// (they need a descriptor); losing the memory figures at exactly that
    /// moment would drop the bound to the pressure cap alone, so the last good
    /// figure stands in until a probe succeeds again.
    pub fn or_last(self, last: &ResourceSnapshot) -> ResourceSnapshot {
        ResourceSnapshot {
            fd_soft_limit: self.fd_soft_limit.or(last.fd_soft_limit),
            mem_available: self.mem_available.or(last.mem_available),
            rss: self.rss.or(last.rss),
        }
    }
}

/// A derived bound and the input that binds it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Derived {
    pub limits: TotalLimits,
    pub source: BudgetSource,
}

#[derive(Debug, Clone, Copy)]
struct Pressure {
    total: usize,
    fd: Option<usize>,
    source: PressureKind,
}

/// Live occupancy fed to [`ResourceModel::derive`].
#[derive(Debug, Clone, Copy)]
pub struct Occupancy {
    /// Live paths of every kind.
    pub total: usize,
    /// Live paths that own a descriptor.
    pub fd: usize,
    /// Live paths that have carried validated protocol traffic.
    pub active: usize,
    /// Live paths worth protecting under pressure (authenticated peers, or
    /// validated traffic). Only filled for pressure episodes; zero otherwise.
    pub protected: usize,
    /// The protected paths that own a descriptor.
    pub protected_fd: usize,
}

/// Measured per-connection cost plus pressure feedback.
#[derive(Debug)]
pub struct ResourceModel {
    per_connection: f64,
    last: Option<(u64, usize)>,
    /// Consecutive connection-count growths that showed no RSS growth.
    flat_growths: u32,
    pressure: Option<Pressure>,
}

impl Default for ResourceModel {
    fn default() -> Self {
        Self {
            per_connection: PRIOR_CONNECTION_BYTES,
            last: None,
            flat_growths: 0,
            pressure: None,
        }
    }
}

impl ResourceModel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Current estimate of the resident cost of one connection.
    pub fn per_connection_bytes(&self) -> f64 {
        self.per_connection
            .max(PRIOR_CONNECTION_BYTES * ESTIMATE_FLOOR_FRACTION)
    }

    /// Feed a fresh RSS reading together with the live connection count. Only
    /// growth in the connection count is informative: RSS rarely shrinks when
    /// a connection closes, so shrinking steps are ignored.
    pub fn observe(&mut self, rss: Option<u64>, live: usize) {
        let Some(rss) = rss else {
            return;
        };
        if let Some((prev_rss, prev_live)) = self.last {
            if live > prev_live {
                let added = (live - prev_live) as f64;
                let grown = rss.saturating_sub(prev_rss) as f64;
                let floor = PRIOR_CONNECTION_BYTES * ESTIMATE_FLOOR_FRACTION;
                if grown > 0.0 {
                    let sample = (grown / added)
                        .clamp(floor, PRIOR_CONNECTION_BYTES * OBSERVATION_CEILING_PRIORS);
                    self.per_connection =
                        self.per_connection * (1.0 - RSS_SMOOTHING) + sample * RSS_SMOOTHING;
                    self.flat_growths = 0;
                } else {
                    // Connections grew and RSS did not. One or two such steps
                    // are allocator reuse; a sustained run means the estimate
                    // is too high, so it decays towards its floor.
                    self.flat_growths = self.flat_growths.saturating_add(1);
                    if self.flat_growths >= ZERO_GROWTH_PATIENCE {
                        self.per_connection =
                            self.per_connection * (1.0 - RSS_SMOOTHING) + floor * RSS_SMOOTHING;
                    }
                }
            }
        }
        self.last = Some((rss, live));
    }

    /// The OS refused allocations/accepts/dials during one tick (ONE episode,
    /// however many error events it held). Retain
    /// [`PRESSURE_RETAIN_NUM`]/[`PRESSURE_RETAIN_DEN`] of the live paths, but
    /// never fewer than the paths worth protecting (`occupancy.protected`):
    /// pressure sheds unauthenticated, traffic-less paths first and never
    /// authenticated peers. Repeated episodes compound; the cap lasts until the
    /// next periodic sample ([`ResourceModel::clear_pressure`]).
    pub fn note_pressure(&mut self, kind: PressureKind, occupancy: Occupancy) {
        let total = retain_share(occupancy.total)
            .max(occupancy.protected)
            .max(1);
        let fd = match kind {
            PressureKind::Fd => Some(
                retain_share(occupancy.fd)
                    .max(occupancy.protected_fd)
                    .max(1),
            ),
            PressureKind::Mem => None,
        };
        self.pressure = Some(match self.pressure {
            Some(old) => Pressure {
                total: old.total.min(total).max(occupancy.protected).max(1),
                fd: match (old.fd, fd) {
                    (Some(a), Some(b)) => Some(a.min(b).max(occupancy.protected_fd).max(1)),
                    (a, b) => a.or(b),
                },
                source: kind,
            },
            None => Pressure {
                total,
                fd,
                source: kind,
            },
        });
    }

    /// Drop the pressure cap (called when a fresh periodic sample arrives).
    pub fn clear_pressure(&mut self) {
        self.pressure = None;
    }

    /// True while a pressure cap is in force.
    pub fn under_pressure(&self) -> bool {
        self.pressure.is_some()
    }

    /// Derive the bound from the latest snapshot, the live occupancy and the
    /// platform scale (permille, 1000 = unscaled). `None` when nothing at all
    /// can be derived.
    pub fn derive(
        &self,
        snapshot: &ResourceSnapshot,
        occupancy: Occupancy,
        scale_permille: u32,
    ) -> Option<Derived> {
        let descriptor_free = occupancy.total.saturating_sub(occupancy.fd);
        let b_fd = snapshot
            .fd_soft_limit
            .map(|limit| to_usize((limit as f64 * FD_SHARE) as u64).max(1));
        let b_mem = snapshot.mem_available.map(|avail| {
            let per = self.per_connection_bytes();
            let pool = occupancy.total as f64 * per + avail as f64;
            to_usize((pool * MEMORY_SHARE / per) as u64).max(occupancy.active)
        });

        let mut derived = match (b_fd, b_mem) {
            (Some(fd), Some(mem)) => {
                let fd_equivalent = fd.saturating_add(descriptor_free);
                if mem < fd_equivalent {
                    Some((mem, fd, BudgetSource::Mem))
                } else {
                    Some((fd_equivalent, fd, BudgetSource::Fd))
                }
            }
            // No memory probe: the descriptor bound stands in for every path.
            (Some(fd), None) => Some((fd, fd, BudgetSource::Fd)),
            (None, Some(mem)) => Some((mem, usize::MAX, BudgetSource::Mem)),
            (None, None) => None,
        };

        if let Some((total, fd, source)) = derived.as_mut() {
            if scale_permille < 1000 {
                let scaled_total = scale(*total, scale_permille);
                if scaled_total < *total {
                    *total = scaled_total;
                    *source = BudgetSource::Platform;
                }
                if *fd != usize::MAX {
                    *fd = scale(*fd, scale_permille);
                }
            }
        }

        if let Some(pressure) = self.pressure {
            let (mut total, mut fd, mut source) = derived.unwrap_or((
                usize::MAX,
                usize::MAX,
                match pressure.source {
                    PressureKind::Fd => BudgetSource::Fd,
                    PressureKind::Mem => BudgetSource::Mem,
                },
            ));
            if pressure.total < total {
                total = pressure.total;
                source = match pressure.source {
                    PressureKind::Fd => BudgetSource::Fd,
                    PressureKind::Mem => BudgetSource::Mem,
                };
            }
            if let Some(cap) = pressure.fd {
                fd = fd.min(cap);
            }
            derived = Some((total, fd, source));
        }

        derived.map(|(total, fd, source)| Derived {
            limits: TotalLimits {
                total: total.max(1),
                fd: fd.max(1),
            },
            source,
        })
    }
}

/// One pressure episode's retained share of `live` paths (rounded up, but an
/// episode always sheds at least one path while more than one is live, so small
/// counts converge too).
pub fn retain_share(live: usize) -> usize {
    live.saturating_mul(PRESSURE_RETAIN_NUM)
        .div_ceil(PRESSURE_RETAIN_DEN)
        .min(live.saturating_sub(1).max(1))
}

fn scale(value: usize, permille: u32) -> usize {
    let scaled = (value as u128 * u128::from(permille)) / 1000;
    to_usize(u64::try_from(scaled).unwrap_or(u64::MAX)).max(1)
}

fn to_usize(value: u64) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

// ---------------------------------------------------------------------------
// Platform power scale
// ---------------------------------------------------------------------------

static PLATFORM_SCALE_PERMILLE: AtomicU32 = AtomicU32::new(1000);

/// Scale factor (permille) for a device power state. A charging device pays
/// nothing extra for a kept-alive path. An unplugged one pays radio wakeups
/// for every path, so the bound scales linearly from one half (empty) to the
/// full figure (full battery); it never drops below half because the paths
/// carry the user's messages.
pub fn scale_for_power(battery_pct: u8, charging: bool) -> u32 {
    if charging {
        1000
    } else {
        500 + 5 * u32::from(battery_pct.min(100))
    }
}

/// Platform hook: the mobile bridge reports battery state here.
pub fn set_platform_power(battery_pct: u8, charging: bool) {
    PLATFORM_SCALE_PERMILLE.store(scale_for_power(battery_pct, charging), Ordering::Relaxed);
}

/// Current platform scale in permille (1000 = unscaled).
pub fn platform_scale_permille() -> u32 {
    PLATFORM_SCALE_PERMILLE.load(Ordering::Relaxed)
}

// ---------------------------------------------------------------------------
// Probes
// ---------------------------------------------------------------------------

/// Read the machine now.
pub fn sample_resources() -> ResourceSnapshot {
    ResourceSnapshot {
        fd_soft_limit: probe::fd_soft_limit(),
        mem_available: probe::mem_available(),
        rss: probe::rss(),
    }
}

/// `MemAvailable` from `/proc/meminfo` text, in bytes.
#[cfg_attr(not(any(target_os = "linux", target_os = "android")), allow(dead_code))]
pub(crate) fn parse_meminfo_available(text: &str) -> Option<u64> {
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("MemAvailable:") {
            let kib: u64 = rest.trim().trim_end_matches("kB").trim().parse().ok()?;
            return Some(kib.saturating_mul(1024));
        }
    }
    None
}

/// Resident pages (second field) from `/proc/self/statm` text.
#[cfg_attr(not(any(target_os = "linux", target_os = "android")), allow(dead_code))]
pub(crate) fn parse_statm_resident_pages(text: &str) -> Option<u64> {
    text.split_whitespace().nth(1)?.parse().ok()
}

/// Headroom under a cgroup v2 limit from the text of `memory.max` and
/// `memory.current`. `None` when the group is unlimited or unparsable.
#[cfg_attr(not(any(target_os = "linux", target_os = "android")), allow(dead_code))]
pub(crate) fn parse_cgroup_headroom(max: &str, current: &str) -> Option<u64> {
    let max = max.trim();
    if max == "max" {
        return None;
    }
    let max: u64 = max.parse().ok()?;
    let current: u64 = current.trim().parse().ok()?;
    Some(max.saturating_sub(current))
}

#[cfg(any(target_os = "linux", target_os = "android"))]
mod probe {
    use super::{parse_cgroup_headroom, parse_meminfo_available, parse_statm_resident_pages};

    pub fn fd_soft_limit() -> Option<u64> {
        super::unix_fd_soft_limit()
    }

    pub fn mem_available() -> Option<u64> {
        let meminfo = std::fs::read_to_string("/proc/meminfo").ok()?;
        let host = parse_meminfo_available(&meminfo)?;
        let cgroup = std::fs::read_to_string("/sys/fs/cgroup/memory.max")
            .ok()
            .zip(std::fs::read_to_string("/sys/fs/cgroup/memory.current").ok())
            .and_then(|(max, current)| parse_cgroup_headroom(&max, &current));
        Some(cgroup.map_or(host, |headroom| headroom.min(host)))
    }

    pub fn rss() -> Option<u64> {
        let statm = std::fs::read_to_string("/proc/self/statm").ok()?;
        let pages = parse_statm_resident_pages(&statm)?;
        // SAFETY: sysconf takes an integer name and returns a plain integer;
        // it has no memory-safety preconditions.
        let page = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
        let page = u64::try_from(page).ok().filter(|p| *p > 0)?;
        Some(pages.saturating_mul(page))
    }
}

#[cfg(all(
    unix,
    not(any(target_os = "linux", target_os = "android")),
    not(target_arch = "wasm32")
))]
mod probe {
    pub fn fd_soft_limit() -> Option<u64> {
        super::unix_fd_soft_limit()
    }

    pub fn mem_available() -> Option<u64> {
        None
    }

    pub fn rss() -> Option<u64> {
        None
    }
}

#[cfg(windows)]
mod probe {
    #[repr(C)]
    #[allow(dead_code)] // mirrors the Win32 layout; not every field is read
    struct MemoryStatusEx {
        length: u32,
        memory_load: u32,
        total_phys: u64,
        avail_phys: u64,
        total_page_file: u64,
        avail_page_file: u64,
        total_virtual: u64,
        avail_virtual: u64,
        avail_extended_virtual: u64,
    }

    #[repr(C)]
    #[allow(dead_code)] // mirrors the Win32 layout; not every field is read
    struct ProcessMemoryCounters {
        cb: u32,
        page_fault_count: u32,
        peak_working_set_size: usize,
        working_set_size: usize,
        quota_peak_paged_pool_usage: usize,
        quota_paged_pool_usage: usize,
        quota_peak_non_paged_pool_usage: usize,
        quota_non_paged_pool_usage: usize,
        pagefile_usage: usize,
        peak_pagefile_usage: usize,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GlobalMemoryStatusEx(buffer: *mut MemoryStatusEx) -> i32;
        fn GetCurrentProcess() -> isize;
        fn K32GetProcessMemoryInfo(
            process: isize,
            counters: *mut ProcessMemoryCounters,
            cb: u32,
        ) -> i32;
    }

    pub fn fd_soft_limit() -> Option<u64> {
        Some(super::WINDOWS_HANDLE_LIMIT)
    }

    pub fn mem_available() -> Option<u64> {
        let mut status = MemoryStatusEx {
            length: std::mem::size_of::<MemoryStatusEx>() as u32,
            memory_load: 0,
            total_phys: 0,
            avail_phys: 0,
            total_page_file: 0,
            avail_page_file: 0,
            total_virtual: 0,
            avail_virtual: 0,
            avail_extended_virtual: 0,
        };
        // SAFETY: `status` is a live, exclusively borrowed repr(C) struct that
        // matches MEMORYSTATUSEX, with `length` initialised as the API requires.
        let ok = unsafe { GlobalMemoryStatusEx(&mut status) };
        (ok != 0).then_some(status.avail_phys)
    }

    pub fn rss() -> Option<u64> {
        let mut counters = ProcessMemoryCounters {
            cb: std::mem::size_of::<ProcessMemoryCounters>() as u32,
            page_fault_count: 0,
            peak_working_set_size: 0,
            working_set_size: 0,
            quota_peak_paged_pool_usage: 0,
            quota_paged_pool_usage: 0,
            quota_peak_non_paged_pool_usage: 0,
            quota_non_paged_pool_usage: 0,
            pagefile_usage: 0,
            peak_pagefile_usage: 0,
        };
        // SAFETY: GetCurrentProcess returns a pseudo-handle that needs no
        // closing; `counters` is a live, exclusively borrowed repr(C) struct
        // matching PROCESS_MEMORY_COUNTERS with `cb` set to its size.
        let ok = unsafe {
            K32GetProcessMemoryInfo(
                GetCurrentProcess(),
                &mut counters,
                std::mem::size_of::<ProcessMemoryCounters>() as u32,
            )
        };
        (ok != 0).then_some(counters.working_set_size as u64)
    }
}

#[cfg(not(any(unix, windows)))]
mod probe {
    pub fn fd_soft_limit() -> Option<u64> {
        None
    }

    pub fn mem_available() -> Option<u64> {
        None
    }

    pub fn rss() -> Option<u64> {
        None
    }
}

/// Soft RLIMIT_NOFILE on unix; `None` when unlimited or unreadable.
#[cfg(unix)]
#[allow(clippy::unnecessary_cast)]
fn unix_fd_soft_limit() -> Option<u64> {
    let mut limit = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    // SAFETY: `limit` is a live, exclusively borrowed rlimit; getrlimit only
    // writes through the pointer and has no other preconditions.
    let rc = unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut limit) };
    if rc != 0 || limit.rlim_cur == libc::RLIM_INFINITY {
        return None;
    }
    Some(limit.rlim_cur as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    const GIB: u64 = 1024 * 1024 * 1024;

    fn occ(total: usize, fd: usize) -> Occupancy {
        Occupancy {
            total,
            fd,
            active: 0,
            protected: 0,
            protected_fd: 0,
        }
    }

    #[test]
    fn fd_bound_is_a_share_of_the_soft_limit() {
        let model = ResourceModel::new();
        let snap = ResourceSnapshot {
            fd_soft_limit: Some(1024),
            mem_available: None,
            rss: None,
        };
        let d = model.derive(&snap, occ(0, 0), 1000).expect("derived");
        assert_eq!(d.limits.total, 512);
        assert_eq!(d.limits.fd, 512);
        assert_eq!(d.source, BudgetSource::Fd);
    }

    #[test]
    fn memory_bound_uses_available_memory_over_measured_cost() {
        let model = ResourceModel::new();
        // Pool = 1 GiB free + 100 connections x 256 KiB; half of it is usable.
        let snap = ResourceSnapshot {
            fd_soft_limit: None,
            mem_available: Some(GIB),
            rss: None,
        };
        let d = model.derive(&snap, occ(100, 100), 1000).expect("derived");
        assert_eq!(d.limits.total, 50 + 2048);
        assert_eq!(d.source, BudgetSource::Mem);
        assert_eq!(d.limits.fd, usize::MAX);
    }

    #[test]
    fn quic_paths_do_not_consume_the_descriptor_bound() {
        let model = ResourceModel::new();
        let snap = ResourceSnapshot {
            fd_soft_limit: Some(200), // 100 descriptor-owning paths
            mem_available: Some(64 * GIB),
            rss: None,
        };
        // 500 live paths of which only 40 own a descriptor: the descriptor
        // limit still allows 100 of them, plus every QUIC path.
        let d = model.derive(&snap, occ(500, 40), 1000).expect("derived");
        assert_eq!(d.limits.fd, 100);
        assert_eq!(d.source, BudgetSource::Fd);
        assert_eq!(
            d.limits.total,
            100 + 460,
            "fd bound + descriptor-free paths"
        );
    }

    #[test]
    fn admitting_connections_alone_never_lowers_the_memory_bound() {
        let model = ResourceModel::new();
        let per = model.per_connection_bytes() as u64;
        let pool = 4 * GIB;
        // The same machine, with 0 / 500 / 5000 of OUR connections resident:
        // free memory falls by exactly their footprint, the bound does not move.
        let bound_at = |used: u64| {
            let snap = ResourceSnapshot {
                fd_soft_limit: None,
                mem_available: Some(pool - used * per),
                rss: None,
            };
            model
                .derive(&snap, occ(used as usize, used as usize), 1000)
                .expect("derived")
                .limits
                .total
        };
        assert_eq!(bound_at(0), bound_at(500));
        assert_eq!(bound_at(0), bound_at(5000));
    }

    #[test]
    fn memory_taken_by_something_else_lowers_the_bound_but_never_cuts_active_paths() {
        let model = ResourceModel::new();
        let snap = ResourceSnapshot {
            fd_soft_limit: None,
            mem_available: Some(0),
            rss: None,
        };
        // Memory is gone: only half the pool (our own footprint) is usable.
        let idle = model.derive(&snap, occ(100, 100), 1000).expect("derived");
        assert_eq!(idle.limits.total, 50);
        assert_eq!(idle.source, BudgetSource::Mem);
        // 80 of the 100 paths carry traffic: they are the floor.
        let busy = Occupancy {
            total: 100,
            fd: 100,
            active: 80,
            protected: 80,
            protected_fd: 80,
        };
        let floored = model.derive(&snap, busy, 1000).expect("derived");
        assert_eq!(floored.limits.total, 80);
    }

    #[test]
    fn the_smaller_of_memory_and_descriptors_binds() {
        let model = ResourceModel::new();
        let snap = ResourceSnapshot {
            fd_soft_limit: Some(1_000_000),
            mem_available: Some(16 * 1024 * 1024), // 8 MiB usable = 32 conns
            rss: None,
        };
        let d = model.derive(&snap, occ(0, 0), 1000).expect("derived");
        assert_eq!(d.limits.total, 32);
        assert_eq!(d.source, BudgetSource::Mem);
    }

    #[test]
    fn nothing_to_derive_from_means_no_bound() {
        let model = ResourceModel::new();
        assert!(model
            .derive(&ResourceSnapshot::default(), occ(5, 5), 1000)
            .is_none());
    }

    #[test]
    fn rss_growth_per_new_connection_is_measured_and_smoothed() {
        let mut model = ResourceModel::new();
        let prior = model.per_connection_bytes();
        // First reading establishes the reference point only.
        model.observe(Some(100 * 1024 * 1024), 10);
        assert_eq!(model.per_connection_bytes(), prior);
        // 10 new connections cost 1 MiB total = 100 KiB each (below the prior).
        model.observe(Some(101 * 1024 * 1024), 20);
        let after = model.per_connection_bytes();
        assert!(after < prior, "{after} should fall below the prior {prior}");
        assert!(after > 100.0 * 1024.0, "smoothing keeps part of the prior");
        // Many consistent observations converge on the measured cost.
        let mut rss = 101 * 1024 * 1024u64;
        let mut live = 20usize;
        for _ in 0..60 {
            rss += 100 * 1024 * 10;
            live += 10;
            model.observe(Some(rss), live);
        }
        let converged = model.per_connection_bytes();
        assert!(
            (converged - 100.0 * 1024.0).abs() < 4.0 * 1024.0,
            "{converged}"
        );
    }

    #[test]
    fn a_one_off_allocation_cannot_collapse_the_bound() {
        let mut model = ResourceModel::new();
        model.observe(Some(100 * 1024 * 1024), 10);
        // One connection coincides with a 500 MiB allocation spike.
        model.observe(Some(600 * 1024 * 1024), 11);
        let est = model.per_connection_bytes();
        assert!(
            est <= PRIOR_CONNECTION_BYTES * (1.0 + OBSERVATION_CEILING_PRIORS),
            "{est}"
        );
        assert!(
            est <= PRIOR_CONNECTION_BYTES * 2.0,
            "one sample moves it by a quarter only"
        );
    }

    #[test]
    fn shrinking_or_flat_rss_does_not_lower_the_estimate_to_zero() {
        let mut model = ResourceModel::new();
        model.observe(Some(100 * 1024 * 1024), 10);
        model.observe(Some(100 * 1024 * 1024), 20); // no growth: ignored
        model.observe(Some(90 * 1024 * 1024), 30); // shrink: ignored
        assert_eq!(model.per_connection_bytes(), PRIOR_CONNECTION_BYTES);
    }

    #[test]
    fn a_sustained_run_of_free_connections_decays_an_inflated_estimate() {
        let mut model = ResourceModel::new();
        let mut rss = 100 * 1024 * 1024u64;
        let mut live = 10usize;
        model.observe(Some(rss), live);
        // A burst of expensive connections inflates the estimate above the prior.
        for _ in 0..8 {
            rss += 4 * PRIOR_CONNECTION_BYTES as u64 * 10;
            live += 10;
            model.observe(Some(rss), live);
        }
        let inflated = model.per_connection_bytes();
        assert!(inflated > 2.0 * PRIOR_CONNECTION_BYTES, "{inflated}");
        // Then connections keep arriving at no measurable cost.
        for _ in 0..40 {
            live += 10;
            model.observe(Some(rss), live);
        }
        let decayed = model.per_connection_bytes();
        assert!(
            decayed < inflated / 2.0,
            "{decayed} should decay from {inflated}"
        );
        assert!(
            decayed >= PRIOR_CONNECTION_BYTES * ESTIMATE_FLOOR_FRACTION,
            "never below the floor"
        );
    }

    #[test]
    fn a_failed_probe_keeps_the_last_good_figures() {
        let good = ResourceSnapshot {
            fd_soft_limit: Some(1024),
            mem_available: Some(GIB),
            rss: Some(100 * 1024 * 1024),
        };
        // Under EMFILE the /proc reads fail; getrlimit still answers.
        let degraded = ResourceSnapshot {
            fd_soft_limit: Some(1024),
            mem_available: None,
            rss: None,
        };
        assert_eq!(degraded.or_last(&good), good);
        let fresh = ResourceSnapshot {
            fd_soft_limit: Some(2048),
            mem_available: Some(2 * GIB),
            rss: Some(1),
        };
        assert_eq!(fresh.or_last(&good), fresh, "a successful probe wins");
    }

    #[test]
    fn pressure_is_per_episode_multiplicative_and_floored_at_protected_paths() {
        let mut model = ResourceModel::new();
        let snap = ResourceSnapshot {
            fd_soft_limit: Some(10_000),
            mem_available: None,
            rss: None,
        };
        let normal = model.derive(&snap, occ(100, 100), 1000).expect("derived");
        assert_eq!(normal.limits.total, 5_000);
        // One episode (however many error events it held) keeps 3/4.
        model.note_pressure(PressureKind::Fd, occ(100, 100));
        assert!(model.under_pressure());
        let shrunk = model.derive(&snap, occ(100, 100), 1000).expect("derived");
        assert_eq!(shrunk.limits.total, 75);
        assert_eq!(shrunk.limits.fd, 75);
        assert_eq!(shrunk.source, BudgetSource::Fd);
        // Sustained pressure compounds but stops at the protected floor.
        let mut live = 75usize;
        for _ in 0..20 {
            let now = Occupancy {
                protected: 40,
                protected_fd: 40,
                ..occ(live, live)
            };
            model.note_pressure(PressureKind::Fd, now);
            live = model
                .derive(&snap, now, 1000)
                .expect("derived")
                .limits
                .total
                .min(live);
        }
        assert_eq!(live, 40, "authenticated paths are never cut");
        model.clear_pressure();
        let back = model.derive(&snap, occ(40, 40), 1000).expect("derived");
        assert_eq!(back.limits.total, 5_000, "the periodic sample restores it");
    }

    #[test]
    fn memory_pressure_applies_even_without_any_probe() {
        let mut model = ResourceModel::new();
        model.note_pressure(PressureKind::Mem, occ(40, 10));
        let d = model
            .derive(&ResourceSnapshot::default(), occ(40, 10), 1000)
            .expect("pressure alone yields a bound");
        assert_eq!(d.limits.total, 30);
        assert_eq!(d.limits.fd, usize::MAX, "memory pressure leaves fd alone");
        assert_eq!(d.source, BudgetSource::Mem);
    }

    #[test]
    fn pressure_never_drops_the_bound_below_one_path() {
        let mut model = ResourceModel::new();
        let mut live = 3usize;
        for _ in 0..50 {
            model.note_pressure(PressureKind::Fd, occ(live, live));
            live = model
                .derive(&ResourceSnapshot::default(), occ(live, live), 1000)
                .expect("derived")
                .limits
                .total
                .min(live);
        }
        assert_eq!(live, 1);
    }

    #[test]
    fn platform_scale_shrinks_the_bound_on_battery() {
        let model = ResourceModel::new();
        let snap = ResourceSnapshot {
            fd_soft_limit: Some(2000),
            mem_available: None,
            rss: None,
        };
        let full = model.derive(&snap, occ(0, 0), 1000).expect("derived");
        assert_eq!(full.limits.total, 1000);
        let low = model
            .derive(&snap, occ(0, 0), scale_for_power(0, false))
            .expect("derived");
        assert_eq!(low.limits.total, 500);
        assert_eq!(low.source, BudgetSource::Platform);
    }

    #[test]
    fn power_scale_is_linear_and_never_below_half() {
        assert_eq!(scale_for_power(100, true), 1000);
        assert_eq!(scale_for_power(5, true), 1000);
        assert_eq!(scale_for_power(100, false), 1000);
        assert_eq!(scale_for_power(0, false), 500);
        assert_eq!(scale_for_power(50, false), 750);
        assert_eq!(scale_for_power(250, false), 1000, "clamped to 100 percent");
    }

    #[test]
    fn platform_hook_round_trips_through_the_atomic() {
        set_platform_power(20, false);
        assert_eq!(platform_scale_permille(), 600);
        set_platform_power(20, true);
        assert_eq!(platform_scale_permille(), 1000);
    }

    #[test]
    fn marker_uses_the_documented_spelling() {
        assert_eq!(
            format_total_marker(512, 40, BudgetSource::Fd),
            "[CONN] total_budget=512 used=40 source=fd"
        );
        assert_eq!(BudgetSource::Mem.as_str(), "mem");
        assert_eq!(BudgetSource::Platform.as_str(), "platform");
    }

    #[test]
    fn proc_parsers_read_the_documented_formats() {
        let meminfo = "MemTotal:       16384000 kB\nMemFree:         1000000 kB\nMemAvailable:    8192000 kB\n";
        assert_eq!(parse_meminfo_available(meminfo), Some(8_192_000 * 1024));
        assert_eq!(parse_meminfo_available("MemTotal: 1 kB\n"), None);
        assert_eq!(
            parse_statm_resident_pages("5000 1234 300 1 0 800 0"),
            Some(1234)
        );
        assert_eq!(parse_statm_resident_pages("5000"), None);
        assert_eq!(parse_cgroup_headroom("max\n", "123\n"), None);
        assert_eq!(parse_cgroup_headroom("1000\n", "400\n"), Some(600));
        assert_eq!(parse_cgroup_headroom("1000\n", "4000\n"), Some(0));
        assert_eq!(parse_cgroup_headroom("junk", "1"), None);
    }

    #[cfg(unix)]
    #[test]
    fn the_descriptor_limit_is_readable_on_unix() {
        let snap = sample_resources();
        assert!(snap.fd_soft_limit.is_some_and(|limit| limit > 0));
    }

    #[cfg(any(target_os = "linux", target_os = "android", windows))]
    #[test]
    fn memory_and_rss_probes_work_on_this_platform() {
        let snap = sample_resources();
        assert!(snap.mem_available.is_some_and(|bytes| bytes > 0));
        assert!(snap.rss.is_some_and(|bytes| bytes > 0));
    }
}
