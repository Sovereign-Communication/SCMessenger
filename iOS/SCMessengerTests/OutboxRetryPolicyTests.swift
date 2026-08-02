import XCTest
@testable import SCMessenger

@MainActor
final class OutboxRetryPolicyTests: XCTestCase {
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
