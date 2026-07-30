#!/usr/bin/env python3
"""
SCMessenger Code Audit System
Uses local Ollama qwen2.5-coder:7b to audit all code files for issues.
"""

import os
import json
import subprocess
import sys
from pathlib import Path
from typing import List, Dict, Any, Optional
from dataclasses import dataclass, asdict
import time
import hashlib
from concurrent.futures import ThreadPoolExecutor, as_completed
import threading

# Configuration
OLLAMA_MODEL = "qwen2.5-coder:7b"
OLLAMA_URL = "http://localhost:11434/api/generate"
MAX_CONTEXT_TOKENS = 4096
MAX_FILE_LINES = 500  # Process files in chunks of this many lines
CHUNK_OVERLAP = 50    # Overlap between chunks
MAX_WORKERS = 2       # Conservative for local model
REPO_ROOT = Path(r"C:\Users\SCM\Documents\GitHub\SCMessenger")
AUDIT_OUTPUT_DIR = REPO_ROOT / "audit_system" / "results"
HANDOFF_FILE = REPO_ROOT / "audit_system" / "AUDIT_HANDOFF.md"

# File extensions to audit
AUDIT_EXTENSIONS = {
    '.rs', '.kt', '.swift', '.toml', '.yaml', '.yml', 
    '.json', '.md', '.py', '.sh', '.ps1', '.gradle',
    '.kt', '.kts', '.xml', '.plist', '.xcconfig'
}

# Directories to skip
SKIP_DIRS = {
    'target', 'build', '.git', 'node_modules', '__pycache__',
    '.gradle', 'dist', 'out', 'bin', 'obj', 'tmp', 'scratch',
    'SCMessengerCore.xcframework', 'rustup-init.exe'
}

# Prompt template
AUDIT_PROMPT_TEMPLATE = """You are a senior code auditor reviewing SCMessenger code for the V1.0.0 release. Find EVERYTHING that falls short of perfection. We prefer FALSE POSITIVES over FALSE NEGATIVES.

## AUDIT SCOPE - FIND ALL OF THESE:

### 1. INCOMPLETENESS MARKERS
- `TODO`, `FIXME`, `XXX`, `HACK`, `TEMP`, `TEMPORARY`, `unimplemented!()`, `todo!()`, `stub`, `mock` (in production code)
- `placeholder`, `FIX ME`, `FIXME:`, `XXX:`, `NOTE:`, `XXX -`, `HACK:`
- Commented-out code blocks that look intentional
- `#[allow(dead_code)]` used suspiciously

### 2. MAGIC NUMBERS & HARDCODED VALUES
- Numeric literals as limits, timeouts, sizes, thresholds without named constants
- String literals as keys, tags, IDs repeated in multiple places
- Hardcoded paths, URLs, ports, versions
- Time durations without named constants (e.g., `Duration::from_secs(30)`)

### 3. INCONSISTENT NAMING
- Mixed naming conventions (snake_case vs camelCase in same file)
- Abbreviations vs full words inconsistently
- Similar concepts named differently across files
- Generic names (`data`, `info`, `manager`, `handler`, `util`, `helper`, `temp`, `val`, `res`)

### 4. UNSAFE/UNSOUND PATTERNS
- `unsafe` blocks without safety comments
- `unwrap()`, `expect()`, `panic!()` in production paths
- `.unreachable_unchecked()`, `std::mem::transmute`, pointer arithmetic
- `#[allow(unused)]` on public APIs
- `Rc<RefCell<>>` or `Arc<Mutex<>>` where lock-free would be better

### 5. ERROR HANDLING ISSUES
- `Result`/`Option` ignored with `let _ =` or `;`
- Different error types for same failure mode
- Error messages that don't help debugging
- `map_err(|_| ...)` losing context
- `?` in loops without proper handling
- Missing `Result` returns where fallible

### 6. DEAD/UNUSED CODE
- `#[cfg(test)]` items in production modules
- Public items never used
- Private methods never called
- Struct fields never read
- Enum variants never constructed
- Imported but unused items

### 7. INCOMPLETE IMPLEMENTATIONS
- Functions returning `unimplemented!()` or `panic!("not implemented")`
- Traits with default impls that should be required
- Partial trait implementations
- `// TODO: implement` comments
- Empty match arms (`_ => {}`)
- `Default::default()` where explicit is clearer

### 8. TESTING GAPS
- No tests for public API
- Tests only for happy path
- No property-based/fuzzing tests for crypto/parsing
- Mock-heavy tests without integration tests
- Test files missing for modules

### 9. PERFORMANCE ISSUES
- `Vec`/`String` allocations in hot loops
- `clone()` where `&` would work
- `HashMap` with String keys where `&str` or enum would work
- Blocking operations in async contexts
- Unbounded channels/buffers
- Missing `reserve()`/`with_capacity()`

### 10. THREAD SAFETY
- `!Send`/`!Sync` types in async tasks
- Data races potential (unsynchronized shared mutable state)
- `RwLock` where `Mutex` needed or vice versa
- Lock ordering issues (deadlock potential)
- Long-held locks

### 11. API DESIGN ISSUES
- Functions with >5 parameters
- Functions returning tuples instead of structs
- Public fields that should be private with getters
- Missing `const`/`async` where appropriate
- Inconsistent `&self` vs `&mut self` vs `self`
- Builder patterns missing for complex construction

### 12. IOS/ANDROID PARITY (CRITICAL)
- Methods in Android `MeshRepository.kt` missing in iOS `MeshRepository.swift`
- Methods in iOS missing in Android
- Different parameter types/names for same operation
- Different return types for same operation
- Different error handling patterns
- Different async patterns (suspend vs async/await vs callbacks)
- Missing ViewModels on one platform
- Missing screens/views on one platform
- Different transport implementations where parity expected

### 13. CRYPTO/SECURITY ISSUES
- Hardcoded keys, nonces, salts
- Non-constant-time comparisons
- Missing zeroization of secrets
- Weak randomness sources
- Missing authentication tags
- Reused nonces/IVs
- Side-channel vulnerable code

### 14. ARCHITECTURAL INCONSISTENCIES
- Different patterns for same concept across modules
- Mixed async/sync boundaries
- Circular dependencies
- God objects (too many responsibilities)
- Missing abstraction layers
- Direct dependencies on concrete types instead of traits

### 15. DOCUMENTATION GAPS
- Public APIs without docs
- Complex logic without comments
- Outdated comments
- Missing module-level documentation
- No architecture decision records for complex choices

## OUTPUT FORMAT - ONE JSON OBJECT PER LINE:

```json
{
  "file": "relative/path/to/file.ext",
  "line": 123,
  "column": 45,
  "severity": "critical|high|medium|low|info",
  "category": "incompleteness|magic_number|naming|unsafe|error_handling|dead_code|incomplete|testing|performance|thread_safety|api_design|parity|crypto|architecture|docs",
  "title": "Brief descriptive title",
  "description": "Detailed explanation of the issue",
  "code_snippet": "the problematic code line(s)",
  "suggestion": "How to fix (optional)"
}
```

## FILE TO ANALYZE:
{file_path}

## FILE CONTENT (lines {start_line}-{end_line}):
```{lang}
{content}
```

Analyze this code chunk thoroughly. Output ONE JSON object per line for each issue found. If no issues, output nothing.
"""

@dataclass
class AuditIssue:
    file: str
    line: int
    column: int
    severity: str
    category: str
    title: str
    description: str
    code_snippet: str
    suggestion: str = ""

class AuditSystem:
    def __init__(self):
        self.results: List[AuditIssue] = []
        self.processed_files = set()
        self.lock = threading.Lock()
        self.stats = {
            'files_processed': 0,
            'chunks_processed': 0,
            'issues_found': 0,
            'errors': 0,
            'start_time': time.time()
        }
        AUDIT_OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    
    def get_file_language(self, file_path: Path) -> str:
        ext = file_path.suffix.lower()
        lang_map = {
            '.rs': 'rust', '.kt': 'kotlin', '.swift': 'swift',
            '.py': 'python', '.js': 'javascript', '.ts': 'typescript',
            '.json': 'json', '.yaml': 'yaml', '.yml': 'yaml',
            '.toml': 'toml', '.md': 'markdown', '.sh': 'bash',
            '.ps1': 'powershell', '.gradle': 'groovy', '.xml': 'xml',
            '.plist': 'xml', '.xcconfig': 'text', '.kts': 'kotlin'
        }
        return lang_map.get(ext, 'text')
    
    def should_audit_file(self, file_path: Path) -> bool:
        # Skip directories
        for part in file_path.parts:
            if part in SKIP_DIRS:
                return False
        # Check extension
        return file_path.suffix.lower() in AUDIT_EXTENSIONS
    
    def read_file_lines(self, file_path: Path) -> List[str]:
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                return f.readlines()
        except UnicodeDecodeError:
            try:
                with open(file_path, 'r', encoding='latin-1') as f:
                    return f.readlines()
            except:
                return []
        except Exception:
            return []
    
    def chunk_file(self, lines: List[str], file_path: Path) -> List[Dict]:
        """Split file into overlapping chunks for processing."""
        chunks = []
        total_lines = len(lines)
        
        if total_lines <= MAX_FILE_LINES:
            chunks.append({
                'start_line': 1,
                'end_line': total_lines,
                'content': ''.join(lines),
                'file_path': str(file_path.relative_to(REPO_ROOT))
            })
        else:
            for i in range(0, total_lines, MAX_FILE_LINES - CHUNK_OVERLAP):
                start = i
                end = min(i + MAX_FILE_LINES, total_lines)
                chunk_lines = lines[start:end]
                chunks.append({
                    'start_line': start + 1,
                    'end_line': end,
                    'content': ''.join(chunk_lines),
                    'file_path': str(file_path.relative_to(REPO_ROOT))
                })
                if end >= total_lines:
                    break
        return chunks
    
    def call_ollama(self, prompt: str) -> Optional[str]:
        """Call local Ollama API."""
        payload = {
            "model": OLLAMA_MODEL,
            "prompt": prompt,
            "stream": False,
            "options": {
                "temperature": 0.1,
                "num_ctx": MAX_CONTEXT_TOKENS,
                "top_p": 0.9,
                "top_k": 40
            }
        }
        try:
            result = subprocess.run(
                ['curl', '-s', '-X', 'POST', OLLAMA_URL, 
                 '-H', 'Content-Type: application/json',
                 '-d', json.dumps(payload)],
                capture_output=True,
                text=True,
                timeout=180
            )
            if result.returncode == 0:
                response = json.loads(result.stdout)
                return response.get('response', '')
            else:
                print(f"Ollama error: {result.stderr}")
                return None
        except subprocess.TimeoutExpired:
            print("Ollama timeout")
            return None
        except Exception as e:
            print(f"Ollama call failed: {e}")
            return None
    
    def parse_ollama_response(self, response: str, file_path: str, chunk_start_line: int) -> List[AuditIssue]:
        """Parse JSON lines from Ollama response."""
        issues = []
        for line in response.strip().split('\n'):
            line = line.strip()
            if not line:
                continue
            # Try to extract JSON from the line
            try:
                # Find JSON object in the line
                start = line.find('{')
                end = line.rfind('}') + 1
                if start >= 0 and end > start:
                    json_str = line[start:end]
                    data = json.loads(json_str)
                    # Adjust line number to be absolute in file
                    if 'line' in data and data['line'] > 0:
                        data['line'] = chunk_start_line + data['line'] - 1
                    data['file'] = file_path
                    issues.append(AuditIssue(**data))
            except json.JSONDecodeError:
                # Try to find JSON in markdown code block
                if '```json' in line or '```' in line:
                    continue
                pass
        return issues
    
    def process_chunk(self, chunk: Dict) -> List[AuditIssue]:
        """Process a single chunk through Ollama."""
        lang = self.get_file_language(Path(chunk['file_path']))
        prompt = AUDIT_PROMPT_TEMPLATE.format(
            file_path=chunk['file_path'],
            start_line=chunk['start_line'],
            end_line=chunk['end_line'],
            lang=lang,
            content=chunk['content']
        )
        
        response = self.call_ollama(prompt)
        if response:
            issues = self.parse_ollama_response(response, chunk['file_path'], chunk['start_line'])
            return issues
        return []
    
    def process_file(self, file_path: Path) -> List[AuditIssue]:
        """Process a single file through the audit system."""
        lines = self.read_file_lines(file_path)
        if not lines:
            return []
        
        chunks = self.chunk_file(lines, file_path)
        all_issues = []
        
        for chunk in chunks:
            issues = self.process_chunk(chunk)
            all_issues.extend(issues)
            with self.lock:
                self.stats['chunks_processed'] += 1
        
        return all_issues
    
    def discover_files(self) -> List[Path]:
        """Discover all files to audit."""
        files = []
        for ext in AUDIT_EXTENSIONS:
            files.extend(REPO_ROOT.rglob(f'*{ext}'))
        
        # Filter
        filtered = [f for f in files if self.should_audit_file(f)]
        return sorted(filtered)
    
    def run_audit(self):
        """Run the full audit."""
        print("Discovering files...")
        files = self.discover_files()
        print(f"Found {len(files)} files to audit")
        
        # Save file list
        with open(AUDIT_OUTPUT_DIR / 'file_list.json', 'w') as f:
            json.dump([str(f.relative_to(REPO_ROOT)) for f in files], f, indent=2)
        
        # Process files in parallel
        with ThreadPoolExecutor(max_workers=MAX_WORKERS) as executor:
            future_to_file = {executor.submit(self.process_file, f): f for f in files}
            
            for future in as_completed(future_to_file):
                file_path = future_to_file[future]
                try:
                    issues = future.result()
                    with self.lock:
                        self.results.extend(issues)
                        self.stats['files_processed'] += 1
                        self.stats['issues_found'] += len(issues)
                    
                    if self.stats['files_processed'] % 10 == 0:
                        self.save_progress()
                        elapsed = time.time() - self.stats['start_time']
                        print(f"Processed {self.stats['files_processed']}/{len(files)} files, "
                              f"{self.stats['issues_found']} issues found, "
                              f"{elapsed:.1f}s elapsed")
                except Exception as e:
                    with self.lock:
                        self.stats['errors'] += 1
                    print(f"Error processing {file_path}: {e}")
        
        self.save_final_results()
        self.generate_handoff_report()
        self.print_summary()
    
    def save_progress(self):
        """Save intermediate results."""
        output_file = AUDIT_OUTPUT_DIR / 'audit_results_partial.jsonl'
        with open(output_file, 'w') as f:
            for issue in self.results:
                f.write(json.dumps(asdict(issue)) + '\n')
    
    def save_final_results(self):
        """Save final results in multiple formats."""
        # JSONL for processing
        with open(AUDIT_OUTPUT_DIR / 'audit_results.jsonl', 'w') as f:
            for issue in self.results:
                f.write(json.dumps(asdict(issue)) + '\n')
        
        # JSON array for reading
        with open(AUDIT_OUTPUT_DIR / 'audit_results.json', 'w') as f:
            json.dump([asdict(i) for i in self.results], f, indent=2)
        
        # CSV for spreadsheet analysis
        import csv
        with open(AUDIT_OUTPUT_DIR / 'audit_results.csv', 'w', newline='') as f:
            if self.results:
                writer = csv.DictWriter(f, fieldnames=asdict(self.results[0]).keys())
                writer.writeheader()
                for issue in self.results:
                    writer.writerow(asdict(issue))
    
    def generate_handoff_report(self):
        """Generate the handoff markdown report."""
        # Group by severity
        by_severity = {}
        for issue in self.results:
            by_severity.setdefault(issue.severity, []).append(issue)
        
        # Group by category
        by_category = {}
        for issue in self.results:
            by_category.setdefault(issue.category, []).append(issue)
        
        # Group by file
        by_file = {}
        for issue in self.results:
            by_file.setdefault(issue.file, []).append(issue)
        
        with open(HANDOFF_FILE, 'w') as f:
            f.write("# SCMessenger V1.0.0 Code Audit Report\n\n")
            f.write(f"**Generated:** {time.strftime('%Y-%m-%d %H:%M:%S')}\n")
            f.write(f"**Model:** {OLLAMA_MODEL}\n")
            f.write(f"**Files Audited:** {self.stats['files_processed']}\n")
            f.write(f"**Chunks Processed:** {self.stats['chunks_processed']}\n")
            f.write(f"**Total Issues Found:** {self.stats['issues_found']}\n")
            f.write(f"**Errors During Audit:** {self.stats['errors']}\n")
            f.write(f"**Duration:** {time.time() - self.stats['start_time']:.1f}s\n\n")
            
            # Summary by severity
            f.write("## Summary by Severity\n\n")
            severity_order = ['critical', 'high', 'medium', 'low', 'info']
            for sev in severity_order:
                count = len(by_severity.get(sev, []))
                f.write(f"- **{sev.upper()}:** {count}\n")
            f.write("\n")
            
            # Summary by category
            f.write("## Summary by Category\n\n")
            for cat in sorted(by_category.keys()):
                count = len(by_category[cat])
                f.write(f"- **{cat}:** {count}\n")
            f.write("\n")
            
            # Top files by issue count
            f.write("## Top Files by Issue Count\n\n")
            sorted_files = sorted(by_file.items(), key=lambda x: len(x[1]), reverse=True)
            for file_path, issues in sorted_files[:20]:
                sev_counts = {}
                for issue in issues:
                    sev_counts[issue.severity] = sev_counts.get(issue.severity, 0) + 1
                sev_str = ', '.join(f"{k}:{v}" for k,v in sorted(sev_counts.items()))
                f.write(f"- `{file_path}`: {len(issues)} issues ({sev_str})\n")
            f.write("\n")
            
            # Critical and High issues detail
            f.write("## Critical & High Severity Issues\n\n")
            for sev in ['critical', 'high']:
                issues = by_severity.get(sev, [])
                if issues:
                    f.write(f"### {sev.upper()} ({len(issues)} issues)\n\n")
                    for issue in issues[:50]:  # Limit to top 50
                        f.write(f"**{issue.file}:{issue.line}** - {issue.title}\n")
                        f.write(f"> {issue.description}\n\n")
                        f.write(f"```\n{issue.code_snippet}\n```\n\n")
                        if issue.suggestion:
                            f.write(f"*Suggestion:* {issue.suggestion}\n\n")
                        f.write("---\n\n")
            
            # All issues by file (for reference)
            f.write("## All Issues by File\n\n")
            for file_path, issues in sorted_files:
                f.write(f"### {file_path} ({len(issues)} issues)\n\n")
                for issue in issues:
                    f.write(f"- **Line {issue.line}** [{issue.severity.upper()}] {issue.title}\n")
                    f.write(f"  > {issue.description[:200]}{'...' if len(issue.description) > 200 else ''}\n")
                f.write("\n")
    
    def print_summary(self):
        """Print final summary."""
        elapsed = time.time() - self.stats['start_time']
        print("\n" + "="*60)
        print("AUDIT COMPLETE")
        print("="*60)
        print(f"Files processed: {self.stats['files_processed']}")
        print(f"Chunks processed: {self.stats['chunks_processed']}")
        print(f"Total issues found: {self.stats['issues_found']}")
        print(f"Errors: {self.stats['errors']}")
        print(f"Time: {elapsed:.1f}s")
        print(f"Results saved to: {AUDIT_OUTPUT_DIR}")
        print(f"Handoff report: {HANDOFF_FILE}")

def main():
    print("SCMessenger Code Audit System")
    print(f"Using model: {OLLAMA_MODEL}")
    print(f"Repo: {REPO_ROOT}")
    print()
    
    # Check Ollama is running
    try:
        result = subprocess.run(['curl', '-s', 'http://localhost:11434/api/tags'], 
                               capture_output=True, text=True, timeout=5)
        if result.returncode != 0:
            print("ERROR: Ollama not running. Start with 'ollama serve'")
            sys.exit(1)
        tags = json.loads(result.stdout)
        models = [m['name'] for m in tags.get('models', [])]
        if OLLAMA_MODEL not in models:
            print(f"ERROR: Model {OLLAMA_MODEL} not found. Available: {models}")
            sys.exit(1)
        print(f"Ollama ready with {OLLAMA_MODEL}")
    except Exception as e:
        print(f"ERROR checking Ollama: {e}")
        sys.exit(1)
    
    # Run audit
    audit = AuditSystem()
    audit.run_audit()

if __name__ == '__main__':
    main()