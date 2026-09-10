package com.scmessenger.android.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Block
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.scmessenger.android.R
import com.scmessenger.android.ui.components.IdenticonFromPeerId
import com.scmessenger.android.ui.viewmodels.RequestsInboxViewModel
import com.scmessenger.android.ui.viewmodels.RequestItem
import com.scmessenger.android.utils.toEpochMillis
import java.text.SimpleDateFormat
import java.util.*

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun RequestsInboxScreen(
    onNavigateBack: () -> Unit,
    onNavigateToChat: ((peerId: String) -> Unit)? = null,
    viewModel: RequestsInboxViewModel = hiltViewModel()
) {
    val requests by viewModel.requests.collectAsState()
    val isLoading by viewModel.isLoading.collectAsState()
    val error by viewModel.error.collectAsState()

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(stringResource(R.string.requests_inbox_title)) },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = stringResource(R.string.requests_inbox_content_desc_back))
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
            // Show error banner if error state is set
            if (error != null) {
                Spacer(modifier = Modifier.height(8.dp))
            }

            if (isLoading) {
                Box(
                    modifier = Modifier.weight(1f),
                    contentAlignment = Alignment.Center
                ) {
                    CircularProgressIndicator()
                }
            } else if (requests.isEmpty()) {
                EmptyState(paddingValues)
            } else {
                RequestList(
                    requests = requests,
                    onAccept = { viewModel.acceptRequest(it) },
                    onReject = { viewModel.rejectRequest(it) },
                    onBlockAndDelete = { viewModel.blockAndDelete(it) },
                    onNavigateToChat = onNavigateToChat,
                    modifier = Modifier.weight(1f)
                )
            }
        }
    }
}

@Composable
private fun RequestList(
    requests: List<RequestItem>,
    onAccept: (peerId: String) -> Unit,
    onReject: (peerId: String) -> Unit,
    onBlockAndDelete: (peerId: String) -> Unit,
    onNavigateToChat: ((peerId: String) -> Unit)?,
    modifier: Modifier = Modifier
) {
    // FIX(Compose-crash): Column+verticalScroll to eliminate LazyColumn prefetch crash
    // (SlotTableKt.dataAnchor ArrayIndexOutOfBounds via PrefetchHandleProvider/
    // LayoutNodeSubcompositionsState.onRelease during MainActivity destroy at 19:42).
    // Requests list is small (typically <20) so Column overhead is negligible.
    val stableRequests = remember(requests) {
        requests.filter { it.peerId.isNotBlank() }.distinctBy { it.peerId }
    }
    Column(
        modifier = modifier
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        stableRequests.forEach { request ->
            key(request.peerId) {
                RequestItem(
                    request = request,
                    onAccept = onAccept,
                    onReject = onReject,
                    onBlockAndDelete = onBlockAndDelete,
                    onNavigateToChat = onNavigateToChat
                )
            }
        }
    }
}

@Composable
private fun RequestItem(
    request: RequestItem,
    onAccept: (peerId: String) -> Unit,
    onReject: (peerId: String) -> Unit,
    onBlockAndDelete: (peerId: String) -> Unit,
    @Suppress("UNUSED_PARAMETER") onNavigateToChat: ((peerId: String) -> Unit)?,
    modifier: Modifier = Modifier
) {
    Card(
        modifier = modifier.fillMaxWidth()
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Row(
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    IdenticonFromPeerId(
                        peerId = request.peerId,
                        modifier = Modifier.size(48.dp)
                    )

                    Column(modifier = Modifier.weight(1f)) {
                        Text(
                            text = request.nickname?.takeIf { it.isNotBlank() }
                                ?: request.peerId.take(12) + "...",
                            style = MaterialTheme.typography.titleSmall,
                            fontWeight = FontWeight.Bold
                        )

                        Text(
                            text = request.messagePreview,
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            maxLines = 1,
                            overflow = androidx.compose.ui.text.style.TextOverflow.Ellipsis
                        )

                        Text(
                            text = formatTimestamp(request.messageTimestamp),
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.outline
                        )
                    }
                }

                // Action buttons - show all for now, can be hidden in the future
                // For now, just show a simplified view with primary actions
            }

            // Action buttons row
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                // Accept button (adds contact + opens chat)
                Button(
                    onClick = { onAccept(request.peerId) },
                    modifier = Modifier.weight(1f),
                    enabled = true
                ) {
                    Icon(Icons.Default.Check, contentDescription = null)
                    Spacer(modifier = Modifier.width(8.dp))
                    Text(stringResource(R.string.action_accept))
                }

                // Reject button (blocks + deletes, keeps from showing in inbox)
                OutlinedButton(
                    onClick = { onReject(request.peerId) },
                    modifier = Modifier.weight(1f),
                    enabled = true
                ) {
                    Icon(Icons.Default.Block, contentDescription = null)
                    Spacer(modifier = Modifier.width(8.dp))
                    Text(stringResource(R.string.action_reject))
                }
            }

            // Block & Delete button (blocks + deletes all messages)
            TextButton(
                onClick = { onBlockAndDelete(request.peerId) },
                modifier = Modifier.fillMaxWidth(),
                enabled = true
            ) {
                Icon(Icons.Default.Delete, contentDescription = null)
                Spacer(modifier = Modifier.width(8.dp))
                Text(stringResource(R.string.action_block_and_delete))
            }
        }
    }
}

@Composable
private fun EmptyState(paddingValues: PaddingValues) {
    Box(
        modifier = Modifier
            .fillMaxSize()
            .padding(paddingValues),
        contentAlignment = Alignment.Center
    ) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center
        ) {
            Icon(
                imageVector = Icons.Default.Check,
                contentDescription = null,
                modifier = Modifier.size(64.dp),
                tint = MaterialTheme.colorScheme.outline
            )
            Spacer(modifier = Modifier.height(16.dp))
            Text(
                text = stringResource(R.string.requests_inbox_empty_title),
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
            Spacer(modifier = Modifier.height(8.dp))
            Text(
                text = stringResource(R.string.requests_inbox_empty_description),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.outline,
                textAlign = androidx.compose.ui.text.style.TextAlign.Center
            )
        }
    }
}

private fun formatTimestamp(timestamp: Long): String {
    val date = Date(timestamp)
    val sdf = SimpleDateFormat("MMM d, yyyy HH:mm", Locale.getDefault())
    return sdf.format(date)
}
