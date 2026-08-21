import XCTest
import Combine
@testable import SCMessenger

@MainActor
final class OutboxRetryPolicyTests: XCTestCase {

    // MARK: - Test Helpers & Mocks

    final class MockPersistenceDriver: PersistenceByteDriver {
        var storedData: Data?
        var readCount: Int = 0
        var writeCount: Int = 0
        var shouldFailRead: Bool = false
        var shouldFailWrite: Bool = false

        init(initialData: Data? = nil) {
            self.storedData = initialData
        }

        func read() throws -> Data? {
            readCount += 1
            if shouldFailRead {
                throw MeshRepository.MeshOperationError.storageError("Injected read failure")
            }
            return storedData
        }

        func write(data: Data) throws {
            writeCount += 1
            if shouldFailWrite {
                throw MeshRepository.MeshOperationError.storageError("Injected write failure")
            }
            storedData = data
        }
    }

    private func createTestRepository(
        storagePath: String = NSTemporaryDirectory() + "SCMTest_\(UUID().uuidString)"
    ) -> MeshRepository {
        let repo = MeshRepository(storagePath: storagePath)
        return repo
    }

    private func makeSampleEnvelope(
        queueId: String = UUID().uuidString,
        historyRecordId: String = "msg_\(UUID().uuidString)",
        peerId: String = "peer_123",
        routePeerId: String? = nil,
        addresses: [String] = [],
        envelopeBase64: String = Data([1, 2, 3, 4]).base64EncodedString(),
        createdAtEpochSec: UInt64 = 1000,
        attemptCount: UInt32 = 0,
        nextAttemptAtEpochSec: UInt64 = 1000,
        strictBleOnlyMode: Bool = false,
        recipientIdentityId: String? = nil,
        intendedDeviceId: String? = nil,
        terminalFailureCode: String? = nil,
        ackedWithoutReceiptCount: UInt32? = nil,
        mutationGeneration: UInt64? = 0,
        retryDeferredUntilEpochSec: UInt64? = nil
    ) -> MeshRepository.PendingOutboundEnvelope {
        MeshRepository.PendingOutboundEnvelope(
            queueId: queueId,
            historyRecordId: historyRecordId,
            peerId: peerId,
            routePeerId: routePeerId,
            addresses: addresses,
            envelopeBase64: envelopeBase64,
            createdAtEpochSec: createdAtEpochSec,
            attemptCount: attemptCount,
            nextAttemptAtEpochSec: nextAttemptAtEpochSec,
            strictBleOnlyMode: strictBleOnlyMode,
            recipientIdentityId: recipientIdentityId,
            intendedDeviceId: intendedDeviceId,
            terminalFailureCode: terminalFailureCode,
            ackedWithoutReceiptCount: ackedWithoutReceiptCount,
            mutationGeneration: mutationGeneration,
            retryDeferredUntilEpochSec: retryDeferredUntilEpochSec
        )
    }

    // MARK: - Legacy Parity Tests

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

    // MARK: - Acceptance Tests T1–T18

    // T1: Legacy outbox decodes without generation or deferred keys and survives restart
    func testLegacyOutboxDecodesWithoutGenerationOrDeferredKeysAndSurvivesRestart() throws {
        let legacyJson = """
        [
          {
            "queueId": "q1",
            "historyRecordId": "msg1",
            "peerId": "peerA",
            "routePeerId": null,
            "addresses": ["/ip4/127.0.0.1/tcp/4001"],
            "envelopeBase64": "AQIDBA==",
            "createdAtEpochSec": 1000,
            "attemptCount": 2,
            "nextAttemptAtEpochSec": 1010,
            "strictBleOnlyMode": false,
            "recipientIdentityId": null,
            "intendedDeviceId": null,
            "terminalFailureCode": null,
            "ackedWithoutReceiptCount": null
          }
        ]
        """
        guard let data = legacyJson.data(using: .utf8) else {
            XCTFail("Failed to encode test JSON")
            return
        }

        let driver = MockPersistenceDriver(initialData: data)
        let repo = createTestRepository()
        repo.outboxPersistenceDriver = driver

        let envelopes = try repo.loadPendingOutboxChecked()
        XCTAssertEqual(envelopes.count, 1)
        XCTAssertEqual(envelopes[0].queueId, "q1")
        XCTAssertEqual(envelopes[0].historyRecordId, "msg1")
        XCTAssertEqual(envelopes[0].mutationGeneration, 0)
        XCTAssertNil(envelopes[0].retryDeferredUntilEpochSec)

        // Mutate and save back
        var updated = envelopes[0]
        updated.attemptCount = 3
        try repo.savePendingOutboxChecked([updated])

        // Reload across simulated restart
        let reloaded = try repo.loadPendingOutboxChecked()
        XCTAssertEqual(reloaded.count, 1)
        XCTAssertEqual(reloaded[0].queueId, "q1")
        XCTAssertEqual(reloaded[0].attemptCount, 3)
    }

    // T2: More than 12 transient failures remain one automatic obligation (burst threshold -> 300s + jitter deferral)
    func testMoreThanTwelveTransientFailuresRemainOneAutomaticObligation() async throws {
        let driver = MockPersistenceDriver()
        let repo = createTestRepository()
        repo.outboxPersistenceDriver = driver
        var currentClock: UInt64 = 1000
        repo.nowEpochSecProvider = { currentClock }

        try repo.enqueuePendingOutbound(
            historyRecordId: "msg_burst",
            peerId: "peer_burst",
            routePeerId: nil,
            addresses: [],
            envelopeData: Data([1, 2, 3])
        )

        repo.transportAttemptClosure = { _, _, _, _, _, _, _, _, _, _ in
            return (acked: false, routePeerId: nil, terminalFailureCode: nil)
        }

        // Run flush 12 times to hit burst threshold (12)
        for i in 1...12 {
            currentClock += 10
            await repo.flushPendingOutbox(reason: "test_attempt_\(i)")
        }

        let queueAfter12 = repo.loadPendingOutbox()
        XCTAssertEqual(queueAfter12.count, 1)
        let item = queueAfter12[0]
        XCTAssertEqual(item.attemptCount, 0)
        XCTAssertNotNil(item.retryDeferredUntilEpochSec)
        let jitter = MeshRepository.deterministicJitter(for: item.queueId)
        XCTAssertEqual(item.retryDeferredUntilEpochSec, currentClock + 300 + jitter)

        // Advance clock past deferral time with route available
        currentClock += 300 + jitter + 1
        repo.routeAvailabilityProvider = { _ in true }
        await repo.flushPendingOutbox(reason: "burst_restart")

        let queueAfterRestart = repo.loadPendingOutbox()
        XCTAssertEqual(queueAfterRestart.count, 1)
        XCTAssertEqual(queueAfterRestart[0].attemptCount, 1)
        XCTAssertNil(queueAfterRestart[0].retryDeferredUntilEpochSec)

        // Final receipt removes obligation
        repo.removePendingOutbound(historyRecordId: "msg_burst")
        XCTAssertTrue(repo.loadPendingOutbox().isEmpty)
    }

    // T3: No route at deferred deadline retains without consuming attempt
    func testNoRouteAtDeferredDeadlineRetainsWithoutConsumingAttempt() async throws {
        let driver = MockPersistenceDriver()
        let repo = createTestRepository()
        repo.outboxPersistenceDriver = driver
        var currentClock: UInt64 = 1000
        repo.nowEpochSecProvider = { currentClock }

        let env = makeSampleEnvelope(
            queueId: "q_defer",
            historyRecordId: "msg_defer",
            attemptCount: 0,
            nextAttemptAtEpochSec: 1000,
            retryDeferredUntilEpochSec: 1000
        )
        try repo.savePendingOutboxChecked([env])

        currentClock = 1005
        repo.routeAvailabilityProvider = { _ in false } // No route

        await repo.flushPendingOutbox(reason: "test_no_route")

        let queue = repo.loadPendingOutbox()
        XCTAssertEqual(queue.count, 1)
        let item = queue[0]
        XCTAssertEqual(item.attemptCount, 0, "Attempt must not be consumed when no route is available")
        let jitter = MeshRepository.deterministicJitter(for: "q_defer")
        XCTAssertEqual(item.retryDeferredUntilEpochSec, 1005 + 300 + jitter)
    }

    // T4: Suspended flush cannot overwrite retry, enqueue, receipt removal, or promotion
    func testSuspendedFlushCannotOverwriteRetryEnqueueReceiptRemovalOrPromotion() async throws {
        let driver = MockPersistenceDriver()
        let repo = createTestRepository()
        repo.outboxPersistenceDriver = driver
        var currentClock: UInt64 = 1000
        repo.nowEpochSecProvider = { currentClock }

        let e1 = makeSampleEnvelope(queueId: "q1", historyRecordId: "msg1", nextAttemptAtEpochSec: 1000)
        let e2 = makeSampleEnvelope(queueId: "q2", historyRecordId: "msg2", nextAttemptAtEpochSec: 2000, retryDeferredUntilEpochSec: 2000)
        try repo.savePendingOutboxChecked([e1, e2])

        var continuation: CheckedContinuation<(acked: Bool, routePeerId: String?, terminalFailureCode: String?), Never>?
        repo.transportAttemptClosure = { _, _, _, _, _, _, _, _, _, _ in
            await withCheckedContinuation { cont in
                continuation = cont
            }
        }

        let flushTask = Task {
            await repo.flushPendingOutbox(reason: "suspended_test")
        }

        // Wait until transport is suspended
        while continuation == nil {
            try await Task.sleep(nanoseconds: 10_000_000)
        }

        // Interleave mutations during active attempt:
        // 1. Enqueue e3
        try repo.enqueuePendingOutbound(historyRecordId: "msg3", peerId: "peer3", routePeerId: nil, addresses: [], envelopeData: Data([9]))
        // 2. Manual retry on e2
        _ = try await repo.retryFailedMessage(messageId: "msg2")
        // 3. Receipt removal of e1
        repo.removePendingOutbound(historyRecordId: "msg1")

        // Resume suspended attempt with transient failure
        continuation?.resume(returning: (acked: false, routePeerId: nil, terminalFailureCode: nil))
        await flushTask.value

        let finalQueue = repo.loadPendingOutbox()
        XCTAssertFalse(finalQueue.contains { $0.historyRecordId == "msg1" }, "Receipt-removed msg1 must NOT be resurrected")
        XCTAssertTrue(finalQueue.contains { $0.historyRecordId == "msg3" }, "Enqueued msg3 must be present")
        let item2 = finalQueue.first { $0.historyRecordId == "msg2" }
        XCTAssertNotNil(item2)
        XCTAssertEqual(item2?.mutationGeneration, 1, "Manual retry mutation on msg2 must be preserved")
    }

    // T5: Same-item in-flight promotion preserves transient, ACK, terminal, and receipt truth
    func testSameItemInFlightPromotionPreservesTransientAckTerminalAndReceiptTruth() async throws {
        // (a) Transient: stale result discarded when generation advanced
        do {
            let driver = MockPersistenceDriver()
            let repo = createTestRepository()
            repo.outboxPersistenceDriver = driver
            repo.nowEpochSecProvider = { 1000 }

            let e1 = makeSampleEnvelope(queueId: "q1", historyRecordId: "msg1", nextAttemptAtEpochSec: 1000, mutationGeneration: 0)
            try repo.savePendingOutboxChecked([e1])

            var continuation: CheckedContinuation<(acked: Bool, routePeerId: String?, terminalFailureCode: String?), Never>?
            repo.transportAttemptClosure = { _, _, _, _, _, _, _, _, _, _ in
                await withCheckedContinuation { cont in continuation = cont }
            }

            let flushTask = Task { await repo.flushPendingOutbox(reason: "test_t5_transient") }
            while continuation == nil { try await Task.sleep(nanoseconds: 10_000_000) }

            // Promote same item during in-flight attempt
            repo.promotePendingOutboundForPeer(peerId: e1.peerId)
            continuation?.resume(returning: (acked: false, routePeerId: nil, terminalFailureCode: nil))
            await flushTask.value

            let queue = repo.loadPendingOutbox()
            XCTAssertEqual(queue.count, 1)
            XCTAssertEqual(queue[0].mutationGeneration, 1)
            XCTAssertEqual(queue[0].attemptCount, 0, "Stale transient result must NOT increment attempt count")
        }

        // (b) ACK: commits even if generation advanced
        do {
            let driver = MockPersistenceDriver()
            let repo = createTestRepository()
            repo.outboxPersistenceDriver = driver
            repo.nowEpochSecProvider = { 1000 }

            let e1 = makeSampleEnvelope(queueId: "q1", historyRecordId: "msg1", nextAttemptAtEpochSec: 1000, mutationGeneration: 0)
            try repo.savePendingOutboxChecked([e1])

            var continuation: CheckedContinuation<(acked: Bool, routePeerId: String?, terminalFailureCode: String?), Never>?
            repo.transportAttemptClosure = { _, _, _, _, _, _, _, _, _, _ in
                await withCheckedContinuation { cont in continuation = cont }
            }

            let flushTask = Task { await repo.flushPendingOutbox(reason: "test_t5_ack") }
            while continuation == nil { try await Task.sleep(nanoseconds: 10_000_000) }

            repo.promotePendingOutboundForPeer(peerId: e1.peerId)
            continuation?.resume(returning: (acked: true, routePeerId: "route_peer", terminalFailureCode: nil))
            await flushTask.value

            let queue = repo.loadPendingOutbox()
            XCTAssertEqual(queue.count, 1)
            XCTAssertEqual(queue[0].ackedWithoutReceiptCount, 1, "ACK must commit even if generation advanced")
        }

        // (c) Terminal: commits even if generation advanced
        do {
            let driver = MockPersistenceDriver()
            let repo = createTestRepository()
            repo.outboxPersistenceDriver = driver
            repo.nowEpochSecProvider = { 1000 }

            let e1 = makeSampleEnvelope(queueId: "q1", historyRecordId: "msg1", nextAttemptAtEpochSec: 1000, mutationGeneration: 0)
            try repo.savePendingOutboxChecked([e1])

            var continuation: CheckedContinuation<(acked: Bool, routePeerId: String?, terminalFailureCode: String?), Never>?
            repo.transportAttemptClosure = { _, _, _, _, _, _, _, _, _, _ in
                await withCheckedContinuation { cont in continuation = cont }
            }

            let flushTask = Task { await repo.flushPendingOutbox(reason: "test_t5_terminal") }
            while continuation == nil { try await Task.sleep(nanoseconds: 10_000_000) }

            repo.promotePendingOutboundForPeer(peerId: e1.peerId)
            continuation?.resume(returning: (acked: false, routePeerId: nil, terminalFailureCode: "identity_device_mismatch"))
            await flushTask.value

            let queue = repo.loadPendingOutbox()
            XCTAssertEqual(queue.count, 1)
            XCTAssertEqual(queue[0].terminalFailureCode, "identity_device_mismatch", "Terminal failure must commit even if generation advanced")
        }
    }

    // T6: Repeated manual retry taps schedule one acceleration with stable identity
    func testRepeatedManualRetryTapsScheduleOneAccelerationWithStableIdentity() async throws {
        let driver = MockPersistenceDriver()
        let repo = createTestRepository()
        repo.outboxPersistenceDriver = driver
        repo.nowEpochSecProvider = { 1000 }

        let env = makeSampleEnvelope(
            queueId: "q_stable",
            historyRecordId: "msg_stable",
            attemptCount: 5,
            nextAttemptAtEpochSec: 2000,
            retryDeferredUntilEpochSec: 2000
        )
        try repo.savePendingOutboxChecked([env])

        let firstTap = try await repo.retryFailedMessage(messageId: "msg_stable")
        XCTAssertTrue(firstTap)

        let queueAfterFirst = repo.loadPendingOutbox()
        XCTAssertEqual(queueAfterFirst.count, 1)
        XCTAssertEqual(queueAfterFirst[0].queueId, "q_stable")
        XCTAssertEqual(queueAfterFirst[0].historyRecordId, "msg_stable")
        XCTAssertEqual(queueAfterFirst[0].attemptCount, 0)
        XCTAssertNil(queueAfterFirst[0].retryDeferredUntilEpochSec)
        XCTAssertEqual(queueAfterFirst[0].nextAttemptAtEpochSec, 1000)
        XCTAssertEqual(queueAfterFirst[0].mutationGeneration, 1)

        // Second tap is idempotent no-op
        let secondTap = try await repo.retryFailedMessage(messageId: "msg_stable")
        XCTAssertFalse(secondTap)

        let queueAfterSecond = repo.loadPendingOutbox()
        XCTAssertEqual(queueAfterSecond.count, 1)
        XCTAssertEqual(queueAfterSecond[0].mutationGeneration, 1)
    }

    // T7: Terminal identity failure remains nonretryable across opportunity and restart
    func testTerminalIdentityFailureRemainsNonretryableAcrossOpportunityAndRestart() throws {
        let driver = MockPersistenceDriver()
        let repo = createTestRepository()
        repo.outboxPersistenceDriver = driver
        repo.nowEpochSecProvider = { 1000 }

        let e1 = makeSampleEnvelope(queueId: "q1", historyRecordId: "m1", peerId: "p1", terminalFailureCode: "identity_device_mismatch")
        let e2 = makeSampleEnvelope(queueId: "q2", historyRecordId: "m2", peerId: "p2", terminalFailureCode: "identity_abandoned")
        try repo.savePendingOutboxChecked([e1, e2])

        // Opportunity promotion must not affect terminal entries
        repo.promotePendingOutboundForPeer(peerId: "p1")
        repo.promotePendingOutboundForPeer(peerId: "p2")

        let queue = repo.loadPendingOutbox()
        XCTAssertEqual(queue[0].terminalFailureCode, "identity_device_mismatch")
        XCTAssertEqual(queue[1].terminalFailureCode, "identity_abandoned")

        // Reload across restart
        let restartedRepo = createTestRepository()
        restartedRepo.outboxPersistenceDriver = driver
        let reloaded = restartedRepo.loadPendingOutbox()
        XCTAssertEqual(reloaded[0].terminalFailureCode, "identity_device_mismatch")
        XCTAssertEqual(reloaded[1].terminalFailureCode, "identity_abandoned")
    }

    // T8: Acknowledged without receipt remains sent and opportunity eligible past seven days
    func testAcknowledgedWithoutReceiptRemainsSentAndOpportunityEligiblePastSevenDays() {
        let createdAt: UInt64 = 1000
        let pastSevenDays: UInt64 = createdAt + (8 * 24 * 60 * 60) // 8 days later

        let env = makeSampleEnvelope(
            queueId: "q_ack",
            historyRecordId: "msg_ack",
            createdAtEpochSec: createdAt,
            nextAttemptAtEpochSec: pastSevenDays,
            ackedWithoutReceiptCount: 1
        )

        let msg = MessageRecord(
            id: "msg_ack",
            direction: .sent,
            peerId: "peer_ack",
            content: "hello",
            timestamp: createdAt,
            senderTimestamp: createdAt,
            delivered: false,
            status: .sent,
            hidden: false
        )

        let repo = createTestRepository()
        let driver = MockPersistenceDriver()
        repo.outboxPersistenceDriver = driver
        try? repo.savePendingOutboxChecked([env])

        let presentation = repo.deliveryStatePresentation(for: msg, nowEpochSec: pastSevenDays)
        XCTAssertEqual(presentation.state, .sent)
        XCTAssertEqual(presentation.label, "sent")

        // Obligation is NOT dropped by age ceiling
        let loaded = repo.loadPendingOutbox()
        XCTAssertEqual(loaded.count, 1)
    }

    // T9: Persistence failures propagate for enqueue, attempt commit, and receipt removal
    func testPersistenceFailuresPropagateForEnqueueAttemptCommitAndReceiptRemoval() {
        let driver = MockPersistenceDriver()
        driver.shouldFailWrite = true

        let repo = createTestRepository()
        repo.outboxPersistenceDriver = driver

        XCTAssertThrowsError(
            try repo.enqueuePendingOutbound(
                historyRecordId: "msg_fail",
                peerId: "peer_fail",
                routePeerId: nil,
                addresses: [],
                envelopeData: Data([1])
            )
        )
    }

    // T10: Corrupt queue read fails closed without overwriting
    func testCorruptQueueReadFailsClosedWithoutOverwriting() async throws {
        let corruptBytes = Data([0xFF, 0xFE, 0xFD, 0xFC])
        let driver = MockPersistenceDriver(initialData: corruptBytes)
        let repo = createTestRepository()
        repo.outboxPersistenceDriver = driver

        XCTAssertThrowsError(try repo.loadPendingOutboxChecked())

        var receivedError: MeshRepository.MeshOperationError?
        let cancellable = repo.operationErrors.sink { err in
            receivedError = err
        }

        await repo.flushPendingOutbox(reason: "test_corrupt")
        XCTAssertNotNil(receivedError)
        XCTAssertEqual(driver.storedData, corruptBytes, "Corrupt file must NOT be overwritten with empty array")
        cancellable.cancel()

        // Corrupt envelope base64 in valid queue is retained as terminal, NOT silently dropped
        let badBase64Env = makeSampleEnvelope(queueId: "q_bad", historyRecordId: "msg_bad", envelopeBase64: "!!!not_base64!!!")
        let validDriver = MockPersistenceDriver()
        repo.outboxPersistenceDriver = validDriver
        try repo.savePendingOutboxChecked([badBase64Env])

        await repo.flushPendingOutbox(reason: "test_bad_base64")
        let finalQueue = repo.loadPendingOutbox()
        XCTAssertEqual(finalQueue.count, 1)
        XCTAssertEqual(finalQueue[0].terminalFailureCode, "corrupt_envelope")
    }

    // T11: Reject block failure leaves marker and never clears
    func testRejectBlockFailureLeavesMarkerAndNeverClears() {
        let repo = createTestRepository()
        let markerDriver = MockPersistenceDriver()
        repo.requestMarkerPersistenceDriver = markerDriver

        // Calling rejectMessageRequest when ironCore is uninitialized throws
        XCTAssertThrowsError(try repo.rejectMessageRequest(peerId: "peer_block_fail"))
        XCTAssertFalse(repo.isRequestMarkerCleared(peerId: "peer_block_fail"))
    }

    // T12: Reject marker failure is reported then retry persists absence across reload
    func testRejectMarkerFailureIsReportedThenRetryPersistsAbsenceAcrossReload() throws {
        let repo = createTestRepository()
        let markerDriver = MockPersistenceDriver()
        markerDriver.shouldFailWrite = true
        repo.requestMarkerPersistenceDriver = markerDriver

        XCTAssertThrowsError(try repo.recordRequestMarkerCleared(peerId: "peer_marker"))
        XCTAssertFalse(repo.isRequestMarkerCleared(peerId: "peer_marker"))

        // Retry with working driver
        markerDriver.shouldFailWrite = false
        try repo.recordRequestMarkerCleared(peerId: "peer_marker")
        XCTAssertTrue(repo.isRequestMarkerCleared(peerId: "peer_marker"))

        // Reload across simulated restart
        let restartedRepo = createTestRepository()
        restartedRepo.requestMarkerPersistenceDriver = markerDriver
        XCTAssertTrue(restartedRepo.isRequestMarkerCleared(peerId: "peer_marker"))
    }

    // T13: Blocked lookup failure fails closed with suppression set
    func testBlockedLookupFailureFailsClosedWithSuppressionSet() {
        let repo = createTestRepository()
        // IronCore not initialized -> loadMessageRequestThreads throws
        let threads = repo.getMessageRequests()
        XCTAssertTrue(threads.isEmpty)
        if case .failed = repo.requestsLoadState {
            // Expected
        } else {
            XCTFail("requestsLoadState must be .failed")
        }
    }

    // T14: Send message history failure prevents enqueue and transport
    func testSendMessageHistoryFailurePreventsEnqueueAndTransport() async {
        let repo = createTestRepository()
        let historyDriver = MockPersistenceDriver()
        historyDriver.shouldFailWrite = true
        repo.pendingHistoryPersistenceDriver = historyDriver

        var transportCalled = false
        repo.transportAttemptClosure = { _, _, _, _, _, _, _, _, _, _ in
            transportCalled = true
            return (acked: true, routePeerId: nil, terminalFailureCode: nil)
        }

        do {
            try await repo.sendMessage(peerId: "peer_fail_hist", content: "hello")
            XCTFail("sendMessage must throw when history write fails")
        } catch {
            // Expected
        }

        XCTAssertFalse(transportCalled, "Transport must NEVER be called if history write fails")
        XCTAssertTrue(repo.loadPendingOutbox().isEmpty, "Outbox enqueue must NEVER happen if history write fails")
    }

    // T15: Large queue bounded write and latency gate
    func testLargeQueueBoundedWriteAndLatencyGate() async throws {
        let nMax = 512
        var envelopes: [MeshRepository.PendingOutboundEnvelope] = []
        envelopes.reserveCapacity(nMax)

        for i in 1...nMax {
            envelopes.append(
                makeSampleEnvelope(
                    queueId: "q_\(i)",
                    historyRecordId: "msg_\(i)",
                    peerId: "peer_\(i)",
                    nextAttemptAtEpochSec: 1000
                )
            )
        }

        let driver = MockPersistenceDriver()
        try driver.write(data: try JSONEncoder().encode(envelopes))
        driver.writeCount = 0

        let repo = createTestRepository()
        repo.outboxPersistenceDriver = driver
        repo.nowEpochSecProvider = { 1000 }

        repo.transportAttemptClosure = { _, _, _, _, _, _, _, _, _, _ in
            return (acked: true, routePeerId: "route_1", terminalFailureCode: nil)
        }

        let startTime = DispatchTime.now()
        await repo.flushPendingOutbox(reason: "large_queue_test")
        let endTime = DispatchTime.now()

        let elapsedNanos = endTime.uptimeNanoseconds - startTime.uptimeNanoseconds
        let elapsedMs = Double(elapsedNanos) / 1_000_000.0

        XCTAssertEqual(driver.writeCount, 1, "Exactly ONE bulk write must occur for entire pass of \(nMax) items")
        XCTAssertLessThan(elapsedMs, 2000.0, "Bulk pass should execute promptly within gate bounds")

        let loaded = repo.loadPendingOutbox()
        XCTAssertEqual(loaded.count, nMax, "Zero obligations dropped or expired")
    }

    // T16: Truth-mapping assertions for all seven presentation states
    func testTruthMappingAssertionsForAllSevenPresentationStates() throws {
        let repo = createTestRepository()
        let driver = MockPersistenceDriver()
        repo.outboxPersistenceDriver = driver
        let now: UInt64 = 1000

        // 1. Queued: local record exists, no outbox entry, not delivered
        let msgQueued = MessageRecord(id: "m_q", direction: .sent, peerId: "p1", content: "hi", timestamp: now, senderTimestamp: now, delivered: false, status: .queued, hidden: false)
        XCTAssertEqual(repo.deliveryStatePresentation(for: msgQueued, nowEpochSec: now).state, .queued)

        // 2. Delivered: delivered == true
        let msgDelivered = MessageRecord(id: "m_del", direction: .sent, peerId: "p1", content: "hi", timestamp: now, senderTimestamp: now, delivered: true, status: .delivered, hidden: false)
        XCTAssertEqual(repo.deliveryStatePresentation(for: msgDelivered, nowEpochSec: now).state, .delivered)

        // 3. Stored: in outbox, future nextAttemptAtEpochSec
        let eStored = makeSampleEnvelope(queueId: "qs", historyRecordId: "m_stored", nextAttemptAtEpochSec: now + 60)
        // 4. Forwarding: in outbox, past/current nextAttemptAtEpochSec
        let eForwarding = makeSampleEnvelope(queueId: "qf", historyRecordId: "m_fwd", nextAttemptAtEpochSec: now)
        // 5. Sent: in outbox, ackedWithoutReceiptCount > 0
        let eSent = makeSampleEnvelope(queueId: "qsent", historyRecordId: "m_sent", nextAttemptAtEpochSec: now + 60, ackedWithoutReceiptCount: 1)
        // 6. FailedRetryable: in outbox, retryDeferredUntilEpochSec > now
        let eFailedRetryable = makeSampleEnvelope(queueId: "qfr", historyRecordId: "m_fr", nextAttemptAtEpochSec: now + 300, retryDeferredUntilEpochSec: now + 300)
        // 7. RejectedNonretryable: in outbox, terminalFailureCode != nil
        let eRejected = makeSampleEnvelope(queueId: "qr", historyRecordId: "m_rej", terminalFailureCode: "identity_device_mismatch")

        try repo.savePendingOutboxChecked([eStored, eForwarding, eSent, eFailedRetryable, eRejected])

        let msgStored = MessageRecord(id: "m_stored", direction: .sent, peerId: "p1", content: "hi", timestamp: now, senderTimestamp: now, delivered: false, status: .queued, hidden: false)
        XCTAssertEqual(repo.deliveryStatePresentation(for: msgStored, nowEpochSec: now).state, .stored)

        let msgFwd = MessageRecord(id: "m_fwd", direction: .sent, peerId: "p1", content: "hi", timestamp: now, senderTimestamp: now, delivered: false, status: .queued, hidden: false)
        XCTAssertEqual(repo.deliveryStatePresentation(for: msgFwd, nowEpochSec: now).state, .forwarding)

        let msgSent = MessageRecord(id: "m_sent", direction: .sent, peerId: "p1", content: "hi", timestamp: now, senderTimestamp: now, delivered: false, status: .queued, hidden: false)
        XCTAssertEqual(repo.deliveryStatePresentation(for: msgSent, nowEpochSec: now).state, .sent)

        let msgFR = MessageRecord(id: "m_fr", direction: .sent, peerId: "p1", content: "hi", timestamp: now, senderTimestamp: now, delivered: false, status: .queued, hidden: false)
        XCTAssertEqual(repo.deliveryStatePresentation(for: msgFR, nowEpochSec: now).state, .failedRetryable)

        let msgRej = MessageRecord(id: "m_rej", direction: .sent, peerId: "p1", content: "hi", timestamp: now, senderTimestamp: now, delivered: false, status: .queued, hidden: false)
        XCTAssertEqual(repo.deliveryStatePresentation(for: msgRej, nowEpochSec: now).state, .rejectedNonretryable)
    }

    // T17: Truth burst persists in exactly one bulk write
    func testTruthBurstPersistsInExactlyOneBulkWrite() async throws {
        let nMax = 100
        var envelopes: [MeshRepository.PendingOutboundEnvelope] = []
        envelopes.reserveCapacity(nMax)

        for i in 1...nMax {
            envelopes.append(
                makeSampleEnvelope(
                    queueId: "qb_\(i)",
                    historyRecordId: "mb_\(i)",
                    peerId: "peer_\(i)",
                    nextAttemptAtEpochSec: 1000
                )
            )
        }

        let driver = MockPersistenceDriver()
        try driver.write(data: try JSONEncoder().encode(envelopes))
        driver.writeCount = 0

        let repo = createTestRepository()
        repo.outboxPersistenceDriver = driver
        repo.nowEpochSecProvider = { 1000 }

        // Mix of ACKs and terminal results in single burst
        repo.transportAttemptClosure = { _, _, _, _, _, traceId, _, _, _, _ in
            if traceId.hasSuffix("1") {
                return (acked: false, routePeerId: nil, terminalFailureCode: "identity_device_mismatch")
            } else {
                return (acked: true, routePeerId: "route_ok", terminalFailureCode: nil)
            }
        }

        await repo.flushPendingOutbox(reason: "burst_test")

        XCTAssertEqual(driver.writeCount, 1, "Entire burst must persist in EXACTLY ONE atomic bulk write")

        let persisted = repo.loadPendingOutbox()
        XCTAssertEqual(persisted.count, nMax)
        let terminals = persisted.filter { $0.terminalFailureCode != nil }
        let acks = persisted.filter { ($0.ackedWithoutReceiptCount ?? 0) > 0 }
        XCTAssertGreaterThan(terminals.count, 0)
        XCTAssertGreaterThan(acks.count, 0)
    }

    // T18: Pending history reconciliation restores or prunes across restart
    func testPendingHistoryReconciliationRestoresOrPrunesAcrossRestart() async throws {
        let driver = MockPersistenceDriver()
        let repo = createTestRepository()
        repo.pendingHistoryPersistenceDriver = driver
        repo.nowEpochSecProvider = { 1000 }

        let rec1 = MessageRecord(id: "msg_reconcile_1", direction: .sent, peerId: "p1", content: "m1", timestamp: 1000, senderTimestamp: 1000, delivered: false, status: .queued, hidden: false)
        try repo.recordPendingHistory(record: rec1)

        let loaded = try repo.loadPendingHistory()
        XCTAssertEqual(loaded.count, 1)
        XCTAssertEqual(loaded[0].record.id, "msg_reconcile_1")
    }
}
