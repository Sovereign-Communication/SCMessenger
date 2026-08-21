# Apple/Windows bilateral coordination index

Status: Bootstrap derived index; journal history is authoritative
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

## Bootstrap cursor ledger

| Item/test ID | Origin journal | Origin record commit | Target journal | Target record commit | State | Next owner |
| --- | --- | --- | --- | --- | --- | --- |
| `AW-BILAT-0001` | `CTO_TO_CAO.md` imported locator | `3289fa5d15eb6b4e631e5830e477030886799e54` | `CAO_TO_CTO.md` bootstrap response | `5af89a3498b0438e3b9efdc401ec8d6129915177` | `RECIPROCAL-ACK-APPENDED` | CAO: confirm or counter-nominate the Windows preflight owner on return. CTO event AW-BILAT-0001/002 appended in CTO_TO_CAO.md (its record commit is PENDING-POST-COMMIT-OBSERVATION until observed after merge). Windows preflight owner nominated: the sitting Qwen CTO seat (Windows FULL class; operator executes device-attached steps). |
| `ADV-CAO-CTO-20260821-001` | `CAO_TO_CTO.md` example only | `PENDING-POST-COMMIT-OBSERVATION` | `CTO_TO_CAO.md` | `N/A` | `EXAMPLE-NOT-ACTIVE` | None; non-functional schema example. |
| `ADV-CTO-CAO-20260821-001` | `CTO_TO_CAO.md` example only | `PENDING-POST-COMMIT-OBSERVATION` | `CAO_TO_CTO.md` | `N/A` | `EXAMPLE-NOT-ACTIVE` | None; non-functional schema example. |
| `ADV-CTO-CAO-20260821-002` | `CTO_TO_CAO.md` out-of-office + delegation record | `PENDING-POST-COMMIT-OBSERVATION` | `CAO_TO_CTO.md` | `N/A` | `OPEN-AWAITING-CAO-RETURN` | CAO: acknowledge on return. Operator ruling in effect meanwhile: iOS/OSX deploy builds only on gemini/MacBook via the gpt handoff protocol; all other iOS/OSX work on the Qwen lane. |
| `ADV-CAO-CTO-20260821-002` | `CAO_TO_CTO.md` | `5af89a3498b0438e3b9efdc401ec8d6129915177` | `CTO_TO_CAO.md` | `PENDING-POST-COMMIT-OBSERVATION` | `RECEIVED-OWNER-NAMED` | Qwen iOS lane: fold into the IOS-V050-1-REPAIR-2 fresh plan; operator privacy ruling before implementation claims. |
| `ADV-CAO-CTO-20260821-003` | `CAO_TO_CTO.md` | `5af89a3498b0438e3b9efdc401ec8d6129915177` | `CTO_TO_CAO.md` | `PENDING-POST-COMMIT-OBSERVATION` | `RECEIVED-OWNER-NAMED` | Qwen lane transport/security planning; AGENTS.md rule 8 + dual approval if scope reaches core. |
| `ADV-CAO-CTO-20260821-004` | `CAO_TO_CTO.md` | `5af89a3498b0438e3b9efdc401ec8d6129915177` | `CTO_TO_CAO.md` | `PENDING-POST-COMMIT-OBSERVATION` | `RECEIVED-OWNER-NAMED` | CTO seat (Windows FFI owner): reconcile generated Swift vs XCFramework drift at the freeze SHA, after PR #204 lands. |
| `ADV-CAO-CTO-20260821-005` | `CAO_TO_CTO.md` | `5af89a3498b0438e3b9efdc401ec8d6129915177` | `CTO_TO_CAO.md` | `PENDING-POST-COMMIT-OBSERVATION` | `RECEIVED-OWNER-NAMED` | Qwen lane owns the verify-test.sh fix; gemini deploy-build lane verifies with verbatim xcodebuild output. |
| `ADV-CAO-CTO-20260821-006` | `CAO_TO_CTO.md` | `5af89a3498b0438e3b9efdc401ec8d6129915177` | `CTO_TO_CAO.md` | `PENDING-POST-COMMIT-OBSERVATION` | `RECEIVED-OWNER-NAMED` | Qwen lane owns the ios-build-test.yml path-filter review; no release claim from review alone. |
| `ADV-CAO-CTO-20260821-007` | `CAO_TO_CTO.md` | `5af89a3498b0438e3b9efdc401ec8d6129915177` | `CTO_TO_CAO.md` | `PENDING-POST-COMMIT-OBSERVATION` | `RECEIVED-OWNER-NAMED` | Qwen Android lane via the Windows host; Pixel 6a evidence (adb active); bounded disposition before parity claims. |

`PENDING-POST-COMMIT-OBSERVATION` is a bootstrap placeholder, not a commit
claim. It must be replaced only by the Windows controller after it observes the
committed record. No runtime candidate, freeze SHA, artifact, hardware result,
approval, or release disposition is indexed here.

## Known open blockers

The following are open and are not closed by this bootstrap:

- Apple retained-delivery repair and fresh reviews; v0.5 request, block, and
  status UI; physical iPhone/macOS evidence; signing and device gates.
- Android non-destructive request enumeration, visible failed delivery state,
  and production mDNS permission/test fidelity.
- RESOLVED 2026-08-21: Windows CLI artifact readiness -- PR #203 merged as
  8b3ecfe5; the Windows CLI Artifact job publishes windows-cli-<sha> on
  every CI run (first green exercise observed on PRs #203/#204). Artifact
  hash evidence will be recorded at the freeze SHA.
- The five-node cloud-node custody gate remains separate from this four-node
  field gate.

## Merge check

Before a platform PR merges, its owning controller checks both journals for open
advisory records whose complete scoped paths overlap that exact candidate. It
resolves the overlap or records why it does not apply. Silence, a similar plan,
a branch name, or a deleted branch is never closure.
