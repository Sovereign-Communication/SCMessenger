package com.scmessenger.android.transport

import android.content.Context
import android.net.ConnectivityManager
import android.net.LinkAddress
import android.net.LinkProperties
import android.net.NetworkCapabilities
import android.os.Build
import com.scmessenger.android.service.TransportType
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.cancel
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlinx.coroutines.sync.Semaphore
import kotlinx.coroutines.sync.withPermit
import kotlinx.coroutines.withTimeoutOrNull
import timber.log.Timber
import java.net.Inet4Address
import java.net.InetSocketAddress
import java.nio.channels.AsynchronousSocketChannel
import java.nio.channels.CompletionHandler
import java.util.concurrent.ConcurrentHashMap
import kotlin.coroutines.resume

/**
 * TCP-based LAN subnet probe.
 *
 * Motivation: mDNS (NsdManager) on Android and libp2p-mdns on Rust peers use
 * different service types and often fail to discover each other. Even when both
 * speak mDNS, multicast DNS is link-local (224.0.0.251) and does NOT cross
 * routers, different broadcast domains, or some NAT'd virtual interfaces
 * (notably WSL2). The libp2p swarm listens on TCP 9001 and the relay/WS on
 * 9002, both of which respond to unicast TCP.
 *
 * This service periodically scans common LAN subnets for an open port and
 * reports any successful hit as a peer candidate. The reported multiaddr is
 * fed through the same onLanAddressResolved callback that mDNS uses, so
 * downstream dial logic in MeshRepository -> SwarmBridge is unchanged.
 *
 * Behaviour:
 * - Discover the device's own IPv4 link-local subnets (192.168.x, 10.x,
 *   172.16-31.x) by querying ConnectivityManager. Each interface's
 *   /24 (or shorter) prefix is queued for probing.
 * - Additionally, scan a small set of common "fallback" subnets to catch
 *   devices on neighbouring VLANs or split-WiFi networks.
 * - For each candidate host, attempt a TCP connect to ports 9001 and 9002
 *   with a 1s timeout. If either succeeds, derive a libp2p multiaddr and
 *   notify the callback.
 * - Cancellable via [stop]. Re-entrant calls to [start] are no-ops.
 * - Battery friendly: full sweep every [scanIntervalMs] (default 30s) and
 *   the per-host connect timeout is short (1s). Concurrency is bounded by
 *   [maxConcurrentProbes].
 */
class SubnetProbe(
    private val context: Context,
    /**
     * Called when a host:port is found open and identified as a likely peer.
     * The multiaddr is in libp2p format suitable for SwarmBridge.dial.
     * Example: "/ip4/192.168.0.230/tcp/9001" or with peer id:
     * "/ip4/192.168.0.230/tcp/9001/p2p/12D3Koo..."
     */
    private val onLanAddressResolved: (multiaddr: String, transport: TransportType) -> Unit,
    /**
     * Optional callback to request the local libp2p peer id so we can
     * exclude self during scanning. When null, self-exclusion falls back
     * to a best-effort check on the device's own IP addresses.
     */
    private val getLocalPeerId: (() -> String?)? = null,
    /** Optional: limit probed ports (defaults: libp2p TCP 9001, WS 9002). */
    private val targetPorts: IntArray = intArrayOf(9001, 9002),
    /** Full-sweep interval. Default 30s keeps battery impact minimal. */
    private val scanIntervalMs: Long = 30_000L,
    /** Per-TCP-connect timeout. */
    private val connectTimeoutMs: Int = 1_000,
    /** Bounded parallelism for the connect probe fan-out. */
    private val maxConcurrentProbes: Int = 32,
    /**
     * Pause between a successful raw TCP probe and reporting the address to
     * the dialer. The probe socket leaves the peer's port in TIME_WAIT; an
     * immediate libp2p dial to the same ip:port can be rejected by the OS
     * (observed against the Windows CLI daemon). 500ms lets it clear.
     */
    private val dialDelayMs: Long = 500L
) {
    @Volatile private var isRunning = false
    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    private var sweepJob: Job? = null

    // Track (host,port) tuples we've already reported within this lifetime
    // so we don't spam the dialer. The dialer has its own backoff; this is
    // just a defence-in-depth rate limit.
    private val recentlyReported = ConcurrentHashMap<String, Long>()
    private val reportDedupMs = 60_000L

    // Track which subnets we've already expanded this cycle to avoid
    // re-listing them across overlapping interfaces.
    private val probedSubnetsThisCycle = ConcurrentHashMap<String, Long>()

    /**
     * Begin periodic scanning. Safe to call multiple times; only the first
     * call has effect.
     */
    fun start() {
        if (isRunning) {
            Timber.d("SubnetProbe already running; start() ignored")
            return
        }
        isRunning = true
        Timber.i("SubnetProbe starting (interval=${scanIntervalMs}ms, ports=${targetPorts.toList()})")
        sweepJob = scope.launch {
            // Run an immediate first sweep, then loop on the interval.
            while (isRunning) {
                try {
                    runSweep()
                } catch (t: Throwable) {
                    Timber.w(t, "SubnetProbe sweep failed")
                }
                if (!isRunning) break
                delay(scanIntervalMs)
            }
        }
    }

    /**
     * Cancel all in-flight probes and stop the sweep loop.
     */
    fun stop() {
        if (!isRunning) return
        isRunning = false
        sweepJob?.cancel()
        sweepJob = null
        Timber.i("SubnetProbe stopped")
    }

    /**
     * Release coroutine resources. Call when the owning component is
     * destroyed (e.g. in TransportManager.cleanup()).
     */
    fun cleanup() {
        stop()
        scope.cancel()
    }

    // ----------------------------------------------------------------------
    // Sweep implementation
    // ----------------------------------------------------------------------

    private suspend fun runSweep() {
        val localSelfIps = enumerateLocalIPv4Addresses()
        val subnets = enumerateCandidateSubnets()
        if (subnets.isEmpty()) {
            Timber.d("SubnetProbe: no candidate subnets (no WiFi/Ethernet link?)")
            return
        }
        Timber.d("SubnetProbe: scanning ${subnets.size} subnets, skipping self=$localSelfIps")

        val candidates = mutableListOf<Pair<String, Int>>()
        for (subnet in subnets) {
            // Avoid re-scanning the same /24 within the dedup window.
            if (probedSubnetsThisCycle[subnet]?.let { (System.currentTimeMillis() - it) < reportDedupMs } == true) {
                continue
            }
            probedSubnetsThisCycle[subnet] = System.currentTimeMillis()
            candidates += expandSubnet(subnet)
        }

        // Fan out probes in bounded parallel batches. The semaphore is the
        // kotlinx.coroutines suspending variant: waiting coroutines suspend
        // instead of parking Dispatchers.IO threads. (The previous
        // java.util.concurrent.Semaphore froze the entire IO pool — thousands
        // of coroutines each blocking a real thread in acquire() — which
        // starved all other background work and fired an ANR on "Rescan".)
        val sem = Semaphore(maxConcurrentProbes)
        val jobs = candidates.map { (host, port) ->
            scope.async {
                sem.withPermit {
                    if (!isRunning) return@withPermit
                    if (host in localSelfIps) return@withPermit
                    probeHost(host, port, localSelfIps)
                }
            }
        }
        jobs.awaitAll()
    }

    private suspend fun probeHost(host: String, port: Int, localSelfIps: Set<String>) {
        if (host in localSelfIps) return
        val key = "$host:$port"
        val now = System.currentTimeMillis()
        recentlyReported.entries.removeAll { (_, ts) -> (now - ts) > reportDedupMs }
        if (recentlyReported.containsKey(key)) return

        // Non-blocking connect: AsynchronousSocketChannel completes on its own
        // small NIO group instead of parking an IO-dispatcher thread per probe.
        // A connect that is accepted is a sufficient liveness signal — we no
        // longer peek-read a byte, which used to hold the socket (and a
        // thread) open for up to another 200ms per host.
        val opened = withTimeoutOrNull(connectTimeoutMs.toLong() + 250L) {
            connectAsync(host, port)
        } ?: false

        if (!opened) return

        recentlyReported[key] = now
        // Port 9002 is a WebSocket listener — append /ws so libp2p uses the
        // correct transport. Port 9001 is raw TCP.
        val multiaddr = if (port == 9002) "/ip4/$host/tcp/$port/ws" else "/ip4/$host/tcp/$port"
        Timber.i("SubnetProbe: open port $host:$port (likely SCMessenger peer) -> $multiaddr")
        // Let the probe socket's TIME_WAIT state clear on the peer before the
        // dialer opens the real libp2p connection to the same ip:port.
        if (dialDelayMs > 0) delay(dialDelayMs)
        try {
            onLanAddressResolved(multiaddr, TransportType.TCP_MDNS)
        } catch (t: Throwable) {
            Timber.w(t, "SubnetProbe: onLanAddressResolved callback threw")
        }
    }

    /**
     * Suspending TCP connect via NIO [AsynchronousSocketChannel]. Never blocks
     * the calling thread; cancellation closes the channel, which aborts the
     * in-flight connect. Returns true iff the connection was accepted.
     */
    private suspend fun connectAsync(host: String, port: Int): Boolean =
        suspendCancellableCoroutine { cont ->
            val channel = try {
                AsynchronousSocketChannel.open()
            } catch (t: Throwable) {
                cont.resume(false)
                return@suspendCancellableCoroutine
            }
            cont.invokeOnCancellation {
                try { channel.close() } catch (_: Throwable) {}
            }
            try {
                channel.connect(
                    InetSocketAddress(host, port),
                    Unit,
                    object : CompletionHandler<Void?, Unit> {
                        override fun completed(result: Void?, attachment: Unit) {
                            try { channel.close() } catch (_: Throwable) {}
                            if (cont.isActive) cont.resume(true)
                        }

                        override fun failed(exc: Throwable, attachment: Unit) {
                            try { channel.close() } catch (_: Throwable) {}
                            if (cont.isActive) cont.resume(false)
                        }
                    }
                )
            } catch (t: Throwable) {
                try { channel.close() } catch (_: Throwable) {}
                if (cont.isActive) cont.resume(false)
            }
        }

    // ----------------------------------------------------------------------
    // Subnet enumeration
    // ----------------------------------------------------------------------

    private fun enumerateLocalIPv4Addresses(): Set<String> {
        val result = HashSet<String>()
        try {
            val cm = context.applicationContext.getSystemService(Context.CONNECTIVITY_SERVICE) as? ConnectivityManager
                ?: return result
            // Enumerate across ALL networks, not just the active one. The active
            // network can be a VPN or cellular while Wi-Fi (the interface the
            // candidate-subnet sweep scans) carries the device's LAN address;
            // missing it here let the probe dial the device's own listener
            // (self-dial -> NoAddresses exception every sweep).
            val networks = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.LOLLIPOP) {
                cm.allNetworks
            } else {
                null
            }
            if (networks != null) {
                for (net in networks) {
                    val link = cm.getLinkProperties(net) ?: continue
                    for (addr in link.linkAddresses) {
                        val ip = addr.address
                        if (ip is Inet4Address && !ip.isLoopbackAddress) {
                            result += ip.hostAddress.orEmpty()
                        }
                    }
                }
            } else {
                // Legacy fallback (< API 21) — should not happen on minSdk 26.
                val link = cm.activeNetwork?.let { cm.getLinkProperties(it) }
                for (addr in link?.linkAddresses.orEmpty()) {
                    val ip = addr.address
                    if (ip is Inet4Address && !ip.isLoopbackAddress) {
                        result += ip.hostAddress.orEmpty()
                    }
                }
            }
        } catch (t: Throwable) {
            Timber.w(t, "SubnetProbe: failed to enumerate local IPv4 addresses")
        }
        return result
    }

    /**
     * Returns a list of /24 subnets (in "a.b.c.0/24" form) that are worth
     * scanning. The list is built from:
     *   1. The device's own IPv4 link-local prefixes (with /24 applied if
     *      the prefix is shorter than /24, e.g. /16 home networks are
     *      narrowed to a /24 around the local address).
     *   2. A small fallback list of common router defaults.
     */
    private fun enumerateCandidateSubnets(): List<String> {
        val out = LinkedHashSet<String>()
        try {
            val cm = context.applicationContext.getSystemService(Context.CONNECTIVITY_SERVICE) as? ConnectivityManager
            if (cm != null) {
                val networks = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.LOLLIPOP) {
                    cm.allNetworks
                } else {
                    null
                }
                if (networks != null) {
                    for (net in networks) {
                        val caps = cm.getNetworkCapabilities(net) ?: continue
                        // Only scan interfaces that are useful for LAN discovery.
                        if (!caps.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) &&
                            !caps.hasTransport(NetworkCapabilities.TRANSPORT_ETHERNET) &&
                            !caps.hasTransport(NetworkCapabilities.TRANSPORT_VPN)) {
                            continue
                        }
                        val link = cm.getLinkProperties(net) ?: continue
                        for (addr in link.linkAddresses) {
                            val ip = addr.address
                            if (ip !is Inet4Address || ip.isLoopbackAddress) continue
                            out += narrowToScanSubnet(addr)
                        }
                    }
                } else {
                    // Legacy fallback (< API 21) — should not happen on minSdk 26.
                    val link = cm.activeNetwork?.let { cm.getLinkProperties(it) }
                    if (link != null) {
                        for (addr in link.linkAddresses) {
                            val ip = addr.address
                            if (ip !is Inet4Address || ip.isLoopbackAddress) continue
                            out += narrowToScanSubnet(addr)
                        }
                    }
                }
            }
        } catch (t: Throwable) {
            Timber.w(t, "SubnetProbe: failed to enumerate candidate subnets")
        }

        // Always probe a few common defaults in case the device's own
        // interface is misreported (e.g. mobile hotspot on a different /24
        // than the host that runs the daemon).
        for (fallback in FALLBACK_SUBNETS) {
            out += fallback
        }
        return out.toList()
    }

    private fun narrowToScanSubnet(addr: LinkAddress): String {
        val ip = addr.address as Inet4Address
        val rawPrefix = addr.prefixLength
        val octets = ip.hostAddress.split(".")
        if (octets.size != 4) return "${octets[0]}.${octets[1]}.${octets[2]}.0/24"
        return when {
            // Use /24 around the local host for the typical /16 home LAN.
            rawPrefix < 24 -> "${octets[0]}.${octets[1]}.${octets[2]}.0/24"
            // /30 or smaller — scan the whole thing.
            else -> {
                val mask = ipv4NetmaskInt(rawPrefix)
                val ipInt = (octets[0].toInt() and 0xff shl 24) or
                    (octets[1].toInt() and 0xff shl 16) or
                    (octets[2].toInt() and 0xff shl 8) or
                    (octets[3].toInt() and 0xff)
                val netInt = ipInt and mask
                val a = (netInt ushr 24) and 0xff
                val b = (netInt ushr 16) and 0xff
                val c = (netInt ushr 8) and 0xff
                val d = netInt and 0xff
                "$a.$b.$c.$d/$rawPrefix"
            }
        }
    }

    private fun ipv4NetmaskInt(prefixLen: Int): Int {
        if (prefixLen <= 0) return 0
        if (prefixLen >= 32) return -1 // 0xFFFFFFFF
        return (0xFFFFFFFF.toInt() shl (32 - prefixLen))
    }

    /**
     * Expand a /24 (or smaller) subnet into host candidates. For /24 we
     * probe 1..254 (skipping .0 network and .255 broadcast). For shorter
     * prefixes we just probe the local /24 around the device address.
     */
    private fun expandSubnet(subnet: String): List<Pair<String, Int>> {
        val parts = subnet.split("/")
        val base = parts[0]
        val prefix = parts.getOrNull(1)?.toIntOrNull() ?: 24
        val octets = base.split(".")
        if (octets.size != 4) return emptyList()
        val a = octets[0].toIntOrNull() ?: return emptyList()
        val b = octets[1].toIntOrNull() ?: return emptyList()
        val c = octets[2].toIntOrNull() ?: return emptyList()
        val d = octets[3].toIntOrNull() ?: return emptyList()

        val out = ArrayList<Pair<String, Int>>(256)
        val ips = if (prefix >= 24) {
            (1..254).map { last -> "$a.$b.$c.$last" }
        } else {
            // /16 or wider: only scan the local /24 around the device
            // (i.e. just the .0/24 from the local address). Expanding a /16
            // would mean 65k probes, which is unacceptable.
            (1..254).map { last -> "$a.$b.$c.$last" }
        }
        for (host in ips) {
            for (port in targetPorts) {
                out += host to port
            }
        }
        return out
    }

    companion object {
        // A handful of common router defaults. Keep this list short — the
        // point is to catch the obvious cases (home WiFi /24 = 192.168.0.x
        // and 192.168.1.x, plus 10.0.0.x used by some mesh routers).
        private val FALLBACK_SUBNETS = listOf(
            "192.168.0.0/24",
            "192.168.1.0/24",
            "10.0.0.0/24",
            "10.0.1.0/24",
            "172.16.0.0/24",
            "172.20.0.0/24",
            "172.24.0.0/24",
            "172.26.0.0/24",
            "172.30.0.0/24"
        )
    }
}
