package com.scmessenger.android.transport.ble

import android.annotation.SuppressLint
import android.bluetooth.BluetoothAdapter
import android.bluetooth.BluetoothManager
import android.bluetooth.le.ScanCallback
import android.bluetooth.le.ScanFilter
import android.bluetooth.le.ScanResult
import android.bluetooth.le.ScanSettings
import android.content.Context
import android.os.Handler
import android.os.Looper
import android.os.ParcelUuid
import com.scmessenger.android.utils.BackoffStrategy
import kotlinx.coroutines.*
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import timber.log.Timber
import java.util.UUID
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.atomic.AtomicInteger
import java.util.concurrent.atomic.AtomicLong

/**
 * Handles Bluetooth Low Energy scanning for mesh peers.
 *
 * Features:
 * - Duty-cycle management (scan window/interval configurable)
 * - Background vs foreground scan mode switching
 * - Scan result caching to avoid duplicate processing
 * - Configurable scan settings based on AutoAdjustEngine profile
 * - Scans for devices advertising the SCMessenger Service UUID (0xDF01)
 */
class BleScanner(
    private val context: Context,
    private val onPeerDiscovered: (String) -> Unit,
    private val onDataReceived: (String, ByteArray) -> Unit,
    private val quotaManager: BleQuotaManager = BleQuotaManager(),
    private val backoffStrategy: BackoffStrategy = BackoffStrategy(),
    private val onScanFailure: (() -> Unit)? = null
) {
    data class BleDiscoveryStats(
        val advertisementsSeen: Int,
        val peersDiscovered: Int,
        val scanFailures: Int,
        val peerCacheSize: Int
    )

    private val bluetoothManager by lazy { context.getSystemService(Context.BLUETOOTH_SERVICE) as? BluetoothManager }
    private val bluetoothAdapter by lazy { bluetoothManager?.adapter }
    private val scanner by lazy { bluetoothAdapter?.bluetoothLeScanner }

    // Scan session management
    private var currentScanSession: android.bluetooth.le.BluetoothLeScanner? = null
    private val scanLock = Mutex()

    private var isScanning = false
    private var isBackgroundMode = false

    // Duty cycle management
    private var scanWindowMs: Long = 10000L  // 10 seconds
    private var scanIntervalMs: Long = 30000L  // 30 seconds
    private val handler by lazy { Handler(Looper.getMainLooper()) }
    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    private var dutyCycleRunnable: Runnable? = null

    // Scan result caching to avoid duplicate processing
    private val recentlySeenPeers = ConcurrentHashMap<String, Long>()
    private val peerCacheTimeoutMs = 5000L  // 5 seconds
    private val advertisementsSeen = AtomicInteger(0)
    private val peersDiscoveredCount = AtomicInteger(0)
    private val scanFailures = AtomicInteger(0)
    private val lastMatchedAdvertisementAtMs = AtomicLong(0L)
    private val scanSessionStartedAtMs = AtomicLong(0L)
    private var fallbackScanEnabled = false
    private var fallbackPromotionRunnable: Runnable? = null

    // Android 12+ Scan Quota Management — delegated to BleQuotaManager

    // SCMessenger Service UUID: 0xDF01
    // Full UUID: 0000DF01-0000-1000-8000-00805F9B34FB
    companion object {
        val SERVICE_UUID = UUID.fromString("0000DF01-0000-1000-8000-00805F9B34FB")
        val PARCEL_UUID = ParcelUuid(SERVICE_UUID)

        // Scan modes
        // Foreground: continuous scan (window == interval) — no off-window dead time.
        // Android 7+ enforces a scan-restart quota (5 starts in 30s); keeping the
        // scanner running continuously avoids the quota and maximises discovery speed.
        const val DEFAULT_SCAN_WINDOW_MS = 10000L
        const val DEFAULT_SCAN_INTERVAL_MS = 30000L
        const val FOREGROUND_SCAN_WINDOW_MS = 30000L    // continuous: window == interval
        const val FOREGROUND_SCAN_INTERVAL_MS = 30000L  // no pause in foreground
        const val BACKGROUND_SCAN_WINDOW_MS = 5000L
        const val BACKGROUND_SCAN_INTERVAL_MS = 60000L
        private const val FALLBACK_SCAN_PROMOTION_DELAY_MS = 20_000L
        private const val ADVERTISED_NAME = "SCMesh"
    }

    // --- Named callback methods wired from ScanCallback ---

    /**
     * Process a BLE scan result, extracting peer ID and service data.
     * Wired from ScanCallback.onScanResult.
     * Filters for mesh advertisements, deduplicates, and notifies discovery.
     */
    fun onScanResult(@Suppress("UNUSED_PARAMETER") callbackType: Int, result: ScanResult?) {
        result?.let { scanResult ->
            val device = scanResult.device
            val peerId = device.address
            if (!matchesMeshAdvertisement(scanResult)) {
                return
            }

            advertisementsSeen.incrementAndGet()
            lastMatchedAdvertisementAtMs.set(System.currentTimeMillis())

            // Check if we've recently seen this peer
            val now = System.currentTimeMillis()
            val lastSeen = recentlySeenPeers[peerId]
            if (lastSeen != null && (now - lastSeen) < peerCacheTimeoutMs) {
                // Skip - we've processed this peer recently
                return
            }

            // Update cache
            recentlySeenPeers[peerId] = now
            peersDiscoveredCount.incrementAndGet()

            // Prune old entries
            pruneOldPeers(now)

            val rssi = scanResult.rssi
            val scanRecord = scanResult.scanRecord

            // Extract Service Data
            val serviceData = scanRecord?.getServiceData(PARCEL_UUID)

            if (serviceData != null) {
                Timber.v("Discovered peer: $peerId (RSSI: $rssi, Data: ${serviceData.size} bytes)")
                // Notify discovery
                onPeerDiscovered(peerId)
                // Notify data received
                onDataReceived(peerId, serviceData)
            } else {
                // Just discovery (legacy or beacon)
                Timber.v("Discovered peer (no data): $peerId (RSSI: $rssi)")
                onPeerDiscovered(peerId)
            }
        }
    }

    /**
     * Handle BLE scan failure with proper error classification and retry logic.
     * Wired from ScanCallback.onScanFailed.
     * Logs the error code, notifies transport manager, and schedules retry with backoff.
     */
    fun onScanFailed(errorCode: Int) {
        val errorDescription = when (errorCode) {
            ScanCallback.SCAN_FAILED_ALREADY_STARTED -> "SCAN_FAILED_ALREADY_STARTED"
            ScanCallback.SCAN_FAILED_APPLICATION_REGISTRATION_FAILED -> "SCAN_FAILED_APPLICATION_REGISTRATION_FAILED"
            ScanCallback.SCAN_FAILED_INTERNAL_ERROR -> "SCAN_FAILED_INTERNAL_ERROR"
            ScanCallback.SCAN_FAILED_FEATURE_UNSUPPORTED -> "SCAN_FAILED_FEATURE_UNSUPPORTED"
            else -> "UNKNOWN_ERROR_$errorCode"
        }

        Timber.e("BLE Scan failed with error code: $errorCode ($errorDescription)")
        scanFailures.incrementAndGet()
        isScanning = false

        // Notify transport manager of BLE failure for graceful degradation
        onScanFailure?.invoke()

        // Schedule retry with exponential backoff for recoverable errors
        when (errorCode) {
            ScanCallback.SCAN_FAILED_ALREADY_STARTED -> {
                Timber.w("BLE scan already started; stopping existing scan before retry")
                scope.launch {
                    try {
                        stopScanning()
                        delay(500)
                        startScanning()
                    } catch (e: Exception) {
                        Timber.e(e, "Failed to restart BLE scan after already-started error")
                    }
                }
            }
            ScanCallback.SCAN_FAILED_APPLICATION_REGISTRATION_FAILED -> {
                Timber.w("BLE scan app registration failed; scheduling retry")
                scope.launch {
                    handleScanFailure(Exception("Scan failed: application registration failed (error $errorCode)"))
                }
            }
            ScanCallback.SCAN_FAILED_FEATURE_UNSUPPORTED -> {
                Timber.e("BLE scan feature unsupported on this device; cannot retry")
                // No retry for unsupported feature
            }
            else -> {
                scope.launch {
                    handleScanFailure(Exception("Scan failed with error code: $errorCode ($errorDescription)"))
                }
            }
        }
    }

    /**
     * Handle scan failure with proper backoff and retry logic.
     * Schedules a retry after the current backoff delay.
     */
    suspend fun handleScanFailure(e: Exception): Boolean {
        Timber.e(e, "BLE scan failed")
        // On scan failure, trigger backoff and schedule retry
        isScanning = false
        val retryDelay = backoffStrategy.nextDelay()
        Timber.w("Scheduling BLE scan retry in ${retryDelay}ms due to failure")
        handler.postDelayed({
            scope.launch {
                try {
                    startScanning()
                } catch (e: Exception) {
                    Timber.e(e, "Failed to restart BLE scan after failure")
                }
            }
        }, retryDelay)
        return false
    }

    /**
     * Apply BLE scan settings based on battery/motion state.
     * Adjusts scan duty cycle (window/interval) based on AutoAdjust profile.
     * Restarts scanning if currently active to apply new settings.
     */
    fun applyScanSettings(scanIntervalMs: UInt) {
        // Convert AutoAdjust interval to duty cycle
        val window = minOf(scanIntervalMs.toLong(), 20000L)
        val interval = maxOf(scanIntervalMs.toLong(), window + 5000L)

        setScanDutyCycle(window, interval)
    }

    // --- End of named callback methods ---

    private val scanCallback = object : ScanCallback() {
        override fun onScanResult(callbackType: Int, result: ScanResult?) {
            this@BleScanner.onScanResult(callbackType, result)
        }

        override fun onScanFailed(errorCode: Int) {
            this@BleScanner.onScanFailed(errorCode)
        }
    }

    /**
     * Set scan duty cycle parameters.
     */
    fun setScanDutyCycle(windowMs: Long, intervalMs: Long) {
        scanWindowMs = windowMs
        scanIntervalMs = intervalMs
        Timber.d("Scan duty cycle updated: window=${windowMs}ms, interval=${intervalMs}ms")

        // Restart scanning if active
        if (isScanning) {
            scope.launch {
                stopScanning()
                startScanning()
            }
        }
    }

    /**
     * Switch to background scan mode (lower duty cycle).
     */
    fun setBackgroundMode(background: Boolean) {
        if (isBackgroundMode == background) return

        isBackgroundMode = background

        if (background) {
            setScanDutyCycle(BACKGROUND_SCAN_WINDOW_MS, BACKGROUND_SCAN_INTERVAL_MS)
        } else {
            setScanDutyCycle(FOREGROUND_SCAN_WINDOW_MS, FOREGROUND_SCAN_INTERVAL_MS)
        }

        Timber.i("Scan mode changed: background=$background")
    }

    @SuppressLint("MissingPermission")
    suspend fun startScanning(): Boolean = scanLock.withLock {
        if (scanner == null) {
            Timber.w("Bluetooth Scanner not available")
            return@withLock false
        }

        // Scan session reuse: if already scanning, don't restart (prevents SCAN_FAILED_ALREADY_STARTED)
        if (isScanning) {
            Timber.d("BLE scan already in progress, reusing existing session")
            return@withLock true
        }

        // Initialize current scan session if needed
        if (currentScanSession == null) {
            currentScanSession = bluetoothAdapter?.bluetoothLeScanner
        }

        val session = currentScanSession
        if (session == null) {
            Timber.w("Bluetooth Scanner not available")
            return@withLock false
        }

        // Check if Bluetooth is enabled
        if (bluetoothAdapter?.isEnabled != true) {
            Timber.w("Bluetooth is not enabled, cannot start scanning")
            return@withLock false
        }

        // Android 12+ Quota Management via BleQuotaManager
        val quotaDelay = quotaManager.checkQuota()
        if (quotaDelay > 0) {
            // Don't set isScanning = false when quota is exhausted
            // Keep the scanning state as false, and schedule retry after quota cooldown
            handler.postDelayed({
                scope.launch {
                    try {
                        startScanning()
                    } catch (e: Exception) {
                        Timber.e(e, "Failed to restart BLE scan after quota delay")
                    }
                }
            }, quotaDelay)
            Timber.w("BLE scan quota exhausted, retrying in ${quotaDelay}ms")
            return@withLock false
        }

        advertisementsSeen.set(0)
        fallbackScanEnabled = false
        scanSessionStartedAtMs.set(System.currentTimeMillis())
        lastMatchedAdvertisementAtMs.set(0L)

        return try {
            // Stop any existing scan first to avoid SCAN_FAILED_ALREADY_STARTED
            // This is safe even if no scan is running (idempotent)
            try {
                session.stopScan(scanCallback)
            } catch (_: Exception) {
                // Ignore errors from stopping non-existent scan
            }

            session.startScan(currentFilters(), buildScanSettings(), scanCallback)
            isScanning = true
            quotaManager.recordScanStart()
            backoffStrategy.reset()
            Timber.i("BLE Scanning started (background=$isBackgroundMode, fallback=$fallbackScanEnabled)")
            scheduleFallbackPromotion()

            // Start duty cycle if intervals are configured
            if (scanWindowMs < scanIntervalMs) {
                startDutyCycle()
            }

            true
        } catch (e: Exception) {
            Timber.e(e, "Failed to start BLE scan")
            // On scan failure, trigger backoff and schedule retry
            isScanning = false
            val retryDelay = backoffStrategy.nextDelay()
            Timber.w("Scheduling BLE scan retry in ${retryDelay}ms due to failure")
            handler.postDelayed({
                scope.launch {
                    try {
                        startScanning()
                    } catch (e: Exception) {
                        Timber.e(e, "Failed to restart BLE scan after failure")
                    }
                }
            }, retryDelay)
            false
        }
    }

    private fun startDutyCycle() {
        // Cancel any existing duty cycle
        stopDutyCycle()

        dutyCycleRunnable = object : Runnable {
            override fun run() {
                if (isScanning) {
                    // Stop scanning for the rest of the interval
                    stopScanningInternal()

                    // Schedule restart after pause
                    handler.postDelayed({
                        if (isScanning) {
                            startScanningInternal()
                        }
                    }, scanIntervalMs - scanWindowMs)
                }

                // Schedule next cycle
                if (isScanning) {
                    handler.postDelayed(this, scanIntervalMs)
                }
            }
        }

        // Start first cycle after scan window
        val runnable = dutyCycleRunnable ?: return
        handler.postDelayed(runnable, scanWindowMs)
        Timber.d("Duty cycle started: ${scanWindowMs}ms scan / ${scanIntervalMs}ms interval")
    }

    private fun stopDutyCycle() {
        dutyCycleRunnable?.let { handler.removeCallbacks(it) }
        dutyCycleRunnable = null
    }

    private fun currentFilters(): List<ScanFilter> {
        return if (fallbackScanEnabled) {
            emptyList()
        } else {
            listOf(
                ScanFilter.Builder()
                    .setServiceUuid(ParcelUuid(SERVICE_UUID))
                    .build()
            )
        }
    }

    private fun buildScanSettings(): ScanSettings {
        val scanMode = if (isBackgroundMode) {
            ScanSettings.SCAN_MODE_LOW_POWER
        } else {
            ScanSettings.SCAN_MODE_LOW_LATENCY
        }

        return ScanSettings.Builder()
            .setScanMode(scanMode)
            .setMatchMode(ScanSettings.MATCH_MODE_AGGRESSIVE)
            .setCallbackType(ScanSettings.CALLBACK_TYPE_ALL_MATCHES)
            .setNumOfMatches(ScanSettings.MATCH_NUM_ONE_ADVERTISEMENT)
            .build()
    }

    private fun scheduleFallbackPromotion() {
        fallbackPromotionRunnable?.let { handler.removeCallbacks(it) }
        fallbackPromotionRunnable = Runnable {
            if (!isScanning || fallbackScanEnabled) {
                return@Runnable
            }
            if (advertisementsSeen.get() > 0 || lastMatchedAdvertisementAtMs.get() > 0L) {
                return@Runnable
            }
            val elapsedMs = System.currentTimeMillis() - scanSessionStartedAtMs.get()
            if (elapsedMs < FALLBACK_SCAN_PROMOTION_DELAY_MS) {
                return@Runnable
            }
            fallbackScanEnabled = true
            Timber.w(
                "BLE scan fallback enabled after %d ms without mesh advertisements; switching to unfiltered scan",
                elapsedMs
            )
            restartActiveScan()
        }
        handler.postDelayed(fallbackPromotionRunnable!!, FALLBACK_SCAN_PROMOTION_DELAY_MS)
    }

    @SuppressLint("MissingPermission")
    private fun restartActiveScan() {
        val s = scanner
        if (s == null || !isScanning) return

        try {
            s.stopScan(scanCallback)
        } catch (_: Exception) {
        }

        try {
            s.startScan(currentFilters(), buildScanSettings(), scanCallback)
            Timber.i("BLE scan restarted (background=$isBackgroundMode, fallback=$fallbackScanEnabled)")
        } catch (e: Exception) {
            Timber.e(e, "Failed to restart BLE scan")
        }
    }

    private fun matchesMeshAdvertisement(result: ScanResult): Boolean {
        val record = result.scanRecord
        val serviceUuidMatch = record?.serviceUuids?.any { it.uuid == SERVICE_UUID } == true
        val serviceDataMatch = record?.getServiceData(PARCEL_UUID) != null
        val advertisedName = record?.deviceName?.trim()
        val deviceName = try { result.device.name?.trim() } catch (_: SecurityException) { null }
        val nameMatch = advertisedName == ADVERTISED_NAME || deviceName == ADVERTISED_NAME

        return serviceUuidMatch || serviceDataMatch || nameMatch
    }

    @SuppressLint("MissingPermission")
    private fun startScanningInternal() {
        val s = scanner
        if (s == null || isScanning) return

        try {
            s.startScan(currentFilters(), buildScanSettings(), scanCallback)
            isScanning = true
            Timber.v("BLE scan window started")
        } catch (e: Exception) {
            Timber.e(e, "Failed to restart BLE scan")
            // Set isScanning to false so retry logic can trigger
            isScanning = false
        }
    }

    @SuppressLint("MissingPermission")
    private fun stopScanningInternal() {
        val s = scanner
        if (s == null) return

        try {
            s.stopScan(scanCallback)
            Timber.v("BLE scan window ended")
        } catch (e: Exception) {
            Timber.e(e, "Failed to stop BLE scan window")
        }
    }
    
    /**
     * Force restart scanning after a failure with proper backoff.
     * This is called when scan fails and we need to recover.
     * The scan will be scheduled after the current backoff delay.
     */
    fun forceRestartScanning() {
        Timber.i("Force restarting BLE scanning with backoff")
        isScanning = false
        val retryDelay = backoffStrategy.nextDelay()
        Timber.w("Scheduling BLE scan force restart in ${retryDelay}ms")
        handler.postDelayed({
            scope.launch {
                try {
                    startScanning()
                } catch (e: Exception) {
                    Timber.e(e, "Failed to restart BLE scan after force restart")
                }
            }
        }, retryDelay)
    }

    @SuppressLint("MissingPermission")
    suspend fun stopScanning() = scanLock.withLock {
        stopScanningLocked()
    }

    /**
     * Internal stop-scan logic that MUST be called while holding the scanLock.
     */
    @SuppressLint("MissingPermission")
    private fun stopScanningLocked() {
        if (currentScanSession == null || !isScanning) {
            // P1_ANDROID_022: even if we are not actively scanning, ensure any stale
            // peer cache is purged so re-discovery can occur on the next session.
            clearPeerCache()
            return
        }

        stopDutyCycle()
        fallbackPromotionRunnable?.let { handler.removeCallbacks(it) }
        fallbackPromotionRunnable = null

        try {
            currentScanSession?.stopScan(scanCallback)
            isScanning = false
            Timber.i("BLE Scanning stopped")
        } catch (e: Exception) {
            Timber.e(e, "Failed to stop BLE scan")
        }
        // P1_ANDROID_022: drop stale cache entries on every stop so subsequent
        // discovery sessions don't suppress already-seen peers (gratuitous
        // persistence between runs). See P1_ANDROID_022_BLE_Stale_Cache_Cleanup.
        clearPeerCache()
    }

    /**
     * Pause transport: stop scanning and clear the peer cache so a subsequent
     * resume re-discovers everything (no stale entries from a previous session).
     * Called when the app is backgrounded or the user explicitly pauses BLE.
     */
    suspend fun onTransportPause() = scanLock.withLock {
        Timber.i("BLE transport paused — stopping scan and clearing peer cache")
        stopScanningLocked()
        clearPeerCache()
    }

    /**
     * Clear the peer cache to allow re-discovery.
     */
    fun clearPeerCache() {
        recentlySeenPeers.clear()
        Timber.d("Peer cache cleared")
    }

    fun getDiscoveryStats(): BleDiscoveryStats {
        return BleDiscoveryStats(
            advertisementsSeen = advertisementsSeen.get(),
            peersDiscovered = peersDiscoveredCount.get(),
            scanFailures = scanFailures.get(),
            peerCacheSize = recentlySeenPeers.size
        )
    }

    /**
     * Get the current BLE quota count from the BleQuotaManager.
     * Wired from BleQuotaManager.currentCount.
     */
    fun getQuotaCount(): Int {
        return quotaManager.currentCount()
    }

    /**
     * Prune old entries from peer cache.
     */
    private fun pruneOldPeers(currentTimeMs: Long) {
        val iterator = recentlySeenPeers.entries.iterator()
        while (iterator.hasNext()) {
            val entry = iterator.next()
            if ((currentTimeMs - entry.value) > peerCacheTimeoutMs) {
                iterator.remove()
            }
        }
    }
}
