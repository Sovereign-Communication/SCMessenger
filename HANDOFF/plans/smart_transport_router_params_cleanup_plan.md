# Design Plan: SmartTransportRouter & ConversationItem Parameters Cleanup/Wiring

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This design plan addresses two overlapping tickets regarding unused and suppressed parameters in the Android transport and UI modules:
1. `ANDROID_SWEEP_02_NEEDS_PLANNING_smart_transport_router_unused_params.md`
2. `P3_ANDROID_NEEDS_PLANNING_SMARTTRANSPORTROUTER_DEAD_PARAMS.md`

---

## 1. Overview of Unused Parameters

### 1.1 `SmartTransportRouter.attemptDelivery`
The racing router function `attemptDelivery` in [SmartTransportRouter.kt](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt) currently suppresses warnings for four parameters:
- `envelopeData: ByteArray`: The actual encrypted message data. The racing lambdas passed from `MeshRepository.kt` capture this payload via lexical closure rather than receiving it from the router.
- `listeners: List<String>`: Direct dial address multiaddrs.
- `traceMessageId: String?`: Context trace ID for tracing delivery.
- `attemptContext: String?`: Tracing scope/context.

The single call site in [MeshRepository.kt](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt#L5975) passes valid data for all of these parameters.

### 1.2 `ConversationItem`
The Composable function `ConversationItem` in [ConversationsScreen.kt](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/android/ui/screens/ConversationsScreen.kt#L265) suppresses warnings for:
- `deliveryState: DeliveryStatePresentation`: Intended to display the state of the conversation's latest message.
- `onNavigateToChat: (String) -> Unit`: Intended to trigger navigation, but the clickable modifier in `ConversationItem` uses a separate `onClick` lambda wrapper.

---

## 2. Option A: Clean Pruning (Dead Code Removal)

This option removes all unused parameters from the method/composable signatures and updates all call sites to clean up the API surface.

### 2.1 Code Diffs for Option A

#### [SmartTransportRouter.kt](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt)
```diff
--- android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt
+++ android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt
@@ -296,11 +296,7 @@
     suspend fun attemptDelivery(
         peerId: String,
-        @Suppress("UNUSED_PARAMETER") envelopeData: ByteArray,
         wifiPeerId: String?,
         blePeerId: String?,
         tcpMdnsPeerId: String?,
         routePeerCandidates: List<String>,
-        @Suppress("UNUSED_PARAMETER") listeners: List<String>,
-        @Suppress("UNUSED_PARAMETER") traceMessageId: String?,
-        @Suppress("UNUSED_PARAMETER") attemptContext: String?,
         tryWifi: suspend (String) -> Boolean,
         tryBle: suspend (String) -> Boolean,
         tryTcpMdns: suspend (String) -> Boolean,
         tryCore: suspend (String) -> Boolean
     ): TransportDeliveryResult {
```

#### [MeshRepository.kt](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt)
```diff
--- android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt
+++ android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt
@@ -5975,15 +5975,11 @@
             smartTransportRouter!!.attemptDelivery(
                 peerId = routePeerFallback,
-                envelopeData = encryptedData,
                 wifiPeerId = if (strictBleOnly) null else wifiPeerId,
                 blePeerId = effectiveBlePeerId,
                 tcpMdnsPeerId = routePeerCandidates
                     .firstOrNull { candidate ->
                         val trimmed = candidate.trim()
                         mdnsLanPeers[trimmed]?.isNotEmpty() == true
                     }
                     ?.trim(),
                 routePeerCandidates = routePeerCandidates,
-                listeners = listeners,
-                traceMessageId = traceMessageId,
-                attemptContext = attemptContext,
                 tryWifi = { wifiId ->
```

#### [ConversationsScreen.kt](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/android/app/src/main/java/com/scmessenger/android/ui/screens/ConversationsScreen.kt)
```diff
--- android/app/src/main/java/com/scmessenger/android/ui/screens/ConversationsScreen.kt
+++ android/app/src/main/java/com/scmessenger/android/ui/screens/ConversationsScreen.kt
@@ -163,14 +163,12 @@
                         ConversationItem(
                             displayName = displayName,
                             peerId = peerId,
                             messages = messages,
                             onClick = {
                                 onNavigateToChat(peerId)
                             },
                             onRequestDelete = {
                                 conversationToDelete = peerId to messages
                                 showDeleteDialog = true
-                            },
-                            deliveryState = viewModel.resolveDeliveryState(messages.first()),
-                            onNavigateToChat = onNavigateToChat
+                            }
                         )
                         Spacer(modifier = Modifier.height(8.dp))
                     }
                 }
             }
         }
     }
 
     if (showClearHistoryDialog) {
@@ -265,9 +263,7 @@
 fun ConversationItem(
     displayName: String,
     peerId: String,
     messages: List<uniffi.api.MessageRecord>,
     onClick: () -> Unit,
-    onRequestDelete: () -> Unit,
-    @Suppress("UNUSED_PARAMETER") deliveryState: DeliveryStatePresentation = DeliveryStatePresentation(DeliveryStateSurface.PENDING, "pending", ""),
-    @Suppress("UNUSED_PARAMETER") onNavigateToChat: (String) -> Unit = {},
+    onRequestDelete: () -> Unit
 ) {
```

---

## 3. Option B: Wiring for Logging and Functionality

This option wires the unused parameters for logging correlation, payload verification, routing fallback, and UI status reporting.

### 3.1 Routing and Tracing Features
1. **Log Context Prefixing**: Prepend `[traceId=..., ctx=...]` context metadata to all of `SmartTransportRouter`'s internal `Timber` statements, allowing direct cross-referencing with `MeshRepository` delivery trails in logcat.
2. **Payload Size Verification**: Intercept empty/invalid byte payloads before performing parallel racing.
3. **Core Direct Listeners Fallback**: If `routePeerCandidates` is empty, use the `listeners` addresses to dial the recipient node (`peerId`) directly through the `tryCore` transport.
4. **Visual UI Delivery State**: Use `deliveryState` inside `ConversationItem` to display an inline indicator for sent messages (e.g. "Delivered", "Pending", "Failed").
5. **Unified Navigation Signature**: Streamline Composable events by substituting `onClick` with `onNavigateToChat(peerId)`.

### 3.2 Code Diffs for Option B

#### [SmartTransportRouter.kt](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt)
```diff
--- android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt
+++ android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt
@@ -296,15 +296,27 @@
     suspend fun attemptDelivery(
         peerId: String,
-        @Suppress("UNUSED_PARAMETER") envelopeData: ByteArray,
+        envelopeData: ByteArray,
         wifiPeerId: String?,
         blePeerId: String?,
         tcpMdnsPeerId: String?,
         routePeerCandidates: List<String>,
-        @Suppress("UNUSED_PARAMETER") listeners: List<String>,
-        @Suppress("UNUSED_PARAMETER") traceMessageId: String?,
-        @Suppress("UNUSED_PARAMETER") attemptContext: String?,
+        listeners: List<String>,
+        traceMessageId: String?,
+        attemptContext: String?,
         tryWifi: suspend (String) -> Boolean,
         tryBle: suspend (String) -> Boolean,
         tryTcpMdns: suspend (String) -> Boolean,
         tryCore: suspend (String) -> Boolean
     ): TransportDeliveryResult {
+        val logCtx = "[traceId=${traceMessageId?.take(8) ?: "none"}, ctx=${attemptContext ?: "none"}]"
+
+        if (envelopeData.isEmpty()) {
+            Timber.tag(TAG).w("$logCtx Rejected delivery attempt: empty envelope data")
+            return TransportDeliveryResult(
+                transport = TransportType.CORE,
+                success = false,
+                latencyMs = 0,
+                error = "empty_envelope_data"
+            )
+        }
+
         val startTime = System.currentTimeMillis()
 
         // Determine available transports
         data class TransportAttempt(
             val type: TransportType,
             val target: String,
             val attempt: suspend () -> Boolean
         )
 
         val availableTransports = mutableListOf<TransportAttempt>()
 
         val wifiTarget = wifiPeerId?.trim()?.takeIf { it.isNotEmpty() }
         if (wifiTarget != null) {
             availableTransports.add(TransportAttempt(TransportType.WIFI_DIRECT, wifiTarget) { tryWifi(wifiTarget) })
         }
 
         val bleTarget = blePeerId?.trim()?.takeIf { it.isNotEmpty() }
         if (bleTarget != null) {
             availableTransports.add(TransportAttempt(TransportType.BLE, bleTarget) { tryBle(bleTarget) })
         }
 
         val tcpMdnsTarget = tcpMdnsPeerId?.trim()?.takeIf { it.isNotEmpty() }
         if (tcpMdnsTarget != null) {
             availableTransports.add(TransportAttempt(TransportType.TCP_MDNS, tcpMdnsTarget) { tryTcpMdns(tcpMdnsTarget) })
         }
 
-        val coreTarget = routePeerCandidates.firstOrNull()?.trim()?.takeIf { it.isNotEmpty() }
+        // Fallback: If no route candidates are found but direct listener addresses exist, attempt dialing peer via core
+        var coreTarget = routePeerCandidates.firstOrNull()?.trim()?.takeIf { it.isNotEmpty() }
+        if (coreTarget == null && listeners.isNotEmpty()) {
+            coreTarget = peerId.trim().takeIf { it.isNotEmpty() }
+            Timber.tag(TAG).i("$logCtx No route candidates, using listeners direct fallback for $coreTarget")
+        }
         if (coreTarget != null) {
             availableTransports.add(TransportAttempt(TransportType.CORE, coreTarget) { tryCore(coreTarget) })
         }
 
         if (availableTransports.isEmpty()) {
-            Timber.tag(TAG).w("No available transports for peer ${peerId.take(8)}")
+            Timber.tag(TAG).w("$logCtx No available transports for peer ${peerId.take(8)}")
             return TransportDeliveryResult(
                 transport = TransportType.CORE,
                 success = false,
                 latencyMs = 0,
                 error = "no_available_transports"
             )
         }
 
         // Get preferred transport
         val preferredTransport = getPreferredTransport(peerId)
 
         // If we have a preferred transport, try it first with timeout
         if (preferredTransport != null) {
             val preferredAttempt = availableTransports.find { it.type == preferredTransport }
             if (preferredAttempt != null) {
-                Timber.tag(TAG).i("Trying preferred transport ${preferredTransport.value} for peer ${peerId.take(8)}")
+                Timber.tag(TAG).i("$logCtx Trying preferred transport ${preferredTransport.value} for peer ${peerId.take(8)}")
 
                 // Race preferred transport against timeout
                 val preferredResult = withTimeoutOrNull(PREFERRED_TRANSPORT_TIMEOUT_MS) {
                     preferredAttempt.attempt()
                 }
 
                 if (preferredResult == true) {
                     val latencyMs = System.currentTimeMillis() - startTime
                     recordSuccess(peerId, preferredTransport, latencyMs)
-                    Timber.tag(TAG).i("[OK] Preferred transport ${preferredTransport.value} succeeded in ${latencyMs}ms")
+                    Timber.tag(TAG).i("$logCtx [OK] Preferred transport ${preferredTransport.value} succeeded in ${latencyMs}ms")
                     return TransportDeliveryResult(
                         transport = preferredTransport,
                         success = true,
                         latencyMs = latencyMs,
                         error = null
                     )
                 }
 
                 // Preferred transport failed or timed out - race all transports
-                Timber.tag(TAG).w("Preferred transport ${preferredTransport.value} failed/timed out, racing all transports")
+                Timber.tag(TAG).w("$logCtx Preferred transport ${preferredTransport.value} failed/timed out, racing all transports")
             }
         }
 
         // Race all available transports in parallel
-        Timber.tag(TAG).i("Racing ${availableTransports.count()} transports for peer ${peerId.take(8)}")
+        Timber.tag(TAG).i("$logCtx Racing ${availableTransports.count()} transports for peer ${peerId.take(8)}")
 
         val result = coroutineScope {
             val deferreds = availableTransports.map { transportAttempt ->
                 async {
                     val transportStart = System.currentTimeMillis()
                     val success = try {
                         transportAttempt.attempt()
                     } catch (e: Exception) {
-                        Timber.tag(TAG).w(e, "Transport ${transportAttempt.type.value} failed")
+                        Timber.tag(TAG).w(e, "$logCtx Transport ${transportAttempt.type.value} failed")
                         false
                     }
                     val latencyMs = System.currentTimeMillis() - transportStart
                     Triple(transportAttempt.type, success, latencyMs)
                 }
             }
 
             // Wait for first successful result
             var firstSuccess: Triple<TransportType, Boolean, Long>? = null
             for (deferred in deferreds) {
                 val result = deferred.await()
                 if (result.second) {
                     firstSuccess = result
                     break
                 }
             }
 
             // Cancel remaining coroutines
             deferreds.forEach { it.cancel() }
 
             firstSuccess
         }
 
         return if (result != null) {
             recordSuccess(peerId, result.first, result.third)
-            Timber.tag(TAG).i("[OK] Transport ${result.first.value} succeeded in ${result.third}ms")
+            Timber.tag(TAG).i("$logCtx [OK] Transport ${result.first.value} succeeded in ${result.third}ms")
             TransportDeliveryResult(
                 transport = result.first,
                 success = true,
                 latencyMs = result.third,
                 error = null
             )
         } else {
             // All transports failed
             val latencyMs = System.currentTimeMillis() - startTime
             availableTransports.forEach { transportAttempt ->
                 recordFailure(peerId, transportAttempt.type, "all_transports_failed")
             }
-            Timber.tag(TAG).e("[FAIL] All transports failed for peer ${peerId.take(8)}")
+            Timber.tag(TAG).e("$logCtx [FAIL] All transports failed for peer ${peerId.take(8)}")
             TransportDeliveryResult(
                 transport = TransportType.CORE,
                 success = false,
                 latencyMs = latencyMs,
                 error = "all_transports_failed"
             )
         }
```

#### [ConversationsScreen.kt](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/android/app/src/main/java/com/scmessenger/android/ui/screens/ConversationsScreen.kt)
```diff
--- android/app/src/main/java/com/scmessenger/android/ui/screens/ConversationsScreen.kt
+++ android/app/src/main/java/com/scmessenger/android/ui/screens/ConversationsScreen.kt
@@ -163,14 +163,10 @@
                         ConversationItem(
                             displayName = displayName,
                             peerId = peerId,
                             messages = messages,
-                            onClick = {
-                                onNavigateToChat(peerId)
-                            },
+                            onNavigateToChat = onNavigateToChat,
                             onRequestDelete = {
                                 conversationToDelete = peerId to messages
                                 showDeleteDialog = true
                             },
                             deliveryState = viewModel.resolveDeliveryState(messages.first()),
-                            onNavigateToChat = onNavigateToChat
                         )
                         Spacer(modifier = Modifier.height(8.dp))
                     }
                 }
             }
         }
     }
 
     if (showClearHistoryDialog) {
@@ -265,9 +261,7 @@
 fun ConversationItem(
     displayName: String,
     peerId: String,
     messages: List<uniffi.api.MessageRecord>,
-    onClick: () -> Unit,
+    onNavigateToChat: (String) -> Unit,
     onRequestDelete: () -> Unit,
-    @Suppress("UNUSED_PARAMETER") deliveryState: DeliveryStatePresentation = DeliveryStatePresentation(DeliveryStateSurface.PENDING, "pending", ""),
-    @Suppress("UNUSED_PARAMETER") onNavigateToChat: (String) -> Unit = {},
+    deliveryState: DeliveryStatePresentation
 ) {
     val lastMessage = messages.firstOrNull() ?: return
     val undeliveredCount = messages.count { !it.delivered }
     val dismissState = rememberSwipeToDismissBoxState(
@@ -315,8 +309,8 @@
         Card(
             modifier = Modifier
                 .fillMaxWidth()
-                .clickable(onClick = onClick)
+                .clickable(onClick = { onNavigateToChat(peerId) })
         ) {
             Row(
                 modifier = Modifier
                     .fillMaxWidth()
                     .padding(16.dp),
                 horizontalArrangement = Arrangement.SpaceBetween
             ) {
                 Column(
                     modifier = Modifier.weight(1f)
                 ) {
                     Row(
                         horizontalArrangement = Arrangement.SpaceBetween,
                         modifier = Modifier.fillMaxWidth()
                     ) {
                         Text(
                             text = displayName,
                             style = MaterialTheme.typography.titleMedium,
                             maxLines = 1,
                             overflow = TextOverflow.Ellipsis,
                             modifier = Modifier.weight(1f)
                         )
                         Text(
                             text = formatTimestamp(lastMessage.timestamp),
                             style = MaterialTheme.typography.bodySmall,
                             color = MaterialTheme.colorScheme.onSurfaceVariant
                         )
                     }
 
                     Spacer(modifier = Modifier.height(4.dp))
 
                     Row(
                         modifier = Modifier.fillMaxWidth(),
                         horizontalArrangement = Arrangement.SpaceBetween
                     ) {
                         Text(
                             text = when (lastMessage.direction) {
                                 uniffi.api.MessageDirection.SENT -> "You: ${lastMessage.content}"
                                 uniffi.api.MessageDirection.RECEIVED -> lastMessage.content
                             },
                             style = MaterialTheme.typography.bodyMedium,
                             maxLines = 1,
                             overflow = TextOverflow.Ellipsis,
                             modifier = Modifier.weight(1f),
                             color = if (undeliveredCount > 0 && lastMessage.direction == uniffi.api.MessageDirection.SENT) {
                                 MaterialTheme.colorScheme.onSurfaceVariant
                             } else {
                                 MaterialTheme.colorScheme.onSurface
                             }
                         )
 
                         if (undeliveredCount > 0 && lastMessage.direction == uniffi.api.MessageDirection.SENT) {
                             Badge {
                                 Text(undeliveredCount.toString())
                             }
                         }
                     }
+
+                    if (lastMessage.direction == uniffi.api.MessageDirection.SENT) {
+                        Spacer(modifier = Modifier.height(2.dp))
+                        Text(
+                            text = "Status: ${deliveryState.label}",
+                            style = MaterialTheme.typography.bodySmall,
+                            color = when (deliveryState.state) {
+                                com.scmessenger.android.ui.chat.DeliveryStateSurface.DELIVERED -> MaterialTheme.colorScheme.primary
+                                com.scmessenger.android.ui.chat.DeliveryStateSurface.REJECTED -> MaterialTheme.colorScheme.error
+                                else -> MaterialTheme.colorScheme.onSurfaceVariant
+                            }
+                        )
+                    }
```

---

## 4. Comparison & Recommendations

| Metric / Dimension | Option A: Clean Pruning | Option B: Wiring for Logging & UX |
| :--- | :--- | :--- |
| **API Signature Hygiene** | Excellent. Simplifies signatures and decreases parameter passing boilerplate. | Moderately complex signature. Passes several contextual attributes. |
| **Trace Diagnostics** | Poor. Prevents tracing router race actions using context IDs. | Excellent. Directly matches repository actions with transport races. |
| **Routing Resiliency** | Standard. Only uses configured routing candidates. | Enhanced. Dials peer through core using `listeners` multiaddr fallback. |
| **User Experience (UI)** | Unchanged. No extra visual feedback for delivery. | Improved. Displays immediate delivery status for sent messages. |
| **Implementation Risk** | Very Low. Simple removal of unused code elements. | Low. Minimal additions for logging and Composable status rendering. |

### Clear Recommendation
It is highly recommended to implement **Option B** for both issues:
1. **Diagnostics Priority**: Trace correlation is critical when debugging network racing conditions in multi-transport scenarios. Standardizing `traceMessageId` across logging boundaries avoids guesswork when reading logs from external tests.
2. **Direct Dialer Fallback**: Allowing the router to fall back to the direct `listeners` addresses on peer lookup failures reduces message-dropping bugs when a routing proxy fails.
3. **UI Polish**: Incorporating the `deliveryState` in `ConversationItem` provides useful and expected user-facing status indicators right in the conversation list, aligning with standard messaging application patterns.
