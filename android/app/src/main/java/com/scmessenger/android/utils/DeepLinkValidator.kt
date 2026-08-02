package com.scmessenger.android.utils

import timber.log.Timber

/**
 * Validates and sanitizes multiaddr strings from deep links.
 *
 * SECURITY: This is a new attack surface -- untrusted QR codes can embed
 * attacker-chosen addresses. We validate strictly before accepting.
 */
object DeepLinkValidator {

    private const val MAX_LISTENERS = 5

    /**
     * Sanitize and validate a list of raw multiaddr strings.
     *
     * @param raw List of raw multiaddr strings from the deep link
     * @param deviceIp The device's current IP address (for subnet validation), or null
     * @return List of validated multiaddr strings (max 5)
     */
    internal fun sanitizeDeepLinkMultiaddrs(
        raw: List<String>,
        deviceIp: String?
    ): List<String> {
        return raw
            .asSequence()
            .map { it.trim() }
            .filter { it.isNotEmpty() }
            .distinct()
            .take(MAX_LISTENERS * 2) // Take extra in case some fail validation
            .filter { multiaddr ->
                validateMultiaddr(multiaddr, deviceIp).also { valid ->
                    if (!valid) {
                        Timber.w("Deep link multiaddr rejected: $multiaddr")
                    }
                }
            }
            .take(MAX_LISTENERS)
            .toList()
    }

    /**
     * Validate a single multiaddr string.
     *
     * Requirements:
     * - Must start with '/ip4/' or '/ip6/' and contain '/tcp/'
     * - Reject loopback (127.0.0.0/8, ::1)
     * - Reject link-local (169.254.0.0/16, fe80::)
     * - Reject multicast (224.0.0.0/4 for IPv4, ff00::/8 for IPv6)
     * - Reject private ranges (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16)
     *   UNLESS the address is on the device's own subnet (LAN pairing use case)
     */
    private fun validateMultiaddr(multiaddr: String, deviceIp: String?): Boolean {
        // Must start with /ip4/ or /ip6/
        val isIpv4 = multiaddr.startsWith("/ip4/")
        val isIpv6 = multiaddr.startsWith("/ip6/")
        if (!isIpv4 && !isIpv6) {
            return false
        }

        // Must contain /tcp/ AND carry a syntactically valid port. Presence of
        // the literal "/tcp/" is not enough -- "/tcp/notaport" would otherwise
        // be accepted and fail only at dial time.
        if (!multiaddr.contains("/tcp/")) {
            return false
        }
        if (!hasValidTcpPort(multiaddr)) {
            return false
        }

        // Extract the IP address portion
        val ip = extractIp(multiaddr, isIpv4) ?: return false

        return when {
            isIpv4 -> validateIpv4(ip, deviceIp)
            isIpv6 -> validateIpv6(ip, deviceIp)
            else -> false
        }
    }

    /**
     * Verify the `/tcp/<port>` component carries a port in 1..65535.
     * Port 0 is rejected: it means "any port" to the OS and is never a
     * meaningful dial target.
     */
    private fun hasValidTcpPort(multiaddr: String): Boolean {
        val segments = multiaddr.split("/")
        val tcpIndex = segments.indexOf("tcp")
        if (tcpIndex == -1 || tcpIndex + 1 >= segments.size) return false
        val port = segments[tcpIndex + 1].toIntOrNull() ?: return false
        return port in 1..65535
    }

    private fun extractIp(multiaddr: String, isIpv4: Boolean): String? {
        return try {
            val prefix = if (isIpv4) "/ip4/" else "/ip6/"
            val afterPrefix = multiaddr.removePrefix(prefix)
            val nextSlash = afterPrefix.indexOf('/')
            if (nextSlash == -1) null else afterPrefix.substring(0, nextSlash)
        } catch (e: Exception) {
            null
        }
    }

    private fun validateIpv4(ip: String, deviceIp: String?): Boolean {
        val parts = ip.split(".")
        if (parts.size != 4) return false

        val octets = parts.mapNotNull { it.toIntOrNull() }
        if (octets.size != 4) return false
        if (octets.any { it !in 0..255 }) return false

        // Reject leading zeros ("010.1.2.3"): they are parsed as octal by some
        // resolvers, so the same string can denote two different hosts.
        if (parts.any { it.length > 1 && it.startsWith("0") }) return false

        val first = octets[0]
        val second = octets[1]

        // Reject "this host" / wildcard: 0.0.0.0/8
        if (first == 0) return false

        // Reject loopback: 127.0.0.0/8
        if (first == 127) return false

        // Reject link-local: 169.254.0.0/16
        if (first == 169 && second == 254) return false

        // Reject multicast: 224.0.0.0/4 (224-239)
        if (first in 224..239) return false

        // Reject reserved 240.0.0.0/4 and the 255.255.255.255 broadcast.
        // Without this, "255.255.255.255" validated as a dialable address.
        if (first >= 240) return false

        // Reject documentation/benchmark ranges that must never be dialed:
        // TEST-NET-1 192.0.2.0/24, TEST-NET-2 198.51.100.0/24,
        // TEST-NET-3 203.0.113.0/24, benchmark 198.18.0.0/15.
        if (first == 192 && second == 0 && octets[2] == 2) return false
        if (first == 198 && second == 51 && octets[2] == 100) return false
        if (first == 203 && second == 0 && octets[2] == 113) return false
        if (first == 198 && second in 18..19) return false

        // Reject private ranges unless on device subnet
        val isPrivate = when {
            first == 10 -> true // 10.0.0.0/8
            first == 172 && second in 16..31 -> true // 172.16.0.0/12
            first == 192 && second == 168 -> true // 192.168.0.0/16
            else -> false
        }

        if (isPrivate && deviceIp != null) {
            // Check if device is on the same subnet
            return isOnSameSubnet(ip, deviceIp)
        }

        return !isPrivate
    }

    private fun validateIpv6(ip: String, deviceIp: String?): Boolean {
        val lowerIp = ip.lowercase()

        // Reject loopback: ::1
        if (lowerIp == "::1" || lowerIp == "0:0:0:0:0:0:0:1") return false

        // Reject link-local: fe80::/10
        if (lowerIp.startsWith("fe8") || lowerIp.startsWith("fe9") ||
            lowerIp.startsWith("fea") || lowerIp.startsWith("feb")) {
            return false
        }

        // Reject multicast: ff00::/8
        if (lowerIp.startsWith("ff")) return false

        // Reject unique-local (private): fc00::/7
        if (lowerIp.startsWith("fc") || lowerIp.startsWith("fd")) {
            // Check if device is on same subnet
            return deviceIp != null && isOnSameSubnet(ip, deviceIp)
        }

        return true
    }

    /**
     * Check if two IPv4 addresses are on the same /24 subnet.
     * For IPv6, just check if they match exactly (simplified for now).
     */
    private fun isOnSameSubnet(ip1: String, ip2: String): Boolean {
        return try {
            if (ip1.contains(":") || ip2.contains(":")) {
                // IPv6: simplified check -- exact match only
                ip1.equals(ip2, ignoreCase = true)
            } else {
                // IPv4: check /24 subnet match
                val parts1 = ip1.split(".")
                val parts2 = ip2.split(".")
                if (parts1.size != 4 || parts2.size != 4) return false

                // Compare first 3 octets (same /24 subnet)
                parts1[0] == parts2[0] && parts1[1] == parts2[1] && parts1[2] == parts2[2]
            }
        } catch (e: Exception) {
            false
        }
    }
}
