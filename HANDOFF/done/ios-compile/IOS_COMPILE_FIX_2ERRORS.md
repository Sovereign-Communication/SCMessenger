# TASK: Fix 2 Swift compile errors breaking the iOS CI build

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: READY FOR DELEGATION (Qwen CODER, diff mode)
Scope: EXACTLY two files, surgical fixes only. Do not reformat, do not fix
lint violations, do not touch anything else.

## Error 1 — iOS/SCMessenger/SCMessenger/Transport/MultipeerTransport.swift:127

CI error: `cannot convert value of type 'String' to expected argument type 'OSLogMessage'`

Current code (lines 126-129):
```swift
        logger.info(
            "Reconnect: scheduling attempt \(attempt + 1)/\(self.maxReconnectAttempts) " +
            "for \(name) in \(Int(cappedDelay))s"
        )
```

os_log `Logger.info(_:)` takes an `OSLogMessage` (a string-interpolation
literal), NOT a runtime-concatenated `String`. The `+` concatenation makes it
a `String`, which fails to compile.

REQUIRED FIX — single interpolation literal, no `+` concatenation:
```swift
        logger.info("Reconnect: scheduling attempt \(attempt + 1)/\(self.maxReconnectAttempts) for \(name) in \(Int(cappedDelay))s")
```

## Error 2 — iOS/SCMessenger/SCMessenger/Transport/mDNSServiceDiscovery.swift:171

CI error: `cannot convert value of type 'UnsafeBufferPointer<sockaddr>' to specified type 'UnsafePointer<sockaddr>'`

Current code (lines 170-172):
```swift
        address.withUnsafeBytes { ptr in
            let sockaddrPtr: UnsafePointer<sockaddr> = ptr.bindMemory(to: sockaddr.self)
            guard let firstSockaddr = sockaddrPtr.first else { return }
```

`withUnsafeBytes` yields an `UnsafeRawBufferPointer`; `bindMemory(to:)`
returns `UnsafeBufferPointer<sockaddr>` (a BUFFER pointer), which cannot
convert to `UnsafePointer<sockaddr>`. The type annotation is wrong.

REQUIRED FIX — remove the incorrect annotation (let inference produce
`UnsafeBufferPointer<sockaddr>`):
```swift
        address.withUnsafeBytes { ptr in
            let sockaddrPtr = ptr.bindMemory(to: sockaddr.self)
            guard let firstSockaddr = sockaddrPtr.first else { return }
```

Everything else in both closures stays byte-identical.

## Constraints

- Diff mode only. No full-file rewrites.
- No other edits in either file (no lint fixes, no trailing-newline changes,
  no renaming). These two files are also covered by a separate lint campaign;
  keep this change minimal so it merges cleanly with that work.
- Verify there is no trailing-newline removal at EOF in your diff.
