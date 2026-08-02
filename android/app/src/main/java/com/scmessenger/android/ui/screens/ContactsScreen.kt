package com.scmessenger.android.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.CameraAlt
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.ContentPaste
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.PersonAdd
import androidx.compose.material.icons.filled.Sensors
import androidx.compose.material3.SwipeToDismissBox
import androidx.compose.material3.SwipeToDismissBoxValue
import androidx.compose.material3.rememberSwipeToDismissBoxState
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.google.android.gms.common.api.CommonStatusCodes
import com.google.mlkit.common.MlKitException
import com.google.mlkit.vision.barcode.common.Barcode
import com.google.mlkit.vision.codescanner.GmsBarcodeScannerOptions
import com.google.mlkit.vision.codescanner.GmsBarcodeScanning
import androidx.compose.ui.res.stringResource
import com.scmessenger.android.R
import com.scmessenger.android.ui.viewmodels.ContactsViewModel
import com.scmessenger.android.ui.viewmodels.NearbyPeer
import com.scmessenger.android.utils.ContactImportParseResult
import com.scmessenger.android.utils.parseContactImportPayload
import com.scmessenger.android.utils.toEpochMillis
import java.text.SimpleDateFormat
import java.util.*

/**
 * Contacts screen with list, search, and add/remove functionality.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ContactsScreen(
    viewModel: ContactsViewModel = hiltViewModel(),
    onNavigateToChat: (String) -> Unit,
    onNavigateToAddContact: () -> Unit = {},
    onNavigateToContactDetail: ((String) -> Unit)? = null
) {
    val contacts by viewModel.filteredContacts.collectAsState()
    val nearbyPeers by viewModel.nearbyPeers.collectAsState()
    val isLoading by viewModel.isLoading.collectAsState()
    val error by viewModel.error.collectAsState()
    val searchQuery by viewModel.searchQuery.collectAsState()

    var showAddDialog by remember { mutableStateOf(false) }
    var nearbyPrefilledPeer by remember { mutableStateOf<NearbyPeer?>(null) }

    // No inner Scaffold here — the outer Scaffold in MeshApp.kt hosts the
    // TopAppBar (per-screen) and the contacts "+" FAB (route-conditional).
    // Nested Scaffolds caused the FAB to be hidden or mispositioned.
    Column(
        modifier = Modifier.fillMaxSize()
    ) {
        // Per-screen title bar (rendered as plain content, not a TopAppBar,
        // so the outer Scaffold's TopAppBar slot can own window insets)
        Text(
            text = stringResource(R.string.contacts_title, contacts.size),
            style = MaterialTheme.typography.titleLarge,
            modifier = Modifier.padding(start = 16.dp, end = 16.dp, top = 16.dp, bottom = 8.dp)
        )

        // Search bar
        OutlinedTextField(
                value = searchQuery,
                onValueChange = { viewModel.setSearchQuery(it) },
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(16.dp),
                placeholder = { Text(stringResource(R.string.contacts_placeholder_search)) },
                singleLine = true,
                trailingIcon = {
                    if (searchQuery.isNotBlank()) {
                        IconButton(onClick = { viewModel.clearSearch() }) {
                            Icon(Icons.Default.Close, contentDescription = stringResource(R.string.contacts_action_clear_search))
                        }
                    }
                }
            )

            // Error snackbar
            error?.let { errorMsg ->
                Snackbar(
                    modifier = Modifier.padding(16.dp),
                    action = {
                        TextButton(onClick = { viewModel.clearError() }) {
                            Text(stringResource(R.string.chat_action_dismiss))
                        }
                    }
                ) {
                    Text(errorMsg)
                }
            }

            // Loading indicator
            if (isLoading) {
                Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.Center
                ) {
                    CircularProgressIndicator()
                }
            } else if (contacts.isEmpty() && nearbyPeers.isEmpty()) {
                // Empty state
                Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.Center
                ) {
                    Column(
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        Icon(
                            imageVector = Icons.Default.Person,
                            contentDescription = null,
                            modifier = Modifier.size(64.dp),
                            tint = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Spacer(modifier = Modifier.height(16.dp))
                        Text(
                            text = if (searchQuery.isBlank()) {
                                stringResource(R.string.contacts_empty_state_none)
                            } else {
                                stringResource(R.string.contacts_empty_state_not_found)
                            },
                            style = MaterialTheme.typography.bodyLarge,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        if (searchQuery.isBlank()) {
                            Spacer(modifier = Modifier.height(8.dp))
                            Text(
                                text = stringResource(R.string.contacts_empty_state_description),
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    }
                }
            } else {
                LazyColumn(
                    modifier = Modifier.fillMaxSize(),
                    // UI Fix B: bottom padding (~88dp) leaves clearance for the
                    // outer Scaffold's contacts "+" FAB so the last list item
                    // isn't hidden behind the FAB. FAB default size is 56dp
                    // with 16dp margin -> 72dp; we add a small extra to keep
                    // the swipe-to-dismiss affordance on the last row visible.
                    // PaddingValues(start, top, end, bottom) is the 4-arg form.
                    contentPadding = PaddingValues(start = 16.dp, top = 8.dp, end = 16.dp, bottom = 88.dp)
                ) {
                    // Nearby peers section — discovered but not yet saved
                    if (nearbyPeers.isNotEmpty()) {
                        item {
                            Row(
                                verticalAlignment = Alignment.CenterVertically,
                                modifier = Modifier.padding(bottom = 4.dp, top = 4.dp)
                            ) {
                                Icon(
                                    imageVector = Icons.Default.Sensors,
                                    contentDescription = null,
                                    tint = MaterialTheme.colorScheme.primary,
                                    modifier = Modifier.size(16.dp)
                                )
                                Spacer(modifier = Modifier.width(4.dp))
                                Text(
                                    text = stringResource(R.string.contacts_section_nearby, nearbyPeers.size),
                                    style = MaterialTheme.typography.labelMedium,
                                    color = MaterialTheme.colorScheme.primary
                                )
                            }
                        }
                        items(nearbyPeers, key = { "nearby_${it.peerId}" }) { peer ->
                            NearbyPeerItem(
                                peer = peer,
                                onAdd = {
                                    nearbyPrefilledPeer = peer
                                    showAddDialog = true
                                },
                                onConnect = {
                                    val publicKey = peer.publicKey?.trim()
                                    if (publicKey.isNullOrEmpty()) {
                                        nearbyPrefilledPeer = peer
                                        showAddDialog = true
                                    } else {
                                        val bleRoute = peer.blePeerId?.takeIf { it.isNotBlank() }
                                        val notes = bleRoute?.let { "ble_peer_id:$it" }
                                        viewModel.addContact(
                                            peerId = peer.peerId,
                                            publicKey = publicKey,
                                            nickname = peer.nickname,
                                            libp2pPeerId = peer.libp2pPeerId,
                                            listeners = peer.listeners,
                                            notes = notes
                                        )
                                        onNavigateToChat(peer.peerId)
                                    }
                                }
                            )
                            Spacer(modifier = Modifier.height(8.dp))
                        }
                        if (contacts.isNotEmpty()) {
                            item {
                                HorizontalDivider(modifier = Modifier.padding(vertical = 4.dp))
                                Text(
                                    text = stringResource(R.string.contacts_title, contacts.size),
                                    style = MaterialTheme.typography.labelMedium,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                                    modifier = Modifier.padding(bottom = 4.dp)
                                )
                            }
                        }
                    }
                    // Saved contacts
                    items(contacts, key = { it.peerId }) { contact ->
                        ContactItem(
                            contact = contact,
                            onClick = { onNavigateToChat(contact.peerId) },
                            onDetails = {
                                onNavigateToContactDetail?.invoke(contact.peerId)
                            },
                            onDelete = { viewModel.removeContact(contact.peerId) },
                            onEditNickname = { nickname ->
                                viewModel.setLocalNickname(contact.peerId, nickname.ifBlank { null })
                            }
                        )
                        Spacer(modifier = Modifier.height(8.dp))
                    }
                }
            }
        }

    // Add contact dialog
    if (showAddDialog) {
        val nearbyBlePeerId = nearbyPrefilledPeer?.blePeerId ?: ""
        val nearbyLibp2p = nearbyPrefilledPeer?.libp2pPeerId ?: ""
        val nearbyListeners = nearbyPrefilledPeer?.listeners ?: emptyList()

        AddContactDialog(
            prefilledPeerId = nearbyPrefilledPeer?.peerId ?: "",
            prefilledPublicKey = nearbyPrefilledPeer?.publicKey ?: "",
            prefilledNickname = nearbyPrefilledPeer?.nickname ?: "",
            onDismiss = {
                showAddDialog = false
                nearbyPrefilledPeer = null
            },
            onAdd = { peerId, publicKey, nickname, importedLibp2p, importedListeners ->
                val effectiveLibp2p = importedLibp2p ?: nearbyLibp2p.takeIf { it.isNotBlank() }
                val effectiveListeners = if (importedListeners.isNotEmpty()) importedListeners else nearbyListeners
                val notes = nearbyBlePeerId.takeIf { it.isNotBlank() }?.let { "ble_peer_id:$it" }
                viewModel.addContact(peerId, publicKey, nickname, effectiveLibp2p, effectiveListeners, notes)
                showAddDialog = false
                nearbyPrefilledPeer = null
            },
            onAddAndChat = { peerId, publicKey, nickname, importedLibp2p, importedListeners ->
                val id = peerId.trim()
                if (id.isNotBlank() && publicKey.isNotBlank()) {
                    val effectiveLibp2p = importedLibp2p ?: nearbyLibp2p.takeIf { it.isNotBlank() }
                    val effectiveListeners = if (importedListeners.isNotEmpty()) importedListeners else nearbyListeners
                    val notes = nearbyBlePeerId.takeIf { it.isNotBlank() }?.let { "ble_peer_id:$it" }
                    viewModel.addContact(id, publicKey.trim(), nickname?.trim(), effectiveLibp2p, effectiveListeners, notes)
                    showAddDialog = false
                    nearbyPrefilledPeer = null
                    // Contacts are persisted by canonical public-key identity;
                    // use the same key for the chat route after adding a
                    // nearby/imported libp2p peer.
                    onNavigateToChat(publicKey.trim().lowercase())
                }
            }
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ContactItem(
    contact: uniffi.api.Contact,
    onClick: () -> Unit,
    onDetails: () -> Unit,
    onDelete: () -> Unit,
    onEditNickname: (String) -> Unit = {}
) {
    var showDeleteDialog by remember { mutableStateOf(false) }
    var showEditNicknameDialog by remember { mutableStateOf(false) }
    var showDetails by remember { mutableStateOf(false) }

    val dismissState = rememberSwipeToDismissBoxState()

    SwipeToDismissBox(
        state = dismissState,
        backgroundContent = {
            val color = when (dismissState.targetValue) {
                SwipeToDismissBoxValue.StartToEnd, SwipeToDismissBoxValue.EndToStart -> MaterialTheme.colorScheme.error
                else -> Color.Transparent
            }
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .background(color)
                    .padding(horizontal = 20.dp),
                contentAlignment = Alignment.CenterEnd
            ) {
                if (dismissState.targetValue != SwipeToDismissBoxValue.Settled) {
                    Icon(
                        imageVector = Icons.Default.Delete,
                        contentDescription = stringResource(R.string.delete),
                        tint = Color.White,
                        modifier = Modifier.size(32.dp)
                    )
                }
            }
        },
        content = {
        Card(
            modifier = Modifier
                .fillMaxWidth()
                .clickable(onClick = onClick)
        ) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(16.dp),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                // UI Fix A: weight(1f) on the left column so the nickname/peer-id
                // text takes all available width and the trailing action icons
                // stay right-aligned instead of being pushed off-screen by the
                // ID text. Matches the pattern in NearbyPeerItem (line 543).
                Column(modifier = Modifier.weight(1f)) {
                    val unknownFallback = stringResource(R.string.unknown_contact)
                    val currentNickname = contact.localNickname ?: contact.nickname ?: ""
                    Text(
                        text = currentNickname.ifBlank { unknownFallback },
                        style = MaterialTheme.typography.titleMedium
                    )
                    if (contact.localNickname != null && contact.nickname != null) {
                        Text(
                            text = "@${contact.nickname}",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                    Text(
                        text = "ID: ${contact.peerId.take(16)}...",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    val lastSeen = contact.lastSeen
                    if (lastSeen != null) {
                        Text(
                            text = stringResource(R.string.contacts_dialog_details_last_seen, formatTimestamp(lastSeen)),
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }

                Row {
                    IconButton(onClick = { showDetails = true }) {
                        Icon(
                            imageVector = Icons.Default.Info,
                            contentDescription = stringResource(R.string.contacts_action_details),
                            tint = MaterialTheme.colorScheme.primary
                        )
                    }

                    IconButton(onClick = { showEditNicknameDialog = true }) {
                        Icon(
                            imageVector = Icons.Default.Edit,
                            contentDescription = stringResource(R.string.contacts_action_edit_nickname),
                            tint = MaterialTheme.colorScheme.primary
                        )
                    }
                }
            }
        }
        }
    )

    // Contact details dialog
    if (showDetails) {
        AlertDialog(
            onDismissRequest = { showDetails = false },
            title = { Text(stringResource(R.string.contacts_dialog_details_title)) },
            text = {
                Column {
                    Text(
                        text = stringResource(R.string.contacts_dialog_details_peer_id, contact.peerId),
                        style = MaterialTheme.typography.bodyMedium,
                        fontFamily = androidx.compose.ui.text.font.FontFamily.Monospace
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        text = stringResource(R.string.contacts_dialog_details_public_key, contact.publicKey.take(32)),
                        style = MaterialTheme.typography.bodyMedium,
                        fontFamily = androidx.compose.ui.text.font.FontFamily.Monospace
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    val nickname = contact.nickname
                    if (nickname != null) {
                        Text(
                            text = stringResource(R.string.contacts_dialog_details_nickname, nickname),
                            style = MaterialTheme.typography.bodyMedium
                        )
                    }
                    val lastSeen = contact.lastSeen
                    if (lastSeen != null) {
                        Text(
                            text = stringResource(R.string.contacts_dialog_details_last_seen, formatTimestamp(lastSeen)),
                            style = MaterialTheme.typography.bodyMedium
                        )
                    }
                }
            },
            confirmButton = {
                TextButton(
                    onClick = {
                        showDetails = false
                        onDetails()
                    }
                ) {
                    Text(stringResource(R.string.contacts_action_view_full_details))
                }
            },
            dismissButton = {
                TextButton(onClick = { showDetails = false }) {
                    Text(stringResource(R.string.contacts_action_close))
                }
            }
        )
    }

    // Edit nickname dialog
    if (showEditNicknameDialog) {
        var newNickname by remember { mutableStateOf(contact.localNickname ?: contact.nickname ?: "") }
        val focusRequester = remember { FocusRequester() }

        AlertDialog(
            onDismissRequest = { showEditNicknameDialog = false },
            title = { Text(stringResource(R.string.contacts_action_edit_nickname)) },
            text = {
                Column {
                    Text(
                        text = stringResource(R.string.contacts_dialog_edit_nickname_description, contact.peerId.take(16)),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    OutlinedTextField(
                        value = newNickname,
                        onValueChange = { newNickname = it },
                        label = { Text(stringResource(R.string.settings_label_nickname)) },
                        singleLine = true,
                        modifier = Modifier
                            .fillMaxWidth()
                            .focusRequester(focusRequester)
                    )
                    if (contact.nickname != null) {
                        Spacer(modifier = Modifier.height(4.dp))
                        Text(
                            text = "Federated nickname: @${contact.nickname}",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
            },
            confirmButton = {
                TextButton(
                    onClick = {
                        onEditNickname(newNickname.trim())
                        showEditNicknameDialog = false
                    }
                ) {
                    Text(stringResource(R.string.contacts_action_save))
                }
            },
            dismissButton = {
                TextButton(onClick = { showEditNicknameDialog = false }) {
                    Text(stringResource(R.string.cancel))
                }
            }
        )
        
        // Request focus on dialog open
        LaunchedEffect(Unit) {
            focusRequester.requestFocus()
        }
    }

    // Confirm delete dialog
    if (showDeleteDialog) {
        AlertDialog(
            onDismissRequest = { showDeleteDialog = false },
            title = { Text(stringResource(R.string.contacts_dialog_delete_title)) },
            text = {
                Text(stringResource(R.string.contacts_dialog_delete_description, contact.localNickname ?: contact.nickname ?: "this contact"))
            },
            confirmButton = {
                TextButton(
                    onClick = {
                        onDelete()
                        showDeleteDialog = false
                    }
                ) {
                    Text(stringResource(R.string.delete), color = MaterialTheme.colorScheme.error)
                }
            },
            dismissButton = {
                TextButton(onClick = { showDeleteDialog = false }) {
                    Text(stringResource(R.string.cancel))
                }
            }
        )
    }
}

@Composable
fun NearbyPeerItem(
    peer: NearbyPeer,
    onAdd: () -> Unit,
    onConnect: () -> Unit
) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.secondaryContainer.copy(alpha = 0.4f)
        )
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier.weight(1f)
            ) {
                Icon(
                    imageVector = Icons.Default.Sensors,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.primary,
                    modifier = Modifier.size(36.dp)
                )
                Spacer(modifier = Modifier.width(12.dp))
                Column {
                    Text(
                        text = peer.displayName,
                        style = MaterialTheme.typography.titleSmall
                    )
                    if (peer.hasFullIdentity) {
                        Text(
                            text = "● Identity verified",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.primary
                        )
                    } else {
                        Text(
                            text = peer.peerId.take(20) + "…",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            fontFamily = androidx.compose.ui.text.font.FontFamily.Monospace
                        )
                    }
                }
            }
            FilledTonalButton(onClick = if (peer.hasFullIdentity) onConnect else onAdd) {
                Icon(
                    imageVector = Icons.Default.PersonAdd,
                    contentDescription = stringResource(R.string.contacts_content_desc_add_contact),
                    modifier = Modifier.size(16.dp)
                )
                Spacer(modifier = Modifier.width(4.dp))
                Text(stringResource(R.string.contacts_action_add_button))
            }
        }
    }
}

@Composable
fun AddContactDialog(
    prefilledPeerId: String = "",
    prefilledPublicKey: String = "",
    prefilledNickname: String = "",
    onDismiss: () -> Unit,
    onAdd: (String, String, String?, String?, List<String>) -> Unit,
    onAddAndChat: (String, String, String?, String?, List<String>) -> Unit
) {
    var peerId by remember(prefilledPeerId) { mutableStateOf(prefilledPeerId) }
    var publicKey by remember(prefilledPublicKey) { mutableStateOf(prefilledPublicKey) }
    var nickname by remember(prefilledNickname) { mutableStateOf(prefilledNickname) }
    var libp2pPeerId by remember { mutableStateOf<String?>(null) }
    var listeners by remember { mutableStateOf<List<String>>(emptyList()) }
    var parseError by remember { mutableStateOf<String?>(null) }

    val clipboardManager = androidx.compose.ui.platform.LocalClipboardManager.current
    val context = LocalContext.current

    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(stringResource(R.string.contacts_action_add)) },
        text = {
            Column {
                OutlinedButton(
                    onClick = {
                        val text = clipboardManager.getText()?.text?.toString().orEmpty()
                        when (val parsed = parseContactImportPayload(text)) {
                            is ContactImportParseResult.Valid -> {
                                peerId = parsed.payload.peerId
                                publicKey = parsed.payload.publicKey
                                nickname = parsed.payload.nickname ?: nickname
                                libp2pPeerId = parsed.payload.libp2pPeerId
                                listeners = parsed.payload.listeners
                                parseError = null
                            }
                            is ContactImportParseResult.Invalid -> {
                                parseError = parsed.reason
                            }
                        }
                    },
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Icon(Icons.Default.ContentPaste, contentDescription = stringResource(R.string.contacts_content_desc_paste), modifier = Modifier.size(16.dp))
                    Spacer(modifier = Modifier.width(8.dp))
                    Text(stringResource(R.string.contacts_label_paste_identity))
                }

                Spacer(modifier = Modifier.height(8.dp))
                OutlinedButton(
                    onClick = {
                        val options = GmsBarcodeScannerOptions.Builder()
                            .setBarcodeFormats(Barcode.FORMAT_QR_CODE)
                            .build()
                        val scanner = GmsBarcodeScanning.getClient(context, options)
                        scanner.startScan()
                            .addOnSuccessListener { barcode ->
                                val raw = barcode.rawValue.orEmpty()
                                when (val parsed = parseContactImportPayload(raw)) {
                                    is ContactImportParseResult.Valid -> {
                                        peerId = parsed.payload.peerId
                                        publicKey = parsed.payload.publicKey
                                        nickname = parsed.payload.nickname ?: nickname
                                        libp2pPeerId = parsed.payload.libp2pPeerId
                                        listeners = parsed.payload.listeners
                                        parseError = null
                                    }
                                    is ContactImportParseResult.Invalid -> {
                                        parseError = parsed.reason
                                    }
                                }
                            }
                            .addOnFailureListener { e ->
                                if (e is MlKitException && e.errorCode == CommonStatusCodes.CANCELED) {
                                    return@addOnFailureListener
                                }
                                parseError = "Unable to scan QR code."
                            }
                    },
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Icon(Icons.Default.CameraAlt, contentDescription = stringResource(R.string.contacts_content_desc_scan_qr), modifier = Modifier.size(16.dp))
                    Spacer(modifier = Modifier.width(8.dp))
                    Text(stringResource(R.string.contacts_label_scan_qr))
                }

                Spacer(modifier = Modifier.height(16.dp))
                HorizontalDivider()
                Spacer(modifier = Modifier.height(16.dp))

                parseError?.let {
                    Text(
                        text = it,
                        color = MaterialTheme.colorScheme.error,
                        style = MaterialTheme.typography.bodySmall
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                }

                OutlinedTextField(
                    value = peerId,
                    onValueChange = { peerId = it },
                    label = { Text(stringResource(R.string.contacts_label_peer_id)) },
                    modifier = Modifier.fillMaxWidth(),
                    singleLine = true
                )
                Spacer(modifier = Modifier.height(8.dp))
                OutlinedTextField(
                    value = publicKey,
                    onValueChange = { publicKey = it },
                    label = { Text(stringResource(R.string.contacts_label_public_key)) },
                    modifier = Modifier.fillMaxWidth(),
                    singleLine = true
                )
                Spacer(modifier = Modifier.height(8.dp))
                OutlinedTextField(
                    value = nickname,
                    onValueChange = { nickname = it },
                    label = { Text(stringResource(R.string.contacts_label_nickname_optional)) },
                    modifier = Modifier.fillMaxWidth(),
                    singleLine = true
                )
            }
        },
        confirmButton = {
            Row {
                TextButton(
                    onClick = {
                        if (peerId.isNotBlank() && publicKey.isNotBlank()) {
                            onAdd(
                                peerId.trim(),
                                publicKey.trim(),
                                nickname.trim().ifBlank { null },
                                libp2pPeerId,
                                listeners
                            )
                        }
                    },
                    enabled = peerId.isNotBlank() && publicKey.isNotBlank()
                ) {
                    Text(stringResource(R.string.contacts_action_add_button))
                }
                Spacer(modifier = Modifier.width(8.dp))
                TextButton(
                    onClick = {
                        if (peerId.isNotBlank() && publicKey.isNotBlank()) {
                            onAddAndChat(
                                peerId.trim(),
                                publicKey.trim(),
                                nickname.ifBlank { null },
                                libp2pPeerId,
                                listeners
                            )
                        }
                    },
                    enabled = peerId.isNotBlank() && publicKey.isNotBlank()
                ) {
                    Text(stringResource(R.string.contacts_action_chat))
                }
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text(stringResource(R.string.cancel))
            }
        }
    )
}

private fun formatTimestamp(timestamp: ULong): String {
    val timestampMillis = timestamp.toEpochMillis()
    val date = Date(timestampMillis)
    val now = System.currentTimeMillis()
    val diff = now - timestampMillis

    // These need to be accessed via Context or moved to a composable that takes strings
    // For simplicity in a non-composable function, we'll keep it as is but use string resources where possible if we pass them in
    // But since this is a private helper, let's just make it return the relative time.

    return when {
        diff < 60_000 -> "Just now"
        diff < 3600_000 -> "${diff / 60_000}m ago"
        diff < 86400_000 -> "${diff / 3600_000}h ago"
        else -> SimpleDateFormat("MMM d, yyyy", Locale.getDefault()).format(date)
    }
}
