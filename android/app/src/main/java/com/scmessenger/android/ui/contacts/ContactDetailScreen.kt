package com.scmessenger.android.ui.contacts

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.filled.Send
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.scmessenger.android.R
import com.scmessenger.android.ui.components.CopyableText
import com.scmessenger.android.ui.components.LabeledCopyableText
import com.scmessenger.android.ui.components.TruncatedCopyableText
import com.scmessenger.android.ui.components.IdenticonFromHex
import com.scmessenger.android.ui.components.ErrorBanner
import com.scmessenger.android.ui.components.IdenticonFromPeerId
import com.scmessenger.android.ui.theme.StatusOnline
import com.scmessenger.android.ui.theme.StatusOffline
import com.scmessenger.android.ui.viewmodels.ContactsViewModel
import com.scmessenger.android.utils.displayName
import com.scmessenger.android.utils.displayNames
import com.scmessenger.android.utils.formatAsDateTime

/**
 * Contact Detail screen - Display contact info, metrics, and actions.
 *
 * Shows detailed information about a contact including:
 * - Identity information (peer ID, public key)
 * - Connection metrics (last seen, message count)
 * - Actions (send message, edit, delete)
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ContactDetailScreen(
    contactId: String,
    onNavigateBack: () -> Unit,
    onNavigateToChat: (String) -> Unit = {},
    onNavigateToVerify: (String) -> Unit = {},
    viewModel: ContactsViewModel = hiltViewModel()
) {
    val contacts by viewModel.contacts.collectAsState()
    val error by viewModel.error.collectAsState()

    val contact = remember(contacts, contactId) {
        contacts.find { it.peerId == contactId }
    }

    var showDeleteDialog by remember { mutableStateOf(false) }
    var showEditDialog by remember { mutableStateOf(false) }

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        contact?.displayName(stringResource(R.string.contact_detail_title))
                            ?: stringResource(R.string.contact_detail_title)
                    )
                },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = stringResource(R.string.chat_action_dismiss))
                    }
                },
                actions = {
                    IconButton(onClick = { onNavigateToVerify(contactId) }) {
                        Icon(Icons.Default.VerifiedUser, contentDescription = stringResource(R.string.contact_detail_action_verify_safety_number))
                    }
                    IconButton(onClick = { showEditDialog = true }) {
                        Icon(Icons.Default.Edit, contentDescription = stringResource(R.string.action_edit))
                    }
                    IconButton(onClick = { showDeleteDialog = true }) {
                        Icon(Icons.Default.Delete, contentDescription = stringResource(R.string.delete))
                    }
                }
            )
        }
    ) { paddingValues ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
        ) {
            if (contact == null) {
                // Contact not found
                Column(
                    modifier = Modifier
                        .align(Alignment.Center)
                        .padding(32.dp),
                    horizontalAlignment = Alignment.CenterHorizontally
                ) {
                    Icon(
                        imageVector = Icons.Default.Close,
                        contentDescription = null,
                        modifier = Modifier.size(64.dp),
                        tint = MaterialTheme.colorScheme.error
                    )

                    Spacer(modifier = Modifier.height(16.dp))

                    Text(
                        text = stringResource(R.string.contact_detail_not_found),
                        style = MaterialTheme.typography.titleLarge
                    )
                }
            } else {
                // Show contact details
                ContactDetailContent(
                    contact = contact,
                    error = error,
                    onClearError = { viewModel.clearError() },
                    onSendMessage = { onNavigateToChat(contact.peerId) }
                )
            }
        }
    }

    // Delete confirmation dialog
    if (showDeleteDialog) {
        AlertDialog(
            onDismissRequest = { showDeleteDialog = false },
            title = { Text(stringResource(R.string.contact_detail_action_delete)) },
            text = { Text(stringResource(R.string.contact_detail_delete_description)) },
            confirmButton = {
                TextButton(
                    onClick = {
                        viewModel.removeContact(contactId)
                        showDeleteDialog = false
                        onNavigateBack()
                    }
                ) {
                    Text(stringResource(R.string.delete))
                }
            },
            dismissButton = {
                TextButton(onClick = { showDeleteDialog = false }) {
                    Text(stringResource(R.string.cancel))
                }
            }
        )
    }

    // Edit nickname dialog
    if (showEditDialog && contact != null) {
        var newNickname by remember { mutableStateOf(contact.localNickname ?: "") }
        val focusRequester = remember { FocusRequester() }

        AlertDialog(
            onDismissRequest = { showEditDialog = false },
            title = { Text(stringResource(R.string.contact_detail_action_edit_local_nickname)) },
            text = {
                OutlinedTextField(
                    value = newNickname,
                    onValueChange = { newNickname = it },
                    label = { Text(stringResource(R.string.contact_detail_label_local_nickname)) },
                    singleLine = true,
                    modifier = Modifier
                        .fillMaxWidth()
                        .focusRequester(focusRequester)
                )
            },
            confirmButton = {
                TextButton(
                    onClick = {
                        viewModel.setLocalNickname(
                            contactId,
                            newNickname.takeIf { it.isNotBlank() }
                        )
                        showEditDialog = false
                    }
                ) {
                    Text(stringResource(R.string.action_save))
                }
            },
            dismissButton = {
                TextButton(onClick = { showEditDialog = false }) {
                    Text(stringResource(R.string.cancel))
                }
            }
        )
        
        // Request focus on dialog open
        LaunchedEffect(Unit) {
            focusRequester.requestFocus()
        }
    }
}

@Composable
private fun ContactDetailContent(
    contact: uniffi.api.Contact,
    error: String?,
    onClearError: () -> Unit,
    onSendMessage: () -> Unit
) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        // Error banner
        error?.let {
            ErrorBanner(
                message = it,
                onDismiss = onClearError
            )
        }

        // Dual-nickname resolution shared by the identity and metadata cards.
        val names = contact.displayNames()

        // Identity card
        Card {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(24.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                IdenticonFromPeerId(
                    peerId = contact.peerId,
                    size = 96.dp
                )

                val unknownFallback = stringResource(R.string.unknown_contact)
                Text(
                    text = names.primary ?: names.secondary ?: unknownFallback,
                    style = MaterialTheme.typography.headlineMedium,
                    fontWeight = FontWeight.Bold
                )
                if (names.primary != null && names.secondary != null) {
                    Text(
                        text = "(${names.secondary})",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }

                // Online status
                Row(
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Icon(
                        imageVector = Icons.Default.CheckCircle,
                        contentDescription = null,
                        modifier = Modifier.size(16.dp),
                        tint = if (contact.lastSeen != null) StatusOnline else StatusOffline
                    )
                    Text(
                        text = if (contact.lastSeen != null) stringResource(R.string.contact_detail_status_last_seen_recent) else stringResource(R.string.contact_detail_status_never_seen),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }

                // Verification status
                Row(
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Icon(
                        imageVector = Icons.Default.VerifiedUser,
                        contentDescription = null,
                        modifier = Modifier.size(16.dp),
                        tint = if (contact.verifiedAt != null) StatusOnline else MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Text(
                        text = if (contact.verifiedAt != null) stringResource(R.string.contact_detail_status_verified) else stringResource(R.string.contact_detail_status_unverified),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }

                // Send message button
                Button(
                    onClick = onSendMessage,
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Icon(Icons.AutoMirrored.Filled.Send, contentDescription = null)
                    Spacer(modifier = Modifier.width(8.dp))
                    Text(stringResource(R.string.contact_detail_action_send_message))
                }
            }
        }

        // Peer ID (wired via LabeledCopyableText)
        Card {
            Column(modifier = Modifier.padding(16.dp)) {
                LabeledCopyableText(
                    label = stringResource(R.string.contact_detail_label_peer_id),
                    text = contact.peerId,
                    monospace = true
                )
            }
        }

        // Public Key (wired via LabeledCopyableText + IdenticonFromHex)
        Card {
            Column(modifier = Modifier.padding(16.dp)) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    IdenticonFromHex(
                        hexString = contact.publicKey,
                        size = 40.dp
                    )
                    Column(modifier = Modifier.weight(1f)) {
                        LabeledCopyableText(
                            label = stringResource(R.string.contact_detail_label_public_key),
                            text = contact.publicKey,
                            monospace = true
                        )
                    }
                }
            }
        }

        // Metadata
        Card {
            Column(modifier = Modifier.padding(16.dp)) {
                Text(
                    text = stringResource(R.string.contact_detail_section_metadata),
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold
                )

                Spacer(modifier = Modifier.height(8.dp))

                MetadataRow(label = stringResource(R.string.contact_detail_label_added), value = contact.addedAt.formatAsDateTime())

                contact.lastSeen?.let {
                    MetadataRow(label = stringResource(R.string.contact_detail_label_last_seen), value = it.formatAsDateTime())
                }

                Spacer(modifier = Modifier.height(8.dp))

                // Dual-nickname fields: the user's own name for this contact is
                // editable (top-bar edit); the peer's chosen nickname is read-only.
                MetadataRow(
                    label = stringResource(R.string.contact_detail_label_your_name),
                    value = names.primary ?: stringResource(R.string.contact_detail_not_set)
                )
                MetadataRow(
                    label = stringResource(R.string.contact_detail_label_chosen_nickname),
                    value = names.secondary ?: stringResource(R.string.contact_detail_not_set)
                )

                contact.notes?.let {
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        text = stringResource(R.string.contact_detail_label_notes),
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Text(
                        text = it,
                        style = MaterialTheme.typography.bodyMedium
                    )
                }
            }
        }
    }
}

@Composable
private fun MetadataRow(label: String, value: String) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween
    ) {
        Text(
            text = label,
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
        Text(
            text = value,
            style = MaterialTheme.typography.bodyMedium
        )
    }
}
