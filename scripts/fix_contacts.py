import re

with open('android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt', 'r') as f:
    content = f.read()

# Pattern to find Contact(...) calls that are missing lastKnownDeviceId
pattern = r'(uniffi\.api\.Contact\([^)]+notes\s*=\s*[^,)]+)(\s*\))'

def add_last_known_device_id(match):
    before = match.group(1)
    after = match.group(2)
    # Check if lastKnownDeviceId is already there
    if 'lastKnownDeviceId' in before:
        return match.group(0)
    # Add it
    return before + ',\n                                lastKnownDeviceId = null' + after

content = re.sub(pattern, add_last_known_device_id, content)

with open('android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt', 'w') as f:
    f.write(content)

print("Fixed MeshRepository.kt")
