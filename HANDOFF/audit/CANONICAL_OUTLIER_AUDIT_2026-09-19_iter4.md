# Canonical Outlier Audit -- iteration 4 (DIM-D: doc vs code / doc vs doc)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Task: HANDOFF/freebuff/queue/AUDIT_CANONICAL_OUTLIERS_V040_V050_V100_2026-09-19.md
Date: 2026-09-19
Branch: glm/canonical-outlier-audit
HEAD: 1acb63531aaa1e7c22071bb7430c72b5a9753210
Model: Buffy (Freebuff)
Mode: REPORT-ONLY

## Gates run (correct $? capture)

```
bash scripts/docs_sync_check.sh > tmp/canonical_audit_2026-09-19/docs_sync.txt 2>&1; rc=$?
RC=0
docs-sync-check: PASS
```

```
python scripts/check_wiring.py   # (DIM-C, RC=0, see iter3)
```

## Counts

- NEW findings: 3 (CO-D-001..003)
- Verified-consistent: 4 categories (end of file)

## Findings

### CO-D-001 -- DOCUMENTATION.md "Applies to" line still pins v0.3.5 against Cargo.toml 0.4.0; T11 remains open in queue
- Dimension: DIM-D
- Severity: MED
- Target: 0.4.0
- Location: `DOCUMENTATION.md:11` ("Applies to: v0.3.5 (alpha, working toward v1.0.0)") vs `Cargo.toml:9` (`version = "0.4.0"`)
- Authority contradicted: `Cargo.toml:9`; `docs/CURRENT_STATE.md:18-21` (correct
  claims: release line v0.4.0, rc.1 tagged, not released, latest public v0.1.9);
  task ticket `V040_T11_CANONICAL_DOC_RECONCILE.md` acceptance ("No canonical
  document contradicts another on version").
- Evidence: grep version-claims output quoted in transcript (Cargo.toml:9;
  DOCUMENTATION.md:11; CURRENT_STATE.md:18-21). T11 disposition:
  `ls HANDOFF/freebuff/queue/` still lists `V040_T11_CANONICAL_DOC_RECONCILE.md`;
  `ls HANDOFF/freebuff/done/` shows only README.md + V040_T5_DOCS_SYNC_GATE_IS_RED.md
  -- T11 never moved to done.
- Why it is an outlier: the docs ENTRYPOINT states a version one release line
  behind the workspace and behind its own linked CURRENT_STATE; exactly the
  false-claim class SHIP_PLAN 6.3 lists.
- Suggested remediation class: docs-correct (T11 remains the vehicle)
- Status: OPEN

### CO-D-002 -- Three dated 0.4.0 gate verdicts coexist unreconciled (sealed-ready / HALT / NOT COMPLETE)
- Dimension: DIM-D (doc vs doc)
- Severity: MED
- Target: 0.4.0
- Location: `HANDOFF/CTO_STATE.md:4` (committed HEAD copy:
  `tmp/canonical_audit_2026-09-19/CTO_STATE_HEAD.md`; working file is DIRTY from
  another session) "tag-readiness evidence SEALED -- tip CI 7/7 workflows green
  on 1aaf6d34 ... NO tag taken" (last updated 2026-09-15T17:55Z);
  `HANDOFF/audit/SHADOW_AUDIT_V040_V050_ADVERSARIAL_REVIEW_2026-09-16.md:13`
  "VERDICT: HALT v0.4.0 TAG (CRITICAL RELEASE BLOCKERS DETECTED)";
  `HANDOFF/audit/MULTIDIMENSIONAL_AUDIT_2026-09-17.md` section 9 "0.4.0 scope
  verdict: NOT COMPLETE" (TRN-04, TRN-07, AND-06, SEC-03 + un-re-verified rows).
- Authority contradicted: DOCUMENTATION.md / DOCUMENT_STATUS_INDEX rule that
  active docs must not contradict on "whether a gate has passed"; SHIP_PLAN
  checkpoint ledger expects one truth per criterion.
- Evidence: quotes above read this session (CTO_STATE read from COMMITTED HEAD
  because the working-tree file is another session's dirty edit; grep -iE
  "0.4.0|blocker|verdict|tag" output in transcript).
- Why it is an outlier: a reader of CTO_STATE alone sees a sealed, green,
  awaiting-only-operator package; the two newer audits reverse the readiness
  without CTO_STATE being annotated. This is REPORTED, not adjudicated: the tag
  go/no-go is operator-owned (task section 9).
- Suggested remediation class: needs-operator (annotate CTO_STATE with a dated
  pointer to the two newer verdicts, or record which supersedes)
- Status: NEEDS-HUMAN

### CO-D-003 -- docs_sync_check gate passes while a canonical version claim is stale: gate scope does not cover the claim class it exists for
- Dimension: DIM-D (process)
- Severity: LOW
- Target: process
- Location: `scripts/docs_sync_check.sh` (RC=0 PASS on this HEAD) vs
  `DOCUMENTATION.md:11` staleness from CO-D-001
- Authority contradicted: SHIP_PLAN 6 false-claim ledger intent;
  `HANDOFF/freebuff/queue/V040_T10_FFI_SURFACE_GATE_PASSES_VACUOUSLY.md`
  documents the same *pattern* for a different gate (T10's own subject).
- Evidence: RC=0 output above while DOCUMENTATION.md:11 is stale at the same
  HEAD (same session, same commands).
- Why it is an outlier: the mechanical gate is green, so T11-style staleness
  ships unflagged; "gate green" is being used as the doc-truth signal but does
  not check version-claim staleness.
- Suggested remediation class: inventory-ticket (extend docs_sync_check or
  accept T11 manual passes; operator's call)
- Status: OPEN

## Verified-consistent (no finding)

1. `docs/CURRENT_STATE.md` release-line claims match source: :18 release line
   v0.4.0 citing `Cargo.toml:9`; :20 `v0.4.0-rc.1` tagged -- matches local
   `git tag --list` (v0.4.0-rc.1 present, no final v0.4.0); :43 carries the
   T11-style dated honesty label ("baseline captured under v0.3.5; version
   corrected 2026-08-31, contents NOT re-verified since"). One sub-claim is
   UNVERIFIED locally: "no GitHub Release object exists for it" (needs `gh`;
   not queried this session -- marked, not asserted).
2. `DOCUMENTATION.md` link targets: all 45 link paths checked with `test -f`;
   zero MISSING (full file list in the command in transcript; no elision).
3. SHIP_PLAN checkpoint ledger CP2-CP6 empty -- consistent with "no tag taken"
   (CTO_STATE) and latest public release v0.1.9 (CURRENT_STATE).
4. No active doc describes a standalone relay product ROLE (DIM-A pass 2
   search: all "every node is a full relay" doctrine statements; RELAY_OPERATOR_GUIDE
   body explicitly denies the role -- filename outlier only, CO-A-005).

## Not done / UNVERIFIED

- GitHub release-object existence (needs gh/API; marked above).
- `HANDOFF/CTO_STATE.md` working-tree (dirty) content NOT audited -- another
  session's edit; committed HEAD copy used and labeled.

## Next iteration aim

DIM-E (bootstrap/hardcode/sled bypass) + DIM-F (parity debt candidates).
