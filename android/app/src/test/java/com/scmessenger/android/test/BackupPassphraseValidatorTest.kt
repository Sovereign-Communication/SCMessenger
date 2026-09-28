package com.scmessenger.android.utils

import org.junit.Assert.assertEquals
import org.junit.Test

class BackupPassphraseValidatorTest {

    @Test
    fun `passphrase shorter than 8 characters is too short`() {
        assertEquals(
            BackupPassphraseValidation.TooShort,
            validateBackupPassphrase("short", "short")
        )
    }

    @Test
    fun `mismatched confirmation is rejected even if both are long enough`() {
        assertEquals(
            BackupPassphraseValidation.Mismatch,
            validateBackupPassphrase("correct-horse-battery", "correct-horse-batteryy")
        )
    }

    @Test
    fun `matching passphrase of sufficient length is valid`() {
        assertEquals(
            BackupPassphraseValidation.Valid,
            validateBackupPassphrase("correct-horse-battery", "correct-horse-battery")
        )
    }

    @Test
    fun `length check takes priority over mismatch check`() {
        // Both are short AND don't match; TooShort should win so the user
        // isn't told to "fix the mismatch" into another too-short passphrase.
        assertEquals(
            BackupPassphraseValidation.TooShort,
            validateBackupPassphrase("ab", "cd")
        )
    }

    @Test
    fun `four-emoji passphrase is too short despite eight UTF-16 code units`() {
        // Each of these emoji is a supplementary-plane character (a surrogate
        // pair = 2 UTF-16 code units), so four of them make String.length
        // report 8 even though there are only 4 actual characters - the min
        // length must be enforced in code points, not UTF-16 units.
        val fourEmoji = "😀😀😀😀"
        assertEquals(8, fourEmoji.length)
        assertEquals(
            BackupPassphraseValidation.TooShort,
            validateBackupPassphrase(fourEmoji, fourEmoji)
        )
    }
}
