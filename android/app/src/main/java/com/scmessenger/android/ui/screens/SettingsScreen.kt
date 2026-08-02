package com.scmessenger.android.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Block
import androidx.compose.material.icons.filled.ContentCopy
import androidx.compose.material.icons.filled.Share
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.Lock
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import kotlinx.coroutines.launch
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.scmessenger.android.BuildConfig
import com.scmessenger.android.R
import com.scmessenger.android.ui.viewmodels.MeshServiceViewModel
import com.scmessenger.android.ui.viewmodels.SettingsViewModel
import com.scmessenger.android.data.PreferencesRepository
import com.scmessenger.android.ui.settings.MeshSettingsScreen
import com.scmessenger.android.ui.settings.PowerSettingsScreen
import com.scmessenger.android.ui.components.ErrorState
import com.scmessenger.android.ui.components.WarningBanner
import com.scmessenger.android.utils.BackupPassphraseValidation
import com.scmessenger.android.utils.validateBackupPassphrase

/**
 * Settings screen with mesh configuration and app preferences.
 */
@Composable
fun SettingsScreen(
    settingsViewModel: SettingsViewModel = hiltViewModel(),
    serviceViewModel: MeshServiceViewModel = hiltViewModel(),
    onNavigateToIdentity: () -> Unit = {},
    onNavigateToDiagnostics: () -> Unit = {},
    onNavigateToBlockedPeers: () -> Unit = {},
    onNavigateToMeshSettings: () -> Unit = {},
    onNavigateToPowerSettings: () -> Unit = {}
) {
    val meshSettings by settingsViewModel.settings.collectAsState()
    val identityInfo by settingsViewModel.identityInfo.collectAsState()
    val autoStart by settingsViewModel.autoStart.collectAsState()
    val notificationsEnabled by settingsViewModel.notificationsEnabled.collectAsState()
    val notifyDmEnabled by settingsViewModel.notifyDmEnabled.collectAsState()
    val notifyDmRequestEnabled by settingsViewModel.notifyDmRequestEnabled.collectAsState()
    val notifyDmInForeground by settingsViewModel.notifyDmInForeground.collectAsState()
    val notifyDmRequestInForeground by settingsViewModel.notifyDmRequestInForeground.collectAsState()
    val soundEnabled by settingsViewModel.soundEnabled.collectAsState()
    val badgeEnabled by settingsViewModel.badgeEnabled.collectAsState()
    val themeMode by settingsViewModel.themeMode.collectAsState()
    val isLoading by settingsViewModel.isLoading.collectAsState()
    val serviceState by serviceViewModel.serviceState.collectAsState()
    val isRunning by serviceViewModel.isRunning.collectAsState()
    val serviceStats by serviceViewModel.serviceStats.collectAsState()
    val settingsError by settingsViewModel.error.collectAsState()

    val context = LocalContext.current
    val scope = rememberCoroutineScope()

    val importResult by settingsViewModel.importResult.collectAsState()
    val backupExportResult by settingsViewModel.backupExportResult.collectAsState()
    val snackbarHostState = remember { SnackbarHostState() }

    var showImportDialog by remember { mutableStateOf(false) }
    var importText by remember { mutableStateOf("") }
    var importPassphrase by remember { mutableStateOf("") }

    var showExportBackupDialog by remember { mutableStateOf(false) }
    var showApkShareDialog by remember { mutableStateOf(false) }
    var exportPassphrase by remember { mutableStateOf("") }
    var exportPassphraseConfirm by remember { mutableStateOf("") }
    var exportPassphraseError by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(importResult) {
        importResult?.let {
            snackbarHostState.showSnackbar(it)
            settingsViewModel.clearImportResult()
        }
    }

    // Load identity once when Settings screen is first composed.
    // The ViewModel init() already handles the initial load on IO; this ensures
    // the UI is populated even if the screen re-attaches (e.g., back-navigation).
    LaunchedEffect(Unit) {
        settingsViewModel.loadIdentity()
    }

    // Reload identity when service transitions to RUNNING (lazy-start path).
    // Force-refresh to replace cached SharedPreferences data with live Rust core data.
    // Track prior state to avoid re-firing on repeated RUNNING emissions.
    var lastServiceState by remember { mutableStateOf<uniffi.api.ServiceState?>(null) }
    LaunchedEffect(serviceState) {
        if (serviceState == uniffi.api.ServiceState.RUNNING &&
            lastServiceState != uniffi.api.ServiceState.RUNNING) {
            settingsViewModel.loadIdentity(forceRefresh = true)
        }
        lastServiceState = serviceState
    }

    val statsText = remember(serviceStats) { serviceViewModel.getStatsText() }

    Box(modifier = Modifier.fillMaxSize()) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(16.dp)
        ) {
        Text(
            text = stringResource(R.string.settings_title),
            style = MaterialTheme.typography.headlineMedium,
            modifier = Modifier.padding(bottom = 16.dp)
        )

        // Wire ErrorState into settings error display
        settingsError?.let {
            ErrorState(
                error = it,
                onDismiss = { settingsViewModel.clearError() }
            )
        }

        // Wire WarningBanner into settings for service warnings
        if (!isRunning) {
            WarningBanner(
                message = stringResource(R.string.settings_error_not_running),
                onDismiss = {}
            )
        }

        // Service Control Section
        ServiceControlSection(
            isRunning = isRunning,
            serviceState = serviceState,
            onToggleService = { serviceViewModel.toggleService() },
            stats = statsText
        )

        Spacer(modifier = Modifier.height(24.dp))

        // Identity Section
        val resolvedIdentityInfo = identityInfo
        if (resolvedIdentityInfo?.initialized == true) {
            IdentitySection(
                identityInfo = resolvedIdentityInfo,
                onNicknameChange = { settingsViewModel.updateNickname(it) },
                onCopyExport = {
                    scope.launch {
                        val export = settingsViewModel.getIdentityExportString()
                        val clipboard = context.getSystemService(android.content.Context.CLIPBOARD_SERVICE) as android.content.ClipboardManager
                        val clip = android.content.ClipData.newPlainText("Identity Export", export)
                        clipboard.setPrimaryClip(clip)
                    }
                },
                onShowIdentityQr = onNavigateToIdentity,
                onImportIdentity = {
                    importText = ""
                    importPassphrase = ""
                    showImportDialog = true
                },
                onExportIdentityBackup = {
                    exportPassphrase = ""
                    exportPassphraseConfirm = ""
                    exportPassphraseError = null
                    showExportBackupDialog = true
                }
            )
        } else {
            IdentityUnavailableSection(
                onCreateIdentity = onNavigateToIdentity
            )
        }

        Spacer(modifier = Modifier.height(24.dp))

        // Mesh Settings Section
        meshSettings?.let { settings ->
            MeshSettingsSection(
                settings = settings,
                onUpdateSetting = { updater -> updater(settingsViewModel) },
                isLoading = isLoading
            )
        }

        Spacer(modifier = Modifier.height(24.dp))



        // Theme Section
        ThemeSection(
            themeMode = themeMode,
            onThemeModeChange = { settingsViewModel.setThemeMode(it) }
        )

        Spacer(modifier = Modifier.height(24.dp))

        // App Preferences Section
        AppPreferencesSection(
            autoStart = autoStart,
            notificationsEnabled = notificationsEnabled,
            onAutoStartChange = { settingsViewModel.setAutoStart(it) },
            onNotificationsChange = { settingsViewModel.setNotificationsEnabled(it) }
        )

        Spacer(modifier = Modifier.height(24.dp))

        // Notification Sub-Settings Section (visible only when master toggle is on)
        if (notificationsEnabled) {
            NotificationSettingsSection(
                notifyDmEnabled = notifyDmEnabled,
                notifyDmRequestEnabled = notifyDmRequestEnabled,
                notifyDmInForeground = notifyDmInForeground,
                notifyDmRequestInForeground = notifyDmRequestInForeground,
                soundEnabled = soundEnabled,
                badgeEnabled = badgeEnabled,
                onNotifyDmEnabledChange = { settingsViewModel.setNotifyDmEnabled(it) },
                onNotifyDmRequestEnabledChange = { settingsViewModel.setNotifyDmRequestEnabled(it) },
                onNotifyDmInForegroundChange = { settingsViewModel.setNotifyDmInForeground(it) },
                onNotifyDmRequestInForegroundChange = { settingsViewModel.setNotifyDmRequestInForeground(it) },
                onSoundEnabledChange = { settingsViewModel.setSoundEnabled(it) },
                onBadgeEnabledChange = { settingsViewModel.setBadgeEnabled(it) }
            )
            Spacer(modifier = Modifier.height(24.dp))
        }

        // Privacy Section
        PrivacySection(
            onNavigateToBlockedPeers = onNavigateToBlockedPeers
        )

        Spacer(modifier = Modifier.height(24.dp))

        // Info Section
        InfoSection(
            contactCount = settingsViewModel.getContactCount(),
            messageCount = settingsViewModel.getMessageCount(),
            buildProvenance = settingsViewModel.getBuildProvenance()
        )

        Spacer(modifier = Modifier.height(24.dp))

        // Advanced / Diagnostics Section
        AdvancedSettingsSection(
            onNavigateToDiagnostics = onNavigateToDiagnostics,
            onShareApkClick = { showApkShareDialog = true }
        )

        Spacer(modifier = Modifier.height(24.dp))

        // Mesh Settings Navigation
        SettingsToMeshSettingsNavigation(
            onNavigateToMeshSettings = onNavigateToMeshSettings
        )

        Spacer(modifier = Modifier.height(24.dp))

        // Power Settings Navigation
        SettingsToPowerSettingsNavigation(
            onNavigateToPowerSettings = onNavigateToPowerSettings
        )

        Spacer(modifier = Modifier.height(24.dp))

        // Data Management Section
        DataManagementSection(
            onResetAll = { settingsViewModel.resetAllData() }
        )
    }

    if (showImportDialog) {
        AlertDialog(
            onDismissRequest = { showImportDialog = false },
            title = { Text(stringResource(R.string.settings_title_import_identity)) },
            text = {
                Column {
                    OutlinedTextField(
                        value = importText,
                        onValueChange = { importText = it },
                        label = { Text(stringResource(R.string.settings_label_paste_backup)) },
                        minLines = 3,
                        maxLines = 6
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    OutlinedTextField(
                        value = importPassphrase,
                        onValueChange = { importPassphrase = it },
                        label = { Text(stringResource(R.string.settings_label_backup_passphrase_optional)) },
                        visualTransformation = androidx.compose.ui.text.input.PasswordVisualTransformation(),
                        singleLine = true,
                        modifier = Modifier.fillMaxWidth()
                    )
                }
            },
            confirmButton = {
                TextButton(
                    onClick = {
                        settingsViewModel.importIdentityBackup(importText, importPassphrase)
                        showImportDialog = false
                        importText = ""
                        importPassphrase = ""
                    },
                    enabled = importText.isNotBlank()
                ) {
                    Text(stringResource(R.string.settings_action_import))
                }
            },
            dismissButton = {
                TextButton(onClick = { showImportDialog = false }) {
                    Text(stringResource(R.string.cancel))
                }
            }
        )
    }

    if (showExportBackupDialog) {
        AlertDialog(
            onDismissRequest = { showExportBackupDialog = false },
            title = { Text(stringResource(R.string.settings_title_export_identity_backup)) },
            text = {
                Column {
                    Text(
                        text = stringResource(R.string.settings_export_backup_description),
                        style = MaterialTheme.typography.bodySmall,
                        modifier = Modifier.padding(bottom = 12.dp)
                    )
                    OutlinedTextField(
                        value = exportPassphrase,
                        onValueChange = {
                            exportPassphrase = it
                            exportPassphraseError = null
                        },
                        label = { Text(stringResource(R.string.settings_label_backup_passphrase)) },
                        visualTransformation = androidx.compose.ui.text.input.PasswordVisualTransformation(),
                        singleLine = true,
                        modifier = Modifier.fillMaxWidth()
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    OutlinedTextField(
                        value = exportPassphraseConfirm,
                        onValueChange = {
                            exportPassphraseConfirm = it
                            exportPassphraseError = null
                        },
                        label = { Text(stringResource(R.string.settings_label_backup_passphrase_confirm)) },
                        visualTransformation = androidx.compose.ui.text.input.PasswordVisualTransformation(),
                        singleLine = true,
                        modifier = Modifier.fillMaxWidth()
                    )
                    exportPassphraseError?.let {
                        Text(
                            text = it,
                            color = MaterialTheme.colorScheme.error,
                            style = MaterialTheme.typography.bodySmall,
                            modifier = Modifier.padding(top = 4.dp)
                        )
                    }
                }
            },
            confirmButton = {
                TextButton(onClick = {
                    when (validateBackupPassphrase(exportPassphrase, exportPassphraseConfirm)) {
                        BackupPassphraseValidation.TooShort -> {
                            exportPassphraseError = context.getString(R.string.settings_error_passphrase_too_short)
                        }
                        BackupPassphraseValidation.Mismatch -> {
                            exportPassphraseError = context.getString(R.string.settings_error_passphrase_mismatch)
                        }
                        BackupPassphraseValidation.Valid -> {
                            settingsViewModel.exportIdentityBackup(exportPassphrase)
                            showExportBackupDialog = false
                            exportPassphrase = ""
                            exportPassphraseConfirm = ""
                        }
                    }
                }) {
                    Text(stringResource(R.string.settings_action_export_identity_backup))
                }
            },
            dismissButton = {
                TextButton(onClick = { showExportBackupDialog = false }) {
                    Text(stringResource(R.string.cancel))
                }
            }
        )
    }

    backupExportResult?.let { result ->
        AlertDialog(
            onDismissRequest = { settingsViewModel.clearBackupExportResult() },
            title = { Text(stringResource(R.string.settings_title_backup_ready)) },
            text = {
                result.fold(
                    onSuccess = { backup ->
                        Column {
                            Text(
                                text = stringResource(R.string.settings_backup_ready_description),
                                style = MaterialTheme.typography.bodySmall,
                                modifier = Modifier.padding(bottom = 12.dp)
                            )
                            Text(
                                text = backup,
                                style = MaterialTheme.typography.bodySmall,
                                fontFamily = androidx.compose.ui.text.font.FontFamily.Monospace,
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .heightIn(max = 160.dp)
                                    .verticalScroll(rememberScrollState())
                            )
                        }
                    },
                    onFailure = { error ->
                        Text(stringResource(R.string.settings_error_export_backup, error.message ?: ""))
                    }
                )
            },
            confirmButton = {
                result.getOrNull()?.let { backup ->
                    TextButton(onClick = {
                        val clipboard = context.getSystemService(android.content.Context.CLIPBOARD_SERVICE) as android.content.ClipboardManager
                        val clip = android.content.ClipData.newPlainText("Identity Backup", backup)
                        clipboard.setPrimaryClip(clip)
                        scope.launch { snackbarHostState.showSnackbar(context.getString(R.string.settings_backup_copied)) }
                        settingsViewModel.clearBackupExportResult()
                    }) {
                        Text(stringResource(R.string.settings_action_copy_backup))
                    }
                }
            },
            dismissButton = {
                TextButton(onClick = { settingsViewModel.clearBackupExportResult() }) {
                    Text(stringResource(R.string.cancel))
                }
            }
        )
    }

    if (showApkShareDialog) {
        com.scmessenger.android.ui.dialogs.ApkShareDialog(
            onDismiss = { showApkShareDialog = false }
        )
    }

    SnackbarHost(
        hostState = snackbarHostState,
        modifier = Modifier.align(Alignment.BottomCenter)
    )
}
}

@Composable
fun DataManagementSection(
    onResetAll: () -> Unit
) {
    var showConfirmDialog by remember { mutableStateOf(false) }

    if (showConfirmDialog) {
        AlertDialog(
            onDismissRequest = { showConfirmDialog = false },
            title = { Text(stringResource(R.string.settings_title_reset_data)) },
            text = { Text(stringResource(R.string.settings_reset_data_description)) },
            confirmButton = {
                TextButton(
                    onClick = {
                        showConfirmDialog = false
                        onResetAll()
                    },
                    colors = ButtonDefaults.textButtonColors(contentColor = MaterialTheme.colorScheme.error)
                ) {
                    Text(stringResource(R.string.settings_action_reset))
                }
            },
            dismissButton = {
                TextButton(onClick = { showConfirmDialog = false }) {
                    Text(stringResource(R.string.cancel))
                }
            }
        )
    }

    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.errorContainer.copy(alpha = 0.2f)
        )
    ) {
        Column(
            modifier = Modifier.padding(16.dp)
        ) {
            Text(
                text = stringResource(R.string.settings_section_data_management),
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.error,
                modifier = Modifier.padding(bottom = 8.dp)
            )

            Text(
                text = stringResource(R.string.settings_danger_zone_description),
                style = MaterialTheme.typography.bodySmall,
                modifier = Modifier.padding(bottom = 16.dp)
            )

            Button(
                onClick = { showConfirmDialog = true },
                modifier = Modifier.fillMaxWidth(),
                colors = ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.error)
            ) {
                Text(stringResource(R.string.settings_button_delete_reset))
            }
        }
    }
}

@Composable
fun ServiceControlSection(
    isRunning: Boolean,
    serviceState: uniffi.api.ServiceState,
    onToggleService: () -> Unit,
    stats: String
) {
    Card(
        modifier = Modifier.fillMaxWidth()
    ) {
        Column(
            modifier = Modifier.padding(16.dp)
        ) {
            Text(
                text = stringResource(R.string.settings_section_service),
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(bottom = 8.dp)
            )

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Column {
                    Text(stringResource(R.string.settings_label_status_format, serviceState.name))
                    if (isRunning) {
                        Text(
                            text = stringResource(R.string.settings_status_active),
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.primary
                        )
                    }
                }

                Button(onClick = onToggleService) {
                    Text(if (isRunning) stringResource(R.string.action_stop) else stringResource(R.string.action_start))
                }
            }

            if (isRunning) {
                Spacer(modifier = Modifier.height(8.dp))
                HorizontalDivider()
                Spacer(modifier = Modifier.height(8.dp))
                Text(
                    text = stats,
                    style = MaterialTheme.typography.bodySmall,
                    modifier = Modifier.fillMaxWidth()
                )
            }
        }
    }
}

@Composable
fun MeshSettingsSection(
    settings: uniffi.api.MeshSettings,
    onUpdateSetting: ((SettingsViewModel) -> Unit) -> Unit,
    isLoading: Boolean
) {
    Card(
        modifier = Modifier.fillMaxWidth()
    ) {
        Column(
            modifier = Modifier.padding(16.dp)
        ) {
            Text(
                text = stringResource(R.string.mesh_network_label),
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(bottom = 8.dp)
            )

            // Warning about relay requirement
            Card(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(bottom = 8.dp),
                colors = CardDefaults.cardColors(
                    containerColor = MaterialTheme.colorScheme.errorContainer
                )
            ) {
                Column(
                    modifier = Modifier.padding(12.dp)
                ) {
                    Text(
                        text = stringResource(R.string.settings_mesh_relay_warning_title),
                        style = MaterialTheme.typography.titleSmall,
                        fontWeight = FontWeight.Medium,
                        color = MaterialTheme.colorScheme.onErrorContainer
                    )
                    Text(
                        text = stringResource(R.string.settings_mesh_relay_warning_description),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onErrorContainer
                    )
                }
            }

            SwitchPreference(
                title = stringResource(R.string.settings_label_mesh_participation),
                subtitle = stringResource(R.string.settings_description_mesh_participation),
                checked = settings.relayEnabled,
                onCheckedChange = { onUpdateSetting { vm -> vm.updateRelayEnabled(it) } },
                enabled = !isLoading
            )

            SwitchPreference(
                title = stringResource(R.string.settings_label_ble),
                subtitle = stringResource(R.string.settings_description_ble),
                checked = settings.bleEnabled,
                onCheckedChange = { onUpdateSetting { vm -> vm.updateBleEnabled(it) } },
                enabled = !isLoading
            )

            SwitchPreference(
                title = stringResource(R.string.settings_label_wifi_aware),
                subtitle = stringResource(R.string.settings_description_wifi_aware),
                checked = settings.wifiAwareEnabled,
                onCheckedChange = { onUpdateSetting { vm -> vm.updateWifiAwareEnabled(it) } },
                enabled = !isLoading
            )

            SwitchPreference(
                title = stringResource(R.string.settings_label_wifi_direct),
                subtitle = stringResource(R.string.settings_description_wifi_direct),
                checked = settings.wifiDirectEnabled,
                onCheckedChange = { onUpdateSetting { vm -> vm.updateWifiDirectEnabled(it) } },
                enabled = !isLoading
            )

            SwitchPreference(
                title = stringResource(R.string.settings_label_internet_relay),
                subtitle = stringResource(R.string.settings_description_internet_relay),
                checked = settings.internetEnabled,
                onCheckedChange = { onUpdateSetting { vm -> vm.updateInternetEnabled(it) } },
                enabled = !isLoading
            )
        }
    }
}



@Composable
fun AppPreferencesSection(
    autoStart: Boolean,
    notificationsEnabled: Boolean,
    onAutoStartChange: (Boolean) -> Unit,
    onNotificationsChange: (Boolean) -> Unit
) {
    Card(
        modifier = Modifier.fillMaxWidth()
    ) {
        Column(
            modifier = Modifier.padding(16.dp)
        ) {
            Text(
                text = stringResource(R.string.settings_section_preferences),
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(bottom = 8.dp)
            )

            SwitchPreference(
                title = stringResource(R.string.settings_label_auto_start),
                subtitle = stringResource(R.string.settings_description_auto_start),
                checked = autoStart,
                onCheckedChange = onAutoStartChange
            )

            SwitchPreference(
                title = stringResource(R.string.settings_label_notifications),
                subtitle = stringResource(R.string.settings_description_notifications),
                checked = notificationsEnabled,
                onCheckedChange = onNotificationsChange
            )
        }
    }
}

@Composable
fun NotificationSettingsSection(
    notifyDmEnabled: Boolean,
    notifyDmRequestEnabled: Boolean,
    notifyDmInForeground: Boolean,
    notifyDmRequestInForeground: Boolean,
    soundEnabled: Boolean,
    badgeEnabled: Boolean,
    onNotifyDmEnabledChange: (Boolean) -> Unit,
    onNotifyDmRequestEnabledChange: (Boolean) -> Unit,
    onNotifyDmInForegroundChange: (Boolean) -> Unit,
    onNotifyDmRequestInForegroundChange: (Boolean) -> Unit,
    onSoundEnabledChange: (Boolean) -> Unit,
    onBadgeEnabledChange: (Boolean) -> Unit
) {
    Card(
        modifier = Modifier.fillMaxWidth()
    ) {
        Column(
            modifier = Modifier.padding(16.dp)
        ) {
            Text(
                text = stringResource(R.string.settings_label_notifications),
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(bottom = 8.dp)
            )

            SwitchPreference(
                title = stringResource(R.string.settings_label_notify_dm),
                subtitle = stringResource(R.string.settings_description_notify_dm),
                checked = notifyDmEnabled,
                onCheckedChange = onNotifyDmEnabledChange
            )

            SwitchPreference(
                title = stringResource(R.string.settings_label_notify_dm_request),
                subtitle = stringResource(R.string.settings_description_notify_dm_request),
                checked = notifyDmRequestEnabled,
                onCheckedChange = onNotifyDmRequestEnabledChange
            )

            SwitchPreference(
                title = stringResource(R.string.settings_label_notify_dm_foreground),
                subtitle = stringResource(R.string.settings_description_notify_dm_foreground),
                checked = notifyDmInForeground,
                onCheckedChange = onNotifyDmInForegroundChange
            )

            SwitchPreference(
                title = stringResource(R.string.settings_label_notify_dm_request_foreground),
                subtitle = stringResource(R.string.settings_description_notify_dm_request_foreground),
                checked = notifyDmRequestInForeground,
                onCheckedChange = onNotifyDmRequestInForegroundChange
            )

            SwitchPreference(
                title = stringResource(R.string.settings_label_sound),
                subtitle = stringResource(R.string.settings_description_sound),
                checked = soundEnabled,
                onCheckedChange = onSoundEnabledChange
            )

            SwitchPreference(
                title = stringResource(R.string.settings_label_badge),
                subtitle = stringResource(R.string.settings_description_badge),
                checked = badgeEnabled,
                onCheckedChange = onBadgeEnabledChange
            )
        }
    }
}

@Composable
fun ThemeSection(
    themeMode: PreferencesRepository.ThemeMode,
    onThemeModeChange: (PreferencesRepository.ThemeMode) -> Unit
) {
    Card(
        modifier = Modifier.fillMaxWidth()
    ) {
        Column(
            modifier = Modifier.padding(16.dp)
        ) {
            Text(
                text = stringResource(R.string.settings_label_theme),
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(bottom = 8.dp)
            )

            ThemeRadioOption(
                label = stringResource(R.string.settings_theme_system),
                selected = themeMode == PreferencesRepository.ThemeMode.SYSTEM,
                onClick = { onThemeModeChange(PreferencesRepository.ThemeMode.SYSTEM) }
            )
            ThemeRadioOption(
                label = stringResource(R.string.settings_theme_light),
                selected = themeMode == PreferencesRepository.ThemeMode.LIGHT,
                onClick = { onThemeModeChange(PreferencesRepository.ThemeMode.LIGHT) }
            )
            ThemeRadioOption(
                label = stringResource(R.string.settings_theme_dark),
                selected = themeMode == PreferencesRepository.ThemeMode.DARK,
                onClick = { onThemeModeChange(PreferencesRepository.ThemeMode.DARK) }
            )
        }
    }
}

@Composable
fun ThemeRadioOption(
    label: String,
    selected: Boolean,
    onClick: () -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        RadioButton(
            selected = selected,
            onClick = onClick
        )
        Spacer(modifier = Modifier.width(8.dp))
        Text(
            text = label,
            style = MaterialTheme.typography.bodyLarge
        )
    }
}

@Composable
fun SwitchPreference(
    title: String,
    subtitle: String,
    checked: Boolean,
    onCheckedChange: (Boolean) -> Unit,
    enabled: Boolean = true
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 8.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Column(
            modifier = Modifier.weight(1f)
        ) {
            Text(
                text = title,
                style = MaterialTheme.typography.bodyLarge
            )
            Text(
                text = subtitle,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }

        Switch(
            checked = checked,
            onCheckedChange = onCheckedChange,
            enabled = enabled
        )
    }
}

@Composable
fun InfoSection(
    contactCount: UInt,
    messageCount: UInt,
    buildProvenance: String
) {
    Card(
        modifier = Modifier.fillMaxWidth()
    ) {
        Column(
            modifier = Modifier.padding(16.dp)
        ) {
            Text(
                text = stringResource(R.string.settings_section_info),
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(bottom = 8.dp)
            )

            InfoRow(stringResource(R.string.settings_label_contacts), contactCount.toString())
            InfoRow(stringResource(R.string.settings_label_messages), messageCount.toString())
            InfoRow(stringResource(R.string.settings_label_version), "${BuildConfig.VERSION_NAME} (Core: $buildProvenance)")
        }
    }
}

@Composable
fun InfoRow(label: String, value: String) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp),
        horizontalArrangement = Arrangement.SpaceBetween
    ) {
        Text(
            text = label,
            style = MaterialTheme.typography.bodyMedium
        )
        Text(
            text = value,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
    }
}

@Composable
fun IdentitySection(
    identityInfo: uniffi.api.IdentityInfo,
    onNicknameChange: (String) -> Unit,
    onCopyExport: () -> Unit,
    onShowIdentityQr: () -> Unit,
    onImportIdentity: () -> Unit,
    onExportIdentityBackup: () -> Unit
) {
    val context = LocalContext.current

    Card(
        modifier = Modifier.fillMaxWidth()
    ) {
        Column(
            modifier = Modifier.padding(16.dp)
        ) {
            Text(
                text = stringResource(R.string.settings_section_identity),
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(bottom = 8.dp)
            )

            // Nickname Input
            var pendingNickname by remember(identityInfo.nickname) { mutableStateOf(identityInfo.nickname ?: "") }
            OutlinedTextField(
                value = pendingNickname,
                onValueChange = {
                    pendingNickname = it
                    // DO NOT call onNicknameChange here — it triggers a Rust FFI roundtrip per keystroke
                },
                label = { Text(stringResource(R.string.settings_label_nickname)) },
                modifier = Modifier.fillMaxWidth(),
                singleLine = true,
                keyboardOptions = androidx.compose.foundation.text.KeyboardOptions(
                    imeAction = androidx.compose.ui.text.input.ImeAction.Done
                ),
                keyboardActions = androidx.compose.foundation.text.KeyboardActions(
                    onDone = {
                        onNicknameChange(pendingNickname.trim())
                    }
                )
            )
            Spacer(modifier = Modifier.height(4.dp))
            Button(
                onClick = { onNicknameChange(pendingNickname.trim()) },
                modifier = Modifier.fillMaxWidth()
            ) {
                Text(stringResource(R.string.settings_action_save_nickname))
            }

            Spacer(modifier = Modifier.height(16.dp))

            // Peer ID (Network)
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween
            ) {
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = stringResource(R.string.identity_label_peer_id),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Text(
                        text = identityInfo.libp2pPeerId?.take(16) ?: stringResource(R.string.identity_field_unavailable),
                        style = MaterialTheme.typography.bodyMedium,
                        fontFamily = androidx.compose.ui.text.font.FontFamily.Monospace
                    )
                }

                IconButton(onClick = {
                    val clipboard = context.getSystemService(android.content.Context.CLIPBOARD_SERVICE) as android.content.ClipboardManager
                    val clip = android.content.ClipData.newPlainText("Peer ID", identityInfo.libp2pPeerId ?: "")
                    clipboard.setPrimaryClip(clip)
                }) {
                    Icon(Icons.Default.ContentCopy, contentDescription = stringResource(R.string.settings_content_desc_copy_peer_id))
                }
            }

            HorizontalDivider(modifier = Modifier.padding(vertical = 8.dp))

            // Identity Hash
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween
            ) {
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = stringResource(R.string.identity_label_hash),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Text(
                        text = identityInfo.identityId?.take(8) ?: stringResource(R.string.identity_field_unavailable),
                        style = MaterialTheme.typography.bodyMedium,
                        fontFamily = androidx.compose.ui.text.font.FontFamily.Monospace
                    )
                }

                IconButton(onClick = {
                    val clipboard = context.getSystemService(android.content.Context.CLIPBOARD_SERVICE) as android.content.ClipboardManager
                    val clip = android.content.ClipData.newPlainText("Identity Hash", identityInfo.identityId ?: "")
                    clipboard.setPrimaryClip(clip)
                }) {
                    Icon(Icons.Default.ContentCopy, contentDescription = stringResource(R.string.settings_content_desc_copy_identity_hash))
                }
            }

            HorizontalDivider(modifier = Modifier.padding(vertical = 8.dp))

            // Public Key
             Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween
            ) {
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = stringResource(R.string.identity_label_public_key),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Text(
                        text = identityInfo.publicKeyHex?.take(8) ?: stringResource(R.string.identity_field_unavailable),
                        style = MaterialTheme.typography.bodyMedium,
                        fontFamily = androidx.compose.ui.text.font.FontFamily.Monospace
                    )
                }

                IconButton(onClick = {
                    val clipboard = context.getSystemService(android.content.Context.CLIPBOARD_SERVICE) as android.content.ClipboardManager
                    val clip = android.content.ClipData.newPlainText("Public Key", identityInfo.publicKeyHex ?: "")
                    clipboard.setPrimaryClip(clip)
                }) {
                    Icon(Icons.Default.ContentCopy, contentDescription = stringResource(R.string.settings_content_desc_copy_key))
                }
            }

            Spacer(modifier = Modifier.height(8.dp))

            // Full Export Button
            OutlinedButton(
                onClick = onCopyExport,
                modifier = Modifier.fillMaxWidth()
            ) {
                Icon(Icons.Default.Share, contentDescription = stringResource(R.string.settings_content_desc_share_identity_export), modifier = Modifier.size(16.dp))
                Spacer(modifier = Modifier.width(8.dp))
                Text(stringResource(R.string.settings_button_copy_identity))
            }

            Spacer(modifier = Modifier.height(8.dp))

            Button(
                onClick = onShowIdentityQr,
                modifier = Modifier.fillMaxWidth()
            ) {
                Text(stringResource(R.string.settings_button_show_qr))
            }

            Spacer(modifier = Modifier.height(8.dp))

            OutlinedButton(
                onClick = onImportIdentity,
                modifier = Modifier.fillMaxWidth()
            ) {
                Text(stringResource(R.string.settings_button_import_identity))
            }

            Spacer(modifier = Modifier.height(8.dp))

            // Passphrase-encrypted private-key backup: distinct from the public
            // "Copy Full Identity Export" above, which has no encryption and
            // can't restore ratchet sessions or contacts.
            OutlinedButton(
                onClick = onExportIdentityBackup,
                modifier = Modifier.fillMaxWidth()
            ) {
                Icon(Icons.Default.Lock, contentDescription = null, modifier = Modifier.size(16.dp))
                Spacer(modifier = Modifier.width(8.dp))
                Text(stringResource(R.string.settings_action_export_identity_backup))
            }
        }
    }
}

@Composable
fun IdentityUnavailableSection(
    onCreateIdentity: () -> Unit
) {
    Card(
        modifier = Modifier.fillMaxWidth()
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Text(
                text = stringResource(R.string.settings_section_identity),
                style = MaterialTheme.typography.titleMedium
            )
            Text(
                text = stringResource(R.string.settings_identity_unavailable_description),
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
            Button(
                onClick = onCreateIdentity,
                modifier = Modifier.fillMaxWidth()
            ) {
                Text(stringResource(R.string.settings_button_create_identity))
            }
        }
    }
}
@Composable
fun PrivacySection(
    onNavigateToBlockedPeers: () -> Unit
) {
    val context = LocalContext.current
    Card(
        modifier = Modifier.fillMaxWidth()
    ) {
        Column(
            modifier = Modifier.padding(16.dp)
        ) {
            Text(
                text = stringResource(R.string.settings_section_privacy),
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(bottom = 8.dp)
            )

            Button(
                onClick = onNavigateToBlockedPeers,
                modifier = Modifier.fillMaxWidth()
            ) {
                Icon(Icons.Filled.Block, contentDescription = stringResource(R.string.settings_content_desc_manage_blocked_peers), modifier = Modifier.size(16.dp))
                Spacer(modifier = Modifier.width(8.dp))
                Text(stringResource(R.string.settings_button_manage_blocked))
            }

            Spacer(modifier = Modifier.height(8.dp))

            OutlinedButton(
                onClick = {
                    val intent = android.content.Intent(
                        android.content.Intent.ACTION_VIEW,
                        android.net.Uri.parse(context.getString(R.string.privacy_policy_url))
                    )
                    context.startActivity(intent)
                },
                modifier = Modifier.fillMaxWidth()
            ) {
                Text(stringResource(R.string.settings_button_privacy_policy))
            }
        }
    }
}

@Composable
fun AdvancedSettingsSection(
    onNavigateToDiagnostics: () -> Unit,
    onShareApkClick: () -> Unit = {}
) {
    Card(
        modifier = Modifier.fillMaxWidth()
    ) {
        Column(
            modifier = Modifier.padding(16.dp)
        ) {
            Text(
                text = stringResource(R.string.settings_section_advanced),
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(bottom = 8.dp)
            )

            Text(
                text = stringResource(R.string.settings_advanced_reliability_note),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(bottom = 8.dp)
            )

            Text(
                text = stringResource(R.string.settings_advanced_permissions_note),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(bottom = 12.dp)
            )

            Button(
                onClick = onShareApkClick,
                modifier = Modifier.fillMaxWidth(),
                colors = ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.secondary)
            ) {
                Icon(Icons.Default.Share, contentDescription = null, modifier = Modifier.size(16.dp))
                Spacer(modifier = Modifier.width(8.dp))
                Text(stringResource(R.string.settings_button_share_apk))
            }

            Spacer(modifier = Modifier.height(8.dp))

            Button(
                onClick = onNavigateToDiagnostics,
                modifier = Modifier.fillMaxWidth()
            ) {
                Text(stringResource(R.string.settings_button_diagnostics))
            }
        }
    }
}

@Composable
fun SettingsToMeshSettingsNavigation(
    onNavigateToMeshSettings: () -> Unit,
    modifier: Modifier = Modifier
) {
    Card(modifier = modifier.fillMaxWidth()) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text(
                text = stringResource(R.string.settings_nav_mesh_title),
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(bottom = 8.dp)
            )
            Text(
                text = stringResource(R.string.settings_nav_mesh_description),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(bottom = 12.dp)
            )
            Button(
                onClick = onNavigateToMeshSettings,
                modifier = Modifier.fillMaxWidth()
            ) {
                Text(stringResource(R.string.settings_button_configure_mesh))
            }
        }
    }
}

/**
 * Navigation helper to navigate to PowerSettingsScreen.
 */
@Composable
fun SettingsToPowerSettingsNavigation(
    onNavigateToPowerSettings: () -> Unit,
    modifier: Modifier = Modifier
) {
    Card(modifier = modifier.fillMaxWidth()) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text(
                text = stringResource(R.string.settings_nav_power_title),
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(bottom = 8.dp)
            )
            Text(
                text = stringResource(R.string.settings_nav_power_description),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(bottom = 12.dp)
            )
            Button(
                onClick = onNavigateToPowerSettings,
                modifier = Modifier.fillMaxWidth()
            ) {
                Text(stringResource(R.string.settings_button_configure_power))
            }
        }
    }
}
