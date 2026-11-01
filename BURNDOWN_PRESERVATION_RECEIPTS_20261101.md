# Workspace preservation receipt — 2026-11-01

Scope: SCMessenger canonical checkout `C:\Users\SCM\Documents\GitHub\SCMessenger`.
Owner dispatch: #602 comment 6565239719 and #595 comment 6565239439.

## Preservation results

- `scratch-classify-archive/` contains all 10 `tmp/scratch-classify*` directories: 305 files and 232,137 logical bytes. The `manifest.json` records every relative path, byte count, and SHA-256. A full source-to-copy path, size, and hash comparison found zero missing, extra, or changed files. Originals remain in `tmp/`.
- The dispatch estimate of approximately 3.3 GB was not reproduced by the canonical checkout inventory. No reparse points were found in the 10 directories. `scratch-classify-0` has configured remotes and 17 status entries, rather than no remotes and 15 dirty entries as stated in the dispatch; the other nine have no configured remotes. No source repository or remote was changed. The source inventory mismatch is recorded without widening the search or changing any remote.
- `claude-config-archive/` contains 91 `.claude/` files plus `CLAUDE.md`: 92 files and 545,171 logical bytes. The manifest includes paths, byte counts, and SHA-256 hashes; the full source-to-copy comparison passed. The two credential files `.claude/alibaba_cloud_config.env` and `.claude/alibaba_cloud_config.env.bak-20260927T111450Z` were excluded. Their values were not read or copied.
- `zai-orphans/` contains loose copies of `U1` and `W2-T1`: 2,721 files and 42,093,780 logical bytes. The manifest includes every relative path, byte count, and SHA-256. The full source-to-copy comparison found zero missing, extra, or changed files. Both original directories at `C:\Users\SCM\Documents\GitHub\.zai-worktrees\` remain intact.
- The rescue worktree `C:\Users\SCM\Documents\GitHub\scm-train-state\scratch\SCMessenger` remains unchanged at `43702faa9e3005dc2eee1609053648066acc432b`. Its sole untracked item is `.claude/alibaba_cloud_config.env.bak-20260927T111450Z` (336 bytes); classified as a credential by name only, not read, copied, committed, or pushed.
- `.cloud-attachments/` remains untouched and excluded from staging; its inventory was two files / 554,490 bytes.

## Accounted storage

The three copies contain 42,871,088 logical source bytes plus 833,926 bytes of manifests: 43,705,014 bytes total, approximately 0.044 GB decimal. No source was deleted, so GB freed is 0. `Get-PSDrive` reported C: free space as 16,824,475,648 bytes before copying and 16,727,597,056 bytes after copying; these are point-in-time shared-disk readings and the difference is not attributed solely to this operation.

## Local archive manifests

- `scratch-classify-archive/manifest.json`
- `claude-config-archive/manifest.json`
- `zai-orphans/manifest.json`

The archive directories and manifests are local preservation copies. They are not staged for the lane branch. The originals remain available pending the owner's confirmation.
