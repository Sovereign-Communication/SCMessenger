# SCMessenger issues and release status

Status captured: 2026-10-17T05:54:10Z. This is a point-in-time status note, not an execution authority. The active release target is v0.5.1; use the live issue/PR records and the repository's current execution-order guidance. The v0.4.0 [SHIP_PLAN.md](SHIP_PLAN.md) is historical for this release and must not override current v0.5.1 guidance.

## Snapshot

- Current checkout: `preserve/shared-checkout-20261003` at `e4050f84`; this is a preservation snapshot and **NEVER MERGE**.
- Fetched `origin/main`: `b62b631ba7954d8d81c328658c0a525dd6b05e79` (2026-10-16).
- Release target: v0.5.1, per the live v0.5.1 issue/PR tracking reviewed in this pass. The v0.5.0 tag exists; `git ls-remote --tags origin 'refs/tags/v0.5.1*'` returned no match at capture time. `gh release list --limit 1000` showed only a draft backup, not a v0.5.1 release.
- Live pull request inventory: 43 open PRs from `gh pr list`; the full captured list and exact query are in [prs.json](prs.json) (`captured_at_utc=2026-10-17T05:54:10Z`). It is not the old 45-entry inventory.

## Release blockers

1. **Main CI is red.** Run [38998920029](https://github.com/Sovereign-Communication/SCMessenger/actions/runs/38998920029) on `b62b631ba7954d8d81c328658c0a525dd6b05e79` failed in `Lint` at `cargo deny check`; Clippy, FFI surface, Windows/macOS/Ubuntu test jobs, Docs, and other recorded jobs succeeded. The deny output names advisories RUSTSEC-2026-0346, -0347, -0348, and -0349 against rustls 0.23.45, which requires >=0.23.46, and separately warns `chacha20 0.10.1` is yanked. PR #604 head `f4083681d81cda86eacd15d9aad8574c5e967a7d` is OPEN but BEHIND, has no submitted review, and its `Forgery/Negative Security Regression Tests` check is CANCELLED; do not treat the dependency patch as integrated or fully gated.
2. **Transport code is wired; acceptance is not proven by that fact.** The routing feed is called in native and WASM swarm connection paths in [swarm.rs](core/src/transport/swarm.rs#L6685) and through the mobile bridge. The CLI starts its seed-dial loop in [main.rs](cli/src/main.rs#L2660-L2666), and [seed_dial.rs](cli/src/seed_dial.rs#L83) calls `connect_to_seed_peers()`. These are reachable production paths, not zero-caller defects. The live transport-racing and three-node acceptance bars remain unverified.
3. **PR #521 / #599 remain gated.** #521 is OPEN at `230b6169d1d3b82554303f6f9231be1c4e4ed4e4`, BEHIND, with 34 check entries and no submitted review. Its stacked remediation #599 is OPEN at `280625bf37937fc6d7525331405daf8aa718b3db` on base branch #521; its nine Mobile checks succeeded except Android Release Build was SKIPPED, and it has no submitted review. Issues #573-575 and independent Rule-8 issue #577 remain OPEN; verify the documented last-path eviction and /24 fairness contracts on the final head before approval. The prior independent review findings are not themselves an approval for these exact PR heads.
4. **Device/release acceptance remains outstanding.** Issues #592 and #593 track operator-device tests. Release issue #603 requires the three-node verifier PASS plus Jev score >=0.99. These are not evidenced in this snapshot; no current device or cloud-node state was re-derived.
5. **The v0.5.1 burndown is active.** Tracker #595 and its listed items #570, #572-575, #577-582 remain OPEN as rechecked at capture time. Issue #603 specifies the release trigger: tri-node verifier exit 0 and Jev >=99/100, with release withheld on FAIL or INSUFFICIENT DATA. Operator acceptance issues #592 (cellular inbound evidence) and #593 (Pixel Stop/Start) are OPEN. No exact-candidate three-node PASS or Jev >=99/100 evidence was established here.

## Verification boundaries

This note checks source reachability, GitHub PR/CI state, and published-release listings. It does not certify message delivery, a signed release artifact, a three-node run, or Jev completion. No source code, release, merge, or deployment action was performed.
