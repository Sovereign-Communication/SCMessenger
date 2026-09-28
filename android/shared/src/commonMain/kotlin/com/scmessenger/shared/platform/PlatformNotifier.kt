package com.scmessenger.shared.platform

/**
 * Expected interface for platform-specific notifications.
 *
 * Android: uses NotificationHelper with foreground service.
 * Desktop: uses system-native notifications (libnotify, etc.).
 */
expect class PlatformNotifier() {
    /**
     * Show a notification.
     * @param title Notification title
     * @param body Notification body
     * @param tag Optional tag for grouping
     */
    fun notify(title: String, body: String, tag: String? = null)

    /**
     * Clear notifications matching tag.
     */
    fun clearNotifications(tag: String? = null)
}
