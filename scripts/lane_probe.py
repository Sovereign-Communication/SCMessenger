#!/usr/bin/env python3
"""lane_probe.py -- re-measure every lane in scripts/lanes.json.

Lanes die without warning. Run weekly, or immediately after any 401/404, and
never trust a roster past its `_expires` note.

    python scripts/lane_probe.py            # report only
    python scripts/lane_probe.py --write    # also update status/latency in lanes.json

Never prints key values. Exit 0 unless --write fails.
"""
import argparse, json, pathlib, time, urllib.error, urllib.request, os, sys

REPO = pathlib.Path(__file__).resolve().parent.parent
CFG = pathlib.Path.home() / ".config" / "scmorc"
LANES = REPO / "scripts" / "lanes.json"

# A real task, not a ping. "Reply ok" passes on lanes that cannot write code.
TASK = ("Return ONLY a unified diff, no prose. In Rust `src/lib.rs` the line "
        "`let v = map.get(k).unwrap();` must not panic. Return "
        "`Err(Error::Missing)` from a fn returning `Result<T, Error>`.")


def load_key(key_file, key_var):
    p = CFG / (key_file or "")
    if p.exists():
        for line in p.read_text(encoding="utf-8", errors="ignore").splitlines():
            if line.strip().startswith(key_var + "="):
                return line.split("=", 1)[1].strip().strip('"').strip("'")
    return os.environ.get(key_var or "")


def knob(lane):
    if lane.get("provider") != "openrouter":
        return None
    return {"exclude": True} if "exclude" in (lane.get("quirks") or "").lower() \
        else {"effort": "low"}


def probe(lane, timeout=200):
    key = load_key(lane.get("key_file"), lane.get("key_var"))
    if not key:
        return "NO-KEY", None, "credential absent on this host"
    body = {"model": lane["model"],
            "messages": [{"role": "user", "content": TASK}],
            "max_tokens": 1000, "temperature": 0}
    k = knob(lane)
    if k is not None:
        body["reasoning"] = k
    h = {"Content-Type": "application/json", "Authorization": f"Bearer {key}",
         "User-Agent": "scm-lane-probe/1.0"}
    if lane["provider"] == "openrouter":
        h["HTTP-Referer"] = "https://github.com/Sovereign-Communication/SCMessenger"
        h["X-Title"] = "SCMessenger lane probe"
    try:
        t0 = time.time()
        req = urllib.request.Request(lane["endpoint"], data=json.dumps(body).encode(),
                                     headers=h, method="POST")
        d = json.loads(urllib.request.urlopen(req, timeout=timeout).read())
        dt = time.time() - t0
        if "choices" not in d:
            return "ERROR", None, json.dumps(d)[:90]
        c = d["choices"][0]["message"].get("content") or ""
        if not c.strip():
            return "EMPTY", dt, "content empty -- reasoning-token trap, adjust knob"
        return "OK", dt, f"{len(c)} chars"
    except urllib.error.HTTPError as e:
        body_txt = ""
        try:
            body_txt = e.read().decode()[:80].replace("\n", " ")
        except Exception:
            pass
        return f"HTTP{e.code}", None, body_txt
    except Exception as e:
        return "ERROR", None, f"{type(e).__name__}: {str(e)[:70]}"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--write", action="store_true", help="update lanes.json in place")
    args = ap.parse_args()

    roster = json.loads(LANES.read_text(encoding="utf-8"))
    print(f"{'LANE':<32}{'WAS':<10}{'NOW':<10}{'LAT':>8}  NOTE")
    print("-" * 96)
    changed = []
    for L in roster["lanes"]:
        if not L.get("endpoint"):          # agy / native are not HTTP-probeable
            print(f"{L['id']:<32}{L['status']:<10}{'SKIP':<10}{'-':>8}  not an HTTP lane")
            continue
        was = L["status"]
        st, dt, note = probe(L)
        if st != was:
            changed.append((L["id"], was, st))
        if args.write:
            L["status"] = st
            if dt:
                L["latency_s"] = round(dt, 2)
        print(f"{L['id']:<32}{was:<10}{st:<10}"
              f"{(f'{dt:.2f}s' if dt else '-'):>8}  {note}")

    print("-" * 96)
    live = sum(1 for L in roster["lanes"] if L.get("endpoint") and L["status"] == "OK")
    print(f"{live} HTTP lanes live")
    if changed:
        print("\nSTATUS CHANGES (roster was stale):")
        for i, a, b in changed:
            print(f"   {i}: {a} -> {b}")
    else:
        print("no status changes -- roster still accurate")

    if args.write:
        roster["_probed"] = time.strftime("%Y-%m-%d")
        LANES.write_text(json.dumps(roster, indent=2) + "\n", encoding="utf-8")
        print(f"\n[OK] wrote {LANES.relative_to(REPO)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
