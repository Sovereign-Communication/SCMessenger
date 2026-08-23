package com.scmessenger.android.utils

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.widget.Toast
import androidx.appcompat.app.AlertDialog
import androidx.core.content.IntentCompat
import com.scmessenger.android.R
import com.scmessenger.android.data.MeshRepository
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.launch
import timber.log.Timber

/**
 * BroadcastReceiver for handling share intents.
 *
 * Receives Intent.ACTION_SEND and presents a contact picker
 * to encrypt and queue the shared content as a mesh message.
 *
 * Features:
 * - Text/plain sharing support
 * - Contact picker dialog
 * - Encryption via MeshRepository
 * - Background message queueing
 */
class ShareReceiver : BroadcastReceiver() {

    override fun onReceive(context: Context, intent: Intent) {
        when (intent.action) {
            Intent.ACTION_SEND -> handleSingleShare(context, intent)
            Intent.ACTION_SEND_MULTIPLE -> handleMultipleShare(context, intent)
            else -> Timber.w("Unhandled intent action: ${intent.action}")
        }
    }

    /**
     * Handle single item share (text).
     */
    private fun handleSingleShare(context: Context, intent: Intent) {
        val type = intent.type
        if (type == null) {
            Timber.w("Share intent has no type")
            return
        }

        when {
            type == "text/plain" -> {
                val sharedText = intent.getStringExtra(Intent.EXTRA_TEXT)
                if (sharedText != null) {
                    showContactPicker(context, sharedText)
                } else {
                    Timber.w("Shared text is null")
                }
            }
            else -> {
                Toast.makeText(context, "Unsupported share type: $type", Toast.LENGTH_SHORT).show()
                Timber.w("Unsupported share type: $type")
            }
        }
    }

    /**
     * Handle multi-item shares from ACTION_SEND_MULTIPLE.
     * Supported payloads:
     * - EXTRA_TEXT as ArrayList<CharSequence>
     * - EXTRA_STREAM as ArrayList<Uri> (serialized as URIs in message body)
     */
    private fun handleMultipleShare(context: Context, intent: Intent) {
        val textItems = intent.getCharSequenceArrayListExtra(Intent.EXTRA_TEXT)
            ?.map { it.toString().trim() }
            ?.filter { it.isNotEmpty() }
            .orEmpty()

        val streamItems = IntentCompat.getParcelableArrayListExtra(intent, Intent.EXTRA_STREAM, Uri::class.java)
            ?.map { it.toString() }
            ?.filter { it.isNotBlank() }
            .orEmpty()

        if (textItems.isEmpty() && streamItems.isEmpty()) {
            Toast.makeText(context, "No shareable items found", Toast.LENGTH_SHORT).show()
            Timber.w("ACTION_SEND_MULTIPLE had no supported payloads")
            return
        }

        val content = buildString {
            if (textItems.isNotEmpty()) {
                append("Shared text items:")
                textItems.forEachIndexed { index, item ->
                    append("\n${index + 1}. $item")
                }
            }
            if (streamItems.isNotEmpty()) {
                if (isNotEmpty()) append("\n\n")
                append("Shared attachments (URI references):")
                streamItems.forEachIndexed { index, uri ->
                    append("\n${index + 1}. $uri")
                }
            }
        }

        showContactPicker(context, content)
    }

    /**
     * Show contact picker dialog to select recipient.
     */
    private fun showContactPicker(context: Context, content: String) {
        try {
            val repository = MeshRepository(context.applicationContext)
            val contacts = repository.listContacts()

            if (contacts.isEmpty()) {
                Toast.makeText(context, "No contacts available", Toast.LENGTH_SHORT).show()
                Timber.w("No contacts to share with")
                return
            }

            val contactNames = contacts.map { contact ->
                contact.nickname ?: contact.peerId.take(8)
            }.toTypedArray()

            // NOTE: Showing AlertDialog from BroadcastReceiver context can fail on newer
            // Android versions due to background UI restrictions. For production, this should
            // launch a transparent Activity to handle the share flow.
            // For now, attempt dialog but catch WindowManager.BadTokenException
            try {
                AlertDialog.Builder(context)
                    .setTitle("Share to Contact")
                    .setItems(contactNames) { dialog, which ->
                        val selectedContact = contacts[which]
                        sendMessageToContact(context, repository, selectedContact.peerId, content)
                        dialog.dismiss()
                    }
                    .setNegativeButton(R.string.cancel) { dialog, _ ->
                        dialog.dismiss()
                    }
                    .show()
            } catch (e: android.view.WindowManager.BadTokenException) {
                Timber.w("Cannot show dialog from BroadcastReceiver context, using toast fallback")
                Toast.makeText(
                    context,
                    "Open SCMessenger to share content with contacts",
                    Toast.LENGTH_LONG
                ).show()
            }
        } catch (e: Exception) {
            Timber.e(e, "Failed to show contact picker")
            Toast.makeText(context, "Failed to load contacts", Toast.LENGTH_SHORT).show()
        }
    }

    /**
     * Encrypt and queue message via MeshRepository.
     */
    private fun sendMessageToContact(
        context: Context,
        repository: MeshRepository,
        peerId: String,
        content: String
    ) {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
        scope.launch {
            try {
                repository.sendMessage(peerId, content)
                Toast.makeText(context, "Message queued for sending", Toast.LENGTH_SHORT).show()
                Timber.i("Shared message queued for $peerId")
            } catch (e: Exception) {
                Timber.e(e, "Failed to send shared message")
                Toast.makeText(context, "Failed to send message", Toast.LENGTH_SHORT).show()
            } finally {
                scope.cancel()
            }
        }
    }
}
