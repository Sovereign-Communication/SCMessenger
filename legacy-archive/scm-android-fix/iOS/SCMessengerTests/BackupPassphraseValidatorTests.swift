//
//  BackupPassphraseValidatorTests.swift
//  SCMessengerTests
//
//  Mirrors: android/.../test/BackupPassphraseValidatorTest.kt
//

import XCTest
@testable import SCMessenger

final class BackupPassphraseValidatorTests: XCTestCase {
    func testPassphraseShorterThan8CharactersIsTooShort() {
        XCTAssertEqual(validateBackupPassphrase("short", confirmation: "short"), .tooShort)
    }

    func testMismatchedConfirmationIsRejectedEvenIfBothAreLongEnough() {
        XCTAssertEqual(
            validateBackupPassphrase("correct-horse-battery", confirmation: "correct-horse-batteryy"),
            .mismatch
        )
    }

    func testMatchingPassphraseOfSufficientLengthIsValid() {
        XCTAssertEqual(
            validateBackupPassphrase("correct-horse-battery", confirmation: "correct-horse-battery"),
            .valid
        )
    }

    func testLengthCheckTakesPriorityOverMismatchCheck() {
        // Both are short AND don't match; tooShort should win so the user
        // isn't told to "fix the mismatch" into another too-short passphrase.
        XCTAssertEqual(validateBackupPassphrase("ab", confirmation: "cd"), .tooShort)
    }
}
