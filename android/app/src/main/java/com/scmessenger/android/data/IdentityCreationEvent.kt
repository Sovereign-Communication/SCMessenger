package com.scmessenger.android.data

/**
 * Type-agnostic progress events emitted by [MeshRepository.createIdentity] as it
 * works through the six sub-stages of identity creation. The data layer has no
 * UI dependencies, so we keep this enum here rather than reaching into
 * `ui.viewmodels.IdentityProgressStage`. The ViewModel maps each
 * [IdentityCreationEvent] to the corresponding [IdentityProgressStage] so the
 * existing [IdentityProgressDisplay] composable continues to render the same
 * six-stage narrative without any change to its parameter shape.
 *
 * The shape of this enum mirrors [IdentityProgressStage] intentionally — the
 * repo's behavior and the UI's presentation are deliberately aligned, so
 * duplicating the six names is preferable to introducing a new abstraction.
 *
 * The second callback slot is a transient sub-stage detail string the UI can
 * render under the active stage's `detail` line; the repo uses it to show
 * motion during otherwise-blocking work (e.g. SharedPreferences `commit()`,
 * libp2p TCP bind). Pass `null` when no transient detail is available.
 */
typealias IdentityProgressCallback = (IdentityCreationEvent, String?) -> Unit

/**
 * Six-stage proof-of-work phases that [MeshRepository.createIdentity] advances
 * through. The repo fires one of these (with an optional sub-detail) at each
 * sub-stage boundary.
 */
sealed class IdentityCreationEvent {
    /** Repo is waking MeshService and asking for the encrypted vault handle. */
    data object PreparingStorage : IdentityCreationEvent()

    /** Repo is recording the user's consent in the Rust core. */
    data object GeneratingSalt : IdentityCreationEvent()

    /** Repo is generating the Ed25519 keypair in the Rust core (longest step). */
    data object GeneratingKeypair : IdentityCreationEvent()

    /** Repo is computing the BLAKE3 identity fingerprint from the public key. */
    data object ComputingFingerprint : IdentityCreationEvent()

    /** Repo is encrypting the identity backup and persisting to SharedPreferences. */
    data object PersistingToStorage : IdentityCreationEvent()

    /** Repo is starting the libp2p swarm listener and updating the BLE beacon. */
    data object VerifyingIdentity : IdentityCreationEvent()
}
