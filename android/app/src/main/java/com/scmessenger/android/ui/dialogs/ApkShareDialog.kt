package com.scmessenger.android.ui.dialogs

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.widget.Toast
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ContentCopy
import androidx.compose.material.icons.filled.Share
import androidx.compose.material.icons.filled.Stop
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import com.scmessenger.android.R
import com.scmessenger.android.ui.components.QrCodeImage
import com.scmessenger.android.utils.ApkShareManager
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

/**
 * Dialog for sharing the installed SCMessenger APK via:
 * 1. Native System Share (QuickShare, Bluetooth, Wi-Fi Direct, Chat)
 * 2. Local Node QR-Hosted Sideloading (ephemeral HTTP server on local Wi-Fi)
 */
@Composable
fun ApkShareDialog(
    onDismiss: () -> Unit
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    var downloadUrl by remember { mutableStateOf<String?>(null) }
    var isServerRunning by remember { mutableStateOf(false) }
    var secondsRemaining by remember { mutableLongStateOf(900L) } // 15 mins default

    LaunchedEffect(Unit) {
        ApkShareManager.startLocalApkHost(context, durationMinutes = 15) { url ->
            downloadUrl = url
            isServerRunning = true
        }
    }

    // Countdown timer loop
    LaunchedEffect(isServerRunning) {
        if (isServerRunning) {
            while (secondsRemaining > 0 && isServerRunning) {
                delay(1000L)
                secondsRemaining -= 1
            }
            if (secondsRemaining <= 0) {
                ApkShareManager.stopLocalApkHost()
                isServerRunning = false
            }
        }
    }

    DisposableEffect(Unit) {
        onDispose {
            ApkShareManager.stopLocalApkHost()
        }
    }

    AlertDialog(
        onDismissRequest = {
            ApkShareManager.stopLocalApkHost()
            onDismiss()
        },
        title = {
            Text(
                text = "Share SCMessenger App",
                style = MaterialTheme.typography.titleLarge,
                fontWeight = FontWeight.Bold
            )
        },
        text = {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .verticalScroll(rememberScrollState()),
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                Text(
                    text = "Allow nearby friends to install SCMessenger directly from your node.",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    textAlign = TextAlign.Center,
                    modifier = Modifier.padding(bottom = 16.dp)
                )

                // Option 1: Native System Share Button
                Button(
                    onClick = { ApkShareManager.shareApkViaSystemIntent(context) },
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Icon(Icons.Default.Share, contentDescription = null, modifier = Modifier.size(18.dp))
                    Spacer(modifier = Modifier.width(8.dp))
                    Text("Share APK File via Bluetooth/QuickShare")
                }

                Spacer(modifier = Modifier.height(16.dp))
                HorizontalDivider()
                Spacer(modifier = Modifier.height(16.dp))

                // Option 2: Scan to Install QR Code
                Text(
                    text = "Scan to Download over Local Wi-Fi",
                    style = MaterialTheme.typography.titleSmall,
                    fontWeight = FontWeight.SemiBold
                )

                Spacer(modifier = Modifier.height(12.dp))

                if (downloadUrl != null) {
                    Card(
                        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant),
                        modifier = Modifier.padding(8.dp)
                    ) {
                        Column(
                            horizontalAlignment = Alignment.CenterHorizontally,
                            modifier = Modifier.padding(16.dp)
                        ) {
                            QrCodeImage(
                                data = downloadUrl!!,
                                size = 200,
                                contentDescription = "QR Code for APK download"
                            )

                            Spacer(modifier = Modifier.height(12.dp))

                            Text(
                                text = downloadUrl!!,
                                style = MaterialTheme.typography.bodySmall,
                                fontFamily = FontFamily.Monospace,
                                textAlign = TextAlign.Center
                            )

                            Spacer(modifier = Modifier.height(8.dp))

                            OutlinedButton(
                                onClick = {
                                    val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
                                    val clip = ClipData.newPlainText("Download URL", downloadUrl)
                                    clipboard.setPrimaryClip(clip)
                                    Toast.makeText(context, "URL copied to clipboard", Toast.LENGTH_SHORT).show()
                                }
                            ) {
                                Icon(Icons.Default.ContentCopy, contentDescription = null, modifier = Modifier.size(16.dp))
                                Spacer(modifier = Modifier.width(6.dp))
                                Text("Copy Link")
                            }
                        }
                    }

                    val minutes = secondsRemaining / 60
                    val seconds = secondsRemaining % 60
                    val timerStr = String.format("%02d:%02d", minutes, seconds)

                    Text(
                        text = "Ephemeral node server active for $timerStr",
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.primary,
                        modifier = Modifier.padding(top = 8.dp)
                    )
                } else {
                    CircularProgressIndicator(modifier = Modifier.size(32.dp))
                    Spacer(modifier = Modifier.height(8.dp))
                    Text("Starting node HTTP server...", style = MaterialTheme.typography.bodySmall)
                }
            }
        },
        confirmButton = {
            TextButton(
                onClick = {
                    ApkShareManager.stopLocalApkHost()
                    onDismiss()
                }
            ) {
                Icon(Icons.Default.Stop, contentDescription = null, modifier = Modifier.size(16.dp))
                Spacer(modifier = Modifier.width(4.dp))
                Text("Stop Sharing")
            }
        }
    )
}
