package com.scmessenger.android.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
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
    onNavigateToTopology: () -> Unit = {}
) {
    val serviceState by serviceViewModel.serviceState.collectAsState()
    val isRunning by serviceViewModel.isRunning.collectAsState()
    val stats by serviceViewModel.serviceStats.collectAsState()

    val fullPeers by dashboardViewModel.fullPeersCount.collectAsState()
    val headlessPeers by dashboardViewModel.headlessPeersCount.collectAsState()
    val totalPeers by dashboardViewModel.totalPeersCount.collectAsState()

    val meshSettings by settingsViewModel.settings.collectAsState()

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(stringResource(R.string.dashboard_title)) }
            )
        }
    ) { paddingValues ->
        val peers by dashboardViewModel.peers.collectAsState()

        LazyColumn(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues),
            contentPadding = PaddingValues(16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            // Service Status Card
            item {
                StatusCard(
                    isRunning = isRunning,
                    stateName = serviceState.name
                )
            }

            // Quick Stats Grid
            item {
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
                        value = totalPeers.toString(),
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
            }

            // Connection Methods Status
            item {
                ConnectionStatusCard(
                    bleEnabled = meshSettings?.bleEnabled ?: false,
                    wifiAwareEnabled = meshSettings?.wifiAwareEnabled ?: false,
                    wifiDirectEnabled = meshSettings?.wifiDirectEnabled ?: false,
                    internetRelayEnabled = meshSettings?.relayEnabled == true && meshSettings?.internetEnabled == true
                )
            }

            // Detailed Stats
            item {
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
            }

            // Discovered Nodes Header
            item {
                Text(
                    text = stringResource(R.string.dashboard_section_discovered),
                    style = MaterialTheme.typography.titleMedium,
                    modifier = Modifier.padding(top = 8.dp)
                )
            }

            // Discovered Nodes List
            if (peers.isEmpty()) {
                item {
                    Text(
                        text = stringResource(R.string.dashboard_empty_state_discovered),
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            } else {
                items(peers) { peer ->
                    PeerItem(peer)
                    HorizontalDivider(
                        modifier = Modifier.padding(vertical = 4.dp),
                        color = MaterialTheme.colorScheme.surfaceVariant
                    )
                }
            }

            // Navigation to detailed views
            item {
                Spacer(modifier = Modifier.height(16.dp))
            }
            item {
                DashboardToPeerListNavigation(
                    onNavigateToPeerList = { onNavigateToPeerList() },
                    modifier = Modifier.padding(horizontal = 16.dp)
                )
            }
            item {
                DashboardToTopologyNavigation(
                    onNavigateToTopology = { onNavigateToTopology() },
                    modifier = Modifier.padding(horizontal = 16.dp)
                )
            }
            item {
                Spacer(modifier = Modifier.height(16.dp))
            }
        }
    }
}

@Composable
fun PeerItem(peer: com.scmessenger.android.ui.viewmodels.PeerInfo) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
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
            Text(
                text = peer.localNickname
                    ?: peer.nickname
                    ?: when {
                        peer.isFull -> stringResource(R.string.dashboard_label_node)
                        else -> stringResource(R.string.dashboard_label_headless_node)
                    },
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = FontWeight.Bold
            )
            if (peer.nickname != null && peer.localNickname != null) {
                Text(
                    text = "@${peer.nickname}",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
            Text(
                text = buildString {
                    append("ID: ")
                    append(peer.peerId.take(12))
                    append("... • ")
                    append(peer.transport)
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

@Composable
fun StatusCard(
    isRunning: Boolean,
    stateName: String
) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(
            containerColor = if (isRunning) MaterialTheme.colorScheme.primaryContainer else MaterialTheme.colorScheme.surfaceVariant
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
                    text = if (isRunning) stringResource(R.string.dashboard_status_active) else stringResource(R.string.dashboard_status_stopped),
                    style = MaterialTheme.typography.headlineSmall,
                    fontWeight = FontWeight.Bold,
                    color = if (isRunning) MaterialTheme.colorScheme.onPrimaryContainer else MaterialTheme.colorScheme.onSurfaceVariant
                )
                Text(
                    text = stringResource(R.string.dashboard_label_state_format, stateName),
                    style = MaterialTheme.typography.bodyMedium,
                    color = if (isRunning) MaterialTheme.colorScheme.onPrimaryContainer.copy(alpha = 0.8f) else MaterialTheme.colorScheme.onSurfaceVariant
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
