from pathlib import Path
import re

p = Path(r"C:/Users/SCM/Documents/GitHub/SCMessenger/HANDOFF/audit/RECENT_WORK_AUDIT_2026-09-09.md")
t = p.read_text(encoding="utf-8")
# map common status emoji to tags
repl = {
    "\u2705": "[OK]",
    "\u274c": "[ERROR]",
    "\u26a0\ufe0f": "[WARNING]",
    "\u26a0": "[WARNING]",
}
for k, v in repl.items():
    t = t.replace(k, v)
# strip remaining emoji
t2 = re.sub(r"[\U0001F300-\U0001FAFF\u2600-\u27BF\uFE0F\u200d]", "", t)
p.write_text(t2, encoding="utf-8")
print("changed", t != t2, "len", len(t2))
