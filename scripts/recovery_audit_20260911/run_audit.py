import subprocess
import json
import time
import os

claims_dir = r"C:\Users\SCM\Documents\GitHub\SCMessenger\scmessenger_audit\claims"
source_dir = r"C:\Users\SCM\Documents\GitHub\SCMessenger\scmessenger_audit\source"

files = sorted([f for f in os.listdir(claims_dir) if f.endswith('.json')])

results = []

for idx, fname in enumerate(files):
    if fname == "01_is_poison_circuit_listener.json":
        print("Skipping 01 (already done)")
        continue
    
    fbase = fname.replace('.json', '')
    print(f"\n=== [{idx+1}/19] Running {fbase} ===")
    
    claims_file = os.path.join(claims_dir, fname)
    source_file = os.path.join(source_dir, fbase + '.txt')
    
    cmd = [
        'python', '-m', 'harness.cli', 'verify',
        '--claims-file', claims_file,
        '--source-file', source_file,
        '--converge',
        '--max-tokens', '4096',
        '--task-id', f'scm_audit_{fbase}',
        '--out', f'scmessenger_audit/results/{fbase}.json'
    ]
    
    # Create results dir
    os.makedirs('scmessenger_audit/results', exist_ok=True)
    
    start = time.time()
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=600, cwd=r"C:\Users\SCM\Documents\GitHub\Harness")
        elapsed = time.time() - start
        
        # Parse output
        stdout = result.stdout
        stderr = result.stderr
        returncode = result.returncode
        
        # Try to extract JSON from stdout
        json_result = None
        for line in stdout.split('\n'):
            line = line.strip()
            if line.startswith('{'):
                try:
                    json_result = json.loads(line)
                    break
                except:
                    pass
        
        escalation_triggered = False
        plan_generated = False
        escalation_reason = ""
        
        if json_result and 'convergence' in json_result:
            conv = json_result['convergence']
            if conv.get('specialist', {}).get('escalation', {}).get('needed'):
                escalation_triggered = True
                esc = conv['specialist']['escalation']
                escalation_reason = esc.get('reason', '')
            if conv.get('specialist', {}).get('plan'):
                plan_generated = True
        
        results.append({
            'file': fname,
            'elapsed': elapsed,
            'returncode': returncode,
            'escalation_triggered': escalation_triggered,
            'plan_generated': plan_generated,
            'escalation_reason': escalation_reason,
            'stdout_tail': stdout[-2000:],
            'stderr_tail': stderr[-1000:],
        })
        
        print(f"  Time: {elapsed:.1f}s | Escalation: {escalation_triggered} | Plan: {plan_generated}")
        if escalation_triggered:
            print(f"  Reason: {escalation_reason[:100]}")
        
    except subprocess.TimeoutExpired:
        elapsed = time.time() - start
        results.append({
            'file': fname,
            'elapsed': elapsed,
            'returncode': 'TIMEOUT',
            'escalation_triggered': False,
            'plan_generated': False,
            'error': 'Timeout after 600s'
        })
        print(f"  TIMEOUT after {elapsed:.1f}s")

# Print summary
print("\n\n=== SUMMARY ===")
for r in results:
    esc = "YES" if r.get('escalation_triggered') else "no"
    plan = "YES" if r.get('plan_generated') else "no"
    print(f"{r['file']}: {r['elapsed']:.1f}s | Escalation: {esc} | Plan: {plan}")

# Save full results
with open('scmessenger_audit/results/summary.json', 'w') as f:
    json.dump(results, f, indent=2)