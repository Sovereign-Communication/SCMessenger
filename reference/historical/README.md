# Historical Reference Artifacts

Status: Historical
Last updated: 2026-03-02

This folder stores non-canonical operational artifacts that were moved from the repository root during the alpha readiness sanity sweep.

These files are preserved for traceability only and are not part of the active execution/documentation chain.

## Provenance

- Moved from repository root during WS12 alpha readiness closure.
- Original purpose: ad-hoc runtime debugging, one-off log captures, and temporary operator notes.

## Contents

- `parse_connections.py`
- `snapshot_mesh.py`
- `snapshot_mesh2.py`
- `control_android_logs.txt`
- `android_panics.txt`
- `android_crash_logs_buffer.txt`
- `continue_working_on_this.md`

## Usage Rule

- Do not use these files as implementation source of truth.
- If any artifact is needed again, copy relevant details into active docs (`docs/*` or `REMAINING_WORK_TRACKING.md`) instead of editing historical files directly.
