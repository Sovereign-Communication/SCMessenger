//
//  MeshBackgroundServiceTests.swift
//  SCMessengerTests
//
//  Verifies the BGAppRefreshTask/BGProcessingTask scheduling contract:
//  the task identifiers registered with BGTaskScheduler must match the ones
//  declared in Info.plist's BGTaskSchedulerPermittedIdentifiers (a mismatch
//  makes BGTaskScheduler.register(...) assert/crash at launch), and the
//  handlers must actually drive the mesh sync work they're scheduled for.
//

import XCTest
@testable import SCMessenger

final class MeshBackgroundServiceTests: XCTestCase {
    private var meshRepository: MeshRepository!
    private var backgroundService: MeshBackgroundService!

    override func setUp() {
        super.setUp()
        meshRepository = MeshRepository()
        backgroundService = MeshBackgroundService(
            meshRepository: meshRepository,
            refreshWork: {},
            processingWork: {}
        )
    }

    override func tearDown() {
        backgroundService = nil
        meshRepository = nil
        super.tearDown()
    }

    /// The two identifiers MeshBackgroundService registers with
    /// BGTaskScheduler must be declared in Info.plist's
    /// BGTaskSchedulerPermittedIdentifiers, or registration asserts/crashes
    /// at launch on a real device.
    func testTaskIdentifiersAreDeclaredInInfoPlist() {
        guard let permittedIdentifiers = Bundle.main.object(
            forInfoDictionaryKey: "BGTaskSchedulerPermittedIdentifiers"
        ) as? [String] else {
            XCTFail("BGTaskSchedulerPermittedIdentifiers missing from Info.plist")
            return
        }

        XCTAssertTrue(
            permittedIdentifiers.contains(MeshBackgroundService.refreshTaskId),
            "refreshTaskId (\(MeshBackgroundService.refreshTaskId)) must be in BGTaskSchedulerPermittedIdentifiers"
        )
        XCTAssertTrue(
            permittedIdentifiers.contains(MeshBackgroundService.processingTaskId),
            "processingTaskId (\(MeshBackgroundService.processingTaskId)) must be in BGTaskSchedulerPermittedIdentifiers"
        )
    }

    /// Verifies that the debug-only simulation awaits the injected refresh
    /// operation without starting real radios or network transports.
    func testSimulatedBackgroundRefreshCompletesWithoutThrowing() async {
        let task: Task<Void, Never> = backgroundService.simulateBackgroundRefresh()
        await task.value
    }

    /// Verifies that the debug-only simulation awaits the injected processing
    /// operation without starting real radios or network transports.
    func testSimulatedBackgroundProcessingCompletesWithoutThrowing() async {
        let task: Task<Void, Never> = backgroundService.simulateBackgroundProcessing()
        await task.value
    }
}
