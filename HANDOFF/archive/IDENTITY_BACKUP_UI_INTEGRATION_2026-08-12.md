# IDENTITY_BACKUP_UI_INTEGRATION_2026-08-12.md

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: todo
Priority: P1 (next round after v0.4.0 tag)
Lane: Android (Windows orchestrator + delegated)

## Why this ticket exists

On 2026-08-12 a CI-APK flash + RAW FILE restore (`run-as` + tar of
`files/{conf,db,ledger.json,...}`) left the Android app in a hybrid state:
the app generated a new identity, then encountered the restored ledger/conf
state it did not own, and ended up advertising a stale peer id with key
material mismatched against what fleet peers had on file. Windows node logs
show ratchet decryption failures, custody max-attempts, and the peer
dead-marked; the user saw 4 nodes collapse to 1.

Root cause of the HYBRID STATE: the app's own identity backup/restore
subsystem was bypassed. Raw `files/` surgery cannot carry the identity
blob (`shared_prefs/identity_backup_prefs.xml` -> `identity_backup_v1`),
and the backup passphrase lives in `platform_secure_keys.xml`
(`backup_passphrase_v1`), neither of which survives a naive files-only copy
in a consistent form.

## The existing subsystem (recon, 2026-08-12, verified in-tree)

ALL layers are wired; do not rebuild them:

- Core (Rust): `core/src/iron_core.rs` --
  `export_identity_backup` (1657), `export_identity_backup_with_salt`
  (1700), `export_identity_backup_fast` (1738),
  `export_identity_backup_fast_with_salt` (1754), `import_identity_backup`
  (1789). Payload = Ed25519 seed + ratchet sessions + contacts,
  XChaCha20-Poly1305, Argon2id (user passphrase) or Blake3-derived key
  (device auto-backup).
- Repository: `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
  -- `exportIdentityBackup` (3532), `restoreIdentityFromBackup` private
  auto-restore (3457) + public manual import (3500),
  `persistIdentityBackup` (3537, auto-persist after identity creation,
  latched once per process), `getPlatformSecuredPassphrase` (3445),
  `checkReinstallState` (857), `isIdentityInitialized` fallback chain
  (5044-5085: cached fields -> backup blob -> sentinel file -> core).
- ViewModel: `SettingsViewModel.kt` -- `exportIdentityBackup` (390),
  `importIdentityBackup` (368).
- UI: `SettingsScreen.kt` -- export dialog with passphrase + confirm
  (328-386), import dialog (166-307), settings actions (1081-1098).
- Onboarding: `OnboardingScreen.kt`.

## What the next round must change (findings-driven)

1. OPERATOR PROCEDURE (no code): never restore identity by raw `files/`
   surgery. The sanctioned path is: Settings -> Export identity backup
   (user passphrase) BEFORE uninstall/flash; after fresh install,
   Settings -> Import identity backup (paste blob + same passphrase).
   Document in HANDOFF/AGENT_NOTES or the runbook this ticket lands.
2. [GAP-2] Export result is clipboard-only. Add SAF save
   (`ACTION_CREATE_DOCUMENT`) alongside the clipboard path.
3. [GAP-3] Import is paste-only. Add SAF open (`ACTION_OPEN_DOCUMENT`)
   alongside the paste path.
4. [GAP-1] `backup_passphrase_v1` sits plaintext in SharedPreferences.
   Move to `EncryptedSharedPreferences` / Keystore AES-GCM wrap.
   (Security-adjacent: flag for the adversarial review lane, not
   self-approved.)
5. [GAP-4] `IdentityViewModel.kt` exists but backup lives in
   `SettingsViewModel`. Decide consolidation direction; prefer the
   ViewModel that already owns the dialogs unless identity screens gain
   their own nav entry.

## Acceptance criteria

- Flash/reinstall runbook lands in-tree and references ONLY the
  export/import UI path.
- SAF save/open added; existing clipboard/paste paths unchanged.
- Round-trip test (or manual evidence): export on device A, uninstall,
  fresh install, import -> same peer id + contacts, no second identity
  generation in the mesh log.
- Keystore change gated by adversarial review (Rule 8 class).

## Do NOT

- Touch `core/src/{crypto,transport,routing,privacy}/` without review.
- Rebuild the backup format; it is stable and already encrypted.
- Restore from `tmp/android_backup_pre_flash/scm_backup.tar.gz` -- that
  identity's Keystore keys are gone; the blob is debug evidence only.
