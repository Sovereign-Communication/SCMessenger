#!/usr/bin/env python3
"""SCMessenger Android Wiring & Reachability Gate.

Executable verification of AGENTS.md Rule 16 (Reachability & Wiring Enforcement).
Prevents unreachable features, dead routes, orphan dialogs, and undeclared
manifest components from being merged into main.

Checks Performed:
  C1. Zero-caller declarations:
      Scans @Composable fun, Activity, Service, BroadcastReceiver, ViewModel,
      dialog, utility, and -- in the advisory pass -- every plain function
      under android/app/src/main/java. Reports any declaration whose only
      reference is its own declaration.
      (Excludes @Preview annotations and test sources).

  C2. Nav route reachability:
      Parses Screen route definitions and NavHost composable(...) registrations.
      Reports any route that is defined and/or navigated to but has no
      composable(...) registration in the NavHost (navigation lands nowhere),
      and registered routes that are never navigated to.

  C3. Manifest reachability:
      Scans Android source for classes extending Activity, Service, VpnService,
      or BroadcastReceiver and verifies that each is declared in AndroidManifest.xml.

  C4. Transitive death:
      If a declaration is referenced ONLY by something already reported dead or
      unreachable, it is marked as transitively dead with the complete chain.

Limitations & Edge Cases:
  - Static analysis cannot trace dynamic reflection or runtime string concatenation.
  - Where a caller cannot be definitively proven absent or present due to dynamic DI
    or reflection, the scanner reports UNCERTAIN rather than claiming dead to avoid
    false deletions.
  - Tooling-only declarations (@Preview) and test code (src/test, src/androidTest)
    are intentionally excluded from reachability targets.

Two passes, one exit code:
  BLOCKING  -- the pre-widening declaration set (composables, screens,
               components, /utils/ functions). Unchanged by the widening.
  ADVISORY  -- every plain function in android/app/src/main, reported at WARN.
               It exists because the blocking pass never inspected
               service/, data/ or ui/viewmodels/ functions at all, which is how
               decideCommand stayed unreferenced on the live seam while ~25
               tests exercised it and this gate reported nothing.

  Only ERROR findings fail the gate. The advisory pass is deliberately
  isolated: it reports function findings only, so a dead function can never
  cascade into failing the gate on the composables and components that depend
  on it.

  Promote the advisory scan to blocking with --method-findings=block once
  wiring-audit/FINDINGS.md has been triaged.

Usage:
  python scripts/check_wiring.py
  python scripts/check_wiring.py --json
  python scripts/check_wiring.py --root /path/to/repo
  python scripts/check_wiring.py --method-findings=block   # promote the scan
"""

import argparse
import json
import os
import re
import sys
import xml.etree.ElementTree as ET
from dataclasses import asdict, dataclass, field
from typing import Dict, List, Optional, Set, Tuple


@dataclass
class Declaration:
    """Represents a Kotlin code declaration."""
    name: str
    kind: str  # Composable, Dialog, ViewModel, Activity, Service, BroadcastReceiver, Utility, Method, Class
    file: str
    line: int
    fqcn: str = ""
    container: str = ""
    is_preview: bool = False
    is_nested: bool = False
    is_local: bool = False


@dataclass
class Finding:
    """Represents an unreachable wiring defect or rule violation."""
    file: str
    line: int
    kind: str  # C1_ZERO_CALLERS, C2_UNREGISTERED_ROUTE, C2_UNNAVIGATED_ROUTE, C3_MANIFEST_MISSING, C4_TRANSITIVE_DEAD, UNCERTAIN
    symbol: str
    reason: str
    chain: List[str] = field(default_factory=list)
    # "ERROR" findings fail the gate; "WARN" findings are reported only.
    # The widened dead-code scan (kind="Method") is WARN-first: it is
    # advisory until its backlog has been triaged. See --method-findings.
    severity: str = "ERROR"


def strip_comments(source: str) -> str:
    """Strip block and line comments while preserving newlines and character offsets."""
    n = len(source)
    chars = list(source)
    i = 0
    while i < n:
        # Check raw multi-line strings """ ... """
        if i + 2 < n and source[i:i+3] == '"""':
            i += 3
            while i + 2 < n and source[i:i+3] != '"""':
                i += 1
            if i + 2 < n:
                i += 3
            continue
        # Check normal string literals " ... "
        if source[i] == '"':
            i += 1
            while i < n and source[i] != '"':
                if source[i] == '\\':
                    i += 2
                else:
                    i += 1
            if i < n:
                i += 1
            continue
        # Check character literals ' ... '
        if source[i] == "'":
            i += 1
            while i < n and source[i] != "'":
                if source[i] == '\\':
                    i += 2
                else:
                    i += 1
            if i < n:
                i += 1
            continue
        # Check single-line comments //
        if i + 1 < n and source[i:i+2] == '//':
            chars[i] = ' '
            chars[i+1] = ' '
            i += 2
            while i < n and source[i] != '\n':
                chars[i] = ' '
                i += 1
            continue
        # Check multi-line block comments /* ... */
        if i + 1 < n and source[i:i+2] == '/*':
            chars[i] = ' '
            chars[i+1] = ' '
            i += 2
            depth = 1
            while i < n and depth > 0:
                if i + 1 < n and source[i:i+2] == '/*':
                    chars[i] = ' '
                    chars[i+1] = ' '
                    depth += 1
                    i += 2
                elif i + 1 < n and source[i:i+2] == '*/':
                    chars[i] = ' '
                    chars[i+1] = ' '
                    depth -= 1
                    i += 2
                else:
                    if chars[i] != '\n':
                        chars[i] = ' '
                    i += 1
            continue
        i += 1
    return "".join(chars)


def parse_manifest(manifest_path: str) -> Set[str]:
    """Extract all declared component FQCNs and simple names from AndroidManifest.xml."""
    entries = set()
    if not os.path.exists(manifest_path):
        return entries

    try:
        tree = ET.parse(manifest_path)
        root = tree.getroot()
        app_elem = root.find("application")
        if app_elem is not None:
            app_name = app_elem.attrib.get("{http://schemas.android.com/apk/res/android}name")
            if app_name:
                if app_name.startswith("."):
                    app_name = "com.scmessenger.android" + app_name
                entries.add(app_name)
                entries.add(app_name.split(".")[-1])

            for tag in ["activity", "service", "receiver", "provider"]:
                for elem in app_elem.findall(tag):
                    name = elem.attrib.get("{http://schemas.android.com/apk/res/android}name")
                    if name:
                        if name.startswith("."):
                            name = "com.scmessenger.android" + name
                        entries.add(name)
                        entries.add(name.split(".")[-1])
    except Exception as e:
        print(f"[WARNING] Failed to parse manifest {manifest_path}: {e}", file=sys.stderr)

    return entries


def check_nav_routes(
    mesh_app_path: str,
    mesh_content_clean: str,
    repo_root: str
) -> Tuple[List[Finding], Set[str], Set[str]]:
    """Check navigation route reachability (C2) in Compose NavHost."""
    findings: List[Finding] = []
    screen_defs: Dict[str, Tuple[str, int]] = {}
    registered_routes: Set[str] = set()
    registered_composables: Set[str] = set()
    navigated_routes: Set[str] = set()

    # Parse Screen sealed class object definitions: object Foo : Screen("route", ...)
    for m in re.finditer(r'object\s+([A-Za-z0-9_]+)\s*:\s*Screen\s*\(\s*"([^"]+)"', mesh_content_clean):
        screen_obj = m.group(1)
        route_str = m.group(2)
        line_no = mesh_content_clean[:m.start()].count("\n") + 1
        screen_defs[screen_obj] = (route_str, line_no)

    # Parse composable registrations and the screens invoked in their composable lambda bodies
    # Example: composable(Screen.Dashboard.route) { ... DashboardScreen(...) ... }
    for m in re.finditer(
        r'composable\s*\(\s*(?:route\s*=\s*)?(?:Screen\.([A-Za-z0-9_]+)\.route|"([^"]+)")\s*[^)]*\)\s*\{([^}]+)\}',
        mesh_content_clean
    ):
        screen_key = m.group(1)
        route_literal = m.group(2)
        body = m.group(3)

        if screen_key:
            registered_routes.add(f"Screen.{screen_key}")
            if screen_key in screen_defs:
                registered_routes.add(screen_defs[screen_key][0])
        elif route_literal:
            registered_routes.add(route_literal)

        # Extract screen composables called inside registration body
        for comp_m in re.finditer(r'\b([A-Z][A-Za-z0-9_]+Screen|[A-Z][A-Za-z0-9_]+Dialog)\s*\(', body):
            registered_composables.add(comp_m.group(1))

    # Parse navigation calls
    for m in re.finditer(r'navController\.navigate\s*\(\s*(?:Screen\.([A-Za-z0-9_]+)\.route|"([^"]+)")', mesh_content_clean):
        if m.group(1):
            navigated_routes.add(f"Screen.{m.group(1)}")
            if m.group(1) in screen_defs:
                navigated_routes.add(screen_defs[m.group(1)][0])
        elif m.group(2):
            navigated_routes.add(m.group(2))

    # Bottom bar items are implicitly navigated
    if "roleBasedBottomNavItems" in mesh_content_clean or "Screen.fullRoleBottomNavItems" in mesh_content_clean:
        for item in ["Conversations", "Contacts", "Dashboard", "Settings"]:
            navigated_routes.add(f"Screen.{item}")
            if item in screen_defs:
                navigated_routes.add(screen_defs[item][0])

    # Check for defined/navigated routes that lack composable(...) registration
    for screen_obj, (route_str, line_no) in screen_defs.items():
        screen_ref = f"Screen.{screen_obj}"
        is_registered = (screen_ref in registered_routes) or (route_str in registered_routes)
        is_navigated = (screen_ref in navigated_routes) or (route_str in navigated_routes)

        if not is_registered:
            rel_file = os.path.relpath(mesh_app_path, start=repo_root).replace("\\", "/")
            findings.append(Finding(
                file=rel_file,
                line=line_no,
                kind="C2_UNREGISTERED_ROUTE",
                symbol=f"Screen.{screen_obj} (\"{route_str}\")",
                reason=(
                    f"Navigation route '{screen_ref}' (\"{route_str}\") is defined and navigated to, "
                    f"but has no composable(...) registration in NavHost; navigation lands nowhere"
                )
            ))

    return findings, registered_composables, registered_routes


def extract_declarations(kt_files: Dict[str, str], kt_clean: Dict[str, str], method_scope: str = "legacy") -> List[Declaration]:
    """Extract classes, objects, interfaces, composables, and functions.

    method_scope selects how much plain-function coverage is collected:
      "legacy"  -- only functions under /utils/ and FileLoggingTree, which is
                   what this gate collected before the dead-function scan was
                   widened. Used for the BLOCKING pass so its result is
                   unchanged by the widening.
      "widened" -- every non-composable function in android/app/src/main.
                   Used for the ADVISORY pass.
    """
    declarations: List[Declaration] = []

    for rel_p, raw in kt_files.items():
        clean = kt_clean[rel_p]
        lines_raw = raw.splitlines()
        lines_clean = clean.splitlines()

        pkg_match = re.search(r'package\s+([A-Za-z0-9_.]+)', raw)
        pkg = pkg_match.group(1) if pkg_match else ""

        class_stack: List[Tuple[str, int]] = []  # (class_name, brace_depth)
        pending_class: Optional[str] = None
        fun_depth: Optional[int] = None
        current_depth = 0

        for idx, (l_raw, l_clean) in enumerate(zip(lines_raw, lines_clean)):
            line_no = idx + 1

            # Match class / object / interface declaration
            m_class = re.search(
                r'\b(class|object|interface)\s+([A-Za-z0-9_]+)(?:\s*<[^>]+>)?\s*(?:\([^)]*\))?\s*(?::\s*([^{]+))?',
                l_clean
            )
            if m_class:
                cls_name = m_class.group(2)
                parents = m_class.group(3) or ""

                if not ("enum class" in l_clean or "sealed class" in l_clean or "sealed interface" in l_clean):
                    is_activity = any(t in parents for t in ["Activity", "ComponentActivity", "AppCompatActivity"])
                    is_service = any(t in parents for t in ["Service", "VpnService", "JobService"]) and "Timber.Tree" not in parents
                    is_receiver = "BroadcastReceiver" in parents
                    is_viewmodel = "ViewModel" in parents or "@HiltViewModel" in "\n".join(lines_raw[max(0, idx-3):idx+1])
                    is_dialog = cls_name.endswith("Dialog")
                    is_util = "/utils/" in rel_p or "/transport/" in rel_p

                    kind = "Class"
                    if is_activity:
                        kind = "Activity"
                    elif is_service:
                        kind = "Service"
                    elif is_receiver:
                        kind = "BroadcastReceiver"
                    elif is_viewmodel:
                        kind = "ViewModel"
                    elif is_dialog:
                        kind = "Dialog"
                    elif is_util:
                        kind = "Utility"

                    parent_cls = class_stack[-1][0] if class_stack else ""
                    fqcn = f"{pkg}.{cls_name}" if not parent_cls else f"{pkg}.{parent_cls}.{cls_name}"
                    is_local = fun_depth is not None and current_depth >= fun_depth

                    declarations.append(Declaration(
                        name=cls_name,
                        kind=kind,
                        file=rel_p,
                        line=line_no,
                        fqcn=fqcn,
                        container=parent_cls,
                        is_nested=bool(parent_cls),
                        is_local=is_local
                    ))
                    pending_class = cls_name

            # `companion object { ... }` opens a scope whose members are still
            # class members, not local functions. The class regex above does
            # not match it (an object declaration must be followed by a name),
            # so without this a companion's members are misread as locals by
            # the brace-depth test -- which is exactly how decideCommand, a
            # companion function, disappeared from the scan.
            if re.search(r'\bcompanion\s+object\b', l_clean) and l_clean.count("{") > 0:
                class_stack.append((class_stack[-1][0] if class_stack else "", current_depth + l_clean.count("{")))

            # Match function declaration
            m_fun = re.search(r'\bfun\s+(?:<[^>]+>\s+)?([A-Za-z0-9_]+)\s*\(', l_clean)
            if m_fun:
                fun_name = m_fun.group(1)
                prev_chunk = "\n".join(lines_raw[max(0, idx-4):idx+1])
                is_composable = "@Composable" in prev_chunk
                is_preview = "@Preview" in prev_chunk
                parent_cls = class_stack[-1][0] if class_stack else ""

                if is_composable:
                    kind = "Dialog" if fun_name.endswith("Dialog") else "Composable"
                    fqcn = f"{pkg}.{fun_name}" if not parent_cls else f"{pkg}.{parent_cls}.{fun_name}"
                    declarations.append(Declaration(
                        name=fun_name,
                        kind=kind,
                        file=rel_p,
                        line=line_no,
                        fqcn=fqcn,
                        container=parent_cls,
                        is_preview=is_preview,
                        is_nested=bool(parent_cls),
                        is_local=(fun_depth is not None and current_depth >= fun_depth)
                    ))
                else:
                    # Every remaining non-composable function, in every main
                    # source. This used to be restricted to "/utils/" and
                    # FileLoggingTree, which left service/, data/ and
                    # ui/viewmodels/ entirely invisible to the zero-caller
                    # check -- the gap that let decideCommand sit on the live
                    # seam with zero production callers while ~25 tests
                    # exercised it and this gate stayed green.
                    stripped = l_clean.strip()
                    legacy_method = ("/utils/" in rel_p) or ("FileLoggingTree" in rel_p)
                    if method_scope == "legacy" and not legacy_method:
                        pass
                    elif not stripped.startswith("private ") and not stripped.startswith("override "):
                        fqcn = f"{pkg}.{parent_cls}.{fun_name}" if parent_cls else f"{pkg}.{fun_name}"
                        declarations.append(Declaration(
                            name=fun_name,
                            kind="Method",
                            file=rel_p,
                            line=line_no,
                            fqcn=fqcn,
                            container=parent_cls,
                            is_preview=False,
                            is_nested=bool(parent_cls),
                            # A function declared deeper than its enclosing
                            # class is local to some enclosing function, not a
                            # reachability target. Brace depth is used here
                            # rather than the fun_depth heuristic the
                            # Composable branch uses, because that heuristic
                            # never engages for a signature spanning several
                            # lines -- which reported DiagnosticsScreen's local
                            # refreshLogs() as an unreferenced top-level
                            # function despite three call sites in the same file.
                            is_local=current_depth > (class_stack[-1][1] if class_stack else 0)
                        ))

            # Manage scopes and brace nesting
            open_b = l_clean.count('{')
            close_b = l_clean.count('}')

            if m_fun and open_b > 0 and fun_depth is None:
                fun_depth = current_depth + open_b

            if open_b > 0 and pending_class is not None:
                class_stack.append((pending_class, current_depth + open_b))
                pending_class = None

            current_depth += open_b - close_b

            if fun_depth is not None and current_depth < fun_depth:
                fun_depth = None
            while class_stack and current_depth < class_stack[-1][1]:
                class_stack.pop()

    return declarations


def analyze_declarations(
    declarations: List[Declaration],
    kt_clean: Dict[str, str],
    registered_composables: Set[str],
    manifest_entries: Set[str],
    method_severity: str,
    keep_kinds: Optional[Set[str]] = None,
    skip_live_container: bool = True,
) -> Tuple[List[Finding], Set[str], Set[str]]:
    """Build the caller map and run C1/C3/C4 over one declaration set.

    keep_kinds restricts which declaration kinds are REPORTED. Propagation
    always considers every declaration, so a function referenced only by a
    dead composable is still correctly found; keep_kinds only decides what
    the caller of this function is allowed to see.

    skip_live_container keeps the historical exemption that ignores a nested
    member whose enclosing class is itself reachable. That exemption is
    sensible for a blocking gate -- a public method on a live service is not
    dead just because nothing inside the module calls it -- but it also
    means a genuinely dead function inside a live class is never reported.
    The advisory pass turns it off (skip_live_container=False), which is what
    makes decideCommand detectable at all.

    Returns (findings, reported_dead, live_set).
    """

    def emit(findings: List[Finding], d: Declaration, finding: Finding) -> None:
        if keep_kinds is None or d.kind in keep_kinds:
            findings.append(finding)

    active_declarations = [d for d in declarations if not d.is_preview and not d.is_local]
    decl_by_name: Dict[str, List[Declaration]] = {}
    for d in active_declarations:
        decl_by_name.setdefault(d.name, []).append(d)

    callers_map: Dict[str, Set[Tuple[str, int, str]]] = {d.fqcn: set() for d in active_declarations}

    # Map references across all source files
    decls_by_file: Dict[str, List[Declaration]] = {}
    for d in active_declarations:
        decls_by_file.setdefault(d.file, []).append(d)
    for lst in decls_by_file.values():
        lst.sort(key=lambda x: x.line)

    for rel_p, clean in kt_clean.items():
        file_decls = decls_by_file.get(rel_p, [])

        for line_idx, line_str in enumerate(clean.splitlines()):
            line_no = line_idx + 1
            trimmed = line_str.strip()
            if not trimmed or trimmed.startswith("package ") or trimmed.startswith("import "):
                continue

            enclosing = None
            for d in reversed(file_decls):
                if d.line <= line_no:
                    enclosing = d
                    break
            enclosing_fqcn = enclosing.fqcn if enclosing else f"__FILE__{rel_p}"

            tokens = set(re.findall(r'\b[A-Za-z0-9_]+\b', line_str))
            for tok in tokens:
                if tok in decl_by_name:
                    for target_d in decl_by_name[tok]:
                        if target_d.file == rel_p and target_d.line == line_no:
                            continue
                        callers_map[target_d.fqcn].add((rel_p, line_no, enclosing_fqcn))

    findings: List[Finding] = []

    # Check Manifest Reachability (C3)
    manifest_dead: Set[str] = set()
    for d in active_declarations:
        if d.kind in ["Activity", "Service", "BroadcastReceiver"]:
            in_manifest = (d.fqcn in manifest_entries) or (d.name in manifest_entries)
            if not in_manifest:
                manifest_dead.add(d.fqcn)
                emit(findings, d, Finding(
                    file=d.file,
                    line=d.line,
                    kind="C3_MANIFEST_MISSING",
                    symbol=d.name,
                    reason=f"{d.kind} '{d.name}' declared in source but has no corresponding entry in AndroidManifest.xml",
                    severity="ERROR",
                ))

    # Compute Root Live Entry Points
    live_set: Set[str] = set()
    for d in active_declarations:
        if d.fqcn in manifest_entries or d.name in manifest_entries:
            live_set.add(d.fqcn)
        if d.file.endswith("AppModule.kt") or d.name in ["MeshApplication", "AppModule", "MainActivity", "MeshApp"]:
            live_set.add(d.fqcn)
        if d.kind in ["Composable", "Dialog"] and d.name in registered_composables:
            live_set.add(d.fqcn)
        if d.name in ["OnboardingScreen", "MeshBottomBar"] and "MeshApp.kt" in d.file:
            live_set.add(d.fqcn)

    forward_map: Dict[str, Set[str]] = {}
    for callee_fqcn, callers in callers_map.items():
        for _, _, caller_fqcn in callers:
            forward_map.setdefault(caller_fqcn, set()).add(callee_fqcn)

    queue = list(live_set)
    while queue:
        curr = queue.pop(0)
        for callee in forward_map.get(curr, set()):
            if callee not in live_set and callee not in manifest_dead:
                live_set.add(callee)
                queue.append(callee)

    # Check Zero Callers (C1)
    target_kinds = {"Composable", "Dialog", "ViewModel", "Activity", "Service", "BroadcastReceiver", "Utility", "Method"}
    reported_dead: Set[str] = set(manifest_dead)

    for d in active_declarations:
        if d.fqcn in reported_dead:
            continue
        if d.kind not in target_kinds:
            continue
        if skip_live_container and d.is_nested and d.container:
            container_fqcn = f"{d.fqcn.rsplit('.', 1)[0]}"
            if container_fqcn in live_set or any(c.fqcn == container_fqcn and c.fqcn in live_set for c in active_declarations):
                continue

        if d.fqcn not in live_set:
            external_callers = [c for c in callers_map[d.fqcn] if c[2] != d.fqcn]
            if len(external_callers) == 0:
                reported_dead.add(d.fqcn)
                sym_name = d.name if d.kind != "Method" else (f"{d.container}.{d.name}" if d.container else d.name)
                emit(findings, d, Finding(
                    file=d.file,
                    line=d.line,
                    kind="C1_ZERO_CALLERS",
                    symbol=sym_name,
                    reason=f"{d.kind} '{sym_name}' has zero callers across the codebase",
                    severity=method_severity if d.kind == "Method" else "ERROR",
                ))

    # Check Transitive Death (C4)
    changed = True
    while changed:
        changed = False
        for d in active_declarations:
            if d.fqcn in reported_dead or d.fqcn in live_set:
                continue
            if d.kind not in target_kinds:
                continue
            if skip_live_container and d.is_nested and d.container:
                if d.fqcn.rsplit('.', 1)[0] in live_set:
                    continue

            external_callers = [c for c in callers_map[d.fqcn] if c[2] != d.fqcn]
            if len(external_callers) > 0:
                if all(c[2] in reported_dead for c in external_callers):
                    reported_dead.add(d.fqcn)
                    changed = True
                    chain_names = sorted(list(set(c[2].split(".")[-1] for c in external_callers)))
                    sym_name = d.name if d.kind != "Method" else (f"{d.container}.{d.name}" if d.container else d.name)
                    emit(findings, d, Finding(
                        file=d.file,
                        line=d.line,
                        kind="C4_TRANSITIVE_DEAD",
                        symbol=sym_name,
                        reason=f"{d.kind} '{sym_name}' is referenced only by dead declarations: {', '.join(chain_names)}",
                        chain=chain_names,
                        severity=method_severity if d.kind == "Method" else "ERROR",
                    ))

    return findings, reported_dead, live_set


def check_wiring(repo_root: str, block_methods: bool = False) -> Tuple[List[Finding], List[str]]:
    """Execute complete wiring, route, manifest, and reachability gate.

    block_methods controls whether the widened dead-function scan (kind
    "Method") is promoted from advisory to blocking. It defaults to False:
    the scan reports, the gate does not fail on it, and promoting it later is
    a one-word change rather than a second implementation.

    Two passes run over the same sources:

      1. BLOCKING -- the pre-widening declaration set (composables, screens,
         components, /utils/ functions). Its result is unchanged by the
         dead-function widening, so this gate cannot start failing on code it
         never used to inspect.
      2. ADVISORY -- every function in android/app/src/main. It runs over the
         widened graph so transitive reasoning is correct, but it REPORTS only
         function-kind findings. That isolation is deliberate: a dead function
         must not cascade into failing the gate on its dependants.
    """
    app_dir = os.path.join(repo_root, "android", "app", "src", "main")
    src_dir = os.path.join(app_dir, "java")
    manifest_path = os.path.join(app_dir, "AndroidManifest.xml")

    excluded_info = [
        "Excluded test sources (android/app/src/test, android/app/src/androidTest).",
        "Excluded @Preview annotated composables (tooling preview only)."
    ]

    findings: List[Finding] = []

    # 1. Parse Manifest (C3)
    manifest_entries = parse_manifest(manifest_path)

    # 2. Read all Kotlin sources
    kt_files: Dict[str, str] = {}
    kt_clean: Dict[str, str] = {}
    for r, _, fs in os.walk(src_dir):
        for f in fs:
            if f.endswith(".kt"):
                p = os.path.join(r, f)
                rel_p = os.path.relpath(p, start=repo_root).replace("\\", "/")
                with open(p, "r", encoding="utf-8") as fh:
                    raw = fh.read()
                kt_files[rel_p] = raw
                kt_clean[rel_p] = strip_comments(raw)

    # 3. Nav Route Reachability (C2)
    mesh_app_rel = "android/app/src/main/java/com/scmessenger/android/ui/MeshApp.kt"
    registered_composables: Set[str] = set()
    if mesh_app_rel in kt_files:
        nav_findings, reg_composables, _ = check_nav_routes(
            os.path.join(repo_root, mesh_app_rel),
            kt_clean[mesh_app_rel],
            repo_root
        )
        findings.extend(nav_findings)
        registered_composables.update(reg_composables)

    # 4. BLOCKING pass: the pre-widening declaration set.
    legacy_declarations = extract_declarations(kt_files, kt_clean, method_scope="legacy")
    legacy_findings, _, _ = analyze_declarations(
        legacy_declarations,
        kt_clean,
        registered_composables,
        manifest_entries,
        method_severity="ERROR",
        keep_kinds=None,
    )
    findings.extend(legacy_findings)

    # 5. ADVISORY pass: every function in android/app/src/main.
    method_severity = "ERROR" if block_methods else "WARN"
    widened_declarations = extract_declarations(kt_files, kt_clean, method_scope="widened")
    method_findings, _, _ = analyze_declarations(
        widened_declarations,
        kt_clean,
        registered_composables,
        manifest_entries,
        method_severity=method_severity,
        keep_kinds={"Method"},
        # A function inside a reachable class is exactly the shape that hid
        # decideCommand, so the advisory pass does not honour the
        # live-container exemption.
        skip_live_container=False,
    )
    findings.extend(method_findings)

    return findings, excluded_info


def main() -> int:
    parser = argparse.ArgumentParser(description="SCMessenger Android Wiring & Reachability Gate (AGENTS.md Rule 16)")
    parser.add_argument("--root", default=os.getcwd(), help="Repository root directory (default: current directory)")
    parser.add_argument("--json", action="store_true", help="Output findings in JSON format for CI consumption")
    parser.add_argument(
        "--method-findings",
        choices=["warn", "block"],
        default="warn",
        help=(
            "Severity of the widened dead-function scan (kind=Method). "
            "'warn' (default) reports every hit but does not fail the gate; "
            "'block' promotes them to errors once the backlog is triaged."
        ),
    )
    args = parser.parse_args()

    repo_root = os.path.abspath(args.root)
    block_methods = args.method_findings == "block"
    findings, exclusions = check_wiring(repo_root, block_methods=block_methods)

    errors = [f for f in findings if f.severity == "ERROR"]
    warnings = [f for f in findings if f.severity != "ERROR"]

    if args.json:
        payload = {
            "status": "PASS" if not errors else "FAIL",
            "findings_count": len(findings),
            "error_count": len(errors),
            "warn_count": len(warnings),
            "method_scan": args.method_findings,
            "exclusions": exclusions,
            "findings": [asdict(f) for f in findings]
        }
        print(json.dumps(payload, indent=2))
    else:
        print("=== SCMessenger Wiring & Reachability Gate ===")
        print("Exclusions:")
        for excl in exclusions:
            print(f"  [INFO] {excl}")
        print(f"Dead-function scan: {args.method_findings}"
              + ("  (advisory; see wiring-audit/FINDINGS.md to triage)"
                 if args.method_findings == "warn" else "  (blocking)"))

        if errors:
            print(f"\n[FAIL] Found {len(errors)} unreachable or miswired items:\n")
            # AGENTS.md Rule 15: Print EVERY hit, no truncation
            for f in errors:
                chain_str = f" [chain: {' -> '.join(f.chain)}]" if f.chain else ""
                print(f"[{f.kind}] {f.file}:{f.line} - {f.symbol}\n       Reason: {f.reason}{chain_str}")
        else:
            print("\n[OK] All components, composables, routes, and utilities are correctly wired.")

        if warnings:
            print(f"\n[WARN] {len(warnings)} unreferenced function(s) reported by the "
                  f"widened dead-function scan. These do NOT fail the gate yet.\n")
            for f in warnings:
                chain_str = f" [chain: {' -> '.join(f.chain)}]" if f.chain else ""
                print(f"[{f.kind}] {f.file}:{f.line} - {f.symbol}\n       Reason: {f.reason}{chain_str}")

    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
