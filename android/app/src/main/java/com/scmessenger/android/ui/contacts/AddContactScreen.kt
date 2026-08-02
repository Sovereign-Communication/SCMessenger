package com.scmessenger.android.ui.contacts

import android.Manifest
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import android.provider.Settings
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.core.content.ContextCompat
import androidx.hilt.navigation.compose.hiltViewModel
import com.google.android.gms.common.ConnectionResult
import com.google.android.gms.common.GoogleApiAvailability
import com.google.android.gms.common.api.CommonStatusCodes
import com.google.mlkit.common.MlKitException
import com.google.mlkit.vision.codescanner.GmsBarcodeScannerOptions
import com.google.mlkit.vision.codescanner.GmsBarcodeScanning
import com.google.mlkit.vision.barcode.common.Barcode
import com.scmessenger.android.R
import com.scmessenger.android.service.TransportType
import com.scmessenger.android.ui.components.ErrorBanner
import com.scmessenger.android.ui.components.IdenticonFromPeerId
import com.scmessenger.android.ui.viewmodels.ContactsViewModel
import com.scmessenger.android.ui.viewmodels.NearbyPeer
import com.scmessenger.android.utils.ContactImportParseResult
import com.scmessenger.android.utils.parseContactImportPayload
import kotlinx.coroutines.delay
import timber.log.Timber

/**
 * Add Contact screen - QR scan, manual entry, nearby discovery.
 *
 * Provides multiple methods to add contacts:
 * - Manual entry of peer ID and public key
 * - QR code scanning
 * - Nearby peer discovery (future)
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AddContactScreen(
    onNavigateBack: () -> Unit,
    onContactAdded: () -> Unit = {},
    prefilledPeerId: String = "",
    prefilledPublicKey: String = "",
    prefilledNickname: String = "",
    prefilledLibp2pPeerId: String? = null,
    prefilledListeners: List<String> = emptyList(),
    viewModel: ContactsViewModel = hiltViewModel()
) {
    val error by viewModel.error.collectAsState()

    var selectedTab by remember { mutableStateOf(0) }
    var peerId by remember(prefilledPeerId) { mutableStateOf(prefilledPeerId) }
    var publicKey by remember(prefilledPublicKey) { mutableStateOf(prefilledPublicKey) }
    var nickname by remember(prefilledNickname) { mutableStateOf(prefilledNickname) }
    var notes by remember { mutableStateOf("") }
    var libp2pPeerId by remember(prefilledLibp2pPeerId) { mutableStateOf(prefilledLibp2pPeerId) }
    var listeners by remember(prefilledListeners) { mutableStateOf(prefilledListeners) }
    var isAdding by remember { mutableStateOf(false) }
    var qrError by remember { mutableStateOf<String?>(null) }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(stringResource(R.string.contacts_action_add)) },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = stringResource(R.string.chat_action_dismiss))
                    }
                }
            )
        }
    ) { paddingValues ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
        ) {
            // Tab selector
            TabRow(selectedTabIndex = selectedTab) {
                Tab(
                    selected = selectedTab == 0,
                    onClick = { selectedTab = 0 },
                    text = { Text(stringResource(R.string.add_contact_tab_manual)) }
                )
                Tab(
                    selected = selectedTab == 1,
                    onClick = { selectedTab = 1 },
                    text = { Text(stringResource(R.string.add_contact_tab_qr)) }
                )
                Tab(
                    selected = selectedTab == 2,
                    onClick = { selectedTab = 2 },
                    text = { Text(stringResource(R.string.add_contact_tab_nearby)) }
                )
            }

            // Error banner
            error?.let {
                ErrorBanner(
                    message = it,
                    onDismiss = { viewModel.clearError() }
                )
            }
            qrError?.let {
                ErrorBanner(
                    message = it,
                    onDismiss = { qrError = null }
                )
            }

            // Content based on selected tab
            when (selectedTab) {
                0 -> ManualEntryTab(
                    peerId = peerId,
                    onPeerIdChange = { peerId = it },
                    publicKey = publicKey,
                    onPublicKeyChange = { publicKey = it },
                    nickname = nickname,
                    onNicknameChange = { nickname = it },
                    notes = notes,
                    onNotesChange = { notes = it },
                    isAdding = isAdding,
                    onAdd = {
                        if (peerId.isNotBlank() && publicKey.isNotBlank()) {
                            isAdding = true
                            viewModel.addContact(
                                peerId = peerId,
                                publicKey = publicKey,
                                nickname = nickname.takeIf { it.isNotBlank() },
                                libp2pPeerId = libp2pPeerId,
                                listeners = listeners,
                                notes = notes.takeIf { it.isNotBlank() },
                                onComplete = { success ->
                                    isAdding = false
                                    if (success) {
                                        peerId = ""
                                        publicKey = ""
                                        nickname = ""
                                        notes = ""
                                        onContactAdded()
                                    }
                                }
                            )
                        }
                    }
                )
                1 -> QRScanTab(
                    onScanned = { scanned ->
                        qrError = null
                        when (val parsed = parseContactImportPayload(scanned)) {
                            is ContactImportParseResult.Invalid -> {
                                qrError = parsed.reason
                                Timber.w("Invalid contact QR data: ${parsed.reason}")
                            }
                            is ContactImportParseResult.Valid -> {
                                peerId = parsed.payload.peerId
                                publicKey = parsed.payload.publicKey
                                nickname = parsed.payload.nickname ?: nickname
                                libp2pPeerId = parsed.payload.libp2pPeerId
                                listeners = parsed.payload.listeners
                                selectedTab = 0
                            }
                        }
                    },
                    onScanError = { message ->
                        qrError = message
                    }
                )
                2 -> NearbyDiscoveryTab(viewModel = viewModel)
            }
        }
    }
}

@Composable
private fun ManualEntryTab(
    peerId: String,
    onPeerIdChange: (String) -> Unit,
    publicKey: String,
    onPublicKeyChange: (String) -> Unit,
    nickname: String,
    onNicknameChange: (String) -> Unit,
    notes: String,
    onNotesChange: (String) -> Unit,
    isAdding: Boolean,
    onAdd: () -> Unit
) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        // Preview
        if (peerId.isNotBlank()) {
            Card {
                Row(
                    modifier = Modifier.padding(16.dp),
                    horizontalArrangement = Arrangement.spacedBy(16.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    IdenticonFromPeerId(peerId = peerId, size = 64.dp)

                    Column {
                        val unknownFallback = stringResource(R.string.unknown_contact)
                        Text(
                            text = nickname.ifBlank { unknownFallback },
                            style = MaterialTheme.typography.titleMedium,
                            fontWeight = FontWeight.Bold
                        )
                        Text(
                            text = peerId.take(16) + "...",
                            style = MaterialTheme.typography.bodySmall,
                            fontFamily = FontFamily.Monospace
                        )
                    }
                }
            }
        }

        // Peer ID input
        OutlinedTextField(
            value = peerId,
            onValueChange = onPeerIdChange,
            label = { Text(stringResource(R.string.add_contact_label_peer_id_required)) },
            modifier = Modifier.fillMaxWidth(),
            singleLine = true,
            enabled = !isAdding
        )

        // Public Key input
        OutlinedTextField(
            value = publicKey,
            onValueChange = onPublicKeyChange,
            label = { Text(stringResource(R.string.add_contact_label_public_key_required)) },
            modifier = Modifier.fillMaxWidth(),
            minLines = 2,
            maxLines = 4,
            enabled = !isAdding
        )

        // Nickname input
        OutlinedTextField(
            value = nickname,
            onValueChange = onNicknameChange,
            label = { Text(stringResource(R.string.add_contact_label_nickname_optional)) },
            modifier = Modifier.fillMaxWidth(),
            singleLine = true,
            enabled = !isAdding
        )

        // Notes input
        OutlinedTextField(
            value = notes,
            onValueChange = onNotesChange,
            label = { Text(stringResource(R.string.add_contact_label_notes_optional)) },
            modifier = Modifier.fillMaxWidth(),
            minLines = 3,
            maxLines = 5,
            enabled = !isAdding
        )

        // Add button
        Button(
            onClick = onAdd,
            modifier = Modifier.fillMaxWidth(),
            enabled = !isAdding && peerId.isNotBlank() && publicKey.isNotBlank()
        ) {
            if (isAdding) {
                CircularProgressIndicator(
                    modifier = Modifier.size(20.dp),
                    color = MaterialTheme.colorScheme.onPrimary
                )
            } else {
                Icon(Icons.Default.Add, contentDescription = stringResource(R.string.contacts_action_add))
                Spacer(modifier = Modifier.width(8.dp))
                Text(stringResource(R.string.contacts_action_add))
            }
        }
        Spacer(modifier = Modifier.navigationBarsPadding())
    }
}

@Composable
private fun QRScanTab(
    onScanned: (String) -> Unit,
    onScanError: (String) -> Unit
) {
    val context = LocalContext.current
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(32.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Icon(
            imageVector = Icons.Default.Info,
            contentDescription = null,
            modifier = Modifier.size(64.dp),
            tint = MaterialTheme.colorScheme.primary
        )

        Spacer(modifier = Modifier.height(16.dp))

        Text(
            text = stringResource(R.string.add_contact_qr_title),
            style = MaterialTheme.typography.titleLarge
        )

        Spacer(modifier = Modifier.height(8.dp))

        Text(
            text = stringResource(R.string.add_contact_qr_description),
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )

        Spacer(modifier = Modifier.height(24.dp))

        val gmsAvailable = remember {
            GoogleApiAvailability.getInstance().isGooglePlayServicesAvailable(context) == ConnectionResult.SUCCESS
        }
        val gmsUnavailableError = stringResource(R.string.add_contact_error_gms_unavailable)
        val qrEmptyError = stringResource(R.string.add_contact_error_qr_empty)
        val qrFailedError = stringResource(R.string.add_contact_error_qr_failed)

        Button(
            onClick = {
                if (!gmsAvailable) {
                    onScanError(gmsUnavailableError)
                    return@Button
                }
                val options = GmsBarcodeScannerOptions.Builder()
                    .setBarcodeFormats(Barcode.FORMAT_QR_CODE)
                    .build()
                val scanner = GmsBarcodeScanning.getClient(context, options)
                scanner.startScan()
                    .addOnSuccessListener { barcode ->
                        val rawValue = barcode.rawValue
                        if (rawValue.isNullOrBlank()) {
                            onScanError(qrEmptyError)
                        } else {
                            onScanned(rawValue)
                        }
                    }
                    .addOnFailureListener { e ->
                        Timber.w(e, "QR scan failed")
                        if (e is MlKitException && e.errorCode == CommonStatusCodes.CANCELED) {
                            return@addOnFailureListener
                        }
                        onScanError(qrFailedError)
                    }
            },
            enabled = gmsAvailable
        ) {
            Icon(Icons.Default.CameraAlt, contentDescription = stringResource(R.string.contacts_label_scan_qr))
            Spacer(modifier = Modifier.width(8.dp))
            Text(if (gmsAvailable) stringResource(R.string.add_contact_action_scan) else stringResource(R.string.add_contact_action_scan_unavailable))
        }

        if (!gmsAvailable) {
            Spacer(modifier = Modifier.height(8.dp))
            Text(
                text = stringResource(R.string.add_contact_gms_requirement_note),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.error
            )
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun NearbyDiscoveryTab(viewModel: ContactsViewModel) {
    val context = LocalContext.current
    val nearbyPeers by viewModel.nearbyPeers.collectAsState()
    val error by viewModel.error.collectAsState()
    val isScanning by viewModel.isScanning.collectAsState()

    // Permission state — recompute when this composable re-enters composition
    // (e.g. user backgrounds and returns after toggling permissions).
    var hasPermissions by remember { mutableStateOf(isNearbyPermissionsGranted(context)) }

    // Elapsed timer for rescan
    var scanStartMs by remember { mutableLongStateOf(0L) }
    var elapsedSeconds by remember { mutableLongStateOf(0L) }

    LaunchedEffect(isScanning) {
        if (isScanning) {
            scanStartMs = System.currentTimeMillis()
            while (isScanning) {
                elapsedSeconds = (System.currentTimeMillis() - scanStartMs) / 1000L
                delay(1000L)
            }
        }
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(horizontal = 16.dp, vertical = 8.dp)
    ) {
        // Top bar: title + Rescan action
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(vertical = 8.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = stringResource(R.string.add_contact_nearby_title),
                    style = MaterialTheme.typography.titleLarge
                )
                Text(
                    text = stringResource(R.string.add_contact_nearby_description),
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
            if (isScanning) {
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    CircularProgressIndicator(modifier = Modifier.size(16.dp), strokeWidth = 2.dp)
                    Text(
                        text = "${elapsedSeconds}s",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.primary
                    )
                }
            } else {
                IconButton(
                    onClick = {
                        viewModel.refreshDiscovery()
                        hasPermissions = isNearbyPermissionsGranted(context)
                    }
                ) {
                    Icon(
                        imageVector = Icons.Default.Refresh,
                        contentDescription = stringResource(R.string.add_contact_nearby_rescan_content_description)
                    )
                }
            }
        }

        // Error banner (only show if the error isn't about a missing public key,
        // which we surface inline on the row instead).
        val inlineError = error?.contains("no public key", ignoreCase = true) == true
        if (error != null && !inlineError) {
            ErrorBanner(
                message = error ?: "",
                onDismiss = { viewModel.clearError() }
            )
            Spacer(Modifier.height(8.dp))
        }

        // Content
        Box(modifier = Modifier.fillMaxSize()) {
            when {
                !hasPermissions -> PermissionRationaleCard(
                    onGrant = {
                        val intent = Intent(
                            Settings.ACTION_APPLICATION_DETAILS_SETTINGS,
                            Uri.fromParts("package", context.packageName, null)
                        ).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                        context.startActivity(intent)
                    }
                )
                nearbyPeers.isEmpty() -> {
                    if (isScanning) {
                        ScanningState(elapsedSeconds = elapsedSeconds)
                    } else {
                        EmptyState(
                            onRescan = {
                                viewModel.refreshDiscovery()
                            }
                        )
                    }
                }
                else -> PeerList(
                    peers = nearbyPeers,
                    onAdd = { peer ->
                        val ok = viewModel.promoteNearbyPeerToContact(peer)
                        if (ok) {
                            Timber.i("Promoted nearby peer to contact: ${peer.peerId.take(16)}")
                        }
                    },
                    onDismissPeer = { peer ->
                        viewModel.dismissNearbyPeer(peer)
                    },
                    hasInlineError = inlineError,
                    inlineErrorMessage = error,
                    onDismissInlineError = { viewModel.clearError() }
                )
            }
        }
    }
}

@Composable
private fun PermissionRationaleCard(onGrant: () -> Unit) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .padding(top = 16.dp),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.tertiaryContainer
        )
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                Icon(
                    imageVector = Icons.Default.Warning,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.onTertiaryContainer
                )
                Text(
                    text = stringResource(R.string.add_contact_nearby_permission_rationale),
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onTertiaryContainer
                )
            }
            Button(onClick = onGrant) {
                Icon(
                    imageVector = Icons.Default.Settings,
                    contentDescription = null
                )
                Spacer(Modifier.width(8.dp))
                Text(stringResource(R.string.add_contact_nearby_grant_permissions))
            }
        }
    }
}

@Composable
private fun ScanningState(elapsedSeconds: Long = 0L) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(32.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        CircularProgressIndicator()
        Spacer(Modifier.height(16.dp))
        Text(
            text = stringResource(R.string.add_contact_nearby_searching),
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
        if (elapsedSeconds > 0) {
            Spacer(Modifier.height(8.dp))
            Text(
                text = "${elapsedSeconds}s elapsed",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.7f)
            )
        }
    }
}

@Composable
private fun EmptyState(onRescan: () -> Unit) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(32.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Icon(
            imageVector = Icons.Default.Search,
            contentDescription = null,
            modifier = Modifier.size(64.dp),
            tint = MaterialTheme.colorScheme.primary
        )
        Spacer(Modifier.height(16.dp))
        Text(
            text = stringResource(R.string.add_contact_nearby_empty),
            style = MaterialTheme.typography.titleMedium
        )
        Spacer(Modifier.height(16.dp))
        Button(onClick = onRescan) {
            Icon(Icons.Default.Refresh, contentDescription = null)
            Spacer(Modifier.width(8.dp))
            Text(stringResource(R.string.add_contact_nearby_rescan))
        }
    }
}

@Composable
private fun PeerList(
    peers: List<NearbyPeer>,
    onAdd: (NearbyPeer) -> Unit,
    onDismissPeer: (NearbyPeer) -> Unit,
    hasInlineError: Boolean,
    inlineErrorMessage: String?,
    onDismissInlineError: () -> Unit
) {
    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(vertical = 8.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        if (hasInlineError && inlineErrorMessage != null) {
            item(key = "nearby_error") {
                ErrorBanner(message = inlineErrorMessage, onDismiss = onDismissInlineError)
            }
        }
        items(peers, key = { it.peerId }) { peer ->
            NearbyPeerCard(peer = peer, onAdd = { onAdd(peer) }, onDismiss = { onDismissPeer(peer) })
        }
    }
}

@Composable
private fun NearbyPeerCard(peer: NearbyPeer, onAdd: () -> Unit, onDismiss: () -> Unit) {
    Card(
        modifier = Modifier.fillMaxWidth()
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            IdenticonFromPeerId(peerId = peer.peerId, size = 48.dp)
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = peer.displayName,
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.SemiBold
                )
                Text(
                    text = peer.peerId.take(16) + "…",
                    style = MaterialTheme.typography.bodySmall,
                    fontFamily = FontFamily.Monospace,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                if (!peer.isOnline) {
                    Text(
                        text = stringResource(R.string.add_contact_nearby_offline_badge),
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.error
                    )
                }
            }
            TransportIcon(transport = peer.transport)
            IconButton(onClick = onDismiss) {
                Icon(
                    imageVector = Icons.Default.Close,
                    contentDescription = stringResource(R.string.add_contact_nearby_dismiss_content_description)
                )
            }
            FilledTonalIconButton(
                onClick = onAdd,
                enabled = peer.publicKey != null
            ) {
                Icon(
                    imageVector = Icons.Default.Add,
                    contentDescription = stringResource(R.string.add_contact_nearby_add_content_description)
                )
            }
        }
    }
}

@Composable
private fun TransportIcon(transport: TransportType?) {
    val (icon, label) = when (transport) {
        TransportType.BLE -> Icons.Default.Bluetooth to stringResource(R.string.add_contact_nearby_transport_ble)
        TransportType.WIFI_AWARE -> Icons.Default.Wifi to stringResource(R.string.add_contact_nearby_transport_wifi_aware)
        TransportType.WIFI_DIRECT -> Icons.Default.Wifi to stringResource(R.string.add_contact_nearby_transport_wifi_direct)
        TransportType.INTERNET -> Icons.Default.Public to stringResource(R.string.add_contact_nearby_transport_internet)
        TransportType.TCP_MDNS -> Icons.Default.Router to stringResource(R.string.add_contact_nearby_transport_tcp_mdns)
        null -> Icons.Default.HelpOutline to stringResource(R.string.add_contact_nearby_transport_unknown)
    }
    Icon(
        imageVector = icon,
        contentDescription = label,
        tint = MaterialTheme.colorScheme.secondary
    )
}

/**
 * Returns true if the runtime permissions required for nearby BLE + Wi-Fi discovery
 * are currently granted.
 *
 * Android 12 (API 31)+ requires [Manifest.permission.BLUETOOTH_SCAN],
 * [Manifest.permission.BLUETOOTH_CONNECT] and [Manifest.permission.NEARBY_WIFI_DEVICES].
 * Older releases fall back to [Manifest.permission.ACCESS_FINE_LOCATION] for BLE scanning.
 */
fun isNearbyPermissionsGranted(context: Context): Boolean {
    val required: Array<String> = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
        arrayOf(
            Manifest.permission.BLUETOOTH_SCAN,
            Manifest.permission.BLUETOOTH_CONNECT,
            Manifest.permission.NEARBY_WIFI_DEVICES
        )
    } else {
        arrayOf(Manifest.permission.ACCESS_FINE_LOCATION)
    }
    return required.all { perm ->
        ContextCompat.checkSelfPermission(context, perm) == PackageManager.PERMISSION_GRANTED
    }
}
