package com.scmessenger.android.ui.join

import android.content.ClipboardManager
import android.content.Context
import androidx.compose.animation.core.*
import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Error
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import com.google.android.gms.common.ConnectionResult
import com.google.android.gms.common.GoogleApiAvailability
import com.google.android.gms.common.api.CommonStatusCodes
import com.google.mlkit.common.MlKitException
import com.google.mlkit.vision.barcode.common.Barcode
import com.google.mlkit.vision.codescanner.GmsBarcodeScannerOptions
import com.google.mlkit.vision.codescanner.GmsBarcodeScanning
import com.scmessenger.android.R
import com.scmessenger.android.data.MeshRepository
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import timber.log.Timber

/**
 * JoinMeshScreen for joining an existing mesh network.
 *
 * Features:
 * - QR scanner for join bundle (server.rs handle_join_bundle)
 * - Parse join bundle (bootstrap peers, topics, identity exchange)
 * - Dial bootstrap peers via SwarmHandle.dial()
 * - Subscribe to discovered topics
 * - Connection progress animation
 *
 * Join Bundle Format (JSON):
 * {
 *   "bootstrap_peers": ["/ip4/x.x.x.x/tcp/yyyy"],
 *   "topics": ["/scmessenger/global/v1"],
 *   "identity": "base64_encoded_public_key",
 *   "timestamp": 1234567890
 * }
 */
@Composable
fun JoinMeshScreen(
    repository: MeshRepository,
    onJoinSuccess: () -> Unit,
    onCancel: () -> Unit
) {
    val scope = rememberCoroutineScope()
    var joinState by remember { mutableStateOf(JoinState.SCANNING) }
    var errorMessage by remember { mutableStateOf<String?>(null) }
    var connectionProgress by remember { mutableStateOf(0f) }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(16.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        when (joinState) {
            JoinState.SCANNING -> {
                QrScannerView(
                    onQrScanned = { qrData ->
                        joinState = JoinState.PARSING
                        scope.launch {
                            parseAndJoin(
                                repository = repository,
                                qrData = qrData,
                                onProgress = { progress ->
                                    connectionProgress = progress
                                    joinState = JoinState.CONNECTING
                                },
                                onSuccess = {
                                    joinState = JoinState.SUCCESS
                                },
                                onError = { error ->
                                    errorMessage = error
                                    joinState = JoinState.ERROR
                                }
                            )
                        }
                    },
                    onScanError = { message ->
                        errorMessage = message
                        joinState = JoinState.ERROR
                    },
                    onCancel = onCancel
                )
            }

            JoinState.PARSING -> {
                ParsingView()
            }

            JoinState.CONNECTING -> {
                ConnectingView(connectionProgress)
            }

            JoinState.SUCCESS -> {
                SuccessView(onComplete = onJoinSuccess)
            }

            JoinState.ERROR -> {
                ErrorView(
                    errorMessage ?: stringResource(com.scmessenger.android.R.string.unknown_error),
                    onRetry = {
                        joinState = JoinState.SCANNING
                        errorMessage = null
                    },
                    onCancel = onCancel
                )
            }
        }
    }
}

/**
 * QR scanner launcher using Google Code Scanner (ML Kit).
 */
@Composable
private fun QrScannerView(
    onQrScanned: (String) -> Unit,
    onScanError: (String) -> Unit,
    onCancel: () -> Unit
) {
    val context = LocalContext.current
    Column(
        modifier = Modifier.fillMaxSize(),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Text(
            text = stringResource(R.string.join_mesh_qr_title),
            style = MaterialTheme.typography.headlineMedium,
            textAlign = TextAlign.Center
        )

        Spacer(modifier = Modifier.height(16.dp))

        Text(
            text = stringResource(R.string.join_mesh_qr_description),
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center
        )

        Spacer(modifier = Modifier.height(24.dp))

        OutlinedButton(onClick = onCancel) {
            Text(stringResource(R.string.cancel))
        }

        Spacer(modifier = Modifier.height(16.dp))

        val gmsAvailable = remember {
            GoogleApiAvailability.getInstance().isGooglePlayServicesAvailable(context) == ConnectionResult.SUCCESS
        }

        Button(
            onClick = {
                val options = GmsBarcodeScannerOptions.Builder()
                    .setBarcodeFormats(Barcode.FORMAT_QR_CODE)
                    .build()
                val scanner = GmsBarcodeScanning.getClient(context, options)
                val qrEmptyError = context.getString(R.string.add_contact_error_qr_empty)
                val qrFailedError = context.getString(R.string.add_contact_error_qr_failed)

                scanner.startScan()
                    .addOnSuccessListener { barcode ->
                        val rawValue = barcode.rawValue
                        if (rawValue.isNullOrBlank()) {
                            onScanError(qrEmptyError)
                        } else {
                            onQrScanned(rawValue)
                        }
                    }
                    .addOnFailureListener { e ->
                        Timber.w(e, "Join QR scan failed")
                        if (e is MlKitException && e.errorCode == CommonStatusCodes.CANCELED) {
                            return@addOnFailureListener
                        }
                        onScanError(qrFailedError)
                    }
            },
            enabled = gmsAvailable
        ) {
            Text(stringResource(R.string.join_mesh_qr_title))
        }

        Spacer(modifier = Modifier.height(8.dp))

        OutlinedButton(
            onClick = {
                val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as? ClipboardManager
                val clipData = clipboard?.primaryClip
                val text = if (clipData != null && clipData.itemCount > 0) {
                    clipData.getItemAt(0)?.text?.toString()?.trim()
                } else null

                if (!text.isNullOrBlank()) {
                    onQrScanned(text)
                } else {
                    onScanError(context.getString(R.string.add_contact_error_qr_empty))
                }
            }
        ) {
            Text("Paste Join Bundle")
        }

        if (!gmsAvailable) {
            Spacer(modifier = Modifier.height(8.dp))
            Text(
                text = stringResource(R.string.add_contact_gms_requirement_note),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.error,
                textAlign = TextAlign.Center
            )
        }
    }
}

/**
 * Parsing view with spinner.
 */
@Composable
private fun ParsingView() {
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        CircularProgressIndicator()
        Spacer(modifier = Modifier.height(16.dp))
        Text(stringResource(R.string.join_mesh_parsing), style = MaterialTheme.typography.bodyLarge)
    }
}

/**
 * Connecting view with animated progress.
 */
@Composable
private fun ConnectingView(progress: Float) {
    val infiniteTransition = rememberInfiniteTransition(label = "rotation")
    val rotation by infiniteTransition.animateFloat(
        initialValue = 0f,
        targetValue = 360f,
        animationSpec = infiniteRepeatable(
            animation = tween(2000, easing = LinearEasing),
            repeatMode = RepeatMode.Restart
        ),
        label = "rotation"
    )

    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Icon(
            imageVector = Icons.Default.CheckCircle,
            contentDescription = null,
            modifier = Modifier
                .size(64.dp)
                .rotate(rotation),
            tint = MaterialTheme.colorScheme.primary
        )

        Spacer(modifier = Modifier.height(24.dp))

        Text(
            text = stringResource(R.string.join_mesh_connecting),
            style = MaterialTheme.typography.headlineSmall
        )

        Spacer(modifier = Modifier.height(16.dp))

        LinearProgressIndicator(
            progress = { progress },
            modifier = Modifier
                .fillMaxWidth(0.7f)
                .height(8.dp),
        )

        Spacer(modifier = Modifier.height(8.dp))

        Text(
            text = "${(progress * 100).toInt()}%",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
    }
}

/**
 * Success view.
 */
@Composable
private fun SuccessView(onComplete: () -> Unit) {
    LaunchedEffect(Unit) {
        kotlinx.coroutines.delay(1500)
        onComplete()
    }

    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Icon(
            imageVector = Icons.Default.CheckCircle,
            contentDescription = null,
            modifier = Modifier.size(64.dp),
            tint = Color(0xFF4CAF50)
        )

        Spacer(modifier = Modifier.height(16.dp))

        Text(
            text = stringResource(R.string.join_mesh_connected),
            style = MaterialTheme.typography.headlineSmall,
            color = Color(0xFF4CAF50)
        )
    }
}

/**
 * Error view.
 */
@Composable
private fun ErrorView(
    message: String,
    onRetry: () -> Unit,
    onCancel: () -> Unit
) {
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Icon(
            imageVector = Icons.Default.Error,
            contentDescription = null,
            modifier = Modifier.size(64.dp),
            tint = MaterialTheme.colorScheme.error
        )

        Spacer(modifier = Modifier.height(16.dp))

        Text(
            text = stringResource(R.string.join_mesh_connection_failed),
            style = MaterialTheme.typography.headlineSmall,
            color = MaterialTheme.colorScheme.error
        )

        Spacer(modifier = Modifier.height(8.dp))

        Text(
            text = message,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
            modifier = Modifier.padding(horizontal = 32.dp)
        )

        Spacer(modifier = Modifier.height(24.dp))

        Row(
            horizontalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            OutlinedButton(onClick = onCancel) {
                Text(stringResource(R.string.cancel))
            }
            Button(onClick = onRetry) {
                Text(stringResource(R.string.retry))
            }
        }
    }
}

/**
 * Parse join bundle and connect to bootstrap peers.
 */
private suspend fun parseAndJoin(
    repository: MeshRepository,
    qrData: String,
    onProgress: (Float) -> Unit,
    onSuccess: () -> Unit,
    onError: (String) -> Unit
) = withContext(Dispatchers.IO) {
    try {
        Timber.d("Parsing join bundle (length: ${qrData.length})")

        // Parse JSON (simplified - would use kotlinx.serialization)
        val jsonRegex = """"bootstrap_peers":\s*\[(.*?)\]""".toRegex()
        val topicsRegex = """"topics":\s*\[(.*?)\]""".toRegex()

        val peersMatch = jsonRegex.find(qrData)
        val topicsMatch = topicsRegex.find(qrData)

        if (peersMatch == null || topicsMatch == null) {
            onError("Invalid join bundle format")
            return@withContext
        }

        val bootstrapPeers = peersMatch.groupValues[1]
            .split(",")
            .map { it.trim().removeSurrounding("\"") }
            .filter { it.isNotEmpty() }

        val topics = topicsMatch.groupValues[1]
            .split(",")
            .map { it.trim().removeSurrounding("\"") }
            .filter { it.isNotEmpty() }

        Timber.i("Bootstrap peers: $bootstrapPeers")
        Timber.i("Topics: $topics")

        if (bootstrapPeers.isEmpty()) {
            onError("No bootstrap peers in bundle")
            return@withContext
        }

        // Progress: 0.25 after parsing
        onProgress(0.25f)

        // Dial bootstrap peers
        var successCount = 0
        bootstrapPeers.forEachIndexed { index, peer ->
            withContext(Dispatchers.Main) {
                try {
                    repository.dialPeer(peer)
                    successCount++
                    Timber.i("Dialed peer: $peer")
                    onProgress(0.25f + (0.5f * (index + 1) / bootstrapPeers.size))
                } catch (e: Exception) {
                    Timber.w(e, "Failed to dial peer: $peer")
                }
            }
        }

        if (successCount == 0) {
            withContext(Dispatchers.Main) {
                onError("Failed to connect to any bootstrap peers")
            }
            return@withContext
        }

        // Subscribe to topics
        topics.forEach { topic ->
            withContext(Dispatchers.Main) {
                try {
                    repository.subscribeTopic(topic)
                    Timber.i("Subscribed to topic: $topic")
                } catch (e: Exception) {
                    Timber.w(e, "Failed to subscribe to topic: $topic")
                }
            }
        }

        onProgress(1.0f)
        withContext(Dispatchers.Main) {
            onSuccess()
        }

    } catch (e: Exception) {
        Timber.e(e, "Failed to parse and join")
        onError(e.message ?: "Unknown error")
    }
}

/**
 * Join state.
 */
private enum class JoinState {
    SCANNING,
    PARSING,
    CONNECTING,
    SUCCESS,
    ERROR
}
