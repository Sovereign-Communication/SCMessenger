#!/usr/bin/env python3
"""
SCMessenger Dual-Pass Function-Level Audit System (LM Studio)
- Parses files into individual functions
- Pass 1: Code quality audit (bugs, style, security, performance, etc.)
- Pass 2: iOS/Android parity audit (missing methods, mismatched signatures)
- Saves state after EVERY function
- Resumes from checkpoint
- Zero parallelization until proven working
"""

import os
import json
import subprocess
import sys
import time
import re
from pathlib import Path
from typing import List, Dict, Any, Optional, Tuple
from dataclasses import dataclass, asdict, field
from datetime import datetime

# ============ CONFIGURATION ============
LM_STUDIO_URL = "http://localhost:1234/v1/chat/completions"
LM_STUDIO_MODEL = "gemma-4-e4b-instruct"  # Already loaded in LM Studio
MAX_CONTEXT = 8192
REPO_ROOT = Path(r"C:\Users\SCM\Documents\GitHub\SCMessenger")
AUDIT_DIR = REPO_ROOT / "audit_system" / "results_dualpass"
STATE_FILE = AUDIT_DIR / "audit_state.json"
RESULTS_FILE = AUDIT_DIR / "audit_results.jsonl"
REPORT_FILE = AUDIT_DIR / "AUDIT_REPORT.md"

# Priority directories to audit
PRIORITY_DIRS = [
    "core/src",
    "android/app/src/main/java/com/scmessenger/android",
    "iOS/SCMessenger/SCMessenger",
    "cli/src",
    "desktop_bridge/src",
]

# Extensions to audit
AUDIT_EXTS = {'.rs', '.kt', '.swift'}

# Skip directories
SKIP_DIRS = {
    'target', 'build', '.git', 'node_modules', '__pycache__',
    '.gradle', 'dist', 'out', 'bin', 'obj', 'tmp', 'scratch',
    'SCMessengerCore.xcframework', '.claude', '.agents', '.bob',
    '.codex', '.github', '.cargo', 'rustup-init.exe'
}

AUDIT_DIR.mkdir(parents=True, exist_ok=True)


# ============ DATA CLASSES ============
@dataclass
class FunctionInfo:
    file: str
    function_name: str
    start_line: int
    end_line: int
    content: str
    language: str
    signature: str = ""
    platform: str = ""  # "android", "ios", "core", "cli", "desktop"


@dataclass
class AuditIssue:
    file: str
    function: str
    line: int
    severity: str
    category: str
    title: str
    description: str
    code_snippet: str
    suggestion: str = ""
    pass_type: str = "quality"  # "quality" or "parity"
    timestamp: str = field(default_factory=lambda: datetime.now().isoformat())


@dataclass
class AuditState:
    completed_functions: List[str] = field(default_factory=list)  # "file::function"
    current_file: str = ""
    current_function: str = ""
    total_functions: int = 0
    total_issues: int = 0
    start_time: str = field(default_factory=lambda: datetime.now().isoformat())
    last_update: str = field(default_factory=lambda: datetime.now().isoformat())
    errors: List[str] = field(default_factory=list)


# ============ FUNCTION PARSERS ============
class FunctionParser:
    """Extract individual functions from source files"""
    
    @staticmethod
    def parse_rust(content: str, file_path: str) -> List[FunctionInfo]:
        functions = []
        lines = content.split('\n')
        
        # Pattern for Rust functions: fn name(...) { ... }
        fn_pattern = re.compile(
            r'^\s*(?:(?:pub|async|const|unsafe|extern)\s+)*fn\s+(\w+)\s*(?:<.*?>)?\s*\([^)]*\)'
        )
        
        i = 0
        while i < len(lines):
            line = lines[i]
            match = fn_pattern.match(line)
            if match:
                fn_name = match.group(1)
                start_line = i + 1
                
                # Find the opening brace
                brace_line = i
                while brace_line < len(lines) and '{' not in lines[brace_line]:
                    brace_line += 1
                    if brace_line - i > 10:
                        break
                
                if brace_line >= len(lines) or '{' not in lines[brace_line]:
                    i += 1
                    continue
                
                # Count braces to find end
                brace_count = 0
                end_line = brace_line
                for j in range(brace_line, len(lines)):
                    brace_count += lines[j].count('{')
                    brace_count -= lines[j].count('}')
                    if brace_count == 0 and j >= brace_line:
                        end_line = j
                        break
                
                fn_content = '\n'.join(lines[start_line-1:end_line+1])
                platform = FunctionParser._detect_platform(file_path)
                functions.append(FunctionInfo(
                    file=file_path,
                    function_name=fn_name,
                    start_line=start_line,
                    end_line=end_line + 1,
                    content=fn_content,
                    language='rust',
                    signature=line.strip(),
                    platform=platform
                ))
                i = end_line + 1
            else:
                i += 1
        
        return functions
    
    @staticmethod
    def parse_kotlin(content: str, file_path: str) -> List[FunctionInfo]:
        functions = []
        lines = content.split('\n')
        
        fn_pattern = re.compile(
            r'^\s*(?:(?:public|private|protected|internal|inline|tailrec|suspend|operator|infix|expect|actual)\s+)*fun\s+(?:<\w+>\s+)?(?:\w+\.)?(\w+)\s*(?:<.*?>)?\s*\([^)]*\)'
        )
        
        i = 0
        while i < len(lines):
            line = lines[i]
            match = fn_pattern.match(line)
            if match:
                fn_name = match.group(1)
                start_line = i + 1
                
                # Find opening brace
                brace_line = i
                while brace_line < len(lines) and '{' not in lines[brace_line]:
                    brace_line += 1
                    if brace_line - i > 15:
                        break
                
                if brace_line >= len(lines) or '{' not in lines[brace_line]:
                    # Single expression function
                    if '=' in line and '{' not in line:
                        platform = FunctionParser._detect_platform(file_path)
                        functions.append(FunctionInfo(
                            file=file_path,
                            function_name=fn_name,
                            start_line=start_line,
                            end_line=start_line,
                            content=line.strip(),
                            language='kotlin',
                            signature=line.strip(),
                            platform=platform
                        ))
                    i += 1
                    continue
                
                # Count braces
                brace_count = 0
                end_line = brace_line
                for j in range(brace_line, len(lines)):
                    brace_count += lines[j].count('{')
                    brace_count -= lines[j].count('}')
                    if brace_count == 0 and j >= brace_line:
                        end_line = j
                        break
                
                fn_content = '\n'.join(lines[start_line-1:end_line+1])
                platform = FunctionParser._detect_platform(file_path)
                functions.append(FunctionInfo(
                    file=file_path,
                    function_name=fn_name,
                    start_line=start_line,
                    end_line=end_line + 1,
                    content=fn_content,
                    language='kotlin',
                    signature=line.strip(),
                    platform=platform
                ))
                i = end_line + 1
            else:
                i += 1
        
        return functions
    
    @staticmethod
    def parse_swift(content: str, file_path: str) -> List[FunctionInfo]:
        functions = []
        lines = content.split('\n')
        
        fn_pattern = re.compile(
            r'^\s*(?:(?:public|private|internal|fileprivate|open|static|class|final|override|mutating|async|throws|rethrows)\s+)*func\s+(\w+)\s*(?:<.*?>)?\s*\([^)]*\)'
        )
        
        i = 0
        while i < len(lines):
            line = lines[i]
            match = fn_pattern.match(line)
            if match:
                fn_name = match.group(1)
                start_line = i + 1
                
                # Find opening brace
                brace_line = i
                while brace_line < len(lines) and '{' not in lines[brace_line]:
                    brace_line += 1
                    if brace_line - i > 15:
                        break
                
                if brace_line >= len(lines) or '{' not in lines[brace_line]:
                    i += 1
                    continue
                
                # Count braces
                brace_count = 0
                end_line = brace_line
                for j in range(brace_line, len(lines)):
                    brace_count += lines[j].count('{')
                    brace_count -= lines[j].count('}')
                    if brace_count == 0 and j >= brace_line:
                        end_line = j
                        break
                
                fn_content = '\n'.join(lines[start_line-1:end_line+1])
                platform = FunctionParser._detect_platform(file_path)
                functions.append(FunctionInfo(
                    file=file_path,
                    function_name=fn_name,
                    start_line=start_line,
                    end_line=end_line + 1,
                    content=fn_content,
                    language='swift',
                    signature=line.strip(),
                    platform=platform
                ))
                i = end_line + 1
            else:
                i += 1
        
        return functions
    
    @staticmethod
    def _detect_platform(file_path: str) -> str:
        if "android" in file_path:
            return "android"
        elif "iOS" in file_path or "iOS" in file_path:
            return "ios"
        elif "cli" in file_path:
            return "cli"
        elif "desktop_bridge" in file_path:
            return "desktop"
        return "core"
    
    @staticmethod
    def parse_file(file_path: Path) -> List[FunctionInfo]:
        try:
            content = file_path.read_text(encoding='utf-8')
        except UnicodeDecodeError:
            try:
                content = file_path.read_text(encoding='latin-1')
            except:
                return []
        except:
            return []
        
        rel_path = str(file_path.relative_to(REPO_ROOT))
        ext = file_path.suffix.lower()
        
        if ext == '.rs':
            return FunctionParser.parse_rust(content, rel_path)
        elif ext == '.kt':
            return FunctionParser.parse_kotlin(content, rel_path)
        elif ext == '.swift':
            return FunctionParser.parse_swift(content, rel_path)
        return []


# ============ LM STUDIO CLIENT ============
class LMStudioClient:
    def __init__(self):
        self.model = LM_STUDIO_MODEL
        self.url = LM_STUDIO_URL
        self.timeout = 90
    
    def audit_function(self, func: FunctionInfo, pass_type: str) -> List[AuditIssue]:
        """Audit a single function for either quality or parity"""
        prompt = self._build_prompt(func, pass_type)
        
        payload = {
            "model": self.model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.0,
            "max_tokens": 2048,
            "top_p": 0.9
        }
        
        try:
            result = subprocess.run(
                ['curl', '-s', '-X', 'POST', self.url, 
                 '-H', 'Content-Type: application/json',
                 '-d', json.dumps(payload)],
                capture_output=True, text=True, timeout=self.timeout
            )
            
            if result.returncode != 0:
                return [AuditIssue(
                    file=func.file, function=func.function_name, line=func.start_line,
                    severity="info", category="error", title="LM Studio call failed",
                    description=result.stderr[:200], code_snippet="", pass_type=pass_type
                )]
            
            response_data = json.loads(result.stdout)
            response = response_data.get('choices', [{}])[0].get('message', {}).get('content', '')
            return self._parse_response(response, func, pass_type)
            
        except subprocess.TimeoutExpired:
            return [AuditIssue(
                file=func.file, function=func.function_name, line=func.start_line,
                severity="info", category="error", title="LM Studio timeout",
                description="Request timed out after 90s", code_snippet="", pass_type=pass_type
            )]
        except Exception as e:
            return [AuditIssue(
                file=func.file, function=func.function_name, line=func.start_line,
                severity="info", category="error", title="Audit error",
                description=str(e)[:200], code_snippet="", pass_type=pass_type
            )]
    
    def _build_prompt(self, func: FunctionInfo, pass_type: str) -> str:
        if pass_type == "quality":
            return f"""You are a senior code auditor. Analyze this {func.language} function for CODE QUALITY issues.

FUNCTION: {func.function_name} in {func.file}:{func.start_line}-{func.end_line}
SIGNATURE: {func.signature}
PLATFORM: {func.platform}

CODE:
```{func.language}
{func.content[:6000]}
```

FIND ALL CODE QUALITY ISSUES (even minor). Categories:
- todo: TODO/FIXME/XXX/HACK/placeholder comments
- magic_number: Hardcoded numbers without constants
- naming: Inconsistent/confusing names
- unsafe: unwrap(), expect(), panic!, unsafe blocks, force unwraps (!), .unwrap() in production
- error_handling: Ignored Results/Options, map_err(|_|), bare catches, missing throws
- dead_code: Unused code, #[allow(dead_code)] on public items
- incomplete: unimplemented!(), todo!(), panic!("not implemented")
- testing: Missing tests for public functions
- performance: clone() in hot path, String in loops, unbounded collections
- thread_safety: !Send/!Sync in async, data races, lock ordering issues
- api_design: >5 params, tuple returns, public fields
- crypto: Hardcoded keys, weak randomness, missing zeroize
- architecture: Circular deps, god objects, missing abstractions
- docs: Missing public API docs, outdated comments

OUTPUT: One JSON object per line ONLY:
{{"file":"{func.file}","function":"{func.function_name}","line":{func.start_line},"severity":"critical|high|medium|low|info","category":"<cat>","title":"<title>","description":"<desc>","code_snippet":"<code>","suggestion":"<fix>","pass_type":"quality"}}

If no issues, output nothing."""
        
        else:  # parity pass
            return f"""You are a senior mobile architect. Analyze this {func.language} function for iOS/ANDROID PARITY issues.

FUNCTION: {func.function_name} in {func.file}:{func.start_line}-{func.end_line}
SIGNATURE: {func.signature}
PLATFORM: {func.platform}

CODE:
```{func.language}
{func.content[:6000]}
```

FIND PARITY ISSUES by comparing to the other platform's equivalent. Categories:
- missing_method: Method exists on one platform but not the other
- signature_mismatch: Same method name but different params/return types
- async_mismatch: suspend (Android) vs async/await (iOS) vs sync (core)
- error_mismatch: Result<T, E> vs throws vs Result return
- naming_inconsistency: Different names for same concept across platforms
- missing_annotation: Missing @MainActor, @Throws, etc.
- missing_feature: Feature implemented on one platform but not the other
- delegate_mismatch: Different delegate/callback patterns

OUTPUT: One JSON object per line ONLY:
{{"file":"{func.file}","function":"{func.function_name}","line":{func.start_line},"severity":"critical|high|medium|low|info","category":"parity","title":"<title>","description":"<desc>","code_snippet":"<code>","suggestion":"<fix>","pass_type":"parity"}}

If no parity issues, output nothing."""
    
    def _parse_response(self, response: str, func: FunctionInfo, pass_type: str) -> List[AuditIssue]:
        issues = []
        for line in response.strip().split('\n'):
            line = line.strip()
            if not line or not line.startswith('{'):
                continue
            try:
                data = json.loads(line)
                data.setdefault('file', func.file)
                data.setdefault('function', func.function_name)
                data.setdefault('line', func.start_line)
                data.setdefault('pass_type', pass_type)
                issues.append(AuditIssue(**data))
            except json.JSONDecodeError:
                pass
        return issues


# ============ STATE MANAGEMENT ============
def load_state() -> Optional[AuditState]:
    if STATE_FILE.exists():
        try:
            data = json.loads(STATE_FILE.read_text())
            return AuditState(**data)
        except:
            pass
    return None


def save_state(state: AuditState):
    state.last_update = datetime.now().isoformat()
    STATE_FILE.write_text(json.dumps(asdict(state), indent=2))


def append_result(issue: AuditIssue):
    with open(RESULTS_FILE, 'a') as f:
        f.write(json.dumps(asdict(issue)) + '\n')


# ============ MAIN AUDIT LOOP ============
def discover_all_functions() -> List[FunctionInfo]:
    """Find all functions in priority directories"""
    all_functions = []
    
    for priority_dir in PRIORITY_DIRS:
        dir_path = REPO_ROOT / priority_dir
        if not dir_path.exists():
            continue
        
        for ext in AUDIT_EXTS:
            for file_path in dir_path.rglob(f'*{ext}'):
                if any(skip in file_path.parts for skip in SKIP_DIRS):
                    continue
                
                funcs = FunctionParser.parse_file(file_path)
                all_functions.extend(funcs)
    
    return all_functions


def run_audit():
    print("=" * 60)
    print("SCMessenger Dual-Pass Function Audit (LM Studio)")
    print("=" * 60)
    
    # Load or create state
    state = load_state()
    
    if state and state.completed_functions:
        print(f"Resuming from checkpoint: {len(state.completed_functions)} functions completed")
        print(f"Issues found so far: {state.total_issues}")
        
        # Reconstruct function list
        all_functions = discover_all_functions()
    else:
        print("Discovering all functions in priority directories...")
        all_functions = discover_all_functions()
        print(f"Found {len(all_functions)} functions to audit")
        
        state = AuditState(
            completed_functions=[],
            total_functions=len(all_functions)
        )
        save_state(state)
    
    # Build completed set
    completed = set(state.completed_functions)
    
    # Filter to remaining
    remaining = [f for f in all_functions if f"{f.file}::{f.function_name}" not in completed]
    print(f"Remaining: {len(remaining)} functions")
    
    if not remaining:
        print("All functions already audited!")
        return
    
    # Dual-pass audit loop
    client = LMStudioClient()
    
    for idx, func in enumerate(remaining):
        func_id = f"{func.file}::{func.function_name}"
        state.current_file = func.file
        state.current_function = func.function_name
        
        print(f"\n[{idx+1}/{len(remaining)}] {func.file}::{func.function_name} (lines {func.start_line}-{func.end_line}) [{func.platform}]")
        
        # ===== PASS 1: CODE QUALITY =====
        print("  Pass 1: Code Quality...", end=" ", flush=True)
        start = time.time()
        quality_issues = client.audit_function(func, "quality")
        elapsed = time.time() - start
        
        for issue in quality_issues:
            append_result(issue)
            state.total_issues += 1
        
        print(f"done ({len(quality_issues)} issues, {elapsed:.1f}s)")
        
        # ===== PASS 2: iOS/ANDROID PARITY =====
        print("  Pass 2: Parity...", end=" ", flush=True)
        start = time.time()
        parity_issues = client.audit_function(func, "parity")
        elapsed = time.time() - start
        
        for issue in parity_issues:
            append_result(issue)
            state.total_issues += 1
        
        print(f"done ({len(parity_issues)} issues, {elapsed:.1f}s)")
        
        # Checkpoint
        state.completed_functions.append(func_id)
        state.current_index = idx
        save_state(state)
        
        # Progress update every 10 functions
        if (idx + 1) % 10 == 0:
            elapsed_total = time.time() - time.mktime(datetime.fromisoformat(state.start_time).timetuple())
            rate = (idx + 1) / elapsed_total if elapsed_total > 0 else 0
            eta = (len(remaining) - idx - 1) / rate if rate > 0 else 0
            print(f"  Progress: {idx+1}/{len(remaining)} | {state.total_issues} total issues | {rate:.1f} func/s | ETA: {eta/60:.1f}min")
        
        # Small delay to prevent overwhelming
        time.sleep(0.5)
    
    # Final report
    generate_final_report(state)
    print("\n" + "=" * 60)
    print("AUDIT COMPLETE")
    print("=" * 60)
    print(f"Total functions: {state.total_functions}")
    print(f"Completed: {len(state.completed_functions)}")
    print(f"Total issues: {state.total_issues}")
    print(f"Report: {REPORT_FILE}")


def generate_final_report(state: AuditState):
    """Generate markdown report from results"""
    issues = []
    if RESULTS_FILE.exists():
        for line in RESULTS_FILE.read_text().strip().split('\n'):
            if line.strip():
                try:
                    issues.append(AuditIssue(**json.loads(line)))
                except:
                    pass
    
    # Group by severity and category
    by_sev = {}
    by_cat = {}
    by_file = {}
    by_pass = {}
    
    for issue in issues:
        by_sev.setdefault(issue.severity, []).append(issue)
        by_cat.setdefault(issue.category, []).append(issue)
        by_file.setdefault(issue.file, []).append(issue)
        by_pass.setdefault(issue.pass_type, []).append(issue)
    
    with open(REPORT_FILE, 'w') as f:
        f.write("# SCMessenger Dual-Pass Function Audit Report\n\n")
        f.write(f"**Generated:** {datetime.now().isoformat()}\n")
        f.write(f"**Model:** {LM_STUDIO_MODEL}\n")
        f.write(f"**Functions Audited:** {len(state.completed_functions)}/{state.total_functions}\n")
        f.write(f"**Total Issues:** {state.total_issues}\n\n")
        
        f.write("## By Severity\n\n")
        for sev in ['critical', 'high', 'medium', 'low', 'info']:
            count = len(by_sev.get(sev, []))
            f.write(f"- **{sev.upper()}:** {count}\n")
        f.write("\n")
        
        f.write("## By Category\n\n")
        for cat in sorted(by_cat.keys()):
            f.write(f"- **{cat}:** {len(by_cat[cat])}\n")
        f.write("\n")
        
        f.write("## By Pass Type\n\n")
        for pt in ['quality', 'parity']:
            count = len(by_pass.get(pt, []))
            f.write(f"- **{pt}:** {count}\n")
        f.write("\n")
        
        f.write("## Top Files by Issue Count\n\n")
        for file, file_issues in sorted(by_file.items(), key=lambda x: -len(x[1]))[:30]:
            sev_counts = {}
            pass_counts = {}
            for i in file_issues:
                sev_counts[i.severity] = sev_counts.get(i.severity, 0) + 1
                pass_counts[i.pass_type] = pass_counts.get(i.pass_type, 0) + 1
            sev_str = ', '.join(f"{k}:{v}" for k,v in sorted(sev_counts.items()))
            pass_str = ', '.join(f"{k}:{v}" for k,v in sorted(pass_counts.items()))
            f.write(f"- `{file}`: {len(file_issues)} issues ({sev_str}) [{pass_str}]\n")
        f.write("\n")
        
        # Critical and High issues detail
        f.write("## Critical & High Severity Issues\n\n")
        for sev in ['critical', 'high']:
            for issue in by_sev.get(sev, [])[:100]:
                f.write(f"### {issue.file} :: {issue.function} (line {issue.line})\n")
                f.write(f"**Pass:** {issue.pass_type} | **Category:** {issue.category} | **Severity:** {issue.severity}\n\n")
                f.write(f"**Title:** {issue.title}\n\n")
                f.write(f"{issue.description}\n\n")
                if issue.code_snippet:
                    lang = issue.function.split('::')[-1] if '::' in issue.function else 'text'
                    f.write(f"```{lang}\n{issue.code_snippet}\n```\n\n")
                if issue.suggestion:
                    f.write(f"**Suggestion:** {issue.suggestion}\n\n")
                f.write("---\n\n")


if __name__ == '__main__':
    run_audit()