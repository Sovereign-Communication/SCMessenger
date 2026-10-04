# Morph Lite Handoff: Orchestrator Integration Guide

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Audience:** `/scmorc`, `/scmqwen`, and fork-mode orchestrators  
**Effective:** 2026-07-14  
**Status:** ACTIVE

---

## Quick Reference

**What:** Lightweight, cost-capped code transformation using Morph V3 Fast (OpenRouter)  
**When:** Single-file, < 500-line edits; fast verification or apply  
**Cost:** $0.001 hard ceiling per call  
**Lane:** Between free Qwen/Gemini and Claude workers (verification tier)

---

## When to Dispatch Morph Lite

###  Good Candidates

- Single-file fix under 500 lines
- Verify a delivery-logic change (outbox, receipt, custody) before commit
- Apply a snippet-based edit (config, const, small function)
- Reconcile a simple merge conflict
- Dry-run (`--verify-only`) to preview a change before applying

###  Escalate Instead

| Condition | Route To |
|---|---|
| Multi-file refactor | Qwen [THINK] or Claude Sonnet |
| Architectural redesign | Claude Opus |
| Crypto/transport security audit | Claude Fable + adversarial review |
| Complex protocol change | Sonnet (high) or Opus |
| Custom model fine-tuning | Custom provider |

---

## Setup (One-Time)

### 1. Ensure OpenRouter API Key

```bash
# On orchestrator host
export OPENROUTER_API_KEY="sk-or-v1-..."
```

Verify access:
```bash
curl -s "https://api.openrouter.ai/api/v1/models" \
  -H "Authorization: Bearer $OPENROUTER_API_KEY" | grep -i morph
```

### 2. Script is Ready

The function lives at: `scripts/morph_lite.py`  
Already deployed with this commit.

---

## Usage: Orchestrator Calling Pattern

### Pattern A: Verify Only (Dry Run)

```bash
python scripts/morph_lite.py \
  --file <target-path> \
  --instruction "<change-description>" \
  --edit-snippet "<desired-code-snippet>" \
  --verify-only
```

**Output:**
```
[RESULT] PASS | cost=$0.00045 | tokens_in=523 tokens_out=412
[DRY-RUN] Transformation ready (not applied)
--- PROPOSED ---
fn flush_on_connect(&mut self, limit: u32) {
  for peer in self.peers().take(limit) {
    self.persist_msg(peer)?;
  }
}
--- END PREVIEW ---
```

**Action:** Orchestrator manually reviews, then either:
1. Approves and applies via `Edit` tool
2. Rejects and escalates to Qwen/Claude
3. Iterates with refined instruction, re-runs Morph Lite

### Pattern B: Apply Directly

```bash
python scripts/morph_lite.py \
  --file <target-path> \
  --instruction "<change-description>" \
  --edit-snippet "<desired-code-snippet>"
```

**Output (Success):**
```
[RESULT] PASS | cost=$0.00045 | tokens_in=523 tokens_out=412
[OK] Applied to core/src/store/outbox.rs
```

**Action:** Orchestrator verifies, commits, moves task to `HANDOFF/done/`.

**Output (Cost Exceeded):**
```
[RESULT] FAIL | cost=$0.0012 | tokens_in=8192 tokens_out=0
[ERROR] Cost $0.0012 exceeds limit $0.001
```

**Action:** Re-run with `--verify-only`, or escalate to Qwen/Claude.

### Pattern C: Farm WS-A Triangulation (Delivery Logic)

**Requirement (FARM_FINAL_PLAN.md):**  
Before committing to outbox/receipt/custody/retry, triangulate via ONE of:

**Option 1: Morph Lite (this)**
```bash
# Verify the fix is sound
python scripts/morph_lite.py \
  --file core/src/store/outbox.rs \
  --instruction "Verify flush_on_connect persists all pending msgs" \
  --edit-snippet "// Check: loop over peers, persist each" \
  --verify-only

# Log to farm ledger: HANDOFF/FARM_SESSION_LEDGER_<date>.md
# Entry: "[Morph Lite] outbox flush verified, cost $0.00045"

# Then commit
git add -A && git commit -m "farm: fixed outbox flush [triangulated via morph_lite]"
```

**Option 2: Qwen [THINK] (3 calls)**
```bash
# Dispatch 3 distinct Qwen [THINK] verifiers
# Each reviews the change independently
# Log results to farm ledger
# Commit if 3/3 pass
```

**Option 3: fusion_lite (multi-model consensus)**
```bash
# See fusion_lite.py docs
# Log result to farm ledger
# Commit if consensus passes
```

Pick ONE option per task, log evidence to `HANDOFF/FARM_SESSION_LEDGER_<date>.md`, then commit.

---

## Orchestrator Integration (scmorc & scmqwen)

### Via `/scmorc` (Claude native)

Add Morph Lite as a dispatch option **before** spawning Claude workers:

```markdown
## The Loop (updated)

...
3. PRE-DISPATCH VALIDATION: ...
4. CHOOSE LANE (canonical ladder):
   - Free lanes first (Qwen, Gemini)
   - **MORPH LITE if:** single-file, <500 lines, <$0.001 estimate
     * `python scripts/morph_lite.py --file ... --verify-only`
   - fusion_lite if: multi-model consensus needed
   - Claude workers if: complex reasoning or audit-gate required
5. LAUNCH & VERIFY: ...
```

Example dispatch:

```bash
# Pre-dispatch validation found a single-file fix
# Estimate: 200 lines, ~1k instruction
python scripts/morph_lite.py \
  --file core/src/store/outbox.rs \
  --instruction "Add retry budget check to flush_on_connect()" \
  --edit-snippet "if self.retry_budget < 10 { return Err(...); }" \
  --verify-only

# Inspect output; if good:
python scripts/morph_lite.py \
  --file core/src/store/outbox.rs \
  --instruction "Add retry budget check to flush_on_connect()" \
  --edit-snippet "if self.retry_budget < 10 { return Err(...); }"

# Log to dispatch_log.md
echo "[2026-07-14 14:30] morph-lite apply core/src/store/outbox.rs result=done cost=\$0.00045" >> tmp/scmorc/dispatch_log.md

# Verify gate
CARGO_INCREMENTAL=0 cargo check --workspace -q

# Commit
git add -A && git commit -m "core: fixed outbox flush [applied via morph_lite]"
```

### Via `/scmqwen` (Qwen free tier)

Morph Lite is **not** a Qwen dispatch (different API endpoint), but orchestrator can route a task to it:

```markdown
## Task Routing

- [FLASH] tier: doc fixes, HANDOFF hygiene
- [CODER] tier: Rust/Kotlin implementation
- **MORPH LITE** if: `<500 lines AND single-file AND task-class=VERIFY`
- [THINK] tier: root-cause analysis, hard refactors
```

Example:

```bash
# Orchestrator reads task file, notes it's a single-file verification
# Instead of dispatching to Qwen, use Morph Lite:
python scripts/morph_lite.py \
  --file android/app/src/main/kotlin/.../MeshService.kt \
  --instruction "Verify BLE interrupt handler is async-safe" \
  --edit-snippet "suspend fun interruptHandler() { ... }" \
  --verify-only

# If verified, apply manually or log for human review
# Commit and move task to done/
```

---

## Logging Format

Add to `tmp/scmorc/dispatch_log.md` or `tmp/scmqwen/dispatch_log.md`:

```markdown
[YYYY-MM-DD HH:MM] morph-lite <action> <file> result=<done|fail|requeued> cost=$<usd>

Example:
[2026-07-14 14:30] morph-lite verify core/src/store/outbox.rs result=done cost=$0.00045
[2026-07-14 14:35] morph-lite apply android/manifest.xml result=done cost=$0.00038
[2026-07-14 15:02] morph-lite verify core/src/routing/ttl.rs result=fail error="cost exceeded" cost=$0.0015
```

---

## Exit Codes

| Code | Meaning | Orchestrator Action |
|---|---|---|
| **0** | Success (applied or verified) | Commit; move task to done/ |
| **1** | Validation/cost gate failed | Re-run with `--verify-only`, escalate, or rewrite instruction |
| **2** | API error or config issue | Check API key, OpenRouter status; retry or escalate |

---

## Constraints & Limits (Enforced in Code)

| Limit | Value | Why |
|---|---|---|
| Max file size | 500 lines | Stays within Morph's fast-apply window |
| Instruction length | 1000 chars | Forces concise change specs |
| Edit snippet length | 2000 chars | Bounded context for snippet-based edits |
| Cost ceiling | $0.001 | Hard gate; prevents runaway spending; operator approval to raise |
| One file per call | 1 | Deterministic; multi-file = multi-dispatch |

---

## Troubleshooting

### "OPENROUTER_API_KEY not set"

```bash
export OPENROUTER_API_KEY="sk-or-v1-..."
# Verify: curl -s "https://api.openrouter.ai/api/v1/models" -H "Authorization: Bearer $OPENROUTER_API_KEY" | head -20
```

### "Cost $0.0012 exceeds limit $0.001"

1. Snippet was too verbose; retry with shorter instruction or smaller snippet
2. File was at upper size limit; split into smaller parts or escalate
3. If acceptable cost, need operator approval to raise `--max-cost` flag

### "File too large (X lines, max 500)"

1. Extract the function/section being changed into a temp file
2. Run Morph Lite on temp file
3. Orchestrator manually merges result back
4. Or escalate to Qwen/Claude for multi-line refactors

### "Morph returned syntax error"

1. Instruction was ambiguous; clarify the change
2. Snippet wasn't valid code; fix and retry
3. Pattern is novel; escalate to Qwen [THINK] or Claude

### "API timeout"

```bash
# Check OpenRouter status
curl -s https://status.openrouter.ai/api/v2/status.json | jq .incidents

# Retry once
# If persistent, escalate to Qwen/Claude
```

---

## Farm WS-A Integration (Delivery-Logic Triangulation)

**Requirement:** Before committing outbox, receipt, custody, or retry changes, triangulate via **one** of:

1. **Morph Lite** (this doc)
   ```bash
   python scripts/morph_lite.py --file <target> --instruction "..." --edit-snippet "..." --verify-only
   ```

2. **Qwen [THINK]** (3 independent verifiers)
   ```bash
   # Dispatch 3 calls to different Qwen models via scmqwen
   ```

3. **fusion_lite** (multi-model consensus)
   ```bash
   python scripts/fusion_lite.py --panel <models> --judge <m> --max-cost 0.01 ...
   ```

**Then log evidence to farm ledger:**
```markdown
## Farm Session Ledger (2026-07-14)

**A1 (outbox flush-on-connect):**
- Morph Lite verification: PASS, cost $0.00045
- Commit: `git commit -m "farm: fixed outbox [triangulated morph_lite]"`
```

---

## Cost Transparency

**Morph V3 Fast pricing (OpenRouter):**
- Input: $0.80 / 1M tokens
- Output: $1.20 / 1M tokens
- Typical call: 1k instruction + 25k file + 10k output ≈ $0.00042

**In dispatch logs:**
```
cost=$0.00045  ← actual cost for this invocation
```

**Budget tracking:**
- Morph Lite is "free lane" by cost (~$0.0005 per call)
- Doesn't burn Anthropic quota (`/scmorc`) or Qwen quota (`/scmqwen`)
- OpenRouter API account charged directly

---

## References

- **Implementation:** `scripts/morph_lite.py`
- **Design spec:** `docs/MORPH_LITE_DESIGN.md`
- **Morph docs:** https://docs.morphllm.com/quickstart
- **OpenRouter:** https://openrouter.ai/morph/morph-v3-fast
- **Orchestration ladder:** `docs/ORCHESTRATION.md` Section 2.1
- **Farm plan:** `HANDOFF/plans/FARM_FINAL_PLAN.md`
