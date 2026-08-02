# Audit corpus: model provenance correction (2026-08-02)

Correcting the record before anyone reasons further from the existing findings.

## What actually produced the 4,634 findings

Verified in source on branch `audit_system`:

    audit_system/run_dualpass_audit.py:24-25
    LM_STUDIO_URL   = "http://localhost:1234/v1/chat/completions"
    LM_STUDIO_MODEL = "gemma-4-e4b-instruct"

So the audit ran against **Gemma 4 E4B via LM Studio**, not ollama and not
qwen2.5-coder:7b. Per the operator, an EARLIER portion of the run did use
qwen2.5-coder before the switch.

## The corpus has no model provenance

`audit_results.jsonl` schema is:
file, function, line, severity, category, title, description, code_snippet,
suggestion, pass_type, timestamp.

There is **no `model` field**. Findings from qwen2.5-coder and from
Gemma 4 E4B are interleaved with no way to separate them.

Consequence: the false-positive rates previously quoted from a triage pass
(~10% for unwrap findings, ~60% for platform-specific-leak claims) were
measured across the MIXED set. They do not calibrate either model and must not
be used to predict what a different model would produce.

## Why the run cannot resume

`~/.lmstudio` (6.6 GB of local models) was deleted during a disk-space
emergency on 2026-08-02, along with `~/.ollama` (4.36 GB). The audit's backend
is gone. Continuing means either re-downloading the local model or moving the
run to the cloud free tier.

## Recommended fix if the audit continues in any form

Add a `model` field (and ideally a prompt/version hash) to every emitted
finding. Without it a multi-model corpus cannot be triaged by reliability, and
every future quality claim about these findings inherits the same ambiguity.

## Status of the existing 4,634 findings

Still useful as LEADS. Not usable as a calibrated dataset. Any decision to
extend the sweep should weigh that the existing corpus cannot be quality-scored
per model.
