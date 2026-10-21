import XCTest
@testable import SCMessenger

/// #469 T7/T8 guard behaviour for the BLE central and mDNS paths, exercised
/// through the extracted decision helpers (no radios, no Network framework).
@MainActor
final class DiscoveryTransportGuardsTests: XCTestCase {

    // MARK: BLE: no deferred scan may start after stop

    func testStartWhilePoweredOffDefersAndPoweredOnStartsIt() {
        var intent = BLEScanIntent()
        XCTAssertFalse(intent.start(poweredOn: false))
        XCTAssertTrue(intent.pendingScanOnReady)
        XCTAssertTrue(intent.bluetoothPoweredOn())
        XCTAssertFalse(intent.pendingScanOnReady)
    }

    func testStopCancelsDeferredStartSoPowerOnDoesNotScan() {
        var intent = BLEScanIntent()
        _ = intent.start(poweredOn: false)
        intent.stop()
        XCTAssertFalse(intent.pendingScanOnReady)
        XCTAssertFalse(intent.bluetoothPoweredOn())
    }

    func testQueuedDutyCycleIsRefusedAfterStop() {
        var intent = BLEScanIntent()
        XCTAssertTrue(intent.start(poweredOn: true))
        XCTAssertTrue(intent.allowsQueuedCycle)
        intent.stop()
        XCTAssertFalse(intent.allowsQueuedCycle)
    }

    func testSchedulerResetAfterStopNeverRestartsScan() {
        var intent = BLEScanIntent()
        _ = intent.start(poweredOn: true)
        intent.stop()
        XCTAssertFalse(intent.discoveryReset(poweredOn: true))
        XCTAssertFalse(intent.discoveryReset(poweredOn: false))
        XCTAssertFalse(intent.pendingScanOnReady)
    }

    func testSchedulerResetWhilePoweredOffDefersWhenScanWanted() {
        var intent = BLEScanIntent()
        _ = intent.start(poweredOn: true)
        XCTAssertFalse(intent.discoveryReset(poweredOn: false))
        XCTAssertTrue(intent.pendingScanOnReady)
        XCTAssertTrue(intent.bluetoothPoweredOn())
    }

    func testSchedulerResetWhilePoweredOnRestartsScan() {
        var intent = BLEScanIntent()
        _ = intent.start(poweredOn: true)
        XCTAssertTrue(intent.discoveryReset(poweredOn: true))
    }

    // MARK: BLE: stopScan runs on the main queue

    nonisolated func testScanWindowStopRunsOnTheGivenQueue() {
        let queue = DispatchQueue(label: "scm.test.scanwindow")
        let key = DispatchSpecificKey<Bool>()
        queue.setSpecific(key: key, value: true)
        let ran = expectation(description: "stop ran")
        var onQueue = false
        BLEScanWindow.scheduleStop(after: 0.01, on: queue) {
            onQueue = DispatchQueue.getSpecific(key: key) == true
            ran.fulfill()
        }
        wait(for: [ran], timeout: 2.0)
        XCTAssertTrue(onQueue)
    }

    nonisolated func testScanWindowStopDefaultsToMainThread() {
        let ran = expectation(description: "stop ran")
        var onMain = false
        // Schedule from a background thread: the stop must still hop to main.
        DispatchQueue.global().async {
            BLEScanWindow.scheduleStop(after: 0.01) {
                onMain = Thread.isMainThread
                ran.fulfill()
            }
        }
        wait(for: [ran], timeout: 2.0)
        XCTAssertTrue(onMain)
    }

    // MARK: mDNS: stale browser failure is ignored

    func testStaleBrowserFailureIsIgnoredNotRescheduled() {
        let action = MDNSBrowseFailurePolicy.decide(
            browserIsCurrent: false,
            browsingRequested: true,
            retryDelay: 5
        )
        XCTAssertEqual(action, .ignoreStale)
    }

    func testStaleBrowserFailureIgnoredEvenWithoutScheduler() {
        XCTAssertEqual(
            MDNSBrowseFailurePolicy.decide(browserIsCurrent: false, browsingRequested: false, retryDelay: nil),
            .ignoreStale
        )
    }

    func testCurrentBrowserFailureSchedulesOneRetryWithSchedulerDelay() {
        XCTAssertEqual(
            MDNSBrowseFailurePolicy.decide(browserIsCurrent: true, browsingRequested: true, retryDelay: 7.5),
            .resetAndRetry(after: 7.5)
        )
    }

    func testCurrentBrowserFailureAfterStopDoesNotRetry() {
        XCTAssertEqual(
            MDNSBrowseFailurePolicy.decide(browserIsCurrent: true, browsingRequested: false, retryDelay: 7.5),
            .resetWithoutRetry
        )
    }

    func testCurrentBrowserFailureWithoutSchedulerDoesNotRetry() {
        XCTAssertEqual(
            MDNSBrowseFailurePolicy.decide(browserIsCurrent: true, browsingRequested: true, retryDelay: nil),
            .resetWithoutRetry
        )
    }
}
