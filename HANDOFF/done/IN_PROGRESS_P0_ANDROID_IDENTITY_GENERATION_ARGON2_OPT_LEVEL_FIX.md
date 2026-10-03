# P0  Android: Identity Generation Took 2+ Minutes (Argon2 KDF in Unoptimized Debug Build)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Status:** FIX APPLIED AND COMMITTED  one open regression question unanswered by user
**Severity:** P0 (blocked onboarding / QR export on Android)
**Source:** Antigravity CLI (Gemini) session `ef178839-72b0-4d39-b7b5-c25ccfb56acd`, conversation DB at
`C:\Users\SCM\.gemini\antigravity-cli\conversations\ef178839-72b0-4d39-b7b5-c25ccfb56acd.db`
(largest/most-recently-written Antigravity conversation for this repo as of 2026-07-02 15:34 HST;
recovered by reading the sqlite trajectory directly since Antigravity has no text transcript export).

## User report (verbatim, repeated several times across the session)

> "Initial reactions - identity takes over 2 minutes to generate - is this absolutely necessary?
> Also, the QR Code doesn't show up when I click on Show Identity QR. Pull the logs from the
> device and debug completely."

Device: Pixel 6a, connected via USB (adb).

## Root cause

`get_identity_export` (Rust core)  `encrypt_backup`  `derive_key_argon2id`. Argon2id key
derivation is CPU-bound by design (that's the point of a KDF), but the `scmessenger-core` crypto
dependencies were being compiled under Cargo's `dev` profile with **no optimization**
(`opt-level = 0` default). On a Pixel 6a's mobile ARM64 CPU, an unoptimized Argon2id pass that
should take ~1-2 seconds instead took 2+ minutes. This blocked both identity creation and
"Show Identity QR" (which re-derives the export key).

## Fix applied

`Cargo.toml` (workspace root), `[profile.dev]`:

```toml
[profile.dev]
opt-level = 3
```

Confirmed present at [Cargo.toml:127-129](../../Cargo.toml#L127-L129), committed in
`561c7478 "Commit before restart"` (2026-07-02 15:38:22 -1000)  same commit as the BLE
`start_scan` retry fix from the Windows setup session. Working tree is clean; nothing here is
uncommitted.

Antigravity's own verification message after applying the fix:

> "Instant Identity Generation: Generating an identity should now take milliseconds rather than
> over 2 minutes. The massive delay was because the Rust cryptography library (specifically
> Argon2 key derivation) was being compiled in strict debug mode. I added a rule to Cargo.toml to
> always compile cryptographic dependencies at opt-level = 3 so they run at full speed even
> during local development."

## Open question  unresolved regression signal (session ended here)

Later in the same session, the user did a **fresh reinstall** on the Pixel 6a (uninstall, then
`adb install`) specifically to test "recognizes this as update and picks up existing identity."
That feature failed, and the user reported: *"still taking a while to generate identity. at a
minute and still going.... debug!"*

Antigravity pulled fresh logcat and found the core **did** detect the cached identity correctly on
startup (`Cached identity fields: peerId=12D3KooW, id=f3c7c854`), but no
`getIdentityExportString` / QR-related exception appeared in the log grep. Its conclusion: this
looks like a **UI state propagation bug**, not a recurrence of the Argon2 slowness  but it
couldn't confirm without more detail, and the session ended on this unanswered clarifying question
to the user:

1. Did the app drop back onto the **Welcome / Create Identity** onboarding screen instead of the
   Contacts list?
2. Did it bypass onboarding but get stuck on **Settings  Identity** showing "Restoring your
   identity..." or "Identity not initialized"?
3. Or did it look normal, but clicking **Show Identity QR** took 2 minutes again (i.e. the Argon2
   fix didn't actually apply to this build)?

**Next step:** ask the user which of the three happened. If (3), the APK on-device may not have
been rebuilt/reinstalled with the `opt-level = 3` fix (check build timestamp vs. commit
`561c7478`), or the fix doesn't cover a code path used by the QR export flow specifically. If (1)
or (2), this is a separate Kotlin/Rust bridge state bug unrelated to Argon2 timing  likely in
`MeshRepository.getIdentityExportString()` or the onboarding/settings identity-restore path.

**UPDATE 2026-07-05  likely resolved, this was option (3) all along, but not the cause it named.**
A fresh live report this session ("QR code loading REALLY slowly... took like 45 seconds")
triggered a full read of the actual QR call chain. Confirmed: `getIdentityExportString()` (the QR
path) **never calls `exportIdentityBackup`/`derive_key_argon2id` at all**  it is a separate,
unencrypted "public identity card" export (`MeshRepository.kt:3193-3194`, `:8774-8813`). So this
ticket's `opt-level=3` fix was never going to affect QR load time either way  the QR slowness is
a distinct bug (unconditional/redundant FFI calls and an uncached identity re-fetch on the QR hot
path, likely also coupled to MeshService cold-start taking up to 60s by design). Filed as its own
ticket: `HANDOFF/todo/P1_ANDROID_QR_Code_Identity_Export_Extremely_Slow.md`  that ticket owns the
fix. This ticket's own scope (Argon2id opt-level for the encrypted-backup path) can be considered
closed/verified once a fresh `cargo build --workspace` + timed backup-export repro confirms the
opt-level=3 fix still compiles and applies (not re-verified this session, but the Cargo.toml
setting was confirmed still present at lines 127-131).

## Note on a separate, unrelated, older ticket

`HANDOFF/IN_PROGRESS/IN_PROGRESS_P0_ANDROID_024_IDENTITY_GENERATION_REGRESSION.md` (still marked
OPEN, dated 2026-06-05) describes a **different** identity bug: re-entrant
`initializeIdentity()` calls during onboarding (8-10 calls/second), unrelated to Argon2 KDF
timing. That ticket's root cause was reentrancy/race, not compile-time optimization. Do not
conflate the two  `[VALIDATED]_P0_ANDROID_024_Identity_Generation_Reentrant_Guard.md` in
`HANDOFF/done/` appears to be its resolution, but the ticket file itself was never moved out of
`IN_PROGRESS/`, so it should be reconciled separately.

## Build verification status

Not independently re-verified in this recovery pass  Antigravity's session already confirmed the
`opt-level = 3` change compiles and the fix was observed working once (before the reinstall
regression signal). Per repo rules, before closing this ticket: `cargo build --workspace` and a
fresh `adb install` + timed identity-creation repro on the Pixel 6a are required.
