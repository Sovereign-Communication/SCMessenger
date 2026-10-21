import urllib.request, json, os, re

key = "sk-ws-H.YXMYHR.DVqB.MEUCIQD5eHzEkD6tQ0zdAx38ou06RCQNpAHhn9w11hc8yEBVGgIgeSUBtEFtyKEYnYBKzKS98i4D5lgjTjsQ6bfAWom6Zd4"
base = "https://ws-2vzz894jwsk3t27r.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1/chat/completions"

prompt = """
You are a senior Rust systems engineer. Your task is to implement P1-13: Hardcode Sweep (Retire 9001/9002/9010).

Please provide the FULL, completely updated contents of the following files using standard Markdown code blocks with the exact filename as the first line inside the code block (e.g. `// cli/src/main.rs`). DO NOT output partial files or snippets.

1. `cli/src/main.rs`:
   - At line 192, change the default listen multiaddr from `/ip4/0.0.0.0/tcp/9001` to `/ip4/0.0.0.0/tcp/0` (ephemeral port) and update the comment.
   - At line 1502, remove the `fallback_ports = [9001u16, 4001, 9000, 8000]` loop completely. `swarm.dial` now handles its own port ladder internally (added in P1-12). Just log if `swarm_clone.dial` fails, but don't do the fallback loop. Keep the rest of the CLI dial logic the same.

2. `cli/src/cli.rs`:
   - Change the `listen` arg default value in `Commands::Relay` from `/ip4/0.0.0.0/tcp/9001` to `/ip4/0.0.0.0/tcp/0` (around line 197). Update the comment.

3. `core/src/relay/client.rs`:
   - Change the `quic_port` default from `9002` to `443` (around line 65) and update the comment at line 50.

Provide only the file contents in code blocks. Do not omit any existing code in the files; output the entire updated files.
"""

files_to_read = [
    "cli/src/main.rs",
    "cli/src/cli.rs",
    "core/src/relay/client.rs"
]

prompt += "\n\n### Current File Contents:\n"
for f in files_to_read:
    try:
        with open(f, "r", encoding="utf-8") as file:
            prompt += f"\n--- {f} ---\n```\n{file.read()}\n```\n"
    except Exception as e:
        print(f"Warning: Could not read {f}: {e}")

data = {
    "model": "qwen-max",
    "messages": [
        {"role": "system", "content": "You are a senior Rust engineer. strictly provide full file contents."},
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
        
        with open("C:/Users/SCM/.gemini/antigravity/brain/be003baa-7229-45c9-9370-66ef55401336/scratch/qwen_p1_13_response.md", "w", encoding="utf-8") as f:
            f.write(content)
            
        print("Response received and saved!")
        
        blocks = re.findall(r"```[a-z]*\n(.*?)\n```", content, re.DOTALL)
        for block in blocks:
            lines = block.split("\n")
            first_line = lines[0].strip()
            if first_line.startswith("// ") or first_line.startswith("# "):
                filename = first_line.replace("// ", "").replace("# ", "").strip()
                if filename in files_to_read:
                    print(f"Writing updated {filename}...")
                    with open(filename, "w", encoding="utf-8") as f:
                        f.write("\n".join(lines[1:]))
                        
except Exception as e:
    print("ERROR:", e)
