# P1 - Track and remediate rustls RUSTSEC-2026-0285 (waived in deny.toml, untracked)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Closed -- waiver removed, guard re-armed (2026-09-25)
Priority: P1 (MUST be resolved before the first PUBLIC release; not a v0.4.0
tag blocker -- see scope note)
Filed: 2026-09-14 by the Freebuff BoD lane, passive audit of PR #288
Author of the waiver: Claude Cowork sandbox, commit b87e390f
Ticket origin: audit finding -- the deny.toml ignore entry was committed with
no tracking ticket; every other ignore in deny.toml carries one (precedent:
RUSTSEC-2026-0104, RUSTSEC-2026-0173 comments).

## The advisory (facts from rustsec.org, fetched 2026-09-14)

- RUSTSEC-2026-0285 / GHSA-2mjx-qc3c-rqvc / CVSS 5.3 MEDIUM.
- Package: rustls. Category: crypto-failure. Issued 2026-09-14 (same day as
  the waiver).
- Title: TLS 1.3 handshake messages incorrectly accepted across encryption
  level boundaries. A network-position attacker cannot alter or complete a
  handshake (transcript stays authenticated); the practical effect is that a
  peer can send should-be-encrypted handshake messages in plaintext without
  the connection being rejected. Functionally the same bug as Go's
  GO-2026-4340 (CVE-2025-61730).
- Patched: rustls >= 0.23.45. Unaffected: < 0.23.13.

## Why it matters here

SCMessenger's transport TLS (WSS paths, QUIC via quinn) terminates in rustls.
A crypto-failure class advisory in the TLS layer of a secure messenger must
not be silently perpetual. The waiver is defensible ONLY as a timed,
tracked bridge -- the advisory was issued the same day, and the patch (0.23.45)
post-dates the libp2p pin.

## Scope note (v0.4.0 tag vs public release)

- v0.4.0 tag / dev-channel: NOT blocked. The advisory is MEDIUM, affects
  handshake-level parsing (no integrity break), and the exposure window is
  the existing dev fleet.
- FIRST PUBLIC RELEASE: BLOCKED until remediation lands. Per the standing
  publish-gate conditions (CTO_STATE 2026-08-29 section), security posture
  must be honestly disclosed; a known-waived crypto-failure advisory in the
  TLS stack is exactly the kind of item the release body must either not
  carry or disclose.

## Remediation path (in order; STOP and escalate if a step breaks)

1. Inspect Cargo.lock: `rg -n "name = \"rustls\"" -A 1 Cargo.lock` -- record
   the resolved version(s). If >= 0.23.45 already (transitive refresh), the
   waiver can be REMOVED outright and this ticket closed as no-op.
2. If < 0.23.45: attempt a lockfile-only bump
   (`cargo update -p rustls --precise 0.23.45` or newer) and run the full
   gate suite (fmt, clippy, cargo test --workspace). rustls 0.23.x patches
   are semver-compatible; this should be a no-code-change bump.
3. If the bump is blocked by the vendored/patched crate chain
   (`patch/libp2p-quic/` pins quinn -> rustls): the fix lands upstream in
   that pin. Update the pin's rustls requirement, rebuild, gate, and record
   the pin change in the PR that carries it.
4. On success: remove the RUSTSEC-2026-0285 ignore from deny.toml in the
   same commit, so the guard re-arms (a waiver that outlives its advisory is
   a permanent blindfold).
5. If none of 1-3 is possible before public release: disclose the advisory,
   the CVSS, and the mitigation posture in the release body, and re-scope
   this ticket with an operator ruling.

## Resolution (2026-09-25) -- remediation path step 1, no-op removal

The waiver outlived its reason. The advisory is patched in rustls >= 0.23.45
and the lockfile already resolves the patched version, so step 1 of the
remediation path applies verbatim ("the waiver can be REMOVED outright and this
ticket closed as no-op") and steps 2-3 (lockfile bump) were not needed.

Evidence (all commands run in the merge-train worktree on this branch):

```text
$ grep -A 1 '^name = "rustls"' Cargo.lock
name = "rustls"
version = "0.23.45"
$ grep -c 'name = "rustls"' Cargo.lock
1
$ grep -c 'RUSTSEC-2026-0285' deny.toml     # after the edit
0
```

- `Cargo.lock` carries exactly ONE rustls entry, 0.23.45, which is the patched
  release named by the advisory (patched: >= 0.23.45). No second, unpatched
  rustls is reachable through any `[patch.crates-io]` entry: main's
  `Cargo.toml` has no `[patch]` section (the D9 libp2p-swarm vendor patch lives
  on the #372 branch, which does not pin or vendor rustls).
- The advisory check runs in CI in two places, and both are the verifiers for
  this change: `.github/workflows/ci.yml` (Lint job, `cargo install cargo-deny
  --version 0.20.2 --locked` then `cargo deny check`) and
  `.github/workflows/security.yml` (`cargo deny check advisories`). Green runs
  on this branch's head are recorded in the PR.
- `cargo deny check` was NOT run on this host: `python scripts/disk_budget.py`
  blocks local cargo builds, and CI is the only builder. The Lint job on the
  branch head is the verification, per the plan.

## Acceptance criteria

- [x] deny.toml no longer ignores RUSTSEC-2026-0285, OR the ticket carries a
      recorded operator ruling for why it must.
- [x] `grep 'RUSTSEC-2026-0285' deny.toml` is empty, and `cargo deny check`
      passes with resolved rustls >= 0.23.45 evidenced in Cargo.lock (the
      `cargo deny check` verdict is the CI Lint job on the branch head; the
      Cargo.lock evidence is above).
- [ ] Full gate suite green on the Windows host after the bump. NOT APPLICABLE
      and NOT CLAIMED: no bump was made, and this host cannot run cargo (disk
      guard). The equivalent gate is the full CI matrix on the branch head.
- [x] This file moved to HANDOFF/done/ with evidence headers.
