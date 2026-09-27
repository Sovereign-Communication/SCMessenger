# BATCH: Android Group E  Kotlin wiring (8 tasks)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# AGENT: implementer
# MODEL: qwen3-coder-next:cloud
# FALLBACK: glm-5.1:cloud
# TARGET FILES: android/app/src/main/java/com/scmessenger/android/ (multiple)

1. **clearSearch**  Wire into ContactsViewModel search state reset and UI back-navigation.
2. **isAtMaxDelay**  Wire into BackoffStrategy exponential backoff cap and retry scheduling.
3. **clearAllHistory**  Wire into ConversationsViewModel history purge and local database cleanup.
4. **clearAnrEvents**  Wire into PerformanceMonitor ANR event log rotation and memory pressure handler.
5. **clearInput**  Wire into ChatViewModel message input reset and draft clearing on send.
6. **clearAllRequestNotifications**  Wire into NotificationHelper request notification channel cleanup.
7. **resolveDeliveryState**  Wire into ConversationsViewModel message status resolution and read receipt display.
8. **clearMessageNotifications**  Wire into NotificationHelper message notification dismissal on conversation open.

CRITICAL: You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.