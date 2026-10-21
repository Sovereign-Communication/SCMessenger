//
//  LocalTransportFallback.swift
//  SCMessenger
//
//  Deterministic local transport fallback ordering for iOS.
//

import Foundation

struct LocalTransportFallbackResult: Equatable {
    let multipeerAttempted: Bool
    let multipeerAcked: Bool
    let bleAttempted: Bool
    let bleAcked: Bool

    var acked: Bool {
        multipeerAcked || bleAcked
    }
}

enum LocalTransportFallback {
    static func attemptMultipeerThenBle(
        multipeerPeerId: String?,
        blePeerId: String?,
        tryMultipeer: (String) -> Bool,
        tryBle: (String) -> Bool
    ) -> LocalTransportFallbackResult {
        let normalizedMultipeer: String? = {
            guard let value = multipeerPeerId?.trimmingCharacters(in: .whitespacesAndNewlines),
                  !value.isEmpty else {
                return nil
            }
            return value
        }()
        if let normalizedMultipeer {
            if tryMultipeer(normalizedMultipeer) {
                return LocalTransportFallbackResult(
                    multipeerAttempted: true,
                    multipeerAcked: true,
                    bleAttempted: false,
                    bleAcked: false
                )
            }
        }

        let normalizedBle: String? = {
            guard let value = blePeerId?.trimmingCharacters(in: .whitespacesAndNewlines),
                  !value.isEmpty else {
                return nil
            }
            return value
        }()
        if let normalizedBle {
            let bleAcked: Bool = tryBle(normalizedBle)
            return LocalTransportFallbackResult(
                multipeerAttempted: normalizedMultipeer != nil,
                multipeerAcked: false,
                bleAttempted: true,
                bleAcked: bleAcked
            )
        }

        return LocalTransportFallbackResult(
            multipeerAttempted: normalizedMultipeer != nil,
            multipeerAcked: false,
            bleAttempted: false,
            bleAcked: false
        )
    }
}
