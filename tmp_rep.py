import sys
def rep_file(path, pairs):
    with open(path, encoding="utf-8", newline="") as f:
        s = f.read()
    assert "\r\n" not in s
    for old, new, count in pairs:
        n = s.count(old)
        assert n == count, (n, old[:100])
        s = s.replace(old, new)
    with open(path, "w", encoding="utf-8", newline="") as f:
        f.write(s)
    print("ok", path, len(pairs))
