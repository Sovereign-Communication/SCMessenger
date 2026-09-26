<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Triage Decision -- 2026-06-08

**Status:** ready
**Bucket:** pending-dispatch
**Cross-ref:** see `HANDOFF/STATE/2026-06-08_UNIFIED_BACKLOG.md`
**Decided by:** Claude Code (Overseer) sweep

**Rationale:** Ticket is a real remaining work item with no shipped code on the
integration branch. No blocker identified. Ready for `/orchestrate` dispatch on
the next cloud slot allocation. Per Lucas directive 2026-06-08 "I want it all
fixed," this is part of the ~30-ticket remaining backlog.

---
# MODEL: qwen3-coder-next:cloud
# BUDGET: 1200
# token_budget: 12000

# P1_CLI_032_Control_API_Missing_GET_Contacts_Endpoint

**Status:** VERIFIED REMAINING WORK (driven by Claude Code 2026-06-04, /api/* route enumeration)
**Agent:** rust-coder
**Budget:** 1200s (MIXED tier)
**Phase:** v0.2.1 P1  API completeness
**Source:** Live enumeration of `cli/src/api.rs:908-924` route table
**Depends on:** P0_BUILD_001, P0_CLI_023

---

## Verified Gap (with reproduction)

The control API at `127.0.0.1:9876` exposes:

```
POST   /api/send
POST   /api/contacts             only POST (add); no GET (list)
GET    /api/peers
GET    /api/swarm/stats
GET    /api/listeners
POST   /api/history
GET    /api/external-address
GET    /api/connection-path-state
GET    /api/diagnostics
GET    /api/drift-status
GET    /api/discovery/status
POST   /api/discovery/scan
GET    /api/discovery/peers
POST   /api/shutdown
```

There is no `GET /api/contacts`. The CLI subcommand `contact list` exists (`cli/src/cli.rs:36`)
but it talks to the same sled backend the API uses, just via the local DB. The
in-process handle_add_contact handler adds a contact but provides no way to read them
back from the API. This forced my entire drive-and-test session to use the buggy
"add then send and pray" workflow because there was no way to verify the contact
was actually persisted (or to enumerate which contacts exist).

The fix in `P0_CLI_023` (shared backend key collision) will make this visible, but a
proper `GET /api/contacts` is needed for observability and integration.

## Scope (~60 LoC across 2 files)

### Part A: Add `GET /api/contacts` (LOC: ~40)

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

Add to the route table:
```rust
.route("/api/contacts", get(handle_get_contacts).post(handle_add_contact))
```

### Part B: Add a `DELETE /api/contacts/:peer_id` (LOC: ~20)

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

## File Targets

- `cli/src/api.rs` [EDIT  add `handle_get_contacts`, `handle_remove_contact`, route entries,
  request/response types]

## Build Verification Commands

```bash
cargo check -p scmessenger-cli
cargo test -p scmessenger-cli
```

## Acceptance Gates

1. After `POST /api/contacts {"peer_id":"X",}`, `GET /api/contacts` returns
   `[{"peer_id":"X",}]` (only after P0_CLI_023 is fixed)
2. `DELETE /api/contacts/X` then `GET /api/contacts` returns `[]`
3. New test in `cli/src/api.rs` (cfg(test) mod) covers the round-trip
4. The CORS layer's allowed methods list is updated to include `DELETE` (in
   `cli/src/api.rs:901-904`)

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: RUST] [REQUIRES: QWEN_CODER_NEXT] [DEPENDS_ON: P0_BUILD_001, P0_CLI_023]
