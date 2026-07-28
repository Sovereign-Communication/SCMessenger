//
//  NotificationLogger.swift
//  SCMessenger
//
//  Verification logging for notification testing
//

import Foundation
import os

/// Logger for notification verification tests
final class NotificationLogger {
    static let shared: NotificationLogger = NotificationLogger()

    private let logger: OSLog = OSLog(subsystem: "com.scmessenger.notification", category: "Verification")
    private let logFileURL: URL

    private init() {
        // Create logs directory in app's documents folder
        // urls(for:in:) always returns at least one element for .documentDirectory
        // swiftlint:disable:next force_unwrapping
        let documentsPath = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first!
        let logsDirectory = documentsPath.appendingPathComponent("NotificationLogs", isDirectory: true)

        do {
            try FileManager.default.createDirectory(at: logsDirectory, withIntermediateDirectories: true, attributes: nil)
            logFileURL = logsDirectory.appendingPathComponent("notification_verification_\(Date().iso8601StringFormatter.string(from: Date())).log")
        } catch {
            // Fallback to console-only logging
            logFileURL = documentsPath.appendingPathComponent("notification_verification.log")
            print("Failed to create log file: \(error)")
        }
    }

    /// Log a verification event with timestamp
    func log(_ message: String) {
        let timestamp = Date().iso8601StringFormatter.string(from: Date())
        let entry: String = "[\(timestamp)] \(message)\n"

        // Log to OSLog
        os_log("%{public}s", log: logger, type: .info, message)

        // Log to file
        appendToFile(entry)
    }

    /// Log a test result
    func logTestResult(_ test: String, passed: Bool, details: String? = nil) {
        let status = passed ? "PASS" : "FAIL"
        var message: String = "\(test): \(status)"

        if let details = details, !details.isEmpty {
            message += " - \(details)"
        }

        log(message)
    }

    /// Log permission verification
    func logPermissionResult(_ result: PermissionVerificationResult) {
        let status: String
        switch result.status {
        case .authorized:
            status = "AUTHORIZED"
        case .denied:
            status = "DENIED"
        case .notDetermined:
            status = "NOT_DETERMINED"
        case .unknown:
            status = "UNKNOWN"
        }

        let action: String
        switch result.requiredAction {
        case .none:
            action = "NONE"
        case .requested:
            action = "REQUESTED"
        case .manualEnable:
            action = "MANUAL_ENABLE"
        }

        log("Permission: \(status) - Action: \(action) - Success: \(result.success)")
    }

    /// Log notification delivery test
    func logDeliveryTest(_ test: String, delivered: Bool, delay: TimeInterval? = nil) {
        var message: String = "Delivery Test '\(test)': \(delivered ? "DELIVERED" : "NOT_DELIVERED")"
        if let delay = delay {
            message += " - Delay: \(String(format: "%.2f", delay))s"
        }
        log(message)
    }

    /// Log notification interaction test
    func logInteractionTest(_ test: String, handled: Bool) {
        log("Interaction Test '\(test)': \(handled ? "HANDLED" : "NOT_HANDLED")")
    }

    /// Get all log entries
    func getLogEntries() -> [String] {
        guard FileManager.default.fileExists(atPath: logFileURL.path) else {
            return []
        }

        do {
            let content = try String(contentsOf: logFileURL, encoding: .utf8)
            return content.components(separatedBy: .newlines).filter { !$0.isEmpty }
        } catch {
            print("Failed to read log file: \(error)")
            return []
        }
    }

    /// Clear all log entries
    func clearLogs() {
        do {
            try FileManager.default.removeItem(at: logFileURL)
        } catch {
            print("Failed to clear logs: \(error)")
        }
    }

    // MARK: - Private Helpers

    private func appendToFile(_ content: String) {
        if !FileManager.default.fileExists(atPath: logFileURL.path) {
            // Create file if it doesn't exist
            FileManager.default.createFile(atPath: logFileURL.path, contents: nil, attributes: nil)
        }

        guard let handle = FileHandle(forWritingAtPath: logFileURL.path) else {
            print("Failed to open log file for writing")
            return
        }

        defer {
            handle.closeFile()
        }

        handle.seekToEndOfFile()
        handle.write(content.data(using: .utf8) ?? Data())
    }
}

// MARK: - ISO8601 Date Formatter

extension Date {
    private static let sharedISO8601Formatter: ISO8601DateFormatter = {
        let formatter = Foundation.ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        return formatter
    }()

    var iso8601StringFormatter: ISO8601DateFormatter {
        Self.sharedISO8601Formatter
    }
}
