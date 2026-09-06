package com.scmessenger.android.ui

import android.os.Bundle
import android.os.Handler
import android.os.Looper
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.ui.Modifier
import androidx.core.view.WindowCompat
import com.scmessenger.android.service.AndroidPlatformBridge
import com.scmessenger.android.service.AnrWatchdog
import com.scmessenger.android.R
import com.scmessenger.android.ui.theme.SCMessengerTheme
import dagger.hilt.android.AndroidEntryPoint
import timber.log.Timber
import android.Manifest
import android.content.pm.PackageManager
import android.os.Build
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AlertDialog
import androidx.core.content.ContextCompat
import androidx.core.splashscreen.SplashScreen.Companion.installSplashScreen
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.utils.Permissions
import com.scmessenger.android.utils.NotificationHelper
import javax.inject.Inject
import java.util.concurrent.atomic.AtomicBoolean
import androidx.activity.viewModels
import android.content.Intent
import com.scmessenger.android.ui.viewmodels.MainViewModel
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import androidx.compose.runtime.getValue
import androidx.compose.runtime.collectAsState
import androidx.compose.foundation.isSystemInDarkTheme
import com.scmessenger.android.data.PreferencesRepository

/**
 * Main activity for SCMessenger.
 *
 * This is the UI entry point, hosting the Compose navigation graph.
 */
@AndroidEntryPoint
class MainActivity : ComponentActivity() {
    @Inject
    lateinit var meshRepository: MeshRepository

    @Inject
    lateinit var platformBridge: AndroidPlatformBridge

    private val mainViewModel: MainViewModel by viewModels()

    private val permissionRequestInProgress = AtomicBoolean(false)
    private val permissionRequestDebounceMs = 500L
    private val handler = Handler(Looper.getMainLooper())

    // ANR watchdog for UI thread monitoring
    private lateinit var anrWatchdog: AnrWatchdog

    // Standalone POST_NOTIFICATIONS launcher (API 33+) with rationale dialog.
    // We use a single-permission launcher for this one because the user
    // rationale is specific to notifications, and a system dialog that
    // only asks for the single permission feels less overwhelming than a
    // batch of 6+ runtime permissions on first launch.
    private val notificationPermissionLauncher = registerForActivityResult(
        ActivityResultContracts.RequestPermission()
    ) { granted ->
        if (granted) {
            Timber.i("POST_NOTIFICATIONS granted by user")
        } else {
            Timber.w("POST_NOTIFICATIONS denied by user")
        }
    }

    // Tracks whether we already showed the notification rationale dialog
    // this session so we do not show it twice.
    private var notificationRationaleShown = false

    private val requestPermissionLauncher = registerForActivityResult(
        ActivityResultContracts.RequestMultiplePermissions()
    ) { permissions ->
        permissions.entries.forEach {
            Timber.d("Permission ${it.key} granted: ${it.value}")
        }
        val denied = permissions.filterValues { granted -> !granted }.keys
        if (denied.isNotEmpty()) {
            Timber.w("Permissions denied: $denied")
        }

        // Reset flag after handling
        schedulePermissionReset()

        if (meshRepository.hasRequiredRuntimePermissions()) {
            meshRepository.onRuntimePermissionsGranted()
            // R2-4: first-launch path — permissions were just granted through
            // this callback, so this is the moment the FGS can legally start.
            ensureMeshForegroundService()
        }
    }

    private fun schedulePermissionReset() {
        handler.postDelayed({
            permissionRequestInProgress.set(false)
            Timber.d("Permission request state reset")
        }, permissionRequestDebounceMs)
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        installSplashScreen()
        super.onCreate(savedInstanceState)

        // Enable edge-to-edge and proper IME handling
        WindowCompat.setDecorFitsSystemWindows(window, false)

        Timber.d("MainActivity created")

        // Start ANR watchdog monitoring for UI thread responsiveness
        startAnrMonitoring()

        checkPermissions()

        // Request POST_NOTIFICATIONS specifically (API 33+) with a
        // dedicated rationale dialog. This is a separate, focused flow
        // so the user understands the value before the system dialog
        // appears. Falls through silently on older API levels.
        requestNotificationPermissionIfNeeded()

        setContent {
            val themeMode by mainViewModel.themeMode.collectAsState()
            val darkTheme = when (themeMode) {
                PreferencesRepository.ThemeMode.LIGHT -> false
                PreferencesRepository.ThemeMode.DARK -> true
                PreferencesRepository.ThemeMode.SYSTEM -> isSystemInDarkTheme()
            }
            SCMessengerTheme(darkTheme = darkTheme) {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    MeshApp(mainViewModel = mainViewModel)
                }
            }
        }

        // Handle deep links for cold start
        intent?.let {
            when (it.action) {
                Intent.ACTION_VIEW -> {
                    it.data?.let { uri ->
                        Timber.d("Handling deep link on cold start: $uri")
                        mainViewModel.handleDeepLink(uri)
                    }
                }
                NotificationHelper.ACTION_OPEN_REQUESTS -> {
                    Timber.d("Opening requests inbox on cold start")
                    mainViewModel.navigateToRequestsInbox()
                }
                else -> {}
            }
        }

        // Defer heavy UI work to background using lifecycleScope (structured concurrency)
        lifecycleScope.launch {
            withContext(Dispatchers.IO) {
                initializeUiComponents()
            }
        }
    }

    /**
     * Start ANR watchdog monitoring for UI thread responsiveness.
     */
    private fun startAnrMonitoring() {
        try {
            anrWatchdog = AnrWatchdog(this)
            anrWatchdog.start()
            Timber.i("ANR watchdog started for UI thread monitoring")
        } catch (e: Exception) {
            Timber.w(e, "Failed to start ANR watchdog")
        }
    }

    /**
     * Initialize heavy UI components on a background thread.
     * Called from lifecycleScope on Dispatchers.IO to avoid blocking onCreate.
     */
    private fun initializeUiComponents() {
        try {
            // Initialize repository (heavy storage/FFI operations)
            meshRepository.initializeRepository()

            // Pre-warm compose state caches
            Timber.d("UI components initialization completed")
        } catch (e: Exception) {
            Timber.e(e, "Failed to initialize UI components")
        }
    }

    private fun checkPermissions() {
        // Prevent concurrent permission requests
        if (!permissionRequestInProgress.compareAndSet(false, true)) {
            Timber.d("Permission request already in progress, skipping")
            return
        }

        val permissions = mutableListOf(
            Manifest.permission.ACCESS_FINE_LOCATION,
            Manifest.permission.ACCESS_COARSE_LOCATION
        )

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            permissions.add(Manifest.permission.BLUETOOTH_SCAN)
            permissions.add(Manifest.permission.BLUETOOTH_ADVERTISE)
            permissions.add(Manifest.permission.BLUETOOTH_CONNECT)
        }

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            permissions.add(Manifest.permission.POST_NOTIFICATIONS)
            permissions.add(Manifest.permission.NEARBY_WIFI_DEVICES)
        }

        val toRequest = permissions.filter {
            ContextCompat.checkSelfPermission(this, it) != PackageManager.PERMISSION_GRANTED
        }

        if (toRequest.isEmpty()) {
            permissionRequestInProgress.set(false)
            Timber.d("All permissions already granted")
            return
        }

        // Determine which permissions need a rationale dialog
        val rationalePermissions = toRequest.filter {
            shouldShowRequestPermissionRationale(it)
        }

        if (rationalePermissions.isNotEmpty()) {
            showPermissionRationale(rationalePermissions, toRequest)
        } else {
            Timber.i("Requesting permissions: $toRequest")
            requestPermissionLauncher.launch(toRequest.toTypedArray())
        }
    }

    /**
     * Show a rationale dialog explaining why permissions are needed before requesting them.
     */
    private fun showPermissionRationale(
        rationalePermissions: List<String>,
        allToRequest: List<String>
    ) {
        val message = buildString {
            appendLine(getString(R.string.permissions_rationale_intro))
            appendLine()
            rationalePermissions.forEach { permission ->
                appendLine("• ${Permissions.getPermissionName(permission)}: ${Permissions.getRationale(permission)}")
            }
        }

        AlertDialog.Builder(this)
            .setTitle(R.string.permissions_rationale_title)
            .setMessage(message.trim())
            .setPositiveButton(R.string.permissions_action_grant) { _, _ ->
                Timber.i("Requesting permissions after rationale: $allToRequest")
                requestPermissionLauncher.launch(allToRequest.toTypedArray())
            }
            .setNegativeButton(R.string.cancel) { _, _ ->
                Timber.w("User cancelled permission rationale")
                schedulePermissionReset()
            }
            .setCancelable(false)
            .show()
    }

    /**
     * POST_NOTIFICATIONS (API 33+) is a special-case runtime permission
     * because the user benefit is high (message alerts) and the rationale
     * is non-obvious to people who have not used a mesh messenger before.
     *
     * Flow:
     *  - API < 33: no runtime grant needed; declared in manifest only.
     *  - API >= 33 + already granted: nothing to do.
     *  - API >= 33 + not granted + never asked: show rationale dialog,
     *    then launch the system permission dialog.
     *  - API >= 33 + not granted + previously denied: show rationale
     *    dialog with a "Go to settings" hint (handled by checking
     *    shouldShowRequestPermissionRationale). If the user has selected
     *    "Don't ask again", we just skip silently — the mesh service
     *    will still run, just without system notifications.
     */
    private fun requestNotificationPermissionIfNeeded() {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) return

        val alreadyGranted = ContextCompat.checkSelfPermission(
            this,
            Manifest.permission.POST_NOTIFICATIONS
        ) == PackageManager.PERMISSION_GRANTED

        if (alreadyGranted) {
            Timber.d("POST_NOTIFICATIONS already granted; skipping rationale")
            return
        }

        if (notificationRationaleShown) {
            // The system dialog will be re-launched from the rationale
            // dialog's positive button instead.
            return
        }
        notificationRationaleShown = true

        val needsRationale = shouldShowRequestPermissionRationale(
            Manifest.permission.POST_NOTIFICATIONS
        )

        if (needsRationale) {
            // User previously denied — explain why we need it again.
            AlertDialog.Builder(this)
                .setTitle(R.string.notification_permission_rationale_title)
                .setMessage(R.string.notification_permission_rationale_message)
                .setPositiveButton(R.string.notification_permission_request_button) { _, _ ->
                    Timber.i("Requesting POST_NOTIFICATIONS after rationale")
                    notificationPermissionLauncher.launch(Manifest.permission.POST_NOTIFICATIONS)
                }
                .setNegativeButton(R.string.notification_permission_skip_button) { _, _ ->
                    Timber.w("User declined POST_NOTIFICATIONS rationale")
                }
                .setCancelable(true)
                .show()
        } else {
            // First time — fire the system dialog directly.
            Timber.i("Requesting POST_NOTIFICATIONS (first time)")
            notificationPermissionLauncher.launch(Manifest.permission.POST_NOTIFICATIONS)
        }
    }

    override fun onResume() {
        super.onResume()
        Timber.d("MainActivity resumed")
        platformBridge.notifyForeground()
        checkPermissions()
        if (meshRepository.hasRequiredRuntimePermissions()) {
            meshRepository.onRuntimePermissionsGranted()
            ensureMeshForegroundService()
        }
    }

    /**
     * RCA-D1 (2026-09-06): the mesh service was only ever started from
     * BootReceiver, the AnrWatchdog recovery path, or the Settings retry
     * button — so a normal app launch left no foreground service running and
     * Android froze the whole process (mesh core included) once the activity
     * left the foreground. The mesh must run whenever the app runs. Starting
     * the service is idempotent: onStartCommand with ACTION_START while the
     * mesh is already running just refreshes the notification.
     */
    private fun ensureMeshForegroundService() {
        try {
            val intent = android.content.Intent(this, com.scmessenger.android.service.MeshForegroundService::class.java)
                .apply { action = com.scmessenger.android.service.MeshForegroundService.ACTION_START }
            startForegroundService(intent)
            Timber.d("Mesh foreground service ensured (ACTION_START)")
        } catch (e: Exception) {
            // R1-6: background-start restrictions and permission gaps land here.
            // Distinguish them from unexpected failures so a systematic denial
            // is visible instead of silently swallowed.
            if (e is IllegalStateException) {
                Timber.w(e, "Mesh foreground service start rejected (background-start restriction?)")
            } else {
                Timber.e(e, "Failed to start mesh foreground service")
            }
        }
    }

    override fun onNewIntent(intent: Intent?) {
        super.onNewIntent(intent)
        intent?.let {
            when (it.action) {
                Intent.ACTION_VIEW -> {
                    it.data?.let { uri ->
                        Timber.d("Handling deep link on new intent: $uri")
                        mainViewModel.handleDeepLink(uri)
                    }
                }
                NotificationHelper.ACTION_OPEN_REQUESTS -> {
                    Timber.d("Opening requests inbox on intent")
                    mainViewModel.navigateToRequestsInbox()
                }
                else -> {}
            }
        }
    }

    override fun onPause() {
        super.onPause()
        Timber.d("MainActivity paused")
        platformBridge.notifyBackground()
    }

    override fun onDestroy() {
        super.onDestroy()
        // Stop ANR watchdog when activity is destroyed
        try {
            anrWatchdog.stop()
            Timber.d("ANR watchdog stopped")
        } catch (e: Exception) {
            Timber.w(e, "Failed to stop ANR watchdog")
        }
    }
}
