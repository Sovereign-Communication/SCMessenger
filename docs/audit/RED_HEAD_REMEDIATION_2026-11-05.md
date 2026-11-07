# Red-car per-job classification and fork-head remediation patches

Status: classification complete; two patches verified; delivery is owner-blocked
Date: 2026-11-05
Lane: `lane/burndown` (SCMessenger)

Scope: the standing cluster order C1, C2, C4, C3 (#178 only), C5, #208. Every
car below was read at its exact `headRefOid`, every non-success job classified by
its own annotation, and the annotations (not the job names) decide the class.

## 1. Per-job classification, all cars in lane order

| Order | PR | head | non-success | billing | never-acquired | genuine | read |
|---|---|---|---|---|---|---|---|
| C1 | #660 | `210b7d932f4e` | 14 | 13 | 1 | **0** | 14:27-14:29Z |
| C2 | #611 | `e03c1d7df43a` | 8 | **8** | 0 | **0** | 14:36-14:37Z |
| C4 | #506 | `16b4edf7540b` | 11 | **11** | 0 | **0** | 14:36-14:37Z |
| C4 | #507 | `f4d5155d4af5` | 3 | 3 | 0 | 0 (structural, A25) | 14:28Z |
| C4 | #508 | `475816012256` | 11 | **11** | 0 | **0** | 14:36-14:37Z |
| C3 | #178 | `c1b457bbb1ce` | 3 | 0 | 0 | **1 root cause** | 14:36-14:42Z |
| C5 | #521 | `5c6b5e52ed29` | 14 | **14** | 0 | **0** | 14:36-14:37Z |
| C5 | #599 | `a7ef2f299753` | 0 | 0 | 0 | 0 (0/5 required exist, A17) | 14:36Z |
| - | #208 | `11f3e5e93f2a` | 6 | 0 | 0 | **2 root causes** | 14:36-14:42Z |

Read method: `commits/<head>/check-runs?filter=latest&per_page=100` plus
`check-runs/<id>/annotations` for every non-success job, 2026-11-05T14:27-14:42Z.

Conclusion for the billing class: **#611, #506, #508 and #521 are 44 of 44
billing refusals** - every non-success job carries "The job was not started
because your account is locked due to a billing issue" with `steps=0` and an
empty runner. None of those four has a single genuine failure, so none can be
improved by a re-run, a refresh or a re-base.

## 2. The two cars with genuine failures are the two fork heads

Both are from `pixiegirlchristy/SCMessenger` with `permissions.push=false`
(re-read 14:38:02Z), so the lane cannot deliver a repaired head (A10). Per the
lane rule, the deliverable the owner needs to act on is a verified patch, filed
here rather than described.

### #208 - two root causes across three jobs

| job | annotation | class |
|---|---|---|
| Repository Hygiene Checks (`97004746509`) | `HANDOFF/coordination/apple-windows/CTO_TO_CAO.md:310: new blank line at EOF.` | genuine, style |
| Rust Linting (`97004746383`) | `Diff in .../cli/src/ble_daemon.rs:76` | genuine, rustfmt |
| Lint (`97004746376`) | same `ble_daemon.rs:76` rustfmt diff | genuine, rustfmt (duplicate of the above) |
| Swift Linting (`97004746455`) | `The following taps are not trusted: aws/tap` | environmental (runner image) |
| `label`, Android JVM Unit Tests | Node.js 20 deprecation notice | tooling, not the branch |

Patch: `red_head_remediation/208-hygiene-and-rustfmt.patch` (21 lines, two files).

Verification on this host:

- `rustfmt --check --edition 2021` on the **original** `ble_daemon.rs` exits 1
  with the identical diff at line 76 that CI reported; on the **patched** file it
  exits **0** with no diff. The local toolchain reproduces CI's finding, so this
  is a real formatting defect and the fix is the one rustfmt asks for.
- `CTO_TO_CAO.md`: exactly 1 trailing byte removed; the file now ends with a
  single newline and no blank line at EOF (the hygiene rule CI enforced).
- Both files apply to #208's real head blobs and reproduce the verified build
  byte-for-byte (`git apply` then `diff`, `208 PATCH==VERIFIED-BUILD`).

### #178 - one genuine code defect

`iOS Build` (`95544925861`) failed with exit 65 and
`iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift:4414:10: error: invalid
redeclaration of 'sendMultipeerPacket(peerId:data:)'`. The file declares the same
method twice, at line 4351 and line 4414.

The two copies are **not** interchangeable, which is why the patch deletes a
specific one:

- line 4351 records `appendDiagnostic("multipeer_send_failed ...")` on both
  failure paths;
- line 4414 has no diagnostics and a different warning string.

Patch: `red_head_remediation/178-duplicate-multipeer-packet.patch` (25 lines)
keeps the diagnostic-instrumented copy and removes the later duplicate.

Verification: the patched file has **1** declaration (was 2), net brace balance
is unchanged at 0 (7250 -> 7234 lines), `sendBlePacket` and the functions around
the removed block are byte-identical, and the patch applies to #178's real head
blob reproducing the build (`178 PATCH==VERIFIED-BUILD`).

**Not verified here, and must not be reported as verified:** the iOS build
itself. `xcodebuild` exists only on the operator's Mac (AGENTS.md A6), so the
compile proof for this patch is the operator's, not this host's. The defect
signature is unambiguous (a duplicate declaration is a compile error, and exactly
one now remains), but green is unproven until an iOS gate runs.

## 3. Delivery - what the owner must decide

The patches cannot be landed by the lane, and the reason is access, not content.
Three concrete options, in order of cost:

1. **Author applies the patch** on the fork branch (the PR's own
   `maintainer_can_modify=true` is the author's standing offer, and these are her
   files). Cheapest, no access change.
2. **Grant the lane push on the fork**, then the lane pushes the repaired head
   and reports check-runs on it.
3. **Re-home the fixes** into a base-repo branch owned by this lane, which
   abandons the two PRs' authorship context and needs a product decision.

Until one of those happens, #178 and #208 stay red for reasons that are now
written down precisely, with a verified fix sitting in the repository.

## 4. Reads

- `commits/<head>/check-runs?filter=latest` + `check-runs/<id>/annotations` for
  #660, #611, #506, #507, #508, #521, #599, #178, #208, 14:27-14:42Z.
- `actions/jobs/97004746509|97004746383|97004746376|95544925861/logs` for the
  failure text and file:line, 14:39-14:41Z.
- `pulls/178`, `pulls/208` (`head.repo.full_name`, `maintainer_can_modify`) and
  `repos/pixiegirlchristy/SCMessenger` `.permissions`, 14:38:02Z.
- `refs/pull/208/head` and `refs/pull/178/head` fetched read-only into
  `refs/tmp/p208`, `refs/tmp/p178`; both patches proven by
  `git apply` + `diff` against the verified builds, 14:47Z.
- `rustfmt 1.10.0-stable`, `cargo 1.99.0` on this host, 14:39Z.

## 5. Delivery executed on the authors' PRs (2026-11-05T14:50:35Z)

Push to the forks is impossible (A10), so option 1 from section 3 was executed
directly instead of waiting: each patch was posted as a comment on the PR whose
branch it fixes, with the diff inline and the exact apply command.

- #208: `#issuecomment-6661021545` (2227 chars, patch block embedded).
- #178: `#issuecomment-6661022160` (2121 chars, patch block embedded).

Read back by id at 14:51:11Z: both bodies intact, fences balanced, diff lines
present (10 and 18 `+`/`-` lines respectively).

This is delivery of a fix, not a landing: the branches are unchanged, no CI was
dispatched, and the author or the owner still decides whether to apply it.

## 6. The defects are branch-local - main carries none of them

Checked so that a later reader cannot mistake these patches for main-side fixes,
and so that applying them cannot regress main:

| defect | main | fork branch |
|---|---|---|
| `sendMultipeerPacket(peerId:data:)` duplicate | **0** occurrences of the name in `MeshRepository.swift` (7498 lines) | 2 declarations at 4351 / 4414 (7250 lines) |
| `ble_daemon.rs` rustfmt diff | different call shape: `AssertUnwindSafe(manager.adapters())` at line 39; **0** occurrences of `AssertUnwindSafe(adapter.adapter_state())` | the line CI's rustfmt rejects |
| blank line at EOF | `HANDOFF/coordination/apple-windows/CTO_TO_CAO.md` does not exist; the main-side file is `HANDOFF/gpt/CTO_TO_CAO.md`, which ends with a single `\n` | `HANDOFF/coordination/apple-windows/CTO_TO_CAO.md:310` |

Read: `git show origin/main:<path>` and `git ls-tree -r --name-only origin/main`,
2026-11-05T14:49Z. Register row A30.
