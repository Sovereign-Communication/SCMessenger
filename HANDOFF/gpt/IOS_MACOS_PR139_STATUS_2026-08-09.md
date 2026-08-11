# PR #139 iOS and macOS lane status

DATE: 2026-08-09
PR_HEAD: `d48558a89eec24a267fff4b7d9fe352a983ec09a`

## macOS

The exact PR head was rebuilt and launched with the preserved macOS driver
data directory. `/version` reported `0.4.0`, git hash `d48558a8`, branch
`codex/pr139-hardening`, and the expected build timestamp. The preserved
macOS PeerId is `12D3KooWP1hvZbqCCPMMfrZbW16EHy7wXp41pDPWtHzdn3MbwG5e`.

A direct SCMessenger CLI coordination message was sent to the Windows PeerId
`12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw`. The CLI accepted the
message into the local outbox, but it remains undelivered because the Windows
peer was not reachable; the macOS peer table stayed empty and repeated direct
dials failed. This is coordination evidence, not a delivered-message claim.

## iOS

The connected physical iPhone was resolved unambiguously and the exact PR
head built successfully for device deployment. Automatic signing succeeded
with team `JSZ36WH4C` and the development profile
`1e6871f6-f76b-431f-8c3b-7a25c821239e`. The app was installed in place with
`UNINSTALL_FIRST=0`, so no destructive uninstall or container wipe occurred.

The installed bundle is `SovereignCommunications.SCMessenger`, version `0.4.0`,
build `9`. Launch was attempted, but CoreDevice denied it because the phone
was locked. iOS runtime identity/history, messaging, receipts, and restart
evidence therefore remain pending a user-unlocked launch.

## CI fix

The failing `iOS Build & Simulator Test` job was traced to synchronous calls
from `mDNSServiceDiscovery` into the `@MainActor` `MeshRepository`. The
discovery object is now explicitly `@MainActor`; a local simulator build
completed successfully after this change. The generated Swift binding update
from the exact PR head is included alongside the source fix.

The unrelated `HANDOFF_AUDIT` and turbo-fieldfare audit changes in the shared
checkout were left untouched and unstaged.
