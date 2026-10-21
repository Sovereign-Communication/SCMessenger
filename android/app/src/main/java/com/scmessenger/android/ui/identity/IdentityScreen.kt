package com.scmessenger.android.ui.identity

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
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.scmessenger.android.R
import com.scmessenger.android.ui.components.CopyableText
import com.scmessenger.android.ui.components.ErrorBanner
import com.scmessenger.android.ui.components.IdenticonFromPeerId
import com.scmessenger.android.ui.viewmodels.IdentityViewModel

import com.scmessenger.android.data.IdentityState

/**
 * Identity screen - Display public key, QR code, and export options.
 *
 * Shows the user's identity information including peer ID, public key,
 * and a scannable QR code for easy contact sharing.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun IdentityScreen(
    onNavigateBack: () -> Unit,
    viewModel: IdentityViewModel = hiltViewModel()
) {
    val identityInfo by viewModel.identityInfo.collectAsState()
    val identityState by viewModel.identityState.collectAsState()
    val isLoading by viewModel.isLoading.collectAsState()
    val error by viewModel.error.collectAsState()
    val successMessage by viewModel.successMessage.collectAsState()
    // P0_ANDROID_IDENTITY_PROOF_OF_WORK: pull the high-level proof-of-work stage
    // out of the ViewModel so we can show 6 named stages instead of a tiny
    // centered spinner. The user can see exactly which step of the cryptographic
    // pipeline is currently running.
    val progressStage by viewModel.progressStage.collectAsState()
    // P0_ANDROID_PROGRESS_CALLBACK: transient sub-stage detail from inside the
    // FFI call. Renders as a smaller brighter line under the active stage's
    // `detail` in IdentityProgressDisplay so the user sees motion during the
    // SharedPreferences commit() and the libp2p swarm bind.
    val progressSubDetail by viewModel.progressSubDetail.collectAsState()

    // Collect QR code data from ViewModel StateFlow
    val qrCodeData by viewModel.qrCodeData.collectAsState()

    // P0_ANDROID_QR_FIX: belt-and-suspenders poll. IdentityViewModel.loadIdentity
    // has its own retry loop (~1.55s) but the Rust core may hydrate LATER if the
    // service transitions to RUNNING *after* the VM was constructed (e.g., a slow
    // cold start where the user reaches Settings before MeshService.onCreate finished).
    // This LaunchedEffect is keyed on identityInfo itself, so it re-fires whenever
    // the state changes — but it also fires once on first composition. We schedule
    // two delayed polls (2s, 4s) to catch late hydration without busy-spinning.
    LaunchedEffect(identityInfo?.initialized) {
        if (identityInfo?.initialized != true) {
            kotlinx.coroutines.delay(2_000L)
            viewModel.loadIdentity(forceRefresh = true)
            kotlinx.coroutines.delay(2_000L)
            viewModel.loadIdentity(forceRefresh = true)
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(stringResource(R.string.identity_title)) },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = stringResource(R.string.chat_action_dismiss))
                    }
                },
                actions = {
                    IconButton(onClick = {
                        viewModel.loadIdentity()
                        viewModel.refreshQrCode()
                    }) {
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
            when (identityState) {
                IdentityState.CachedPendingHydration -> {
                    Column(
                        modifier = Modifier.align(Alignment.Center),
                        horizontalAlignment = Alignment.CenterHorizontally,
                        verticalArrangement = Arrangement.spacedBy(16.dp)
                    ) {
                        CircularProgressIndicator()
                        Text(
                            text = "Restoring your identity…",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }

                IdentityState.Restoring -> {
                    IdentityNotInitializedView(
                        isCreating = true,
                        progressStage = progressStage,
                        progressSubDetail = progressSubDetail,
                        onCreateIdentity = { nickname -> viewModel.createIdentity(nickname) },
                        modifier = Modifier.align(Alignment.Center)
                    )
                }

                IdentityState.Ready -> {
                    val resolvedIdentity = identityInfo
                    if (resolvedIdentity != null && resolvedIdentity.initialized) {
                        IdentityContent(
                            identityInfo = resolvedIdentity,
                            qrCodeData = qrCodeData,
                            error = error,
                            successMessage = successMessage,
                            onClearError = { viewModel.clearError() },
                            onClearSuccess = { viewModel.clearSuccessMessage() }
                        )
                    } else {
                        // Fallback in case state claims ready but info is not here yet
                        CircularProgressIndicator(
                            modifier = Modifier.align(Alignment.Center)
                        )
                    }
                }

                else -> {
                    IdentityNotInitializedView(
                        isCreating = progressStage !is com.scmessenger.android.ui.viewmodels.IdentityProgressStage.Idle,
                        progressStage = progressStage,
                        progressSubDetail = progressSubDetail,
                        onCreateIdentity = { nickname -> viewModel.createIdentity(nickname) },
                        modifier = Modifier.align(Alignment.Center)
                    )
                }
            }
        }
    }
}

@Composable
private fun IdentityNotInitializedView(
    isCreating: Boolean,
    // v0.3.4 (P0_ANDROID_CRASHFIX): parameter is now non-nullable. The previous
    // `IdentityProgressStage?` was the root cause of the NPE crash at
    // ProofOfWorkList.currentStage.id — the call site relied on a smart-cast
    // through a `if (progressStage != null)` guard that did not survive
    // Compose's recomposition, so a null value reached the call site. With
    // IdentityViewModel._progressStage typed as non-nullable StateFlow<IdentityProgressStage>
    // and initialized to IdentityProgressStage.Idle, the type system guarantees
    // progressStage is always a valid stage here. The Idle check below is the
    // only remaining null-equivalent branch we need.
    progressStage: com.scmessenger.android.ui.viewmodels.IdentityProgressStage,
    // P0_ANDROID_PROGRESS_CALLBACK: transient sub-stage detail from inside the
    // FFI call. Passed down from the parent composable (which collects from
    // IdentityViewModel.progressSubDetail) so we can render the secondary
    // line under the active stage's `detail`.
    progressSubDetail: String?,
    onCreateIdentity: (nickname: String) -> Unit,
    modifier: Modifier = Modifier
) {
    var nickname by remember { mutableStateOf("") }

    Column(
        modifier = modifier.padding(32.dp).verticalScroll(rememberScrollState()),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        Text(
            text = stringResource(R.string.identity_not_initialized_title),
            style = MaterialTheme.typography.titleLarge
        )

        Text(
            text = stringResource(R.string.identity_not_initialized_description),
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )

        OutlinedTextField(
            value = nickname,
            onValueChange = { nickname = it },
            label = { Text(stringResource(R.string.identity_label_nickname)) },
            modifier = Modifier.fillMaxWidth(0.8f),
            singleLine = true,
            enabled = !isCreating
        )

        // P0_ANDROID_IDENTITY_PROOF_OF_WORK: Inline progress indicator. The
        // button now mirrors IdentityCreationFlow's pattern: while the FFI call
        // runs (several seconds for Ed25519 keygen + entropy + storage write),
        // the button shows a CircularProgressIndicator + "Generating Identity
        // keys…" text, and the button itself is disabled. Previously this button
        // had no progress indication at all, which made the in-settings
        // identity creation flow feel broken.
        Button(
            onClick = { onCreateIdentity(nickname) },
            enabled = nickname.isNotBlank() && !isCreating,
            modifier = Modifier.fillMaxWidth(0.8f).height(56.dp)
        ) {
            if (isCreating) {
                CircularProgressIndicator(
                    modifier = Modifier.size(20.dp),
                    strokeWidth = 2.dp,
                    color = MaterialTheme.colorScheme.onPrimary
                )
                Spacer(modifier = Modifier.size(8.dp))
                Text(stringResource(R.string.onboarding_generating_keys))
            } else {
                Text(stringResource(R.string.identity_action_create))
            }
        }

        // P0_ANDROID_IDENTITY_PROOF_OF_WORK: render the 6 named proof-of-work
        // stages below the button. Each stage shows:
        //   - [DONE] checkmark (done)
        //   - spinner + label (active)
        //   - dimmed label (pending)
        //   - detail text under the active stage explaining what is happening
        //
        // This is the "high level proof of work" the user asked for. The user
        // can see exactly which step of the cryptographic pipeline is currently
        // running, and the 6 stages match MainViewModel/IdentityViewModel one
        // for one, so onboarding + in-settings have a unified narrative.
        // P0_ANDROID_CRASHFIX: full fix landed in v0.3.4. The previous
        // `progressStage?.let { stage -> ... }` was a defense-in-depth
        // workaround for the NPE; the real fix is to type the StateFlow as
        // non-nullable with Idle as the sentinel. Now the gate is a clean
        // `if (progressStage !is Idle)` — the type system enforces non-null,
        // and the !is Idle check enforces "we are creating right now".
        if (progressStage !is com.scmessenger.android.ui.viewmodels.IdentityProgressStage.Idle) {
            Spacer(modifier = Modifier.height(8.dp))
            IdentityProgressDisplay(
                currentStage = progressStage,
                // P0_ANDROID_PROGRESS_CALLBACK: in-settings create flow uses
                // the same callback plumbing as the onboarding flow, so the
                // sub-stage detail reaches the display here too.
                subStageDetail = progressSubDetail,
                modifier = Modifier.fillMaxWidth(0.95f)
            )
        }
    }
}

/**
 * Renders the 6-stage identity-generation proof-of-work list. Each stage is
 * marked as "done" ([DONE]), "active" (spinner), or "pending" (dimmed), with the
 * current active stage's detail text displayed below the list so the user can
 * see exactly what cryptographic work is happening.
 *
 * Includes:
 *   - a stage counter "Step X of 6" so the user sees numeric progress
 *   - a LinearProgressIndicator driven by the per-stage etaMs so the bar moves
 *     smoothly even when the Rust FFI call blocks (the longest single step)
 *   - a "About Ns remaining" hint that updates as the active stage changes
 *   - a percent-complete number that climbs as stages complete
 */
@Composable
private fun IdentityContent(
    identityInfo: uniffi.api.IdentityInfo,
    qrCodeData: String?,
    error: String?,
    successMessage: String?,
    onClearError: () -> Unit,
    @Suppress("UNUSED_PARAMETER") onClearSuccess: () -> Unit
) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(16.dp)
            .verticalScroll(rememberScrollState()),
        verticalArrangement = Arrangement.spacedBy(24.dp)
    ) {
        // Error banner
        error?.let {
            ErrorBanner(
                message = it,
                onDismiss = onClearError
            )
        }

        // Success message
        successMessage?.let {
            Card(
                colors = CardDefaults.cardColors(
                    containerColor = MaterialTheme.colorScheme.primaryContainer
                )
            ) {
                Text(
                    text = it,
                    modifier = Modifier.padding(16.dp),
                    color = MaterialTheme.colorScheme.onPrimaryContainer
                )
            }
        }

        // Identicon
        IdenticonFromPeerId(
            peerId = identityInfo.libp2pPeerId ?: identityInfo.identityId ?: stringResource(R.string.identity_field_unavailable),
            size = 96.dp,
            modifier = Modifier.align(Alignment.CenterHorizontally)
        )

        // QR Code
        if (qrCodeData != null) {
            com.scmessenger.android.ui.components.QrCodeImage(
                data = qrCodeData!!,
                contentDescription = stringResource(R.string.identity_label_qr_code),
                modifier = Modifier.align(Alignment.CenterHorizontally)
            )
        } else {
            Box(
                modifier = Modifier
                    .size(256.dp)
                    .align(Alignment.CenterHorizontally),
                contentAlignment = Alignment.Center
            ) {
                CircularProgressIndicator()
            }
        }

        // Identity Hash (human fingerprint)
        Card {
            Column(modifier = Modifier.padding(16.dp)) {
                Text(
                    text = stringResource(R.string.identity_label_hash),
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold
                )

                Spacer(modifier = Modifier.height(8.dp))

                CopyableText(
                    text = identityInfo.identityId ?: stringResource(R.string.identity_field_unavailable),
                    monospace = true
                )
            }
        }

        // Peer ID (Network) — libp2p Peer ID for contact add / routing
        Card {
            Column(modifier = Modifier.padding(16.dp)) {
                Text(
                    text = stringResource(R.string.identity_label_peer_id),
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold
                )

                Spacer(modifier = Modifier.height(8.dp))

                CopyableText(
                    text = identityInfo.libp2pPeerId ?: stringResource(R.string.identity_field_unavailable),
                    monospace = true
                )
            }
        }

        // Public Key (canonical identity)
        Card {
            Column(modifier = Modifier.padding(16.dp)) {
                Text(
                    text = stringResource(R.string.identity_label_public_key),
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold
                )

                Spacer(modifier = Modifier.height(8.dp))

                CopyableText(
                    text = identityInfo.publicKeyHex ?: stringResource(R.string.identity_field_unavailable),
                    monospace = true
                )
            }
        }
    }
}

