# P1: Docker Control API Security Hardening & Privilege Dropping (CLI-01)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Status:** OPEN
**Priority:** P1 (v0.4.0 Release Blocker)
**Target Branch:** `feat/v040-multi-transport-store-forward`
**Components:** `docker/entrypoint.sh`, `docker/Dockerfile`, `cli/src/api.rs`
**Reference Audit:** `HANDOFF/audit/SHADOW_AUDIT_V040_V050_ADVERSARIAL_REVIEW_2026-09-16.md`

## Problem Description
In `docker/entrypoint.sh` (line 76), the node is launched with:
```bash
NEW_ARGS+=("--http-bind" "0.0.0.0:9876")
```
In `cli/src/api.rs:1425-1431`, Axum is initialized with permissive CORS `allow_origin(Any)` and zero authentication:
1. `POST /api/shutdown` terminates the daemon process via `std::process::exit(0)`. Any entity able to reach port 9876 (or any webpage visited by an operator via CSRF due to CORS Any) can remotely kill cloud and relay nodes.
2. `POST /api/send` allows unauthenticated callers to forge cryptographically signed messages from the node identity.
3. `GET /api/history` dumps plaintext message logs.
4. `docker/Dockerfile` runs as `root` without a non-root `USER` directive.

## Acceptance Criteria
1. Restrict HTTP API binding in `docker/entrypoint.sh` to loopback (`127.0.0.1:9876`) by default.
2. Require explicit authentication (pre-shared bearer token or local IPC cookie) for non-loopback bindings or sensitive endpoints (`/api/shutdown`, `/api/send`, `/api/history`).
3. Restrict CORS from `Any` to explicit trusted origins or disable CORS by default.
4. Add a non-root system user (`USER scm`) in `docker/Dockerfile`.
