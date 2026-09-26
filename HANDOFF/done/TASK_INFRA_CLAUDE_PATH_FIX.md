# TASK: Fix Claude Code Executable Path in Environment

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Status:** TODO
**Priority:** P2 (Infra / convenience)
**Target:** Windows environment PATH variable

## Problem
The user reported that `claude` (Claude Code) is not in the system `PATH`. They currently have to target it directly at `C:\Users\SCM\.local\bin\claude.exe` or launch it via settings:
```powershell
C:\Users\SCM\.local\bin\claude.exe --settings C:\Users\SCM\.claude\settings.local.OR.json --continue --dangerously-skip-permissions
```

This task is to:
1. Determine why `C:\Users\SCM\.local\bin` is missing from the system or user `PATH`.
2. Permanently add it to the user's `PATH` environment variable using `setx` or PowerShell environment modifiers, ensuring it persists.
3. Validate that `claude` can be run globally from a fresh terminal session.

## Steps to Execute
* Open a PowerShell session.
* Read the current user `PATH` variable:
  ```powershell
  [Environment]::GetEnvironmentVariable("Path", "User")
  ```
* Check if `C:\Users\SCM\.local\bin` is indeed missing.
* Append the directory to the User `PATH` (ensuring no duplicates and not overwriting the existing path):
  ```powershell
  $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
  if ($currentPath -notlike "*C:\Users\SCM\.local\bin*") {
      $newPath = $currentPath + ";C:\Users\SCM\.local\bin"
      [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
      echo "[OK] Added C:\Users\SCM\.local\bin to User PATH."
  } else {
      echo "[INFO] Already in User PATH."
  }
  ```
* Verify it works. Note: newly spawned shell processes will inherit the updated `PATH`.
