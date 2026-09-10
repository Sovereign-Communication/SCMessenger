# RCA: MiMo/agent requests fail inside SCMessenger folder

Status: FIXED by neutralizing project `mimocode.json`
Date: 2026-09-10

## Symptom

MiMo Desktop / MiMo Code works in the default workspace, but every agent
request fails when the session's project root is the SCMessenger (or
MiMoSCMessengerFresh) folder.

## Root cause

Project-level `.mimocode/mimocode.json` overrode the host MiMo provider stack:

1. `disabled_providers` included `"xiaomi"` and `"XiaomiMiMo"`, which removes
   the working host provider.
2. Every agent (`build`, `plan`, `compose`, `rust-coder`, `mobile-dev`, …) was
   pinned to OpenRouter free-tier models such as
   `openrouter/nex-agi/nex-n2-pro:free`.
3. Those models require `OPENROUTER_API_KEY`, which is not present in the
   environment. The working host stack uses `MIMO_LLM_BASE_URL` +
   `MIMO_LLM_API_KEY` instead.
4. Result: requests leave MiMo's local LLM proxy and die on OpenRouter 401 /
   missing-key / dead free model — only when that folder is the project root.

## Why it "worked here in MiMo"

Sessions whose project root is not under SCMessenger never load that
`mimocode.json`, so the host defaults remain intact.

## Fix

1. Previous config saved as `mimocode.json.bak-openrouter-override-20260910`
   in the same `.mimocode/` directory (also applied on SCMessenger main and
   MiMoSCMessengerFresh).
2. Active project config neutralized to an empty shell (schema + username
   only). MiMo inherits host defaults.
3. Optional OpenRouter agents can be restored from the `.bak` later only when
   `OPENROUTER_API_KEY` is intentionally provisioned.

## Verification

From this worktree, a smoke request through MiMo host provider should succeed.
Do not re-add `disabled_providers` entries for xiaomi/XiaomiMiMo unless there
is an explicit operator ruling.
