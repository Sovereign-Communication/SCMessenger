# Ticket: G1 ledger retire/prune for ghost peer_ids

Status: OPEN
Filed: 2026-09-11T06:07Z by CTO seat
Owner: implementer worker (orchestrate packet) — not CTO controller
Related: GHOST-IDENTITY-001 RCA `HANDOFF/audit/RCA_OLD_PIXEL_577fd171_2026-09-11.md`
         fix commits `b258f1db` / `fe6f895f` on PR #281

## Problem

After identity rotation (pm clear / reinstall), `files/ledger.json` still
contains the retired public key as a dialable peer_id:

```
peer_id:    577fd1715f9f95fae10da5ea01aa20ac6789dfd898c3351ca3b6f62b249c4fb8
public_key: null
multiaddr:  /ip4/192.168.0.134/tcp/9001   # THIS device
success_count: 0
failure_count: 24
```

UI and gossipsub auto-subscribe are now gated (GHOST-IDENTITY-001). The row
still wastes seed scans and can re-infect a future UI path that forgets the
gate.

## Acceptance criteria

1. Ledger rows matching GhostIdentityGate criteria are retired/pruned on a
   bounded schedule (e.g. on load + daily), or marked `retired` and excluded
   from peer-list exchange / seed_dial / topic auto-negotiate.
2. Unit test reproduces the 577fd171 row shape and asserts prune/retire.
3. No regression: proven peers (success_count > 0) never pruned.
4. Harness verify of the claim (free tier) + Windows gates.
5. Passive log evidence: post-fix phone ledger has 0 ghost rows (or retired
   flag); `GHOST-IDENTITY-001` still present as defense-in-depth.

## Non-goals

- Do not `pm clear` to “clean” the phone.
- Do not delete Windows/AWS contact history without operator approval.

## Evidence

- `tmp/cto/LOGPULL_LOADPEERS_20260911T060527Z/phone_ledger.json`
- loadPeers PASS: 2 unified, 0×577fd171 in logcat after UI
