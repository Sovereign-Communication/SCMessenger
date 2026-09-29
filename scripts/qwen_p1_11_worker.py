import urllib.request, json, os, re

key = "sk-ws-H.YXMYHR.DVqB.MEUCIQD5eHzEkD6tQ0zdAx38ou06RCQNpAHhn9w11hc8yEBVGgIgeSUBtEFtyKEYnYBKzKS98i4D5lgjTjsQ6bfAWom6Zd4"
base = "https://ws-2vzz894jwsk3t27r.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1/chat/completions"

prompt = """
You are a senior Rust and Kotlin systems engineer. Your task is to implement P1-11: Listen-Side Adaptive Port Selection.

Please provide the FULL, completely updated contents of the following files using standard Markdown code blocks with the exact filename as the first line inside the code block (e.g. `// core/src/transport/multiport.rs` or `// cli/src/main.rs`). DO NOT output partial files or snippets.

1. `core/src/transport/multiport.rs`:
   - Expand `MultiPortConfig` to include `pub preferred_port: Option<u16>`. Update its `Default` impl to set `preferred_port: None`.
   - Update `generate_listen_addresses` to prepend the `preferred_port` BEFORE `COMMON_PORTS`. It must add both `/tcp/{port}` and `/tcp/{port}/ws` for the preferred port if enabled. Oh wait, actually just add it as a normal tcp port for now (the WS logic is in swarm.rs, or you can add it here if it's cleaner).

2. `core/src/transport/swarm.rs`:
   - Locate and remove the literal `9002/ws` bind (around line 2032).
   - Collect addresses from `NewListenAddr` into a shared state and add an accessor `pub async fn get_bound_addresses(&self) -> Result<Vec<Multiaddr>>` to `SwarmHandle` (backed by a new `SwarmCommand::GetBoundAddresses`).

3. `cli/src/main.rs`:
   - Map the `--port` argument to `preferred_port` in `MultiPortConfig` and pass `Some(config)` to `start_swarm_with_config` (line 1395/2465).
   - Remove the redundant CLI WS bind logic (the `swarm.listen_on(ws_addr)` around line 1406).

4. `core/src/mobile_bridge.rs`:
   - Update `start_swarm` to construct `Some(MultiPortConfig)` with `preferred_port` derived from `listen_multiaddr` and pass it to `start_swarm_with_config`.

Provide only the file contents in code blocks. Do not omit any existing code in the files; output the entire updated files.
"""

files_to_read = [
    "core/src/transport/multiport.rs",
    "core/src/transport/swarm.rs",
    "cli/src/main.rs",
    "core/src/mobile_bridge.rs"
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
        {"role": "system", "content": "You are a senior Rust/Kotlin engineer. strictly provide full file contents."},
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
        
        with open("C:/Users/SCM/.gemini/antigravity/brain/be003baa-7229-45c9-9370-66ef55401336/scratch/qwen_p1_11_response.md", "w", encoding="utf-8") as f:
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
                        f.write(block)
                        
except Exception as e:
    print("ERROR:", e)
