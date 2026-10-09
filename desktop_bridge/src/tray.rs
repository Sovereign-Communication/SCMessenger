// System tray / App Indicator status reporting.
//
// Provides TrayStatus snapshots and state management for the
// desktop client's system tray widget (AppIndicator / StatusIcon).
//
// On Linux: delegates to DesktopDelegate::on_tray_state_changed.
// On other targets: returns Unsupported error.

use crate::TrayStatus;

/// Compute the current tray status from mesh service metrics.
///
/// # Arguments
/// * `icon_state` — Current connection state for the tray icon
/// * `unread_count` — Number of unread messages
/// * `connected_peers` — Number of currently connected mesh peers
/// * `status_text` — Localized tooltip text for the tray icon
pub fn build_tray_status(
    icon_state: crate::TrayIconState,
    unread_count: u32,
    connected_peers: u32,
    status_text: String,
) -> TrayStatus {
    TrayStatus {
        icon_state,
        unread_count,
        connected_peers,
        status_text,
    }
}

/// Get a human-readable status text for the tray tooltip.
/// Returns a (state, text) tuple.
pub fn tray_status_for_state(state: crate::TrayIconState, unread: u32, peers: u32) -> TrayStatus {
    let text = match state {
        crate::TrayIconState::Disconnected => "SCMessenger — Disconnected".to_string(),
        crate::TrayIconState::Connected => {
            if peers > 0 {
                format!(
                    "SCMessenger — {peers} peer{} connected",
                    if peers == 1 { "" } else { "s" }
                )
            } else {
                "SCMessenger — Connected".to_string()
            }
        }
        crate::TrayIconState::UnreadMessages => {
            format!(
                "SCMessenger — {unread} unread message{}",
                if unread == 1 { "" } else { "s" }
            )
        }
        crate::TrayIconState::Error => "SCMessenger — Connection error".to_string(),
    };

    build_tray_status(state, unread, peers, text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_disconnected() {
        let status = tray_status_for_state(crate::TrayIconState::Disconnected, 0, 0);
        assert_eq!(status.connected_peers, 0);
        assert!(status.status_text.contains("Disconnected"));
    }

    #[test]
    fn test_tray_connected_with_peers() {
        let status = tray_status_for_state(crate::TrayIconState::Connected, 0, 3);
        assert_eq!(status.connected_peers, 3);
        assert!(status.status_text.contains("3"));
    }

    #[test]
    fn test_tray_unread() {
        let status = tray_status_for_state(crate::TrayIconState::UnreadMessages, 5, 2);
        assert_eq!(status.unread_count, 5);
        assert!(status.status_text.contains("5"));
    }
}
