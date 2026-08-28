# SCMessenger Philosophy Canon

Status: Draft from repository context, pending owner confirmation.
Last updated: 2026-02-24

## Mission and product intent
- Build a sovereign, end-to-end encrypted messenger where users own identity and communication without centralized account dependency.
- Deliver one dependable product across Android, iOS, and Web with aligned critical-path behavior.
- Support global organic adoption through community-operated infrastructure.

## User promises and trust boundaries
- Do not require phone number, email, or centralized account for core messaging identity.
- Keep cryptographic authority in Rust core; platform adapters must not redefine protocol guarantees.
- Require first-run consent that explains security and privacy boundaries before first use.

## Engineering principles
- Keep core protocol/security logic centralized in Rust core with shared bindings.
- Preserve identity canonicalization around `public_key_hex`; treat `identity_id` and `libp2p_peer_id` as derived metadata.
- Enforce deterministic, testable behavior across Android, iOS, and Web for critical controls.
- Prioritize parity and correctness over platform-specific divergence.

## UX and interaction principles
- Keep relay semantics user-understandable and consistent: relay ON enables messaging; relay OFF blocks inbound/outbound relay messaging while local history remains readable offline.
- Keep mental model and critical controls aligned across Android, iOS, and Web.
- Use English-only for alpha while preserving an i18n path for post-alpha expansion.

## Security and privacy posture
- Maintain current crypto baseline: Ed25519 identity/signing, X25519 ECDH, XChaCha20-Poly1305, Blake3 derivation/hash.
- Preserve end-to-end boundaries; avoid plaintext persistence outside approved boundaries.
- Treat security posture as non-negotiable on identity, cryptography, and trust boundary behavior.

## Reliability and operability posture
- Keep app available during active sessions when open/relaying.
- Enforce durable store-and-forward so messages are not dropped under transient connectivity loss.
- Treat delivery, reconnect, and active-session availability as eventual-consistency targets that converge to 100% over a sufficient time horizon.
- Use measurable gates and CI-backed parity checks as readiness criteria.

## Performance posture
- Target practical global usability across mixed network conditions (regional/international, mixed NAT/firewall environments).
- Accept bounded complexity to preserve reliability and trust guarantees.

## Explicit anti-goals
- Do not optimize for region-gated rollout sequencing.
- Do not treat abuse controls/compliance mapping as alpha-blocking gates (track post-alpha hardening).
- Do not allow unbounded local storage growth.
- Do not allow platform-specific protocol drift.

## Prioritized tradeoff matrix
1. User trust and security boundaries
2. Reliable delivery and durability
3. Cross-platform interoperability and parity consistency
4. Performance optimization
5. Delivery velocity

## Decision examples
### Accepted examples
- Choosing `public_key_hex` as canonical cross-platform identity.
- Requiring relay ON/OFF semantics to be identical across clients.
- Requiring consent gate and bounded retention for alpha.

### Rejected examples
- Allowing Web parity to remain optional before GA.
- Allowing platform-specific behavior for core privacy/relay semantics.
- Using static-only bootstrap configuration without dynamic resolution strategy.

## Enforceable rules

| Rule ID | Rule text | Scope | Criticality | Verification | Exception path |
|---|---|---|---|---|---|
| PHIL-001 | Use `public_key_hex` as canonical persisted/exchange identity across Android, iOS, and Web. | code, docs, UX | non-negotiable | API/schema checks, cross-platform interop tests, doc review | owner-approved exception with migration plan and rollback |
| PHIL-002 | Keep relay semantics identical across clients: ON enables relay messaging; OFF blocks inbound and outbound relay messaging while local history remains readable offline. | code, UX, tests | non-negotiable | parity tests and UX flow validation | owner-approved temporary exception with explicit risk window |
| PHIL-003 | Keep protocol and cryptographic authority in Rust core; platform adapters do not redefine cryptographic behavior. | architecture, code | non-negotiable | architecture/code review | owner-approved exception with security review |
| PHIL-004 | Enforce first-run consent gate that communicates security/privacy boundaries before first use. | UX, code | non-negotiable | UI flow tests and release checklist | owner-approved exception not permitted for GA |
| PHIL-005 | Maintain bounded retention policy to prevent unbounded local growth. | code, settings, ops | non-negotiable | storage policy tests and config inspection | owner-approved temporary exception with cleanup deadline |
| PHIL-006 | Preserve tri-platform interoperability and parity (Android, iOS, Web) on critical-path identity, send/receive, history, and settings before GA. | roadmap, CI, code | non-negotiable | CI parity gates and release criteria | no waiver; interoperability is non-waivable |
| PHIL-007 | Support global organic rollout with community-operated infrastructure (self-hosted and third-party nodes valid). | roadmap, docs, ops | negotiable | rollout docs and config behavior | owner-approved alternate go-to-market rationale |
| PHIL-009 | Anti-abuse controls are non-blocking in alpha and become required before beta release. | roadmap, security, ops | non-negotiable | release checklist and gating tests | owner-approved schedule shift only with explicit risk acceptance |
| PHIL-008 | Keep alpha scope English-only while preserving i18n scaffolding. | UX, roadmap | negotiable | localization config and backlog checks | owner-approved locale expansion plan |
| PHIL-010 | Keep critical UX controls and semantics in parity across Android, iOS, and Web; do not allow a temporary lead platform for critical UX behavior. | UX, roadmap, release | non-negotiable | tri-platform UX parity checklist and release sign-off | no waiver; UX parity is required |

## Open items requiring owner confirmation
- None.
