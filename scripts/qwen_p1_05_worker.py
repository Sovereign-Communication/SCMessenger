import urllib.request, json, os, re

key = "sk-ws-H.YXMYHR.DVqB.MEUCIQD5eHzEkD6tQ0zdAx38ou06RCQNpAHhn9w11hc8yEBVGgIgeSUBtEFtyKEYnYBKzKS98i4D5lgjTjsQ6bfAWom6Zd4"
base = "https://ws-2vzz894jwsk3t27r.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1/chat/completions"

prompt = """
You are a senior Rust and Android developer. Your task is to implement "P1-05 Build-provenance stamps" for the SCMessenger project.
The goal is to surface a build stamp (e.g., git hash, branch, and build date) from `core/build.rs` into the Rust core, expose it via UniFFI, print it in the CLI, and expose it in the Android Kotlin Repository.

Please provide the FULL, completely updated contents of the following files using standard Markdown code blocks with the exact filename as the first line inside the code block (e.g. `// core/build.rs`).

1. `core/build.rs`: Modify this to run `git rev-parse --short HEAD` and `git rev-parse --abbrev-ref HEAD` and a timestamp. Emit them as `cargo:rustc-env=SCM_BUILD_STAMP=...`. If git fails, fallback to "unknown".
2. `core/src/api.udl`: In the `namespace api { ... };` block, add a global function `string get_build_provenance();`.
3. `core/src/lib.rs`: Implement the global function `pub fn get_build_provenance() -> String`. Use `option_env!("SCM_BUILD_STAMP").unwrap_or("development build").to_string()`.
4. `cli/src/main.rs`: Find the CLI startup or version logic and print the provenance stamp.
5. `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`: Add a function `fun getBuildProvenance(): String { return uniffi.api.getBuildProvenance() }` to the interface and implementation.

Provide only the file contents in code blocks. Do not omit any existing code in the files; output the entire updated files.
"""

files_to_read = [
    "core/build.rs",
    "core/src/api.udl",
    "core/src/lib.rs",
    "cli/src/main.rs",
    "android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt"
]

prompt += "\n\n### Current File Contents:\n"
for f in files_to_read:
    try:
        with open(f, "r", encoding="utf-8") as file:
            prompt += f"\n--- {f} ---\n```\n{file.read()}\n```\n"
    except Exception as e:
        print(f"Warning: Could not read {f}: {e}")

data = {
    "model": "qwen-plus",
    "messages": [
        {"role": "system", "content": "You are a senior Rust/Kotlin engineer. Follow the instructions strictly and provide full file contents."},
        {"role": "user", "content": prompt}
    ],
    "temperature": 0.1
}

req = urllib.request.Request(base, headers={
    "Authorization": "Bearer " + key,
    "Content-Type": "application/json"
}, data=json.dumps(data).encode("utf-8"))

print("Sending request to Qwen API...")
try:
    with urllib.request.urlopen(req) as r:
        resp = json.loads(r.read().decode("utf-8"))
        content = resp["choices"][0]["message"]["content"]
        
        # Save raw response
        with open("C:/Users/SCM/.gemini/antigravity/brain/be003baa-7229-45c9-9370-66ef55401336/scratch/qwen_response.md", "w", encoding="utf-8") as f:
            f.write(content)
            
        print("Response received and saved to qwen_response.md!")
        
        # Parse blocks
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
