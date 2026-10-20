#!/usr/bin/env bash
# Spin up an SCMessenger permanent role on Freebuff.
# Usage: spin-up.sh <implementer|landing|shepherd|reviewer|overseer>
# Prints the role's launch prompt to paste into a new Freebuff session.
set -euo pipefail
DIR="$(cd "$(dirname "$0")" && pwd)"

usage() {
  echo "Usage: $0 <role>"
  echo "Roles:"
  echo "  implementer  — writes code, opens PRs, fixes CI (never merges)"
  echo "  landing      — owns the merge train, main's health, tag + publish"
  echo "  shepherd     — Landing's helper: rebase + CI-watch only (never merges)"
  echo "  reviewer     — Rule-8 reviews + the three-node release gate"
  echo "  overseer     — brief for the standing assistant/audit (NOT a Freebuff session)"
}

case "${1:-}" in
  implementer|landing|shepherd|reviewer|overseer)
    echo "=== Paste everything below into a new Freebuff session (role: ${1}) ==="
    echo
    cat "$DIR/${1}.md"
    ;;
  -h|--help|help)
    usage
    ;;
  *)
    usage
    exit 1
    ;;
esac
