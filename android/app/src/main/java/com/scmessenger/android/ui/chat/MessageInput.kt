package com.scmessenger.android.ui.chat

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.Send
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp

import androidx.compose.ui.res.stringResource
import com.scmessenger.android.R

/**
 * Message input component for chat UI.
 *
 * Provides text input field with send button.
 * Handles text composition and sending.
 */
@Composable
fun MessageInput(
    value: String,
    onValueChange: (String) -> Unit,
    onSend: () -> Unit,
    enabled: Boolean = true,
    modifier: Modifier = Modifier
) {
    Surface(
        modifier = modifier.fillMaxWidth(),
        shadowElevation = 8.dp,
        tonalElevation = 3.dp
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp, vertical = 12.dp),
            verticalAlignment = Alignment.Bottom,
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            // Text input field
            OutlinedTextField(
                value = value,
                onValueChange = onValueChange,
                modifier = Modifier
                    .weight(1f)
                    .testTag("message_input"),
                placeholder = { Text(stringResource(R.string.chat_placeholder_type_message)) },
                enabled = enabled,
                maxLines = 5,
                shape = MaterialTheme.shapes.medium
            )

            // Send button
            FloatingActionButton(
                onClick = onSend,
                modifier = Modifier
                    .size(48.dp)
                    .testTag("send_button"),
                containerColor = if (enabled && value.isNotBlank()) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.surfaceVariant,
                contentColor = if (enabled && value.isNotBlank()) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.onSurfaceVariant
            ) {
                Icon(
                    imageVector = Icons.AutoMirrored.Filled.Send,
                    contentDescription = stringResource(R.string.contact_detail_action_send_message),
                    modifier = Modifier.size(24.dp)
                )
            }
        }
    }
}
