# SCMessenger Phase 2 additive backup plan

Status: Proposed; execution blocked pending owner approval and Overseer acknowledgement
Last updated: 2026-10-24 UTC (mission date 2026-10-23 HST)
Owner: Sovereign-Communication/SCMessenger; accountable operator Treystu
Scope: Preservation planning and publication of these planning documents only

## Decision requested and safety invariant

Review this plan and its complete item ledgers. Approval must identify this document manifest and its GitHub commit. Approval of this plan does not approve a backup batch, a CI configuration change, a purge, a PR change, or a release. Every future exact backup batch needs its own refreshed evidence, Jev v1 decision, owner execution approval, and Overseer acknowledgement. No such action executes in this planning phase.

The zero-loss guarantee is a deletion interlock: an item remains in place until its complete content is remotely retained, retrieved and restored successfully, and explicitly acknowledged. This document does not assert that dirty/ignored data or excluded locations are already preserved. Unreadable, changing, secret-bearing, unavailable, or owner-unknown items stay purge-ineligible. Metadata publication alone preserves no file contents.

The [published Phase 1 index](https://github.com/Sovereign-Communication/SCMessenger/issues/602#issuecomment-6380129325) is the inventory authority: 67 read-back-verified parts, two clones, 18 worktrees, seven dirty trees, 2,733 refs, zero stashes. Its report SHA256 is `3f90bd7b1671809c2a97b22a412cf87a9f8dd6afa9e6d67a56f00b0cf252c663`. This plan uses that inventory and targeted checks, not a new filesystem sweep. Main was captured and rechecked at `941e669da7b1fe9c0c5350dcd010e0ee5c6ecd58`. Fresh evidence time and source hashes are in [evidence](evidence.md).

## Disposition vocabulary and ownership

- **Preserve**: retain the original; after separate approval, create a new `backup/<class>-<name>-<sha12>-20261023` branch in the approved destination. If the name exists, compare and reuse positive proof or allocate a different new name. Never update it or force-push. Existing branch pointers and PR heads remain untouched.
- **Prove**: an identified live GitHub ref/ancestry already retains the exact object. Re-read and restore it before relying on it. API object availability, tracking names, patch equivalence, commit subjects and a green build are insufficient durability or supersession proof.
- **Owner-decision**: operator Treystu is accountable for assigning the named custodian and obtaining a recorded decision. Until accepted, preserve locally and prohibit purge. Secret material uses an owner-controlled encrypted archive, never a shared branch. Explicit discard is a separate owner decision; this plan authorizes none.

The WIP planner/executor owns the non-secret preservation ledger and verification receipts. Existing lane owners retain their files and processes. The Overseer independently acknowledges exact receipts on #602. A role named below is a requested accountable role, not a claim that the person has accepted or completed it.

## 1. The 29 initially UNPROVEN refs

[Every ref, full SHA, location, witness and action is listed here](unproven-refs.md). All 29 are primary-clone tracking refs, representing 25 unique commit objects. Targeted `git ls-remote` checks of the canonical repository, Treystu fork, pixiegirlchristy fork and private archive found exact live witnesses for 24 refs (20 unique objects). They receive **prove**, not duplicate upload. Thirteen refs point to the private archive; their content must not be copied to the public repository. Eleven rows refer to seven unique objects on the pixiegirlchristy fork. Require each remote custodian's retention acknowledgement or a separately approved, visibility-compatible new backup.

The five rows without a known live witness receive **preserve** after approval: `refs/remotes/pr/528` at `f5fd47555d1d51d9d718b5bd0718aa1bfa6a2c53`, and Treystu tracking refs for libc at `6b422285864922b18246dad61b66935c4a5889ec`, rustc-hash at `abfb99fb1407c1dfe26c5c5ce3aba7e88eee714d`, x25519-dalek at `a13db14436c189a7ea8d3380f3ae585929426c28`, and rust-toolchain at `b3a449281fe5a4f164de86730db75ed5a6619f2d`. Their prior exact five-ref batch remains [Jev-escalated](https://github.com/Sovereign-Communication/SCMessenger/issues/602#issuecomment-6380261543). This plan does not resolve or execute that batch. Availability through the public commit API is not live-ref retention.

All five detached HEADs and all remaining refs retain their inventory **prove** status only for the recorded objects, not uncommitted files. [Noncommit refs](noncommit-refs.md) retain raw-object and peeled-target identities separately. Branch refs must point to commits. For a tree or annotated-tag object lacking raw-object retention, preserve its exact typed bytes through the carrier protocol below; never replace or rewrite a tag. Commit ancestry alone does not retain an annotated wrapper.

## 2. Unreachable objects and commit frontiers

[Commit ledger](commits.md) and [complete object ledgers](objects-index.md) assign every inventoried object a disposition. Transitive graph analysis, not just immediate-parent subtraction, gives 79 maximal commit frontiers covering all 97 primary unreachable commits. One frontier, `6fe7e9b6525aefff01d67290f8c6f2b9066c2a81`, has an exact live `refs/claude-teleport/c937f977ffe42da0` witness: **prove** with explicit retention acknowledgement because that namespace is not a stable backup guarantee. The other 78 frontiers are **preserve**. Their closure retains the other 18 non-frontier commits as well; those ancestors need no separate ref after closure verification. No commit is declared superseded by this plan.

Primary inventory totals are 19,784 objects: 97 commits, 9,910 trees and 9,777 blobs. Frontier closure covers 368 inventoried objects; one of those is already live. The remaining 19,416 objects (9,711 trees and 9,705 blobs) lie outside both known live ancestry and frontier closure. Each receives **preserve / owner secret screen**, not discard. Negative coverage remains incomplete where live tips are unavailable locally. The second clone's 14 objects (one commit, eight trees, five blobs) all have positive known-live ancestry and receive **prove**. Its single commit frontier is still individually listed. Every positive proof must be refreshed before execution or deletion.

Executor procedure after approval: first retain the non-secret commit frontiers through NEW backup branches. For orphan objects, noncommit wrappers, or a raw typed object not retained by traversal, create a carrier commit on a NEW backup branch containing a self-contained Git pack (or exact `type`, size and raw bytes for each object) plus an object-ID/type/size/SHA256 manifest. Preserve the entire referenced closure, including object dependencies outside the inventoried set. A metadata list or mode-160000 gitlink does not preserve objects. Validate pack prerequisites and full closure in an empty object database. Do not assume that pointing a branch at a commit preserves unrelated blobs/trees.

Secret-screen all proposed objects and their complete newly uploaded closure without printing values. Scanner uncertainty, unknown binary content, identity/database payloads, or a credential match routes the affected archive to **owner-decision**. The operator's secret custodian keeps a restorable encrypted archive in an approved private vault; plaintext secrets never enter a shared branch, including a private shared repository. Publish only safe receipt metadata. Such items cannot satisfy the mission's GitHub-only requirement without an explicit owner-approved exception; until then closure and purge remain blocked. No object content was inspected for this planning proof, so no object is certified secret-free now.

## 3. Seven dirty worktrees

[The seven-tree ledger](dirty-worktrees.md) records exact paths, HEADs, branches and all inventoried tracked/untracked status entries. Targeted checks found the same HEAD and porcelain status entries in all seven at the evidence timestamp; raw byte equality was not measured. Ignored contents were not read or certified. The extra isolated writer worktree for these planning documents is not a retroactive addition to the inventory.

| Item | Custodian to acknowledge on #602 | Disposition and required capture |
|---|---|---|
| D1 shared checkout, staged CEO/CTO docs and untracked reports/task documents | Shared-checkout CEO/CTO lane owners; Treystu resolves ownership | Preserve after owner quiet-window approval; separate original index and working-file snapshots; never commit their current index |
| D2 nested older work-ahead backup tree, 15 untracked files | Original work-ahead lane owner, assigned by Treystu | Preserve all 15 and ignored payload inventory; an existing backup branch at HEAD does not cover these files |
| D3 live-state-checkpoint, two modified state documents | Live-state lane owner, assigned by Treystu | Preserve raw working bytes and original index separately |
| D4 scratch tree, credential-looking environment backup | Treystu's secret custodian | Owner-decision for the whole dirty capture until safe partition is reviewed; never read values into output or upload to a shared branch |
| D5 active burn-down tree, modified admission configuration | Active Merge Train/controller owner | Preserve only after owning lane acknowledges a consistent capture point; do not change its branch, index, config or process |
| D6 nested active #510 SwiftLint/iOS work | PR #510 lane owner | Preserve after owner capture acknowledgement; active file edits remain untouched; no duplicate implementation or PR push |
| D7 second clone, CLI/outbox changes and three untracked tests | OC worktree owner, assigned by Treystu | Preserve non-secret code after ownership acceptance; ignored contacts/history databases and tmp evidence require owner-controlled privacy handling |

For each accepted non-secret snapshot, work from an isolated copy and an alternative index, never `checkout -b`, stash, stage or commit in the shared source. Before capture, record source HEAD, branch, common Git dir, full original index bytes, split-index dependencies/sharedindex files, every conflict stage and staged blob closure. Record raw tracked, untracked and ignored file bytes, deletions, empty directories, links/reparse metadata, permissions/ACLs and alternate data streams where present. Git clean filters or CRLF normalization must not alter the raw-byte archive. Include nested worktrees as separately identified scopes to avoid omitting or duplicating their changing content.

Obtain an owner-agreed quiet window or consistent owner-provided snapshot. Compute pre/post per-file byte hashes and a complete directory manifest; any change, unreadable path, metadata failure or external link target invalidates that capture. Retry only through the owner; do not stop processes. For live databases use the owner's consistent database backup/export mechanism and validate restore. Preserve local Git administrative material (refs, reflogs, config with credentials quarantined, index dependencies, alternates, shallow/promisor state, LFS/submodule payload dependencies) through an owner-approved archive; a commit history alone does not cover it.

Package safe raw snapshots as explicit allowlisted payloads in NEW `backup/wip-<D-id>-<name>-20261023` carrier branches, with new commit subject `wip: backup 2026-10-23 [skip ci]`. Preserve original HEAD and index/working state independently in that archive. Exclude no ignored directory merely because of its name. Classify regenerable build/dependency contents only after proving no unique files; unresolved bytes remain owner-decision and purge-ineligible. Restore into a separate empty location and verify every byte hash, link type, deletion and stage entry. Run no files or filters from the archive. Original source pointers and bytes must remain unchanged throughout.

Both inventories report zero stashes. Recheck `refs/stash`/stash list immediately before each approved capture. A newly appearing stash becomes a new preserve item, including its index and untracked parents, with a separate manifest. No stash creation or deletion is planned here.

## 4. Missing live tips and tracking refs

[All 1,000 captured entries are reconciled individually](missing-tips.md). Primary's 37 entries are PR merge refs; second's 963 comprise 514 branch heads, 332 PR refs, 116 tag records and one other ref. These numbers describe live GitHub tips whose objects were missing in that clone's captured store. They are not evidence of loss from GitHub or 1,000 only-local refs. Peeled tag advertisements and raw tags remain distinct records.

Fresh `ls-remote` evidence shows 998 entries still advertise the captured SHA: **prove**, then fetch into an isolated verifier to validate closure; never reset source tracking refs. Two entries, both clone records of `refs/pull/528/merge`, advanced from `442b44cbc6ba848bb8a788e3789bc6ba1018233d` to `8d26d8d85aafca3a89aa1f9622f1066b54c445ff`: **preserve old SHA unless live ancestry is proven**. No captured ref disappeared in this check. The old merge object is not available locally in either inventory; executor must test full-SHA API/fetch availability and live ancestry, then retain exact bytes in a new backup if recoverable and not already retained. If unrecoverable, the PR #528 custodian and Treystu must record the specific missing-object risk; parent/head equivalence cannot certify exact old merge bytes. No synthetic merge is silently discarded.

For future ref drift, compare repository, full refname and raw SHA against the captured ledger: same is prove; advanced/deleted triggers old-SHA ancestry proof or preserve; unavailable on both sides is owner-decision, never presumed absent everywhere. A stale local tracking ref is still a work product until its old object is retained or explicitly dispositioned. The 29 rows above apply this rule to tracking refs. API 200 proves object access only; API 404/422 and abbreviated SHA ambiguity do not prove nonexistence. Every execution batch refreshes active claims and exact ref witnesses.

## 5. Discovery exclusions and residual risk

| Excluded class | Assigned accountable owner | Acceptance before any affected purge |
|---|---|---|
| node_modules and target | Each worktree owner; WIP executor records decision | Classify actual files, prove reproducibility or capture unique/modified bytes, identities and evidence; no blanket deletion exemption |
| .git internals | Each clone custodian | Verify complete objects plus refs/reflogs/index/alternates/LFS/submodule dependencies and credential-safe administrative archive; preserve unknown internals |
| 7,280 discovery errors/inaccessible or vanished paths | Treystu assigns host filesystem custodian | Resolve each inventory error path or explicit owner-signed exclusion with residual risk; permissions failure is not empty coverage |
| 706 link/reparse exclusions | Host filesystem custodian and target owners | Resolve literal link and target ownership, volume and scope; preserve link metadata and independently attest/capture targets; never follow outside scope silently |
| Other users or machines | Treystu and each device/account owner | Named custodian attestation and manifests/remote receipts for each relevant checkout; current host cannot certify their disks |
| Unmounted/offline/removable drives | Host/device owner | Enumerate intended volumes and mount/verify or explicit signed residual-risk acceptance; no global zero-local-work claim until resolved |
| Unmarked bare repositories | Host filesystem custodian | Bounded targeted bare-repo discovery and origin verification under owner-approved roots; archive or signed exclusion, not a guessed absence |
| Ignored private databases, keys, evidence and non-Git files | Data owner/secret custodian | Consistent encrypted backup and restore receipts under approved custody; unresolved data remains in place |

The original scan visited 215,604 directories and inspected a union of 793 candidate paths under C:/Users/SCM, C:/tmp, C:/temp and C:/ocv. These limits remain explicit. This planning phase does not reopen that sweep or claim physical-disk completeness. The published inventory retains every error/link record. An owner-signed residual-risk acceptance documents the gap but is not 100% preservation proof and cannot itself authorize purge of that unverified content.

## 6. Other-machine priority commits

| Prefix | Proposed accountable custodian | Disposition and acceptance |
|---|---|---|
| `6d75b5b9` (api/send option-B) | FreeBuff laptop lane owner; Treystu confirms actual holder | Owner-decision, then preserve: actual machine and checkout path, unambiguous full SHA/type, parents/tree, source ref, secret clearance, new backup ref and remote retrieval proof |
| `b686ebd5` (bundle signature verify) | Same | Same, independently verified; no guessing full SHA from prefix |
| `876f5700` (outbox drain spellings) | Same | Same, independently verified |

These prefixes did not resolve in either captured local object store. Canonical API lookups returned HTTP 422 and remain inconclusive. No claim of current possession, backup or global absence follows. The WIP planner has not established access to the laptop from this host. Treystu must name the executing laptop custodian on #602; that custodian checks full object IDs locally, reserves unique backup names, obtains the same secret/CI/Jev approvals, and posts GitHub full-SHA read-back and restoration evidence. If the laptop is offline, preserve its storage unchanged and record a waiting owner-decision. Other laptop WIP is not limited to these three commits. Without that attestation, mission completeness and any laptop purge remain blocked.

## Zero-CI prerequisite and sequence

No CI settings are changed by this plan. The private archive `Treystu/SCMessenger-backup` currently returns Actions `enabled=true`; it is **not an approved CI-inert destination**. Cancellation after a run starts cannot satisfy zero backup CI. `[skip ci]` is additional protection for new commits and is not a universal event filter.

Before any data backup, the operator/CI custodian must designate a visibility-compatible destination with Actions disabled, read back `actions/permissions.enabled=false`, and attest or audit repository/org hooks, installed GitHub Apps, external CI and automation. Require zero reacting integrations, no concurrent re-enable, no PR/tag/release/dispatch, and recheck immediately before each write. If disabling an existing repository could affect active work, use a separately authorized dedicated inert destination; do not change canonical CI or cancel lane jobs. Creating/configuring any repository is a separate owner-approved action. If this proof cannot be obtained, backups stay blocked.

Current canonical-main workflow review supports publishing these planning documents on a NEW non-main branch only: all branch push triggers are main-only, tag triggers use v*, and no create workflow exists. Repo hooks were empty. A `[skip ci]` docs commit with no PR/tag/dispatch adds protection. This limited planning publication proof does not cover arbitrary historical backup workflows or undisclosed external integrations; monitor only the exact planning branch and halt on unexpected activity.

After owner approval and Overseer acknowledgement of the plan, execute separately gated small batches in this order: (1) re-prove existing witnesses without copying private data; (2) preserve the five unresolved tracking commits and 78 unresolved commit frontiers after complete-closure secret screening; (3) preserve orphan/noncommit objects after screening; (4) owner-consistent dirty snapshots; (5) remaining missing-object, laptop and discovery-gap work. Refresh lane claims before every batch. Protect Batch C and #510/#556/#521/#599; never write existing heads #489/#521/#550/#554/#556/#562. No automatic expiry permits taking over claimed work.

## Remote verification, restoration and acknowledgement

For every future batch, publish a complete preflight manifest: source repository/common-dir/worktree, source full SHA or file manifest, object types/closure, destination visibility, proposed new names, CI-inert proof, secret clearance, owner quiet-window receipt, Jev decision and independent review. Check destination nonexistence immediately before creation. Use create-only semantics; no PATCH/update, force or existing-ref write. Handle an ambiguous network result by querying the same name and comparing the exact SHA before any retry.

Read back each new branch with both `git ls-remote --exit-code <destination> refs/heads/backup/<name>` and GitHub REST `GET /repos/<owner>/<repo>/git/ref/heads/backup/<name>`. Compare full SHA to the expected manifest, not abbreviated SHA or returned success alone. Fetch into an isolated empty verifier, validate object type, parents/tree and complete closure with fsck/rev-list, and restore every carrier/object pack/raw file/index stage. Verify SHA256 raw-file manifests and recomputed Git object IDs and sizes. Check for LFS/submodule/promisor/alternate dependencies; a restore needing the original disk is a failure. Retrieve encrypted owner archives and test restore under owner custody; publish no plaintext or sensitive hashes without owner clearance.

Post expected and observed full SHAs, ref URLs, manifest hashes, verification commands/results, zero-CI evidence and safe restore receipts on #602. Require a second independent Overseer read-back. Explicit acknowledgement must name the plan commit, batch manifest hash, every ref/SHA and all exceptions; a generic reaction, silence or prior acknowledgement does not qualify. Preserve originals while acknowledgement is pending.

Deletion remains outside this plan. A later deletion proposal must enumerate exact absolute paths and complete content scope, show 100% coverage including ignored/private/admin state, confirm no active lane claim/running image/dependency, refresh remote retention, and obtain both explicit Overseer acknowledgement and separate operator deletion approval. No purge if any item is unknown, merely patch-equivalent, inaccessible, secret-unresolved or covered only by API object availability. Never touch preserve/ directories or protected evidence. Trash emptying requires its own operator confirmation.

## Publication and review acceptance

The WIP planner publishes this document, every item ledger, source-evidence hashes, independent review and actual Jev receipt under one NEW planning branch. A manifest binds the exact document bytes. Verify every GitHub-read-back file against that manifest and link them on #602. This is documentation publication only; it creates no data-backup refs and changes no CI/PR state.

Jev v1 evaluates the exact document manifest and the planning-publication action before outward publication. Apply the established destructive >=0.5 escalation, disposition-lead <0.20 escalation and advances-goal <0.95 investigation rules. Record the actual decision without relabeling it. Investigate below-bar results using these ledgers and independent review; resolve only evidence-backed planning uncertainty under the owner's instruction that actual risk blocks. A genuine unresolved risk or explicit unresolved escalation holds publication. Plan publication never grants backup or purge authority. The recorded decision and any investigation are linked in [gate](gate.md).

Acceptance for owner review: all 29 refs, 19,798 unreachable objects, seven dirty trees, 1,000 missing-tip records, all exclusion classes and three priority prefixes have explicit preserve/prove/owner-decision routes; counts reconcile; private contents are absent; scoped documentation is remotely SHA-verified. Execution acceptance remains separate and is currently unmet for CI-inert destination, ownership acknowledgements, secret clearance/restores, gaps and laptop custody. No deletion or global preservation claim is made.
