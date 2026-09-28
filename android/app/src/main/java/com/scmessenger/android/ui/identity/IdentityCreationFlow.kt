package com.scmessenger.android.ui.identity

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp
import com.scmessenger.android.R
import com.scmessenger.android.ui.components.EntropyCanvas
import androidx.compose.ui.res.stringResource

/**
 * Shared Identity Creation Flow composable.
 *
 * Provides a consistent UX for identity creation across Settings and Onboarding paths:
 * - Nickname text field
 * - EntropyCanvas (appears after nickname is entered)
 * - Generate button with inline spinner during creation
 * - Re-entrancy protection (button disabled while isCreating)
 */
@Composable
fun IdentityCreationFlow(
    isCreating: Boolean,
    onCreate: (nickname: String, salt: ByteArray?) -> Unit,
    onImport: () -> Unit = {},
    showImportButton: Boolean = false,
    modifier: Modifier = Modifier
) {
    var nickname by remember { mutableStateOf("") }
    var touchEntropySalt by remember { mutableStateOf<ByteArray?>(null) }

    Column(
        modifier = modifier
            .fillMaxWidth()
            .padding(32.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        OutlinedTextField(
            value = nickname,
            onValueChange = {
                nickname = it
                touchEntropySalt = null
            },
            label = { Text(stringResource(R.string.onboarding_label_nickname)) },
            placeholder = { Text(stringResource(R.string.onboarding_placeholder_nickname)) },
            singleLine = true,
            modifier = Modifier
                .fillMaxWidth(0.8f)
                .imePadding()
                .testTag("nickname_field")
        )

        if (nickname.trim().isNotEmpty()) {
            Spacer(modifier = Modifier.height(16.dp))
            EntropyCanvas(
                onEntropyComplete = { salt ->
                    touchEntropySalt = salt
                }
            )
        }

        Spacer(modifier = Modifier.height(12.dp))

        Button(
            onClick = {
                onCreate(nickname.trim(), touchEntropySalt)
            },
            enabled = nickname.trim().isNotEmpty() && touchEntropySalt != null && !isCreating,
            modifier = Modifier
                .fillMaxWidth()
                .height(56.dp)
                .testTag("create_identity_button")
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

        // Optional: Import button (onboarding-only)
        if (showImportButton) {
            Spacer(modifier = Modifier.height(8.dp))
            OutlinedButton(
                onClick = onImport,
                modifier = Modifier.fillMaxWidth().height(56.dp)
            ) {
                Text(stringResource(R.string.onboarding_button_import_join))
            }
        }
    }
}
