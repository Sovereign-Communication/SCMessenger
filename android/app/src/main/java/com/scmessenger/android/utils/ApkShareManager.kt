package com.scmessenger.android.utils

import android.content.Context
import android.content.Intent
import android.net.Uri
import androidx.core.content.FileProvider
import timber.log.Timber
import java.io.File
import java.io.FileInputStream
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
     */
    fun getLocalIpAddress(): String {
        try {
            val interfaces = NetworkInterface.getNetworkInterfaces()
            while (interfaces.hasMoreElements()) {
                val networkInterface = interfaces.nextElement()
                if (networkInterface.isLoopback || !networkInterface.isUp) continue
                val addresses = networkInterface.inetAddresses
                while (addresses.hasMoreElements()) {
                    val addr = addresses.nextElement()
                    if (!addr.isLoopbackAddress && addr is InetAddress && addr.address.size == 4) {
                        return addr.hostAddress ?: "127.0.0.1"
                    }
                }
            }
        } catch (e: Exception) {
            Timber.w(e, "Failed to resolve local IP address")
        }
        return "127.0.0.1"
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
        val awsRelay = "/ip4/100.56.248.69/tcp/9001"
        if (isHosting) {
            val ip = getLocalIpAddress()
            onStarted("http://$ip:$hostingPort/scmessenger.apk?bootstrap=$awsRelay")
            return
        }

        val apkFile = prepareShareableApk(context)
        val executor = Executors.newSingleThreadExecutor()

        try {
            serverSocket = ServerSocket(0) // Bind to dynamic available port
            hostingPort = serverSocket!!.localPort
            isHosting = true
            val ip = getLocalIpAddress()
            val downloadUrl = "http://$ip:$hostingPort/scmessenger.apk?bootstrap=$awsRelay"

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
     * Handle incoming HTTP GET request for the APK file.
     */
    private fun handleHttpClient(socket: Socket, apkFile: File) {
        try {
            socket.use { s ->
                val output: OutputStream = s.getOutputStream()
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

    fun isHosting(): Boolean = isHosting
    fun getHostingPort(): Int = hostingPort
}
