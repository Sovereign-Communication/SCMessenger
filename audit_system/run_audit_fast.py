#!/usr/bin/env python3
"""
SCMessenger Fast Code Audit System
Optimized for local Ollama qwen2.5-coder:7b - targets critical code only.
"""

import os
import json
import subprocess
import sys
import time
from pathlib import Path
from typing import List, Dict, Any, Optional
from dataclasses import dataclass, asdict
from concurrent.futures import ThreadPoolExecutor, as_completed
import threading

# Configuration
OLLAMA_MODEL = "qwen2.5-coder:7b"
OLLAMA_URL = "http://localhost:11434/api/generate"
MAX_CONTEXT_TOKENS = 4096
MAX_FILE_LINES = 300  # Smaller chunks for speed
CHUNK_OVERLAP = 30
MAX_WORKERS = 1  # Sequential for stability - we can parallelize later
REPO_ROOT = Path(r"C:\Users\SCM\Documents\GitHub\SCMessenger")
AUDIT_OUTPUT_DIR = REPO_ROOT / "audit_system" / "results_fast"
HANDOFF_FILE = REPO_ROOT / "audit_system" / "AUDIT_HANDOFF_FAST.md"

# CRITICAL: Only audit these high-value directories first
PRIORITY_DIRS = [
    "core/src",
    "android/app/src/main/java/com/scmessenger/android",
    "iOS/SCMessenger/SCMessenger",
    "cli/src",
    "desktop_bridge/src",
]

# Extensions to audit
AUDIT_EXTS = {'.rs', '.kt', '.swift', '.py', '.toml', '.json', '.yaml', '.yml'}

# Directories to ALWAYS skip
SKIP_DIRS = {
    'target', 'build', '.git', 'node_modules', '__pycache__',
    '.gradle', 'dist', 'out', 'bin', 'obj', 'tmp', 'scratch',
    'SCMessengerCore.xcframework', '.claude', '.agents', '.bob',
    '.codex', '.github', '.cargo', 'rustup-init.exe'
}

# Concise audit prompt - optimized for token efficiency
# Using double braces to escape for .format()
AUDIT_PROMPT = """You are a senior code auditor. Find ALL issues in this SCMessenger code chunk. Report as JSON lines.

SEVERITY: critical|high|medium|low|info
CATEGORY: todo|magic_number|naming|unsafe|error_handling|dead_code|incomplete|testing|performance|thread_safety|api_design|parity|crypto|architecture|docs

OUTPUT ONE JSON PER LINE:
{{"file": "<file>", "line": <line>, "severity": "<sev>", "category": "<cat>", "title": "<title>", "desc": "<desc>", "code": "<code>", "fix": "<fix>"}}

FILE: {file} (lines {start}-{end})
```{lang}
{content}
```"""

@dataclass
class AuditIssue:
    file: str
    line: int
    severity: str
    category: str
    title: str
    desc: str
    code: str
    fix: str = ""

class FastAuditor:
    def __init__(self):
        self.results: List[AuditIssue] = []
        self.stats = {'files': 0, 'chunks': 0, 'issues': 0, 'errors': 0, 'start': time.time()}
        self.lock = threading.Lock()
        AUDIT_OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    
    def should_audit(self, path: Path) -> bool:
        for part in path.parts:
            if part in SKIP_DIRS:
                return False
        # Must be in priority dirs or have priority extension
        in_priority = any(str(path).startswith(str(REPO_ROOT / d)) for d in PRIORITY_DIRS)
        return in_priority and path.suffix.lower() in AUDIT_EXTS
    
    def discover_files(self) -> List[Path]:
        files = []
        for ext in AUDIT_EXTS:
            files.extend(REPO_ROOT.rglob(f'*{ext}'))
        return sorted([f for f in files if self.should_audit(f)])
    
    def read_lines(self, path: Path) -> List[str]:
        try:
            return path.read_text(encoding='utf-8').splitlines(keepends=True)
        except:
            try:
                return path.read_text(encoding='latin-1').splitlines(keepends=True)
            except:
                return []
    
    def chunk_file(self, lines: List[str], file_path: Path) -> List[Dict]:
        chunks = []
        total = len(lines)
        if total <= MAX_FILE_LINES:
            chunks.append({'start': 1, 'end': total, 'content': ''.join(lines), 'file': str(file_path.relative_to(REPO_ROOT))})
        else:
            for i in range(0, total, MAX_FILE_LINES - CHUNK_OVERLAP):
                end = min(i + MAX_FILE_LINES, total)
                chunks.append({'start': i+1, 'end': end, 'content': ''.join(lines[i:end]), 'file': str(file_path.relative_to(REPO_ROOT))})
                if end >= total: break
        return chunks
    
    def call_ollama(self, prompt: str) -> Optional[str]:
        payload = {
            "model": OLLAMA_MODEL,
            "prompt": prompt,
            "stream": False,
            "options": {"temperature": 0.0, "num_ctx": MAX_CONTEXT_TOKENS, "top_p": 0.9}
        }
        try:
            result = subprocess.run(
                ['curl', '-s', '-X', 'POST', OLLAMA_URL, '-H', 'Content-Type: application/json', '-d', json.dumps(payload)],
                capture_output=True, text=True, timeout=120
            )
            if result.returncode == 0:
                return json.loads(result.stdout).get('response', '')
        except Exception as e:
            print(f"  Ollama error: {e}")
        return None
    
    def parse_response(self, response: str, file: str, chunk_start: int) -> List[AuditIssue]:
        issues = []
        for line in response.strip().split('\n'):
            line = line.strip()
            if not line or not line.startswith('{'):
                continue
            try:
                data = json.loads(line)
                # Adjust line number
                if 'line' in data and data['line'] > 0:
                    data['line'] = chunk_start + data['line'] - 1
                data['file'] = file
                issues.append(AuditIssue(**data))
            except json.JSONDecodeError:
                pass
        return issues
    
    def process_chunk(self, chunk: Dict) -> List[AuditIssue]:
        lang = chunk['file'].split('.')[-1]
        prompt = AUDIT_PROMPT.format(
            file=chunk['file'], start=chunk['start'], end=chunk['end'],
            lang=lang, content=chunk['content'][:8000]  # Hard limit content
        )
        response = self.call_ollama(prompt)
        if response:
            return self.parse_response(response, chunk['file'], chunk['start'])
        return []
    
    def process_file(self, file_path: Path) -> List[AuditIssue]:
        lines = self.read_lines(file_path)
        if not lines:
            return []
        chunks = self.chunk_file(lines, file_path)
        all_issues = []
        for chunk in chunks:
            issues = self.process_chunk(chunk)
            all_issues.extend(issues)
            with self.lock:
                self.stats['chunks'] += 1
        return all_issues
    
    def run(self):
        print("Discovering priority files...")
        files = self.discover_files()
        print(f"Found {len(files)} files to audit")
        
        # Save file list
        (AUDIT_OUTPUT_DIR / 'file_list.json').write_text(json.dumps([str(f.relative_to(REPO_ROOT)) for f in files], indent=2))
        
        # Process sequentially for stability
        for i, file_path in enumerate(files):
            print(f"[{i+1}/{len(files)}] {file_path.relative_to(REPO_ROOT)}")
            issues = self.process_file(file_path)
            with self.lock:
                self.results.extend(issues)
                self.stats['files'] += 1
                self.stats['issues'] += len(issues)
            
            # Periodic save
            if (i + 1) % 10 == 0:
                self.save_progress()
                elapsed = time.time() - self.stats['start']
                print(f"  Progress: {self.stats['files']} files, {self.stats['issues']} issues, {elapsed:.1f}s")
        
        self.save_final()
        self.generate_report()
        self.print_summary()
    
    def save_progress(self):
        out = AUDIT_OUTPUT_DIR / 'audit_results_partial.jsonl'
        with open(out, 'w') as f:
            for issue in self.results:
                f.write(json.dumps(asdict(issue)) + '\n')
    
    def save_final(self):
        # JSONL
        (AUDIT_OUTPUT_DIR / 'audit_results.jsonl').write_text('\n'.join(json.dumps(asdict(i)) for i in self.results))
        # JSON
        (AUDIT_OUTPUT_DIR / 'audit_results.json').write_text(json.dumps([asdict(i) for i in self.results], indent=2))
        # CSV
        import csv
        with open(AUDIT_OUTPUT_DIR / 'audit_results.csv', 'w', newline='') as f:
            if self.results:
                writer = csv.DictWriter(f, fieldnames=asdict(self.results[0]).keys())
                writer.writeheader()
                writer.writerows(asdict(i) for i in self.results)
    
    def generate_report(self):
        by_sev = {}
        by_cat = {}
        by_file = {}
        for issue in self.results:
            by_sev.setdefault(issue.severity, []).append(issue)
            by_cat.setdefault(issue.category, []).append(issue)
            by_file.setdefault(issue.file, []).append(issue)
        
        with open(HANDOFF_FILE, 'w') as f:
            f.write(f"# SCMessenger Fast Audit Report\n\n")
            f.write(f"**Generated:** {time.strftime('%Y-%m-%d %H:%M:%S')}\n")
            f.write(f"**Model:** {OLLAMA_MODEL}\n")
            f.write(f"**Files:** {self.stats['files']} | **Chunks:** {self.stats['chunks']} | **Issues:** {self.stats['issues']}\n\n")
            
            f.write("## By Severity\n")
            for sev in ['critical', 'high', 'medium', 'low', 'info']:
                count = len(by_sev.get(sev, []))
                f.write(f"- **{sev.upper()}:** {count}\n")
            f.write("\n")
            
            f.write("## By Category\n")
            for cat in sorted(by_cat.keys()):
                f.write(f"- **{cat}:** {len(by_cat[cat])}\n")
            f.write("\n")
            
            f.write("## Top Files\n")
            for file, issues in sorted(by_file.items(), key=lambda x: -len(x[1]))[:30]:
                sev_counts = {}
                for i in issues:
                    sev_counts[i.severity] = sev_counts.get(i.severity, 0) + 1
                f.write(f"- `{file}`: {len(issues)} ({', '.join(f'{k}:{v}' for k,v in sorted(sev_counts.items()))})\n")
            f.write("\n")
            
            f.write("## Critical & High Issues\n")
            for sev in ['critical', 'high']:
                for issue in by_sev.get(sev, [])[:50]:
                    f.write(f"\n### {issue.file}:{issue.line} - {issue.title}\n")
                    f.write(f"**Category:** {issue.category} | **Severity:** {issue.severity}\n\n")
                    f.write(f"{issue.desc}\n\n")
                    f.write(f"```\n{issue.code}\n```\n\n")
                    if issue.fix:
                        f.write(f"**Fix:** {issue.fix}\n\n")
                    f.write("---\n")
    
    def print_summary(self):
        elapsed = time.time() - self.stats['start']
        print("\n" + "="*50)
        print("AUDIT COMPLETE")
        print("="*50)
        print(f"Files: {self.stats['files']}")
        print(f"Chunks: {self.stats['chunks']}")
        print(f"Issues: {self.stats['issues']}")
        print(f"Errors: {self.stats['errors']}")
        print(f"Time: {elapsed:.1f}s")
        print(f"Results: {AUDIT_OUTPUT_DIR}")
        print(f"Report: {HANDOFF_FILE}")

def main():
    # Quick Ollama check
    try:
        r = subprocess.run(['curl', '-s', 'http://localhost:11434/api/tags'], capture_output=True, text=True, timeout=5)
        if OLLAMA_MODEL not in r.stdout:
            print(f"Model {OLLAMA_MODEL} not found in Ollama")
            sys.exit(1)
    except:
        print("Ollama not running. Start with: ollama serve")
        sys.exit(1)
    
    auditor = FastAuditor()
    auditor.run()

if __name__ == '__main__':
    main()