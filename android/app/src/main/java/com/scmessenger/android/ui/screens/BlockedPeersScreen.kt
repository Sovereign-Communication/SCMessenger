package com.scmessenger.android.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Block
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.compose.ui.res.stringResource
import com.scmessenger.android.R
import com.scmessenger.android.ui.viewmodels.ConversationsViewModel
import java.text.SimpleDateFormat
import java.util.*

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun BlockedPeersScreen(
    onNavigateBack: () -> Unit,
    viewModel: ConversationsViewModel = hiltViewModel()
) {
    val blockedPeers by viewModel.blockedPeers.collectAsState()
    var showUnblockConfirm by remember { mutableStateOf<uniffi.api.BlockedIdentity?>(null) }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(stringResource(R.string.blocked_peers_title)) },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = stringResource(R.string.chat_action_dismiss))
                    }
                }
            )
        }
    ) { paddingValues ->
        if (blockedPeers.isEmpty()) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(paddingValues),
                contentAlignment = Alignment.Center
            ) {
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Icon(
                        Icons.Default.Block,
                        contentDescription = null,
                        modifier = Modifier.size(64.dp),
                        tint = MaterialTheme.colorScheme.outline
                    )
                    Spacer(modifier = Modifier.height(16.dp))
                    Text(
                        stringResource(R.string.blocked_peers_empty_state),
                        style = MaterialTheme.typography.bodyLarge,
                        color = MaterialTheme.colorScheme.outline
                    )
                }
            }
        } else {
            // FIX(Compose-crash): Column+verticalScroll to eliminate LazyColumn prefetch crash
            // (SlotTableKt.dataAnchor ArrayIndexOutOfBounds via PrefetchHandleProvider/
            // LayoutNodeSubcompositionsState.onRelease during MainActivity destroy at 19:42).
            // Blocked list is small (<1k) so Column overhead is negligible and avoids
            // LazyLayout prefetch/subcomposition lifecycle race entirely.
            val stableBlocked = remember(blockedPeers) {
                blockedPeers.filter { it.peerId.isNotBlank() }.distinctBy { it.peerId }
            }
            Column(
                modifier = Modifier
                    .fillMaxSize()
                    .verticalScroll(rememberScrollState())
                    .padding(paddingValues)
                    .padding(16.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                stableBlocked.forEach { blocked ->
                    key(blocked.peerId) {
                        BlockedPeerItem(
                            blocked = blocked,
                            onUnblock = { showUnblockConfirm = blocked }
                        )
                    }
                }
            }
        }
    }

    if (showUnblockConfirm != null) {
        val peer = showUnblockConfirm!!
        AlertDialog(
            onDismissRequest = { showUnblockConfirm = null },
            title = { Text(stringResource(R.string.blocked_peers_dialog_unblock_title)) },
            text = { Text(stringResource(R.string.blocked_peers_dialog_unblock_description, peer.peerId)) },
            confirmButton = {
                TextButton(
                    onClick = {
                        viewModel.unblockPeer(peer.peerId)
                        showUnblockConfirm = null
                    }
                ) {
                    Text(stringResource(R.string.blocked_peers_action_unblock))
                }
            },
            dismissButton = {
                TextButton(onClick = { showUnblockConfirm = null }) {
                    Text(stringResource(R.string.cancel))
                }
            }
        )
    }
}

@Composable
fun BlockedPeerItem(
    blocked: uniffi.api.BlockedIdentity,
    onUnblock: () -> Unit
) {
    Card(
        modifier = Modifier.fillMaxWidth()
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = blocked.peerId.take(24) + "...",
                    style = MaterialTheme.typography.titleSmall,
                    fontWeight = FontWeight.Bold
                )
                Text(
                    text = stringResource(R.string.blocked_peers_label_blocked_on, formatDate(blocked.blockedAt)),
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                if (!blocked.reason.isNullOrBlank()) {
                    Text(
                        text = stringResource(R.string.blocked_peers_label_reason, blocked.reason!!),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }
            IconButton(onClick = onUnblock) {
                Icon(
                    Icons.Default.Delete,
                    contentDescription = stringResource(R.string.blocked_peers_action_unblock),
                    tint = MaterialTheme.colorScheme.error
                )
            }
        }
    }
}

private fun formatDate(seconds: ULong): String {
    val date = Date(seconds.toLong() * 1000)
    val sdf = SimpleDateFormat("MMM d, yyyy HH:mm", Locale.getDefault())
    return sdf.format(date)
}
