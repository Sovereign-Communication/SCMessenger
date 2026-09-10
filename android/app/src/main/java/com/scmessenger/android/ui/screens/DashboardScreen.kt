package com.scmessenger.android.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Bluetooth
import androidx.compose.material.icons.filled.Bolt
import androidx.compose.material.icons.filled.NetworkWifi
import androidx.compose.material.icons.filled.People
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.Router
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material.icons.filled.Wifi
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.navigation.NavHostController
import androidx.compose.ui.res.stringResource
import com.scmessenger.android.R
import com.scmessenger.android.ui.dashboard.PeerListScreen
import com.scmessenger.android.ui.dashboard.TopologyScreen
import com.scmessenger.android.utils.toEpochMillis
import com.scmessenger.android.ui.viewmodels.MeshServiceViewModel
import com.scmessenger.android.ui.viewmodels.DashboardViewModel
import com.scmessenger.android.ui.viewmodels.SettingsViewModel
import com.scmessenger.android.ui.settings.MeshSettingsScreen
import com.scmessenger.android.ui.settings.PowerSettingsScreen

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DashboardScreen(
    serviceViewModel: MeshServiceViewModel = hiltViewModel(),
    dashboardViewModel: DashboardViewModel = hiltViewModel(),
    settingsViewModel: SettingsViewModel = hiltViewModel(),
    onNavigateToPeerList: () -> Unit = {},
    onNavigateToTopology: () -> Unit = {},
    onNavigateToJoinMesh: () -> Unit = {},
    onPeerClick: (com.scmessenger.android.ui.viewmodels.PeerInfo) -> Unit = {}
) {
    val serviceState by serviceViewModel.serviceState.collectAsState()
    val isRunning by serviceViewModel.isRunning.collectAsState()
    val isStorageDegraded by serviceViewModel.isStorageDegraded.collectAsState()
    val stats by serviceViewModel.serviceStats.collectAsState()

    val fullPeers by dashboardViewModel.fullPeersCount.collectAsState()
    val headlessPeers by dashboardViewModel.headlessPeersCount.collectAsState()

    val meshSettings by settingsViewModel.settings.collectAsState()
    val nearbyCount by dashboardViewModel.nearbyPeersCount.collectAsState()

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(stringResource(R.string.dashboard_title)) }
            )
        }
    ) { paddingValues ->
        val peers by dashboardViewModel.peers.collectAsState()
        // UNIFICATION_V2 crash guard: ensure distinct keys for Compose stability (outside LazyColumn scope)
        // UNIFICATION FIX: nearbyCount is discovery-based (BLE/TCP/mDNS direct + isRecent) not ledger history — 2 nearby vs 9 total.
        // sortedPeers is unified single-list (online first, offline last) via DashboardViewModel.sortPeersForUnifiedView; total is authoritative.
        val sortedPeers = remember(peers) { peers.filter { it.peerId.isNotBlank() }.distinctBy { it.peerId } }

        // FIX: Use Column+verticalScroll instead of LazyColumn to avoid Compose MutableVector crash on rapid list updates
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
                .verticalScroll(rememberScrollState())
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            StatusCard(
                isRunning = isRunning,
                stateName = serviceState.name,
                isStorageDegraded = isStorageDegraded
            )

            // Quick Stats Grid
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                StatCard(
                    modifier = Modifier.weight(1.5f),
                    title = buildString {
                        append(stringResource(R.string.dashboard_stat_nodes_format, fullPeers))
                        if (headlessPeers > 0) append(stringResource(R.string.dashboard_stat_headless_format, headlessPeers))
                    },
                    // UNIFICATION FIX: nearby accuracy — discovery-based direct transport (BLE/TCP/LAN) + isOnline, not ledger history. Fixes 9 vs 2.
                    value = "$nearbyCount / ${sortedPeers.size}",
                    icon = Icons.Filled.People,
                    color = MaterialTheme.colorScheme.primary
                )
                StatCard(
                    modifier = Modifier.weight(1f),
                    title = stringResource(R.string.dashboard_label_relayed),
                    value = stats?.messagesRelayed?.toString() ?: "0",
                    icon = Icons.Filled.Router,
                    color = MaterialTheme.colorScheme.tertiary
                )
            }

            // Connection Methods Status
            ConnectionStatusCard(
                bleEnabled = meshSettings?.bleEnabled ?: false,
                wifiAwareEnabled = meshSettings?.wifiAwareEnabled ?: false,
                wifiDirectEnabled = meshSettings?.wifiDirectEnabled ?: false,
                internetRelayEnabled = meshSettings?.relayEnabled == true && meshSettings?.internetEnabled == true
            )

            // Detailed Stats
            Card(
                modifier = Modifier.fillMaxWidth(),
                colors = CardDefaults.cardColors(
                    containerColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f)
                )
            ) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Text(
                        text = stringResource(R.string.dashboard_section_performance),
                        style = MaterialTheme.typography.titleMedium,
                        modifier = Modifier.padding(bottom = 8.dp)
                    )

                    TextDetailRow(stringResource(R.string.dashboard_label_uptime), formatDuration(stats?.uptimeSecs ?: 0uL))
                    TextDetailRow(stringResource(R.string.dashboard_label_data_transferred), formatBytes(stats?.bytesTransferred ?: 0uL))
                }
            }

            // Discovered Nodes Header
            Text(
                text = stringResource(R.string.dashboard_section_discovered),
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(top = 8.dp)
            )

            // UNIFICATION_V2: single unified sorted list — classification via badge, not section
            // Clickable: mesh nodes open peer detail / conversation
            if (sortedPeers.isEmpty()) {
                Text(
                    text = stringResource(R.string.dashboard_empty_state_discovered),
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            } else {
                // FIX(Compose-crash): stable key per peerId prevents SlotTable MutableVector corruption
                // on rapid discoveredPeers updates. Dashboard list is small (<50), Column eliminates
                // LazyColumn prefetch race; key() ensures SwipeToDismissBox/state not reused across peerIds.
                sortedPeers.forEach { peer ->
                    key(peer.peerId) {
                        Column {
                            PeerItem(peer, onClick = { onPeerClick(peer) })
                            HorizontalDivider(
                                modifier = Modifier.padding(vertical = 4.dp),
                                color = MaterialTheme.colorScheme.surfaceVariant
                            )
                        }
                    }
                }
            }

            // Navigation to detailed views
            Spacer(modifier = Modifier.height(16.dp))
            DashboardToPeerListNavigation(
                onNavigateToPeerList = { onNavigateToPeerList() },
                modifier = Modifier.padding(horizontal = 16.dp)
            )
            DashboardToTopologyNavigation(
                onNavigateToTopology = { onNavigateToTopology() },
                modifier = Modifier.padding(horizontal = 16.dp)
            )
            DashboardToJoinMeshNavigation(
                onNavigateToJoinMesh = { onNavigateToJoinMesh() },
                modifier = Modifier.padding(horizontal = 16.dp)
            )
            Spacer(modifier = Modifier.height(16.dp))
        }
    }
}

@Composable
fun PeerItem(peer: com.scmessenger.android.ui.viewmodels.PeerInfo, onClick: (() -> Unit)? = null) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(enabled = onClick != null) { onClick?.invoke() }
            .padding(vertical = 8.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Box(
            modifier = Modifier
                .size(40.dp)
                .background(
                    if (peer.isOnline) MaterialTheme.colorScheme.primaryContainer else MaterialTheme.colorScheme.surfaceVariant,
                    CircleShape
                ),
            contentAlignment = Alignment.Center
        ) {
            Icon(
                when {
                    peer.isFull -> Icons.Filled.Person
                    else -> Icons.Filled.People
                },
                contentDescription = null,
                tint = if (peer.isOnline) MaterialTheme.colorScheme.onPrimaryContainer else MaterialTheme.colorScheme.onSurfaceVariant
            )
        }

        Spacer(modifier = Modifier.width(12.dp))

        Column(modifier = Modifier.weight(1f)) {
            // UNIFICATION FIX: displayName shows localNickname first, filters synthetic peer-... fallback.
            // Previously showed synthetic peer-... as primary when local was null, even though contact has ChristyLove.
            // Now synthetic is treated as blank, so PK:... fallback shows until real name arrives, and contact's local wins.
            val primary = peer.localNickname?.trim()?.takeIf { it.isNotEmpty() }?.takeUnless { it.lowercase().startsWith("peer-") }
            val secondary = peer.nickname?.trim()?.takeIf { it.isNotEmpty() }?.takeUnless { it.lowercase().startsWith("peer-") }
            val display = primary ?: secondary
            Text(
                text = display
                    ?: when {
                        peer.isFull -> stringResource(R.string.dashboard_label_node)
                        else -> stringResource(R.string.dashboard_label_headless_node)
                    },
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = FontWeight.Bold
            )
            if (secondary != null && primary != null && secondary != primary) {
                Text(
                    text = "@${secondary}",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
            // UNIFICATION_V2: All nodes are relays — no infrastructure label (former isInfrastructure badge removed).
            // UNIFICATION_V2_IDENTITY: peerId is canonical public_key_hex (64 hex), not libp2p 12D3.
            // Explicit PK:/P2P: prefix avoids identity hash confusion; 12D3 shown only for legacy transport IDs.
            // NODE-TRANSPORT-VIS-001: show every known transport distinctly.
            Text(
                text = buildString {
                    val idPrefix = if (peer.peerId.startsWith("12D3")) "P2P:" else "PK:"
                    val idTrunc = if (peer.peerId.startsWith("12D3")) peer.peerId.take(12) else peer.peerId.take(8)
                    append(idPrefix)
                    append(idTrunc)
                    append(" • ")
                    append(peer.transports.ifEmpty { listOf(peer.transport) }.joinToString(" / "))
                    append(" • ")
                    append(
                        when {
                            peer.isFull -> stringResource(R.string.dashboard_label_node)
                            else -> stringResource(R.string.dashboard_label_headless_node)
                        }
                    )
                },
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
            if (!peer.isOnline && peer.lastSeen != null && peer.lastSeen > 0uL) {
                Text(
                    text = stringResource(R.string.peer_list_last_seen, formatRelativeTimestamp(peer.lastSeen)),
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        }

        if (peer.isOnline) {
            Box(
                modifier = Modifier
                    .size(8.dp)
                    .background(Color.Green, CircleShape)
            )
        }
    }
}

private fun formatRelativeTimestamp(timestamp: ULong): String {
    val diff = (System.currentTimeMillis() - timestamp.toEpochMillis()) / 1000
    return when {
        diff < 60 -> "just now"
        diff < 3600 -> "${diff / 60}m ago"
        diff < 86400 -> "${diff / 3600}h ago"
        else -> "${diff / 86400}d ago"
    }
}

@Composable
fun StatusCard(
    isRunning: Boolean,
    stateName: String,
    isStorageDegraded: Boolean = false
) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(
            containerColor = when {
                isRunning -> MaterialTheme.colorScheme.primaryContainer
                isStorageDegraded -> MaterialTheme.colorScheme.errorContainer
                else -> MaterialTheme.colorScheme.surfaceVariant
            }
        )
    ) {
        Row(
            modifier = Modifier
                .padding(24.dp)
                .fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            Column {
                Text(
                    text = when {
                        isRunning -> stringResource(R.string.dashboard_status_active)
                        isStorageDegraded -> stringResource(R.string.storage_error_title)
                        else -> stringResource(R.string.dashboard_status_stopped)
                    },
                    style = MaterialTheme.typography.headlineSmall,
                    fontWeight = FontWeight.Bold,
                    color = when {
                        isRunning -> MaterialTheme.colorScheme.onPrimaryContainer
                        isStorageDegraded -> MaterialTheme.colorScheme.onErrorContainer
                        else -> MaterialTheme.colorScheme.onSurfaceVariant
                    }
                )
                Text(
                    text = if (isStorageDegraded) {
                        stringResource(R.string.storage_error_degraded_description)
                    } else {
                        stringResource(R.string.dashboard_label_state_format, stateName)
                    },
                    style = MaterialTheme.typography.bodyMedium,
                    color = when {
                        isRunning -> MaterialTheme.colorScheme.onPrimaryContainer.copy(alpha = 0.8f)
                        isStorageDegraded -> MaterialTheme.colorScheme.onErrorContainer.copy(alpha = 0.9f)
                        else -> MaterialTheme.colorScheme.onSurfaceVariant
                    }
                )
            }
        }
    }
}

@Composable
fun StatCard(
    modifier: Modifier = Modifier,
    title: String,
    value: String,
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    color: Color
) {
    Card(
        modifier = modifier
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            horizontalAlignment = Alignment.Start
        ) {
            Box(
                modifier = Modifier
                    .size(40.dp)
                    .background(color.copy(alpha = 0.2f), CircleShape),
                contentAlignment = Alignment.Center
            ) {
                Icon(icon, contentDescription = null, tint = color)
            }
            Spacer(modifier = Modifier.height(12.dp))
            Text(
                text = value,
                style = MaterialTheme.typography.headlineMedium,
                fontWeight = FontWeight.Bold
            )
            Text(
                text = title,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
    }
}

@Composable
fun ConnectionStatusCard(
    bleEnabled: Boolean,
    wifiAwareEnabled: Boolean,
    wifiDirectEnabled: Boolean,
    internetRelayEnabled: Boolean
) {
    Card(modifier = Modifier.fillMaxWidth()) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text(
                text = stringResource(R.string.dashboard_section_transports),
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(bottom = 12.dp)
            )

            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                TransportItem("BLE", Icons.Filled.Bluetooth, bleEnabled)
                TransportItem("WiFi Aware", Icons.Filled.Wifi, wifiAwareEnabled)
                TransportItem("WiFi Direct", Icons.Filled.Router, wifiDirectEnabled)
                TransportItem("Internet Relay", Icons.Filled.NetworkWifi, internetRelayEnabled)
            }
        }
    }
}

@Composable
fun TransportItem(name: String, icon: androidx.compose.ui.graphics.vector.ImageVector, enabled: Boolean) {
    Column(horizontalAlignment = Alignment.CenterHorizontally) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = if (enabled) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.outline
        )
        Spacer(modifier = Modifier.height(4.dp))
        Text(text = name, style = MaterialTheme.typography.labelSmall)
    }
}

@Composable
fun TextDetailRow(label: String, value: String) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp),
        horizontalArrangement = Arrangement.SpaceBetween
    ) {
        Text(text = label, style = MaterialTheme.typography.bodyMedium)
        Text(text = value, style = MaterialTheme.typography.bodyMedium, fontWeight = FontWeight.SemiBold)
    }
}

private fun formatBytes(bytes: ULong): String {
    return when {
        bytes < 1024u -> "$bytes B"
        bytes < 1024u * 1024u -> "${bytes / 1024u} KB"
        bytes < 1024u * 1024u * 1024u -> "${bytes / (1024u * 1024u)} MB"
        else -> "${bytes / (1024u * 1024u * 1024u)} GB"
    }
}

private fun formatDuration(seconds: ULong): String {
    val secs = seconds.toLong()
    val hours = secs / 3600
    val minutes = (secs % 3600) / 60
    return "${hours}h ${minutes}m"
}

/**
 * Navigation helper to navigate to PeerListScreen.
 */
@Composable
fun DashboardToPeerListNavigation(
    onNavigateToPeerList: () -> Unit,
    modifier: Modifier = Modifier
) {
    Card(modifier = modifier.fillMaxWidth()) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text(
                text = stringResource(R.string.dashboard_nav_peers_title),
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(bottom = 8.dp)
            )
            Text(
                text = stringResource(R.string.dashboard_nav_peers_description),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(bottom = 12.dp)
            )
            Button(
                onClick = onNavigateToPeerList,
                modifier = Modifier.fillMaxWidth()
            ) {
                Text(stringResource(R.string.dashboard_nav_peers_action))
            }
        }
    }
}

/**
 * Navigation helper to navigate to TopologyScreen.
 */
@Composable
fun DashboardToTopologyNavigation(
    onNavigateToTopology: () -> Unit,
    modifier: Modifier = Modifier
) {
    Card(modifier = modifier.fillMaxWidth()) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text(
                text = stringResource(R.string.dashboard_nav_topology_title),
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(bottom = 8.dp)
            )
            Text(
                text = stringResource(R.string.dashboard_nav_topology_description),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(bottom = 12.dp)
            )
            Button(
                onClick = onNavigateToTopology,
                modifier = Modifier.fillMaxWidth()
            ) {
                Text(stringResource(R.string.dashboard_nav_topology_action))
            }
        }
    }
}

/**
 * Navigation helper to navigate to JoinMeshScreen.
 */
@Composable
fun DashboardToJoinMeshNavigation(
    onNavigateToJoinMesh: () -> Unit,
    modifier: Modifier = Modifier
) {
    Card(modifier = modifier.fillMaxWidth()) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text(
                text = stringResource(R.string.join_mesh_qr_title),
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(bottom = 8.dp)
            )
            Text(
                text = stringResource(R.string.join_mesh_qr_description),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(bottom = 12.dp)
            )
            Button(
                onClick = onNavigateToJoinMesh,
                modifier = Modifier.fillMaxWidth()
            ) {
                Text(stringResource(R.string.join_mesh_qr_title))
            }
        }
    }
}
