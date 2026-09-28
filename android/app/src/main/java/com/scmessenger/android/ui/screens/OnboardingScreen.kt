package com.scmessenger.android.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Lock
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Shield
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.google.mlkit.vision.barcode.common.Barcode
import com.google.mlkit.vision.codescanner.GmsBarcodeScannerOptions
import com.google.mlkit.vision.codescanner.GmsBarcodeScanning
import com.scmessenger.android.R
import com.scmessenger.android.ui.viewmodels.MainViewModel
import com.scmessenger.android.ui.identity.IdentityCreationFlow
import timber.log.Timber

@OptIn(com.google.accompanist.permissions.ExperimentalPermissionsApi::class)
@Composable
fun OnboardingScreen(
    onOnboardingComplete: () -> Unit,
    viewModel: MainViewModel = hiltViewModel(),
    modifier: Modifier = Modifier
) {
    val permissionsToRequest = remember {
        val list = mutableListOf(
            android.Manifest.permission.ACCESS_FINE_LOCATION
            // Add Bluetooth Scan/Connect/Advertise if API >= 31
        ).apply {
            if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.S) {
                add(android.Manifest.permission.BLUETOOTH_SCAN)
                add(android.Manifest.permission.BLUETOOTH_ADVERTISE)
                add(android.Manifest.permission.BLUETOOTH_CONNECT)
            }
            if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.TIRAMISU) {
                add(android.Manifest.permission.NEARBY_WIFI_DEVICES)
                add(android.Manifest.permission.POST_NOTIFICATIONS)
            }
        }
        list.toList()
    }

    val permissionsState = com.google.accompanist.permissions.rememberMultiplePermissionsState(
        permissions = permissionsToRequest
    )

    val importError by viewModel.importError.collectAsState()
    val importSuccess by viewModel.importSuccess.collectAsState()
    val isReady by viewModel.isReady.collectAsState()
    val onboardingCompleted by viewModel.onboardingCompleted.collectAsState()
    val isCreating by viewModel.isCreatingIdentity.collectAsState()
    val identityError by viewModel.identityError.collectAsState()
    // P0_ANDROID_IDENTITY_PROGRESS: subscribe to the high-level progress stage so
    // the onboarding flow shows the user exactly which step of the cryptographic
    // pipeline is running, with a percent-complete bar + ETA. Without this, the
    // user sees a tiny spinner + "Generating Identity keys..." for 3-5 seconds
    // with no indication of progress, which feels like a hang.
    val identityProgressStage by viewModel.identityProgressStage.collectAsState()
    // P0_ANDROID_PROGRESS_CALLBACK: transient sub-stage detail from inside the
    // FFI call. Renders as a smaller brighter line under the active stage's
    // `detail` so the user sees motion during the SharedPreferences commit()
    // and the libp2p swarm bind in persistIdentityBackup / initializeAndStartSwarm.
    val identityProgressSubDetail by viewModel.identityProgressSubDetail.collectAsState()
    var showImportDialog by remember { mutableStateOf(false) }
    var importCode by remember { mutableStateOf("") }
    var hasAcceptedConsent by remember { mutableStateOf(false) }
    var consentChecked by remember { mutableStateOf(false) }

    LaunchedEffect(isReady) {
        if (isReady) {
            onOnboardingComplete()
        }
    }

    LaunchedEffect(onboardingCompleted) {
        if (onboardingCompleted) {
            onOnboardingComplete()
        }
    }

    LaunchedEffect(importSuccess) {
        if (importSuccess) {
            viewModel.clearImportState()
            showImportDialog = false
            importCode = ""
            onOnboardingComplete()
        }
    }

    if (showImportDialog) {
        ImportContactDialog(
            importCode = importCode,
            onImportCodeChange = { importCode = it },
            importError = importError,
            onImport = { if (importCode.isNotBlank()) viewModel.importContact(importCode) },
            onDismiss = {
                showImportDialog = false
                importCode = ""
                viewModel.clearImportState()
            }
        )
    }

    Box(
        modifier = modifier
            .fillMaxSize()
            .imePadding()
            .padding(24.dp),
        contentAlignment = Alignment.Center
    ) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center,
            modifier = Modifier
                .fillMaxWidth()
                .verticalScroll(rememberScrollState())
        ) {
            Icon(
                imageVector = Icons.Filled.Lock,
                contentDescription = null,
                modifier = Modifier.size(80.dp),
                tint = MaterialTheme.colorScheme.primary
            )

            Spacer(modifier = Modifier.height(32.dp))

            Text(
                text = "Welcome to SCMessenger",
                style = MaterialTheme.typography.headlineMedium,
                textAlign = TextAlign.Center,
                modifier = Modifier.testTag("onboarding_welcome_title")
            )

            Spacer(modifier = Modifier.height(16.dp))

            Text(
                text = "Secure, private communication without central servers. Your identity is generated locally and never leaves your device.",
                style = MaterialTheme.typography.bodyLarge,
                textAlign = TextAlign.Center,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )

            Spacer(modifier = Modifier.height(32.dp))

            // ── Consent Gate ──
            if (!hasAcceptedConsent) {
                Card(
                    modifier = Modifier.fillMaxWidth(),
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.surfaceVariant
                    )
                ) {
                    Column(
                        modifier = Modifier.padding(16.dp),
                        verticalArrangement = Arrangement.spacedBy(12.dp)
                    ) {
                        Text(
                            text = "Before You Begin",
                            style = MaterialTheme.typography.titleMedium
                        )

                        ConsentInfoItem(
                            icon = Icons.Filled.Lock,
                            title = "Keypair Identity",
                            detail = "Your identity is a cryptographic keypair stored only on this device. No phone numbers, emails, or accounts."
                        )
                        ConsentInfoItem(
                            icon = Icons.Filled.Shield,
                            title = "Local-Only Data & E2E Encryption",
                            detail = "All data is stored locally. Messages are end-to-end encrypted. Only the recipient can read them."
                        )
                        ConsentInfoItem(
                            icon = Icons.Filled.CheckCircle,
                            title = "Relay Participation",
                            detail = "Your device relays encrypted messages for others. This is how the mesh network operates."
                        )
                        ConsentInfoItem(
                            icon = Icons.Filled.Warning,
                            title = "Alpha Software",
                            detail = "Expect bugs and breaking changes. Do not rely on this for critical communications."
                        )

                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            modifier = Modifier.fillMaxWidth()
                        ) {
                            Checkbox(
                                checked = consentChecked,
                                onCheckedChange = { consentChecked = it },
                                modifier = Modifier.testTag("consent_checkbox")
                            )
                            Spacer(modifier = Modifier.width(8.dp))
                            Text(
                                text = "I understand and accept these terms",
                                style = MaterialTheme.typography.bodyMedium
                            )
                        }

                        Button(
                            onClick = {
                                hasAcceptedConsent = true
                                viewModel.grantConsent()
                            },
                            enabled = consentChecked,
                            modifier = Modifier
                                .fillMaxWidth()
                                .testTag("onboarding_continue_button")
                        ) {
                            Text(stringResource(R.string.onboarding_action_continue))
                        }
                    }
                }
            } else {

                IdentityCreationFlow(
                    isCreating = isCreating,
                    onCreate = { nickname, salt ->
                        viewModel.createIdentity(nickname, salt)
                    },
                    onImport = {
                        importCode = ""
                        viewModel.clearImportState()
                        showImportDialog = true
                    },
                    showImportButton = true,
                    modifier = Modifier
                        .fillMaxWidth()
                        .imePadding()
                )

                // P0_ANDROID_IDENTITY_PROGRESS: full 6-stage proof-of-work display
                // so the user sees real progress feedback (step counter, percent
                // bar, ETA, per-stage list) during the 3-5 second Ed25519 keygen.
                // The previous tiny spinner + "Generating Identity keys..." text
                // gave no progress indication, which felt like a hang. The display
                // is gated on `!is Idle` so it only appears while creation is
                // actively running.
                if (isCreating &&
                    identityProgressStage !is com.scmessenger.android.ui.viewmodels.IdentityProgressStage.Idle) {
                    Spacer(modifier = Modifier.height(16.dp))
                    com.scmessenger.android.ui.identity.IdentityProgressDisplay(
                        currentStage = identityProgressStage,
                        // P0_ANDROID_PROGRESS_CALLBACK: pass the repo's
                        // sub-stage detail through MainViewModel (the second
                        // slot of the createIdentity callback) so the user
                        // sees motion during the FFI call.
                        subStageDetail = identityProgressSubDetail,
                        modifier = Modifier.fillMaxWidth()
                    )
                }

                Spacer(modifier = Modifier.height(8.dp))

                OutlinedButton(
                    onClick = { viewModel.skipOnboardingForRelayOnlyInstall() },
                    enabled = !isCreating,
                    modifier = Modifier.fillMaxWidth().height(56.dp)
                ) {
                    Text(stringResource(R.string.onboarding_action_skip_relay))
                }

                identityError?.let { error ->
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        text = error,
                        style = MaterialTheme.typography.bodySmall,
                        textAlign = TextAlign.Center,
                        color = MaterialTheme.colorScheme.error
                    )
                }

                Spacer(modifier = Modifier.height(8.dp))
                Text(
                    text = "You can create an identity later from Settings > Identity without reinstalling.",
                    style = MaterialTheme.typography.bodySmall,
                    textAlign = TextAlign.Center,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )

                if (!permissionsState.allPermissionsGranted) {
                    Spacer(modifier = Modifier.height(12.dp))
                    OutlinedButton(
                        onClick = { permissionsState.launchMultiplePermissionRequest() },
                        modifier = Modifier.fillMaxWidth().height(52.dp)
                    ) {
                        Text(stringResource(R.string.onboarding_action_grant_permissions))
                    }
                }
            } // end consent else
        }
    }
}

@Composable
private fun ConsentInfoItem(
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    title: String,
    detail: String
) {
    Row(
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        modifier = Modifier.fillMaxWidth()
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            modifier = Modifier.size(24.dp),
            tint = MaterialTheme.colorScheme.primary
        )
        Column {
            Text(
                text = title,
                style = MaterialTheme.typography.titleSmall
            )
            Text(
                text = detail,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
    }
}

@Composable
private fun ImportContactDialog(
    importCode: String,
    onImportCodeChange: (String) -> Unit,
    importError: String?,
    onImport: () -> Unit,
    onDismiss: () -> Unit
) {
    val context = LocalContext.current
    var qrScanError by remember { mutableStateOf<String?>(null) }

    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(stringResource(R.string.onboarding_title_import)) },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text(
                    text = "Paste the identity JSON shared by your contact, or scan their QR code.",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                
                Button(
                    onClick = {
                        val options = GmsBarcodeScannerOptions.Builder()
                            .setBarcodeFormats(Barcode.FORMAT_QR_CODE)
                            .build()
                        val scanner = GmsBarcodeScanning.getClient(context, options)
                        val qrEmptyError = context.getString(R.string.add_contact_error_qr_empty)
                        val qrFailedError = context.getString(R.string.add_contact_error_qr_failed)

                        scanner.startScan()
                            .addOnSuccessListener { barcode ->
                                val rawValue = barcode.rawValue
                                if (rawValue.isNullOrBlank()) {
                                    qrScanError = qrEmptyError
                                } else {
                                    onImportCodeChange(rawValue)
                                    qrScanError = null
                                }
                            }
                            .addOnFailureListener { e ->
                                Timber.w(e, "Onboarding QR scan failed")
                                qrScanError = qrFailedError
                            }
                    },
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Text(stringResource(R.string.onboarding_action_scan_qr))
                }

                Spacer(modifier = Modifier.height(4.dp))

                OutlinedTextField(
                    value = importCode,
                    onValueChange = onImportCodeChange,
                    label = { Text(stringResource(R.string.onboarding_label_identity_json)) },
                    modifier = Modifier.fillMaxWidth().heightIn(min = 120.dp),
                    minLines = 4,
                    maxLines = 8,
                    textStyle = LocalTextStyle.current.copy(fontFamily = FontFamily.Monospace),
                    placeholder = {
                        Text(
                            text = "{\"public_key\":\"...\",\"identity_id\":\"...\"}",
                            fontFamily = FontFamily.Monospace,
                            style = MaterialTheme.typography.bodySmall
                        )
                    }
                )
                
                val errorText = importError ?: qrScanError
                errorText?.let { error ->
                    Text(
                        text = error,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.error
                    )
                }
            }
        },
        confirmButton = {
            Button(onClick = onImport, enabled = importCode.isNotBlank()) {
                Text(stringResource(R.string.settings_action_import))
            }
        },
        dismissButton = {
            OutlinedButton(onClick = onDismiss) { Text(stringResource(R.string.cancel)) }
        }
    )
}
