WINDOWS + FREEBUFF 24/7 STRATEGY FOR SCMESSENGER LANES (v4)

Verified facts (from the freebuff npm package v0.2.30 itself, inspected
2026-10-19):
- Freebuff officially supports win32 (x64 + arm64); the launcher fetches
  freebuff.exe for Windows. Install: npm install -g freebuff (Node >=16).
- Freebuff is ad-supported free. Models are paid in HOURS: each model has
  a multiplier per one-hour session (default MiMo v2.6 Flash Fast = 1x;
  a 3x model burns 3 hours of balance per session-hour). Free allowance:
  5 hours/day (limited mode; 4 on VPN/proxy); earned hours accrue to a
  wallet used after the daily allowance.
- Consequence: agent time is metered in one-hour session blocks. The
  supervision layer must therefore cost ZERO agent time, and sessions
  must batch work instead of spawn-per-event.
- Some listed models retain prompts (stealth betas); lane work uses
  named non-retaining models only.

Architecture (three layers):
LAYER 0 - THE MACHINE (Windows settings, one-time):
- Never sleep on AC: powercfg /change standby-timeout-ac 0;
  powercfg /change hibernate-timeout-ac 0. (Laptop lid: set close action
  to 'do nothing'.) BIOS: restore power after AC loss.
- Windows Update active hours set to the full day via Settings (or
  registry UX\Settings ActiveHoursStart/End) so update reboots never
  kill a session mid-work; the role system is stateless by design, so a
  reboot costs at most the current session-hour's tail.
- One-time interactive setup as the lane user: Node LTS, npm i -g
  freebuff, run freebuff once and log in (credentials persist in the
  user profile; all scheduled work runs as that same user).
LAYER 1 - SUPERVISION (Windows-native, zero agent cost):
- Sessions run ONLY in the interactive logon session. The lane box
  stays logged in (auto-login enabled once via Settings > Accounts >
  Sign-in options, or netplwiz). Both the watchdog and any freebuff.exe
  run as the lane user in that session. Task Scheduler 'run whether the
  user is logged on or not' is BANNED for agent processes: it executes
  in session 0 with no desktop, where a TUI agent hangs or fails
  silently. The watchdog task itself is pure PowerShell and may use
  either mode, but keep it in the interactive session for one simple
  rule. Register tasks with the PowerShell cmdlets so settings are
  explicit:
    $a = New-ScheduledTaskAction -Execute 'powershell.exe' -Argument
      '-NoProfile -ExecutionPolicy Bypass -File C:\scm\watchdog.ps1'
    $t = New-ScheduledTaskTrigger -Once -At (Get-Date) -RepetitionInterval (New-TimeSpan -Minutes 10)
    $s = New-ScheduledTaskSettingsSet -MultipleInstances IgnoreNew -ExecutionTimeLimit (New-TimeSpan -Hours 1)
    Register-ScheduledTask -TaskName SCM-Watchdog -Action $a -Trigger $t -Settings $s -User $env:USERNAME
- Spawn rule: freebuff sessions must run NON-INTERACTIVELY from the
  watchdog. The spawn line in watchdog.ps1 (a single variable, set
  after the on-box freebuff --help check) uses the verified one-shot
  invocation with the role prompt. If the box check shows no reliable
  non-interactive mode, use the FALLBACK: at boot the watchdog starts
  ONE persistent interactive freebuff per role in the logged-on
  desktop (Startup-folder shortcut), and 'wakes' it by writing a WAKE
  note in its checkout; the session polls its queue. Same protocol,
  no session-0 involvement.
- Failover on the box is concrete: the watchdog writes a PID file per
  spawned session. Stale heartbeat + process alive = HUNG: close the
  window politely (CloseMainWindow), then Stop-Process -Id; journal
  the kill; the next cycle respawns. Stale heartbeat + no process =
  clean death: just respawn. No process + fresh heartbeat = the session
  self-exited after draining: normal sleep.
- Credential plumbing: the GitHub PAT lives in a user-scope environment
  variable (never echoed in logs); git uses the Windows Credential
  Manager. Per lane checkout: git config core.autocrlf false (keeps
  doc diffs clean) and git pull --ff-only before every state read.
- Clock: run w32tm /resync once; server timestamps remain the
  authority for heartbeat freshness (CONTINUITY.md).
- watchdog.ps1 implements CONTINUITY.md in PowerShell: polls the GitHub
  API (Invoke-RestMethod; PAT in a user environment variable), reads
  the role state doc, applies the wake table, checks heartbeat
  freshness (GitHub server timestamps), coalesces events (max one wake
  per role per cycle), and writes its own run marker that the 6h audit
  cross-checks. It never spawns a second session for a role that has a
  live one (process check + fresh heartbeat).
LAYER 2 - FREEBUFF SESSIONS (metered; batch, don't sprinkle):
- One git checkout per role on the box (C:\scm\landing, C:\scm\impl),
  each holding docs/roles/ from the repo. The watchdog writes a WAKE
  note (trigger + queue snapshot) into the checkout and starts
  freebuff there with the role prompt (docs/roles/prompts/<role>.md).
- Hour-block discipline (the Windows amendment to the sleep protocol):
  a session that drains its queue does NOT exit immediately — exiting
  and respawning starts a new metered hour. It idle-polls its queue
  inside the session until the hour block ends or new work arrives,
  then exits. Sleep still costs zero; the saving is avoiding
  respawn-per-drizzle.
- Daily burn ledger in the state doc: the watchdog records session-
  hours per role per day against a per-role cap. On exhaustion the
  system degrades gracefully: Landing keeps priority on remaining
  hours, non-critical roles wait for the daily reset. The queue slows,
  never corrupts; no surprise bills because there are no bills — the
  free allowance just runs out.
- Budget shape: Landing + Implementer bursts on the 1x default model
  fit the 5 free hours/day on normal days; heavy days draw the wallet.
  The watchdog enforces a daily session-hour cap per role and reports
  burn in the state doc, so cost is visible, never a surprise.

Residual verification (cannot be done off-box; 5-minute checklist):
- freebuff --help on the box: confirm one-shot instruction invocation
  (freebuff "instruction" / stdin) the watchdog's spawn line uses; the
  spawn line is a single variable in watchdog.ps1 for exactly this.
- Confirm a Task Scheduler 'run whether logged on or not' task can
  start freebuff.exe (if not, run the watchdog in the logged-on session
  with auto-login enabled — noted trade-off).
- Confirm login persistence across reboots for the lane user.


Windows realities (v3 additions, learned the hard way by Windows users):
- Toolchain prerequisites on the box (one-time, Layer 0): Git for
  Windows, Node LTS, Rust via rustup, and the MSVC C++ Build Tools
  (link.exe) + protoc — SCMessenger is a Rust workspace; without the
  MSVC linker the lanes cannot build or test locally. Keep lane
  checkouts shallow (C:\scm\...) and set git config core.longpaths
  true in each: Rust target paths blow past the legacy 260-char limit
  otherwise.
- npm shims on Windows are .cmd files. The watchdog's spawn line uses
  the FULL path to freebuff.cmd under the lane user's %APPDATA%\npm
  (Task Scheduler environments do not reliably inherit the interactive
  PATH), and the on-box freebuff --help check confirms whether it is
  invoked directly, via cmd /c, or via Start-Process.
- First-ever freebuff launch is interactive by design (login + the
  binary download + any SmartScreen/Defender prompt are handled once,
  by a human, before any automation depends on it).
- Watchdog task resilience settings: -StartWhenAvailable (runs missed
  cycles after downtime) and RestartOnFailure (RestartCount 3,
  RestartInterval 1 minute) in the same SettingsSet as IgnoreNew —
  a watchdog that dies silently defeats the design, so its task is as
  supervised as the lanes: its run marker is checked by the 6h audit.
- Outbound-only networking: freebuff and git talk out to codebuff.com
  and github.com; no inbound firewall rules are needed or wanted.

Hostile host events (v4 additions — reboots, standby, lock):
- Reboots are assumed, not prevented. Active hours reduce them; they
  do not eliminate forced update reboots. Recovery is structural: an
  AtLogOn trigger runs watchdog.ps1 -Sweep at every logon; the sweep
  treats all leases as suspect, re-derives queue state from GitHub,
  and respawns whatever the queues say is pending. A session killed
  mid-hour loses only its tail — all state lives in the docs, so the
  replacement continues the recorded operation (warm handoff). No
  step requires a human after a reboot.
- Sleep states: check with powercfg /a. Desktops (the preferred lane
  host) have no Modern Standby problem. On a laptop reporting ONLY
  'Standby (S0 Low Power Idle)', classic timeouts may not bite: set
  the PlatformAoAcOverride value (DWORD 0) under
  HKLM\SYSTEM\CurrentControlSet\Control\Power, reboot, and re-check
  powercfg /a — verify on the box before trusting it.
- Lock is fine, logoff is fatal: a locked console session keeps
  processes running; logging off kills them. The lane user never logs
  off. If administering over RDP, disconnect by redirecting the
  session to the console (tscon <session-id> /dest:console, elevated)
  instead of signing out, so the lanes keep their live session.