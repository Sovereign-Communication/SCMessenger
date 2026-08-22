package com.scmessenger.android.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Block
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.automirrored.filled.Send
import androidx.compose.material.icons.outlined.ChatBubbleOutline
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.compose.ui.res.stringResource
import uniffi.api.*
import com.scmessenger.android.R
import com.scmessenger.android.utils.toEpochMillis
import com.scmessenger.android.ui.viewmodels.ConversationsViewModel
import com.scmessenger.android.ui.viewmodels.ContactsViewModel
import com.scmessenger.android.ui.viewmodels.ChatViewModel
import com.scmessenger.android.ui.chat.MessageInput
import com.scmessenger.android.service.MeshEventBus
import com.scmessenger.android.service.UiEvent
import kotlinx.coroutines.launch
import timber.log.Timber
import java.text.SimpleDateFormat
import java.util.*

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ChatScreen(
    conversationId: String, // Treat as PeerID
    onNavigateBack: () -> Unit,
    viewModel: ConversationsViewModel = hiltViewModel(),
    chatViewModel: ChatViewModel = hiltViewModel()
) {
    // Wire setPeer into navigation to chat
    LaunchedEffect(conversationId) {
        chatViewModel.setPeer(conversationId)
        // Wire loadConversation into conversation tap/open flow
        viewModel.loadConversation(conversationId).collect { messages ->
            // Messages loaded into the shared state by loadMessages
        }
    }

    LaunchedEffect(conversationId) {
        MeshEventBus.emitUiEvent(UiEvent.ConversationOpened(conversationId))
    }

    DisposableEffect(conversationId) {
        onDispose {
            // GlobalScope is used here because the composable's scope is already cancelled
            @Suppress("OPT_IN_USAGE")
            kotlinx.coroutines.GlobalScope.launch {
                MeshEventBus.emitUiEvent(UiEvent.ConversationClosed)
            }
        }
    }


    val messages by viewModel.messages.collectAsState()
    val error by viewModel.error.collectAsState()
    val isLoading by viewModel.isLoading.collectAsState()
    val blockedPeers by viewModel.blockedPeers.collectAsState()
    val isBlocked = remember(blockedPeers, conversationId) {
        blockedPeers.any { it.peerId == conversationId }
    }
    val chatMessages = remember(messages, conversationId) {
        // MSG-ORDER-001: Sort strictly by sender-assigned timestamp to ensure consistent ordering across platforms
        messages.filter { viewModel.isSamePeer(it.peerId, conversationId) }.sortedBy { it.senderTimestamp }
    }

    // Wire updateInputText/clearInput into ChatViewModel
    val inputText by chatViewModel.inputText.collectAsState()
    val listState = rememberLazyListState()
    val coroutineScope = rememberCoroutineScope()

    // Wire loadMoreMessages into scroll-to-load-more
    val shouldLoadMore by remember {
        derivedStateOf {
            val lastVisible = listState.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: 0
            lastVisible <= 3 && chatMessages.size > 10
        }
    }
    LaunchedEffect(shouldLoadMore) {
        if (shouldLoadMore) {
            chatViewModel.loadMoreMessages()
        }
    }

    // AND-SEND-BTN-001: Use remember with derivedStateOf to avoid calling getContactForPeer
    // and isPeerAvailable on every recomposition. The underlying canonicalContactId() now
    // uses an in-memory cache (identityIdCache), so the first call resolves via FFI and
    // subsequent calls hit the cache. Combined with remember keys, this prevents the
    // UI-thread-blocking FFI calls that froze the send button.
    val normalizedPeerId = com.scmessenger.android.utils.PeerIdValidator.normalize(conversationId)

    val contact = remember(normalizedPeerId) {
        viewModel.getContactForPeer(normalizedPeerId)
    }
    val isPeerAvailable = remember(normalizedPeerId) {
        viewModel.isPeerAvailable(normalizedPeerId)
    }

    val localNickname = contact?.localNickname?.trim().orEmpty()
    val federatedNickname = contact?.nickname?.trim().orEmpty()
    val displayName = when {
        localNickname.isNotEmpty() -> localNickname
        federatedNickname.isNotEmpty() -> federatedNickname
        else -> conversationId.take(12) + "..."
    }

    Timber.d("CHAT_SCREEN: conversationId=$conversationId, normalizedPeerId=$normalizedPeerId, displayName=$displayName, localNick=$localNickname, fedNick=$federatedNickname, contactFound=${contact != null}, isBlocked=$isBlocked")
    var showAddContactDialog by remember { mutableStateOf(false) }
    var showBlockConfirmation by remember { mutableStateOf(false) }

    LaunchedEffect(conversationId) {
        viewModel.loadMessages(limit = 200u)
    }

    LaunchedEffect(chatMessages.size) {
        if (chatMessages.isNotEmpty()) {
            listState.animateScrollToItem(chatMessages.size - 1)
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Text(displayName)
                },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = stringResource(R.string.chat_content_desc_back))
                    }
                },
                actions = {
                    // Block/Unblock button
                    IconButton(
                        onClick = {
                            if (isBlocked) {
                                viewModel.unblockPeer(conversationId)
                            } else {
                                showBlockConfirmation = true
                            }
                        }
                    ) {
                        Icon(
                            imageVector = Icons.Default.Block,
                            contentDescription = if (isBlocked) "Unblock" else "Block",
                            tint = if (isBlocked) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onSurface
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
            error?.let { errorMsg ->
                Snackbar(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 16.dp, vertical = 8.dp),
                    action = {
                        TextButton(onClick = { viewModel.clearError() }) {
                            Text(stringResource(R.string.chat_action_dismiss))
                        }
                    }
                ) {
                    Text(errorMsg)
                }
            }

            // Show banner if peer is not in contacts
            if (contact == null && isPeerAvailable) {
                Card(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 16.dp, vertical = 8.dp),
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.secondaryContainer
                    )
                ) {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(12.dp),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Column(modifier = Modifier.weight(1f)) {
                            Text(
                                text = stringResource(R.string.chat_not_in_contacts),
                                style = MaterialTheme.typography.titleSmall,
                                color = MaterialTheme.colorScheme.onSecondaryContainer
                            )
                            Text(
                                text = stringResource(R.string.chat_add_to_send_messages),
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onSecondaryContainer
                            )
                        }
                        Button(onClick = { showAddContactDialog = true }) {
                            Text(stringResource(R.string.chat_action_add_contact))
                        }
                    }
                }
            }

            // Messages List
            when {
                chatMessages.isEmpty() && isLoading -> {
                    Box(
                        modifier = Modifier
                            .weight(1f)
                            .fillMaxWidth(),
                        contentAlignment = Alignment.Center
                    ) {
                        CircularProgressIndicator()
                    }
                }
                chatMessages.isEmpty() && !isLoading -> {
                    Box(
                        modifier = Modifier
                            .weight(1f)
                            .fillMaxWidth(),
                        contentAlignment = Alignment.Center
                    ) {
                        Column(horizontalAlignment = Alignment.CenterHorizontally) {
                            Icon(
                                imageVector = Icons.Outlined.ChatBubbleOutline,
                                contentDescription = null,
                                modifier = Modifier.size(64.dp),
                                tint = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                            Text(
                                "No messages yet",
                                style = MaterialTheme.typography.titleMedium
                            )
                            Text(
                                "Send a message to start the conversation",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    }
                }
                else -> {
                    LazyColumn(
                        state = listState,
                        modifier = Modifier
                            .weight(1f)
                            .fillMaxWidth()
                            .padding(horizontal = 16.dp),
                        contentPadding = PaddingValues(vertical = 16.dp),
                        verticalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        items(chatMessages) { message ->
                            MessageBubble(
                                message = message,
                                isMe = message.direction == uniffi.api.MessageDirection.SENT
                            )
                        }
                    }
                }
            }

            // Input Area
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .imePadding()
                    .padding(16.dp),
                contentAlignment = Alignment.Center
            ) {
                if (isBlocked) {
                    Card(
                        modifier = Modifier.fillMaxWidth(),
                        colors = CardDefaults.cardColors(
                            containerColor = MaterialTheme.colorScheme.errorContainer.copy(alpha = 0.5f)
                        )
                    ) {
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(12.dp),
                            horizontalArrangement = Arrangement.Center,
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Icon(Icons.Default.Block, contentDescription = stringResource(R.string.chat_content_desc_blocked), tint = MaterialTheme.colorScheme.onErrorContainer)
                            Spacer(modifier = Modifier.width(8.dp))
                            Text(
                                "Peer blocked. Unblock to send messages.",
                                color = MaterialTheme.colorScheme.onErrorContainer,
                                style = MaterialTheme.typography.bodyMedium
                            )
                        }
                    }
                } else {
                    MessageInput(
                        value = inputText,
                        onValueChange = { chatViewModel.updateInputText(it) },
                        onSend = {
                            if (isBlocked) {
                                Timber.w("SEND_BUTTON_CLICKED: Peer is blocked, ignoring click")
                                return@MessageInput
                            }

                            val messageToSend = inputText.trim()
                            if (messageToSend.isEmpty()) {
                                Timber.w("SEND: Attempted to send empty message")
                                return@MessageInput
                            }

                            Timber.d("SEND: Processing message send, contentLength=${messageToSend.length}")

                            // Wire clearInput into post-send callback
                            val originalInput = inputText
                            chatViewModel.clearInput()
                            Timber.d("SEND: Input cleared immediately for instant feedback")

                            coroutineScope.launch {
                                try {
                                    Timber.d("SEND: Launching sendMessage coroutine")
                                    val success = viewModel.sendMessage(conversationId, messageToSend)
                                    Timber.d("SEND: Message sent, success=$success")
                                    if (success) {
                                        // Scroll to bottom to show new message
                                        if (chatMessages.isNotEmpty()) {
                                            listState.animateScrollToItem(chatMessages.size)
                                        }
                                    }
                                } catch (e: Exception) {
                                    Timber.e(e, "SEND: Failed to send message")
                                    // Restore input if send failed
                                    chatViewModel.updateInputText(originalInput)
                                }
                            }
                        },
                        enabled = !isBlocked,
                        modifier = Modifier.fillMaxWidth()
                    )
                }
            }
        }
    }

    // Quick add contact dialog
    if (showAddContactDialog) {
        val peerInfo = viewModel.getPeerInfo(conversationId)
        if (peerInfo != null) {
            val (publicKey, suggestedNickname) = peerInfo
            var nickname by remember { mutableStateOf(suggestedNickname) }
            val contactsViewModel: ContactsViewModel = hiltViewModel()

            AlertDialog(
                onDismissRequest = { showAddContactDialog = false },
                title = { Text(stringResource(R.string.chat_action_add_contact)) },
                text = {
                    Column {
                        Text(stringResource(R.string.chat_add_contact_description))
                        Spacer(modifier = Modifier.height(8.dp))
                        OutlinedTextField(
                            value = nickname,
                            onValueChange = { nickname = it },
                            label = { Text(stringResource(R.string.settings_label_nickname)) },
                            singleLine = true,
                            modifier = Modifier.fillMaxWidth()
                        )
                    }
                },
                confirmButton = {
                    TextButton(
                        onClick = {
                            coroutineScope.launch {
                                try {
                                    contactsViewModel.addContact(
                                        peerId = conversationId,
                                        publicKey = publicKey,
                                        nickname = nickname
                                    )
                                    showAddContactDialog = false
                                    Timber.i("Contact added successfully via ContactsViewModel")
                                } catch (e: Exception) {
                                    Timber.e(e, "Failed to add contact")
                                }
                            }
                        }
                    ) {
                        Text(stringResource(R.string.chat_action_add_contact))
                    }
                },
                dismissButton = {
                    TextButton(onClick = { showAddContactDialog = false }) {
                        Text(stringResource(R.string.cancel))
                    }
                }
            )
        } else {
            // Peer not available - close dialog
            showAddContactDialog = false
        }
    }

    // Block confirmation dialog
    if (showBlockConfirmation) {
        AlertDialog(
            onDismissRequest = { showBlockConfirmation = false },
            title = { Text(stringResource(R.string.chat_dialog_block_title)) },
            text = { Text(stringResource(R.string.chat_dialog_block_description)) },
            confirmButton = {
                TextButton(
                    onClick = {
                        viewModel.blockPeer(conversationId, "Blocked from chat")
                        showBlockConfirmation = false
                    },
                    colors = ButtonDefaults.textButtonColors(contentColor = MaterialTheme.colorScheme.error)
                ) {
                    Text(stringResource(R.string.action_block))
                }
            },
            dismissButton = {
                TextButton(onClick = { showBlockConfirmation = false }) {
                    Text(stringResource(R.string.cancel))
                }
            }
        )
    }
}


@Composable
fun MessageBubble(
    message: uniffi.api.MessageRecord,
    isMe: Boolean
) {
    val bubbleColor = if (isMe) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.surfaceVariant
    val textColor = if (isMe) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.onSurfaceVariant
    val alignment = if (isMe) Alignment.End else Alignment.Start
    val shape = if (isMe) RoundedCornerShape(16.dp, 16.dp, 4.dp, 16.dp) else RoundedCornerShape(16.dp, 16.dp, 16.dp, 4.dp)

    Column(
        modifier = Modifier.fillMaxWidth(),
        horizontalAlignment = alignment
    ) {
        Box(
            modifier = Modifier
                .clip(shape)
                .background(bubbleColor)
                .padding(horizontal = 16.dp, vertical = 10.dp)
        ) {
            Text(
                text = message.content,
                color = textColor,
                style = MaterialTheme.typography.bodyLarge
            )
        }
        // Zero-Status Architecture: show only sender-assigned timestamp, no delivery state.
        Text(
            text = formatTimestamp(message.senderTimestamp),
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.outline,
            modifier = Modifier.padding(top = 4.dp, start = 4.dp, end = 4.dp)
        )
    }
}

private fun formatTimestamp(timestamp: ULong): String {
    val date = Date(timestamp.toEpochMillis())
    val sdf = SimpleDateFormat("HH:mm", Locale.getDefault())
    return sdf.format(date)
}
