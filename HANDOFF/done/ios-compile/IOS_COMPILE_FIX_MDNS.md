# TASK: Fix 1 Swift compile error in mDNSServiceDiscovery.swift

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: READY FOR DELEGATION (Qwen CODER, diff mode)
Scope: ONE file, ONE 1-line-class fix. Nothing else.

## Error — iOS/SCMessenger/SCMessenger/Transport/mDNSServiceDiscovery.swift:171

CI error: `cannot convert value of type 'UnsafeBufferPointer<sockaddr>' to specified type 'UnsafePointer<sockaddr>'`

Current code (lines 170-172):
```swift
        address.withUnsafeBytes { ptr in
            let sockaddrPtr: UnsafePointer<sockaddr> = ptr.bindMemory(to: sockaddr.self)
            guard let firstSockaddr = sockaddrPtr.first else { return }
```

`withUnsafeBytes` yields `UnsafeRawBufferPointer`; `bindMemory(to:)` returns
`UnsafeBufferPointer<sockaddr>`, which cannot convert to the declared
`UnsafePointer<sockaddr>`.

REQUIRED FIX — delete the wrong type annotation, keep everything else identical:
```swift
        address.withUnsafeBytes { ptr in
            let sockaddrPtr = ptr.bindMemory(to: sockaddr.self)
            guard let firstSockaddr = sockaddrPtr.first else { return }
```

## Constraints

- Diff mode only.
- No other edits in the file (no lint fixes, no trailing-newline change, no
  renames). Byte-identical except the annotation removal.
