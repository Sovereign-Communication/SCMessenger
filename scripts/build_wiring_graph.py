#!/usr/bin/env python3
"""build_wiring_graph.py -- REPO_MAP Wiring Graph Data Processor.

Parses HANDOFF_AUDIT/REPO_MAP.jsonl, HANDOFF/discovery/REPO_MAP.jsonl,
and repo_map_index.json. Produces wiring_graph.json and
unwired_functions.json for the Cytoscape.js dashboard.

Usage:
    python scripts/build_wiring_graph.py
"""

import json
import os
import re
import sys
from collections import defaultdict
from pathlib import Path

# ---------------------------------------------------------------------------
# Paths
# ---------------------------------------------------------------------------
REPO_ROOT = Path(__file__).resolve().parent.parent
AUDIT_JSONL = REPO_ROOT / "HANDOFF_AUDIT" / "REPO_MAP.jsonl"
DISCOVERY_JSONL = REPO_ROOT / "HANDOFF" / "discovery" / "REPO_MAP.jsonl"
INDEX_JSON = REPO_ROOT / "HANDOFF_AUDIT" / "repo_map_index.json"
OUT_DIR = REPO_ROOT / "log-visualizer" / "public" / "data"

# ---------------------------------------------------------------------------
# Code artifact / keyword filter
# ---------------------------------------------------------------------------
# Characters that signal code fragments (not valid identifiers on their own)
CODE_ARTIFACT_CHARS = set(r'{}[]();=+-*/<>&|!#@%^~,\'"`')

# Rust keywords (standalone, not inside :: paths)
RUST_KEYWORDS = {
    "if", "else", "for", "while", "loop", "match", "let", "mut", "fn",
    "impl", "use", "mod", "pub", "return", "break", "continue", "async",
    "await", "move", "ref", "self", "Self", "super", "crate", "where",
    "type", "enum", "struct", "trait", "const", "static", "unsafe", "extern",
    "dyn", "box",
}

RUST_MACROS = {
    "assert!", "assert_eq!", "assert_ne!", "debug_assert!", "debug_assert_eq!",
    "panic!", "unreachable!", "todo!", "unimplemented!", "format!",
    "println!", "eprintln!", "dbg!", "vec!", "ok", "err", "some", "none",
    "Ok", "Err", "Some", "None", "true", "false",
}

KOTLIN_KEYWORDS = {
    "class", "fun", "val", "var", "object", "interface", "data", "sealed",
    "abstract", "open", "override", "private", "protected", "internal",
    "public", "companion", "init", "constructor", "this", "super", "when",
    "is", "as", "in", "out", "null", "true", "false", "it", "Unit", "Any",
    "Nothing", "String", "Int", "Boolean", "List", "Map", "Set",
}

PYTHON_KEYWORDS = {
    "def", "class", "lambda", "pass", "yield", "raise", "except", "finally",
    "with", "import", "from", "global", "nonlocal", "del", "exec", "True",
    "False", "None", "self", "cls", "print", "len", "range", "str", "int",
    "float", "list", "dict", "set", "tuple", "bool", "isinstance",
    "hasattr", "getattr", "setattr", "type",
}

ALL_KEYWORDS = RUST_KEYWORDS | RUST_MACROS | KOTLIN_KEYWORDS | PYTHON_KEYWORDS

# ---------------------------------------------------------------------------
# Crate / language detection from path
# ---------------------------------------------------------------------------
LANGUAGE_COLORS = {
    "rust":      "#00ced1",  # cyan
    "kotlin":    "#3de1ad",  # green
    "swift":     "#f472b6",  # pink
    "python":    "#facc15",  # yellow
    "unknown":   "#94a3b8",  # grey
}

CRATE_COLORS = {
    "scmessenger-core":    "#00ced1",
    "scmessenger-cli":     "#22d3ee",
    "scmessenger-wasm":    "#4ade80",
    "scmessenger-mobile":  "#fbbf24",
    "Android":             "#3de1ad",
    "iOS":                 "#f472b6",
    "Python":              "#facc15",
    "Patch":               "#fb923c",
}

def detect_crate_and_lang(file_path: str) -> tuple:
    """Return (crate_name, language, color) for a file path."""
    fp = file_path.replace("\\", "/")
    if fp.startswith("core/"):
        return ("scmessenger-core", "rust", CRATE_COLORS["scmessenger-core"])
    if fp.startswith("cli/"):
        return ("scmessenger-cli", "rust", CRATE_COLORS["scmessenger-cli"])
    if fp.startswith("wasm/"):
        return ("scmessenger-wasm", "rust", CRATE_COLORS["scmessenger-wasm"])
    if fp.startswith("mobile/"):
        return ("scmessenger-mobile", "rust", CRATE_COLORS["scmessenger-mobile"])
    if fp.startswith("android/") or fp.startswith("app/"):
        return ("Android", "kotlin", CRATE_COLORS["Android"])
    if fp.startswith("iOS/") or fp.startswith("ios/"):
        return ("iOS", "swift", CRATE_COLORS["iOS"])
    if fp.startswith("patch/"):
        return ("Patch", "rust", CRATE_COLORS["Patch"])
    if fp.startswith("scripts/") or fp.endswith(".py"):
        return ("Python", "python", CRATE_COLORS["Python"])
    return ("Unknown", "unknown", LANGUAGE_COLORS["unknown"])


def derive_module(file_path: str) -> str:
    """Return the parent module directory, e.g. core/src/transport."""
    fp = file_path.replace("\\", "/")
    parts = fp.split("/")
    # Drop the filename, keep the directory path as module
    if len(parts) > 1:
        return "/".join(parts[:-1])
    return "root"


# ---------------------------------------------------------------------------
# JSONL multi-line parser
# ---------------------------------------------------------------------------
def parse_audit_jsonl(path: Path):
    """Parse AUDIT REPO_MAP.jsonl — multi-line JSON separated by BOM markers.

    Format: <record1 JSON>\\n<BOM><record2 JSON>\\n<BOM>...
    """
    with open(path, "r", encoding="utf-8-sig") as fh:
        raw = fh.read()

    segments = raw.split("﻿")
    for seg in segments:
        seg = seg.strip()
        if not seg:
            continue
        try:
            yield json.loads(seg)
        except json.JSONDecodeError as exc:
            print(f"  [WARN] skipping malformed segment in {path.name}: "
                  f"{str(exc)[:100]}")


def parse_discovery_jsonl(path: Path):
    """Parse discovery REPO_MAP.jsonl — standard JSONL (one record per line)."""
    with open(path, "r", encoding="utf-8-sig") as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            try:
                yield json.loads(line)
            except json.JSONDecodeError as exc:
                print(f"  [WARN] skipping malformed line in {path.name}: "
                      f"{str(exc)[:100]}")


# ---------------------------------------------------------------------------
# Main processing
# ---------------------------------------------------------------------------
def main():
    print("=" * 60)
    print("REPO_MAP Wiring Graph Builder")
    print("=" * 60)

    # 1. Load index → build (basename, chunk) → full_path lookup
    print("\n[1/6] Loading repo_map_index.json ...")
    with open(INDEX_JSON, "r", encoding="utf-8-sig") as fh:
        index = json.load(fh)
    files_index = index.get("files", {})

    basename_chunk_to_path = {}
    for full_path, meta in files_index.items():
        basename = os.path.basename(full_path)
        for ch in meta.get("chunks", []):
            key = (basename, ch)
            if key in basename_chunk_to_path:
                # Should never happen per our collision analysis, but be safe
                existing = basename_chunk_to_path[key]
                print(f"  [WARN] collision: {key} -> {existing} vs {full_path}")
            basename_chunk_to_path[key] = full_path

    print(f"  Index: {len(files_index)} files, "
          f"{len(basename_chunk_to_path)} (basename,chunk) entries")

    # 2. Parse AUDIT REPO_MAP.jsonl (primary source)
    print("\n[2/6] Parsing HANDOFF_AUDIT/REPO_MAP.jsonl ...")
    audit_functions = []  # list of normalized dicts
    audit_parse_errors = 0
    for rec in parse_audit_jsonl(AUDIT_JSONL):
        basename = rec.get("file", "")
        chunk = rec.get("chunk", 0)
        full_path = basename_chunk_to_path.get((basename, chunk))
        if full_path is None:
            audit_parse_errors += 1
            continue

        crate_name, language, color = detect_crate_and_lang(full_path)
        module = derive_module(full_path)

        for func in rec.get("funcs", []):
            audit_functions.append({
                "name": func.get("name", ""),
                "file": full_path,
                "line": func.get("line", 0),
                "chunk": chunk,
                "calls_out_to": func.get("calls_out_to", []),
                "is_stub": False,  # AUDIT doesn't track this
                "crate": crate_name,
                "language": language,
                "color": color,
                "module": module,
                "source": "audit",
            })

    print(f"  Parsed {len(audit_functions)} function records "
          f"({audit_parse_errors} unresolved)")

    # 3. Parse DISCOVERY REPO_MAP.jsonl (overlay for is_stub + extra funcs)
    print("\n[3/6] Parsing HANDOFF/discovery/REPO_MAP.jsonl ...")
    discovery_functions = []
    disc_parse_errors = 0
    for rec in parse_discovery_jsonl(DISCOVERY_JSONL):
        full_path = rec.get("file_path", "")
        if not full_path:
            disc_parse_errors += 1
            continue

        crate_name, language, color = detect_crate_and_lang(full_path)
        module = derive_module(full_path)

        for func in rec.get("functions", []):
            discovery_functions.append({
                "name": func.get("name", ""),
                "file": full_path,
                "line": func.get("line_approx", 0),
                "chunk": 0,  # discovery doesn't track chunks
                "calls_out_to": func.get("calls_out_to", []),
                "is_stub": func.get("is_stub_or_incomplete", False),
                "crate": crate_name,
                "language": language,
                "color": color,
                "module": module,
                "source": "discovery",
            })

    print(f"  Parsed {len(discovery_functions)} function records "
          f"({disc_parse_errors} unresolved)")

    # TODO(kunal, 2026-05-04): REPO_MAP generation must enforce is_stub tracking
    # in the AUDIT pipeline. Currently the AUDIT file has no is_stub field,
    # so stub detection relies solely on the discovery overlay. Normalize
    # both schemas to include `is_stub: bool` as a required field.

    # 4. Deduplicate by id = file::function_name (merge, prefer earliest line,
    #    is_stub OR from either source)
    print("\n[4/6] Deduplicating nodes ...")
    nodes_by_id = {}

    def merge_func(existing, incoming):
        """Merge incoming into existing. existing is modified in place."""
        # Merge calls_out_to (union, preserve order)
        existing_calls = set(existing["calls_out_to"])
        for c in incoming.get("calls_out_to", []):
            if c not in existing_calls:
                existing["calls_out_to"].append(c)
                existing_calls.add(c)
        # Stub flag: true if EITHER source says so
        existing["is_stub"] = existing["is_stub"] or incoming.get("is_stub", False)
        # Keep earliest line
        inc_line = incoming.get("line", 0)
        if inc_line and (existing["line"] == 0 or inc_line < existing["line"]):
            existing["line"] = inc_line
        # Track sources
        existing["sources"].add(incoming.get("source", "unknown"))

    # First pass: AUDIT (primary)
    for f in audit_functions:
        nid = f"{f['file']}::{f['name']}"
        f["sources"] = {f["source"]}
        if nid in nodes_by_id:
            merge_func(nodes_by_id[nid], f)
        else:
            nodes_by_id[nid] = f

    # Second pass: DISCOVERY (overlay)
    for f in discovery_functions:
        nid = f"{f['file']}::{f['name']}"
        f["sources"] = {f["source"]}
        if nid in nodes_by_id:
            merge_func(nodes_by_id[nid], f)
        else:
            nodes_by_id[nid] = f

    print(f"  Unique nodes after dedup: {len(nodes_by_id)}")

    # 5. Filter and resolve edges
    print("\n[5/6] Resolving edges ...")

    # Build a global name → [node_ids] index for internal resolution
    simple_name_index = defaultdict(list)
    for nid, node in nodes_by_id.items():
        simple_name_index[node["name"]].append(nid)

    # Also build file-local name index (for same-file resolution)
    file_func_index = defaultdict(lambda: defaultdict(list))
    for nid, node in nodes_by_id.items():
        file_func_index[node["file"]][node["name"]].append(nid)

    def is_artifact(call_str: str) -> bool:
        """Check if a call string is a code artifact, not a valid target."""
        cs = call_str.strip()
        if not cs or len(cs) > 120:
            return True
        # Contains code-syntax characters → artifact
        for ch in cs:
            if ch in CODE_ARTIFACT_CHARS:
                return True
        # Exact keyword match → artifact
        if cs in ALL_KEYWORDS:
            return True
        # Starts with keyword followed by space (code fragments)
        first_word = cs.split()[0] if " " in cs else cs
        if first_word in ALL_KEYWORDS:
            return True
        return False

    def classify_call(call_str: str) -> str:
        """Classify a valid call: 'external', 'simple'."""
        if "::" in call_str or "." in call_str:
            return "external"
        return "simple"

    edges = []
    external_refs = defaultdict(list)   # name -> [caller_node_ids]
    unresolved_refs = defaultdict(list)  # name -> [caller_node_ids]

    for nid, node in nodes_by_id.items():
        for call in node["calls_out_to"]:
            if is_artifact(call):
                continue

            call_type = classify_call(call)

            if call_type == "external":
                external_refs[call].append(nid)
                edges.append({
                    "source": nid,
                    "target": f"ext::{call}",
                    "type": "external",
                    "label": call,
                })
                continue

            # Simple name resolution
            call_name = call.strip()
            target = resolve_simple_name(
                call_name, node["file"], nodes_by_id,
                simple_name_index, file_func_index,
            )
            if target:
                edges.append({
                    "source": nid,
                    "target": target["id"],
                    "type": target["resolution"],
                })
            else:
                unresolved_refs[call_name].append(nid)

    print(f"  Internal edges: {sum(1 for e in edges if e['type'] != 'external')}")
    print(f"  External refs:  {len(external_refs)} unique ({sum(len(v) for v in external_refs.values())} total)")
    print(f"  Unresolved:     {len(unresolved_refs)} unique ({sum(len(v) for v in unresolved_refs.values())} total)")

    # 6. Compute metadata: in/out degree, is_unwired, called_by
    print("\n[6/6] Computing metadata ...")

    in_degree = defaultdict(int)
    out_degree = defaultdict(int)
    called_by = defaultdict(list)

    for edge in edges:
        src = edge["source"]
        tgt = edge["target"]
        out_degree[src] += 1
        in_degree[tgt] += 1
        called_by[tgt].append(src)

    nodes_out = []
    for nid, node in nodes_by_id.items():
        ind = in_degree.get(nid, 0)
        outd = out_degree.get(nid, 0)
        nodes_out.append({
            "id": nid,
            "name": node["name"],
            "file": node["file"],
            "line": node["line"],
            "crate": node["crate"],
            "language": node["language"],
            "color": node["color"],
            "module": node["module"],
            "group_file": node["file"],
            "group_module": node["module"],
            "group_crate": node["crate"],
            "group_ext": "ext" if ind == 0 and outd == 0 else "wired",
            "in_degree": ind,
            "out_degree": outd,
            "is_unwired": outd == 0 and ind == 0,
            "is_stub": node["is_stub"],
            "calls_out_to": [c for c in node["calls_out_to"] if not is_artifact(c)],
            "called_by": called_by.get(nid, []),
        })

    # Separate unwired
    unwired = [n for n in nodes_out if n["is_unwired"] and not n["is_stub"]]
    stubs = [n for n in nodes_out if n["is_stub"]]

    # Build stats
    wired_count = sum(1 for n in nodes_out if not n["is_unwired"])
    total_edges_internal = sum(1 for e in edges if e["type"] != "external")
    total_edges_external = sum(1 for e in edges if e["type"] == "external")

    stats = {
        "total_functions": len(nodes_out),
        "wired": wired_count,
        "unwired": len(unwired),
        "stubs": len(stubs),
        "internal_edges": total_edges_internal,
        "external_refs": len(external_refs),
        "unresolved": len(unresolved_refs),
        "crates": sorted(set(n["crate"] for n in nodes_out)),
        "files": len(set(n["file"] for n in nodes_out)),
        "generated_at": _now_iso(),
    }

    # Build output structures
    wiring_graph = {
        "meta": {
            "version": "1.0",
            "generated_at": stats["generated_at"],
            "description": "REPO_MAP call graph for Cytoscape.js wiring visualizer",
        },
        "stats": stats,
        "nodes": nodes_out,
        "edges": edges,
        "external_refs": [
            {"name": name, "callers": callers, "count": len(callers)}
            for name, callers in sorted(
                external_refs.items(), key=lambda x: -len(x[1])
            )
        ],
        "unresolved": [
            {"name": name, "callers": callers, "count": len(callers)}
            for name, callers in sorted(
                unresolved_refs.items(), key=lambda x: -len(x[1])
            )
        ],
    }

    unwired_functions = {
        "meta": {
            "version": "1.0",
            "generated_at": stats["generated_at"],
            "description": "Unwired functions — no callers and no callees",
        },
        "count": len(unwired),
        "functions": sorted(unwired, key=lambda n: (n["file"], n["line"])),
    }

    # 7. Write outputs
    print("\nWriting outputs ...")
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    graph_path = OUT_DIR / "wiring_graph.json"
    with open(graph_path, "w", encoding="utf-8") as fh:
        json.dump(wiring_graph, fh, indent=2, ensure_ascii=False)
    size_mb = graph_path.stat().st_size / (1024 * 1024)
    print(f"  {graph_path} ({size_mb:.1f} MB)")

    unwired_path = OUT_DIR / "unwired_functions.json"
    with open(unwired_path, "w", encoding="utf-8") as fh:
        json.dump(unwired_functions, fh, indent=2, ensure_ascii=False)
    print(f"  {unwired_path}")

    # Summary
    print("\n" + "=" * 60)
    print("GRAPH SUMMARY")
    print("=" * 60)
    print(f"  Functions:    {stats['total_functions']}")
    print(f"  Wired:        {stats['wired']}  ({100*wired_count//max(1,stats['total_functions'])}%)")
    print(f"  Unwired:      {stats['unwired']}")
    print(f"  Stubs:        {stats['stubs']}")
    print(f"  Int edges:    {stats['internal_edges']}")
    print(f"  Ext refs:     {stats['external_refs']}")
    print(f"  Unresolved:   {stats['unresolved']}")
    print(f"  Crates:       {', '.join(stats['crates'])}")
    print("=" * 60)
    print("Done.")


# ---------------------------------------------------------------------------
# Simple-name edge resolution
# ---------------------------------------------------------------------------
def resolve_simple_name(name, source_file, nodes_by_id,
                        simple_name_index, file_func_index):
    """Resolve a simple (no ::, no .) call name to a target node.

    Resolution order:
      1. Same file → internal
      2. Same crate → cross_file (pick earliest line)
      3. Any match → cross_file (ambiguous if >1)
      4. None → None (unresolved, caller handles it)
    """
    candidates = simple_name_index.get(name, [])
    if not candidates:
        return None

    # 1. Prefer same-file
    same_file = [c for c in candidates
                 if nodes_by_id.get(c, {}).get("file") == source_file]
    if same_file:
        return {"id": same_file[0], "resolution": "internal"}

    # 2. Single global match → cross_file
    if len(candidates) == 1:
        return {"id": candidates[0], "resolution": "cross_file"}

    # 3. Multiple matches → ambiguous, pick earliest line
    best = min(candidates,
               key=lambda c: nodes_by_id.get(c, {}).get("line", 99999))
    return {"id": best, "resolution": "ambiguous"}


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
def _now_iso() -> str:
    from datetime import datetime, timezone
    return datetime.now(timezone.utc).isoformat()


if __name__ == "__main__":
    main()
