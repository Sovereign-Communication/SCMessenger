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
    def _detect_platform(file_path: str) -> str:
        if file_path.startswith("android/"):
            return "android"
        elif file_path.startswith("iOS/"):
            return "ios"
        elif file_path.startswith("core/src"):
            return "core"
        elif file_path.startswith("cli/src"):
            return "cli"
        elif file_path.startswith("desktop_bridge/src"):
            return "desktop"
        return "unknown"
    
    @staticmethod
    def parse_rust(content: str, file_path: str) -> List[FunctionInfo]:
        functions = []
        lines = content.split('\n')
        
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
                
                # Find opening brace
                brace_line = i
                while brace_line < len(lines) and '{' not in lines[brace_line]:
                    brace_line += 1
                    if brace_line - i > 10:
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
        self.url = LM_STUDIO_URL
        self.model = LM_STUDIO_MODEL
        self.timeout = 120
    
    def audit_function(self, func: FunctionInfo, pass_type: str) -> List[AuditIssue]:
        """Audit a single function with specified pass type"""
        prompt = self._build_prompt(func, pass_type)
        
        payload = {
            "model": self.model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.0,
            "max_tokens": 4096,
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
                    description=result.stderr[:200], code_snippet="",
                    pass_type=pass_type
                )]
            
            response = json.loads(result.stdout)
            content = response.get('choices', [{}])[0].get('message', {}).get('content', '')
            return self._parse_response(content, func, pass_type)
            
        except subprocess.TimeoutExpired:
            return [AuditIssue(
                file=func.file, function=func.function_name, line=func.start_line,
                severity="info", category="error", title="LM Studio timeout",
                description=f"Request timed out after {self.timeout}s", code_snippet="",
                pass_type=pass_type
            )]
        except Exception as e:
            return [AuditIssue(
                file=func.file, function=func.function_name, line=func.start_line,
                severity="info", category="error", title="Audit error",
                description=str(e)[:200], code_snippet="",
                pass_type=pass_type
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

FIND ALL ISSUES (even minor). Categories:
- todo: TODO/FIXME/XXX/HACK/placeholder comments
- magic_number: Hardcoded numbers without constants
- naming: Inconsistent/confusing names
- unsafe: unwrap(), expect(), panic!, unsafe blocks, force unwraps
- error_handling: Ignored Results/Options, map_err(|_|), bare catches
- dead_code: Unused code, #[allow(dead_code)] on public items
- incomplete: unimplemented!(), todo!(), panic!("not implemented")
- testing: Missing tests for public functions
- performance: clone() in hot path, String in loops, unbounded collections
- thread_safety: !Send/!Sync in async, data races, lock ordering
- api_design: >5 params, tuple returns, public fields
- crypto: Hardcoded keys, weak randomness, missing zeroize
- architecture: Circular deps, god objects, missing abstractions
- docs: Missing public API docs, outdated comments

OUTPUT: One JSON object per line ONLY:
{{"file":"{func.file}","function":"{func.function_name}","line":{func.start_line},"severity":"critical|high|medium|low|info","category":"<cat>","title":"<title>","description":"<desc>","code_snippet":"<code>","suggestion":"<fix>","pass_type":"quality"}}

If no issues, output nothing."""
        
        else:  # parity pass
            return f"""You are a platform parity auditor. Analyze this {func.language} function for iOS/ANDROID PARITY issues.

FUNCTION: {func.function_name} in {func.file}:{func.start_line}-{func.end_line}
SIGNATURE: {func.signature}
PLATFORM: {func.platform}

CODE:
```{func.language}
{func.content[:6000]}
```

FIND PARITY ISSUES between iOS and Android implementations. Categories:
- missing_method: Method exists on one platform but not the other
- signature_mismatch: Same method name but different params/return types
- async_mismatch: suspend vs async/await vs callbacks for same operation
- error_handling_mismatch: Different error handling patterns
- naming_mismatch: Different names for same concept across platforms
- behavior_mismatch: Same method but different behavior/side effects
- platform_specific_leak: Platform-specific types in shared interfaces

OUTPUT: One JSON object per line ONLY:
{{"file":"{func.file}","function":"{func.function_name}","line":{func.start_line},"severity":"critical|high|medium|low|info","category":"<cat>","title":"<title>","description":"<desc>","code_snippet":"<code>","suggestion":"<fix>","pass_type":"parity"}}

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
    RESULTS_FILE.write_text(
        RESULTS_FILE.read_text() + json.dumps(asdict(issue)) + '\n' if RESULTS_FILE.exists() 
        else json.dumps(asdict(issue)) + '\n'
    )


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
            total_functions=len(all_functions)
        )
        save_state(state)
    
    # Filter remaining
    completed = set(state.completed_functions)
    remaining = [f for f in all_functions if f"{f.file}::{f.function_name}" not in completed]
    print(f"Remaining: {len(remaining)} functions")
    
    if not remaining:
        print("All functions already audited!")
        return
    
    # Audit loop - one function at a time, dual pass
    client = LMStudioClient()
    
    for idx, func in enumerate(remaining):
        func_id = f"{func.file}::{func.function_name}"
        state.current_file = func.file
        state.current_function = func.function_name
        
        print(f"\n[{idx+1}/{len(remaining)}] {func.file}::{func.function_name} (lines {func.start_line}-{func.end_line}) [{func.platform}]")
        
        # PASS 1: Code Quality
        quality_issues = client.audit_function(func, "quality")
        for issue in quality_issues:
            append_result(issue)
            state.total_issues += 1
        
        # PASS 2: Parity (only for platform-specific code)
        parity_issues = client.audit_function(func, "parity")
        for issue in parity_issues:
            append_result(issue)
            state.total_issues += 1
        
        # Save checkpoint
        state.completed_functions.append(func_id)
        state.current_index = idx + 1
        save_state(state)
        
        # Progress
        total = len(quality_issues) + len(parity_issues)
        if total > 0:
            print(f"  Found {len(quality_issues)} quality + {len(parity_issues)} parity = {total} issues")
            for issue in (quality_issues + parity_issues)[:3]:
                print(f"    [{issue.severity}] {issue.category}: {issue.title}")
            if total > 3:
                print(f"    ... and {total - 3} more")
        else:
            print(f"  No issues found")
        
        # Small delay to prevent rate limiting
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
    
    # Group by pass type and severity
    quality_by_sev = {}
    parity_by_sev = {}
    quality_by_cat = {}
    parity_by_cat = {}
    
    for issue in issues:
        if issue.pass_type == "quality":
            quality_by_sev.setdefault(issue.severity, []).append(issue)
            quality_by_cat.setdefault(issue.category, []).append(issue)
        else:
            parity_by_sev.setdefault(issue.severity, []).append(issue)
            parity_by_cat.setdefault(issue.category, []).append(issue)
    
    with open(REPORT_FILE, 'w') as f:
        f.write("# SCMessenger Dual-Pass Audit Report\n\n")
        f.write(f"**Generated:** {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n")
        f.write(f"**Model:** {LM_STUDIO_MODEL} (LM Studio)\n")
        f.write(f"**Functions Audited:** {len(state.completed_functions)}\n")
        f.write(f"**Total Issues:** {state.total_issues}\n\n")
        
        # Quality pass summary
        f.write("## Code Quality Pass\n\n")
        for sev in ['critical', 'high', 'medium', 'low', 'info']:
            count = len(quality_by_sev.get(sev, []))
            f.write(f"- **{sev.upper()}:** {count}\n")
        f.write("\n")
        
        f.write("### By Category\n")
        for cat in sorted(quality_by_cat.keys()):
            f.write(f"- **{cat}:** {len(quality_by_cat[cat])}\n")
        f.write("\n")
        
        # Parity pass summary
        f.write("## iOS/Android Parity Pass\n\n")
        for sev in ['critical', 'high', 'medium', 'low', 'info']:
            count = len(parity_by_sev.get(sev, []))
            f.write(f"- **{sev.upper()}:** {count}\n")
        f.write("\n")
        
        f.write("### By Category\n")
        for cat in sorted(parity_by_cat.keys()):
            f.write(f"- **{cat}:** {len(parity_by_cat[cat])}\n")
        f.write("\n")
        
        # Detailed issues
        f.write("## Critical & High Severity Issues\n\n")
        for pass_type, by_sev in [("Quality", quality_by_sev), ("Parity", parity_by_sev)]:
            for sev in ['critical', 'high']:
                for issue in by_sev.get(sev, []):
                    f.write(f"\n### {issue.pass_type}: {issue.file}:{issue.line} in `{issue.function}`\n")
                    f.write(f"**Category:** {issue.category} | **Severity:** {issue.severity}\n\n")
                    f.write(f"**Title:** {issue.title}\n\n")
                    f.write(f"{issue.description}\n\n")
                    if issue.code_snippet:
                        f.write(f"```{issue.file.split('.')[-1]}\n{issue.code_snippet}\n```\n\n")
                    if issue.suggestion:
                        f.write(f"**Suggestion:** {issue.suggestion}\n\n")
                    f.write("---\n\n")


if __name__ == '__main__':
    run_audit()