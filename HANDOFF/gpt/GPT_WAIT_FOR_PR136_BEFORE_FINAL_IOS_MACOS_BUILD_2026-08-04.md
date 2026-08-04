# GPT: 5-Node Run 2 Kickoff Protocol (iOS + macOS lane)

**Date:** 2026-08-04
**Priority:** CRITICAL — this is the timing gate for 5-node run 2
**From:** Orchestrator (Windows/Claude)
**Status:** ARMED — waiting on PR #136 CI green + merge

---

## TL;DR

Do NOT do the final iOS/macOS fresh install yet. Wait for PR #136 to go green
and merge to main. The moment it does, rebuild both nodes from post-merge main
and report ready. Details below.

---

## Why you must wait

PR #136 (`fix/identity-canonicalization-steps2-5`) fixes a critical crypto bug:
the sender was encrypting with `identity_id` (a blake3 HASH of the public key)
instead of `public_key_hex` (the real key material). This is the root cause of
run 1's "wrong key" send failures.

The fix is in `core/src/iron_core.rs` and `core/src/store/contacts.rs` — shared
Rust core, exposed to iOS through UniFFI Swift bindings and used directly by the
macOS CLI. **Any iOS or macOS build compiled against pre-merge `main` lacks this
fix and will still fail to decrypt in run 2.** If you already did a fresh
install, treat it as STALE — it must be rebuilt after the merge.

---

## Current PR #136 status

CI caught a real regression that we are actively fixing (a message-request
gating test: a message from an existing contact was being surfaced as a pending
request from a stranger, because the known-contact lookup was keyed on the old
identifier). A fix is in flight. Everything else is green: Repository Hygiene,
Rust Linting, all four Android arch builds, WASM, iOS, FFI Surface Contract,
CodeQL, Docs, Kotlin/Swift/JS linting.

Watch for green + merged:

```bash
gh pr checks 136 --repo Sovereign-Communication/SCMessenger
```

```bash
gh pr view 136 --repo Sovereign-Communication/SCMessenger --json state,mergedAt,mergeCommit
```

---

## Your kickoff sequence (execute when merged)

### 1. Sync to post-merge main

```bash
git fetch origin main && git checkout main && git pull origin main
```

Confirm the fix is actually present before building — do not trust the merge
notification alone:

```bash
grep -n "public_key_hex" core/src/iron_core.rs | head -5
```

The sender_id assignment in `prepare_message_internal` must use
`public_key_hex()`, NOT `identity_id()`.

### 2. Prefer CI artifacts over local builds

Where a green CI job already produced the binary you need, download that artifact
instead of rebuilding locally — it is faster and it is the exact bits CI
verified. Check the run for downloadable artifacts:

```bash
gh run list --repo Sovereign-Communication/SCMessenger --branch main --limit 3
```

```bash
gh run download <run-id> --repo Sovereign-Communication/SCMessenger
```

Build locally only for what CI cannot produce for you (device-signed iOS
installs, or anything needing your local signing identity).

### 3. Fresh install, fresh identity

- Regenerate Swift bindings if your workflow requires it (`gen_swift`).
- Uninstall the old iOS app before installing — do not upgrade in place.
- For macOS CLI, delete stale `identity.json` / `contacts.db` from prior runs so
  the canonicalization migration path is exercised cleanly from zero.

### 4. Verify identity is canonical before declaring ready

Confirm the outbound sender_id is a 64-hex-character public key, not a blake3
hash. This is the single most important check — it is the whole point of the
release.

### 5. Deliver

Write `HANDOFF/gpt/IOS_MACOS_RUN2_READY_2026-08-04.md` with real observed
evidence (actual log lines, actual key values — not claimed ones).

---

## POLICY: hardcoded IPs are EPHEMERAL — never trust one from a doc

**Operator directive, 2026-08-04.** Any IP address written into a doc, script,
config, or test in this repo is to be treated as ephemeral and probably stale.
Always resolve the live address at use time.

This is not hypothetical. The AWS relay was documented everywhere as
`100.56.248.69`. Its actual address today is different, because **no Elastic IP
is attached**, so the public IP changes on every stop/start. The relay was also
fully down (relay and health ports both refusing connections) while docs
described it as healthy.

Consequences for your lane:
- Do NOT hardcode a relay address into any iOS/macOS bootstrap config or test
  script from memory or from an existing doc.
- Get the current relay address from the orchestrator (or from the live AWS
  describe-instances output) at the moment you configure the nodes.
- A rebuild with a stable Elastic IP is in progress; the resulting address will
  be published in `HANDOFF/audit/AWS_RELAY_REBUILD_2026-08-04.md`. Read it fresh
  when you configure bootstrap — do not copy an address out of this file or any
  other doc into a config.

---

## Run 2 node roster

| Node | Owner | Gate |
|---|---|---|
| Windows CLI | Orchestrator | rebuild from post-merge main |
| Android (Pixel 6a) | Orchestrator | fresh install, APK from CI artifact |
| iOS | GPT | fresh install from post-merge main |
| macOS CLI | GPT | fresh build + wiped identity |
| AWS cloud relay | Orchestrator | rebuild in progress, Elastic IP being attached |

All five must be fresh and on post-merge main before run 2 starts.
