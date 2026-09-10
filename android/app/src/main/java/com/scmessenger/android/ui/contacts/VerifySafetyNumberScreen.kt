package com.scmessenger.android.ui.contacts

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.scmessenger.android.R
import com.scmessenger.android.ui.components.QrCodeImage
import com.scmessenger.android.ui.theme.StatusOffline
import com.scmessenger.android.ui.theme.StatusOnline
import com.scmessenger.android.ui.viewmodels.ContactsViewModel
import com.scmessenger.android.utils.displayName
import com.scmessenger.android.utils.formatAsDateTime

/**
 * Safety-number verification screen: derives a Signal-style numeric
 * fingerprint from our identity and the contact's public key, so the user
 * can compare it with the contact out-of-band (in person, over a trusted
 * channel) before marking the contact as verified.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun VerifySafetyNumberScreen(
    contactId: String,
    onNavigateBack: () -> Unit,
    viewModel: ContactsViewModel = hiltViewModel()
) {
    val contacts by viewModel.contacts.collectAsState()
    val contact = remember(contacts, contactId) {
        contacts.find { it.peerId == contactId }
    }
    // T9: collect identity readiness as state so the safetyNumberRaw memo
    // below re-computes once the identity initializes after first
    // composition, instead of caching a pre-identity null forever.
    val identityInfo by viewModel.identityInfo.collectAsState()

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(stringResource(R.string.verify_safety_number_title)) },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = stringResource(R.string.chat_action_dismiss))
                    }
                }
            )
        }
    ) { paddingValues ->
        if (contact == null) {
            Box(modifier = Modifier.fillMaxSize().padding(paddingValues)) {
                Text(
                    text = stringResource(R.string.contact_detail_not_found),
                    modifier = Modifier.align(Alignment.Center).padding(32.dp)
                )
            }
            return@Scaffold
        }

        val safetyNumberRaw = remember(contact.publicKey, identityInfo) {
            viewModel.computeSafetyNumber(contact.publicKey)
        }
        val displayName = contact.displayName(contact.peerId.take(16))

        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
                .verticalScroll(rememberScrollState())
                .padding(16.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            if (safetyNumberRaw == null) {
                Text(
                    text = stringResource(R.string.verify_safety_number_error_no_identity),
                    color = MaterialTheme.colorScheme.error
                )
                return@Column
            }
            // S5: Rust's safetyNumber() returns "" (not an all-zero fallback
            // number) for malformed keys - that must be a distinct error
            // state, never rendered or usable to back the verify action.
            if (safetyNumberRaw.isEmpty()) {
                Text(
                    text = stringResource(R.string.verify_safety_number_error_invalid_key_data),
                    color = MaterialTheme.colorScheme.error
                )
                return@Column
            }
            val safetyNumber = safetyNumberRaw

            Text(
                text = stringResource(R.string.verify_safety_number_description, displayName),
                style = MaterialTheme.typography.bodyMedium,
                textAlign = androidx.compose.ui.text.style.TextAlign.Center
            )

            QrCodeImage(
                data = safetyNumber,
                contentDescription = stringResource(R.string.verify_safety_number_qr_content_description)
            )

            Text(
                text = safetyNumber,
                style = MaterialTheme.typography.titleMedium,
                fontFamily = FontFamily.Monospace,
                textAlign = androidx.compose.ui.text.style.TextAlign.Center,
                modifier = Modifier.fillMaxWidth()
            )

            VerificationStatusRow(verifiedAt = contact.verifiedAt)

            if (contact.verifiedAt != null) {
                OutlinedButton(
                    onClick = { viewModel.unverifyContact(contact.peerId) },
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Text(stringResource(R.string.verify_safety_number_action_clear_verification))
                }
            } else {
                Button(
                    onClick = { viewModel.markContactVerified(contact.peerId) },
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Icon(Icons.Default.CheckCircle, contentDescription = null)
                    Spacer(modifier = Modifier.width(8.dp))
                    Text(stringResource(R.string.verify_safety_number_action_mark_verified))
                }
            }
        }
    }
}

@Composable
private fun VerificationStatusRow(verifiedAt: ULong?) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        Icon(
            imageVector = if (verifiedAt != null) Icons.Default.CheckCircle else Icons.Default.Close,
            contentDescription = null,
            tint = if (verifiedAt != null) StatusOnline else StatusOffline
        )
        Text(
            text = verifiedAt?.let {
                stringResource(R.string.verify_safety_number_verified_since, it.formatAsDateTime())
            } ?: stringResource(R.string.contact_detail_status_unverified),
            style = MaterialTheme.typography.bodyMedium,
            fontWeight = FontWeight.Medium
        )
    }
}
