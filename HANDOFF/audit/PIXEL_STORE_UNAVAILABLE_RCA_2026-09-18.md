# Pixel "Message Store Unavailable" — RCA + field recovery on the merged binaries

Date: 2026-09-18 (UTC), device work 14:02Z-14:30Z. Device: Pixel 6a
(`adb-26261JEGR01896-6pHTac`), app 0.4.0 versionCode 15 installed 2026-09-16
01:17 (predates the #288 train, #295 coldstart work, #305, and the zombie fix).
Evidence pass per operator order; every number below is from a command run this
session. Raw captures: `tmp/pixel-store-regression/` (803k-line logcat,
verified app-data archive, clean store-file pull, prefs, new APK).

## 1. The symptom, precisely

Operator saw "Message Store Unavailable" in the app UI. That string is
`storage_error_title` (strings.xml:545), shown by DashboardScreen/SettingsScreen
when `MeshRepository._isStorageDegraded` is set. Device logcat, 15 occurrences
in the buffer, e.g.:

```
09-18 13:46:01.028 E MeshRepository: Persistent storage is degraded or locked;
  mesh service cannot start safely
09-18 13:46:01.028 E MeshRepository: uniffi.api.IronCoreException$StorageException:
  Storage error
```

Trigger code: `MeshRepository.startMeshService` catch block (MeshRepository.kt
2802-2813) classifies the exception, sets the flag, stops the service. Free
space was never the cause: `Storage maintenance check: free=4634MB / total=112912MB`.

## 2. Root cause (from the node's own structured log)

The top-level sled identity store at `files/` (files `conf`, `db`,
`snap.00000000000380C1`, `blobs/`) is crash-torn. `files/db` frozen at exactly
**524,287 bytes (512 KiB - 1)**, mtime = the last failed start. sled 0.34.7's
verdict, verbatim:

```
unexpected corruption encountered in storage snapshot file.
stable lsn 71371 should be >= snapshot.stable_lsn 229569
Failed to open sled storage at '.../files': corruption detected ...
```

The snapshot references log data past the tear; recovery from this state is
correctly refused by the storage engine on every version — **updating the app
does not fix a damaged store**. History: the device already self-quarantined a
corrupted `contacts.db` on 2026-09-16 08:52Z
(`backup_corrupted_1789548764942/`), so this is the second storage-corruption
event on this device, not a new regression in the merged code.

## 3. Update attempt (evidence that updating alone is insufficient)

CI Android artifact `android-debug-apk` from the #305-merge Mobile run
35394539632 (SHA `55ef300b` — tree delta to main `fb3ce1ae` is HANDOFF-docs-only,
+283 lines, zero code; Mobile does not run on main pushes by design and its
workflow has no dispatch trigger). `-r` install failed with
`INSTALL_FAILED_UPDATE_INCOMPATIBLE` (signature mismatch — CI debug keys are
per-run ephemeral), so a full uninstall/reinstall was required. Before any
destructive step:

- Full app-data archive pulled and verified:
  `tmp/pixel-store-regression/appdata-full-archive.tar` (161,274,368 bytes,
  44 entries; `files/db` member byte-identical to a clean `exec-out run-as cat`
  pull).
- Prefs pulled individually (6 XMLs incl. `identity_backup_prefs.xml`,
  12,480 bytes).
- Pre-update logcat preserved (803,102 lines; 15 degraded hits).

Reinstall (`uninstall` -> `install` -> restore archive via run-as tar ->
re-grant the 7 runtime permissions the uninstall reset) produced:
**the same StorageException** (14:19:29Z, `MeshService.start` at api.kt:15692,
new core) — proving the failure is data-state, not version.

## 4. Two latent defects surfaced during recovery (not fixed in this pass)

1. **Keystore-bound identity backup**: the prefs-side identity backup is wrapped
   by an Android Keystore key; the signature-forced uninstall destroyed the
   key, so `restoreIdentityFromBackup` fails with
   `KeyStoreException: Signature/MAC verification failed (code -30)` on both
   primary and legacy paths ("Cryptographic error"). The designed recovery
   cannot survive a reinstall. SecurityUtils correctly quarantined the corrupt
   encrypted prefs (`scmessenger_secure_prefs_corrupt_1789777404674.xml`) and
   attempted Keystore-reset recovery — the backup remains undecryptable.
2. **Second identical corruption event**: the 09-15/16 crash tore one store
   (contacts.db, self-quarantined then) and this one (identity store,
   torn 524,287-byte log). Two torn-store events on one device in three days
   wants an RCA of its own (crash timing, fsync policy on sled close).

## 5. The field recovery that worked (all steps reversible, nothing deleted)

Sled log-only recovery: the identity was created at the store's birth
(2026-09-15 21:05, `conf` mtime) and the torn log's valid prefix extends to
lsn 71371 — far past identity creation — so the identity lives in the log
prefix; only the impossible snapshot blocks the open.

1. Quarantined the corrupt identity store:
   `files/quarantine_corrupted_identity_20260918T1421Z/{conf,db,blobs,snap.00000000000380C1}`.
   App then started on a fresh empty store; the backup-restore defect (4.1)
   left it identity-less ("onboarding required") — not acceptable.
2. Set the fresh empty store aside
   (`files/fresh_store_discarded_1423Z/`), restored `conf` + `db` from
   quarantine **without** the snapshot file -> forces log-only recovery.
3. Launch: store opened clean. Verbatim:

```
Corruption check: contacts=2, messages=108
SC_IDENTITY_OWN p2p_id=12D3KooWD776DQdWh6iHV8Qcnpj9jvTXhpJAgSsFPbMCaRtQpFmn
  pk=30dce2bb779b4f1419f6d7d9e91b3ae201aed9e3b181aef674a9496f340a0645
  id=f83ab16319ca5b801f1c088935f2215c6aae9aa246b992f01c0f27f06b03cbe5
Mesh service started successfully
```

The original identity (Lucas, the third live mesh peer) recovered intact —
no Keystore needed, as predicted (the sled store is not Keystore-bound).

## 6. Post-recovery functional proof (new APK, recovered identity)

- Both relay circuits registered within seconds of start:
  `/ip4/192.168.0.121:443/p2p/12D3KooWD6vZ.../p2p-circuit/p2p/12D3KooWD776...`
  (Windows) and `/ip4/18.234.62.247:9001/p2p/12D3KooWGvCW.../p2p-circuit/p2p/12D3KooWD776...`
  (AWS); external address observed `147.81.41.188:9090`.
- Real message Windows -> Pixel `0653bca9-4cb3-4a64-83fb-90b97c3e178c`:
  Pixel `delivery_attempt ... phase=rx outcome=received ... sender=985a25f9...`,
  canonicalized to the known contact, `outcome=processed detail=stored_in_history`;
  signed delivery receipt sent and `outcome=acked ... transport_ack=true`;
  Windows shows `status:"delivered","delivered":true` for the message.

## 7. End state and rollback

- Pixel: app 0.4.0 (55ef300b artifact, same core tree as deployed main
  fb3ce1ae), original identity + 2 contacts + 108 messages intact, mesh up on
  both circuit paths. First boot after repair re-froze nothing (db now grows
  normally under the new core).
- Quarantined artifacts remain on device:
  `files/quarantine_corrupted_identity_20260918T1421Z/` (corrupt store),
  `files/fresh_store_discarded_1423Z/` (empty store), and the pre-existing
  `backup_corrupted_1789548764942/`.
- Full rollback available on the operator host:
  `tmp/pixel-store-regression/` (verified 161 MB archive, prefs, frozen db,
  pre-update logcat).
- Windows/AWS nodes: untouched this pass, still 0fd69fb2, identities unchanged.

## 8. What remains (named, not done here)

1. **Keystore-independent backup restore** (defect 4.1) — the designed recovery
   should not depend on a Keystore key that a reinstall destroys; the restore
   path needs either a passphrase-derived wrap or a documented re-pairing flow.
2. **Second-corruption RCA** (defect 4.2) — why did this device tear two sled
   stores in three days (crash timing? fsync-on-close absence? vendor I/O)?
3. The quarantined corrupt stores stay on device until the operator rules on
   their disposal; they are the only physical evidence of the tear shape.
