<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Triage Decision -- 2026-06-11

**Status:** ready
**Bucket:** pending-dispatch
**Cross-ref:** `STATE/PLAN_VERIFICATION_2026-06-11.md` 1 (CLI completeness)
**Decided by:** Hermes Agent (overseer) post-session audit
**Routing model:** `gemini-3.5-flash:cloud` (Rust axum route + handler  mechanical)
**Rationale:** P1_CLI_032 from the existing backlog. Two new axum handlers, two new request/response types, CORS list update. Pure CRUD on existing `contacts_store_manager()`. ~60 LoC, no algorithm. Flash handles Rust CRUD well. The existing ticket is written for `qwen3-coder-next:cloud` (1200s budget)  this is a smaller scope and Flash can ship in 300s.

---

# MODEL: gemini-3.5-flash:cloud
# BUDGET: 300
# token_budget: 8000

# P1_GEMINI_FLASH_005  CLI: GET + DELETE /api/contacts Endpoints

**Status:** VERIFIED REMAINING WORK
**Agent:** gemini-coder (Gemini 3.5 Flash)
**Budget:** 300s (MICRO tier)
**Phase:** v0.2.1 P1  API completeness
**Source:** `todo/[VALIDATED]_P1_CLI_032_Control_API_Missing_GET_Contacts_Endpoint.md` (re-scoping for Flash tier)
**Depends on:** P0_BUILD_001 (test gate must be green), P0_CLI_023 (shared backend key collision)

---

## Verified Gap

`cli/src/api.rs:908-924` has 13 routes. `POST /api/contacts` exists (add), but no `GET` (list) or `DELETE /api/contacts/:peer_id`. Integration tests must currently use "add then send and pray"  no observability into persisted contacts.

## Scope (~60 LoC across 1 file)

### Part A: Add `GET /api/contacts` handler (LOC: ~40)

In `cli/src/api.rs`:

```rust
#[derive(Serialize)]
struct GetContactsResponse {
    contacts: Vec<ContactEntry>,
}

#[derive(Serialize)]
struct ContactEntry {
    peer_id: String,
    nickname: Option<String>,
    local_nickname: Option<String>,
    public_key: String,
    added_at: u64,
    last_seen: Option<u64>,
}

async fn handle_get_contacts(
    State(ctx): State<Arc<ApiContext>>,
) -> Result<AxumJson<GetContactsResponse>, (StatusCode, String)> {
    let mgr = ctx.core.contacts_store_manager();
    let list = mgr.list().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", e)))?;
    Ok(AxumJson(GetContactsResponse {
        contacts: list.into_iter().map(|c| ContactEntry {
            peer_id: c.peer_id,
            nickname: c.nickname,
            local_nickname: c.local_nickname,
            public_key: c.public_key,
            added_at: c.added_at,
            last_seen: c.last_seen,
        }).collect()
    }))
}
```

### Part B: Add `DELETE /api/contacts/:peer_id` handler (LOC: ~20)

```rust
async fn handle_remove_contact(
    State(ctx): State<Arc<ApiContext>>,
    Path(peer_id): Path<String>,
) -> Result<AxumJson<AddContactResponse>, (StatusCode, String)> {
    ctx.core.contacts_store_manager()
        .remove(peer_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", e)))?;
    Ok(AxumJson(AddContactResponse { success: true, error: None }))
}
```

Add to the route table:
```rust
.route("/api/contacts", get(handle_get_contacts).post(handle_add_contact))
.route("/api/contacts/:peer_id", delete(handle_remove_contact))
```

Update CORS layer (`cli/src/api.rs:901-904`) to include `DELETE` in allowed methods.

## File Targets

- `cli/src/api.rs` [EDIT  2 new handlers, 2 new types, 2 route entries, 1 CORS line, ~60 LoC]

## Build Verification

```bash
cargo check -p scmessenger-cli
cargo test -p scmessenger-cli
# Smoke:
cargo run -p scmessenger-cli -- daemon &
sleep 2
curl -s -X POST http://127.0.0.1:9876/api/contacts -H "Content-Type: application/json" -d '{"peer_id":"test_peer","public_key":"abcd"}'
curl -s http://127.0.0.1:9876/api/contacts | jq .
curl -s -X DELETE http://127.0.0.1:9876/api/contacts/test_peer
curl -s http://127.0.0.1:9876/api/contacts | jq .  # should be []
```

## Acceptance Gates

1. `cargo check -p scmessenger-cli` 0 errors
2. `cargo test -p scmessenger-cli` all pass
3. POST  GET  DELETE  GET round-trip works in smoke test
4. New unit test covers both handlers (cfg(test) mod in `api.rs`)

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: RUST] [REQUIRES: AXUM] [REQUIRES: GEMINI_FLASH] [DEPENDS_ON: P0_BUILD_001, P0_CLI_023] [SERIAL_NEEDED: false] [PRIORITY: 5]
