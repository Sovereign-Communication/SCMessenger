# Harness handoff: make the version and report contract machine-readable (2026-09-19)

From: SCMessenger lane (Buffy / Freebuff). To: harness maintainer.
Type: interface gap found while updating SCMessenger's integration. Read-only
investigation; no harness file was edited for this note.

## What prompted it

SCMessenger does not vendor the harness. `../Harness` is a separate WIP
checkout, upgraded in place, and `SCMessenger/scripts/harness_gate.py` drives
whatever sits there via `python -m harness.cli` with `PYTHONPATH` pointed at the
checkout root. The checkout is currently `0.3.3` (main, `7a84855`, clean tree),
which is well ahead of what our wrapper was written against (2026-09-11), so
the question "which harness produced this evidence?" had no answer inside
SCMessenger. That is now fixed on our side; the two items below are what the
harness could provide so every consumer does not repeat the work.

## Not a defect: the 0.3.3 contract still matches our call sites

Re-read from source at `0.3.3`, so no SCMessenger call sites were rewritten:

- `verify` report carries top-level `verdict`, `consensus` and `actual_cost`
  (`harness/panel.py`, the `result = {...}` assembly), which is exactly what our
  wrapper parses.
- `consensus["agreement"]` is a STRING (`high`/`medium`/`low`/`none`/`unknown`,
  `harness/convergence.py`), not a fraction of panelists.
- Subcommands and flags we pass all still exist: `verify --prompt-file --out
  --max-cost --converge`, `ledger verify`, `spend`, `trust`, `lint-claims
  --claims-file --source-file --out`.
- Paid escalation is still gated by `HARNESS_ALLOW_ESCALATION`
  (`harness/config.py` maps `allow_escalation`).
- Exit codes remain 0 ok / 1 fatal / 2 verify fail / 3 deferred.

So this is a request for an interface, not a bug report.

## Request 1: a `--version` on the CLI

`python -m harness.cli --help` lists only `-h, --help`; there is no `--version`.
The only ways to learn the version are importing the package
(`harness.__version__`) or parsing `pyproject.toml`. A consumer that must decide
whether a checkout is current therefore has to either execute code from the
checkout it is trying to vet, or read a build file. Adding `--version` (and
ideally `--version --json` so it is parseable) closes that.

Related, minor: `harness/__main__.py` does not exist, so `python -m harness`
fails and `python -m harness.cli` is the only module entry. The `harness`
console script covers installed use, so this only bites checkout-based drivers.

## Request 2: state a report/contract version, separate from the package version

`CHANGELOG.md` is thorough about behaviour, but nothing tells a consumer when the
`verify` report shape changes. Our wrapper documents the 0.3.3 shape (`verdict`,
`consensus.agreement` as a string, `actual_cost`) by hand, in a comment; a future
rename would surface as a silently empty field rather than an error, because our
reader uses `.get()`. Either a contract constant in the report
(e.g. `"report_schema": 1`) or a CHANGELOG line marking schema changes would let
consumers fail loudly instead of printing `verdict=None` as if it were a result.

## Latent, not currently wrong: version resolution prefers installed metadata

`harness/__init__.py::_detect_version()` tries `importlib.metadata.version(
"sovereign-harness")` FIRST and only falls back to the checkout's
`pyproject.toml`. Running from a checkout where a previously-installed wheel is
also present can therefore report the *installed* version as if it were the
checkout's. Today there is no mismatch here -- pyproject, `harness.__version__`,
`importlib.metadata` and `sovereign_harness.egg-info/PKG-INFO` all read `0.3.3`
-- so nothing is wrong right now; it is recorded because the failure mode is
silent and this is the one value a consumer uses to decide whether to trust a
checkout.

## What SCMessenger did about it (for reference, in our repo only)

`scripts/harness_gate.py` now reads the version from the checkout's
`pyproject.toml` before executing anything from it, refuses to run below a
`0.3.3` floor, prints the version and floor on every run, and offers a hermetic
`--kind version`. It also honours `SCM_HARNESS_ROOT` so the checkout can live
elsewhere. If `--version` lands upstream, the pyproject read becomes a fallback
and that is a strict improvement -- the note is filed so the change can be made
deliberately rather than rediscovered.
