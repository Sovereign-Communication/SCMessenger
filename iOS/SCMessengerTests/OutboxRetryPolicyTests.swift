import XCTest
@testable import SCMessenger

final class OutboxRetryPolicyTests: XCTestCase {
    func testInitialReceiptWindowMatchesAndroidParity() {
        XCTAssertEqual(MeshRepository.initialReceiptAwaitSeconds, 60)
    }

    func testAcknowledgedReceiptRetryScheduleMatchesAndroidParity() {
        XCTAssertEqual(MeshRepository.receiptRetryDelaySeconds(ackedWithoutReceiptCount: 1), 60)
        XCTAssertEqual(MeshRepository.receiptRetryDelaySeconds(ackedWithoutReceiptCount: 3), 60)
        XCTAssertEqual(MeshRepository.receiptRetryDelaySeconds(ackedWithoutReceiptCount: 4), 30)
        XCTAssertEqual(MeshRepository.receiptRetryDelaySeconds(ackedWithoutReceiptCount: 8), 30)
        XCTAssertEqual(MeshRepository.receiptRetryDelaySeconds(ackedWithoutReceiptCount: 9), 120)
    }

    func testTransportSettingsPersistWithoutLiveActionsWhenServiceIsStopped() {
        XCTAssertEqual(
            MeshRepository.liveTransportActions(
                serviceRunning: false,
                previousBleEnabled: true,
                nextBleEnabled: false,
                previousInternetEnabled: true,
                nextInternetEnabled: false
            ),
            []
        )
    }

    func testTransportSettingsPlanOnlyChangedSupportedTransports() {
        XCTAssertEqual(
            MeshRepository.liveTransportActions(
                serviceRunning: true,
                previousBleEnabled: true,
                nextBleEnabled: false,
                previousInternetEnabled: false,
                nextInternetEnabled: true
            ),
            [.stopBle, .startInternet]
        )
        XCTAssertEqual(
            MeshRepository.liveTransportActions(
                serviceRunning: true,
                previousBleEnabled: false,
                nextBleEnabled: true,
                previousInternetEnabled: true,
                nextInternetEnabled: false
            ),
            [.startBle, .stopInternet]
        )
    }

    func testTransportSettingsPlanIsEmptyWhenSupportedValuesAreUnchanged() {
        XCTAssertEqual(
            MeshRepository.liveTransportActions(
                serviceRunning: true,
                previousBleEnabled: true,
                nextBleEnabled: true,
                previousInternetEnabled: false,
                nextInternetEnabled: false
            ),
            []
        )
    }

    func testInternetOnOffOnPlansAStopThenFreshStart() {
        XCTAssertEqual(
            MeshRepository.liveTransportActions(
                serviceRunning: true,
                previousBleEnabled: true,
                nextBleEnabled: true,
                previousInternetEnabled: true,
                nextInternetEnabled: false
            ),
            [.stopInternet]
        )
        XCTAssertEqual(
            MeshRepository.liveTransportActions(
                serviceRunning: true,
                previousBleEnabled: true,
                nextBleEnabled: true,
                previousInternetEnabled: false,
                nextInternetEnabled: true
            ),
            [.startInternet]
        )
    }

    func testAckedWithoutReceiptStopsAtPatientAgeCeiling() {
        XCTAssertTrue(
            MeshRepository.shouldStopAckedWithoutReceiptRetries(
                ackedWithoutReceiptCount: 1,
                createdAtEpochSec: 100,
                nowEpochSec: 100 + (7 * 24 * 60 * 60),
                maxAgeSeconds: 7 * 24 * 60 * 60
            )
        )
    }

    func testAckedWithoutReceiptContinuesBeforeAgeCeiling() {
        XCTAssertFalse(
            MeshRepository.shouldStopAckedWithoutReceiptRetries(
                ackedWithoutReceiptCount: 50,
                createdAtEpochSec: 100,
                nowEpochSec: 100 + (7 * 24 * 60 * 60) - 1,
                maxAgeSeconds: 7 * 24 * 60 * 60
            )
        )
    }

    func testGenuineFailureDoesNotUseAckAgeCeiling() {
        XCTAssertFalse(
            MeshRepository.shouldStopAckedWithoutReceiptRetries(
                ackedWithoutReceiptCount: 0,
                createdAtEpochSec: 100,
                nowEpochSec: 100 + (30 * 24 * 60 * 60),
                maxAgeSeconds: 7 * 24 * 60 * 60
            )
        )
    }
}
