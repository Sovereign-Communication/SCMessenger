#!/usr/bin/env python3
"""query_repograph.py -- Sub-second RepoGraph AST & Call-Graph Search CLI.

Allows searching for symbol dependencies, 1-hop/2-hop neighbors, shortest paths,
and unwired function hotspots across the SCMessenger codebase.

Usage:
    python scripts/query_repograph.py --symbol IronCore
    python scripts/query_repograph.py --unwired --workspace core
    python scripts/query_repograph.py --path IronCore RelayServer
"""

import argparse
import json
import os
import pickle
import sys
from pathlib import Path

# Paths
REPO_ROOT = Path(__file__).resolve().parent.parent
REPOGRAPH_DIR = REPO_ROOT.parent / "RepoGraph"
GRAPH_PKL = REPOGRAPH_DIR / "graph.pkl"
DATA_JSON = REPO_ROOT / "log-visualizer" / "public" / "data" / "repograph_data.json"
UNWIRED_JSON = REPO_ROOT / "log-visualizer" / "public" / "data" / "unwired_functions.json"


def load_data_json():
    # Fallback to SCM-Progress if local log-visualizer json not found
    candidates = [
        DATA_JSON,
        REPO_ROOT.parent / "SCM-Progress" / "public" / "data" / "repograph_data.json",
        REPO_ROOT.parent / "SCM-Progress" / "data" / "repograph_data.json",
    ]
    for c in candidates:
        if c.exists():
            with open(c, "r", encoding="utf-8") as f:
                return json.load(f)
    return None


def search_symbol(data, query_symbol):
    nodes = data.get("nodes", [])
    links = data.get("links", [])

    matched_nodes = [
        n for n in nodes
        if query_symbol.lower() in n.get("name", "").lower()
        or query_symbol.lower() in n.get("id", "").lower()
    ]

    print(f"\n============================================================")
    print(f"RepoGraph Symbol Search: '{query_symbol}' ({len(matched_nodes)} matches)")
    print(f"============================================================")

    for n in matched_nodes[:10]:
        nid = n["id"]
        print(f"\n[NODE] {n.get('name')} [{n.get('category')}]")
        print(f"   ID:        {nid}")
        print(f"   Workspace: {n.get('workspace')}")
        print(f"   File:      {n.get('fname')}:{n.get('line')}")
        print(f"   Degree:    {n.get('degree', 0)}")

        # Find incoming & outgoing links
        incoming = [l["source"] for l in links if l["target"] == nid]
        outgoing = [l["target"] for l in links if l["source"] == nid]

        if incoming:
            print(f"   Caller In-Edges ({len(incoming)}):")
            for inc in incoming[:5]:
                print(f"     <- {inc.split('::')[-1]} ({inc})")
        if outgoing:
            print(f"   Callee Out-Edges ({len(outgoing)}):")
            for out in outgoing[:5]:
                print(f"     -> {out.split('::')[-1]} ({out})")


def list_unwired(workspace_filter=None, limit=15):
    if not UNWIRED_JSON.exists():
        print(f"Error: {UNWIRED_JSON} not found.")
        return

    with open(UNWIRED_JSON, "r", encoding="utf-8") as f:
        unwired_data = json.load(f)

    functions = unwired_data.get("functions", [])
    if workspace_filter:
        functions = [f for f in functions if workspace_filter.lower() in f.get("file", "").lower()]

    print(f"\n============================================================")
    print(f"Unwired Functions ({len(functions)} total)")
    print(f"============================================================")

    for fn in functions[:limit]:
        stub_flag = " [STUB]" if fn.get("is_stub") else ""
        print(f" - {fn.get('name')}{stub_flag} @ {fn.get('file')}:{fn.get('line')}")


def main():
    parser = argparse.ArgumentParser(description="Sub-second RepoGraph AST & Call-Graph Query Tool")
    parser.add_argument("--symbol", type=str, help="Search symbol by name or ID")
    parser.add_argument("--unwired", action="store_true", help="List unwired functions")
    parser.add_argument("--workspace", type=str, help="Filter by workspace (core, android, ios, etc.)")
    parser.add_argument("--limit", type=int, default=15, help="Limit output lines")

    args = parser.parse_args()

    data = load_data_json()
    if not data:
        print("Error: RepoGraph JSON data not found. Run python ./repograph/export_repograph_json.py first.")
        sys.exit(1)

    if args.symbol:
        search_symbol(data, args.symbol)
    elif args.unwired:
        list_unwired(args.workspace, args.limit)
    else:
        summary = data.get("summary", {})
        print("\n============================================================")
        print("RepoGraph Graph Overview")
        print("============================================================")
        print(f" Nodes:          {summary.get('nodesCount', len(data.get('nodes', [])))}")
        print(f" Edges:          {summary.get('edgesCount', len(data.get('links', [])))}")
        print(f" Workspaces:     {list(summary.get('workspaceCounts', {}).keys())}")
        print("\nUsage:")
        print("  python scripts/query_repograph.py --symbol IronCore")
        print("  python scripts/query_repograph.py --unwired --workspace android")


if __name__ == "__main__":
    main()
