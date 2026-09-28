# P0_ANDROID_007: Network Diagnostics Implementation

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Priority:** P0 CRITICAL
**Platform:** Android
**Status:** Open
**Routing Tags:** [REQUIRES: NETWORK_SYNC] [REQUIRES: FINALIZATION]

## Objective
Implement comprehensive network diagnostics and connectivity testing to identify and troubleshoot network failures. From ANDROID_PIXEL_6A_AUDIT_2026-04-17, all 4 relay servers are failing with "Network error" but detailed reasons are not logged.

## Root Cause
From ANDROID_PIXEL_6A_AUDIT_2026-04-17:
- All 4 bootstrap relay nodes failing with generic "Network error"
- No detailed error diagnostics (DNS, timeout, refusal, TLS)
- No network connectivity testing at startup
- Missing alternative bootstrap node sources
- Inadequate error classification and logging

## Implementation Plan

### 1. Detailed Error Diagnostics
**File:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
```kotlin
private fun enhanceNetworkErrorLogging(exception: Exception, node: BootstrapNode) {
    val errorDetails = when (exception) {
        is UnknownHostException -> 
            "DNS resolution failed for ${node.address.host}"
        is ConnectException -> 
            "Connection refused - port may be blocked or service down"
        is SocketTimeoutException -> 
            "Connection timeout after ${node.timeoutMs}ms"
        is SSLException -> 
            "TLS handshake failed: ${exception.message}"
        is IOException -> 
            "Network I/O error: ${exception.message}"
        else -> 
            "Unknown network error: ${exception.javaClass.simpleName}: ${exception.message}"
    }
    
    Timber.w("Bootstrap failed for ${node.address} - $errorDetails")
    
    // Track failure metrics for adaptive routing
    trackNetworkFailure(node.id, errorDetails, exception)
}

private fun trackNetworkFailure(nodeId: String, reason: String, exception: Exception) {
    networkFailureMetrics.recordFailure(nodeId, reason, exception)
    
    // Update node health status
    updateNodeHealthStatus(nodeId, HealthStatus.UNHEALTHY, reason)
    
    // Trigger fallback if needed
    if (networkFailureMetrics.isNodeUnreachable(nodeId)) {
        triggerFallbackProtocol(nodeId)
    }
}
```

### 2. Startup Connectivity Testing
**File:** `android/app/src/main/java/com/scmessenger/android/network/NetworkDiagnostics.kt` (NEW)
```kotlin
class NetworkDiagnostics @Inject constructor(
    private val connectivityManager: ConnectivityManager,
    private val context: Context
) {
    
    suspend fun testNetworkConnectivity(): NetworkTestResults {
        val results = NetworkTestResults()
        
        // Test basic internet connectivity
        results.internetConnectivity = testInternetConnectivity()
        
        // Test DNS resolution
        results.dnsResolution = testDnsResolution()
        
        // Test common ports
        results.commonPorts = testCommonPorts()
        
        // Test relay-specific connectivity
        results.relayConnectivity = testRelaySpecificConnectivity()
        
        // Detect network type and restrictions
        results.networkType = detectNetworkType()
        results.restrictions = detectNetworkRestrictions()
        
        return results
    }
    
    private fun testInternetConnectivity(): Boolean {
        return try {
            val url = URL("https://www.google.com")
            val connection = url.openConnection() as HttpURLConnection
            connection.connectTimeout = 5000
            connection.readTimeout = 5000
            connection.requestMethod = "HEAD"
            connection.responseCode == 200
        } catch (e: Exception) {
            false
        }
    }
    
    private fun testDnsResolution(): Map<String, Boolean> {
        val domains = listOf(
            "google.com",
            "cloudflare.com", 
            "bootstrap.scmessenger.net",
            "relay.scmessenger.net"
        )
        
        return domains.associateWith { domain ->
            try {
                InetAddress.getByName(domain) != null
            } catch (e: Exception) {
                false
            }
        }
    }
}
```

### 3. Alternative Bootstrap Sources
**File:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
```kotlin
private fun implementAlternativeBootstrapSources() {
    // Multiple bootstrap source strategies
    val bootstrapSources = listOf(
        // 1. Environment variable override
        EnvironmentBootstrapSource(),
        
        // 2. Remote config endpoint
        RemoteConfigBootstrapSource("https://config.scmessenger.net/bootstrap"),
        
        // 3. Hardcoded fallback
        StaticBootstrapSource(),
        
        // 4. Peer-provided bootstrap
        PeerBootstrapSource(),
        
        // 5. mDNS local discovery
        MdnsBootstrapSource()
    )
    
    // Try sources in order until successful
    for (source in bootstrapSources) {
        try {
            val nodes = source.getBootstrapNodes()
            if (nodes.isNotEmpty()) {
                Timber.i("Using bootstrap source: ${source.name} with ${nodes.size} nodes")
                return nodes
            }
        } catch (e: Exception) {
            Timber.w("Bootstrap source ${source.name} failed: ${e.message}")
        }
    }
    
    throw IllegalStateException("No bootstrap sources available")
}

// Bootstrap source interface
interface BootstrapSource {
    val name: String
    fun getBootstrapNodes(): List<BootstrapNode>
}
```

### 4. Network Type Detection
**File:** `android/app/src/main/java/com/scmessenger/android/network/NetworkTypeDetector.kt` (NEW)
```kotlin
class NetworkTypeDetector @Inject constructor(
    private val connectivityManager: ConnectivityManager
) {
    
    fun detectNetworkType(): NetworkType {
        val network = connectivityManager.activeNetwork
        val caps = connectivityManager.getNetworkCapabilities(network)
        
        return when {
            caps == null -> NetworkType.UNKNOWN
            caps.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR) -> {
                detectCellularNetworkType(caps)
            }
            caps.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) -> {
                detectWifiNetworkType(caps)
            }
            caps.hasTransport(NetworkCapabilities.TRANSPORT_ETHERNET) -> {
                NetworkType.ETHERNET
            }
            caps.hasTransport(NetworkCapabilities.TRANSPORT_VPN) -> {
                NetworkType.VPN
            }
            else -> NetworkType.UNKNOWN
        }
    }
    
    private fun detectCellularNetworkType(caps: NetworkCapabilities): NetworkType {
        return when {
            caps.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET) -> {
                // Check for common cellular restrictions
                if (isPortBlocked(9001) || isPortBlocked(9010)) {
                    NetworkType.CELLULAR_RESTRICTED
                } else {
                    NetworkType.CELLULAR
                }
            }
            else -> NetworkType.CELLULAR_NO_INTERNET
        }
    }
    
    private fun isPortBlocked(port: Int): Boolean {
        return try {
            val socket = Socket()
            socket.connect(InetSocketAddress("8.8.8.8", port), 3000)
            socket.close()
            false // Port is open
        } catch (e: Exception) {
            true // Port is blocked
        }
    }
}

enum class NetworkType {
    UNKNOWN, CELLULAR, CELLULAR_RESTRICTED, CELLULAR_NO_INTERNET,
    WIFI, WIFI_RESTRICTED, ETHERNET, VPN
}
```

### 5. Comprehensive Diagnostics Reporting
**File:** `android/app/src/main/java/com/scmessenger/android/network/DiagnosticsReporter.kt` (NEW)
```kotlin
class DiagnosticsReporter {
    
    fun generateNetworkDiagnosticsReport(): NetworkDiagnosticsReport {
        return NetworkDiagnosticsReport(
            timestamp = System.currentTimeMillis(),
            networkType = networkDetector.detectNetworkType(),
            connectivityTest = networkDiagnostics.testNetworkConnectivity(),
            failureMetrics = networkFailureMetrics.getSummary(),
            nodeHealth = getNodeHealthStatuses(),
            recommendations = generateRecommendations()
        )
    }
    
    private fun generateRecommendations(): List<String> {
        val recommendations = mutableListOf<String>()
        
        if (networkFailureMetrics.hasDnsFailures()) {
            recommendations.add("DNS resolution failing - try alternative DNS servers")
        }
        
        if (networkFailureMetrics.hasPortBlocking()) {
            recommendations.add("Port blocking detected - enable WebSocket fallback")
        }
        
        if (networkDetector.detectNetworkType() == NetworkType.CELLULAR_RESTRICTED) {
            recommendations.add("Cellular network restrictions detected - use standard ports")
        }
        
        return recommendations
    }
    
    fun showDiagnosticsToUser() {
        val report = generateNetworkDiagnosticsReport()
        
        // Show user-friendly summary
        val message = buildString {
            append("Network Status: ${report.networkType}\n")
            append("Internet: ${if (report.connectivityTest.internetConnectivity) "" else ""}\n")
            
            if (report.recommendations.isNotEmpty()) {
                append("\nRecommendations:\n")
                report.recommendations.forEach { rec ->
                    append(" $rec\n")
                }
            }
        }
        
        // Show in UI
        showNetworkStatusDialog(message)
    }
}
```

## Files to Modify/Create
1. `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` - Enhanced error logging
2. `android/app/src/main/java/com/scmessenger/android/network/NetworkDiagnostics.kt` (NEW) - Connectivity testing
3. `android/app/src/main/java/com/scmessenger/android/network/NetworkTypeDetector.kt` (NEW) - Network type detection
4. `android/app/src/main/java/com/scmessenger/android/network/DiagnosticsReporter.kt` (NEW) - Reporting
5. `android/app/src/main/java/com/scmessenger/android/utils/NetworkFailureMetrics.kt` (NEW) - Failure tracking
6. `android/app/src/main/java/com/scmessenger/android/ui/dialogs/NetworkStatusDialog.kt` (NEW) - User UI

## Test Plan
1. **Error Classification**: Test all network error types are properly classified
2. **Connectivity Testing**: Verify startup connectivity tests work
3. **Network Detection**: Test cellular vs WiFi vs restricted detection
4. **Fallback Sources**: Test alternative bootstrap source prioritization
5. **User Reporting**: Verify diagnostics are user-friendly and actionable

## Success Criteria
-  Detailed error diagnostics for all network failure types
-  Comprehensive connectivity testing at startup
-  Accurate network type and restriction detection
-  Multiple fallback bootstrap sources
-  User-friendly diagnostics reporting

## Priority: URGENT
Network connectivity is essential for app functionality. Without proper diagnostics, failures appear as generic "Network error" with no troubleshooting path.

**Estimated LOC:** ~400-500 LOC across 6 files
**Time Estimate:** 4-5 hours implementation + 2 hours testing