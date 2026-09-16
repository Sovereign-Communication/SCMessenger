package com.scmessenger.android.utils

import java.net.URLDecoder
import java.net.URLEncoder

/**
 * Combined install payload for the APK-share QR (emit side).
 *
 * Replaces the bare `http://LAN/scmessenger.apk` URL with a single-scan
 * `scmessenger://install?apk=<urlencoded>&v=<versionName>&vc=<code>[&sha256=...]`
 * string carrying the download URL plus version identity. The SHA-256 slot is
 * reserved for hash-pinned verification; emit currently sends null until the
 * hash source lands. Accept-side parsing in JoinMesh is a deferred follow-up.
 */
data class InstallPayload(
    val apkUrl: String,
    val versionName: String,
    val versionCode: Int,
    val sha256Hex: String? = null
)

private const val INSTALL_SCHEME_PREFIX = "scmessenger://install?"

/**
 * Build the compact install URI for QR emission.
 *
 * Never throws for non-null inputs; callers pass the LAN download URL from
 * [ApkShareManager.startLocalApkHost] with [com.scmessenger.android.BuildConfig]
 * version fields.
 */
fun buildInstallPayloadUri(
    apkUrl: String,
    versionName: String,
    versionCode: Int,
    sha256Hex: String? = null
): String {
    val encodedApk = URLEncoder.encode(apkUrl, "UTF-8")
    val encodedVersion = URLEncoder.encode(versionName, "UTF-8")
    val builder = StringBuilder(INSTALL_SCHEME_PREFIX)
    builder.append("apk=").append(encodedApk)
    builder.append("&v=").append(encodedVersion)
    builder.append("&vc=").append(versionCode)
    if (!sha256Hex.isNullOrBlank()) {
        builder.append("&sha256=").append(sha256Hex.trim())
    }
    return builder.toString()
}

/**
 * Parse a combined install URI back to [InstallPayload].
 *
 * Returns null on any malformed input and never throws: blank strings, wrong
 * scheme/host, missing or invalid params, bad percent-encoding, or a present
 * but non-hex SHA-256 all yield null.
 */
fun parseInstallPayloadUri(raw: String): InstallPayload? {
    try {
        val trimmed = raw.trim()
        if (trimmed.isBlank()) return null
        if (!trimmed.startsWith(INSTALL_SCHEME_PREFIX)) return null
        val query = trimmed.substringAfter("?", missingDelimiterValue = "")
        if (query.isBlank()) return null

        val params = mutableMapOf<String, String>()
        for (part in query.split("&")) {
            if (part.isEmpty()) continue
            val equals = part.indexOf('=')
            if (equals == -1) return null
            val key = part.substring(0, equals)
            val value = part.substring(equals + 1)
            if (key.isEmpty()) return null
            if (!params.containsKey(key)) {
                params[key] = value
            }
        }

        val apkEncoded = params["apk"] ?: return null
        val versionEncoded = params["v"] ?: return null
        val versionCodeRaw = params["vc"] ?: return null
        if (apkEncoded.isEmpty() || versionEncoded.isEmpty() || versionCodeRaw.isEmpty()) return null

        val apkUrl = URLDecoder.decode(apkEncoded, "UTF-8").trim()
        if (apkUrl.isEmpty()) return null
        if (!apkUrl.startsWith("http://") && !apkUrl.startsWith("https://")) return null

        val versionName = URLDecoder.decode(versionEncoded, "UTF-8").trim()
        if (versionName.isEmpty()) return null

        val versionCode = versionCodeRaw.toIntOrNull() ?: return null
        if (versionCode <= 0) return null

        val shaRaw = params["sha256"]
        val sha256Hex = when {
            shaRaw == null -> null
            shaRaw.isEmpty() -> return null
            !shaRaw.matches(Regex("^[0-9a-fA-F]{64}$")) -> return null
            else -> shaRaw.lowercase()
        }

        return InstallPayload(
            apkUrl = apkUrl,
            versionName = versionName,
            versionCode = versionCode,
            sha256Hex = sha256Hex
        )
    } catch (e: Exception) {
        return null
    }
}
