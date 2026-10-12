//
//  DiscoveryGuards.swift
//  SCMessenger
//
//  Small pure / injectable decision helpers extracted from the BLE, mDNS and
//  invite-redeem paths (#469 T3/T7/T8) so their guard behaviour can be unit
//  tested without CoreBluetooth, the Network framework or a live core.
//

import Foundation

// MARK: - BLE scan intent

/// Tracks whether the owner still wants a BLE scan. Deferred work (a start
/// queued until Bluetooth powers on, a scheduled duty cycle, a scheduler
/// reset) consults it so nothing can start the radio after `stop`.
struct BLEScanIntent: Equatable {
    private(set) var wantsScan: Bool = false
    private(set) var pendingScanOnReady: Bool = false

    /// The owner asked to scan. Returns true when a scan may begin now; when
    /// Bluetooth is not powered on the start is remembered and deferred.
    mutating func start(poweredOn: Bool) -> Bool {
        wantsScan = true
        if poweredOn {
            pendingScanOnReady = false
            return true
        }
        pendingScanOnReady = true
        return false
    }

    /// The owner stopped scanning: cancels any deferred start.
    mutating func stop() {
        wantsScan = false
        pendingScanOnReady = false
    }

    /// Bluetooth reached poweredOn. Returns true only when a start was
    /// deferred and the owner still wants it.
    mutating func bluetoothPoweredOn() -> Bool {
        guard pendingScanOnReady && wantsScan else { return false }
        pendingScanOnReady = false
        return true
    }

    /// A scheduler reset arrived. Returns true when a scan should restart now;
    /// when Bluetooth is off the start is deferred instead.
    mutating func discoveryReset(poweredOn: Bool) -> Bool {
        guard wantsScan else { return false }
        if poweredOn { return true }
        pendingScanOnReady = true
        return false
    }

    /// A queued duty-cycle block may run only while a scan is still wanted.
    var allowsQueuedCycle: Bool { wantsScan }
}

/// Schedules the end of a background scan window. CBCentralManager and the
/// scanning flag are confined to the main queue, so the stop must run there.
nonisolated enum BLEScanWindow {
    static func scheduleStop(
        after seconds: TimeInterval,
        on queue: DispatchQueue = .main,
        _ stop: @escaping () -> Void
    ) {
        queue.asyncAfter(deadline: .now() + seconds, execute: stop)
    }
}

// MARK: - mDNS browser failure handling

/// What to do when a NetServiceBrowser reports a search failure.
enum MDNSBrowseFailureAction: Equatable {
    /// The failing browser belongs to a session already replaced: do nothing.
    case ignoreStale
    /// Tear the session down; nobody wants browsing (or no scheduler), no retry.
    case resetWithoutRetry
    /// Tear the session down and retry after this many seconds.
    case resetAndRetry(after: TimeInterval)
}

enum MDNSBrowseFailurePolicy {
    static func decide(
        browserIsCurrent: Bool,
        browsingRequested: Bool,
        retryDelay: TimeInterval?
    ) -> MDNSBrowseFailureAction {
        guard browserIsCurrent else { return .ignoreStale }
        if browsingRequested, let delay = retryDelay {
            return .resetAndRetry(after: delay)
        }
        return .resetWithoutRetry
    }
}

// MARK: - Invite redemption

/// Single-consumption ledger for `SCI1:` invite tokens. The token is claimed
/// before the core call and released only when core rejects it.
struct InviteClaimLedger {
    private var claimed: Set<String> = []

    func isClaimed(_ token: String) -> Bool { claimed.contains(token) }

    /// Returns true for exactly one caller per token.
    mutating func claim(_ token: String) -> Bool {
        claimed.insert(token).inserted
    }

    mutating func release(_ token: String) {
        claimed.remove(token)
    }

    /// Claim `token`, then run `coreCall`. The claim is made before the call so
    /// a concurrent caller sees it; it is released when the call throws so a
    /// corrected retry is possible, and kept once the call succeeded. The
    /// closure receives whether the token is claimed at the moment core runs.
    mutating func consume<T>(
        _ token: String,
        _ coreCall: (_ tokenIsClaimed: Bool) throws -> T
    ) -> InviteConsumeResult<T> {
        guard claim(token) else { return .alreadyClaimed }
        do {
            return .accepted(try coreCall(isClaimed(token)))
        } catch {
            release(token)
            return .rejected(error)
        }
    }
}

enum InviteConsumeResult<T> {
    case alreadyClaimed
    case accepted(T)
    case rejected(Error)
}

/// Gate for the UI: one redeem at a time (scan frames repeat, paste can race a scan).
struct InviteRedeemInFlightGate: Equatable {
    private(set) var isRedeeming: Bool = false

    mutating func begin() -> Bool {
        guard !isRedeeming else { return false }
        isRedeeming = true
        return true
    }

    mutating func end() { isRedeeming = false }
}
