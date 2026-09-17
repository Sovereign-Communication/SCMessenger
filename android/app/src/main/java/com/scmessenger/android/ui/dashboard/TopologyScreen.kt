package com.scmessenger.android.ui.dashboard

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.compose.ui.res.stringResource
import com.scmessenger.android.R
import com.scmessenger.android.ui.components.ErrorBanner
import com.scmessenger.android.ui.theme.*
import com.scmessenger.android.ui.viewmodels.DashboardViewModel
import com.scmessenger.android.ui.viewmodels.NetworkTopology
import com.scmessenger.android.ui.viewmodels.TopologyNode
import kotlin.math.cos
import kotlin.math.sin

/**
 * Topology screen - Canvas-based network graph visualization.
 *
 * Displays the mesh topology as an interactive graph:
 * - Nodes represent peers (self node highlighted)
 * - Edges represent connections (colored by transport type)
 * - Layout uses circular arrangement for clarity
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun TopologyScreen(
    onNavigateBack: () -> Unit,
    viewModel: DashboardViewModel = hiltViewModel()
) {
    val topology by viewModel.topology.collectAsState()
    val isLoading by viewModel.isLoading.collectAsState()
    val error by viewModel.error.collectAsState()

    LaunchedEffect(Unit) {
        viewModel.refreshData()
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(stringResource(R.string.topology_title)) },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = stringResource(R.string.chat_action_dismiss))
                    }
                },
                actions = {
                    IconButton(onClick = { viewModel.refreshData() }) {
                        Icon(Icons.Default.Refresh, contentDescription = stringResource(R.string.diagnostics_action_refresh))
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
            when {
                isLoading -> {
                    CircularProgressIndicator(
                        modifier = Modifier.align(Alignment.Center)
                    )
                }

                topology.nodes.isEmpty() -> {
                    Column(
                        modifier = Modifier
                            .align(Alignment.Center)
                            .padding(32.dp),
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        Text(
                            text = stringResource(R.string.topology_no_data),
                            style = MaterialTheme.typography.titleLarge
                        )

                        Spacer(modifier = Modifier.height(8.dp))

                        Text(
                            text = stringResource(R.string.topology_no_data_description),
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }

                else -> {
                    Column(
                        modifier = Modifier
                            .fillMaxSize()
                            .verticalScroll(rememberScrollState())
                    ) {
                        // Error banner
                        error?.let {
                            ErrorBanner(
                                message = it,
                                onDismiss = { viewModel.clearError() }
                            )
                        }

                        // Stats
                        TopologyStats(topology = topology)

                        // Graph visualization
                        TopologyGraph(
                            topology = topology,
                            modifier = Modifier
                                .fillMaxWidth()
                                .height(500.dp)
                                .padding(16.dp)
                        )

                        // Legend
                        TopologyLegend(
                            modifier = Modifier.padding(16.dp)
                        )

                        // TOPOLOGY-SCROLL-001: the app shell's bottom navigation
                        // bar overlaps the bottom of this screen's scroll range, so
                        // the legend's last connection-type row could never be
                        // scrolled into view (operator-reported "mesh topology does
                        // not scroll fully"): at maximum scroll the final row sat
                        // under the bar. Trailing content space lets it clear the
                        // bar, the same inset DashboardScreen gets from its own
                        // trailing content padding.
                        Spacer(modifier = Modifier.height(72.dp))
                    }
                }
            }
        }
    }
}

@Composable
private fun TopologyStats(
    topology: NetworkTopology,
    modifier: Modifier = Modifier
) {
    Surface(
        modifier = modifier.fillMaxWidth(),
        tonalElevation = 1.dp
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            horizontalArrangement = Arrangement.SpaceEvenly
        ) {
            StatItem(
                label = stringResource(R.string.topology_stat_nodes),
                value = topology.nodes.size.toString()
            )

            StatItem(
                label = stringResource(R.string.topology_stat_connections),
                value = topology.edges.size.toString()
            )

            StatItem(
                label = stringResource(R.string.topology_stat_online),
                value = topology.nodes.count { it.isOnline }.toString()
            )
        }
    }
}

@Composable
private fun StatItem(
    label: String,
    value: String,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier,
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Text(
            text = value,
            style = MaterialTheme.typography.headlineMedium,
            fontWeight = FontWeight.Bold,
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
private fun TopologyGraph(
    topology: NetworkTopology,
    modifier: Modifier = Modifier
) {
    val nodeColor = MaterialTheme.colorScheme.primary
    val selfNodeColor = MaterialTheme.colorScheme.secondary
    val offlineNodeColor = MaterialTheme.colorScheme.surfaceVariant

    Canvas(modifier = modifier.background(MaterialTheme.colorScheme.surface)) {
        val centerX = size.width / 2
        val centerY = size.height / 2
        // TOPOLOGY-CLIP-001: margin must scale with density - a raw 60px margin
        // is only ~23dp on modern phones, so nodes at the circle edge (radius
        // 25px + 35px highlight halo) drew past the Canvas bounds and were
        // clipped at the screen edges. Scale with density and keep enough room
        // for the halo plus breathing space.
        val margin = 48.dp.toPx() + 60f
        val radius = minOf(size.width, size.height) / 2 - margin

        // Calculate node positions in a circle
        val nodePositions = mutableMapOf<String, Offset>()

        topology.nodes.forEachIndexed { index, node ->
            if (node.isSelf) {
                // Place self node in center
                nodePositions[node.id] = Offset(centerX, centerY)
            } else {
                // Place other nodes in a circle
                val otherCount = topology.nodes.size - 1
                if (otherCount > 0) {
                    val angle = (2 * Math.PI * index) / otherCount
                    val x = centerX + (radius * cos(angle)).toFloat()
                    val y = centerY + (radius * sin(angle)).toFloat()
                    nodePositions[node.id] = Offset(x, y)
                }
            }
        }

        // Draw edges first (behind nodes)
        topology.edges.forEach { edge ->
            val source = nodePositions[edge.source]
            val target = nodePositions[edge.target]

            if (source != null && target != null) {
                val edgeColor = getTransportColor(edge.transport)

                drawLine(
                    color = edgeColor,
                    start = source,
                    end = target,
                    strokeWidth = 3f,
                    cap = StrokeCap.Round
                )
            }
        }

        // Draw nodes on top
        topology.nodes.forEach { node ->
            val position = nodePositions[node.id] ?: return@forEach
            val color = when {
                node.isSelf -> selfNodeColor
                !node.isOnline -> offlineNodeColor
                else -> nodeColor
            }

            // Outer circle (highlight for self)
            if (node.isSelf) {
                drawCircle(
                    color = color.copy(alpha = 0.3f),
                    radius = 35f,
                    center = position
                )
            }

            // Main node circle
            drawCircle(
                color = color,
                radius = 25f,
                center = position,
                style = Stroke(width = 4f)
            )

            // Inner fill
            drawCircle(
                color = color.copy(alpha = 0.2f),
                radius = 21f,
                center = position
            )
        }
    }
}

@Composable
private fun TopologyLegend(
    modifier: Modifier = Modifier
) {
    Card(modifier = modifier.fillMaxWidth()) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Text(
                text = stringResource(R.string.topology_label_legend),
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold
            )

            // Node types
            LegendItem(
                color = MaterialTheme.colorScheme.secondary,
                label = stringResource(R.string.topology_label_this_device)
            )
            LegendItem(
                color = MaterialTheme.colorScheme.primary,
                label = stringResource(R.string.topology_label_connected_peer)
            )
            LegendItem(
                color = MaterialTheme.colorScheme.surfaceVariant,
                label = stringResource(R.string.topology_label_offline_peer)
            )

            HorizontalDivider()

            // Transport types
            Text(
                text = stringResource(R.string.topology_label_connection_types),
                style = MaterialTheme.typography.titleSmall
            )

            LegendItem(color = TransportBLE, label = "BLE")
            LegendItem(color = TransportWiFiAware, label = "WiFi Aware")
            LegendItem(color = TransportWiFiDirect, label = "WiFi Direct")
            LegendItem(color = TransportInternet, label = "Internet")
        }
    }
}

@Composable
private fun LegendItem(
    color: Color,
    label: String,
    modifier: Modifier = Modifier
) {
    Row(
        modifier = modifier,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Box(
            modifier = Modifier
                .size(20.dp)
                .background(color, shape = MaterialTheme.shapes.small)
        )

        Text(
            text = label,
            style = MaterialTheme.typography.bodyMedium
        )
    }
}

/**
 * Get color for transport type.
 */
private fun getTransportColor(transport: String): Color {
    return when (transport) {
        "BLE" -> TransportBLE
        "WiFi Aware" -> TransportWiFiAware
        "WiFi Direct" -> TransportWiFiDirect
        "Internet" -> TransportInternet
        else -> Color.Gray
    }
}
