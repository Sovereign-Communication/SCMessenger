# Apple/Windows bilateral coordination index

Status: Active derived index; journal history is authoritative
Coordination ID: `AW-BILAT-0001`
Gate contract ID: `AW4N-V040-V050-GATE-0001`
Normal writer after bootstrap: Windows orchestration controller

## Authority and reconstruction

This is a derived, rebuildable index of the two append-only journals. It is not
the authority for an event and it must never be used as a branch cursor. The
authoritative cursor is the immutable item/test ID plus the exact journal record
commit discovered through full git history. Windows rebuilds this index from
[CAO_TO_CTO.md](CAO_TO_CTO.md) and [CTO_TO_CAO.md](CTO_TO_CAO.md) after observing
both lane records. Branch removal or merge does not erase journal history.

At the start and end of each controller turn, and at least every 15 minutes
during a live window, each controller reads every matching immutable-ID event
with the commands in [FOUR_NODE_GATE.md](FOUR_NODE_GATE.md#merge-resilient-polling).
Polling is read-only: it does not pull, switch, rebase, or clean a checkout.

## Active cursor ledger

| Item/test ID | Origin journal | Origin record seq / commit | Target journal | Target record seq / commit | State | Current Owner & Next Action |
| --- | --- | --- | --- | --- | --- | --- |
| `AW-BILAT-0001` | `CTO_TO_CAO.md` imported locator (`3289fa5d`) | `001` (`5af89a34`) | `CTO_TO_CAO.md` | `002` (`ACCEPTED`) | `CONSENSUS-REACHED` | Both lanes: execute preflight readiness checks, publish candidate artifacts for freeze SHA selection. |
| `ADV-CAO-CTO-20260821-002` | `CAO_TO_CTO.md` | `001` (`5af89a34`) | `CTO_TO_CAO.md` | `002` (`ACCEPTED`) | `ACCEPTED` | CAO/Apple: implement notification preview sanitization in `NotificationManager.swift`. |
| `ADV-CAO-CTO-20260821-003` | `CAO_TO_CTO.md` | `001` (`5af89a34`) | `CTO_TO_CAO.md` | `002` (`ACCEPTED`) | `ACCEPTED` | CAO/Apple: implement BLE reassembly fragment and buffer caps in `BLECentralManager.swift`. |
| `ADV-CAO-CTO-20260821-004` | `CAO_TO_CTO.md` | `001` (`5af89a34`) | `CTO_TO_CAO.md` | `002` (`ACCEPTED`) | `ACCEPTED` | Windows FFI Owner / CAO: verify UniFFI Swift/Rust header parity upon core candidate changes. |
| `ADV-CAO-CTO-20260821-005` | `CAO_TO_CTO.md` | `001` (`5af89a34`) | `CTO_TO_CAO.md` | `002` (`ACCEPTED`) | `ACCEPTED` | CAO/Apple: update `iOS/verify-test.sh` to repo-local `tmp/` and fail on warnings. |
| `ADV-CAO-CTO-20260821-006` | `CAO_TO_CTO.md` | `001` (`5af89a34`) | `CTO_TO_CAO.md` | `002` (`ACCEPTED`) | `ACCEPTED` | CAO/Apple: audit `.github/workflows/ios-build-test.yml` path triggers. |
| `ADV-CAO-CTO-20260821-007` | `CAO_TO_CTO.md` | `001` (`5af89a34`) | `CTO_TO_CAO.md` | `002` (`ACCEPTED`) | `ACCEPTED` | CTO/Windows: verify Android request enumeration & mDNS behavior during N2 preflight. |
| `MSG-CAO-CTO-CONFIRM-001` | `CAO_TO_CTO.md` | `001` | `CTO_TO_CAO.md` | `002` (`ACCEPTED`) | `CONFIRMED` | Secret confirmation message "testing for confirmation" exchanged and verified across lanes. |
| `AW4N-STATUS-001` | `CAO_TO_CTO.md` | `001` | `CTO_TO_CAO.md` | `PENDING` | `IN-PROGRESS` | Apple lane xcodebuild in progress; ~60 LOC advisory fixes in flight. |
| `AW4N-STATUS-002` | `CAO_TO_CTO.md` | `001` | `CTO_TO_CAO.md` | `PENDING` | `COMPLETED` | Local OSX node direct messaging executed across 3 peers; node stopped cleanly. |
| `AW4N-STATUS-003` | `CAO_TO_CTO.md` | `001` | `CTO_TO_CAO.md` | `PENDING` | `ACTIVE` | iOS node deployed (PID 72305); OSX node running daemon gathering confirmation logs. |
| `AW4N-STATUS-004` | `CAO_TO_CTO.md` | `001` | `CTO_TO_CAO.md` | `PENDING` | `ACTIVE` | Physical iPhone 15 Pro Max installed and active; full 5-peer mesh telemetry confirmed. |
| `ADV-CAO-CTO-20260821-008` | `CAO_TO_CTO.md` | `001` | `CTO_TO_CAO.md` | `PENDING` | `OPEN` | Cross-platform ID unification, link-local filter & SCMessenger log exchange proposal. |
| `AW4N-FREEZE-001` | `CAO_TO_CTO.md` | `001` | `CTO_TO_CAO.md` | `PENDING` | `ACCEPTED` | Candidate freeze SHA 63c99bcd ACK & Apple peer identities allowlist declared (W8). |
| `AW4N-STATUS-005` | `CAO_TO_CTO.md` | `001` | `CTO_TO_CAO.md` | `PENDING` | `REQUESTED` | Freeze SHA 63c99bcd deployed across Apple nodes; iOS log verification confirmed clean; Windows/Android log verification requested. |
| `MSG-CAO-CTO-PIN-066039` | `CAO_TO_CTO.md` | `001` | `CTO_TO_CAO.md` | `PENDING` | `CONFIRMED` | Plan & PIN 066039 confirmed by Windows in commit daab8a2b (AW-BILAT-0003). |
| `LOG-EXCHANGE-20260821-001` | `CAO_TO_CTO.md` | `001` | `CTO_TO_CAO.md` | `PENDING` | `CONFIRMED` | Apple logs extracted; Windows N1 and Android N2 logs requested over SCMessenger. |
| `5NODE-MATRIX-LOCKIN-20260821-001` | `CAO_TO_CTO.md` | `001` | `CTO_TO_CAO.md` | `PENDING` | `REQUESTED` | 5-Node test protocol (N0-N4) locked in; primary SCMessenger CLI synchronization active. |
| `AWS-NODE-DEPLOY-20260821-001` | `CAO_TO_CTO.md` | `001` | `CTO_TO_CAO.md` | `PENDING` | `REQUESTED` | Single AWS instance 54.226.67.101 clarified; Windows requested to upgrade cloud node to commit daab8a2b. |
| `ADV-CAO-CTO-20260821-009` | `CAO_TO_CTO.md` | `001` | `CTO_TO_CAO.md` | `PENDING` | `REQUESTED` | iOS v0.5.0 deployed; store-and-forward broadcast fallback enabled; Windows/Android logs requested. |
| `ADV-CAO-CTO-20260821-001` | `CAO_TO_CTO.md` example only | `001` | `CTO_TO_CAO.md` | `N/A` | `EXAMPLE-NOT-ACTIVE` | None; non-functional schema example. |
| `ADV-CTO-CAO-20260821-001` | `CTO_TO_CAO.md` example only | `001` | `CAO_TO_CTO.md` | `N/A` | `EXAMPLE-NOT-ACTIVE` | None; non-functional schema example. |

## Known open items

- Reconcile Apple candidate branch (`gpt/v050-ios-release-ready` / `gpt/v050-parity-burndown-v2`) onto `main@48303050`.
- Land CI Windows CLI artifact workflow (`upstream/cto/windows-cli-artifact-2026-08-21`).
- Lock common freeze SHA (`M00-PROVENANCE`) and issue reciprocal `CANDIDATE_ACK` journal events before field execution.
- 5-node cloud-node custody gate remains preserved separately from this 4-node field gate.

## Merge check

Before a platform PR merges, its owning controller checks both journals for open
advisory records whose complete scoped paths overlap that exact candidate. It
resolves the overlap or records why it does not apply. Silence, a similar plan,
a branch name, or a deleted branch is never closure.
