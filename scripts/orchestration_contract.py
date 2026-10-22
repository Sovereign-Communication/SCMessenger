#!/usr/bin/env python3
"""Validated access to the repo-owned Orchestration Control Plane v2 contract.

The manifest is JSON-compatible YAML so this helper requires no non-standard
runtime dependency. Frontend adapters and the kernel use this module instead
of maintaining their own copies of authority, lifecycle, or result rules.
"""

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_MANIFEST = ROOT / "orchestration" / "manifest.yaml"
REQUIRED_TOP_LEVEL = {
    "schema_version", "protocol_version", "canonical_document", "kernel",
    "execution_pointer_document",
    "state_schema_version", "semantic_roles", "lifecycle", "protected_prefixes",
    "worker_result", "adapters", "codex_profiles", "model_capabilities",
}


class ContractError(ValueError):
    """Raised when the canonical contract is absent or unsafe to use."""


def resolve_execution_queue(manifest):
    """Resolve the queue named by SHIP_PLAN's authoritative execution pointer."""
    document_name = manifest.get("execution_pointer_document")
    if not isinstance(document_name, str) or not document_name.strip():
        raise ContractError("manifest execution_pointer_document must be a non-empty path")
    document_rel = Path(document_name)
    if document_rel.is_absolute() or ".." in document_rel.parts:
        raise ContractError("manifest execution_pointer_document must stay inside the repository")
    document_path = (ROOT / document_rel).resolve()
    try:
        document_path.relative_to(ROOT)
    except ValueError as exc:
        raise ContractError("manifest execution_pointer_document must stay inside the repository") from exc
    if not document_path.is_file():
        raise ContractError(f"execution pointer document is missing: {document_name}")

    lines = document_path.read_text(encoding="utf-8").splitlines()
    marker = next((i for i, line in enumerate(lines) if "EXECUTION POINTER (authoritative):" in line), None)
    if marker is None:
        raise ContractError(f"execution pointer document has no authoritative execution pointer: {document_name}")
    queue_match = None
    for line in lines[marker:marker + 8]:
        queue_match = re.search(r"\*\*`([^`]+)`\*\*", line)
        if queue_match:
            break
    if not queue_match:
        raise ContractError(f"authoritative execution pointer has no queue path: {document_name}")

    queue_rel = Path(queue_match.group(1).strip())
    if queue_rel.is_absolute() or ".." in queue_rel.parts:
        raise ContractError("authoritative execution queue must stay inside the repository")
    queue_path = (ROOT / queue_rel).resolve()
    try:
        queue_path.relative_to(ROOT)
    except ValueError as exc:
        raise ContractError("authoritative execution queue must stay inside the repository") from exc
    if not queue_path.is_file():
        raise ContractError(f"authoritative execution queue is missing: {queue_rel.as_posix()}")
    return queue_path


def load_manifest(path=DEFAULT_MANIFEST):
    try:
        with Path(path).open(encoding="utf-8") as handle:
            manifest = json.load(handle)
    except (OSError, json.JSONDecodeError) as exc:
        raise ContractError(f"cannot load orchestration manifest: {exc}") from exc
    validate_manifest(manifest)
    return manifest


def validate_manifest(manifest):
    missing = REQUIRED_TOP_LEVEL - set(manifest)
    if missing:
        raise ContractError(f"manifest missing required keys: {sorted(missing)}")
    if manifest["protocol_version"] != manifest["schema_version"]:
        raise ContractError("protocol_version and schema_version must match")
    roles = manifest["semantic_roles"]
    for role in ("CONTROLLER", "IMPLEMENTER", "VALIDATOR", "CRITICAL_VALIDATOR", "OPERATOR"):
        if role not in roles:
            raise ContractError(f"manifest missing semantic role {role}")
    for profile, definition in manifest["codex_profiles"].items():
        if definition.get("semantic_role") not in roles or not (ROOT / definition.get("path", "")).is_file():
            raise ContractError(f"invalid Codex profile mapping: {profile}")
    profile_files = {path.stem for path in (ROOT / ".codex" / "agents").glob("*.toml")}
    if set(manifest["codex_profiles"]) != profile_files:
        raise ContractError("every Codex profile must have one explicit universal role mapping")
    for profile, definition in manifest["codex_profiles"].items():
        if definition.get("review_capable"):
            text = (ROOT / definition["path"]).read_text(encoding="utf-8")
            required_refs = ("docs/ORCHESTRATION.md", "orchestration/manifest.yaml", "protocol v2.0.0", "canonical", "worker footer")
            if not all(reference in text for reference in required_refs):
                raise ContractError(f"review-capable Codex profile lacks thin canonical contract references: {profile}")
    transitions = manifest["lifecycle"].get("transitions", {})
    for state, destinations in transitions.items():
        if not isinstance(destinations, list):
            raise ContractError(f"lifecycle {state} destinations must be a list")
    result = manifest["worker_result"]
    required = {"RESULT", "ROLE", "TASK", "FILES", "VERIFICATION", "SPEC_STATUS", "ESCALATION", "NOTES"}
    if not required.issubset(result.get("required", [])):
        raise ContractError("worker result contract is incomplete")
    resolve_execution_queue(manifest)
    return manifest


def valid_transition(manifest, current, target):
    return target in manifest["lifecycle"]["transitions"].get(current, [])


def protected_paths(manifest, files):
    prefixes = tuple(manifest["protected_prefixes"])
    return [path for path in files if path.replace("\\", "/").startswith(prefixes)]


def requires_delivery_review(manifest, files, description=""):
    text = " ".join([description, *files]).lower()
    return any(word in text for word in manifest.get("delivery_keywords", []))


def validate_result(manifest, result, expected_task=None, expected_role=None):
    required = manifest["worker_result"]["required"]
    missing = [key for key in required if not result.get(key)]
    if missing:
        return False, f"missing result fields: {', '.join(missing)}"
    if result["RESULT"] not in manifest["worker_result"]["result_values"]:
        return False, "invalid RESULT"
    if result["SPEC_STATUS"] not in manifest["worker_result"]["spec_status_values"]:
        return False, "invalid SPEC_STATUS"
    if result["ESCALATION"] not in manifest["worker_result"]["escalation_values"]:
        return False, "invalid ESCALATION"
    if expected_task and result["TASK"] != expected_task:
        return False, "TASK does not match dispatch"
    if expected_role and result["ROLE"] != expected_role:
        return False, "ROLE does not match dispatch"
    if result["RESULT"] != "DONE" or result["SPEC_STATUS"] != "SATISFIED":
        return False, "result is not a successful satisfied completion"
    if result["ESCALATION"] != "NONE":
        return False, "successful completion may not carry an escalation"
    return True, "ok"


def main():
    import argparse
    parser = argparse.ArgumentParser(description="Validate the canonical orchestration manifest")
    parser.add_argument("--manifest", default=str(DEFAULT_MANIFEST))
    parser.add_argument("--print-version", action="store_true")
    args = parser.parse_args()
    try:
        manifest = load_manifest(args.manifest)
    except ContractError as exc:
        print(f"[ERROR] {exc}")
        return 1
    if args.print_version:
        print(manifest["protocol_version"])
    else:
        print(f"[OK] orchestration manifest {manifest['protocol_version']} is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
