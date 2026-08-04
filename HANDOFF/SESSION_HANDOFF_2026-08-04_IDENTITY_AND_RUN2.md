# Session Handoff: Identity Canonicalization + 5-Node Run 2 Readiness

Status: Active
Last updated: 2026-08-04
Branch: `fix/identity-canonicalization-steps2-5` (PR #136, OPEN, CI red)

---

## 1. What this session was for

Get all five nodes fresh and healthy for run 2 (Windows CLI, Android Pixel 6a,
iOS, macOS CLI, AWS cloud relay). Root blocker was identity canonicalization:
senders were encrypting with `identity_id` (a blake3 HASH of the public key)
instead of `public_key_hex` (the real key material), so every message failed to
decrypt with "wrong key".

---

## 2. Landed and verified

### AWS cloud relay: REBUILT and HEALTHY

The relay was DOWN and its address had silently drifted. Both facts contradicted
every doc in the repo.

- Old instance `i-078cb870316683e79` terminated.
- New instance `i-06b37ed4b6976ac56` (t3.micro, AL2023, SG `scm-node-sg`, key
  `scm-node-key`), running `testbotz/scmessenger:latest` via cloud-init.
- **Current address: 34.203.213.35** (NOT `100.56.248.69` from the docs, and not
  the intermediate `54.242.56.150`).
- Independently verified twice, by two different actors:
  `curl http://34.203.213.35:9876/health` returns real `HTTP/1.1 200` with body
  `{"status":"healthy"}`, and TCP connect to 9001 succeeds.

**[WARNING] Elastic IP was DENIED.** `ec2:AllocateAddress` returns
`UnauthorizedOperation` -- explicit deny `DenyElasticIpAllocationBeyondFreeAllowance`
in the `SCMessengerRelayFreeTierOnly` IAM policy. **IP drift is NOT fixed and will
recur on any stop/start.** Operator accepted this risk for run 2 (2026-08-04).
Widening that policy is the permanent fix.

Full detail: `HANDOFF/audit/AWS_RELAY_REBUILD_2026-08-04.md`.

### Hardcoded-IP audit: COMPLETE

4,205 IPv4 literals scanned across 2,736 tracked files.
91 STALE-CRITICAL / 13 STALE-DOC / 869 HISTORICAL / 3,225 BENIGN.

Most important finding: the Rust CLI was correctly de-hardcoded
(`cli/src/bootstrap.rs:26` has an empty `DEFAULT_BOOTSTRAP_NODES`), but the
JavaScript clients never were. `headless/main.js:8` and `ui/app.js:5` still
compile in two literal bootstrap addresses in GCP and Cloudflare ranges -- an
entirely different infrastructure generation, almost certainly dead.

Also: `infra/ec2/alpha-relay-state.json` records the relay TERMINATED on
2026-07-30 with `former_public_ip: 100.56.248.69`, while active runbooks still
dial that address -- and nothing in the codebase reads that file.

Good news: no new mechanism needed. `SC_BOOTSTRAP_NODES`, `scm config bootstrap
add`, and the ledger all already exist.

Full detail: `HANDOFF/audit/HARDCODED_IP_SWEEP_2026-08-04.md`.

### CI hygiene fixes

- `975ce05e` cargo fmt violations in `core/src/store/contacts.rs`.
- `eabf16ee` / `6336f475` CRLF line endings in `scripts/delegate_task.py`
  (Repository Hygiene fails on trailing whitespace; CRLF presents as that).

---

## 3. The CI regression and its fix

CI caught a REAL bug, not a flake. Four jobs (Test ubuntu/macos/windows, macOS
Native Tests) all failed on one test:

```
cli/tests/integration_message_requests.rs:261
test existing_cli_contact_message_is_not_a_pending_request ... FAILED
assertion failed: a message from an existing CLI contact must not be a pending request
```

**Cause.** `ClientIntent::GetPendingMessageRequests` in `cli/src/server.rs` built
its known-contact set from `Contact.peer_id` and tested membership against
`msg.sender_id`. Contacts are stored keyed by `identity_id`; `sender_id` now
carries the public key. The lookup silently misses, so a known contact is
surfaced as a stranger.

The adjacent blocked-peer check had the same flaw, which is worse: **a peer
blocked under one identifier flavor would not be recognized when their message
arrived under the other, so a rejected sender could reappear as a pending
request.** That is a security gate failure, not a cosmetic bug.

**Fix applied** (in `cli/src/server.rs`, `GetPendingMessageRequests`):

1. The contact set now holds BOTH flavors -- `c.peer_id` and `c.public_key`.
2. A helper `derived_identity_id()` computes `hex(blake3(pubkey_bytes))` from a
   32-byte hex public key, returning `None` otherwise. Each inbound message is
   tested against both its `sender_id` and its derived alternate, for the contact
   set AND the blocked set.

This works because the relationship is one-way: you can always derive the
identity_id from a public key, never the reverse. That asymmetry is exactly why
the public key must be canonical.

**[WARNING] NOT YET ADVERSARIALLY REVIEWED.** The repo's security rules
(`.claude/rules/security.md`) require adversarial review for changes to gating
logic. The review agents died on quota exhaustion before running. **Do not merge
PR #136 without that review.** A passing test proves it works; it does not prove
it is safe.

---

## 4. The big open debt: identifier unification audit NOT DONE

This is the most important unfinished item.

**Why it matters.** `public_key_hex` and `identity_id` are BOTH 64-character
lowercase hex strings. They are indistinguishable by length, character set,
regex, or eyeball -- and both are bare `String` in Rust, Kotlin, and Swift.
Neither the compiler nor a code reviewer can catch a mixup. It only surfaces at
runtime as a silently-missed lookup or a "wrong key" decryption failure.

We have now hit this bug class **twice in one PR**: once in the core encryption
path (the original PR #136 fix) and once downstream in the CLI gating path (the
CI regression above). There is no reason to believe those are the only two.
Additional flavors in play: libp2p PeerId (base58), BLE UUID, X25519 derived key.

**An exhaustive audit was designed and launched but produced ZERO output** -- all
seven agents died on the Anthropic session limit. The workflow script is saved
and re-runnable:

```
C:\Users\SCM\.claude\projects\C--Users-SCM-Documents-GitHub-SCMessenger\09cba5c5-aa56-4e2b-b944-47e4c73a6010\workflows\scripts\identifier-unification-audit-wf_0634f975-9c1.js
```

It covers five surfaces (core identity/crypto, core store/iron_core, core
transport/routing/privacy, CLI+WASM rpc, Android/iOS/UniFFI), then builds a
producer-to-consumer matrix, then adversarially refutes each candidate before
confirming, then designs a permanent structural fix.

**Re-run it on FREE LANES, not Claude subagents.** See section 6.

The structural options it was asked to evaluate, for whoever picks this up:
newtype wrappers (`PublicKeyHex` / `IdentityId`) so the compiler enforces
correctness; a single canonical resolution choke point every lookup must pass
through; a wire/storage prefix (`pk:` / `id:`) so the flavors stop being
indistinguishable; debug assertions; a CI lint against raw identifier String
comparison.

---

## 5. Node readiness for run 2

| Node | State | Gate |
|---|---|---|
| AWS cloud relay | **READY**, verified healthy at 34.203.213.35 | none (IP may drift) |
| Android Pixel 6a | Device reachable over wireless ADB (confirmed) | needs post-merge APK |
| Windows CLI | Not rebuilt | needs post-merge build |
| iOS | GPT armed and waiting | needs post-merge rebuild |
| macOS CLI | GPT armed and waiting | needs post-merge rebuild |

GPT's instructions are written and current in
`HANDOFF/gpt/GPT_WAIT_FOR_PR136_BEFORE_FINAL_IOS_MACOS_BUILD_2026-08-04.md`.
It is told to hold until PR #136 merges, then rebuild both nodes from post-merge
main, prefer CI artifacts via `gh run download` over local builds, and never take
a relay address from a doc.

**Everything funnels through one gate: PR #136 going green and merging.**

---

## 6. Lane status and a lesson paid for in quota

**Anthropic session limit was exhausted this session** (resets 1:30am
Pacific/Honolulu). Roughly 1M subagent tokens spent across two workflows; the
identifier audit returned nothing and the fix workflow died at its Design stage.

**Root cause: violated the repo's own free-lanes-first economics.** Claude
subagents were fanned out for bulk code reading -- exactly the mechanical work
that belongs on a free lane. `docs/ORCHESTRATION.md` Section 2.1 puts Claude
native last, for audit gates and judgment calls only. Reading five code surfaces
is not a judgment call.

**Free Qwen tier is also exhausted** -- a dispatch rotated through all nine
models on rate limits before one responded.

Remaining capacity as of session end: paid Qwen (`--provider qwenpaid --model
qwen3.8-max-preview`), Groq, OpenRouter/FusionLite.

**Known tooling trap discovered:** Qwen emits diffs with a Python-style hunk
header (`@@ def handle_jsonrpc_request(`) instead of line numbers
(`@@ -1341,7 +1341,12 @@`). `git apply`, `git apply --3way`, and
`patch --fuzz=5` all reject these as "only garbage was found in the patch
input." The generated CODE is correct; only the framing is broken. Either
post-process the hunk header or transcribe the body manually.

---

## 7. Immediate next actions, in order

1. **Confirm the scoped test passes**:
   `cargo test -j6 -p scmessenger-cli --test integration_message_requests`
   (all FOUR tests, not just the one that was failing).
2. **Adversarial review of the gating fix** -- REQUIRED before merge, per
   `.claude/rules/security.md`. Route to a free lane. Focus: can a blocked peer
   reappear under any identifier flavor; does the accept/reject round trip still
   agree on which flavor it surfaces and consumes.
3. `cargo fmt --all -- --check` and scoped clippy, then commit and push.
4. Watch CI to green, then merge PR #136.
5. Signal GPT (the handoff doc tells it to watch `gh pr view 136`).
6. Rebuild/fresh-install all five nodes from post-merge main.
7. Run the 5-node test.
8. **Then** run the identifier unification audit before trusting any result.

---

## 8. Queued follow-ups (not blocking run 2)

- **Stale infra scripts.** `infra/aws/farm-sim-manage.sh` filters on tag
  `scmessenger-farm-relay` while the real instance is tagged
  `scm-always-on-node`, so `teardown` matches nothing and exits 0 -- a silent
  no-op that reads as success. It also names a nonexistent SSH key.
  `infra/aws/provision-relay.sh` opens ports 443/80 while production uses
  9001/9876.
- **JS bootstrap de-hardcoding** (section 2 above).
- **Elastic IP / DNS** for the relay, gated on widening the IAM policy and on the
  unclosed `/dns4/` filter-bypass finding noted in the IP sweep.
