# TASK: Unification audit findings (client-vs-core duplication sweep)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: TODO, mixed confidence. Found 2026-07-12 while investigating the
delivery-receipt bug (`CRITICAL_ANDROID_FALSE_DELIVERY_FAILURE_NO_RECEIPT_ACK.md`).
Dispatched a broader sweep for other instances of the same "client
reimplements/duplicates something that should live once in the shared core"
pattern, via agy (partial, hit a quota wall) and Qwen (multiple tiers, mixed
reliability -- see Tooling notes at the bottom). Findings below are split by
verification status: some were independently confirmed by direct grep/read,
others are AI-generated leads that need a human/agent to check exact
line numbers before acting -- large-file prompts don't include line numbers,
so cited locations are the model's own estimate and were often off by
hundreds of lines even when the underlying finding was real.

## CONFIRMED findings (independently verified via direct grep, 2026-07-12)

### 1. `Outbox::persistent(...)` initialized independently in 3 places
`cli/src/main.rs:1318` (`cmd_start`), `cli/src/main.rs:2478` (`cmd_relay`),
and `cli/src/main.rs:2932` (`cmd_send_offline`) each construct their own
outbox backend/instance rather than sharing one initialization path. Same
risk class as the already-filed outbox-never-flushes bug: any fix to
retry/persistence logic must be applied in 3 places or silently misses one.
Fix direction: a single `Outbox::open_default(data_dir)` helper in
`core/src/store/outbox.rs`, called from all 3 CLI sites (and anywhere else
that constructs one).

### 2. `["sc-lobby", "sc-mesh"]` topic list hardcoded in 2 CLI sites, separately from core
`cli/src/main.rs:1455` and `cli/src/main.rs:2465` both hardcode the literal
topic-name array. `core/src/transport/swarm.rs` ALSO hardcodes these same
strings separately (e.g. `libp2p::gossipsub::IdentTopic::new("sc-mesh")`).
No single shared constant exists anywhere. If the topic naming convention
changes, 3+ independent locations need coordinated edits with no compiler
help. Fix direction: define `pub const TOPIC_LOBBY: &str = "sc-lobby"` /
`TOPIC_MESH: &str = "sc-mesh"` in `core/src/lib.rs` (or a new
`core/src/constants.rs`) and import everywhere.

### 3. Manual exponential-backoff retry loop in `cli/src/main.rs::cmd_send_offline`
Around `cli/src/main.rs:2869` (`let max_retries = 3; ... let delay_ms = 100u64
<< (attempts - 1);`), the CLI hand-rolls its own retry/backoff instead of
using any shared core mechanism -- because the shared core currently HAS no
such mechanism (matches the already-filed finding that `core/src/store/outbox.rs`
has no retry/flush loop at all). Every client that wants retry-with-backoff
today has to invent its own, with its own (possibly inconsistent) parameters.
Fix direction: this is really the same gap as Site 3 in
`CRITICAL_OUTBOX_NEVER_FLUSHES_DESPITE_ACTIVE_CONNECTION.md` -- build the
retry mechanism once in core, delete the CLI's inline version.

## Dead FFI callback: corrected finding

`core/src/api.udl`'s `CoreDelegate.on_message_received(...)` has zero call
sites (confirmed) -- BUT this is NOT a "messages never arrive" bug like the
receipt case. `core/src/transport/swarm.rs` sends incoming messages through
a SEPARATE, actively-used channel: `event_tx.send(SwarmEvent2::MessageReceived
{...})` (native path `swarm.rs:2693`, wasm path `swarm.rs:4998`). This is a
real, working delivery path (confirmed live this session -- Android
correctly displayed incoming test messages throughout today's testing). So
`on_message_received` is genuinely dead code / an unused legacy callback
(worth removing or documenting why it's kept for API-compat), but it is
**not** evidence of a functional bug the way `on_receipt_received` is --
that one has no equivalent alternate path anywhere (confirmed by grepping
all of `core/src/` for `Receipt` usage: only `iron_core.rs:1572` and the
`lib.rs` re-export, no incoming-message classification logic exists at all
in `swarm.rs`).

**Lesson for whoever picks this up:** an AI-reported "dead callback" finding
needs its OWN check for an alternate delivery path before treating it as a
bug -- "the specific function you expected is unused" and "the underlying
capability is broken" are different claims, and this session's first-pass
report conflated them.

## LEADS worth a follow-up look (not independently verified, line numbers unreliable)

These came from the same Qwen passes; the underlying pattern (some form of
duplicated logic) is plausible given everything else found, but exact
locations need re-grepping before acting -- don't trust the cited line
numbers:

- Message envelope payload format inconsistency (raw UTF-8 bytes for text
  vs bincode-serialized struct for receipts) -- may just be restating the
  already-confirmed receipt format war from another angle rather than a
  separate issue.
- `Envelope` V1 vs `EnvelopeV2` version negotiation -- worth checking
  whether PQC-05/06/07 work already this cycle already handles this; may be
  stale/already-addressed.
- Ledger disk-persistence format (`serde_json::to_string_pretty`) vs
  wire-exchange format -- plausible, not checked.
- History record schema versioning (`adjust_legacy_timestamps()` already
  exists, suggesting drift has happened before) -- plausible, not checked.
- Contact-identifier resolution logic (peer ID vs public key vs identity_id
  detection) possibly duplicated between CLI commands and CLI's own UI
  WebSocket handler -- plausible given the Android side has a confirmed,
  separate instance of "peer ID field accepts a pasted public key with no
  format validation" (see `CRITICAL_ANDROID_FALSE_DELIVERY_FAILURE_NO_RECEIPT_ACK.md`'s
  earlier investigation trail) -- worth checking whether CLI's `contact add`
  has the same asymmetric-validation gap.

## Already-fixed, related item

The wasted per-keystroke `meshRepository.searchContacts()` FFI call in
Android's `ContactsViewModel.setSearchQuery` (confirmed root cause of a
"search is slow" complaint: an unindexed full-contact-store scan on every
keystroke, result discarded, never used since the actual list renders from
a separate local filter) was found and fixed in this same investigation
window -- see git history same day, `ContactsViewModel.kt:703-717`.

## Tooling notes (for whoever runs the next audit pass)

- agy hit an "Individual quota reached" wall partway through its own
  parallel-subagent investigation; its partial transcript (salvaged, not
  lost) independently found the `on_receipt_received` dead callback AND a
  sharper 3-way version of the format-war finding (a SEPARATE core function,
  `Message::receipt()` in `core/src/message/types.rs:181`, also
  bincode-serializes a `Receipt`, disagreeing with both `prepare_receipt`'s
  JSON output and the CLI's own bincode-deserialize expectation -- confirmed
  real, not yet independently re-verified against the current line number).
- `scripts/delegate_task.py` + Qwen "thinking" tier (`qwen3-vl-235b-a22b-thinking`)
  returned empty content for a large multi-file prompt -- likely reasoning-token
  exhaustion against the script's `max_tokens: 16000` cap, or a
  reasoning-field-name mismatch (the script's null-content fallback checks
  `message.get("reasoning")`; if this provider names it differently e.g.
  `reasoning_content`, the fallback silently produces `content = ""` with no
  error). Worth a real fix if "thinking" tier gets used again for large-context
  audits.
- Qwen "max" tier (`qwen3-max`) hit a hard 403 "free quota exhausted" wall
  separately from "thinking" tier -- confirms DashScope's free-tier quota is
  tracked per-model, not account-wide.
- Qwen "standard" tier (`qwen3.5-122b-a10b`) worked reliably for this task
  size and is the one that produced the findings above -- prefer it for
  similar large-file analysis passes until "max"/"thinking" quotas reset.
- None of `delegate_task.py`'s prompt templates (`--mode full` or `--mode
  diff`) are meant for pure investigation -- both hard-code "you are a
  senior Rust engineer, implement/return diffs" framing regardless of what
  the task file actually asks for. It happened to still produce a useful
  prose report anyway (the model apparently prioritized the task file's
  actual "read-only investigation" instructions over the wrapper's), but a
  cleaner fix would be a dedicated `--mode analyze` prompt template for
  non-edit tasks.
