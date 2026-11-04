import Foundation

/// The two core calls made when the platform reports a network path update.
/// `MeshService` conforms; tests substitute a recording fake.
protocol NetworkReportBridge: AnyObject {
    func onNetworkChanged(hasWifi: Bool, hasCellular: Bool)
    func updateDeviceState(profile: DeviceProfile)
}

extension MeshService: NetworkReportBridge {}

/// Decides whether a network report is a real interface change and performs
/// the core calls in the required order (issue #523).
enum NetworkReportGate {
    /// A report is an event only when wifi or cellular actually changed.
    static func isRealChange(
        previousWifi: Bool,
        previousCellular: Bool,
        wifi: Bool,
        cellular: Bool
    ) -> Bool {
        wifi != previousWifi || cellular != previousCellular
    }

    /// Order matters: the core records the new interface state in
    /// `onNetworkChanged`, and `updateDeviceState` echoes it back to the
    /// platform. Reporting the change first keeps that echo consistent.
    static func report(
        previousWifi: Bool,
        previousCellular: Bool,
        wifi: Bool,
        cellular: Bool,
        profile: DeviceProfile,
        bridge: NetworkReportBridge?
    ) {
        if isRealChange(
            previousWifi: previousWifi,
            previousCellular: previousCellular,
            wifi: wifi,
            cellular: cellular
        ) {
            bridge?.onNetworkChanged(hasWifi: wifi, hasCellular: cellular)
        }
        bridge?.updateDeviceState(profile: profile)
    }
}
