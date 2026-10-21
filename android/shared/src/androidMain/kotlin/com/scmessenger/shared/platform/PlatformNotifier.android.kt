package com.scmessenger.shared.platform

/**
 * Android actual implementation of PlatformNotifier.
 *
 * Delegates to Android NotificationHelper.
 */
actual class PlatformNotifier {
    actual fun notify(title: String, body: String, tag: String?) {
        // TODO: Delegate to NotificationHelper on Android
    }

    actual fun clearNotifications(tag: String?) {
        // TODO: Clear notifications on Android
    }
}
