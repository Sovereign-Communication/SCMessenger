# TurboFieldfare audit workflow

This is the supported local-LLM workflow for the IronCore and first-pass audit.
The runner is `scripts/run_triplepass_turbofieldfare.py` and uses only Python's
standard library.

## What failed in agy session `11331d37-ee3d-4311-80ed-7eb9998bf1b1`

The session did not fail because TurboFieldfare could not review code. The
client orchestration failed:

- The runner was repeatedly rewritten while a live audit was in progress.
- It treated TurboFieldfare as a dual-server pool (`8080` and `8081`) and then
  dispatched several functions, each with three parallel scope requests. This
  created far more concurrent requests than the single model server can admit.
- The server returned repeated HTTP 429 responses. The agy task was then
  cancelled with `context canceled by manage_task`.
- The progress-saving revision re-acquired a non-reentrant `PROGRESS_LOCK`
  while already holding it, so a worker could deadlock after writing results.
- `PARTIAL_JSON` was accepted as a completed result, and the append-only JSONL
  file had no run ID, source hash, or duplicate-safe task key. A result file
  therefore could not prove that every function or scope had been reviewed.
- The extractor used raw brace counting and silently filtered missing target
  paths. It could miss declarations, cut a function at a brace in a string or
  comment, and report a partial target set as complete.

The old files `HANDOFF_AUDIT/TURBO_FIELDFARE_AUDIT_RESULTS.jsonl` and
`turbo_fieldfare_progress.json` are retained as historical failed-run
artifacts. They are not a completion record for the new workflow.

## Run it

Start exactly one local server from the sibling TurboFieldfare checkout. Follow
its process and model checks first:

```bash
cd ../turbo-fieldfare
swift build -c release --product TurboFieldfareServer
.build/release/TurboFieldfareServer \
  --model scratch/gemma4.gturbo \
  --port 8080 \
  --max-context 65536 \
  --queue-limit 1 \
  --generation-timeout 540 \
  --expert-cache-slots 128 \
  --expert-cache-policy lfu \
  --expert-read-concurrency 4 \
  --expert-read-mbps 512 \
  --model-verification trusted-receipt
```

The balanced-I/O profile is intentional. `trusted-receipt` validates the manifest,
file set, sizes, and recorded SHA-256 bindings without re-reading every large
model file at startup or first layer touch. `full-sha256` remains available for
an explicit cold-integrity run, but it is not suitable for a long audit when
SSD read pressure is the limiting resource. The shared 512 MiB/s expert-read
ceiling and four in-flight expert reads keep the GPU fed without letting the
prefill scheduler turn cache misses into multi-gigabyte-per-second bursts; the
128-slot cache holds the model's full 128-expert per-layer working set in RAM
after it has been touched, eliminating repeat expert reads for revisited routes.

In another terminal, create and inspect an IronCore manifest without spending
model tokens:

```bash
cd ../SCMessenger-full
RUN_DIR="HANDOFF_AUDIT/turbofieldfare-audit/iron-core-$(date +%Y%m%d-%H%M%S)"
python3 scripts/run_triplepass_turbofieldfare.py \
  --scope iron-core --run-dir "$RUN_DIR" --manifest-only
```

Then run or resume it:

```bash
python3 scripts/run_triplepass_turbofieldfare.py \
  --scope iron-core --run-dir "$RUN_DIR" --resume \
  --max-tokens 8192 --request-timeout 600
```

The first pass is the curated hotspot set plus the automatically ranked files
with the highest recent Git churn, size, function density, and audit-risk
signals:

```bash
RUN_DIR="HANDOFF_AUDIT/turbofieldfare-audit/first-pass-$(date +%Y%m%d-%H%M%S)"
python3 scripts/run_triplepass_turbofieldfare.py \
  --scope first-pass --top-files 12 --run-dir "$RUN_DIR" --manifest-only
python3 scripts/run_triplepass_turbofieldfare.py \
  --scope first-pass --top-files 12 --run-dir "$RUN_DIR" --resume \
  --max-tokens 8192 --request-timeout 600
```

Use `--file core/src/iron_core.rs` for one explicit file. Use `--new-run` when
you want the script to create a timestamped child directory. A changed source
file or changed manifest is rejected on resume unless `--new-run` or the
deliberate `--force-manifest` option is used.

## What each run produces

- `manifest.json`: immutable target list, function symbols, line ranges, source
  hashes, selected-file ranking, and every required pass task.
- `results.jsonl`: only schema-valid per-unit results. It preserves the three
  original lenses: high-friction, integration, and deployment.
- `rejected.jsonl`: server failures, invalid JSON, and schema failures. These
  never count as CLEAN or complete.
- `progress.json`: atomically updated resumable state. Existing valid result
  rows are recovered if the process stops between writes.
- `coverage.json`: the completion gate. The command exits nonzero when any
  required unit/pass is missing.

Large functions are split into overlapping, line-numbered segments and marked
`PARTIAL` when a segment cannot establish a whole-function CLEAN result. A
per-file synthesis pass is additive: it deduplicates evidence and ranks
remediation but never replaces the function-level records.

The runner intentionally uses one in-flight request. This is the throughput
that matches TurboFieldfare's single model owner and single-prefix cache; more
parallel Python workers only increase queue pressure and 429 risk.

The server context is set to its supported maximum (65,536 tokens). The audit
runner allows up to 8,192 response tokens per action, which accommodates the
largest supplied function segment plus evidence without truncating the JSON
contract. The server timeout is shorter than the client timeout so a stalled
generation is cancelled by the model owner before the client retries or splits
the work.

Before a real run, validate the local logic without a server:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 scripts/run_triplepass_turbofieldfare.py --self-test
```
