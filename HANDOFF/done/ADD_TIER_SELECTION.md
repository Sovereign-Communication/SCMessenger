# TASK: Add tier-based model selection to delegate_task.py

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Add a `--tier` argument to `scripts/delegate_task.py` that auto-selects the correct Qwen model based on task difficulty. When `--provider qwen` is used, `--model` becomes optional if `--tier` is provided.

## Tier Map

```python
QWEN_TIER_MAP = {
    "thinking": "qwen3-vl-235b-a22b-thinking",  # Architecture, security review, adversarial audit
    "max":      "qwen3-max",                      # Complex Rust impl, crypto, multi-file changes
    "standard": "qwen3.5-122b-a10b",              # Compile fixes, mechanical refactors, moderate tasks
    "plus":     "qwen-plus-2025-07-28",           # Docs, task file generation, planning
    "flash":    "qwen-max",                       # Simple fixes, small scoped changes, fallback
}
```

## Changes Required

1. Add `--tier` argument: `choices=["thinking", "max", "standard", "plus", "flash"]`, optional
2. When `--provider qwen` and `--tier` is set: resolve model from `QWEN_TIER_MAP[args.tier]`, ignore `--model` if also passed
3. When `--provider qwen` and neither `--tier` nor `--model` given: default to `"max"` tier
4. Print the resolved model name in the dispatch message: `"Dispatching task X to qwen (qwen3-max [tier: max])..."`
5. `--model` still works as before for openrouter/ollama and for explicit qwen overrides

## Example Usage After Change

```bash
# Compile fix -> standard tier
python scripts/delegate_task.py --task HANDOFF/todo/FIX.md --provider qwen --tier standard --files core/src/crypto/ratchet.rs --apply

# Complex crypto impl -> max tier
python scripts/delegate_task.py --task HANDOFF/todo/PQC_09.md --provider qwen --tier max --files core/src/crypto/encrypt.rs --apply

# Doc rewrite -> plus tier
python scripts/delegate_task.py --task HANDOFF/todo/DOCS.md --provider qwen --tier plus --apply

# Security review -> thinking tier
python scripts/delegate_task.py --task HANDOFF/todo/AUDIT.md --provider qwen --tier thinking --files core/src/crypto/ratchet.rs --apply
```

Return the FULL updated `scripts/delegate_task.py` with `// scripts/delegate_task.py` as the first line of the code block.
