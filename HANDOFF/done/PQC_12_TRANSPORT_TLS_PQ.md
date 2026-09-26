# TASK: PQC-12 — Post-quantum TLS key exchange on project-controlled transport hops

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Read `PQC_00_MASTER_PLAN.md` first. Depends on: none (independent). Wave 4. Min tier: Sonnet (build-system heavy).

## Why

Audit finding F6: transport hop encryption (QUIC TLS 1.3, WSS) uses classical groups; rust-libp2p Noise is X25519-only upstream with no PQ option as of mid-2026. E2E content is protected by PQC-06/07 regardless — this task is metadata hardening where we control both endpoints (our relays/bootstrap infra), using rustls's shipped `X25519MLKEM768` (TLS codepoint 0x11EC, rustls >= 0.23.27).

## Steps

1. Inventory actual TLS stacks in the tree (record findings in this file):
   ```bash
   rg -n "rustls|native-tls" Cargo.lock | head -40
   rg -n "quinn|rustls|tls" core/Cargo.toml cli/Cargo.toml patch/libp2p-quic -l
   ```
   Note: `patch/libp2p-quic/` is a repo-local patched crate — its rustls/quinn versions are what matter for QUIC. Desktop WSS currently uses `tokio-tungstenite` with `native-tls` (core/Cargo.toml desktop target) — native-tls (schannel/openssl) does NOT do ML-KEM; Android WSS already uses `rustls-tls-webpki-roots`.
2. QUIC: if the pinned rustls in the quic path is >= 0.23.27, enable the hybrid group (rustls `prefer-post-quantum` feature flag on the rustls dep, or explicit `crypto_provider` group configuration in the patched crate). If the pinned version is older, record the exact versions and what upgrade would be required — if the upgrade is more than a version bump (API churn in quinn/libp2p-quic), STOP after writing findings and escalate; do not attempt a big-bang upgrade inside this task.
3. WSS desktop: switch `tokio-tungstenite` from `native-tls` to `rustls-tls-webpki-roots` (matching Android) so the hybrid group becomes available; verify relay WSS connections still work in whatever integration/e2e test exercises them (`rg -n "tungstenite|websocket" core/src cli/src --type rust -l`).
4. Interop guard: classical-only peers MUST still connect (hybrid groups are negotiated, not mandatory — assert no config sets ONLY the PQ group).
5. Verification: unit/integration where feasible; at minimum a loopback TLS handshake test asserting the negotiated group is X25519MLKEM768 when both ends are ours, plus existing transport tests green.

## Definition of Done

- [ ] Standard gates PASS (full workspace + wasm32 — wasm uses browser TLS, must be untouched).
- [ ] `cargo test -p scmessenger-core --test integration_nat_reflection --test test_multiport` (transport-adjacent) green.
- [ ] Version inventory + what-was-enabled-where table written into this file.
- [ ] Cargo.lock audit (as PQC-01) for any version bumps.
- [ ] File moved to HANDOFF/done/ + committed.

## Do NOT

- Touch libp2p Noise (upstream problem; app layer already covers content).
- Force PQ-only TLS groups anywhere.
- Perform major libp2p/quinn version upgrades inside this task (escalate instead).
