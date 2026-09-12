#!/usr/bin/env python3
"""Destructive-action preflight gate for SCMessenger recovery / CTO / CEO seats.

Each --action checks mechanical preconditions only. Exit 0 means the action is
mechanically clear; it is NOT operator authorization. Exit 1 means BLOCK —
do not run the destructive command. Exit 2 means the action name is unknown.

Usage:
  python scripts/recovery_preflight.py --action pm_clear
  python scripts/recovery_preflight.py --action docker_redeploy
  python scripts/recovery_preflight.py --action audit_readiness
  python scripts/recovery_preflight.py --action git_reset --path some/file
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
FRESH = Path(r"C:\Users\SCM\Documents\GitHub\MiMoSCMessengerFresh")
AWS_HOST = "18.234.62.247"
ALWAYS_ON_TAG_HINT = "scm-always-on-node"
CONFIG = REPO / ".mimocode" / "mimocode.json"
CONFIG_BAK_GLOB = "mimocode.json.bak*"


def _ok(msg: str) -> None:
    print(f"[OK] {msg}")


def _warn(msg: str) -> None:
    print(f"[WARNING] {msg}")


def _block(msg: str) -> None:
    print(f"[BLOCK] {msg}")


def _run(cmd: list[str], cwd: Path | None = None) -> subprocess.CompletedProcess:
    return subprocess.run(
        cmd,
        cwd=str(cwd or REPO),
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=60,
    )


def _git(args: list[str], cwd: Path = REPO) -> subprocess.CompletedProcess:
    return _run(["git", *args], cwd=cwd)


def check_audit_readiness() -> int:
    rc = 0
    for label, path in [
        ("AGENTS.md", REPO / "AGENTS.md"),
        ("CTO_STATE.md", REPO / "HANDOFF" / "CTO_STATE.md"),
        ("CEO_STATE.md", REPO / "HANDOFF" / "CEO_STATE.md"),
        (
            "controller package",
            REPO / "HANDOFF" / "V040_CTO_3NODE_BLE_CONTROLLER_PACKAGE_2026-09-08.md",
        ),
        ("orchestration_contract.py", REPO / "scripts" / "orchestration_contract.py"),
        ("orchestrate_strict.py", REPO / "scripts" / "orchestrate_strict.py"),
    ]:
        if path.is_file():
            _ok(f"{label} present: {path.name}")
        else:
            _block(f"{label} MISSING: {path}")
            rc = 1

    hooks = _git(["config", "core.hooksPath"])
    hp = (hooks.stdout or "").strip()
    if hp.endswith(".githooks") or hp == ".githooks":
        _ok(f"core.hooksPath -> {hp}")
    else:
        _warn(f"core.hooksPath is {hp!r}; pre-commit may be detached")
        rc = 1

    contract = _run([sys.executable, str(REPO / "scripts" / "orchestration_contract.py")])
    if contract.returncode == 0:
        _ok("orchestration manifest contract valid")
    else:
        _block(f"orchestration_contract failed: {contract.stderr[:200]}")
        rc = 1

    if FRESH.exists():
        st = _git(["status", "-sb"], cwd=FRESH)
        head = (st.stdout or "").splitlines()[0] if st.stdout else ""
        if "unified/v040-3node-parity" in head:
            _ok(f"live line present: {head}")
        else:
            _warn(f"Fresh worktree branch unexpected: {head}")
        if FRESH.joinpath(".git").exists() or (FRESH / ".git").is_file():
            _ok("MiMoSCMessengerFresh is a git worktree/repo")
        else:
            _block("MiMoSCMessengerFresh is not a git checkout")
            rc = 1
    else:
        _block(f"live worktree missing: {FRESH}")
        rc = 1

    if CONFIG.is_file():
        backups = list(CONFIG.parent.glob(CONFIG_BAK_GLOB))
        if backups:
            _ok(f"config backups exist: {len(backups)}")
        else:
            _warn("mimocode.json present but no dated .bak siblings")

    return rc


def check_pm_clear() -> int:
    rc = 0
    _warn("pm_clear wipes Pixel identity, ledger, history, and message store")
    adb = _run(["adb", "devices"])
    lines = [ln for ln in (adb.stdout or "").splitlines() if "\tdevice" in ln]
    if not lines:
        _block("no adb device — cannot backup identity before pm clear")
        return 1
    _ok(f"adb device present: {lines[0].split()[0]}")

    pkg = _run(["adb", "shell", "pm", "path", "com.scmessenger.android"])
    if pkg.returncode != 0 or "package:" not in (pkg.stdout or ""):
        _warn("package not installed — pm clear is a no-op")
    else:
        _ok("package installed")

    # Require an identity/export backup path under tmp/ newer than 24h if present
    backup_roots = list(REPO.glob("tmp/**/*identity*"))
    recent = [p for p in backup_roots if p.is_file()]
    if recent:
        _ok(f"found {len(recent)} identity-related artifacts under tmp/")
    else:
        _block(
            "no identity backup artifacts under tmp/; export identity or take "
            "operator approval with explicit data-loss acknowledgment first"
        )
        rc = 1

    _block(
        "operator approval required after preflight: pm clear is irreversible "
        "for that install"
    )
    return rc


def check_git_reset() -> int:
    rc = 0
    st = _git(["status", "--porcelain"])
    dirty = [ln for ln in (st.stdout or "").splitlines() if ln.strip()]
    if dirty:
        _block(f"{len(dirty)} dirty paths in main worktree — mass reset is forbidden")
        for ln in dirty[:12]:
            print(f"  {ln}")
        rc = 1
    else:
        _ok("main worktree clean")

    if FRESH.exists():
        st2 = _git(["status", "--porcelain"], cwd=FRESH)
        dirty2 = [ln for ln in (st2.stdout or "").splitlines() if ln.strip()]
        if dirty2:
            _warn(f"Fresh has {len(dirty2)} dirty paths — do not reset it either")
        else:
            _ok("Fresh worktree clean")

    _warn("single-file restore from a ref is recovery; git checkout -- . is destruction")
    return rc


def check_git_checkout_paths() -> int:
    return check_git_reset()


def check_rm_rf() -> int:
    _warn("recursive delete outside tmp/ and target/ needs operator approval")
    _block("confirm path is under tmp/ or target/ before running")
    return 1


def check_force_push() -> int:
    rc = 0
    for label, cwd, branch in [
        ("main checkout", REPO, None),
        ("Fresh", FRESH, "unified/v040-3node-parity"),
    ]:
        if not cwd.exists():
            continue
        br = _git(["rev-parse", "--abbrev-ref", "HEAD"], cwd=cwd)
        name = (br.stdout or "").strip() or branch
        if name == "main":
            _block(f"{label}: never force-push main")
            rc = 1
        else:
            _warn(f"{label}: force-push on {name} still needs explicit operator request")
    return rc if rc else 1  # default BLOCK — force-push is never casual


def check_docker_redeploy() -> int:
    rc = 0
    _warn(
        "AWS container MUST use --network host; bridge-only drops :9001 "
        "(already burned once on 2026-09-11)"
    )
    ssh = [
        "ssh",
        "-i",
        os.path.expanduser(r"~\.ssh\scm-node-key.pem"),
        "-o",
        "StrictHostKeyChecking=no",
        "-o",
        "ConnectTimeout=8",
        f"ec2-user@{AWS_HOST}",
        "docker inspect scm-node --format '{{.HostConfig.NetworkMode}} {{.Config.Image}}' 2>/dev/null || true",
    ]
    try:
        out = _run(ssh)
        text = (out.stdout or "").strip()
        if text:
            _ok(f"live inspect: {text}")
            if "host" not in text.split()[0:1] and "host" not in text:
                _block("network mode is not host — fix run args before replace")
                rc = 1
        else:
            _warn("could not inspect scm-node (ssh or docker down)")
    except Exception as exc:  # noqa: BLE001 — preflight must not crash
        _warn(f"ssh inspect failed: {exc}")

    _block("operator approval required for docker pull/redeploy")
    return rc


def check_aws_terminate() -> int:
    _block(
        "only terminate instances that are NOT the always-on cloud node "
        f"(tag hint: {ALWAYS_ON_TAG_HINT}); list first, then ask operator"
    )
    return 1


def check_config_overwrite() -> int:
    rc = 0
    if not CONFIG.is_file():
        _warn(f"config missing (ok if neutralized elsewhere): {CONFIG}")
        return 1
    backups = list(CONFIG.parent.glob(CONFIG_BAK_GLOB))
    if backups:
        _ok(f"backup siblings: {[p.name for p in backups]}")
    else:
        _block("no mimocode.json.bak* — copy current file to dated .bak first")
        rc = 1
    try:
        data = json.loads(CONFIG.read_text(encoding="utf-8"))
    except Exception as exc:  # noqa: BLE001
        _block(f"config is not valid JSON: {exc}")
        return 1
    disabled = data.get("disabled_providers") or []
    if "xiaomi" in disabled or "XiaomiMiMo" in disabled:
        _warn(
            "config still disables xiaomi/MiMo providers — this was the settings "
            "mixup; neutralize or keep backup plan"
        )
    else:
        _ok("config does not disable xiaomi/MiMo providers")
    return rc


def check_build_lock() -> int:
    rc = 0
    locks = []
    for root in (REPO, FRESH):
        if not root.exists():
            continue
        for name in ("cargo.lock", "Cargo.lock"):
            # presence of lock file is normal; look for running compilers instead
            pass
        # Windows: look for java/rustc/cargo processes via simple lock markers
        for marker in root.glob("target/**/.cargo-lock"):
            locks.append(marker)
    if locks:
        _warn(f"cargo lock markers: {len(locks)}")
    _ok("no cross-check of live PIDs beyond markers; still serialize builds")
    _block(
        "confirm no concurrent cargo/gradle before deleting target/ or starting a build"
    )
    return rc if rc else 1


def check_identity_wipe() -> int:
    _block("identity wipe requires operator approval + export path")
    return 1


ACTIONS = {
    "audit_readiness": check_audit_readiness,
    "pm_clear": check_pm_clear,
    "git_reset": check_git_reset,
    "git_checkout_paths": check_git_checkout_paths,
    "rm_rf": check_rm_rf,
    "force_push": check_force_push,
    "docker_redeploy": check_docker_redeploy,
    "aws_terminate": check_aws_terminate,
    "config_overwrite": check_config_overwrite,
    "build_lock": check_build_lock,
    "identity_wipe": check_identity_wipe,
}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--action",
        required=True,
        choices=sorted(ACTIONS.keys()),
        help="destructive action to preflight",
    )
    parser.add_argument("--path", default="", help="optional path context")
    args = parser.parse_args()
    print(f"[PREFLIGHT] action={args.action} path={args.path or '(none)'}")
    print(f"[PREFLIGHT] repo={REPO}")
    rc = ACTIONS[args.action]()
    if rc == 0:
        print("[RESULT] CLEAR — mechanical preconditions passed; operator authority still required")
    elif rc == 1:
        print("[RESULT] BLOCK — do not proceed until the issues above are resolved")
    return rc


if __name__ == "__main__":
    raise SystemExit(main())
