import XCTest
@testable import SCMessenger

final class NetworkReportGateTests: XCTestCase {
    private final class FakeBridge: NetworkReportBridge {
        var calls: [String] = []
        func onNetworkChanged(hasWifi: Bool, hasCellular: Bool) {
            calls.append("onNetworkChanged(\(hasWifi),\(hasCellular))")
        }
        func updateDeviceState(profile: DeviceProfile) {
            calls.append("updateDeviceState")
        }
    }

    private func profile(wifi: Bool) -> DeviceProfile {
        DeviceProfile(
            peerId: nil,
            deviceId: nil,
            batteryPct: 50,
            isCharging: false,
            hasWifi: wifi,
            motionState: .unknown
        )
    }

    private func report(
        from previous: (wifi: Bool, cellular: Bool),
        to next: (wifi: Bool, cellular: Bool),
        bridge: FakeBridge
    ) {
        NetworkReportGate.report(
            previousWifi: previous.wifi,
            previousCellular: previous.cellular,
            wifi: next.wifi,
            cellular: next.cellular,
            profile: profile(wifi: next.wifi),
            bridge: bridge
        )
    }

    func testNoOpReportForwardsNoNetworkChange() {
        let bridge = FakeBridge()
        report(from: (true, false), to: (true, false), bridge: bridge)
        XCTAssertFalse(bridge.calls.contains { $0.hasPrefix("onNetworkChanged") })
        XCTAssertEqual(bridge.calls, ["updateDeviceState"])
    }

    func testRealChangeForwardsExactlyOnce() {
        let bridge = FakeBridge()
        report(from: (true, false), to: (false, true), bridge: bridge)
        XCTAssertEqual(bridge.calls.filter { $0.hasPrefix("onNetworkChanged") }.count, 1)
        XCTAssertEqual(bridge.calls.first, "onNetworkChanged(false,true)")
    }

    func testNetworkLossIsARealChange() {
        let bridge = FakeBridge()
        report(from: (true, false), to: (false, false), bridge: bridge)
        XCTAssertEqual(bridge.calls.filter { $0.hasPrefix("onNetworkChanged") }.count, 1)
    }

    func testOnNetworkChangedIsCalledBeforeUpdateDeviceState() {
        let bridge = FakeBridge()
        report(from: (false, false), to: (true, false), bridge: bridge)
        XCTAssertEqual(bridge.calls, ["onNetworkChanged(true,false)", "updateDeviceState"])
    }

    func testMissingBridgeIsSafe() {
        NetworkReportGate.report(
            previousWifi: false,
            previousCellular: false,
            wifi: true,
            cellular: false,
            profile: profile(wifi: true),
            bridge: nil
        )
    }
}
