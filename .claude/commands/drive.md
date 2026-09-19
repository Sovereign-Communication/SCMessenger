# /DRIVE — run the SCMessenger driver once (on demand)

You are initiating the SCMessenger driver **for this invocation only**. There is
no persistent watcher anymore: the FileSystemWatcher (`SCRATCH/driver/watcher.ps1`),
its Startup-folder shortcut, and the ONLOGON scheduled task were removed
2026-09-04 by CEO request. Nothing runs unless you run it.

## What to do

1. Run the trigger once, bounded and idempotent:

   ```bash
   bash SCRATCH/driver/driver.sh
   ```

   It talks to the local node HTTP API (`127.0.0.1:9876`), scans the node log
   and (if a Pixel is on adb) logcat since the watermark, and writes a memo to
   `SCRATCH/driver/inbox/relay-<ts>.md` **only when something is new**. It may
   auto-draft one short reply to an inbound operator chat via a bounded LLM
   print (primary `claude`, fallback `agy`); that is the only LLM cost.

2. Surface what happened: read any `inbox/relay-*.md` files newer than the
   last run and summarize them (new mesh messages, log anomalies, Android
   logcat anomalies, stuck sends, peer-set change). If the driver printed
   `idle`, there is nothing new — say so in one line.

3. If the node is not running, say so plainly and give the exact start command
   (`scmessenger-cli start` with the correct profile) rather than guessing.

## Guards

- One run per invocation. Do NOT loop, do NOT leave anything running, do NOT
  schedule anything.
- The driver is read-only against the node except for the approved outbox
  drain and the auto-reply path. Do not modify `SCRATCH/driver/state/`
  yourself; let driver.sh own it.
- ASCII output only. `[OK]`/`[INFO]`/`[WARNING]`/`[ERROR]` markers, no emoji.