import XCTest
@testable import SCMessenger

/// #469 T3: an invite is redeemed once, and its token is claimed before core runs.
@MainActor
final class InviteRedemptionTests: XCTestCase {
    private struct CoreRejected: Error {}

    func testClaimSucceedsOnceThenRefuses() {
        var ledger = InviteClaimLedger()
        XCTAssertTrue(ledger.claim("SCI1:abc"))
        XCTAssertFalse(ledger.claim("SCI1:abc"))
        XCTAssertTrue(ledger.claim("SCI1:other"))
    }

    func testTokenIsClaimedBeforeTheCoreCallRuns() {
        var ledger = InviteClaimLedger()
        var claimedWhenCoreRan: Bool?
        _ = ledger.consume("SCI1:abc") { tokenIsClaimed -> Int in
            claimedWhenCoreRan = tokenIsClaimed
            return 1
        }
        XCTAssertEqual(claimedWhenCoreRan, true)
    }

    func testCoreCallIsInvokedExactlyOncePerToken() {
        var ledger = InviteClaimLedger()
        var calls = 0
        let first = ledger.consume("SCI1:abc") { _ -> Int in calls += 1; return 42 }
        let second = ledger.consume("SCI1:abc") { _ -> Int in calls += 1; return 43 }
        guard case let .accepted(value) = first else { return XCTFail("first redeem must be accepted") }
        XCTAssertEqual(value, 42)
        guard case .alreadyClaimed = second else { return XCTFail("second redeem must be refused") }
        XCTAssertEqual(calls, 1)
    }

    func testRepeatedScannerFramesRedeemOnce() {
        var ledger = InviteClaimLedger()
        var calls = 0
        for _ in 0..<10 {
            _ = ledger.consume("SCI1:frame") { _ in calls += 1 }
        }
        XCTAssertEqual(calls, 1)
    }

    func testCoreRejectionReleasesTokenForACorrectedRetry() {
        var ledger = InviteClaimLedger()
        let rejected = ledger.consume("SCI1:abc") { _ throws -> Int in throw CoreRejected() }
        guard case let .rejected(error) = rejected else { return XCTFail("expected rejection") }
        XCTAssertTrue(error is CoreRejected)
        XCTAssertFalse(ledger.isClaimed("SCI1:abc"))
        let retry = ledger.consume("SCI1:abc") { _ -> Int in 7 }
        guard case let .accepted(value) = retry else { return XCTFail("retry must be accepted") }
        XCTAssertEqual(value, 7)
    }

    func testAcceptedTokenStaysClaimed() {
        var ledger = InviteClaimLedger()
        _ = ledger.consume("SCI1:abc") { _ -> Int in 1 }
        XCTAssertTrue(ledger.isClaimed("SCI1:abc"))
    }

    // MARK: JoinMeshView in-flight gate

    func testOnlyOneRedeemMayBeInFlight() {
        var gate = InviteRedeemInFlightGate()
        XCTAssertTrue(gate.begin())
        XCTAssertTrue(gate.isRedeeming)
        XCTAssertFalse(gate.begin())
    }

    func testGateReopensAfterCompletion() {
        var gate = InviteRedeemInFlightGate()
        XCTAssertTrue(gate.begin())
        gate.end()
        XCTAssertFalse(gate.isRedeeming)
        XCTAssertTrue(gate.begin())
    }

    func testInviteTextExtractionRejectsNonInvites() {
        XCTAssertNil(InviteText.extract(nil))
        XCTAssertNil(InviteText.extract("hello world"))
        XCTAssertNil(InviteText.extract("SCI1:"))
        XCTAssertEqual(InviteText.extract("join me: SCI1:abc123."), "SCI1:abc123")
    }
}
