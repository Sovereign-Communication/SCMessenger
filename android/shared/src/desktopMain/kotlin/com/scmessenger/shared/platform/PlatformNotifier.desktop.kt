package com.scmessenger.shared.platform

/**
 * Desktop actual implementation of PlatformNotifier.
 *
 * Uses system-native notifications via libnotify or similar.
 */
actual class PlatformNotifier {
    actual fun notify(title: String, body: String, tag: String?) {
        // TODO: Use libnotify or desktop notification library
        println("NOTIFICATION [$tag]: $title - $body")
    }

    actual fun clearNotifications(tag: String?) {
        // TODO: Clear desktop notifications
    }
}
