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
import kotlinx.coroutines.withContext
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
                Toast.makeText(context, context.getString(R.string.share_error_unsupported_type, type), Toast.LENGTH_SHORT).show()
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
            Toast.makeText(context, context.getString(R.string.share_error_no_items), Toast.LENGTH_SHORT).show()
            Timber.w("ACTION_SEND_MULTIPLE had no supported payloads")
            return
        }

        val content = buildString {
            if (textItems.isNotEmpty()) {
                append(context.getString(R.string.share_text_items_header))
                textItems.forEachIndexed { index, item ->
                    append("\n${index + 1}. $item")
                }
            }
            if (streamItems.isNotEmpty()) {
                if (isNotEmpty()) append("\n\n")
                append(context.getString(R.string.share_attachments_header))
                streamItems.forEachIndexed { index, uri ->
                    append("\n${index + 1}. $uri")
                }
            }
        }

        showContactPicker(context, content)
    }

    /**
     * Show contact picker dialog to select recipient.
     * HANG-MAIN-001: onReceive runs on main. MeshRepository construction and
     * listContacts() FFI must never run there — goAsync + IO, then post the dialog.
     */
    private fun showContactPicker(context: Context, content: String) {
        val pendingResult = goAsync()
        val appContext = context.applicationContext
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
        scope.launch {
            try {
                val repository = MeshRepository(appContext)
                val contacts = withContext(Dispatchers.IO) { repository.listContacts() }

                if (contacts.isEmpty()) {
                    withContext(Dispatchers.Main) {
                        Toast.makeText(appContext, appContext.getString(R.string.share_error_no_contacts), Toast.LENGTH_SHORT).show()
                    }
                    Timber.w("No contacts to share with")
                    return@launch
                }

                val contactNames = contacts.map { contact ->
                    contact.nickname ?: contact.peerId.take(8)
                }.toTypedArray()

                withContext(Dispatchers.Main) {
                    try {
                        AlertDialog.Builder(appContext)
                            .setTitle(R.string.share_title_share_to_contact)
                            .setItems(contactNames) { dialog, which ->
                                val selectedContact = contacts[which]
                                sendMessageToContact(appContext, repository, selectedContact.peerId, content)
                                dialog.dismiss()
                            }
                            .setNegativeButton(R.string.cancel) { dialog, _ ->
                                dialog.dismiss()
                            }
                            .show()
                    } catch (e: android.view.WindowManager.BadTokenException) {
                        Timber.w("Cannot show dialog from BroadcastReceiver context, using toast fallback")
                        Toast.makeText(
                            appContext,
                            appContext.getString(R.string.share_toast_open_app_fallback),
                            Toast.LENGTH_LONG
                        ).show()
                    }
                }
            } catch (e: Exception) {
                Timber.e(e, "Failed to show contact picker")
                withContext(Dispatchers.Main) {
                    Toast.makeText(
                        appContext,
                        appContext.getString(R.string.share_error_failed_to_load_contacts),
                        Toast.LENGTH_SHORT
                    ).show()
                }
            } finally {
                pendingResult.finish()
                scope.cancel()
            }
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
                withContext(Dispatchers.Main) {
                    Toast.makeText(context, context.getString(R.string.share_toast_message_queued), Toast.LENGTH_SHORT).show()
                }
                Timber.i("Shared message queued for $peerId")
            } catch (e: Exception) {
                Timber.e(e, "Failed to send shared message")
                withContext(Dispatchers.Main) {
                    Toast.makeText(context, context.getString(R.string.share_error_failed_to_send), Toast.LENGTH_SHORT).show()
                }
            } finally {
                scope.cancel()
            }
        }
    }
}
