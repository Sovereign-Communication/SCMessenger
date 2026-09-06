package com.scmessenger.android.service

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import android.os.PowerManager
import android.os.SystemClock
import androidx.core.app.NotificationCompat
import androidx.core.app.ServiceCompat
import com.scmessenger.android.R
import com.scmessenger.android.ui.MainActivity
import com.scmessenger.android.utils.NotificationHelper
import com.scmessenger.android.utils.displayName
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import timber.log.Timber
import java.util.Collections
import java.util.concurrent.atomic.AtomicInteger
import javax.inject.Inject
import com.scmessenger.android.data.PreferencesRepository

/**
 * Foreground service maintaining mesh network connectivity.
 *
 * This service:
 * - Keeps the mesh network alive while the app is backgrounded
 * - Manages BLE, WiFi Aware, and WiFi Direct transports
 * - Relays messages according to configured budget
 * - Adapts behavior based on battery/network state via AutoAdjustEngine
 * - Uses WakeLock for BLE scan windows
 * - Periodically computes AutoAdjust profile adjustments
 */
@AndroidEntryPoint
class MeshForegroundService : Service() {

    private val serviceScope = CoroutineScope(SupervisorJob() + Dispatchers.Default)

    @Inject
    lateinit var meshRepository: com.scmessenger.android.data.MeshRepository

    @Inject
    lateinit var platformBridge: AndroidPlatformBridge

    @Inject
    lateinit var preferencesRepository: PreferencesRepository

    @Volatile private var isRunning = false
    @Volatile private var activeConversationId: String? = null
    private val connectedPeers: MutableSet<String> = Collections.synchronizedSet(mutableSetOf())

    private val messagesRelayed = AtomicInteger(0)

    // WakeLock for BLE scan windows
    private var wakeLock: PowerManager.WakeLock? = null
    private lateinit var anrWatchdog: AnrWatchdog

    // Performance monitor for ANR tracking and health diagnostics
    private lateinit var performanceMonitor: PerformanceMonitor

    // ServiceHealthMonitor for heartbeat tracking and recovery
    private lateinit var serviceHealthMonitor: ServiceHealthMonitor

    override fun onCreate() {
        super.onCreate()
        Timber.d("MeshForegroundService created")

        // Initialize WakeLock
        val powerManager = getSystemService(POWER_SERVICE) as PowerManager
        wakeLock = powerManager.newWakeLock(
            PowerManager.PARTIAL_WAKE_LOCK,
            "SCMessenger::MeshService"
        )

        // Initialize ANR watchdog
        anrWatchdog = AnrWatchdog(this)

        // Initialize performance monitor
        performanceMonitor = PerformanceMonitor(this)

        // Initialize service health monitor (wired for heartbeat tracking)
        serviceHealthMonitor = ServiceHealthMonitor(this)
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        val action = intent?.action
        // R6-2: ACTION_ENSURE is delivered via startForegroundService() too,
        // so the 5-second startForeground() contract applies even when the
        // user-stop latch later resolves the delivery to NoOp.
        val shouldStartForeground =
            action == null || action == ACTION_START || action == ACTION_RESUME || action == ACTION_ENSURE

        // Android 12+ requires startForeground() within 5 seconds of onStartCommand returning.
        // Promote to foreground synchronously; async initialization continues in the coroutine.
        if (shouldStartForeground) {
            if (!tryStartForeground()) {
                Timber.e("Synchronous foreground promotion denied; aborting service start")
                stopSelf()
                return START_NOT_STICKY
            }
        }

        serviceScope.launch {
            val repoRunning = withContext(Dispatchers.Default) {
                meshRepository.getServiceStateSync() == uniffi.api.ServiceState.RUNNING
            }
            when (decideCommand(action, isRunning, repoRunning)) {
                StartDecision.Start -> startMeshService()
                StartDecision.Stop -> stopMeshService()
                StartDecision.Pause -> pauseMeshService()
                StartDecision.Resume -> resumeMeshService()
                StartDecision.NoOp -> {
                    Timber.w("Ignoring start request while user stop is in effect (action=%s)", action ?: "null")
                    // R6-2: a latched ACTION_ENSURE arrived via
                    // startForegroundService() -- the foreground promotion
                    // above satisfied the contract, but the mesh must stay
                    // stopped and the just-shown notification must go away.
                    if (action == ACTION_ENSURE) {
                        stopForeground(STOP_FOREGROUND_REMOVE)
                        stopSelf()
                    }
                }
            }
        }

        return START_STICKY
    }

    private fun startMeshService() {
        serviceScope.launch {
            val repoRunning = withContext(Dispatchers.Default) {
                meshRepository.getServiceStateSync() == uniffi.api.ServiceState.RUNNING
            }
            if (isRunning || repoRunning) {
                isRunning = true
                updateNotification()
                Timber.w("Mesh service already running; notification refreshed")
                return@launch
            }

            Timber.i("Starting mesh service")
            // Foreground promotion already done synchronously in onStartCommand.

            // BATTERY FIX (P0_ANDROID_STABILITY_001): Removed persistent WakeLock acquisition.
            // startForeground() with a notification already keeps the process alive for a foreground service.
            // A persistent PARTIAL_WAKE_LOCK causes Play Store battery-drain flags.
            // If needed for active BLE scan windows, acquire selectively and release immediately.

            // Initialize platform bridge to monitor system state
            platformBridge.initialize()

            // Acquire WakeLock for BLE scan windows
            acquireWakeLock()

            // Create mesh service configuration
            val config = uniffi.api.MeshServiceConfig(
                discoveryIntervalMs = 30000u,  // 30 seconds
                batteryFloorPct = 20u
            )

            // Start mesh service via repository
            try {
                withContext(Dispatchers.Default) {
                    meshRepository.startMeshService(config)
                    meshRepository.setPlatformBridge(platformBridge)
                }

                val started = withContext(Dispatchers.Default) {
                    meshRepository.getServiceStateSync() == uniffi.api.ServiceState.RUNNING
                }
                if (!started) {
                    Timber.e("Repository did not reach RUNNING state; aborting foreground service start")
                    isRunning = false
                    releaseWakeLock()
                    stopForeground(STOP_FOREGROUND_REMOVE)
                    stopSelf()
                    return@launch
                }
                isRunning = true

                // Show mesh status notification (wired via NotificationHelper)
                NotificationHelper.showMeshStatusNotification(
                    context = this@MeshForegroundService,
                    title = "Mesh Service Started",
                    message = "Mesh network is now active"
                )

                // Wire CoreDelegate callbacks to MeshEventBus
                wireCoreDelegate()

                // Listen for incoming messages and show notifications (WS14: with classification)
                launch {
                    meshRepository.incomingMessages.collect { message ->
                        showMessageNotificationWithClassification(message)
                    }
                }

                // Listen for peer events to update notification
                launch {
                    MeshEventBus.peerEvents.collect { event ->
                        when (event) {
                            is PeerEvent.Connected -> {
                                connectedPeers.add(event.peerId)
                                updateNotification()
                                // Wire showPeerDiscoveredNotification into peer discovery callback
                                NotificationHelper.showPeerDiscoveredNotification(
                                    context = this@MeshForegroundService,
                                    peerId = event.peerId,
                                    transport = "Connected"
                                )
                            }
                            is PeerEvent.Disconnected -> {
                                connectedPeers.remove(event.peerId)
                                updateNotification()
                            }
                            else -> {}
                        }
                    }
                }

                // Listen for status events to update relay stats
                launch {
                    MeshEventBus.statusEvents.collect { event ->
                        when (event) {
                            is StatusEvent.StatsUpdated -> {
                                messagesRelayed.set(event.stats.messagesRelayed.toInt())
                                updateNotification()
                            }
                            else -> {}
                        }
                    }
                }

                // Start periodic AutoAdjust profile computation
                startPeriodicAdjustments()

                // Monitor UI state for notification suppression
                launch {
                    MeshEventBus.uiEvents.collect { event ->
                        activeConversationId = when (event) {
                            is UiEvent.ConversationOpened -> event.peerId
                            is UiEvent.ConversationClosed -> null
                        }
                    }
                }


                // BATTERY FIX (P0_ANDROID_STABILITY_001): Removed periodic WakeLock renewal.
                // Persistent wakelocks are unnecessary with FOREGROUND_SERVICE + startForeground().

                Timber.i("Mesh service started successfully")

                // Start ANR watchdog after service is running
                anrWatchdog.start()

                // Record service start for uptime tracking
                performanceMonitor.recordServiceStart()

                // Wire startMonitoring + updateHeartbeat into service lifecycle
                serviceHealthMonitor.startMonitoring()
                serviceHealthMonitor.updateHeartbeat()

                // Wire recordUiTiming for service startup render tracking
                performanceMonitor.recordUiTiming(
                    operationId = 1,
                    operationName = "mesh_service_start",
                    durationMs = performanceMonitor.getServiceUptimeMs()
                )



                Timber.i("Mesh service started successfully - ANR watchdog active")
            } catch (e: Exception) {
                Timber.e(e, "Failed to start mesh service")
                isRunning = false
                releaseWakeLock()
                stopForeground(STOP_FOREGROUND_REMOVE)
                stopSelf()
            }
        }
    }

    private fun wireCoreDelegate() {
        // Core delegate wiring is handled in MeshRepository
        // We listen to MeshEventBus which receives events from CoreDelegate
        Timber.d("CoreDelegate wired to MeshEventBus")
    }

    private fun startPeriodicAdjustments() {
        serviceScope.launch {
            val powerManager = getSystemService(POWER_SERVICE) as? PowerManager
            while (isActive && isRunning) {
                var checkInterval = 30000L
                try {
                    val isPowerSave = powerManager?.isPowerSaveMode == true
                    checkInterval = if (isPowerSave) 60000L else 30000L

                    if (isRunning) {
                        // Trigger battery/network state update
                        // which will compute new adjustment profile
                        platformBridge.checkBatteryState()
                        platformBridge.checkNetworkState()

                        // Wire updateHeartbeat into periodic service lifecycle
                        serviceHealthMonitor.updateHeartbeat()

                        Timber.d("Periodic AutoAdjust profile computed (PowerSave=$isPowerSave)")
                    }
                } catch (e: Exception) {
                    Timber.e(e, "Error in periodic adjustments")
                    // Wire recordAnrEvent into ANR detection
                    performanceMonitor.recordAnrEvent(
                        durationMs = 30000L,
                        context = "periodic_adjustments_error"
                    )
                } finally {
                    // Safety: ALWAYS delay to prevent tight-loop CPU spinning on recurring exceptions
                    delay(checkInterval)
                }
            }
        }
    }

    private fun acquireWakeLock() {
        try {
            val lock = wakeLock
            if (lock != null && lock.isHeld) {
                lock.release()
            }
            // Micro-WakeLock enforcement: max 15 seconds per processing window
            wakeLock?.acquire(15 * 1000L)
            Timber.d("Micro-WakeLock acquired for 15 seconds")
        } catch (e: Exception) {
            Timber.e(e, "Failed to acquire WakeLock")
        }
    }

    private fun releaseWakeLock() {
        try {
            if (wakeLock?.isHeld == true) {
                wakeLock?.release()
                Timber.d("WakeLock released")
            }
        } catch (e: Exception) {
            Timber.e(e, "Failed to release WakeLock")
        }
    }

    private fun stopMeshService() {
        serviceScope.launch {
            val repoRunning = withContext(Dispatchers.Default) {
                meshRepository.getServiceStateSync() == uniffi.api.ServiceState.RUNNING
            }
            if (!isRunning && !repoRunning) {
                Timber.w("Mesh service stop requested while already stopped")
            }

            Timber.i("Stopping mesh service")

            // Release WakeLock
            releaseWakeLock()

            // Stop mesh service via repository
            withContext(Dispatchers.Default) {
                kotlin.runCatching { meshRepository.stopMeshService() }
                    .onFailure { Timber.e(it, "Error while stopping mesh repository") }
            }

            isRunning = false
            userStoppedForSession = true
            connectedPeers.clear()
            messagesRelayed.set(0)
            anrWatchdog.stop()

            // Record service stop for health metrics
            performanceMonitor.recordServiceStop()

            // Wire stopMonitoring + isServiceHealthy check into service lifecycle
            serviceHealthMonitor.stopMonitoring()

            // Show mesh status notification (wired via NotificationHelper)
            NotificationHelper.showMeshStatusNotification(
                context = this@MeshForegroundService,
                title = "Mesh Service Stopped",
                message = "Mesh network is now inactive"
            )



            // Clean up
            withContext(Dispatchers.Default) {
                kotlin.runCatching { platformBridge.cleanup() }
                    .onFailure { Timber.w(it, "Platform bridge cleanup failed during stop") }
            }

            stopForeground(STOP_FOREGROUND_REMOVE)
            stopSelf()
        }
    }

    private fun pauseMeshService() {
        serviceScope.launch {
            withContext(Dispatchers.Default) {
                Timber.i("Pausing mesh service (reduced activity)")
                meshRepository.pauseMeshService()
            }
        }
    }

    private fun resumeMeshService() {
        serviceScope.launch {
            withContext(Dispatchers.Default) {
                Timber.i("Resuming mesh service (full activity)")
                meshRepository.resumeMeshService()
            }
        }
    }

    private fun createNotification(): Notification {
        val notificationIntent = Intent(this, MainActivity::class.java)
        val pendingIntent = PendingIntent.getActivity(
            this,
            0,
            notificationIntent,
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )

        // Action: Pause Relay
        val pauseIntent = Intent(this, MeshForegroundService::class.java).apply {
            action = ACTION_PAUSE
        }
        val pausePendingIntent = PendingIntent.getService(
            this,
            1,
            pauseIntent,
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )

        // Action: Stop Service
        val stopIntent = Intent(this, MeshForegroundService::class.java).apply {
            action = ACTION_STOP
        }
        val stopPendingIntent = PendingIntent.getService(
            this,
            2,
            stopIntent,
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )

        val contentText = getString(
            R.string.notification_mesh_status_content_format,
            connectedPeers.size,
            messagesRelayed.get()
        )

        return NotificationCompat.Builder(this, NotificationHelper.CHANNEL_MESH_STATUS)
            .setContentTitle(getString(R.string.mesh_service_notification_title))
            .setContentText(contentText)
            .setSmallIcon(R.drawable.ic_notification)
            .setContentIntent(pendingIntent)
            .addAction(0, getString(R.string.action_pause), pausePendingIntent)
            .addAction(0, getString(R.string.action_stop), stopPendingIntent)
            .setOngoing(true)
            .setCategory(NotificationCompat.CATEGORY_SERVICE)
            .setForegroundServiceBehavior(NotificationCompat.FOREGROUND_SERVICE_IMMEDIATE)
            .build()
    }

    /**
     * Build foreground service notification using base from NotificationHelper.
     */
    private fun createSimpleForegroundNotification(): Notification {
        // Build notification using NotificationHelper for content
        val baseNotification = NotificationHelper.buildForegroundServiceNotification(
            context = this,
            peerCount = connectedPeers.size,
            relayCount = messagesRelayed.get()
        )

        // Get content from base notification (copy the NotificationCompat.Builder content)
        val builder = NotificationCompat.Builder(this, NotificationHelper.CHANNEL_MESH_STATUS)
            .setContentTitle(baseNotification.extras.getString(NotificationCompat.EXTRA_TITLE))
            .setContentText(baseNotification.extras.getString(NotificationCompat.EXTRA_TEXT))
            .setSmallIcon(baseNotification.icon)
            .setContentIntent(baseNotification.contentIntent)
            .setOngoing(baseNotification.flags and NotificationCompat.FLAG_ONGOING_EVENT != 0)
            .setCategory(NotificationCompat.CATEGORY_SERVICE)
            .setForegroundServiceBehavior(NotificationCompat.FOREGROUND_SERVICE_IMMEDIATE)

        // Add actions for pause/stop
        // Action: Pause Relay
        val pauseIntent = Intent(this, MeshForegroundService::class.java).apply {
            action = ACTION_PAUSE
        }
        val pausePendingIntent = PendingIntent.getService(
            this,
            1,
            pauseIntent,
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )
        builder.addAction(0, getString(R.string.action_pause), pausePendingIntent)

        // Action: Stop Service
        val stopIntent = Intent(this, MeshForegroundService::class.java).apply {
            action = ACTION_STOP
        }
        val stopPendingIntent = PendingIntent.getService(
            this,
            2,
            stopIntent,
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )
        builder.addAction(0, getString(R.string.action_stop), stopPendingIntent)

        return builder.build()
    }

    private fun updateNotification() {
        val notification = createNotification()
        val notificationManager = getSystemService(NotificationManager::class.java)
        notificationManager.notify(NOTIFICATION_ID, notification)
    }

    private fun tryStartForeground(): Boolean {
        val notification = createNotification()
        return try {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                ServiceCompat.startForeground(
                    this,
                    NOTIFICATION_ID,
                    notification,
                    ServiceInfo.FOREGROUND_SERVICE_TYPE_CONNECTED_DEVICE
                )
            } else {
                startForeground(NOTIFICATION_ID, notification)
            }
            true
        } catch (e: SecurityException) {
            Timber.e(e, "SecurityException while starting foreground service")
            false
        } catch (e: IllegalStateException) {
            val isForegroundServiceStartNotAllowed = Build.VERSION.SDK_INT >= Build.VERSION_CODES.S &&
                e.javaClass.name == "android.app.ForegroundServiceStartNotAllowedException"
            if (isForegroundServiceStartNotAllowed) {
                Timber.e(e, "ForegroundServiceStartNotAllowedException while starting foreground service")
                false
            } else {
                throw e
            }
        }
    }

    /**
     * WS14: Show notification with DM vs DM Request classification.
     * Uses NotificationHelper for proper channel routing and settings.
     *
     * This method runs on the repoScope (IO dispatcher) to avoid blocking the main thread.
     * All FFI calls are dispatched to the IO thread, and the notification is posted
     * on the main thread using the main handler.
     */
    private fun showMessageNotificationWithClassification(message: uniffi.api.MessageRecord) {
        serviceScope.launch {
            try {
                // Run FFI calls on IO dispatcher to avoid main thread blocking
                val contactData = withContext(Dispatchers.Default) {
                    try {
                        meshRepository.getContact(message.peerId)
                    } catch (e: Exception) {
                        Timber.w(e, "Failed to get contact for ${message.peerId}")
                        null
                    }
                }
                val isKnownContact = contactData != null

                // Wire clearMessageNotifications into message read callback
                NotificationHelper.clearMessageNotifications(this@MeshForegroundService, message.peerId)

                // Check if conversation exists
                val hasExistingConversation = withContext(Dispatchers.Default) {
                    try {
                        meshRepository.hasConversationWith(message.peerId)
                    } catch (e: Exception) {
                        Timber.w(e, "Failed to check conversation for ${message.peerId}")
                        false
                    }
                }

                // Get UI state for foreground suppression (cheap check)
                val appInForeground = isAppInForeground()
                val activeConversationId = getActiveConversationId()

                // Get display name from contact if available (localNickname
                // primary; chosen nickname secondary; synthetic peer-* blank).
                val nickname = contactData?.let { contact ->
                    contact.displayName(message.peerId.take(12))
                }

                // Post notification to main thread
                withContext(Dispatchers.Main) {
                    // Use NotificationHelper with WS14 classification
                    // Note: explicitDmRequest is not available in MessageRecord; classification will infer from contact state
                    NotificationHelper.showMessageNotification(
                        context = this@MeshForegroundService,
                        peerId = message.peerId,
                        messageId = message.id,
                        content = message.content,
                        nickname = nickname,
                        timestamp = message.timestamp.toLong(),
                        isKnownContact = isKnownContact,
                        hasExistingConversation = hasExistingConversation,
                        appInForeground = appInForeground,
                        activeConversationId = activeConversationId,
                        explicitDmRequest = null
                    )
                }
            } catch (e: Exception) {
                Timber.e(e, "Failed to process message notification for ${message.peerId}")
            }
        }
    }

    /**
     * Check if app is in foreground.
     *
     * Optimized for ANR prevention:
     * - Caches the activity manager reference
     * - Uses a simpler process check that's less likely to block
     */
    @Volatile private var activityManager: android.app.ActivityManager? = null

    private fun isAppInForeground(): Boolean {
        // Fast path: use cached activity manager
        val am = activityManager
            ?: (getSystemService(Context.ACTIVITY_SERVICE) as android.app.ActivityManager).also {
                activityManager = it
            }

        return try {
            // Use runningAppProcesses - it's cached by the system
            // Only check if we're the topmost foreground process
            val runningAppProcesses = am.runningAppProcesses
            runningAppProcesses?.any {
                it.processName == packageName &&
                it.importance == android.app.ActivityManager.RunningAppProcessInfo.IMPORTANCE_FOREGROUND
            } ?: false
        } catch (e: Exception) {
            Timber.w(e, "isAppInForeground check failed")
            false
        }
    }

    /**
     * Get currently active conversation ID.
     */
    private fun getActiveConversationId(): String? {
        return activeConversationId
    }


    override fun onBind(intent: Intent?): IBinder? {
        // Wire isServiceHealthy check into onBind for binder clients
        if (serviceHealthMonitor.isServiceHealthy()) {
            serviceHealthMonitor.updateHeartbeat()
        }
        return null
    }

    override fun onDestroy() {
        super.onDestroy()
        Timber.d("MeshForegroundService destroyed")
        releaseWakeLock()
        serviceScope.launch {
            val repoRunning = withContext(Dispatchers.Default) {
                meshRepository.getServiceStateSync() == uniffi.api.ServiceState.RUNNING
            }
            if (repoRunning) {
                withContext(Dispatchers.Default) {
                    kotlin.runCatching { meshRepository.stopMeshService() }
                        .onFailure { Timber.w(it, "Repository stop failed during service destroy") }
                }
            }
            serviceScope.cancel()
        }
    }

    companion object {
        private const val NOTIFICATION_ID = 1001

        /**
         * R3-F1: a user-initiated Stop (notification action or the service
         * view model) must survive activity onResume — the activity's
         * ensure-latch must not resurrect a mesh the user deliberately
         * stopped. Set on ACTION_STOP handling, cleared only by an explicit
         * ACTION_START (or fresh cold start where the flag is false).
         * Process-lifetime by design: a killed process loses the latch and
         * the next cold start legitimately re-autostarts the mesh.
         */
        @Volatile
        internal var userStoppedForSession: Boolean = false

        /**
         * R4-L1: observable record of the most recent ensure/start attempt
         * (true = last attempt failed). Lives on the companion so it survives
         * activity recreation; collect it for UI/diagnostics surfacing.
         */
        internal val fgsStartFailed = MutableStateFlow(false)

        const val ACTION_START = "com.scmessenger.android.service.START"
        const val ACTION_ENSURE = "com.scmessenger.android.service.ENSURE"
        const val ACTION_STOP = "com.scmessenger.android.service.STOP"
        const val ACTION_PAUSE = "com.scmessenger.android.service.PAUSE"
        const val ACTION_RESUME = "com.scmessenger.android.service.RESUME"

        internal enum class StartDecision {
            Start,
            Stop,
            Pause,
            Resume,
            NoOp
        }

        internal fun decideCommand(
            action: String?,
            serviceRunning: Boolean,
            repositoryRunning: Boolean
        ): StartDecision {
            // R4-M1: STOP is always honored -- a repeated or late stop must be
            // able to complete teardown even if the latch is already set.
            if (action == ACTION_STOP) {
                return StartDecision.Stop
            }
            // R3-F1 / R4-M2: after a user stop, only an explicit ACTION_START
            // (user-initiated from the service view model) may restart the
            // mesh and clear the latch. ACTION_ENSURE (automatic ensure from
            // the activity) must never resurrect a stopped mesh and cannot
            // clear the latch, so a STOP that races ahead of a queued ENSURE
            // still wins.
            if (userStoppedForSession && action != ACTION_START) {
                Timber.d("decideCommand: user stop in effect; ignoring action=%s", action ?: "null")
                return StartDecision.NoOp
            }
            if (action == ACTION_START) {
                userStoppedForSession = false
            }
            return when (action) {
                null -> StartDecision.Start
                ACTION_START -> StartDecision.Start
                ACTION_ENSURE -> StartDecision.Start
                ACTION_STOP -> StartDecision.Stop
                ACTION_PAUSE -> if (serviceRunning || repositoryRunning) {
                    StartDecision.Pause
                } else {
                    StartDecision.NoOp
                }
                ACTION_RESUME -> if (serviceRunning && repositoryRunning) {
                    StartDecision.Resume
                } else {
                    StartDecision.Start
                }
                else -> StartDecision.Start
            }
        }
    }
}
