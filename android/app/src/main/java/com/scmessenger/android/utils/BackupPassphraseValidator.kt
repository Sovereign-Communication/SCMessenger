package com.scmessenger.android.utils

/** Validation result for a user-chosen identity backup passphrase. */
sealed class BackupPassphraseValidation {
    object Valid : BackupPassphraseValidation()
    object TooShort : BackupPassphraseValidation()
    object Mismatch : BackupPassphraseValidation()
}

private const val MIN_BACKUP_PASSPHRASE_LENGTH = 8

/**
 * Validate a passphrase entered twice (entry + confirmation) before it's used
 * to encrypt an identity backup export. A typo here means an unrecoverable
 * backup, so both a minimum length and an exact match are required.
 */
fun validateBackupPassphrase(passphrase: String, confirmation: String): BackupPassphraseValidation {
    return when {
        // codePointCount, not length: String.length counts UTF-16 code
        // units, so a passphrase of surrogate-pair characters (e.g. most
        // emoji) reports roughly double its actual character count and
        // could pass this check with far fewer real characters than intended.
        passphrase.codePointCount(0, passphrase.length) < MIN_BACKUP_PASSPHRASE_LENGTH ->
            BackupPassphraseValidation.TooShort
        passphrase != confirmation -> BackupPassphraseValidation.Mismatch
        else -> BackupPassphraseValidation.Valid
    }
}
