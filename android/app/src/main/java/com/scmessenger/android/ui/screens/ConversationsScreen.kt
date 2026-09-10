package com.scmessenger.android.ui.screens

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.SwipeToDismissBox
import androidx.compose.material3.SwipeToDismissBoxValue
import androidx.compose.material3.rememberSwipeToDismissBoxState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.Chat
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.scmessenger.android.ui.chat.DeliveryStateMapper
import com.scmessenger.android.ui.chat.DeliveryStatePresentation
import com.scmessenger.android.ui.chat.DeliveryStateSurface
import com.scmessenger.android.service.MeshEventBus
import androidx.compose.ui.res.stringResource
import com.scmessenger.android.R
import com.scmessenger.android.ui.viewmodels.ConversationsViewModel
import com.scmessenger.android.utils.displayName
import com.scmessenger.android.utils.toEpochMillis
import java.text.SimpleDateFormat
import java.util.*

/**
 * Conversations/Chat list screen.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ConversationsScreen(
    onNavigateToChat: (String) -> Unit,
    viewModel: ConversationsViewModel = hiltViewModel()
) {
    val conversations by viewModel.conversations.collectAsState()
    val isLoading by viewModel.isLoading.collectAsState()
    val error by viewModel.error.collectAsState()
    val stats by viewModel.stats.collectAsState()
    var conversationToDelete by remember { mutableStateOf<Pair<String, List<uniffi.api.MessageRecord>>?>(null) }
    var showDeleteDialog by remember { mutableStateOf(false) }
    val peerEventRefreshTick by MeshEventBus.peerEvents.collectAsState(initial = null)

    // Keep compose aware of peer identity updates so display names refresh
    // even when message content is unchanged.
    @Suppress("UNUSED_VARIABLE")
    val _refresh = peerEventRefreshTick

    var showClearHistoryDialog by remember { mutableStateOf(false) }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(stringResource(R.string.conversations_title)) },
                actions = {
                    IconButton(onClick = { showClearHistoryDialog = true }) {
                        Icon(
                            imageVector = Icons.Default.Delete,
                            contentDescription = stringResource(R.string.conversations_action_clear_all)
                        )
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
            // Stats summary
            stats?.let { historyStats ->
                Card(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(16.dp)
                ) {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(16.dp),
                        horizontalArrangement = Arrangement.SpaceEvenly
                    ) {
                        StatItem(stringResource(R.string.total), historyStats.totalMessages.toString())
                        StatItem(stringResource(R.string.sent), historyStats.sentCount.toString())
                        StatItem(stringResource(R.string.received), historyStats.receivedCount.toString())
                        StatItem(stringResource(R.string.delivered), (historyStats.sentCount - historyStats.undeliveredCount).toString())
                    }
                }
            }

            // Error display
            error?.let { errorMsg ->
                Snackbar(
                    modifier = Modifier.padding(16.dp),
                    action = {
                        TextButton(onClick = { viewModel.clearError() }) {
                            Text(stringResource(R.string.dismiss))
                        }
                    }
                ) {
                    Text(errorMsg)
                }
            }

            // Loading or conversation list
            if (isLoading) {
                Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.Center
                ) {
                    CircularProgressIndicator()
                }
            } else if (conversations.isEmpty()) {
                // Empty state
                Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.Center
                ) {
                    Column(
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        Icon(
                            imageVector = Icons.AutoMirrored.Filled.Chat,
                            contentDescription = null,
                            modifier = Modifier.size(64.dp),
                            tint = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Spacer(modifier = Modifier.height(16.dp))
                        Text(
                            text = stringResource(R.string.conversations_empty_state),
                            style = MaterialTheme.typography.bodyLarge,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Spacer(modifier = Modifier.height(8.dp))
                        Text(
                            text = stringResource(R.string.conversations_empty_state_description),
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
            } else {
                // FIX(Compose-crash): LazyColumn prefetch crash (LayoutNode.onChildRemoved NPE,
                // MutableVector.add ArrayIndexOutOfBounds) — add stable key + contentType and
                // avoid recomputing FFI contact lookup during composition without remember.
                // Conversations list is grouped by peerId (stable identity) so key is peerId;
                // contentType isolates ConversationItem from header/footer recomposition.
                // Deduplicate to prevent duplicate-key MutableVector corruption.
                val stableConversations = remember(conversations) {
                    conversations.filter { (peerId, msgs) -> peerId.isNotBlank() && msgs.isNotEmpty() }
                        .distinctBy { (peerId, _) -> peerId }
                }
                // FIX: Column+verticalScroll to avoid LazyColumn SlotTable crash on destroy
                // (SlotTableKt.dataAnchor ArrayIndexOutOfBounds via PrefetchHandleProvider/
                // LayoutNodeSubcompositionsState.onRelease during MainActivity destroy at 19:42).
                // Previous LazyColumn prefetch raced with SlotTable disposal on rapid conversation
                // updates (messageEvents, receipt events). Column eliminates prefetch entirely.
                Column(
                    modifier = Modifier
                        .fillMaxSize()
                        .verticalScroll(rememberScrollState())
                        .padding(horizontal = 16.dp, vertical = 8.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    stableConversations.forEach { (peerId, messages) ->
                        // Memoize FFI-backed lookups — previously getContactForPeer() + resolveDeliveryState()
                        // were called unconditionally on every recomposition inside LazyColumn's item lambda,
                        // triggering FFI on the UI thread during prefetch and amplifying layout thrash.
                        // Use key(peerId) to ensure stable identity for SwipeToDismissBox state; without
                        // key, rememberSwipeToDismissBoxState could be reused across peerIds during
                        // recomposition, corrupting SlotTable on dispose.
                        key(peerId) {
                            val contact = remember(peerId) { viewModel.getContactForPeer(peerId) }
                            val idFallback = peerId.take(8) + "..."
                            val displayName = contact?.displayName(idFallback) ?: idFallback
                            val deliveryState = remember(messages.firstOrNull()?.id) {
                                messages.firstOrNull()?.let { viewModel.resolveDeliveryState(it) }
                                    ?: DeliveryStatePresentation(
                                        state = DeliveryStateSurface.PENDING,
                                        label = DeliveryStateSurface.PENDING.label,
                                        detail = DeliveryStateSurface.PENDING.detail
                                    )
                            }
                            if (messages.isNotEmpty()) {
                                ConversationItem(
                                    displayName = displayName,
                                    peerId = peerId,
                                    messages = messages,
                                    onNavigateToChat = onNavigateToChat,
                                    onRequestDelete = {
                                        conversationToDelete = peerId to messages
                                        showDeleteDialog = true
                                    },
                                    deliveryState = deliveryState
                                )
                            }
                        }
                    }
                }
            }
        }
    }

    if (showClearHistoryDialog) {
        AlertDialog(
            onDismissRequest = { showClearHistoryDialog = false },
            title = { Text(stringResource(R.string.conversations_dialog_clear_all_title)) },
            text = { Text(stringResource(R.string.conversations_dialog_clear_all_description)) },
            confirmButton = {
                TextButton(
                    onClick = {
                        viewModel.clearAllHistory()
                        showClearHistoryDialog = false
                    },
                    colors = ButtonDefaults.textButtonColors(contentColor = MaterialTheme.colorScheme.error)
                ) {
                    Text(stringResource(R.string.conversations_action_clear_all))
                }
            },
            dismissButton = {
                TextButton(onClick = { showClearHistoryDialog = false }) {
                    Text(stringResource(R.string.cancel))
                }
            }
        )
    }

    if (showDeleteDialog) {
        AlertDialog(
            onDismissRequest = { showDeleteDialog = false },
            title = { Text(stringResource(R.string.conversations_dialog_delete_title)) },
            text = {
                val (peerId, _) = conversationToDelete ?: return@AlertDialog
                val contact = viewModel.getContactForPeer(peerId)
                val idFallback = "${peerId.take(8)}..."
                val displayName = contact?.displayName(idFallback) ?: idFallback
                Text(stringResource(R.string.conversations_dialog_delete_description, displayName))
            },
            confirmButton = {
                TextButton(
                    onClick = {
                        val targetPeerId = conversationToDelete?.first
                        if (!targetPeerId.isNullOrBlank()) {
                            viewModel.clearConversation(targetPeerId)
                        }
                        showDeleteDialog = false
                    },
                ) {
                    Text(stringResource(R.string.delete), color = MaterialTheme.colorScheme.error)
                }
            },
            dismissButton = {
                TextButton(onClick = { showDeleteDialog = false }) {
                    Text(stringResource(R.string.cancel))
                }
            },
        )
    }
}

@Composable
fun StatItem(label: String, value: String) {
    Column(
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Text(
            text = value,
            style = MaterialTheme.typography.titleLarge,
            color = MaterialTheme.colorScheme.primary
        )
        Text(
            text = label,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
    }
}

@Composable
fun ConversationItem(
    displayName: String,
    peerId: String,
    messages: List<uniffi.api.MessageRecord>,
    onNavigateToChat: (String) -> Unit,
    onRequestDelete: () -> Unit,
    deliveryState: DeliveryStatePresentation,
) {
    val lastMessage = messages.firstOrNull() ?: return
    val undeliveredCount = messages.count { !it.delivered }
    val dismissState = rememberSwipeToDismissBoxState(
        confirmValueChange = { value ->
            if (value == SwipeToDismissBoxValue.EndToStart) {
                onRequestDelete()
            }
            false
        },
    )

    SwipeToDismissBox(
        state = dismissState,
        enableDismissFromEndToStart = true,
        enableDismissFromStartToEnd = false,
        backgroundContent = {
            Box(
                modifier = Modifier
                    .fillMaxSize(),
                contentAlignment = Alignment.CenterEnd,
            ) {
                Card(
                    modifier = Modifier.fillMaxSize(),
                    colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.error),
                ) {
                    Box(
                        modifier = Modifier
                            .fillMaxSize()
                            .padding(horizontal = 20.dp),
                        contentAlignment = Alignment.CenterEnd,
                    ) {
                        Icon(
                            imageVector = Icons.Default.Delete,
                            contentDescription = stringResource(R.string.conversations_content_desc_delete),
                            tint = Color.White,
                        )
                    }
                }
            }
        },
        content = {
        Card(
            modifier = Modifier
                .fillMaxWidth()
                .clickable(onClick = { onNavigateToChat(peerId) })
        ) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(16.dp),
                horizontalArrangement = Arrangement.SpaceBetween
            ) {
                Column(
                    modifier = Modifier.weight(1f)
                ) {
                    Row(
                        horizontalArrangement = Arrangement.SpaceBetween,
                        modifier = Modifier.fillMaxWidth()
                    ) {
                        Text(
                            text = displayName,
                            style = MaterialTheme.typography.titleMedium,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                            modifier = Modifier.weight(1f)
                        )
                        Text(
                            text = formatTimestamp(lastMessage.timestamp),
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }

                    Spacer(modifier = Modifier.height(4.dp))

                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween
                    ) {
                        Text(
                            text = when (lastMessage.direction) {
                                uniffi.api.MessageDirection.SENT -> "You: ${lastMessage.content}"
                                uniffi.api.MessageDirection.RECEIVED -> lastMessage.content
                            },
                            style = MaterialTheme.typography.bodyMedium,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                            modifier = Modifier.weight(1f),
                            color = if (undeliveredCount > 0 && lastMessage.direction == uniffi.api.MessageDirection.SENT) {
                                MaterialTheme.colorScheme.onSurfaceVariant
                            } else {
                                MaterialTheme.colorScheme.onSurface
                            }
                        )

                        if (undeliveredCount > 0 && lastMessage.direction == uniffi.api.MessageDirection.SENT) {
                            Badge {
                                Text(undeliveredCount.toString())
                            }
                        }
                    }

                    if (lastMessage.direction == uniffi.api.MessageDirection.SENT) {
                        Spacer(modifier = Modifier.height(2.dp))
                        Text(
                            text = "Status: ${deliveryState.label}",
                            style = MaterialTheme.typography.bodySmall,
                            color = when (deliveryState.state) {
                                DeliveryStateSurface.DELIVERED -> MaterialTheme.colorScheme.primary
                                DeliveryStateSurface.REJECTED -> MaterialTheme.colorScheme.error
                                else -> MaterialTheme.colorScheme.onSurfaceVariant
                            }
                        )
                    }

                    Text(
                        text = "${messages.size} messages • ${peerId.take(12)}",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }
        }
        },
    )
}

private fun formatTimestamp(timestamp: ULong): String {
    val timestampMillis = timestamp.toEpochMillis()
    val date = Date(timestampMillis)
    val now = System.currentTimeMillis()
    val diff = now - timestampMillis

    return when {
        diff < 60_000 -> "Now"
        diff < 3600_000 -> "${diff / 60_000}m"
        diff < 86400_000 -> "${diff / 3600_000}h"
        else -> SimpleDateFormat("MMM d", Locale.getDefault()).format(date)
    }
}
