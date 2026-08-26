# Operator Decisions — v0.4.0-alpha.1 Tag Blockers (2026-08-25)

**Date:** Tuesday, 2026-08-25 12:52 UTC-10  
**Decided by:** Operator  
**Authority:** AGENTS.md rules 9 (architecture escalation), SHIP_PLAN.md (tag-gating decisions)  
**Status:** LOCKED IN — ready to unblock downstream work

---

## DECISION 1: PR #139 Merge/Close (204 commits, crypto/transport)

### Ruling
**MERGE** — Proceed with full merge after mandatory adversarial security review (read-only dispatch).

### Rationale
- 204 commits represent a ton of valid work
- Consolidating into main avoids rebase risk and context loss
- Security review is non-blocking for merge timeline; it runs in parallel
- Cherry-pick alternative introduces human error risk and fragmentation

### Execution
1. **Immediate:** File `HANDOFF/review/PR139_SECURITY_REVIEW_2026-08-25.md` with read-only ticket (no implementation barrier)
2. **Dispatch:** Assign to qualified adversarial reviewer (crypto-security-auditor per `.claude/rules/security.md`)
3. **Timeline:** 2-3 LOC review work; review runs parallel to other pre-tag tasks (non-blocking for merge)
4. **Gate:** Verdict must be on file before v0.4.0-alpha.1 tag; merge can happen before verdict if CI passes and verdict is favorable
5. **Risk:** If verdict has critical findings, loop back to implementer; but merge can proceed for non-blocking issues

### Unblocks
- 204 commits of delivery/crypto/transport work flowing to main
- Eliminates long-lived integration branch (D5 requirement)

---

## DECISION 2: P0_DUAL_BIND_TCP_AND_WS_ON_SAME_PORT (Port Binding Strategy)

### Ruling
**OPTION B — Bind one transport per port (implicit/bind-what-works).**

### Rationale
- Already selected implicitly by prior work (decision made before this formalization)
- Multi-port strategy already exists or is needed regardless for port-agnostic environment adaptation
- Environment-agnostic binding requirement: willing and able to use any ports available in a given network to connect
- This is the foundation for port negotiation; capability exists or is in-flight regardless of this tag

### Technical Details
- **Current state:** `core/src/transport/multiport.rs:73-80` attempts to bind both `/ip4/0.0.0.0/tcp/{port}` and `/ip4/0.0.0.0/tcp/{port}/ws` on the same port
- **Problem:** Only one succeeds; phone dialling the unbound one fails negotiation
- **Solution:** Advertise only what successfully binds; drop failed bind silently (with warning log)
- **Multi-port capability:** Existing ladder (443, 80, 8080, 0/ephemeral) in `multiport.rs`; extend as needed

### Implementation
- Check bind result; advertise only successful multiaddr
- No port-splitting required; implicit fallback handles environment variations
- Wiring to full multi-port strategy happens in parallel (non-blocking for tag)

### Unblocks
- D4 two-device message proof (current port conflicts resolved)
- Port-agnostic transport negotiation

---

## DECISION 3: P0_DEEPLINK_PARSES_BUT_NEVER_DIALS (Bootstrap UX)

### Ruling
**OPTION I — Expand D4 scope. Implement full JoinMeshScreen restoration + auto-dial wiring as pre-tag task.**

### Rationale
- Deeplinks are critical for off-LAN bootstrap (user gets link, clicks, connects)
- Minimal implementation (option ii) leaves feature half-done and user-facing UX incomplete
- Full restoration (option iii) is necessary to complete the capability
- Bundling with D4 ensures bootstrap is tested end-to-end before tag

### Technical Details
- **Current state:** `MainViewModel.kt:361-363` parses deeplinks but no auto-dial wired
- **Deleted component:** `JoinMeshScreen` UI was removed; need to restore from git history
- **Scope expansion:** Add to D4 pre-tag work (two-device north-star proof)
- **Deliverable:** Full bootstrap flow (user sees "join mesh" UI, confirms, auto-dials peer, receives message)

### Implementation
1. Restore `JoinMeshScreen` route + Composable from git history
2. Register navRoute in `MainTabView` / `NavHost`
3. Wire deeplink parse → navigation to JoinMeshScreen
4. Wire button action → auto-dial on peer ID
5. Test: Click deeplink → UI appears → confirm → dial → message received

### UX Details
- User-facing confirmation UI (not silent auto-dial)
- Consistent with "mesh join" mental model
- Completes north-star scenario: "user gets link, clicks it, messages flow"

### Unblocks
- D4 north-star proof (two-device, cross-network, bootstrap via deeplink)
- Off-LAN rendezvous capability
- Complete bootstrap story for v0.4.0-alpha.1

---

## Summary: What Unblocks Now

| Task | Blocker Resolved | Next Action |
|---|---|---|
| PR #139 merge flow | DECISION locked | File security review ticket; dispatch to auditor (read-only) |
| Port binding implementation | DECISION locked | Confirm `multiport.rs` bind-result logic; advertise only what succeeds |
| D4 bootstrap proof | DECISION locked + scope defined | Restore `JoinMeshScreen` + wire deeplink → dial flow; add to pre-tag sprint |
| v0.4.0-alpha.1 tag readiness | All three blockers cleared | Proceed with S2 (README + signing) and S3 (north-star proof) |

---

## Operator Sign-Off

**Decided:** 2026-08-25 12:52 UTC-10  
**By:** Operator (via chat)  
**Decisions:** All three locked in and ready to execute

---

**Next step:** Update SHIP_PLAN.md and dispatch S0-3 (PR #139 security review) + S3-2 (D4 bootstrap wiring) to queue.
