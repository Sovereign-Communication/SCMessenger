package com.scmessenger.android.ui.join

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import androidx.annotation.StringRes
import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Error
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.Modifier
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
import com.scmessenger.android.data.InviteFailure
import com.scmessenger.android.data.InviteRedeemResult
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.ui.components.QrCodeImage
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import timber.log.Timber

/**
 * JoinMeshScreen: join the mesh by redeeming a signed `SCI1:` invite.
 *
 * The invite (QR scan, pasted text, or text shared into the app) is the only
 * bootstrap source (#469). Core verifies the Ed25519 signature and imports
 * the seed ledger; the repository then dials the seeds and puts discovery
 * into its aggressive phase. A redeemed invite never reports "no peers":
 * if no seed is reachable yet, discovery simply keeps going.
 *
 * The same screen can also show this node's own invite (QR plus a copyable
 * string) for another device to redeem.
 */
@Composable
fun JoinMeshScreen(
    repository: MeshRepository,
    onJoinSuccess: () -> Unit,
    onCancel: () -> Unit,
    initialInvite: String? = null
) {
    val scope = rememberCoroutineScope()
    var joinState by remember { mutableStateOf(JoinState.SCANNING) }
    var failure by remember { mutableStateOf<InviteFailure?>(null) }
    var scanErrorMessage by remember { mutableStateOf<String?>(null) }
    var importedCount by remember { mutableStateOf(0) }
    var myInvite by remember { mutableStateOf<String?>(null) }

    fun redeem(raw: String?) {
        joinState = JoinState.PARSING
        scope.launch {
            val result = withContext(Dispatchers.IO) { repository.redeemInvite(raw) }
            when (result) {
                is InviteRedeemResult.Success -> {
                    importedCount = result.report.addressesImported
                    joinState = JoinState.SUCCESS
                }
                is InviteRedeemResult.Failure -> {
                    failure = result.reason
                    joinState = JoinState.ERROR
                }
            }
        }
    }

    // An invite shared into the app is redeemed as soon as the screen opens.
    LaunchedEffect(initialInvite) {
        if (!initialInvite.isNullOrBlank()) redeem(initialInvite)
    }

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
                    onInviteText = { text -> redeem(text) },
                    onScanError = { message ->
                        scanErrorMessage = message
                        failure = null
                        joinState = JoinState.ERROR
                    },
                    onShowMyInvite = {
                        joinState = JoinState.MY_INVITE
                        scope.launch {
                            myInvite = withContext(Dispatchers.IO) { repository.createInvite() }
                        }
                    },
                    onCancel = onCancel
                )
            }

            JoinState.PARSING -> {
                ParsingView()
            }

            JoinState.MY_INVITE -> {
                MyInviteView(
                    invite = myInvite,
                    onBack = {
                        myInvite = null
                        joinState = JoinState.SCANNING
                    }
                )
            }

            JoinState.SUCCESS -> {
                SuccessView(importedCount = importedCount, onComplete = onJoinSuccess)
            }

            JoinState.ERROR -> {
                ErrorView(
                    message = scanErrorMessage ?: stringResource(failureMessageRes(failure)),
                    onRetry = {
                        joinState = JoinState.SCANNING
                        failure = null
                        scanErrorMessage = null
                    },
                    onCancel = onCancel
                )
            }
        }
    }
}

/** Maps a redeem failure to its user-facing string resource. */
@StringRes
internal fun failureMessageRes(reason: InviteFailure?): Int = when (reason) {
    InviteFailure.EMPTY -> R.string.join_mesh_error_empty
    InviteFailure.NOT_AN_INVITE -> R.string.join_mesh_error_not_invite
    InviteFailure.INVALID -> R.string.join_mesh_error_invalid
    InviteFailure.BAD_SIGNATURE -> R.string.join_mesh_error_bad_signature
    InviteFailure.NO_IDENTITY -> R.string.join_mesh_error_no_identity
    InviteFailure.UNKNOWN, null -> R.string.join_mesh_error_unknown
}

/**
 * QR scanner launcher using Google Code Scanner (ML Kit), plus paste and
 * "show my invite" entry points.
 */
@Composable
private fun QrScannerView(
    onInviteText: (String) -> Unit,
    onScanError: (String) -> Unit,
    onShowMyInvite: () -> Unit,
    onCancel: () -> Unit
) {
    val context = LocalContext.current
    // The Google Code Scanner is a separate activity whose Task listeners are
    // not lifecycle-bound. Once this view leaves composition a late result
    // must be dropped, not delivered into a screen that no longer exists.
    val viewActive = remember { java.util.concurrent.atomic.AtomicBoolean(true) }
    DisposableEffect(Unit) {
        onDispose { viewActive.set(false) }
    }
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
                        if (!viewActive.get()) return@addOnSuccessListener
                        val rawValue = barcode.rawValue
                        if (rawValue.isNullOrBlank()) {
                            onScanError(qrEmptyError)
                        } else {
                            onInviteText(rawValue)
                        }
                    }
                    .addOnFailureListener { e ->
                        Timber.w(e, "Join QR scan failed")
                        if (!viewActive.get()) return@addOnFailureListener
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
                    onInviteText(text)
                } else {
                    onScanError(context.getString(R.string.join_mesh_error_empty))
                }
            }
        ) {
            Text(stringResource(R.string.join_mesh_paste_invite))
        }

        Spacer(modifier = Modifier.height(8.dp))

        TextButton(onClick = onShowMyInvite) {
            Text(stringResource(R.string.join_mesh_create_button))
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
 * Verifying view with spinner.
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
 * This node's own invite: QR code plus a copyable / shareable string.
 * While the node has no reachable address yet the view says discovery is
 * still running instead of failing.
 */
@Composable
private fun MyInviteView(
    invite: String?,
    onBack: () -> Unit
) {
    val context = LocalContext.current
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Text(
            text = stringResource(R.string.join_mesh_create_title),
            style = MaterialTheme.typography.headlineMedium,
            textAlign = TextAlign.Center
        )
        Spacer(modifier = Modifier.height(8.dp))
        Text(
            text = stringResource(R.string.join_mesh_create_description),
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center
        )
        Spacer(modifier = Modifier.height(16.dp))

        if (invite == null) {
            Text(
                text = stringResource(R.string.join_mesh_create_unavailable),
                style = MaterialTheme.typography.bodyMedium,
                textAlign = TextAlign.Center
            )
        } else {
            QrCodeImage(
                data = invite,
                contentDescription = stringResource(R.string.join_mesh_invite_qr_content_description),
                size = 1024
            )
            Spacer(modifier = Modifier.height(16.dp))
            val copiedLabel = stringResource(R.string.join_mesh_invite_copied)
            Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                OutlinedButton(onClick = {
                    val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as? ClipboardManager
                    clipboard?.setPrimaryClip(ClipData.newPlainText(copiedLabel, invite))
                    android.widget.Toast.makeText(context, copiedLabel, android.widget.Toast.LENGTH_SHORT).show()
                }) {
                    Text(stringResource(R.string.join_mesh_copy_invite))
                }
                Button(onClick = {
                    val send = Intent(Intent.ACTION_SEND).apply {
                        type = "text/plain"
                        putExtra(Intent.EXTRA_TEXT, invite)
                    }
                    try {
                        context.startActivity(Intent.createChooser(send, null))
                    } catch (e: Exception) {
                        Timber.w(e, "No app available to share invite")
                    }
                }) {
                    Text(stringResource(R.string.join_mesh_share_invite))
                }
            }
        }

        Spacer(modifier = Modifier.height(16.dp))
        OutlinedButton(onClick = onBack) {
            Text(stringResource(R.string.join_mesh_back))
        }
    }
}

/**
 * Success view.
 */
@Composable
private fun SuccessView(importedCount: Int, onComplete: () -> Unit) {
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

        Spacer(modifier = Modifier.height(8.dp))

        Text(
            text = stringResource(R.string.join_mesh_connected_detail, importedCount),
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center
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
 * Join state.
 */
private enum class JoinState {
    SCANNING,
    PARSING,
    MY_INVITE,
    SUCCESS,
    ERROR
}
