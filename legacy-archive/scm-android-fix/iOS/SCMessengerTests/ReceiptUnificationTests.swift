//
//  ReceiptUnificationTests.swift
//  SCMessengerTests
//
//  Tests unified receipt encoding/decoding via core's FFI functions.
//  Mirrors: A-04 Android ReceiptUnificationTest.kt pattern
//
//  These tests verify that receipt encoding/decoding uses the unified core functions
//  across all platforms, ensuring consistent wire format and preventing platform-specific bugs.
//

import XCTest
@testable import SCMessenger

final class ReceiptUnificationTests: XCTestCase {

    /// Test round-trip receipt encoding and decoding using unified core functions
    /// - Encodes a Receipt via core's encodeReceipt(receipt:)
    /// - Parses it back via core's decodeReceipt(data:)
    /// - Verifies no data loss in round-trip
    func testRoundTripReceiptEncoding() throws {
        let messageId: String = "test-message-123"
        let timestamp: UInt64 = 1234567890
        let originalReceipt = Receipt(
            messageId: messageId,
            status: .delivered,
            timestamp: timestamp
        )

        // Encode via core function (unified across all platforms)
        let encodedData: Data = try encodeReceipt(receipt: originalReceipt)

        // Verify encoded data is not empty
        XCTAssertGreaterThan(
            encodedData.count,
            0,
            "Encoded receipt should not be empty"
        )

        // Decode back via core function (unified across all platforms)
        let decodedReceipt: Receipt = try decodeReceipt(data: encodedData)

        // Verify round-trip integrity
        XCTAssertEqual(decodedReceipt, originalReceipt)
    }

    /// Test encoding different delivery statuses
    func testEncodeReceiptWithDifferentStatuses() throws {
        let messageId: String = "msg-456"
        let timestamp: UInt64 = 9876543210

        let statuses: [DeliveryStatus] = [.sent, .delivered, .read, .failed]

        for status in statuses {
            let originalReceipt = Receipt(
                messageId: messageId,
                status: status,
                timestamp: timestamp
            )

            let encodedData: Data = try encodeReceipt(receipt: originalReceipt)
            let decodedReceipt: Receipt = try decodeReceipt(data: encodedData)

            XCTAssertEqual(
                decodedReceipt,
                originalReceipt,
                "Status '\(status)' should round-trip correctly"
            )
        }
    }

    /// Test decoding invalid data raises error
    func testDecodingInvalidDataThrows() throws {
        let invalidData: Data = Data([0xFF, 0xFE, 0xFD])

        XCTAssertThrowsError(
            try decodeReceipt(data: invalidData),
            "Decoding invalid data should throw"
        )
    }
}
