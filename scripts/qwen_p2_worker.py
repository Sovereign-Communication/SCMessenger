import urllib.request, json, os, re, sys

key = os.environ.get("QWEN_API_KEY") or os.environ.get("DASHSCOPE_API_KEY")
if not key:
    print("Error: set QWEN_API_KEY or DASHSCOPE_API_KEY (see ~/.config/scmorc/dashscope.env).")
    sys.exit(1)
base = "https://ws-2vzz894jwsk3t27r.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1/chat/completions"

if not os.path.exists("HANDOFF/todo/PQC_06_HYBRID_SESSION_INIT.md"):
    print("Error: PQC-06 task file not found.")
    sys.exit(1)

with open("HANDOFF/todo/PQC_06_HYBRID_SESSION_INIT.md", "r", encoding="utf-8") as f:
    task_content = f.read()

prompt = f"""
You are a senior Rust systems engineer and cryptographer. Your task is to implement PQC-06: Hybrid Session Establishment.

Here is the exact task requirement:
{task_content}

Please provide the FULL, completely updated/new contents of the following files using standard Markdown code blocks with the exact filename as the first line inside the code block (e.g. `// core/src/crypto/session_manager.rs`). DO NOT output partial files or snippets. Output the ENTIRE file content.

Files to modify/create:
1. `core/src/crypto/ratchet.rs` - add init_as_sender_hybrid, init_as_receiver_hybrid
2. `core/src/crypto/session_manager.rs` - migration to ratchet_sessions_v2, suite, transcript_hash
3. `core/src/crypto/encrypt.rs` - encryption/decryption with fallback logic calling the new hybrid methods when suite=0x02
4. `core/tests/integration_pq_session.rs` - new integration tests for PQ session

Make sure to preserve all existing classical logic.
"""

files_to_read = [
    "core/src/crypto/ratchet.rs",
    "core/src/crypto/session_manager.rs",
    "core/src/crypto/encrypt.rs"
]

prompt += "\n\n### Current Relevant File Contents:\n"
for f in files_to_read:
    try:
        with open(f, "r", encoding="utf-8") as file:
            prompt += f"\n--- {f} ---\n```\n{file.read()}\n```\n"
    except Exception as e:
        print(f"Warning: Could not read {f}: {e}")

data = {
    "model": "qwen-max",
    "messages": [
        {"role": "system", "content": "You are a senior Rust/Kotlin engineer. strictly provide full file contents in code blocks starting with // filename."},
        {"role": "user", "content": prompt}
    ],
    "temperature": 0.1
}

req = urllib.request.Request(base, headers={
    "Authorization": "Bearer " + key,
    "Content-Type": "application/json"
}, data=json.dumps(data).encode("utf-8"))

print("Sending request to Qwen-Max API...")
try:
    with urllib.request.urlopen(req) as r:
        resp = json.loads(r.read().decode("utf-8"))
        content = resp["choices"][0]["message"]["content"]
        
        os.makedirs("tmp", exist_ok=True)
        with open("tmp/qwen_p2_06_response.md", "w", encoding="utf-8") as f:
            f.write(content)
            
        print("Response received and saved!")
        
        blocks = re.findall(r"```(?:rust|)\n(.*?)\n```", content, re.DOTALL)
        for block in blocks:
            lines = block.split("\n")
            first_line = lines[0].strip()
            if first_line.startswith("// ") or first_line.startswith("# "):
                filename = first_line.replace("// ", "").replace("# ", "").replace("filename: ", "").strip()
                print(f"Writing updated {filename}...")
                os.makedirs(os.path.dirname(filename), exist_ok=True)
                with open(filename, "w", encoding="utf-8") as f:
                    f.write(block)
                        
except Exception as e:
    print("ERROR:", e)
