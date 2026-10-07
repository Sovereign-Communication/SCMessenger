// OS signals feeding the discovery scheduler (#469, Rule-8 review of #484,
// findings F2 and F3).
//
// F2: network-change notifications.
//   - Windows: `NotifyUnicastIpAddressChange` (IP Helper) -- the OS calls us
//     when any unicast address is added, removed or changed.
//   - Linux: a minimal `NETLINK_ROUTE` socket subscribed to link and address
//     groups, read on a dedicated blocking thread.
//   - Other platforms (macOS, BSD): no notification source is wired yet
//     (SCDynamicStore would need a new dependency). The caller falls back to a
//     poll whose interval is computed from the live scheduler state, see
//     `SeedDialClient::poll_interval`. Tracked in a follow-up issue.
//
// F3: power state. Derived from the platform where it is cheap (Linux
// `/sys/class/power_supply`, Windows `GetSystemPowerStatus`). When the
// platform exposes no battery (desktop, server, container) the computed
// default is `Charging`: a machine with no battery is on mains power.

use scmessenger_core::transport::PowerState;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use tokio::sync::Notify;

/// Battery percentage at or below which an unplugged device counts as Low.
const LOW_BATTERY_PERCENT: u8 = 20;

/// Pure classification of raw platform readings.
///
/// - `ac_online`: `Some(true)` on mains/charging, `Some(false)` on battery,
///   `None` when the platform reports no battery at all.
/// - `battery_percent`: remaining charge when known.
pub fn classify_power(ac_online: Option<bool>, battery_percent: Option<u8>) -> PowerState {
    match ac_online {
        // No battery information: a machine without a battery is mains powered.
        None | Some(true) => PowerState::Charging,
        Some(false) => match battery_percent {
            Some(p) if p <= LOW_BATTERY_PERCENT => PowerState::Low,
            _ => PowerState::Normal,
        },
    }
}

/// Current device power state, read from the OS where available.
pub fn detect_power_state() -> PowerState {
    let (ac, pct) = read_power();
    classify_power(ac, pct)
}

#[cfg(target_os = "linux")]
fn read_power() -> (Option<bool>, Option<u8>) {
    use std::fs;
    let Ok(dir) = fs::read_dir("/sys/class/power_supply") else {
        return (None, None);
    };
    let read = |p: &std::path::Path| -> String {
        fs::read_to_string(p)
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    };
    let mut has_battery = false;
    let mut ac = false;
    let mut pct = None;
    for entry in dir.flatten() {
        let path = entry.path();
        match read(&path.join("type")).as_str() {
            "Mains" | "USB" => {
                if read(&path.join("online")) == "1" {
                    ac = true;
                }
            }
            "Battery" => {
                has_battery = true;
                pct = read(&path.join("capacity")).parse::<u8>().ok();
                if matches!(read(&path.join("status")).as_str(), "Charging" | "Full") {
                    ac = true;
                }
            }
            _ => {}
        }
    }
    if has_battery {
        (Some(ac), pct)
    } else {
        (None, None)
    }
}

#[cfg(windows)]
fn read_power() -> (Option<bool>, Option<u8>) {
    use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
    let mut status = SYSTEM_POWER_STATUS::default();
    // SAFETY: `status` is a valid, exclusively borrowed SYSTEM_POWER_STATUS
    // that outlives the call; the API only writes into it.
    let ok = unsafe { GetSystemPowerStatus(&mut status) }.is_ok();
    if !ok {
        return (None, None);
    }
    // BatteryFlag bit 128 = no system battery; ACLineStatus 255 = unknown.
    if status.BatteryFlag & 128 != 0 || status.ACLineStatus == 255 {
        return (None, None);
    }
    let pct = (status.BatteryLifePercent != 255).then_some(status.BatteryLifePercent);
    (Some(status.ACLineStatus == 1), pct)
}

#[cfg(not(any(target_os = "linux", windows)))]
fn read_power() -> (Option<bool>, Option<u8>) {
    // No cheap platform source: computed default (see module header).
    (None, None)
}

/// A live OS network-change notification source.
pub struct ChangeWatch {
    notify: Arc<Notify>,
    alive: Arc<AtomicBool>,
}

impl ChangeWatch {
    /// Wait for the next OS notification that a local address or link changed.
    pub async fn changed(&self) {
        self.notify.notified().await;
    }

    /// False once the notification source has died; the caller must fall back
    /// to polling.
    pub fn alive(&self) -> bool {
        self.alive.load(Ordering::SeqCst)
    }
}

/// The one process-wide registration. The OS source (a Windows address-change
/// registration, a Linux netlink thread) is created at most once; later calls
/// share it, so repeated calls cannot leak a registration or an `Arc` each.
static SHARED_WATCH: OnceLock<(Arc<Notify>, Arc<AtomicBool>)> = OnceLock::new();
static SHARED_INIT: Mutex<()> = Mutex::new(());

/// Start the OS notification source for this platform, or `None` when the
/// platform has none (or registration failed): the caller then polls. Safe
/// to call repeatedly: only the first successful call registers with the OS.
/// A failed registration is not cached, so a later call may succeed.
pub fn start_change_watch() -> Option<ChangeWatch> {
    let _guard = SHARED_INIT.lock().unwrap_or_else(|e| e.into_inner());
    if SHARED_WATCH.get().is_none() {
        let notify = Arc::new(Notify::new());
        let alive = Arc::new(AtomicBool::new(true));
        if platform_start(&notify, &alive) {
            // Serialised by SHARED_INIT, so this cannot already be set.
            let _ = SHARED_WATCH.set((notify, alive));
        }
    }
    SHARED_WATCH.get().map(|(notify, alive)| ChangeWatch {
        notify: Arc::clone(notify),
        alive: Arc::clone(alive),
    })
}

/// Delay before re-opening a dead netlink socket after `failures`
/// consecutive failed attempts: exponential from the base, bounded so a
/// recovered kernel interface is picked up again within the cap. Never
/// returns a give-up signal.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn reopen_backoff(failures: u32) -> std::time::Duration {
    const BASE_MS: u64 = 500;
    const CAP_MS: u64 = 60_000;
    let shift = failures.min(16);
    std::time::Duration::from_millis(BASE_MS.saturating_mul(1u64 << shift).min(CAP_MS))
}

#[cfg(windows)]
fn platform_start(notify: &Arc<Notify>, _alive: &Arc<AtomicBool>) -> bool {
    use std::ffi::c_void;
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::NetworkManagement::IpHelper::{
        NotifyUnicastIpAddressChange, MIB_NOTIFICATION_TYPE, MIB_UNICASTIPADDRESS_ROW,
    };
    use windows::Win32::Networking::WinSock::AF_UNSPEC;

    unsafe extern "system" fn on_change(
        ctx: *const c_void,
        _row: *const MIB_UNICASTIPADDRESS_ROW,
        _kind: MIB_NOTIFICATION_TYPE,
    ) {
        // SAFETY: `ctx` is the pointer produced by `Arc::into_raw` below and
        // is never freed (the registration lives for the process lifetime), so
        // it always points to a live `Notify`.
        let notify = unsafe { &*(ctx as *const Notify) };
        notify.notify_one();
    }

    // The registration and its context intentionally live for the whole
    // process: the Arc strong count is leaked so the callback pointer can
    // never dangle.
    let ctx = Arc::into_raw(Arc::clone(notify)) as *const c_void;
    let mut handle = HANDLE::default();
    // SAFETY: `on_change` matches the required callback signature, `ctx` is a
    // valid never-freed pointer (see above), and `handle` is a valid
    // out-pointer for the duration of the call.
    let rc = unsafe {
        NotifyUnicastIpAddressChange(AF_UNSPEC, Some(on_change), Some(ctx), false, &mut handle)
    };
    if rc.0 == 0 {
        tracing::info!("[DISCOVERY] network-change source: NotifyUnicastIpAddressChange");
        true
    } else {
        tracing::warn!(
            "[DISCOVERY] NotifyUnicastIpAddressChange failed ({}); falling back to scheduler-driven poll",
            rc.0
        );
        false
    }
}

/// Open and bind a `NETLINK_ROUTE` socket subscribed to link and address
/// changes. Returns the raw descriptor, owned by the caller.
#[cfg(target_os = "linux")]
fn open_netlink() -> std::io::Result<libc::c_int> {
    // Netlink multicast groups (linux/rtnetlink.h).
    const RTMGRP_LINK: u32 = 0x1;
    const RTMGRP_IPV4_IFADDR: u32 = 0x10;
    const RTMGRP_IPV6_IFADDR: u32 = 0x100;

    // SAFETY: plain socket(2) call with constant arguments.
    let fd = unsafe {
        libc::socket(
            libc::AF_NETLINK,
            libc::SOCK_RAW | libc::SOCK_CLOEXEC,
            libc::NETLINK_ROUTE,
        )
    };
    if fd < 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: sockaddr_nl is plain-old-data; all-zero is a valid value.
    let mut addr: libc::sockaddr_nl = unsafe { std::mem::zeroed() };
    addr.nl_family = libc::AF_NETLINK as libc::sa_family_t;
    addr.nl_groups = RTMGRP_LINK | RTMGRP_IPV4_IFADDR | RTMGRP_IPV6_IFADDR;
    // SAFETY: `addr` is a valid sockaddr_nl and the length passed is its size.
    let rc = unsafe {
        libc::bind(
            fd,
            &addr as *const libc::sockaddr_nl as *const libc::sockaddr,
            std::mem::size_of::<libc::sockaddr_nl>() as libc::socklen_t,
        )
    };
    if rc < 0 {
        let err = std::io::Error::last_os_error();
        // SAFETY: `fd` is an open descriptor owned by this function.
        unsafe { libc::close(fd) };
        return Err(err);
    }
    Ok(fd)
}

/// Read one netlink socket until it fails fatally. Receive-buffer overruns
/// (`ENOBUFS`) mean notifications were dropped, which is itself a change
/// hint: wake the monitor (it diffs the real interface set) and keep
/// reading. Returns the fatal error and whether any message was received.
#[cfg(target_os = "linux")]
fn read_netlink(fd: libc::c_int, notify: &Notify) -> (std::io::Error, bool) {
    let mut buf = [0u8; 4096];
    let mut received = false;
    loop {
        // SAFETY: `buf` is a valid writable buffer of the given length and
        // `fd` is an open netlink socket owned by the calling thread.
        let n = unsafe { libc::recv(fd, buf.as_mut_ptr().cast(), buf.len(), 0) };
        if n < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            if err.raw_os_error() == Some(libc::ENOBUFS) {
                notify.notify_one();
                continue;
            }
            return (err, received);
        }
        received = true;
        // Any link/address message is a change hint; the monitor diffs the
        // real interface set.
        notify.notify_one();
    }
}

#[cfg(target_os = "linux")]
fn platform_start(notify: &Arc<Notify>, alive: &Arc<AtomicBool>) -> bool {
    let first_fd = match open_netlink() {
        Ok(fd) => fd,
        Err(e) => {
            tracing::warn!(
                "[DISCOVERY] netlink socket failed ({}); falling back to scheduler-driven poll",
                e
            );
            return false;
        }
    };
    let notify = Arc::clone(notify);
    let alive = Arc::clone(alive);
    let spawned = std::thread::Builder::new()
        .name("scm-netlink-watch".to_string())
        .spawn(move || {
            let mut fd = first_fd;
            let mut failures: u32 = 0;
            loop {
                let (err, received) = read_netlink(fd, &notify);
                // SAFETY: `fd` is still owned by this thread and is closed
                // exactly once before being replaced below.
                unsafe { libc::close(fd) };
                alive.store(false, Ordering::SeqCst);
                // Wake the monitor so it polls while the source is down.
                notify.notify_one();
                // A socket that delivered messages was healthy: restart the
                // backoff; one that died unproductively keeps escalating.
                failures = if received {
                    0
                } else {
                    failures.saturating_add(1)
                };
                tracing::warn!(
                    "[DISCOVERY] netlink read failed ({}); re-opening with backoff, polling meanwhile",
                    err
                );
                // The source is never abandoned: retry until a socket opens.
                fd = loop {
                    std::thread::sleep(reopen_backoff(failures));
                    match open_netlink() {
                        Ok(new_fd) => break new_fd,
                        Err(e) => {
                            failures = failures.saturating_add(1);
                            tracing::warn!("[DISCOVERY] netlink re-open failed ({})", e);
                        }
                    }
                };
                alive.store(true, Ordering::SeqCst);
                tracing::info!("[DISCOVERY] network-change source: netlink re-opened");
                // Changes during the outage were missed: force a diff.
                notify.notify_one();
            }
        });
    match spawned {
        Ok(_) => {
            tracing::info!("[DISCOVERY] network-change source: netlink NETLINK_ROUTE");
            true
        }
        Err(e) => {
            tracing::warn!(
                "[DISCOVERY] netlink thread spawn failed ({}); falling back to scheduler-driven poll",
                e
            );
            // SAFETY: the thread never started, so `first_fd` is still ours.
            unsafe { libc::close(first_fd) };
            false
        }
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
fn platform_start(_notify: &Arc<Notify>, _alive: &Arc<AtomicBool>) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_power_maps_readings() {
        assert_eq!(classify_power(None, None), PowerState::Charging);
        assert_eq!(classify_power(Some(true), Some(5)), PowerState::Charging);
        assert_eq!(classify_power(Some(false), None), PowerState::Normal);
        assert_eq!(classify_power(Some(false), Some(80)), PowerState::Normal);
        assert_eq!(classify_power(Some(false), Some(20)), PowerState::Low);
        assert_eq!(classify_power(Some(false), Some(3)), PowerState::Low);
    }

    #[test]
    fn reopen_backoff_grows_is_bounded_and_never_zero() {
        let mut prev = std::time::Duration::ZERO;
        for failures in 0..64u32 {
            let d = reopen_backoff(failures);
            assert!(d > std::time::Duration::ZERO);
            assert!(d >= prev);
            assert!(d <= std::time::Duration::from_secs(60));
            prev = d;
        }
        assert!(reopen_backoff(3) > reopen_backoff(0));
        assert_eq!(reopen_backoff(u32::MAX), std::time::Duration::from_secs(60));
    }

    #[cfg(any(windows, target_os = "linux"))]
    #[tokio::test]
    async fn repeated_start_shares_one_registration() {
        if let (Some(a), Some(b)) = (start_change_watch(), start_change_watch()) {
            assert!(Arc::ptr_eq(&a.notify, &b.notify));
        }
    }

    #[test]
    fn detect_power_state_never_panics() {
        let _ = detect_power_state();
    }

    #[cfg(any(windows, target_os = "linux"))]
    #[tokio::test]
    async fn change_watch_registers_on_supported_platforms() {
        // Registration may legitimately fail in a locked-down sandbox; the
        // contract is: either a live watch, or None (caller polls).
        if let Some(w) = start_change_watch() {
            assert!(w.alive());
        }
    }
}
