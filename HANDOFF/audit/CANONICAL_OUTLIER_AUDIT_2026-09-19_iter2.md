# Canonical Outlier Audit -- iteration 2 (DIM-B: canonical identifier outliers)

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

## Authorities re-read this session

- `core/src/api.udl:41-46`: "IdentityInfo -- canonical identity = public_key_hex
  (Ed25519 hex)... Always use public_key_hex for persistence, exchange, and
  cross-platform resolution."
- `core/src/message/types.rs:41`: "UNIFICATION_V2: canonical identity is
  public_key_hex; identity_id/libp2p_peer_id are derived metadata only."
- `core/src/identity/keys.rs:39` `identity_id_from_public_key_hex`;
  `core/src/store/ledger_entry.rs:435/454` peer_id <-> pubkey derivations.

## Evidence commands

code_search: `drain_for_peer|queue:|peer_id.to_string|contact.peer_id` (cli/src),
`queue:|fn drain_for_peer|recipient_id|canonicalize` (core/src/store),
`\.enqueue\(|flush_outbox_for_peer` (repo; [WARNING] output cap hit at 84
matches -- wasm/lib.rs rows captured before the cap; test-file rows beyond the
cap were not needed for the classification and are not claimed).
Shell: `git show HEAD:core/src/iron_core.rs > tmp/canonical_audit_2026-09-19/iron_core_HEAD.rs`
(iron_core.rs is DIRTY in the shared checkout -- another session's edit -- so
all iron_core citations below are from the COMMITTED HEAD copy);
grep/sed on wasm/src/lib.rs, ContactsViewModel.kt, DashboardViewModel.kt,
docs/ID_*.md.

## Counts

- NEW findings: 6 (CO-B-001..CO-B-006)
- Prior-audit rows re-verified RESOLVED at HEAD: 2 (SHADOW CLI-03, SHADOW CORE-02)
- Verified-consistent claims: 2 categories (see end)

## Findings

### CO-B-001 -- Wasm outbox flush keys on base58 while every enqueue is hex-keyed: single-form drain can strand messages
- Dimension: DIM-B
- Severity: HIGH
- Target: 0.4.0
- Location: `wasm/src/lib.rs:1874` (`inner.flush_outbox_for_peer(&pid_str)` where
  `pid_str = peer_id.to_string()` of a libp2p `PeerId` -> base58); `IronCore::flush_outbox_for_peer`
  at HEAD `core/src/iron_core.rs:3871-3873` (single `drain_for_peer(peer_id)`,
  no hex fallback); enqueue path `iron_core.rs:898` (`hex::decode(recipient_id)`
  or InvalidInput) and `:1080` (outbox key = `recipient_id.to_string()`, hex).
- Authority contradicted: api.udl:41-46 / types.rs:41 canonical-identity rule;
  task severity guidance "identifier dual-key bugs that strand messages".
- Evidence: sed output of wasm/src/lib.rs:1871-1877 quoted in transcript;
  HEAD iron_core lines quoted; `grep -n flush_outbox_for_peer` on HEAD copy.
- Why it is an outlier: outbox is keyed by canonical hex, but the wasm flush
  surface can only ever pass base58 -- `drain_for_peer` (outbox.rs:399,
  in-memory map keyed by exact string) cannot match, so wasm-queued messages
  are not cleared by the PeerDiscovered flush. Mitigating context: wasm outbox
  is in-memory (`Outbox::new()` at iron_core.rs:371/607 for non-storage
  constructors), so loss is process-scoped, not disk-persistent. The CLI path
  is already fixed by dual-drain (`cli/src/main.rs:3764-3772`), which proves
  the intended pattern; the core API itself still exposes the single-form trap.
- Suggested remediation class: code-fix (dual-drain inside
  IronCore::flush_outbox_for_peer or canonicalize at the outbox boundary) --
  REPORT-ONLY here
- Status: OPEN

### CO-B-002 -- Orphan doc instructs libp2p_peer_id as THE canonical contact identifier
- Dimension: DIM-B
- Severity: HIGH
- Target: 0.4.0
- Location: `docs/ID_UNIFICATION_IMPLEMENTATION.md:53` ("**Contact storage and
  UI routing MUST use `libp2p_peer_id` as the canonical contact
  identifier.**"), also :64, :78
- Authority contradicted: `core/src/api.udl:41-43` (canonical = public_key_hex);
  `core/src/message/types.rs:41`; AGENTS.md identity doctrine restatement.
- Evidence: grep `peer_id.*(canonical|primary|persistent)` output; `head -15`
  shows self-label "Status: Active Implementation" dated 2026-03-10 (pre-dates
  UNIFICATION_V2); grep for the filename in DOCUMENTATION.md /
  DOCUMENT_STATUS_INDEX.md / docs/CURRENT_STATE.md returned NOTHING -- the doc
  is indexed nowhere yet claims Active.
- Why it is an outlier: any agent or contributor following this doc implements
  exactly the non-canonical keying the identity contract forbids; it is also a
  doc-status outlier (Active claim, unindexed, stale).
- Suggested remediation class: docs-correct (mark Superseded/Historical with
  dated correction pointing at api.udl; do not delete)
- Status: OPEN

### CO-B-003 -- Index lists ID_MANAGEMENT_ANALYSIS.md as Active while the doc self-labels Complete and its diagram marks the derived id canonical
- Dimension: DIM-B (doc-vs-doc + doc-vs-code)
- Severity: MED
- Target: 0.4.0
- Location: `docs/DOCUMENT_STATUS_INDEX.md` section 2 row
  `docs/ID_MANAGEMENT_ANALYSIS.md | Active`; `docs/ID_MANAGEMENT_ANALYSIS.md`
  header ("Status: Complete", dated 2026-04-09) and `:24`
  (`A -->|Canonical| C[libp2p_peer_id]`)
- Authority contradicted: api.udl:41-46; DOCUMENT_STATUS_INDEX update rule 7
  (status headers must be accurate).
- Evidence: grep outputs quoted; note `:197/:207` of the same doc DO describe
  hex storage (`peer_id TEXT PRIMARY KEY -- This is canonicalPeerId
  (public_key_hex)`) -- the doc is internally mixed, pre-V2.
- Why it is an outlier: status mismatch (Active vs Complete) plus a diagram
  edge asserting the derived transport id is canonical; readers of an
  index-blessed "Active" doc get pre-V2 doctrine.
- Suggested remediation class: docs-correct
- Status: OPEN

### CO-B-004 -- FFI managers open their own sled databases outside core/src/store (rule 7 / dual-store debt)
- Dimension: DIM-B (also DIM-E architecture)
- Severity: MED
- Target: 1.0.0
- Location: `core/src/contacts_bridge.rs:118-125` (opens `contacts database`
  via `sled::Config::default()` directly); `core/src/mobile_bridge.rs:3176-3180`
  (`HistoryManager` opens `<storage>/history.db` via `sled::Config::default()`)
- Authority contradicted: AGENTS.md rule 7 ("Storage access only through
  `core/src/store/`; IronCore is the single entry point -- never bypass it with
  direct sled access").
- Evidence: `grep -rn "sled::" core/src cli/src | grep -v core/src/store`
  output (quoted in transcript; non-test hits are these two files).
- Why it is an outlier: two more sled databases with lifecycles independent of
  IronCore; exactly the "dual stores" debt the 1.0.0 unification ledger is for.
  Nuance for the reviewer: these are uniffi-exported FFI managers inside
  core/src, likely wrapping store/ types for data access; the DB *handles*,
  paths, and open/retry policy are outside IronCore either way.
- Suggested remediation class: inventory-ticket (route handle ownership through
  IronCore at 1.0.0)
- Status: OPEN

### CO-B-005 -- Parity-audit claim drift: PeerKeyUtils peer-id math now used outside the cold-start fallback path
- Dimension: DIM-B (parity re-check)
- Severity: MED
- Target: 0.5.0
- Location: `android/.../ui/viewmodels/ContactsViewModel.kt:461-463`
  (`PeerKeyUtils.extractPublicKeyFromPeerId(event.peerId)` fills `publicKey`
  on live nearby-peer events when the existing record lacks one);
  `DashboardViewModel.kt:585,612-621` (explicitly labeled "cold-start"
  fallback -- consistent with the audit); `utils/PeerIdValidator.kt` (7
  BigInteger refs per MULTIDIM row AND-06, re-counted below).
- Authority contradicted: `HANDOFF/audit/IDENTIFIER_PARITY_AUDIT_2026-09-15.md`
  claim "7 call sites in MeshRepository.kt -- all in the cold-start/FFI-unavailable
  fallback path".
- Evidence: grep PeerKeyUtils callers across android main (4 files, listed);
  sed of ContactsViewModel.kt:455-470 and DashboardViewModel.kt:585-621.
  AND-06 re-count: `grep -c BigInteger PeerIdValidator.kt` NOT run to a number
  in this session -- the MULTIDIM count of 7 stands UNVERIFIED by me (Rule 12:
  not claimed).
- Why it is an outlier: the mirror is no longer fallback-only, so the
  "Kotlin never recomputes identity outside cold start" defense has eroded;
  consolidation ticket (P1_ANDROID_UNIFFI_CURVE_CHECK_RELOCATION) remains the
  structural fix.
- Suggested remediation class: inventory-ticket
- Status: OPEN

### CO-B-006 -- Doc code-sample comment labels a peer_id field "Canonical peer identifier"
- Dimension: DIM-B
- Severity: LOW
- Target: unknown
- Location: `docs/DEVICE_PEER_RELATIONSHIP_ANALYSIS.md:146`
- Authority contradicted: api.udl:41-46
- Evidence: grep output quoted (`pub peer_id: String, // Canonical peer identifier`).
- Why it is an outlier: stale canonical labeling in an index-Active doc.
- Suggested remediation class: docs-correct
- Status: OPEN

## Prior-audit rows re-verified RESOLVED at HEAD (not findings)

1. **SHADOW audit CLI-03 (outbox flush addressing mismatch strands all
   messages)** -- NOT present at HEAD: `cli/src/main.rs:3764-3772` drains BOTH
   the base58 form and the extracted-hex form (`drain_for_peer(&hex_pk)` after
   `extract_ed25519_public_key_from_peer_id`). Residual note: if the peer id is
   not Ed25519-extractable, only the base58 form is drained; entries keyed
   under hex would then strand -- same class as CO-B-001 but with the fallback
   guard present. DISPOSITION: resolved, residual absorbed into CO-B-001.
2. **SHADOW audit CORE-02 (IronCore in-memory outbox split-brain)** -- NOT
   present at HEAD: `with_storage` uses `Outbox::persistent(backend.clone())`
   (HEAD iron_core.rs:476); `Outbox::new()` only in the two in-memory
   constructors (:371, :607).

## Verified-consistent (no finding)

- **Kotlin does not reimplement blake3**: `grep -rn "blake3" android main .kt`,
  comment-lines excluded, zero code hits. Parity-audit claim holds.
- **12D3KooW literals** present in 10 non-test source files (list obtained:
  contacts_bridge.rs, identity/keys.rs, message/identity_envelope.rs,
  relay/invite.rs, store/blocked.rs, store/history.rs, transport/addr_filter.rs,
  transport/swarm.rs, wasm_support/rpc.rs, cli/ble_mesh.rs). Pattern presence
  verified; per-literal classification as format-prefix detection NOT read
  file-by-file this session -- per-file verdict UNVERIFIED, no violation claimed.

## Not done / UNVERIFIED

- AND-06 BigInteger re-count (stated UNVERIFIED above).
- Per-literal 12D3KooW classification (stated above).
- ios MeshRepository.swift not re-searched this session (prior audit's iOS
  claims re-cited as historical, not re-verified; no iOS toolchain and no
  source-tree change claim made).

## Next iteration aim

DIM-C: run `python scripts/check_wiring.py` (full output pasted), spot-check
flagged dead routes / manifest entries.
