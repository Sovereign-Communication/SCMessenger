import urllib.request, json, os, re

key = "sk-ws-H.YXMYHR.DVqB.MEUCIQD5eHzEkD6tQ0zdAx38ou06RCQNpAHhn9w11hc8yEBVGgIgeSUBtEFtyKEYnYBKzKS98i4D5lgjTjsQ6bfAWom6Zd4"
base = "https://ws-2vzz894jwsk3t27r.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1/chat/completions"

prompt = """
You are a senior Android Kotlin developer. Your task is to implement "P1-ANDROID-MDNS: Phone's mDNS discovery resolves its own broadcast as a discovered peer".
The goal is to filter out the local node's own peer-id when an mDNS service is resolved, preventing self-dialing.

Please provide the FULL, completely updated contents of the following files using standard Markdown code blocks with the exact filename as the first line inside the code block (e.g. `// android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt`).

1. `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt`:
   - Locate `onServiceResolved` (~line 215) and extract the peer-id from the resolved service.
   - Filter out the service if the peer-id matches the local node's identity (`cachedPeerId` or similar).

2. `android/app/src/test/java/com/scmessenger/android/transport/MdnsServiceDiscoveryTest.kt`:
   - Add a unit test to verify that if the resolved service's peer-id matches the local identity's peer-id, `onLanPeerResolved` is NOT invoked.
   - Add a unit test for the normal case: a resolved service record with a *different* peer-id correctly triggers `onLanPeerResolved`.

Provide only the file contents in code blocks. Do not omit any existing code in the files; output the entire updated files.
"""

files_to_read = [
    "android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt",
    "android/app/src/test/java/com/scmessenger/android/transport/MdnsServiceDiscoveryTest.kt"
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
        {"role": "system", "content": "You are a senior Kotlin engineer. Follow the instructions strictly and provide full file contents."},
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
        with open("C:/Users/SCM/.gemini/antigravity/brain/be003baa-7229-45c9-9370-66ef55401336/scratch/qwen_mdns_response.md", "w", encoding="utf-8") as f:
            f.write(content)
            
        print("Response received and saved to qwen_mdns_response.md!")
        
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
