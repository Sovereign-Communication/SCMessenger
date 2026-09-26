# v0.4.0 free-lane dispatch package -- READY TO FIRE

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: RETIRED 2026-07-28 -- all three dispatches (A outbox Site-1, B receipt
round-trip, C ledger choke-point) landed via operator commits f521f142,
8f866bfc, 22b921ca. Kept as the canonical tooling-trap reference
(--files space-separation; repo-root-relative diff paths; exit-3 semantics).
Smoke-tested 2026-07-28 against qwen3.8-max-preview: plumbing verified;
first call rate-limited and auto-rotated (masked downgrade, ledger 429);
rotated worker emitted a no-op diff (vacuous; reverted; retire mark applied
orchestrator-side).
Created: 2026-07-26
Cost note: these dispatch to FREE Qwen/Groq lanes -- zero Anthropic window cost.

## TWO TOOLING TRAPS -- read before dispatching, both cost a wasted run

### Trap 1: `--files` is SPACE-separated, not comma-separated

`delegate_task.py --help` shows `--files [FILES ...]` (argparse `nargs="*"`).

```
WRONG:  --files cli/src/main.rs,core/src/store/outbox.rs
RIGHT:  --files cli/src/main.rs core/src/store/outbox.rs
```

Comma form is parsed as ONE filename, which does not exist, so
`deduce_files.py` drops it and the dispatch runs with ZERO targets. It fails
**silently** -- 0-byte log, no diff, no error. Confirmed 2026-07-26.

### Trap 2: workers emit CRATE-relative diff paths -> `git apply` fails -> vacuous

Observed verbatim on the first real dispatch:

```
[WARN] git apply: error: src/main.rs: No such file or directory
[WARN] diff apply failed; falling back to full-file mode for this task
Warning: No properly formatted code blocks found to apply.
[ROUND 1] Running verification command: cargo test --workspace --no-run -j6
[WARN] verify passed but no changes were ever applied -- vacuous success
EXIT=3
```

The model produced `--- a/src/main.rs` because the prompt named the crate file;
`git apply` runs from the repo root and needs `cli/src/main.rs`. The gate then
PASSED (nothing changed), producing exit 3.

**Mitigation: every prompt file must state explicitly:**
> All diff paths MUST be repository-root-relative, e.g. `a/cli/src/main.rs`,
> NOT crate-relative `a/src/main.rs`. The patch is applied with `git apply`
> from the repository root.

Add that line to all three prompt files before re-dispatching. Also recall
exit-code semantics: `0` = verified (still needs a quality pass), `2` = verify
failed, `3` = vacuous = FAILED.

## Prompt files (written, need the path line above added)

- `tmp/v040-outbox-site1-flush.prompt.md`
- `tmp/v040-receipt-roundtrip.prompt.md`
- `tmp/v040-ledger-choke-point.prompt.md`

## Dispatch commands -- corrected

Fire STRICTLY ONE AT A TIME. Concurrent `--verify` gates risk rlib-lock
corruption on Windows. Check `tasklist | grep -iE "cargo|rustc|gradle|java|ndk"`
before each.

### A -- Outbox Site-1 flush (v0.4.0 blocker: "blocks any real delivery")
```
python scripts/delegate_task.py --task tmp/v040-outbox-site1-flush.prompt.md \
  --provider qwen --model qwen3-coder-plus \
  --files cli/src/main.rs core/src/store/outbox.rs \
  --apply --verify "cargo test --workspace --no-run -j6" --mode diff --max-rounds 3
```

### B -- Receipt round-trip (v0.4.0 blocker: app reports delivered as FAILED)
```
python scripts/delegate_task.py --task tmp/v040-receipt-roundtrip.prompt.md \
  --provider qwen --model qwen3-coder-plus \
  --files core/src/iron_core.rs core/src/message/types.rs cli/src/main.rs \
          cli/src/server.rs \
          android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt \
  --apply --verify "cargo test --workspace --no-run -j6" --mode diff --max-rounds 3
```
Run the Android gate separately afterwards:
`cd android && ./gradlew assembleDebug -x lint --quiet`

### C -- Ledger choke-point refactor (THINK tier; design work)
`core/src/relay/ledger.rs` DOES NOT EXIST -- an earlier version of this command
listed it. The real targets are `core/src/store/ledger_entry.rs` (where
`record_connection` and `exchange_response_entries` live) and
`core/src/transport/addr_filter.rs` (NAT64 / CGNAT predicates).
```
python scripts/delegate_task.py --task tmp/v040-ledger-choke-point.prompt.md \
  --provider qwen --model qwen3-235b-a22b-thinking-2507 \
  --files core/src/store/ledger_entry.rs core/src/transport/addr_filter.rs \
          cli/src/ledger.rs cli/src/main.rs core/src/transport/swarm.rs \
          core/tests/integration_ledger_seeding_hardening.rs \
  --apply --verify "cargo clippy --workspace --all-features -j6 -- -D warnings" \
  --mode diff --max-rounds 3
```
THINK tier is required: per ORCHESTRATION.md lesson 13, FLASH and CODER-flash
cannot do analysis, and this is a refactor with a design decision in it.

## Review checklist for whatever comes back

1. Exit 3 = vacuous = FAILED, regardless of what the gate said.
2. Grep every diff for `simulate|mock|placeholder|in a real implementation`.
3. **Grep for SIBLING call sites.** Three adversarial review rounds on the
   ledger work all failed the same way -- a fix applied to one instance and not
   its equivalent (DNS gate added to `cmd_relay` but not the identical
   `cmd_start`; disclosure filter added to the exchange response but not the
   request). Prompt C requires the worker to list all call sites it found; check
   that it actually did.
4. Record every dispatch: `scripts/lake_route.py --record --lake qwen --model
   <model> --task <id> --result ok|vacuous|error`. The router is blind without it.

## Unresolved design conflict -- needs a decision, not an implementation

v0.4.0 blocker #5 mandates **DNS-name-first** addressing with re-resolution on
IP flip. The security hardening in `9a17b0c4` added **`DnsPolicy::Reject`** for
remote-supplied addresses (a DNS name can resolve to `169.254.169.254`, which is
the SSRF vector). These pull in opposite directions and must be reconciled
deliberately.

Sketch of the likely answer: DNS names are acceptable from LOCAL configuration
(`add_bootstrap`, `SC_BOOTSTRAP_NODES`, user-entered) and must be re-resolved on
dial failure; DNS names from REMOTE peers stay rejected, or are resolved and then
every resolved IP is validated before dialing. That preserves blocker #5's intent
without reopening the SSRF. This is the question worth spending the approved
Fusion Lite budget on -- use a small focused prompt and a panel chosen for
network-protocol reasoning rather than the stock trio.
