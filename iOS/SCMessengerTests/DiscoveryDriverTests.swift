import XCTest
@testable import SCMessenger

/// #469 T7/T8: DiscoveryDriver is lock-protected (no lock held across callbacks) and shutdown is idempotent.
@MainActor
final class DiscoveryDriverTests: XCTestCase {

    func testShutdownIsIdempotent() {
        let driver = DiscoveryDriver()
        XCTAssertFalse(driver.isShutdown)
        driver.shutdown()
        driver.shutdown()
        driver.shutdown()
        XCTAssertTrue(driver.isShutdown)
    }

    func testReportAfterShutdownIsANoOp() {
        let driver = DiscoveryDriver()
        var resets = 0
        driver.onReset = { _ in resets += 1 }
        driver.shutdown()
        XCTAssertTrue(driver.report(.bleOn).isEmpty)
        XCTAssertEqual(resets, 0)
    }

    func testResetHandlerCannotBeInstalledAfterShutdown() {
        let driver = DiscoveryDriver()
        driver.shutdown()
        var resets = 0
        driver.onReset = { _ in resets += 1 }
        XCTAssertNil(driver.onReset)
        _ = driver.report(.bleOn)
        XCTAssertEqual(resets, 0)
    }

    func testResetHandlerRunsWithoutHoldingTheLock() {
        let driver = DiscoveryDriver()
        var sawShutdownFlag: Bool?
        driver.onReset = { _ in
            // Re-enters the driver's lock: would deadlock if held across the callback.
            sawShutdownFlag = driver.isShutdown
        }
        let reset = driver.report(.bleOn)
        if reset.isEmpty {
            XCTAssertNil(sawShutdownFlag)
        } else {
            XCTAssertEqual(sawShutdownFlag, false)
        }
    }
}
