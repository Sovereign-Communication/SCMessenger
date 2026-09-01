# V040-T12 -- Stop burning the CI queue on superseded and irrelevant runs

Status: OPEN (filed 2026-08-31, operator directive on CI pacing)
Priority: P2 -- costs no correctness, costs a lot of wall-clock for everyone
Lane: Freebuff / DeepSeek V4 Flash
Scope: `.github/workflows/*.yml`. **Read the trap in section 3 before editing a
required check -- getting this wrong makes every PR unmergeable.**

## The measurement

Taken 2026-08-31 with the queue under load:

```
repo-wide, last 60 runs:  completed=43  in_progress=3  queued=14
```

Of the 17 unfinished, **10+ belonged to a single docs-only branch**
(`cto/t2-disk-ruling-2026-08-31`, PR #261 -- markdown only), including `CI` x3
and `Cross` x3 across three different SHAs. A documentation branch was consuming
more CI than the 2,172-line Rust change in PR #262.

Two independent causes, both structural:

```bash
# concurrency groups declared, sampled across the main workflows:
ci.yml: 0   cross.yml: 0   lint.yml: 0   mobile.yml: 0
# path filters: only mobile.yml has any, across 16 workflow files
```

1. **No concurrency groups.** Five pushes to one PR queue five full matrices and
   nothing cancels the superseded ones. They all run to completion against code
   nobody will merge.
2. **No path filters.** A single-markdown-file change runs `Cross`
   (multi-target cross-compilation), the Android matrix, `iOS Build & Test`,
   Docker, and CodeQL. PR #260 changed one `.md` file and ran 27 checks.

## 1. Concurrency groups -- do this first, it is the safe half

Add to every workflow that triggers on `pull_request`:

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

A new push then cancels the superseded run for that workflow on that ref. This
alone would have collapsed the five queued matrices above into one.

**Do NOT set `cancel-in-progress: true` for `push` on `main`.** Main's runs are
the evidence that trunk is green at a given SHA; cancelling one destroys that
record and leaves a commit whose status is permanently unknown. Either omit
concurrency for the main-push trigger, or use a group keyed on the SHA with
`cancel-in-progress: false`.

## 2. Path filters -- the bigger win, the sharper edge

Non-required workflows may skip entirely on docs-only changes. Safe to add
`paths-ignore` for `**.md`, `HANDOFF/**`, `docs/**` to: `Cross`, `iOS Build &
Test`, Docker workflows, and the Android/Mobile matrix.

Judgement required, and state your reasoning in the PR: some `docs/**` paths are
load-bearing. `docs/rules/*.md` changes agent behaviour and
`scripts/docs_sync_check.sh` reads the docs tree, so `Repository Hygiene` and
the docs-sync check must still run on doc changes. Skipping a *build* on a docs
change is safe; skipping a *docs check* on a docs change is absurd.

## 3. THE TRAP -- required checks must never be skipped

Branch protection on `main` requires exactly four checks:

```
Repository Hygiene, Lint, Rust Linting, Test (ubuntu-latest)
```

If a path filter causes a **required** check not to run, GitHub does not treat
it as passed -- the PR sits in `Expected — Waiting for status to be reported`
**forever** and cannot be merged. Adding `paths-ignore` to any of those four
workflows would brick every docs PR in the repo, including the ones fixing the
damage.

So for those four, do **not** use a workflow-level path filter. Instead let the
job start and short-circuit inside it:

- add a first step that determines whether the change is docs-only,
- guard the expensive steps with `if:` on that result,
- let the job still complete successfully in seconds.

The check reports success, protection is satisfied, and no build runs. Verify
this by opening a throwaway docs-only PR and confirming all four required checks
report success without running a build -- do not assume it, GitHub's skip
semantics are exactly the thing people get wrong here.

## Acceptance

1. Every `pull_request`-triggered workflow has a concurrency group with
   `cancel-in-progress: true`; main-push runs are NOT cancel-on-supersede.
2. A docs-only PR runs the four required checks (fast, no builds) and skips
   `Cross`, `iOS`, Docker, and the Android matrix.
3. A Rust change still runs everything it runs today -- verify by pushing a
   trivial whitespace change to a `.rs` file on a scratch branch and confirming
   the full matrix triggers, then close it.
4. Pushing twice in quick succession to one PR leaves exactly one running
   matrix, not two.
5. Report the before/after check count for a docs-only PR in the PR body.
6. Never read `$?` after a pipe.

## Rules that apply to this task

- No emojis. `[OK]`, `[FAIL]`, `[WARNING]`, `[INFO]`.
- Do not change what any check actually asserts. This task changes *when* checks
  run, never *whether they can fail*. A check that stops being able to fail is a
  far worse outcome than a slow queue -- see I-21, where a gate that skipped its
  own comparison reported success.
- Shared checkout: touch only what this task requires.
