import urllib.request, json, os, re

key = "sk-ws-H.YXMYHR.DVqB.MEUCIQD5eHzEkD6tQ0zdAx38ou06RCQNpAHhn9w11hc8yEBVGgIgeSUBtEFtyKEYnYBKzKS98i4D5lgjTjsQ6bfAWom6Zd4"
base = "https://ws-2vzz894jwsk3t27r.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1/chat/completions"

prompt = """
You are a senior technical writer and Rust systems engineer. Your task is to implement P1-13 documentation updates (Hardcode Sweep).

The system now uses adaptive port selection (a port ladder: negotiated, last-known-good, 443, 80, 8080) instead of hardcoded 9001 and 9002 ports.
Please provide the FULL, completely updated contents of the following documentation file using standard Markdown code blocks. Replace references to fixed "port 9001" and "port 9002" with explanations of adaptive port selection and the fallback ladder. 

1. `docs/RELAY_OPERATOR_GUIDE.md`:
   - Keep 9001 as the documented preferred-first default for relay operators (so they still open 9001 in firewalls), but explain that clients now use an adaptive port ladder (443, 80, 8080, etc.) if 9001 fails, and relay operators should consider allowing 443.

Provide only the file contents in code blocks. Do not omit any existing text.
"""

files_to_read = [
    "docs/RELAY_OPERATOR_GUIDE.md"
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
        {"role": "system", "content": "You are a senior technical writer. strictly provide full file contents."},
        {"role": "user", "content": prompt}
    ],
    "temperature": 0.2
}

req = urllib.request.Request(base, headers={
    "Authorization": "Bearer " + key,
    "Content-Type": "application/json"
}, data=json.dumps(data).encode("utf-8"))

print("Sending request to Qwen-Max API for docs update...")
try:
    with urllib.request.urlopen(req) as r:
        resp = json.loads(r.read().decode("utf-8"))
        content = resp["choices"][0]["message"]["content"]
        
        blocks = re.findall(r"```[a-z]*\n(.*?)\n```", content, re.DOTALL)
        for block in blocks:
            lines = block.split("\n")
            first_line = lines[0].strip()
            # Try to guess the filename or just apply it
            filename = "docs/RELAY_OPERATOR_GUIDE.md"
            print(f"Writing updated {filename}...")
            # If it included the filename comment, strip it
            if first_line.startswith("// ") or first_line.startswith("# "):
                block_content = "\n".join(lines[1:])
            else:
                block_content = block
                
            with open(filename, "w", encoding="utf-8") as f:
                f.write(block_content)
                
        print("Docs updated successfully by Qwen!")
                        
except Exception as e:
    print("ERROR:", e)
