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
use std::sync::Arc;
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

/// Start the OS notification source for this platform, or `None` when the
/// platform has none (or registration failed): the caller then polls.
pub fn start_change_watch() -> Option<ChangeWatch> {
    let notify = Arc::new(Notify::new());
    let alive = Arc::new(AtomicBool::new(true));
    if platform_start(&notify, &alive) {
        Some(ChangeWatch { notify, alive })
    } else {
        None
    }
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

#[cfg(target_os = "linux")]
fn platform_start(notify: &Arc<Notify>, alive: &Arc<AtomicBool>) -> bool {
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
        tracing::warn!(
            "[DISCOVERY] netlink socket failed ({}); falling back to scheduler-driven poll",
            std::io::Error::last_os_error()
        );
        return false;
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
        tracing::warn!(
            "[DISCOVERY] netlink bind failed ({}); falling back to scheduler-driven poll",
            std::io::Error::last_os_error()
        );
        // SAFETY: `fd` is an open descriptor owned by this function.
        unsafe { libc::close(fd) };
        return false;
    }

    let notify = Arc::clone(notify);
    let alive = Arc::clone(alive);
    let spawned = std::thread::Builder::new()
        .name("scm-netlink-watch".to_string())
        .spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                // SAFETY: `buf` is a valid writable buffer of the given
                // length and `fd` is an open netlink socket owned by this
                // thread.
                let n = unsafe { libc::recv(fd, buf.as_mut_ptr().cast(), buf.len(), 0) };
                if n < 0 {
                    if std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted {
                        continue;
                    }
                    break;
                }
                // Any link/address message is a change hint; the monitor
                // diffs the real interface set.
                notify.notify_one();
            }
            alive.store(false, Ordering::SeqCst);
            // SAFETY: `fd` is still owned by this thread and is closed once.
            unsafe { libc::close(fd) };
            // Wake the monitor so it notices `alive == false` and polls.
            notify.notify_one();
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
            // SAFETY: the thread never started, so `fd` is still ours.
            unsafe { libc::close(fd) };
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
