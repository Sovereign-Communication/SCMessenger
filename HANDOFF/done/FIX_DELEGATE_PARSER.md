# TASK: Fix delegate_task.py --apply parser bugs

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Fix these bugs in `scripts/delegate_task.py` in the `--apply` block (lines ~108-124):

## Bug 1: os.makedirs("") crashes when filename has no directory component

```python
os.makedirs(os.path.dirname(filename), exist_ok=True)
```

`os.path.dirname("somefile.rs")` returns `""`. `os.makedirs("")` raises WinError 3.

Fix: only call makedirs if dirname is non-empty:
```python
dir_name = os.path.dirname(filename)
if dir_name:
    os.makedirs(dir_name, exist_ok=True)
```

## Bug 2: file content includes the `// filename` comment line

The code writes the full `block` (all lines including the first `// filepath` line) to the file. This leaves a comment at the top of every Rust file.

Fix: write `"\n".join(lines[1:])` instead of `block`.

## Bug 3: filename extraction strips `// ` but doesn't validate it's a real path

After extracting `filename`, validate it ends with a known extension (`.rs`, `.toml`, `.md`, `.py`, `.sh`) before attempting to write. If not a valid path, skip the block with a warning.

Please return the FULL corrected `scripts/delegate_task.py`.
