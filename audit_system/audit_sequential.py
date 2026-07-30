#!/usr/bin/env python3
"""
SCMessenger Sequential Function-Level Audit System
- Parses files into individual functions
- Audits each function sequentially with local Ollama
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
OLLAMA_MODEL = "qwen2.5-coder:7b"
OLLAMA_URL = "http://localhost:11434/api/generate"
MAX_CONTEXT = 4096
REPO_ROOT = Path(r"C:\Users\SCM\Documents\GitHub\SCMessenger")
AUDIT_DIR = REPO_ROOT / "audit_system" / "results_sequential"
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
        # Also handles: pub fn, async fn, const fn, unsafe fn, etc.
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
                    if brace_line - i > 10:  # Not a function if no brace within 10 lines
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
                functions.append(FunctionInfo(
                    file=file_path,
                    function_name=fn_name,
                    start_line=start_line,
                    end_line=end_line + 1,
                    content=fn_content,
                    language='rust',
                    signature=line.strip()
                ))
                i = end_line + 1
            else:
                i += 1
        
        return functions
    
    @staticmethod
    def parse_kotlin(content: str, file_path: str) -> List[FunctionInfo]:
        functions = []
        lines = content.split('\n')
        
        # Kotlin function patterns: fun name(...), fun Class.name(...), etc.
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
                    # Single expression function: fun foo() = expr
                    if '=' in line and '{' not in line:
                        functions.append(FunctionInfo(
                            file=file_path,
                            function_name=fn_name,
                            start_line=start_line,
                            end_line=start_line,
                            content=line.strip(),
                            language='kotlin',
                            signature=line.strip()
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
                functions.append(FunctionInfo(
                    file=file_path,
                    function_name=fn_name,
                    start_line=start_line,
                    end_line=end_line + 1,
                    content=fn_content,
                    language='kotlin',
                    signature=line.strip()
                ))
                i = end_line + 1
            else:
                i += 1
        
        return functions
    
    @staticmethod
    def parse_swift(content: str, file_path: str) -> List[FunctionInfo]:
        functions = []
        lines = content.split('\n')
        
        # Swift function patterns: func name(...) { ... }
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
                functions.append(FunctionInfo(
                    file=file_path,
                    function_name=fn_name,
                    start_line=start_line,
                    end_line=end_line + 1,
                    content=fn_content,
                    language='swift',
                    signature=line.strip()
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


# ============ OLLAMA CLIENT ============
class OllamaClient:
    def __init__(self):
        self.model = OLLAMA_MODEL
        self.url = OLLAMA_URL
        self.timeout = 60  # Reduced from 90
    
    def audit_function(self, func: FunctionInfo) -> List[AuditIssue]:
        """Audit a single function"""
        prompt = self._build_prompt(func)
        
        payload = {
            "model": self.model,
            "prompt": prompt,
            "stream": False,
            "options": {
                "temperature": 0.0,
                "num_ctx": MAX_CONTEXT,
                "top_p": 0.9,
                "repeat_penalty": 1.1
            }
        }
        
        try:
            result = subprocess.run(
                ['curl', '-s', '-X', 'POST', self.url, 
                 '-H', 'Content-Type: application/json',
                 '-d', json.dumps(payload)],
                capture_output=True, text=True, timeout=120
            )
            
            if result.returncode != 0:
                return [AuditIssue(
                    file=func.file, function=func.function_name, line=func.start_line,
                    severity="info", category="error", title="Ollama call failed",
                    description=result.stderr[:200], code_snippet=""
                )]
            
            response = json.loads(result.stdout).get('response', '')
            return self._parse_response(response, func)
            
        except subprocess.TimeoutExpired:
            return [AuditIssue(
                file=func.file, function=func.function_name, line=func.start_line,
                severity="info", category="error", title="Ollama timeout",
                description="Request timed out after 120s", code_snippet=""
            )]
        except Exception as e:
            return [AuditIssue(
                file=func.file, function=func.function_name, line=func.start_line,
                severity="info", category="error", title="Audit error",
                description=str(e)[:200], code_snippet=""
            )]
    
    def _build_prompt(self, func: FunctionInfo) -> str:
        return f"""You are a senior code auditor. Analyze this {func.language} function for ANY issues.

FUNCTION: {func.function_name} in {func.file}:{func.start_line}-{func.end_line}
SIGNATURE: {func.signature}

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
- parity: iOS/Android missing equivalents (if applicable)
- crypto: Hardcoded keys, weak randomness, missing zeroize
- architecture: Circular deps, god objects, missing abstractions
- docs: Missing public API docs, outdated comments

OUTPUT: One JSON object per line ONLY:
{{"file":"{func.file}","function":"{func.function_name}","line":{func.start_line},"severity":"critical|high|medium|low|info","category":"<cat>","title":"<title>","description":"<desc>","code_snippet":"<code>","suggestion":"<fix>"}}

If no issues, output nothing."""

    def _parse_response(self, response: str, func: FunctionInfo) -> List[AuditIssue]:
        issues = []
        for line in response.strip().split('\n'):
            line = line.strip()
            if not line or not line.startswith('{'):
                continue
            try:
                # Find the JSON object
                start = line.find('{')
                end = line.rfind('}') + 1
                if start >= 0 and end > start:
                    data = json.loads(line[start:end])
                    data.setdefault('file', func.file)
                    data.setdefault('function', func.function_name)
                    data.setdefault('line', func.start_line)
                    issues.append(AuditIssue(**data))
            except json.JSONDecodeError:
                pass
        return issues


# ============ STATE MANAGEMENT ============
def load_state() -> AuditState:
    if STATE_FILE.exists():
        try:
            data = json.loads(STATE_FILE.read_text())
            return AuditState(**data)
        except:
            pass
    return AuditState()


def save_state(state: AuditState):
    state.last_update = datetime.now().isoformat()
    STATE_FILE.write_text(json.dumps(asdict(state), indent=2))


def append_result(issue: AuditIssue):
    with open(RESULTS_FILE, 'a') as f:
        f.write(json.dumps(asdict(issue)) + '\n')


# ============ MAIN AUDIT LOOP ============
def discover_files() -> List[Path]:
    files = []
    for priority_dir in PRIORITY_DIRS:
        dir_path = REPO_ROOT / priority_dir
        if dir_path.exists():
            for ext in AUDIT_EXTS:
                files.extend(dir_path.rglob(f'*{ext}'))
    # Filter skip dirs
    filtered = []
    for f in files:
        if not any(part in SKIP_DIRS for part in f.parts):
            filtered.append(f)
    return sorted(filtered)


def main():
    print(f"SCMessenger Sequential Function Audit")
    print(f"Model: {OLLAMA_MODEL}")
    print(f"Repo: {REPO_ROOT}")
    print(f"Output: {AUDIT_DIR}")
    print("=" * 60)
    
    # Verify Ollama
    try:
        r = subprocess.run(['curl', '-s', 'http://localhost:11434/api/tags'], 
                          capture_output=True, text=True, timeout=5)
        if OLLAMA_MODEL not in r.stdout:
            print(f"ERROR: Model {OLLAMA_MODEL} not found in Ollama")
            sys.exit(1)
    except Exception as e:
        print(f"ERROR: Ollama not accessible: {e}")
        sys.exit(1)
    
    print("Ollama verified [OK]")
    
    # Load state
    state = load_state()
    client = OllamaClient()
    parser = FunctionParser()
    
    # Discover all files and functions
    print("\nDiscovering files and functions...")
    files = discover_files()
    print(f"Found {len(files)} source files")
    
    all_functions: List[FunctionInfo] = []
    for file_path in files:
        funcs = parser.parse_file(file_path)
        all_functions.extend(funcs)
    
    state.total_functions = len(all_functions)
    print(f"Extracted {len(all_functions)} functions")
    
    # Build completed set
    completed = set(state.completed_functions)
    
    # Filter to remaining
    remaining = [f for f in all_functions if f"{f.file}::{f.function_name}" not in completed]
    print(f"Remaining: {len(remaining)} functions")
    
    if not remaining:
        print("All functions already audited!")
        return
    
    # Sequential audit loop
    print("\nStarting sequential audit...")
    print("=" * 60)
    
    for idx, func in enumerate(remaining):
        func_key = f"{func.file}::{func.function_name}"
        state.current_file = func.file
        state.current_function = func.function_name
        
        print(f"[{idx+1}/{len(remaining)}] {func.file} :: {func.function_name} (lines {func.start_line}-{func.end_line})")
        
        # Audit this function
        issues = client.audit_function(func)
        
        # Save each issue
        for issue in issues:
            append_result(issue)
            state.total_issues += 1
            sev = issue.severity.upper()
            print(f"  [{sev}] {issue.category}: {issue.title}")
        
        # Mark completed
        state.completed_functions.append(func_key)
        save_state(state)
        
        # Progress every 10 functions
        if (idx + 1) % 10 == 0:
            elapsed = time.time() - time.mktime(datetime.fromisoformat(state.start_time).timetuple())
            rate = (idx + 1) / elapsed if elapsed > 0 else 0
            eta = (len(remaining) - idx - 1) / rate if rate > 0 else 0
            print(f"  Progress: {idx+1}/{len(remaining)} | {state.total_issues} issues | {rate:.1f} func/s | ETA: {eta/60:.1f}min")
    
    # Final report
    print("\n" + "=" * 60)
    print("AUDIT COMPLETE")
    print(f"Total functions: {state.total_functions}")
    print(f"Total issues: {state.total_issues}")
    print(f"Results: {RESULTS_FILE}")
    print(f"State: {STATE_FILE}")
    
    generate_report(state)


def generate_report(state: AuditState):
    # Load all issues
    issues = []
    if RESULTS_FILE.exists():
        for line in RESULTS_FILE.read_text().strip().split('\n'):
            if line:
                issues.append(AuditIssue(**json.loads(line)))
    
    # Group by severity
    by_sev = {}
    by_cat = {}
    by_file = {}
    for issue in issues:
        by_sev.setdefault(issue.severity, []).append(issue)
        by_cat.setdefault(issue.category, []).append(issue)
        by_file.setdefault(issue.file, []).append(issue)
    
    with open(REPORT_FILE, 'w') as f:
        f.write(f"# SCMessenger Function-Level Audit Report\n\n")
        f.write(f"**Generated:** {datetime.now().isoformat()}\n")
        f.write(f"**Model:** {OLLAMA_MODEL}\n")
        f.write(f"**Functions Audited:** {state.total_functions}\n")
        f.write(f"**Total Issues:** {state.total_issues}\n\n")
        
        f.write("## By Severity\n")
        for sev in ['critical', 'high', 'medium', 'low', 'info']:
            count = len(by_sev.get(sev, []))
            f.write(f"- **{sev.upper()}:** {count}\n")
        f.write("\n")
        
        f.write("## By Category\n")
        for cat in sorted(by_cat.keys()):
            f.write(f"- **{cat}:** {len(by_cat[cat])}\n")
        f.write("\n")
        
        f.write("## Top Files by Issue Count\n")
        for file, file_issues in sorted(by_file.items(), key=lambda x: -len(x[1]))[:30]:
            sev_counts = {}
            for i in file_issues:
                sev_counts[i.severity] = sev_counts.get(i.severity, 0) + 1
            sev_str = ', '.join(f"{k}:{v}" for k,v in sorted(sev_counts.items()))
            f.write(f"- `{file}`: {len(file_issues)} issues ({sev_str})\n")
        f.write("\n")
        
        f.write("## Critical & High Issues\n")
        for sev in ['critical', 'high']:
            for issue in by_sev.get(sev, []):
                f.write(f"\n### {issue.file} :: {issue.function} (line {issue.line})\n")
                f.write(f"**Category:** {issue.category} | **Severity:** {issue.severity}\n\n")
                f.write(f"{issue.description}\n\n")
                f.write(f"```{issue.function.split('::')[-1] if '::' in issue.function else 'text'}\n{issue.code_snippet}\n```\n\n")
                if issue.suggestion:
                    f.write(f"**Suggestion:** {issue.suggestion}\n\n")
                f.write("---\n")


if __name__ == '__main__':
    main()