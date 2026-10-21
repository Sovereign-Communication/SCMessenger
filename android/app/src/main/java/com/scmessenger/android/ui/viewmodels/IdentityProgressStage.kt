package com.scmessenger.android.ui.viewmodels

/**
 * High-level proof-of-work stages emitted by both [MainViewModel.createIdentity]
 * and [IdentityViewModel.createIdentity].
 *
 * The same 6 stages are emitted on both the onboarding path and the in-settings
 * path so the user sees a single, deterministic progress narrative regardless of
 * which entry point they used.
 *
 * Each stage carries:
 *   - id:        stable ordinal (1..6) used to drive "step X of 6" + checkmarks
 *   - label:     short, user-facing label (1-4 words, no jargon)
 *   - detail:    one short sentence telling the user what's happening
 *                (technical but not too deep — "Setting up secure storage"
 *                beats "GRANT_CONSENT via sled-backed persistence")
 *   - etaMs:     typical wall-clock duration in milliseconds for this stage
 *                on a mid-range Android device (Pixel 6a baseline). The UI
 *                uses this to render a stage progress bar and to drive a
 *                "About 6 seconds remaining" hint. The actual duration is
 *                not a contract — it is a best-effort estimate so the user
 *                sees motion, not a precision claim.
 */
sealed class IdentityProgressStage(
    val id: Int,
    val label: String,
    val detail: String,
    val etaMs: Long,
) {
    /**
     * Sentinel: no identity creation in progress. The single source of truth for
     * "we are not creating right now". Replaces the previous nullable-typed
     * StateFlow to enforce at the type system that [IdentityViewModel.progressStage]
     * and [MainViewModel.identityProgressStage] are never null at runtime, which
     * was the root cause of the v0.3.2 NPE crash in
     * IdentityScreen.ProofOfWorkList(currentStage.id).
     */
    data object Idle : IdentityProgressStage(
        id = 0,
        label = "",
        detail = "",
        etaMs = 0L,
    )

    /** Worker is preparing encrypted storage (grant consent, ensure service running). */
    data object PreparingStorage : IdentityProgressStage(
        id = 1,
        label = "Setting up secure storage",
        detail = "Waking the encrypted key vault and asking for your consent…",
        // This ETA drives the progress bar fraction, not the displayed countdown.
        // On cold starts the actual wall time is 15-60s (sled DB, managers, migrations);
        // on warm starts it's <1ms. We use 5s for smooth progress bar advancement.
        // The displayed ETA text shows "Initializing…" during cold starts instead
        // of a misleading countdown, driven by the sub-detail presence in
        // IdentityProgressDisplay.
        etaMs = 5_000L,
    )

    /** Worker is generating a 256-bit cryptographically-secure salt. */
    data object GeneratingSalt : IdentityProgressStage(
        id = 2,
        label = "Generating a random salt",
        detail = "Drawing 256 bits of fresh randomness to protect your key…",
        etaMs = 50L,
    )

    /** Worker is calling into Rust to generate the Ed25519 keypair. */
    data object GeneratingKeypair : IdentityProgressStage(
        id = 3,
        label = "Creating your identity key",
        detail = "Generating your secret key with strong cryptography (this is the longest step)…",
        etaMs = 3000L,
    )

    /** Worker is computing the BLAKE3 identity fingerprint from the public key. */
    data object ComputingFingerprint : IdentityProgressStage(
        id = 4,
        label = "Deriving your identity fingerprint",
        detail = "Hashing your key into a short, shareable identity code…",
        etaMs = 50L,
    )

    /** Worker is persisting keys to encrypted sled storage. */
    data object PersistingToStorage : IdentityProgressStage(
        id = 5,
        label = "Saving to encrypted storage",
        detail = "Writing the encrypted identity to local storage (XChaCha20-Poly1305)…",
        etaMs = 400L,
    )

    /** Worker is verifying the identity is live and re-readable. */
    data object VerifyingIdentity : IdentityProgressStage(
        id = 6,
        label = "Verifying your identity is ready",
        detail = "Reading the identity back to confirm it persisted correctly…",
        etaMs = 150L,
    )

    companion object {
        /** Total number of stages; used to compute "X of N" progress. */
        const val TOTAL: Int = 6

        /** Total estimated wall-clock time across all stages (sum of etaMs).
         *  On cold starts, PreparingStorage can take 15-60s (the displayed ETA
         *  shows "Initializing…" instead of a countdown). On warm starts the
         *  actual total is ~4s because PreparingStorage completes instantly. */
        const val TOTAL_ETA_MS: Long = 8_650L

        val ALL: List<IdentityProgressStage> = listOf(
            PreparingStorage,
            GeneratingSalt,
            GeneratingKeypair,
            ComputingFingerprint,
            PersistingToStorage,
            VerifyingIdentity,
        )
    }
}
