# Dependabot security triage — pre-tag review (v0.4.0)

Status: OPEN — tracked for pre-tag review (2026-08-30). Does not block the merge lane.
Source: GitHub Dependabot alerts on `main` at `7672a5f5`.

## Findings (3 open; 2 high, 1 moderate)

| # | Sev | Advisory | Package | Lock version | Verdict |
|---|-----|----------|---------|--------------|---------|
| 1 | HIGH | GHSA-vxx9-2994-q338 — Yamux remote Panic via malformed Data frame (SYN set, len=262145) | `yamux` (transitive, libp2p mux) | `0.12.1` | **REACHABLE — ACTION REQUIRED** (fixed in `0.13.10+`) |
| 2 | HIGH | GHSA-3v94-mw7p-v465 — hickory-proto NSEC3 closest-encloser unbounded loop | `hickory-proto` (transitive, libp2p DNS) | `0.25.2` | **MITIGATED** — patched for `>= 0.25.0-alpha.3, <= 0.25.2`; current pin is in range |
| 3 | MEDIUM | GHSA-q2qq-hmj6-3wpp — hickory-proto O(n²) CPU exhaustion on name compression | `hickory-proto` (transitive) | `0.25.2` | **MITIGATED** — patched range `>= 0.3.1, <= 0.26.0` covers `0.25.2` |

## Action items

DEPENDENCY INVESTIGATION RESULT (2026-08-30, read-only; do NOT attempt a direct
Cargo.lock bump):

- `libp2p-yamux v0.47.0` HARD-pins `yamux = "^0.12.1"` (verified via
  `cargo update -p yamux@0.12.1 --precise 0.13.10` → rejected: needs `^0.12.1`).
- The `yamux 0.13.10` entry already in `Cargo.lock` is a DERELICT duplicate: its
  only nominal parent (`libp2p-yamux 0.47.0`) actually requires 0.12, and no crate
i-directly depends on yamux (the `"yamux",` strings in Cargo.toml are libp2p
  FEATURE flags, not deps).
- Therefore the reachable muxer IS `yamux 0.12.1` via the libp2p 0.56 stack, and
  eliminating it requires upgrading `libp2p-yamux` (the dependency/toolchain uplift
  epic), NOT a one-line lock edit. A direct 0.13 pin fails resolution and must not
  be committed.

1. **`yamux` 0.12.1 → HIGH, reachable on the p2p network surface.** The node listens
   for inbound connections, so the malformed-Data-frame Panic is attacker-reachable
   from another mesh peer. Fix = upgrade the libp2p stack so `libp2p-yamux` resolves
   `yamux >= 0.13.10`; a real build + adversarial review, tracked under the
   dependency/toolchain uplift epic. Do NOT fold into the D2/crypto tickets.
   Suggested owner: earliest 0.4.0-follow-up / 0.5.0 dependency uplift; release-level
   exposure decision stays with the operator.
2. **hickory findings (#2 HIGH, #3 MEDIUM) are MITIGATED at the pinned 0.25.2** —
   no version change required. Confirm on next full `cargo update`/lock refresh that the
   pin stays within the patched range before tag.

## Wire-in

- Recorded for pre-tag security review; revisit before creating the `v0.4.0` tag.
- Independent of: #244 (D4 coalescing), #245 (never-drop retry), #246 (desktop
  version), #247 (P0 send-crypto ticket). None touch the dependency set.