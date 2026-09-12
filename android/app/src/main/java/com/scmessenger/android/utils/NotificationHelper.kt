package com.scmessenger.android.utils

import android.Manifest
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationChannelGroup
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.app.Person
import androidx.core.app.RemoteInput
import androidx.core.content.ContextCompat
import androidx.core.graphics.drawable.IconCompat
import com.scmessenger.android.R
import com.scmessenger.android.ui.components.generateIdenticonBitmap
import timber.log.Timber

/**
 * Notification helper for mesh messaging.
 *
 * Features (WS14):
 * - 5 notification channels (Messages/high, Message Requests/high, Mesh Status/low, Peer Events/default, System/low)
 * - DM vs DM Request classification using core notification contract
 * - Grouped message notifications per contact
 * - Reply-from-notification with RemoteInput
 * - Notification actions (Mark Read, Reply, Mute)
 * - Identicon-based contact avatars
 * - Respects DND (Do Not Disturb) settings
 * - Settings parity (notify_dm_enabled, notify_dm_request_enabled, foreground suppression)
 */
object NotificationHelper {

    // Channel IDs
    const val CHANNEL_MESSAGES = "messages"
    const val CHANNEL_MESSAGE_REQUESTS = "message_requests"
    const val CHANNEL_MESH_STATUS = "mesh_status"
    const val CHANNEL_PEER_EVENTS = "peer_events"
    const val CHANNEL_SYSTEM = "system"

    // Channel Group
    private const val GROUP_MESH = "mesh_group"

    // Notification IDs
    const val NOTIFICATION_ID_FOREGROUND_SERVICE = 1001
    private const val NOTIFICATION_ID_MESSAGE_BASE = 2000
    
    // Notification tracking for diagnostics
    private val notificationStats = mutableMapOf<String, Int>(
        "total" to 0,
        "dm" to 0,
        "dm_request" to 0,
        "suppressed_foreground" to 0,
        "suppressed_dnd" to 0,
        "suppressed_settings" to 0
    )
    private const val NOTIFICATION_ID_REQUEST_BASE = 2500
    private const val NOTIFICATION_ID_MESH_STATUS = 3000
    private const val NOTIFICATION_ID_PEER_EVENT = 4000

    // Actions — package-qualified with the applicationId so notification
    // actions share one app identity (was com.scmessenger.*, which looked
    // like a foreign app next to com.scmessenger.android).
    const val ACTION_REPLY = "com.scmessenger.android.ACTION_REPLY"
    const val ACTION_MARK_READ = "com.scmessenger.android.ACTION_MARK_READ"
    const val ACTION_MUTE = "com.scmessenger.android.ACTION_MUTE"
    const val ACTION_OPEN_REQUESTS = "com.scmessenger.android.ACTION_OPEN_REQUESTS"
    const val EXTRA_PEER_ID = "peer_id"
    const val EXTRA_MESSAGE_ID = "message_id"
    const val EXTRA_IS_REQUEST = "is_request"
    const val KEY_REPLY_TEXT = "key_reply_text"

    // Message grouping
    private val messageGroups = mutableMapOf<String, MutableList<NotificationMessage>>()
    private val requestGroups = mutableMapOf<String, MutableList<NotificationMessage>>()

    // Notification settings (defaults per WS14 spec)
    var notificationsEnabled: Boolean = true
    var notifyDmEnabled: Boolean = true
    var notifyDmRequestEnabled: Boolean = true
    var notifyDmInForeground: Boolean = false
    var notifyDmRequestInForeground: Boolean = true
    var soundEnabled: Boolean = true
    var badgeEnabled: Boolean = true

    /**
     * Initialize notification channels on app start.
     * Must be called before posting any notifications.
     * WS14: Added Message Requests channel for DM Request notifications.
     */
    fun createNotificationChannels(context: Context) {
        val notificationManager = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager

        // Create channel group
        val channelGroup = NotificationChannelGroup(GROUP_MESH, context.getString(R.string.notification_channel_group_mesh))
        notificationManager.createNotificationChannelGroup(channelGroup)

        // 1. Messages Channel (HIGH priority) - for known contacts
        val messagesChannel = NotificationChannel(
            CHANNEL_MESSAGES,
            context.getString(R.string.notification_channel_messages),
            NotificationManager.IMPORTANCE_HIGH
        ).apply {
            description = context.getString(R.string.notification_channel_messages_description)
            group = GROUP_MESH
            enableLights(true)
            enableVibration(true)
            setShowBadge(true)
        }

        // 2. Message Requests Channel (HIGH priority) - WS14: for unknown senders
        val messageRequestsChannel = NotificationChannel(
            CHANNEL_MESSAGE_REQUESTS,
            context.getString(R.string.notification_channel_message_requests),
            NotificationManager.IMPORTANCE_HIGH
        ).apply {
            description = context.getString(R.string.notification_channel_message_requests_description)
            group = GROUP_MESH
            enableLights(true)
            enableVibration(true)
            setShowBadge(true)
        }

        // 3. Mesh Status Channel (LOW priority)
        val meshStatusChannel = NotificationChannel(
            CHANNEL_MESH_STATUS,
            context.getString(R.string.notification_channel_mesh_status),
            NotificationManager.IMPORTANCE_LOW
        ).apply {
            description = context.getString(R.string.notification_channel_mesh_status_description)
            group = GROUP_MESH
            enableLights(false)
            enableVibration(false)
            setShowBadge(false)
        }

        // 4. Peer Events Channel (DEFAULT priority)
        val peerEventsChannel = NotificationChannel(
            CHANNEL_PEER_EVENTS,
            context.getString(R.string.notification_channel_peer_events),
            NotificationManager.IMPORTANCE_DEFAULT
        ).apply {
            description = context.getString(R.string.notification_channel_peer_events_description)
            group = GROUP_MESH
            enableLights(false)
            enableVibration(false)
            setShowBadge(false)
        }

        // 5. System Channel (LOW priority)
        val systemChannel = NotificationChannel(
            CHANNEL_SYSTEM,
            context.getString(R.string.notification_channel_system),
            NotificationManager.IMPORTANCE_LOW
        ).apply {
            description = context.getString(R.string.notification_channel_system_description)
            group = GROUP_MESH
            enableLights(false)
            enableVibration(false)
            setShowBadge(false)
        }

        notificationManager.createNotificationChannels(
            listOf(messagesChannel, messageRequestsChannel, meshStatusChannel, peerEventsChannel, systemChannel)
        )

        Timber.d("Notification channels created (WS14: with Message Requests channel)")
    }

    /**
     * Build foreground service notification for MeshForegroundService.
     */
    fun buildForegroundServiceNotification(
        context: Context,
        peerCount: Int,
        relayCount: Int
    ): Notification {
        val intent = context.packageManager.getLaunchIntentForPackage(context.packageName)
        val pendingIntent = if (intent != null) {
            PendingIntent.getActivity(
                context,
                0,
                intent,
                PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
            )
        } else {
            null
        }

        val contentText = context.getString(R.string.notification_mesh_status_content_format, peerCount, relayCount)

        return NotificationCompat.Builder(context, CHANNEL_MESH_STATUS)
            .setContentTitle(context.getString(R.string.mesh_service_notification_title))
            .setContentText(contentText)
            .setSmallIcon(R.drawable.ic_notification)
            .setOngoing(true)
            .setContentIntent(pendingIntent)
            .setSilent(true)
            .build()
    }

    /**
     * Show a new message notification with WS14 DM vs DM Request classification.
     * Groups messages by contact/peerId.
     *
     * @param context Android context
     * @param peerId Sender peer ID
     * @param messageId Message ID
     * @param content Message content
     * @param nickname Sender nickname (null if unknown)
     * @param timestamp Message timestamp
     * @param isKnownContact Whether sender is a known contact
     * @param hasExistingConversation Whether an existing conversation exists
     * @param appInForeground Whether app is in foreground
     * @param activeConversationId Currently active conversation ID (if any)
     * @param explicitDmRequest Explicit DM request flag from message metadata
     */
    fun showMessageNotification(
        context: Context,
        peerId: String,
        messageId: String,
        content: String,
        nickname: String?,
        timestamp: Long,
        isKnownContact: Boolean = false,
        hasExistingConversation: Boolean = false,
        appInForeground: Boolean = false,
        activeConversationId: String? = null,
        explicitDmRequest: Boolean? = null
    ) {
        // Track total notification attempt
        trackNotificationEvent("total")
        
        // Log notification attempt with classification details
        Timber.i("Processing notification - peerId=$peerId, messageId=$messageId, isKnownContact=$isKnownContact, hasExistingConversation=$hasExistingConversation, explicitDmRequest=$explicitDmRequest, appInForeground=$appInForeground")

        // Check global notifications enabled
        if (!notificationsEnabled) {
            trackNotificationEvent("suppressed_settings")
            Timber.w("Notifications disabled globally, skipping notification for peerId=$peerId")
            return
        }

        // Check DND
        if (isDndEnabled(context)) {
            trackNotificationEvent("suppressed_dnd")
            Timber.w("DND enabled, skipping notification for peerId=$peerId")
            return
        }

        // WS14: Classify as DM or DM Request
        val isDmRequest = classifyAsDmRequest(isKnownContact, hasExistingConversation, explicitDmRequest)
        Timber.i("Notification classified as ${if (isDmRequest) "DM_REQUEST" else "DM"} for peerId=$peerId")
        
        if (isDmRequest) {
            trackNotificationEvent("dm_request")
        } else {
            trackNotificationEvent("dm")
        }
        
        // Check per-kind settings
        if (isDmRequest && !notifyDmRequestEnabled) {
            Timber.d("DM Request notifications disabled, skipping")
            return
        }
        if (!isDmRequest && !notifyDmEnabled) {
            Timber.d("DM notifications disabled, skipping")
            return
        }

        // WS14: Foreground suppression
        val isActiveConversation = appInForeground && activeConversationId != null &&
            (activeConversationId == peerId || activeConversationId.equals(peerId, ignoreCase = true))
        
        if (isActiveConversation) {
            val allowForeground = if (isDmRequest) notifyDmRequestInForeground else notifyDmInForeground
            if (!allowForeground) {
                trackNotificationEvent("suppressed_foreground")
                Timber.w("Foreground conversation active, suppressing notification for peerId=$peerId (DM=${!isDmRequest}, DM_REQUEST=$isDmRequest)")
                return
            }
        }

        // Add to appropriate group
        val message = NotificationMessage(messageId, content, timestamp)
        if (isDmRequest) {
            requestGroups.getOrPut(peerId) { mutableListOf() }.add(message)
        } else {
            messageGroups.getOrPut(peerId) { mutableListOf() }.add(message)
        }

        val messages = if (isDmRequest) requestGroups[peerId] else messageGroups[peerId] ?: return
        val displayName = nickname ?: peerId.take(8)

        // Generate identicon for avatar
        val identicon = try {
            generateIdenticonBitmap(peerId.toByteArray(), 128)
        } catch (e: Exception) {
            Timber.e(e, "Failed to generate identicon")
            null
        }

        // Create person for messaging style
        val person = Person.Builder()
            .setName(displayName)
            .apply { identicon?.let { setIcon(IconCompat.createWithBitmap(it)) } }
            .build()

        // Build messaging style notification
        val messagingStyle = NotificationCompat.MessagingStyle(person)
        messages?.forEach { msg ->
            messagingStyle.addMessage(msg.content, msg.timestamp, person)
        }

        // Reply action with RemoteInput (only for DM, not requests)
        val actions = mutableListOf<NotificationCompat.Action>()
        
        if (!isDmRequest) {
            val replyIntent = createReplyIntent(context, peerId, messageId)
            val replyPendingIntent = PendingIntent.getBroadcast(
                context,
                peerId.hashCode(),
                replyIntent,
                PendingIntent.FLAG_MUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
            )

            val remoteInput = RemoteInput.Builder(KEY_REPLY_TEXT)
                .setLabel(context.getString(R.string.notification_action_reply))
                .build()

            val replyAction = NotificationCompat.Action.Builder(
                R.drawable.ic_notification,
                context.getString(R.string.notification_action_reply),
                replyPendingIntent
            )
                .addRemoteInput(remoteInput)
                .setAllowGeneratedReplies(true)
                .build()
            actions.add(replyAction)
        }

        // Mark Read action
        val markReadIntent = createMarkReadIntent(context, peerId, messageId)
        val markReadPendingIntent = PendingIntent.getBroadcast(
            context,
            peerId.hashCode() + 1,
            markReadIntent,
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )

        val markReadAction = NotificationCompat.Action.Builder(
            R.drawable.ic_notification,
            context.getString(R.string.notification_action_mark_read),
            markReadPendingIntent
        ).build()
        actions.add(markReadAction)

        // Mute action
        val muteIntent = createMuteIntent(context, peerId)
        val mutePendingIntent = PendingIntent.getBroadcast(
            context,
            peerId.hashCode() + 2,
            muteIntent,
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )

        val muteAction = NotificationCompat.Action.Builder(
            R.drawable.ic_notification,
            context.getString(R.string.notification_action_mute),
            mutePendingIntent
        ).build()
        actions.add(muteAction)

        // WS14: Tap routing - DM goes to chat, DM Request goes to requests inbox
        val tapIntent = if (isDmRequest) {
            createOpenRequestsIntent(context, peerId, messageId)
        } else {
            context.packageManager.getLaunchIntentForPackage(context.packageName)?.apply {
                putExtra(EXTRA_PEER_ID, peerId)
            }
        }
        val tapPendingIntent = if (tapIntent != null) {
            PendingIntent.getActivity(
                context,
                peerId.hashCode(),
                tapIntent,
                PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
            )
        } else {
            null
        }

        // WS14: Use appropriate channel
        val channelId = if (isDmRequest) CHANNEL_MESSAGE_REQUESTS else CHANNEL_MESSAGES
        val category = if (isDmRequest) NotificationCompat.CATEGORY_MESSAGE else NotificationCompat.CATEGORY_MESSAGE
        val title = if (isDmRequest) context.getString(R.string.notification_message_request_title, displayName) else null

        // Build notification
        val notification = NotificationCompat.Builder(context, channelId)
            .setStyle(messagingStyle)
            .setSmallIcon(R.drawable.ic_notification)
            .setGroup(peerId)
            .setAutoCancel(true)
            .setContentIntent(tapPendingIntent)
            .apply {
                actions.forEach { addAction(it) }
                if (title != null) setContentTitle(title)
                if (soundEnabled) setDefaults(NotificationCompat.DEFAULT_SOUND)
                if (badgeEnabled) setBadgeIconType(NotificationCompat.BADGE_ICON_SMALL)
            }
            .setPriority(NotificationCompat.PRIORITY_HIGH)
            .setCategory(category)
            .build()

        if (!hasNotificationPermission(context)) {
            Timber.w("POST_NOTIFICATIONS permission missing; skipping message notification")
            return
        }
        val notificationId = if (isDmRequest) {
            NOTIFICATION_ID_REQUEST_BASE + peerId.hashCode()
        } else {
            NOTIFICATION_ID_MESSAGE_BASE + peerId.hashCode()
        }
        try {
            Timber.i("Displaying notification - peerId=$peerId, notificationId=$notificationId, type=${if (isDmRequest) "DM_REQUEST" else "DM"}")
            NotificationManagerCompat.from(context).notify(notificationId, notification)
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception while posting message notification for peerId=$peerId")
            return
        }

        Timber.d("${if (isDmRequest) "DM Request" else "DM"} notification shown for $peerId")
    }

    /**
     * WS14: Classify message as DM Request based on contact state.
     *
     * Rules:
     * 1. If explicit_dm_request is true -> DM Request
     * 2. If sender is known contact OR has existing conversation -> DM
     * 3. Otherwise -> DM Request
     */
    private fun classifyAsDmRequest(
        isKnownContact: Boolean,
        hasExistingConversation: Boolean,
        explicitDmRequest: Boolean?
    ): Boolean {
        // Explicit request flag overrides inference
        if (explicitDmRequest == true) {
            return true
        }
        // Known contact or existing conversation = DM
        if (isKnownContact || hasExistingConversation) {
            return false
        }
        // Unknown sender = DM Request
        return true
    }

    /**
     * Create intent to open requests inbox for DM Request notifications.
     */
    private fun createOpenRequestsIntent(context: Context, peerId: String, messageId: String): Intent? {
        return context.packageManager.getLaunchIntentForPackage(context.packageName)?.apply {
            putExtra(EXTRA_PEER_ID, peerId)
            putExtra(EXTRA_MESSAGE_ID, messageId)
            putExtra(EXTRA_IS_REQUEST, true)
            // Add flag to navigate to requests inbox
            action = ACTION_OPEN_REQUESTS
        }
    }

    /**
     * Clear message notifications for a specific peer.
     * WS14: Also clears request notifications.
     */
    fun clearMessageNotifications(context: Context, peerId: String) {
        messageGroups.remove(peerId)
        requestGroups.remove(peerId)
        val notificationId = NOTIFICATION_ID_MESSAGE_BASE + peerId.hashCode()
        val requestId = NOTIFICATION_ID_REQUEST_BASE + peerId.hashCode()
        NotificationManagerCompat.from(context).cancel(notificationId)
        NotificationManagerCompat.from(context).cancel(requestId)
        Timber.d("Cleared notifications for $peerId (DM + Request)")
    }

    /**
     * WS14: Clear all request notifications.
     */
    fun clearAllRequestNotifications() {
        requestGroups.clear()
        // Cancel all request notifications (approximate - in production would track IDs)
        Timber.d("Cleared all request notifications")
    }

    /**
     * Show peer discovery notification.
     */
    fun showPeerDiscoveredNotification(
        context: Context,
        peerId: String,
        transport: String
    ) {
        // Gate: honor the global notifications toggle (was DND-only).
        if (!notificationsEnabled) {
            trackNotificationEvent("suppressed_settings")
            Timber.d("Notifications disabled globally, skipping peer-discovered for peerId=$peerId")
            return
        }
        if (isDndEnabled(context)) return

        val notification = NotificationCompat.Builder(context, CHANNEL_PEER_EVENTS)
            .setContentTitle(context.getString(R.string.notification_peer_discovered_title))
            .setContentText(context.getString(R.string.notification_peer_discovered_format, peerId, transport))
            .setSmallIcon(R.drawable.ic_notification)
            .setAutoCancel(true)
            .setPriority(NotificationCompat.PRIORITY_DEFAULT)
            .build()

        if (!hasNotificationPermission(context)) {
            Timber.w("POST_NOTIFICATIONS permission missing; skipping peer discovered notification")
            return
        }
        try {
            NotificationManagerCompat.from(context).notify(
                NOTIFICATION_ID_PEER_EVENT + peerId.hashCode(),
                notification
            )
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception while posting peer discovered notification")
        }
    }

    // Helper methods

    private fun createReplyIntent(context: Context, peerId: String, messageId: String): Intent {
        return Intent(ACTION_REPLY).apply {
            setPackage(context.packageName)
            putExtra(EXTRA_PEER_ID, peerId)
            putExtra(EXTRA_MESSAGE_ID, messageId)
        }
    }

    private fun createMarkReadIntent(context: Context, peerId: String, messageId: String): Intent {
        return Intent(ACTION_MARK_READ).apply {
            setPackage(context.packageName)
            putExtra(EXTRA_PEER_ID, peerId)
            putExtra(EXTRA_MESSAGE_ID, messageId)
        }
    }

    private fun createMuteIntent(context: Context, peerId: String): Intent {
        return Intent(ACTION_MUTE).apply {
            setPackage(context.packageName)
            putExtra(EXTRA_PEER_ID, peerId)
        }
    }

    /**
     * WS14: Update notification settings.
     */
    fun updateSettings(
        enabled: Boolean? = null,
        dmEnabled: Boolean? = null,
        dmRequestEnabled: Boolean? = null,
        dmInForeground: Boolean? = null,
        dmRequestInForeground: Boolean? = null,
        sound: Boolean? = null,
        badge: Boolean? = null
    ) {
        enabled?.let { notificationsEnabled = it }
        dmEnabled?.let { notifyDmEnabled = it }
        dmRequestEnabled?.let { notifyDmRequestEnabled = it }
        dmInForeground?.let { notifyDmInForeground = it }
        dmRequestInForeground?.let { notifyDmRequestInForeground = it }
        sound?.let { soundEnabled = it }
        badge?.let { badgeEnabled = it }
        Timber.d("Notification settings updated")
    }

    private fun isDndEnabled(context: Context): Boolean {
        val notificationManager = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        return notificationManager.currentInterruptionFilter != NotificationManager.INTERRUPTION_FILTER_ALL
    }
    
    /**
     * Track notification event for diagnostics
     */
    private fun trackNotificationEvent(type: String) {
        notificationStats[type] = (notificationStats[type] ?: 0) + 1
        Timber.d("Notification event tracked: $type (total: ${notificationStats[type]})")
    }
    
    /**
     * Get notification statistics summary
     */
    fun getNotificationStats(): String {
        return notificationStats.entries.joinToString(", ") { (key, value) -> "$key=$value" }
    }
    
    /**
     * Reset notification statistics
     */
    fun resetNotificationStats() {
        notificationStats.keys.forEach { notificationStats[it] = 0 }
        Timber.d("Notification statistics reset")
    }

    private fun hasNotificationPermission(context: Context): Boolean {
        return Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU ||
            ContextCompat.checkSelfPermission(
                context,
                Manifest.permission.POST_NOTIFICATIONS
            ) == PackageManager.PERMISSION_GRANTED
    }

    /**
     * Data class for grouping messages.
     */
    private data class NotificationMessage(
        val messageId: String,
        val content: String,
        val timestamp: Long
    )
}
