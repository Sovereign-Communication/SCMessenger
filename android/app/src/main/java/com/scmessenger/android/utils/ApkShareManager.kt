package com.scmessenger.android.utils

import android.content.Context
import android.content.Intent
import android.net.Uri
import androidx.core.content.FileProvider
import timber.log.Timber
import java.io.BufferedReader
import java.io.File
import java.io.FileInputStream
import java.io.InputStreamReader
import java.io.OutputStream
import java.net.InetAddress
import java.net.NetworkInterface
import java.net.ServerSocket
import java.net.Socket
import java.util.concurrent.Executors
import java.util.concurrent.ScheduledExecutorService
import java.util.concurrent.TimeUnit

/**
 * Utility for extracting and sharing the installed SCMessenger APK.
 *
 * Capabilities:
 * 1. Native System Share: Copies source APK to cache and invokes Intent.ACTION_SEND
 *    for direct sharing via Bluetooth, QuickShare, Wi-Fi Direct, etc.
 * 2. Local Node QR Host: Spins up an ephemeral HTTP server on local IP/port serving
 *    scmessenger.apk with automatic timeout (default 15 mins) or single-download limit.
 */
object ApkShareManager {

    private var serverSocket: ServerSocket? = null
    private var isHosting = false
    private var hostingPort = 8080
    private var scheduler: ScheduledExecutorService? = null

    private const val APK_HTTP_PATH = "/scmessenger.apk"
    private const val MAX_HTTP_REQUEST_BYTES = 8192
    private const val MAX_HTTP_HEADER_LINES = 64
    private const val SOCKET_READ_TIMEOUT_MS = 5000

    /**
     * Get the source APK file of the running application.
     */
    fun getInstalledApkFile(context: Context): File {
        return File(context.applicationInfo.sourceDir)
    }

    /**
     * Prepare a shareable copy of the installed APK in cache directory.
     */
    fun prepareShareableApk(context: Context): File {
        val sourceApk = getInstalledApkFile(context)
        val targetApk = File(context.cacheDir, "scmessenger-v0.4.0.apk")
        if (!targetApk.exists() || targetApk.length() != sourceApk.length()) {
            sourceApk.copyTo(targetApk, overwrite = true)
        }
        return targetApk
    }

    /**
     * Launch system share sheet to share the APK file directly with nearby contacts.
     */
    fun shareApkViaSystemIntent(context: Context) {
        try {
            val apkFile = prepareShareableApk(context)
            val uri: Uri = FileProvider.getUriForFile(
                context,
                "${context.packageName}.fileprovider",
                apkFile
            )

            val intent = Intent(Intent.ACTION_SEND).apply {
                type = "application/vnd.android.package-archive"
                putExtra(Intent.EXTRA_STREAM, uri)
                addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                putExtra(Intent.EXTRA_SUBJECT, "SCMessenger Android App")
            }
            context.startActivity(Intent.createChooser(intent, "Share SCMessenger App"))
        } catch (e: Exception) {
            Timber.e(e, "Failed to share APK via system intent")
        }
    }

    /**
     * Get the primary local IPv4 address (e.g. Wi-Fi or Hotspot interface).
     *
     * Prefers the hotspot / Wi-Fi-Direct group-owner ranges first
     * (192.168.43.x, 192.168.49.x) since the receiver is joined to that
     * network, then other site-local addresses, then the first-up fallback.
     */
    fun getLocalIpAddress(): String {
        try {
            val candidates = mutableListOf<String>()
            val interfaces = NetworkInterface.getNetworkInterfaces()
            while (interfaces.hasMoreElements()) {
                val networkInterface = interfaces.nextElement()
                if (networkInterface.isLoopback || !networkInterface.isUp) continue
                val addresses = networkInterface.inetAddresses
                while (addresses.hasMoreElements()) {
                    val addr = addresses.nextElement()
                    if (!addr.isLoopbackAddress && addr is InetAddress && addr.address.size == 4) {
                        addr.hostAddress?.let { candidates.add(it) }
                    }
                }
            }
            if (candidates.isNotEmpty()) {
                return pickPreferredIpv4(candidates)
            }
        } catch (e: Exception) {
            Timber.w(e, "Failed to resolve local IP address")
        }
        return "127.0.0.1"
    }

    /**
     * Pick the best candidate IPv4 address: hotspot / Wi-Fi-Direct ranges
     * first, then other site-local ranges, then anything else. The sort is
     * stable so equal-rank candidates keep discovery order (the previous
     * first-up behaviour) as the tiebreak.
     */
    private fun pickPreferredIpv4(candidates: List<String>): String {
        return candidates.sortedWith(compareBy(::rankIpv4Address)).firstOrNull() ?: "127.0.0.1"
    }

    private fun rankIpv4Address(host: String): Int {
        return when {
            host.startsWith("192.168.43.") || host.startsWith("192.168.49.") -> 0
            isSiteLocalIpv4(host) -> 1
            host.startsWith("127.") -> 3
            else -> 2
        }
    }

    private fun isSiteLocalIpv4(host: String): Boolean {
        if (host.startsWith("192.168.") || host.startsWith("10.")) return true
        if (host.startsWith("172.")) {
            val secondOctet = host.split(".").getOrNull(1)?.toIntOrNull()
            if (secondOctet != null && secondOctet in 16..31) return true
        }
        return false
    }

    /**
     * Start hosting the APK over an ephemeral local HTTP server.
     *
     * @param context App context
     * @param durationMinutes Duration before auto-stopping (default 15m)
     * @param onStarted Callback when server starts with URL string
     */
    @Synchronized
    fun startLocalApkHost(
        context: Context,
        durationMinutes: Long = 15,
        onStarted: (String) -> Unit
    ) {
        if (isHosting) {
            val ip = getLocalIpAddress()
            onStarted("http://$ip:$hostingPort/scmessenger.apk")
            return
        }

        val apkFile = prepareShareableApk(context)
        val executor = Executors.newSingleThreadExecutor()

        try {
            serverSocket = ServerSocket(0) // Bind to dynamic available port
            hostingPort = serverSocket!!.localPort
            isHosting = true
            val ip = getLocalIpAddress()
            // This URL is a local, ephemeral file host.  Do not append a relay
            // hint: the old hardcoded endpoint was dead and caused installers to
            // seed an unrelated node before the app had verified its identity.
            val downloadUrl = "http://$ip:$hostingPort/scmessenger.apk"

            executor.execute {
                while (isHosting && serverSocket != null && !serverSocket!!.isClosed) {
                    try {
                        val clientSocket = serverSocket!!.accept()
                        handleHttpClient(clientSocket, apkFile)
                    } catch (e: Exception) {
                        if (isHosting) {
                            Timber.d("ApkServer accept loop terminated: ${e.message}")
                        }
                    }
                }
            }

            // Schedule auto-shutdown
            scheduler = Executors.newSingleThreadScheduledExecutor()
            scheduler?.schedule({
                stopLocalApkHost()
            }, durationMinutes, TimeUnit.MINUTES)

            onStarted(downloadUrl)
            Timber.i("Started local APK server at $downloadUrl for $durationMinutes mins")
        } catch (e: Exception) {
            Timber.e(e, "Failed to start local APK server")
            stopLocalApkHost()
        }
    }

    /**
     * Stop the local APK HTTP server.
     */
    @Synchronized
    fun stopLocalApkHost() {
        isHosting = false
        try {
            serverSocket?.close()
        } catch (e: Exception) {
            Timber.w("Error closing server socket: ${e.message}")
        }
        serverSocket = null
        scheduler?.shutdownNow()
        scheduler = null
        Timber.i("Stopped local APK server")
    }

    /**
     * Handle an incoming HTTP connection for the APK file.
     *
     * Reads the request line + headers first (bounded at 8KB with a socket
     * timeout) and serves the APK bytes only for `GET /scmessenger.apk`.
     * Malformed requests get `400 Bad Request`; well-formed requests for any
     * other method/path get `404 Not Found`.
     */
    private fun handleHttpClient(socket: Socket, apkFile: File) {
        try {
            socket.use { s ->
                s.soTimeout = SOCKET_READ_TIMEOUT_MS
                val output: OutputStream = s.getOutputStream()
                val requestLine = try {
                    readHttpRequestLine(s)
                } catch (e: Exception) {
                    Timber.w("APK host: failed to read HTTP request: ${e.message}")
                    null
                }
                if (requestLine.isNullOrEmpty()) {
                    writeHttpError(output, 400, "Bad Request")
                    return
                }
                val parsed = parseHttpRequestLine(requestLine)
                if (parsed == null) {
                    Timber.w("APK host: malformed request line denied")
                    writeHttpError(output, 400, "Bad Request")
                    return
                }
                val (method, path) = parsed
                if (method != "GET" || path != APK_HTTP_PATH) {
                    Timber.w("APK host: denied $method $path")
                    writeHttpError(output, 404, "Not Found")
                    return
                }
                val header = buildString {
                    append("HTTP/1.1 200 OK\r\n")
                    append("Content-Type: application/vnd.android.package-archive\r\n")
                    append("Content-Length: ${apkFile.length()}\r\n")
                    append("Content-Disposition: attachment; filename=\"scmessenger-v0.4.0.apk\"\r\n")
                    append("Connection: close\r\n\r\n")
                }
                output.write(header.toByteArray(Charsets.UTF_8))
                output.flush()

                FileInputStream(apkFile).use { input ->
                    val buffer = ByteArray(8192)
                    var bytesRead: Int
                    while (input.read(buffer).also { bytesRead = it } != -1) {
                        output.write(buffer, 0, bytesRead)
                    }
                }
                output.flush()
                Timber.i("Successfully served APK download to ${s.inetAddress.hostAddress}")
            }
        } catch (e: Exception) {
            Timber.w(e, "Error serving APK client request")
        }
    }

    /**
     * Read the HTTP request line, draining headers under the same byte cap so
     * an unbounded header block cannot pin the single serving thread. Returns
     * null when there is no request line or the cap is exceeded.
     */
    private fun readHttpRequestLine(socket: Socket): String? {
        val reader = BufferedReader(InputStreamReader(socket.getInputStream(), Charsets.US_ASCII))
        var totalBytes = 0
        val requestLine = reader.readLine() ?: return null
        totalBytes += requestLine.length + 2
        if (totalBytes > MAX_HTTP_REQUEST_BYTES) return null
        var headerLines = 0
        while (true) {
            val line = reader.readLine() ?: break
            totalBytes += line.length + 2
            if (totalBytes > MAX_HTTP_REQUEST_BYTES) return null
            if (line.isEmpty()) break
            headerLines++
            if (headerLines > MAX_HTTP_HEADER_LINES) return null
        }
        return requestLine
    }

    /**
     * Parse an HTTP request line into (method, path). Returns null when the
     * line is malformed. Absolute-form targets are reduced to their path;
     * query/fragment-bearing targets are kept verbatim so they miss the exact
     * allow-match and fall through to 404.
     */
    private fun parseHttpRequestLine(requestLine: String): Pair<String, String>? {
        val parts = requestLine.trim().split(" ")
        if (parts.size != 3) return null
        val method = parts[0]
        var target = parts[1]
        if (method.isEmpty() || target.isEmpty()) return null
        if (!parts[2].startsWith("HTTP/")) return null
        if (target.startsWith("http://") || target.startsWith("https://")) {
            val schemeEnd = target.indexOf("://") + 3
            val pathStart = target.indexOf('/', schemeEnd)
            if (pathStart == -1) return null
            target = target.substring(pathStart)
        }
        return Pair(method, target)
    }

    private fun writeHttpError(output: OutputStream, statusCode: Int, reason: String) {
        val bodyBytes = "$statusCode $reason\n".toByteArray(Charsets.UTF_8)
        val header = buildString {
            append("HTTP/1.1 $statusCode $reason\r\n")
            append("Content-Type: text/plain; charset=utf-8\r\n")
            append("Content-Length: ${bodyBytes.size}\r\n")
            append("Connection: close\r\n\r\n")
        }
        output.write(header.toByteArray(Charsets.UTF_8))
        output.write(bodyBytes)
        output.flush()
    }

    fun isHosting(): Boolean = isHosting
    fun getHostingPort(): Int = hostingPort
}
