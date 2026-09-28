package com.scmessenger.android.utils

import android.Manifest
import android.content.Context
import androidx.core.content.ContextCompat
import android.os.Build

/**
 * Permissions required for SCMessenger functionality.
 *
 * Organized by feature and API level.
 */
object Permissions {

    /**
     * Bluetooth permissions (varies by API level).
     */
    val bluetooth: List<String> = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
        // Android 12+ (API 31+)
        listOf(
            Manifest.permission.BLUETOOTH_SCAN,
            Manifest.permission.BLUETOOTH_ADVERTISE,
            Manifest.permission.BLUETOOTH_CONNECT
        )
    } else {
        // Android 11 and below
        listOf(
            Manifest.permission.BLUETOOTH,
            Manifest.permission.BLUETOOTH_ADMIN
        )
    }

    /**
     * Location permissions (required for WiFi Aware).
     */
    val location: List<String> = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
        // Android 10+ (API 29+)
        listOf(
            Manifest.permission.ACCESS_FINE_LOCATION,
            Manifest.permission.ACCESS_COARSE_LOCATION
        )
    } else {
        listOf(
            Manifest.permission.ACCESS_FINE_LOCATION
        )
    }

    /**
     * Nearby WiFi devices (Android 13+).
     */
    val nearbyWifi: List<String> = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
        listOf(Manifest.permission.NEARBY_WIFI_DEVICES)
    } else {
        emptyList()
    }

    /**
     * Notifications permission (Android 13+).
     */
    val notifications: List<String> = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
        listOf(Manifest.permission.POST_NOTIFICATIONS)
    } else {
        emptyList()
    }

    /**
     * All required permissions for core functionality.
     */
    val required: List<String> = bluetooth + location + nearbyWifi

    /**
     * All permissions including optional ones.
     */
    val all: List<String> = required + notifications

    /**
     * Get user-friendly permission names for rationale dialogs.
     */
    fun getPermissionName(permission: String): String = when (permission) {
        Manifest.permission.BLUETOOTH,
        Manifest.permission.BLUETOOTH_ADMIN,
        Manifest.permission.BLUETOOTH_SCAN,
        Manifest.permission.BLUETOOTH_ADVERTISE,
        Manifest.permission.BLUETOOTH_CONNECT -> "Bluetooth"

        Manifest.permission.ACCESS_FINE_LOCATION,
        Manifest.permission.ACCESS_COARSE_LOCATION -> "Location"

        Manifest.permission.NEARBY_WIFI_DEVICES -> "Nearby WiFi Devices"

        Manifest.permission.POST_NOTIFICATIONS -> "Notifications"

        else -> permission
    }

    /**
     * Get rationale explanation for a permission.
     */
    fun getRationale(permission: String): String = when (permission) {
        Manifest.permission.BLUETOOTH_SCAN ->
            "SCMessenger uses Bluetooth to discover nearby mesh nodes."

        Manifest.permission.BLUETOOTH_ADVERTISE ->
            "SCMessenger advertises your device so others can find you."

        Manifest.permission.BLUETOOTH_CONNECT ->
            "Required to connect to discovered mesh nodes."

        Manifest.permission.BLUETOOTH,
        Manifest.permission.BLUETOOTH_ADMIN ->
            "Bluetooth is required for discovering and messaging with nearby peers."

        Manifest.permission.ACCESS_FINE_LOCATION ->
            "Android requires location permission for Bluetooth Low Energy scanning. " +
            "Your location is never shared."

        Manifest.permission.ACCESS_COARSE_LOCATION ->
            "Location access is required for WiFi Aware peer discovery. " +
            "SCMessenger does not track or store your location."

        Manifest.permission.NEARBY_WIFI_DEVICES ->
            "Nearby WiFi devices permission enables WiFi Aware for " +
            "direct peer-to-peer messaging without infrastructure."

        Manifest.permission.POST_NOTIFICATIONS ->
            "Notifications alert you when new messages arrive."

        Manifest.permission.CAMERA ->
            "Camera is used to scan QR codes for adding contacts."

        else -> "This permission is required for mesh networking."
    }

    /**
     * Check if all permissions required for mDNS discovery are granted.
     *
     * On API 33+ (Tiramisu), NEARBY_WIFI_DEVICES is required for NsdManager.
     * On API 29-32, ACCESS_FINE_LOCATION is required for Wi-Fi/mDNS operations.
     * Returns true only when the platform-appropriate permission set is fully granted.
     *
     * NOTE: If this returns false, MdnsServiceDiscovery will refuse to start
     * and set lastFailureReason = "PERMISSION_DENIED" rather than timing out silently.
     */
    fun hasMdnsPermissions(context: Context): Boolean {
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            // API 33+: NEARBY_WIFI_DEVICES is the gating permission for mDNS
            ContextCompat.checkSelfPermission(
                context, Manifest.permission.NEARBY_WIFI_DEVICES
            ) == android.content.pm.PackageManager.PERMISSION_GRANTED
        } else {
            // API 29-32: Location permission gates Wi-Fi/mDNS discovery
            ContextCompat.checkSelfPermission(
                context, Manifest.permission.ACCESS_FINE_LOCATION
            ) == android.content.pm.PackageManager.PERMISSION_GRANTED
        }
    }
}
