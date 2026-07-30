#!/usr/bin/env python3
"""
SCMessenger Sequential Function-Level Audit - FINAL VERSION
- Parses all functions in priority directories
- Audits each function sequentially with local Ollama
- Saves state after EVERY function (checkpoint)
- Resumes from last checkpoint on restart
- Zero parallelization
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
AUDIT_DIR = REPO_ROOT / "audit_system" / "results_final"
STATE_FILE = AUDIT_DIR / "audit_state.json"
RESULTS_FILE = AUDIT_DIR / "audit_results.jsonl"
REPORT_FILE = AUDIT_DIR / "AUDIT_REPORT.md"

# Priority directories to audit (in order)
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
    checksum: str = ""  # For detecting changes

    def __post_init__(self):
        import hashlib
        self.checksum = hashlib.md5(self.content.encode()).hexdigest()[:8]

    def unique_id(self) -> str:
        return f"{self.file}::{self.function_name}"


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
    completed: List[str] = field(default_factory=list)  # List of unique_ids
    all_functions: List[Dict] = field(default_factory=list)  # Serialized FunctionInfo
    total_functions: int = 0
    total_issues: int = 0
    current_index: int = 0
    start_time: str = field(default_factory=lambda: datetime.now().isoformat())
    last_update: str = field(default_factory=lambda: datetime.now().isoformat())
    errors: List[str] = field(default_factory=list)


# ============ FUNCTION PARSERS ============
class FunctionParser:
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
                functions.append(FunctionInfo(
                    file=file_path, function_name=fn_name,
                    start_line=start_line, end_line=end_line + 1,
                    content=fn_content, language='rust', signature=line.strip()
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
                
                # Find opening brace or single-expression
                brace_line = i
                while brace_line < len(lines) and '{' not in lines[brace_line]:
                    brace_line += 1
                    if brace_line - i > 15:
                        break
                
                if brace_line >= len(lines) or '{' not in lines[brace_line]:
                    # Single expression function
                    if '=' in line and '{' not in line:
                        functions.append(FunctionInfo(
                            file=file_path, function_name=fn_name,
                            start_line=start_line, end_line=start_line,
                            content=line.strip(), language='kotlin', signature=line.strip()
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
                    file=file_path, function_name=fn_name,
                    start_line=start_line, end_line=end_line + 1,
                    content=fn_content, language='kotlin', signature=line.strip()
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
                functions.append(FunctionInfo(
                    file=file_path, function_name=fn_name,
                    start_line=start_line, end_line=end_line + 1,
                    content=fn_content, language='swift', signature=line.strip()
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
        self.timeout = 90

    def audit_function(self, func: FunctionInfo) -> List[AuditIssue]:
        """Audit a single function, return list of issues"""
        prompt = self._build_prompt(func)
        
        payload = {
            "model": OLLAMA_MODEL,
            "prompt": prompt,
            "stream": False,
            "options": {"temperature": 0.0, "num_ctx": MAX_CONTEXT, "top_p": 0.9}
        }
        
        try:
            result = subprocess.run(
                ['curl', '-s', '-X', 'POST', OLLAMA_URL, 
                 '-H', 'Content-Type: application/json',
                 '-d', json.dumps(payload)],
                capture_output=True, text=True, timeout=self.timeout
            )
            if result.returncode != 0:
                return [AuditIssue(
                    file=func.file, function=func.function_name, line=func.start_line,
                    severity="info", category="system", title="Ollama Error",
                    description=f"Ollama call failed: {result.stderr[:200]}",
                    code_snippet=""
                )]
            
            response = json.loads(result.stdout).get('response', '')
            return self._parse_response(response, func)
            
        except subprocess.TimeoutExpired:
            return [AuditIssue(
                file=func.file, function=func.function_name, line=func.start_line,
                severity="info", category="system", title="Timeout",
                description="Ollama request timed out after 90s",
                code_snippet=""
            )]
        except Exception as e:
            return [AuditIssue(
                file=func.file, function=func.function_name, line=func.start_line,
                severity="info", category="system", title="Error",
                description=f"Exception: {str(e)[:200]}",
                code_snippet=""
            )]

    def _build_prompt(self, func: FunctionInfo) -> str:
        return f"""You are a senior code auditor. Analyze this {func.language} function for ALL issues.

FUNCTION: {func.function_name} in {func.file} (lines {func.start_line}-{func.end_line})
```{func.language}
{func.content[:3000]}
```

Find issues in these categories:
- TODO/FIXME/XXX/HACK comments
- Magic numbers (hardcoded limits, timeouts, sizes)
- Naming inconsistencies
- unsafe/unwrap/expect/panic in production code
- Error handling gaps (ignored Results, map_err losing context)
- Dead code (unused params, vars, imports)
- Incomplete implementations (unimplemented!, panic!("not implemented"))
- Missing tests for public functions
- Performance issues (allocations in loops, unnecessary clones)
- Thread safety concerns
- API design issues (>5 params, tuple returns, public fields)
- iOS/Android parity gaps (missing methods on one platform)
- Crypto/security issues (hardcoded keys, weak randomness, missing zeroization)
- Architecture inconsistencies
- Missing documentation for public APIs

Output JSON lines ONLY, one per issue:
{{"line": <relative_line_in_function>, "severity": "critical|high|medium|low|info", "category": "todo|magic_number|naming|unsafe|error_handling|dead_code|incomplete|testing|performance|thread_safety|api_design|parity|crypto|architecture|docs", "title": "Brief title", "description": "Detailed explanation", "code": "problematic code snippet", "suggestion": "How to fix"}}

If NO issues, output nothing.
"""

    def _parse_response(self, response: str, func: FunctionInfo) -> List[AuditIssue]:
        issues = []
        for line in response.strip().split('\n'):
            line = line.strip()
            if not line or not line.startswith('{'):
                continue
            try:
                data = json.loads(line)
                # Convert relative line to absolute
                rel_line = data.get('line', 1)
                abs_line = func.start_line + max(0, rel_line - 1)
                
                issues.append(AuditIssue(
                    file=func.file, function=func.function_name, line=abs_line,
                    severity=data.get('severity', 'info'),
                    category=data.get('category', 'docs'),
                    title=data.get('title', 'Issue'),
                    description=data.get('description', ''),
                    code_snippet=data.get('code', ''),
                    suggestion=data.get('suggestion', '')
                ))
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
                # Skip if in SKIP_DIRS
                if any(skip in file_path.parts for skip in SKIP_DIRS):
                    continue
                
                funcs = FunctionParser.parse_file(file_path)
                all_functions.extend(funcs)
    
    return all_functions


def run_audit():
    print("=" * 60)
    print("SCMessenger Sequential Function Audit")
    print("=" * 60)
    
    # Load or create state
    state = load_state()
    
    if state and state.all_functions:
        print(f"Resuming from checkpoint: {state.current_index}/{state.total_functions} functions")
        print(f"Already completed: {len(state.completed)} functions")
        print(f"Issues found so far: {state.total_issues}")
        
        # Reconstruct function list
        all_functions = [FunctionInfo(**f) for f in state.all_functions]
    else:
        print("Discovering all functions in priority directories...")
        all_functions = discover_all_functions()
        print(f"Found {len(all_functions)} functions to audit")
        
        state = AuditState(
            all_functions=[asdict(f) for f in all_functions],
            total_functions=len(all_functions)
        )
        save_state(state)
    
    # Audit loop - one function at a time
    client = OllamaClient()
    
    for idx in range(state.current_index, len(all_functions)):
        func = all_functions[idx]
        func_id = func.unique_id()
        
        # Skip if already completed (checkpoint resume)
        if func_id in state.completed:
            state.current_index = idx + 1
            continue
        
        print(f"\n[{idx+1}/{len(all_functions)}] {func.file}::{func.function_name} (lines {func.start_line}-{func.end_line})")
        
        state.current_index = idx
        state.current_file = func.file
        state.current_function = func.function_name
        
        # Audit this function
        start_time = time.time()
        issues = client.audit_function(func)
        elapsed = time.time() - start_time
        
        # Save results
        for issue in issues:
            append_result(issue)
            state.total_issues += 1
        
        state.completed.append(func_id)
        state.current_index = idx + 1
        
        # Checkpoint every function
        save_state(state)
        
        # Progress update
        if issues:
            print(f"  Found {len(issues)} issues ({elapsed:.1f}s)")
            for issue in issues[:3]:
                print(f"    [{issue.severity}] {issue.category}: {issue.title}")
            if len(issues) > 3:
                print(f"    ... and {len(issues)-3} more")
        else:
            print(f"  No issues found ({elapsed:.1f}s)")
        
        # Small delay to prevent overwhelming
        time.sleep(0.5)
    
    # Final report
    generate_final_report(state)
    print("\n" + "=" * 60)
    print("AUDIT COMPLETE")
    print("=" * 60)
    print(f"Total functions: {state.total_functions}")
    print(f"Completed: {len(state.completed)}")
    print(f"Total issues: {state.total_issues}")
    print(f"Report: {REPORT_FILE}")


def generate_final_report(state: AuditState):
    """Generate markdown report from results"""
    # Read all issues
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
    
    for issue in issues:
        by_sev.setdefault(issue.severity, []).append(issue)
        by_cat.setdefault(issue.category, []).append(issue)
        by_file.setdefault(issue.file, []).append(issue)
    
    with open(REPORT_FILE, 'w') as f:
        f.write("# SCMessenger Code Audit Report\n\n")
        f.write(f"**Generated:** {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n")
        f.write(f"**Model:** {OLLAMA_MODEL}\n")
        f.write(f"**Functions Audited:** {len(state.completed)}/{state.total_functions}\n")
        f.write(f"**Total Issues:** {state.total_issues}\n\n")
        
        f.write("## Summary by Severity\n\n")
        for sev in ['critical', 'high', 'medium', 'low', 'info']:
            count = len(by_sev.get(sev, []))
            f.write(f"- **{sev.upper()}:** {count}\n")
        f.write("\n")
        
        f.write("## Summary by Category\n\n")
        for cat in sorted(by_cat.keys()):
            f.write(f"- **{cat}:** {len(by_cat[cat])}\n")
        f.write("\n")
        
        f.write("## Top Files by Issue Count\n\n")
        for file, file_issues in sorted(by_file.items(), key=lambda x: -len(x[1]))[:30]:
            sev_counts = {}
            for i in file_issues:
                sev_counts[i.severity] = sev_counts.get(i.severity, 0) + 1
            sev_str = ', '.join(f"{k}:{v}" for k,v in sorted(sev_counts.items()))
            f.write(f"- `{file}`: {len(file_issues)} issues ({sev_str})\n")
        f.write("\n")
        
        # Critical and High issues detail
        f.write("## Critical & High Severity Issues\n\n")
        for sev in ['critical', 'high']:
            for issue in by_sev.get(sev, [])[:100]:
                f.write(f"### {issue.file}:{issue.line} in `{issue.function}`\n")
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