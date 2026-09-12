package com.scmessenger.android.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Share
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import kotlinx.coroutines.launch
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.compose.ui.res.stringResource
import com.scmessenger.android.R
import com.scmessenger.android.network.DiagnosticsReporter
import com.scmessenger.android.ui.diagnostics.DiagnosticsBundleFormatter
import com.scmessenger.android.ui.diagnostics.DiagnosticsBundleInput
import com.scmessenger.android.ui.dialogs.NetworkStatusDialog
import com.scmessenger.android.ui.viewmodels.SettingsViewModel
import com.scmessenger.android.utils.DiagnosticsShareController
import com.scmessenger.android.utils.NotificationHelper
import com.scmessenger.android.service.PerformanceMonitor
import com.scmessenger.android.service.ServiceHealthMonitor
import com.scmessenger.android.service.AnrEvent
import com.scmessenger.android.ui.components.WarningBanner
import com.scmessenger.android.ui.components.InfoBanner
import com.scmessenger.android.ui.components.ErrorState
import timber.log.Timber

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DiagnosticsScreen(
    onNavigateBack: () -> Unit,
    viewModel: SettingsViewModel = hiltViewModel()
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    var showNetworkDiagnostics by remember { mutableStateOf(false) }
    var logText by remember { mutableStateOf("") }
    val loadingLogsText = stringResource(R.string.diagnostics_loading_logs)
    LaunchedEffect(Unit) {
        logText = loadingLogsText
    }
    var networkDiagnosticsReport by remember { mutableStateOf<DiagnosticsReporter.NetworkDiagnosticsReport?>(null) }
    var notificationStats by remember { mutableStateOf(NotificationHelper.getNotificationStats()) }
    var performanceHealthStatus by remember { mutableStateOf("") }
    var anrStats by remember { mutableStateOf("") }
    var anrEvents by remember { mutableStateOf<List<AnrEvent>>(emptyList()) }
    var serviceHealthy by remember { mutableStateOf(true) }
    var healthSummary by remember { mutableStateOf("") }

    val performanceMonitor = remember { PerformanceMonitor(context) }
    val healthMonitor = remember { ServiceHealthMonitor(context) }

    fun refreshLogs() {
        scope.launch {
            logText = viewModel.getDiagnosticsLogs(limit = 250)
        }
    }

    // Wire getNetworkDiagnosticsReport into diagnostics display
    LaunchedEffect(Unit) {
        scope.launch {
            networkDiagnosticsReport = viewModel.getNetworkDiagnosticsReport()
        }
        // Wire PerformanceMonitor.getHealthStatus into diagnostics
        performanceHealthStatus = performanceMonitor.getHealthStatus()
        // Wire PerformanceMonitor.getAnrStats into diagnostics
        anrStats = performanceMonitor.getAnrStats()
        // Wire PerformanceMonitor.getAllAnrEvents into diagnostics
        anrEvents = performanceMonitor.getAllAnrEvents()
        // Wire ServiceHealthMonitor.isServiceHealthy into diagnostics
        serviceHealthy = healthMonitor.isServiceHealthy()
        // Wire ServiceHealthMonitor health summary
        healthSummary = healthMonitor.getHealthSummary()
    }

    LaunchedEffect(Unit) {
        refreshLogs()
    }

    // Wire WarningBanner into diagnostics for health warnings
    if (!serviceHealthy) {
        WarningBanner(
            message = stringResource(R.string.diagnostics_error_service_unresponsive),
            onDismiss = {}
        )
    }

    // Wire InfoBanner into diagnostics for informational notices
    InfoBanner(
        message = stringResource(R.string.diagnostics_info_local_data),
        onDismiss = {}
    )

    if (showNetworkDiagnostics) {
        NetworkStatusDialog(
            diagnosticsReporter = DiagnosticsReporter(
                context = context,
                networkDiagnostics = com.scmessenger.android.network.NetworkDiagnostics(context),
                networkTypeDetector = com.scmessenger.android.network.NetworkTypeDetector(context),
                failureMetrics = com.scmessenger.android.utils.NetworkFailureMetrics(),
                networkDetector = com.scmessenger.android.transport.NetworkDetector(context),
                circuitBreaker = com.scmessenger.android.utils.CircuitBreaker()
            ),
            onDismiss = { showNetworkDiagnostics = false },
            onRetryBootstrap = {
                // Retry bootstrap - call the mesh repository method
                viewModel.retryBootstrap()
                showNetworkDiagnostics = false
            }
        )
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(stringResource(R.string.diagnostics_title)) },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = stringResource(R.string.chat_action_dismiss))
                    }
                },
                actions = {
                    IconButton(onClick = { refreshLogs() }) {
                        Icon(Icons.Default.Refresh, contentDescription = stringResource(R.string.diagnostics_action_refresh))
                    }
                    IconButton(onClick = {
                        viewModel.clearDiagnosticsLogs()
                        refreshLogs()
                    }) {
                        Icon(imageVector = Icons.Default.Delete, contentDescription = stringResource(R.string.diagnostics_action_clear))
                    }
                    IconButton(onClick = {
                        scope.launch {
                            // ANR-2026-09-09 recurrence fix: bundle write + URI
                            // resolution run on IO inside the controller; failures
                            // come back as a result value instead of an uncaught
                            // IllegalArgumentException on the main thread.
                            when (val result = DiagnosticsShareController.shareDiagnosticsBundle(
                                context,
                                viewModel.buildTesterDiagnosticsBundle()
                            )) {
                                is DiagnosticsShareController.DiagnosticsShareResult.Started -> Unit
                                is DiagnosticsShareController.DiagnosticsShareResult.Failed ->
                                    Timber.w("Diagnostics share failed: %s", result.reason)
                            }
                        }
                    }) {
                        Icon(Icons.Default.Share, contentDescription = stringResource(R.string.diagnostics_action_share))
                    }
                    IconButton(onClick = { showNetworkDiagnostics = true }) {
                        Icon(Icons.Default.Refresh, contentDescription = stringResource(R.string.diagnostics_action_network))
                    }
                }
            )
        }
    ) { padding ->
        Column(
            modifier = Modifier
                .padding(padding)
                .fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(16.dp)
        ) {
            // Network diagnostics card
            Card(
                modifier = Modifier.fillMaxWidth(),
                colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer)
            ) {
                Column(
                    modifier = Modifier.padding(16.dp)
                ) {
                    Text(
                        text = stringResource(R.string.diagnostics_section_network),
                        style = MaterialTheme.typography.titleMedium
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        text = stringResource(R.string.diagnostics_network_description),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Spacer(modifier = Modifier.height(12.dp))
                    Button(
                        onClick = { showNetworkDiagnostics = true },
                        modifier = Modifier.fillMaxWidth()
                    ) {
                        Text(stringResource(R.string.diagnostics_action_run_network))
                    }
                }
            }
            Spacer(modifier = Modifier.height(16.dp))

            // Network Diagnostics Report (wired via getNetworkDiagnosticsReport)
            networkDiagnosticsReport?.let { report ->
                Card(
                    modifier = Modifier.fillMaxWidth(),
                    colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.tertiaryContainer)
                ) {
                    Column(
                        modifier = Modifier.padding(16.dp)
                    ) {
                        Text(
                        text = stringResource(R.string.diagnostics_section_network_report),
                        style = MaterialTheme.typography.titleMedium
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        text = stringResource(R.string.diagnostics_report_internet_network, report.hasInternet, report.networkType),
                        style = MaterialTheme.typography.bodySmall
                    )
                    // P0_NETWORK_001 Phase 7: Transport priority
                    if (report.transportPriority.isNotEmpty()) {
                        val transportStr = report.transportPriority.joinToString(" > ") {
                            "${it.scheme}:${it.defaultPort}"
                        }
                        Text(
                            text = stringResource(R.string.diagnostics_report_transport, transportStr),
                            style = MaterialTheme.typography.bodySmall
                        )
                    }
                    // P0_NETWORK_001 Phase 7: Port probe results summary
                    if (report.portProbeResults.isNotEmpty()) {
                        val reachable = report.portProbeResults.filterValues { it }.size
                        val blocked = report.portProbeResults.filterValues { !it }.size
                        Text(
                            text = stringResource(R.string.diagnostics_report_port_probe, reachable, blocked),
                            style = MaterialTheme.typography.bodySmall
                        )
                    }
                    // P0_NETWORK_001 Phase 7: Circuit breaker summary
                    if (report.circuitBreakerEntries.isNotEmpty()) {
                        val openCount = report.circuitBreakerEntries.count {
                            it.state == com.scmessenger.android.utils.CircuitBreaker.CircuitState.OPEN
                        }
                        val halfOpenCount = report.circuitBreakerEntries.count {
                            it.state == com.scmessenger.android.utils.CircuitBreaker.CircuitState.HALF_OPEN
                        }
                        val closedCount = report.circuitBreakerEntries.count {
                            it.state == com.scmessenger.android.utils.CircuitBreaker.CircuitState.CLOSED
                        }
                        Text(
                            text = stringResource(R.string.diagnostics_report_circuits, closedCount, openCount, halfOpenCount),
                            style = MaterialTheme.typography.bodySmall
                        )
                    }
                    val reachableLabel = stringResource(R.string.diagnostics_relay_reachable)
                    val unreachableLabel = stringResource(R.string.diagnostics_relay_unreachable)
                    report.relayResults.forEach { (relay, reachable) ->
                        Text(
                            text = stringResource(R.string.diagnostics_report_relay_status, relay, if (reachable) reachableLabel else unreachableLabel),
                            style = MaterialTheme.typography.bodySmall
                        )
                    }
                    }
                }
                Spacer(modifier = Modifier.height(16.dp))
            }

            // Notification Stats (wired via getNotificationStats)
            Card(
                modifier = Modifier.fillMaxWidth(),
                colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.secondaryContainer)
            ) {
                Column(
                    modifier = Modifier.padding(16.dp)
                ) {
                    Text(
                        text = stringResource(R.string.diagnostics_section_notification_stats),
                        style = MaterialTheme.typography.titleMedium
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        text = notificationStats,
                        style = MaterialTheme.typography.bodySmall,
                        fontFamily = FontFamily.Monospace
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        Button(
                            onClick = {
                                // Wire resetNotificationStats into diagnostics
                                NotificationHelper.resetNotificationStats()
                                notificationStats = NotificationHelper.getNotificationStats()
                            },
                            modifier = Modifier.weight(1f)
                        ) {
                            Text(stringResource(R.string.diagnostics_action_reset_stats), style = MaterialTheme.typography.labelSmall)
                        }
                        Button(
                            onClick = {
                                // Wire clearAllRequestNotifications into diagnostics
                                NotificationHelper.clearAllRequestNotifications()
                                notificationStats = NotificationHelper.getNotificationStats()
                            },
                            modifier = Modifier.weight(1f)
                        ) {
                            Text(stringResource(R.string.diagnostics_action_clear_requests), style = MaterialTheme.typography.labelSmall)
                        }
                    }
                }
            }

            Spacer(modifier = Modifier.height(16.dp))

            // Service Health (wired via isServiceHealthy + getHealthSummary)
            Card(
                modifier = Modifier.fillMaxWidth(),
                colors = CardDefaults.cardColors(containerColor = if (serviceHealthy) MaterialTheme.colorScheme.primaryContainer else MaterialTheme.colorScheme.errorContainer)
            ) {
                Column(
                    modifier = Modifier.padding(16.dp)
                ) {
                    Text(
                        text = stringResource(R.string.diagnostics_section_service_health),
                        style = MaterialTheme.typography.titleMedium
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    val statusLabel = if (serviceHealthy) stringResource(R.string.diagnostics_status_healthy) else stringResource(R.string.diagnostics_status_unhealthy)
                    Text(
                        text = stringResource(R.string.settings_label_status_format, statusLabel),
                        style = MaterialTheme.typography.bodyMedium,
                        color = if (serviceHealthy) MaterialTheme.colorScheme.onPrimaryContainer else MaterialTheme.colorScheme.onErrorContainer
                    )
                    Spacer(modifier = Modifier.height(4.dp))
                    Text(
                        text = healthSummary,
                        style = MaterialTheme.typography.bodySmall,
                        fontFamily = FontFamily.Monospace
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    Button(
                        onClick = {
                            // Wire resetHealth into diagnostics settings reset
                            healthMonitor.resetHealth()
                            healthSummary = "Reset"
                            serviceHealthy = true
                            performanceMonitor.clearAnrEvents()
                            anrStats = performanceMonitor.getAnrStats()
                            anrEvents = performanceMonitor.getAllAnrEvents()
                        },
                        modifier = Modifier.fillMaxWidth()
                    ) {
                        Text(stringResource(R.string.diagnostics_action_reset_health), style = MaterialTheme.typography.labelSmall)
                    }
                }
            }

            Spacer(modifier = Modifier.height(16.dp))

            // Performance Monitor (wired via recordUiTiming, getAnrStats, getAllAnrEvents, getHealthStatus)
            Card(
                modifier = Modifier.fillMaxWidth()
            ) {
                Column(
                    modifier = Modifier.padding(16.dp)
                ) {
                    Text(
                        text = stringResource(R.string.diagnostics_section_performance_anr),
                        style = MaterialTheme.typography.titleMedium
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        text = performanceHealthStatus,
                        style = MaterialTheme.typography.bodySmall,
                        fontFamily = FontFamily.Monospace
                    )
                    Spacer(modifier = Modifier.height(4.dp))
                    Text(
                        text = anrStats,
                        style = MaterialTheme.typography.bodySmall,
                        fontFamily = FontFamily.Monospace
                    )
                    if (anrEvents.isNotEmpty()) {
                        Spacer(modifier = Modifier.height(8.dp))
                        Text(
                            text = stringResource(R.string.diagnostics_anr_events_title),
                            style = MaterialTheme.typography.labelMedium
                        )
                        anrEvents.take(5).forEach { event: AnrEvent ->
                            Text(
                                text = stringResource(R.string.diagnostics_anr_event_format, event.context, event.durationMs, event.androidVersion, event.device),
                                style = MaterialTheme.typography.bodySmall,
                                fontFamily = FontFamily.Monospace
                            )
                        }
                    }
                    Spacer(modifier = Modifier.height(8.dp))
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        Button(
                            onClick = {
                                // Wire clearAnrEvents into diagnostics reset
                                performanceMonitor.clearAnrEvents()
                                anrStats = performanceMonitor.getAnrStats()
                                anrEvents = performanceMonitor.getAllAnrEvents()
                                performanceHealthStatus = performanceMonitor.getHealthStatus()
                            },
                            modifier = Modifier.weight(1f)
                        ) {
                            Text(stringResource(R.string.diagnostics_action_clear_anr), style = MaterialTheme.typography.labelSmall)
                        }
                    }
                }
            }

            Spacer(modifier = Modifier.height(16.dp))

            // Log viewer
            Text(
                text = stringResource(R.string.diagnostics_section_logs),
                style = MaterialTheme.typography.titleSmall
            )
            Spacer(modifier = Modifier.height(8.dp))
            Text(
                text = stringResource(R.string.diagnostics_tester_note),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
            Spacer(modifier = Modifier.height(12.dp))
            Text(
                text = logText,
                style = MaterialTheme.typography.bodySmall,
                fontFamily = FontFamily.Monospace
            )
        }
    }
}
