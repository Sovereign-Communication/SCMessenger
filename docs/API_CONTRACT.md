# API_CONTRACT.md

Owner decision 2026-09-27 (option B): for `/api/send`, `success` means
"the server durably accepted responsibility for this message," not "the
message was delivered."

## Lane rule

Any change to client-visible API behavior must cite this document and list
the alternatives considered in the PR body. Semantic changes require owner
review before merge. Do not invent new response semantics in a lane — if
the contract doesn't cover your case, propose an amendment first.

## Envelope conventions

- `success: true` — the server accepted the request and took responsibility
  for it. It does NOT mean the underlying action fully completed.
- `success: false` — the server did NOT take responsibility. The client must
  retry or surface the failure; nothing is queued.
- `error` — present only when `success: false`. Never populated alongside
  `success: true`; use `warning` for non-fatal notes on successful responses.
- `warning` (optional) — human-readable note on a `success: true` response
  (e.g. "queued for retry"). Never present alongside `success: false`.
- HTTP status codes follow standard semantics: `202` means "accepted for
  processing." A `success: false` body on a 2xx status is a contradiction —
  do not produce one.

## POST /api/send

Accepts a message for delivery. The message is persisted to the history
store (and the outbox while the event loop is alive) before dispatch is
attempted.

| HTTP | success | status | Meaning |
|---|---|---|---|
| 200 | true | `"accepted"` | Transport confirmed delivery (swarm ACK or BLE fallback OK) |
| 202 | true | `"retrying"` | Dispatch failed; message is queued for retry. `warning` carries the detail; no `error` field |
| 503 | false | `"rejected"` | Not accepted by any transport and the event loop is down. Client must retry; `error` carries the reason |

- `message_id` is present on 200/202 and on the 503 envelope (the history
  record exists even when nothing will retry it). Use it to poll delivery.
- Delivery confirmation is via `GET /api/send/:message_id` or delivery
  receipts — never via the `success` flag.
- A client must NOT treat `success: true` + `status: "retrying"` as terminal
  failure, and must NOT treat `success: false` as "queued."

## GET /api/send/:message_id

- `200` + `{message_id, status, delivered, peer_id, timestamp}`.
- `status` is `"delivered"` or `"pending"`. `delivered` mirrors it as a bool.
- `404` `"Message not found"` — unknown ID. `500` — store read failure.

## Changing this contract

Field renames, new status values, or changes to what `success` means are
breaking changes: they require owner sign-off and a note in the PR body
describing client impact. Additive optional fields are non-breaking and may
be added by a lane with a one-line justification.
