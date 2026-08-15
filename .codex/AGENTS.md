# Codex adapter

Status: Active
Last updated: 2026-08-15

Read `AGENTS.md` at the repo root -- that file is the canonical contract.
Codex is an adapter, not an alternate authority.

This directory contains:
- `agents/`: role definition TOML files for Codex subagents
- `hooks.json`: hook lifecycle registration
- `hooks/`: session orientation and rule enforcement scripts

All Codex agent roles and hooks must adhere strictly to the universal
contract defined in root `AGENTS.md`.
