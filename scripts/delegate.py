#!/usr/bin/env python3
"""delegate.py -- route one task file to the cheapest healthy lane, validate the
result, escalate on failure, and return ONE verdict line.

Design goal: the orchestrator's context stays clean. Worker output goes to a
file; stdout is a short verdict. Only a BLOCKED verdict needs a human.

    python scripts/delegate.py --task HANDOFF/todo/FOO.md --tier scoped
    python scripts/delegate.py --task T1.md --tier micro --files core/src/a.rs
    python scripts/delegate.py --list-lanes

Exit codes:  0 = PASS   3 = EMPTY/invalid after retries   4 = BLOCKED (no lane)
             5 = task file missing
"""
import argparse, json, os, pathlib, sys, time, urllib.error, urllib.request

REPO = pathlib.Path(__file__).resolve().parent.parent
CFG = pathlib.Path.home() / ".config" / "scmorc"
LANES = REPO / "scripts" / "lanes.json"
OUTDIR = REPO / "tmp" / "delegate"


def load_key(key_file, key_var):
    p = CFG / key_file
    if not p.exists():
        return os.environ.get(key_var)
    for line in p.read_text(encoding="utf-8", errors="ignore").splitlines():
        line = line.strip()
        if line.startswith(key_var + "="):
            return line.split("=", 1)[1].strip().strip('"').strip("'")
    return os.environ.get(key_var)


def reasoning_knob(lane):
    """Free reasoning models burn the whole max_tokens budget on hidden reasoning
    and return content=''. reasoning.effort=low is the measured fix.

    This parameter is OpenRouter-only. Google, NVIDIA NIM, Cerebras and Groq
    reject or ignore it, so it must never be sent to them."""
    if lane.get("provider") != "openrouter":
        return None
    q = (lane.get("quirks") or "").lower()
    if "exclude" in q:
        return {"exclude": True}
    return {"effort": "low"}


def call(lane, prompt, max_tokens, timeout):
    key = load_key(lane.get("key_file", ""), lane.get("key_var", ""))
    if not key:
        raise RuntimeError(f"no credential for lane {lane['id']}")
    body = {
        "model": lane["model"],
        "messages": [{"role": "user", "content": prompt}],
        "max_tokens": max_tokens,
        "temperature": 0,
    }
    knob = reasoning_knob(lane)
    if knob is not None:
        body["reasoning"] = knob
    # Per-lane extra body params merged verbatim (e.g. z.ai/GLM "thinking" toggle,
    # which must be disabled on hybrid-reasoning models to avoid the empty-content trap).
    extra = lane.get("body_extra")
    if isinstance(extra, dict):
        body.update(extra)
    headers = {
        "Content-Type": "application/json",
        "Authorization": f"Bearer {key}",
        # Groq sits behind Cloudflare and 1010s a missing/!default User-Agent.
        "User-Agent": "scm-delegate/1.0",
    }
    if lane["provider"] == "openrouter":
        headers["HTTP-Referer"] = "https://github.com/Sovereign-Communication/SCMessenger"
        headers["X-Title"] = "SCMessenger delegate"
    req = urllib.request.Request(lane["endpoint"], data=json.dumps(body).encode(),
                                 headers=headers, method="POST")
    t0 = time.time()
    with urllib.request.urlopen(req, timeout=timeout) as r:
        data = json.loads(r.read())
    if "choices" not in data:
        raise RuntimeError(f"upstream: {json.dumps(data)[:160]}")
    msg = data["choices"][0]["message"]
    return (msg.get("content") or ""), time.time() - t0, data.get("model", lane["model"])


def build_prompt(task_text, files, mode):
    parts = [task_text.strip(), ""]
    if files:
        parts.append("## Source files")
        for f in files:
            p = REPO / f
            if not p.exists():
                parts.append(f"\n### {f}\n(FILE NOT FOUND -- do not invent its contents)")
                continue
            parts.append(f"\n### {f}\n```\n{p.read_text(encoding='utf-8', errors='ignore')}\n```")
    parts.append("")
    if mode == "diff":
        parts.append("## Output contract\nReturn ONLY a unified diff in a ```diff fenced block. "
                     "No prose before or after. If you cannot produce a correct diff, reply with "
                     "exactly: BLOCKED: <one line reason>")
    else:
        parts.append("## Output contract\nReturn the complete file(s) in fenced blocks, each preceded "
                     "by its path. If you cannot, reply with exactly: BLOCKED: <one line reason>")
    return "\n".join(parts)


def validate(text, mode):
    """Return (ok, reason). Catches the empty-content trap and refusals."""
    t = (text or "").strip()
    if not t:
        return False, "empty content (reasoning-token trap -- retry with different knob)"
    if t.upper().startswith("BLOCKED:"):
        return False, t.splitlines()[0][:160]
    if mode == "diff" and "```" not in t and "@@" not in t and "--- " not in t:
        return False, "no diff or fenced block in output"
    if len(t) < 20:
        return False, f"output too short to be real ({len(t)} chars)"
    return True, "ok"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--task", help="Path to task markdown file")
    ap.add_argument("--tier", default="scoped",
                    choices=["micro", "scoped", "reasoning", "long-context", "shell"])
    ap.add_argument("--files", nargs="*", default=[])
    ap.add_argument("--mode", choices=["diff", "full"], default="diff")
    ap.add_argument("--max-tokens", type=int, default=4000)
    ap.add_argument("--timeout", type=int, default=240)
    ap.add_argument("--max-lanes", type=int, default=3,
                    help="How many lanes to try before declaring BLOCKED")
    ap.add_argument("--lane", help="Force a specific lane id, skipping the ladder")
    ap.add_argument("--list-lanes", action="store_true")
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    roster = json.loads(LANES.read_text(encoding="utf-8"))
    lanes = roster["lanes"]

    if args.list_lanes:
        print(f"{'ID':<30}{'PROVIDER':<12}{'COST':<12}{'STATUS':<9}{'LAT':>7}{'CTX':>9}  TIERS")
        for L in sorted(lanes, key=lambda x: x.get("latency_s") or 999):
            lat, ctx = L.get("latency_s"), L.get("context")
            print(f"{L['id']:<30}{L['provider']:<12}{L['cost_class']:<12}{L['status']:<9}"
                  f"{(f'{lat:.2f}s' if lat else '-'):>7}{(f'{ctx//1000}k' if ctx else '-'):>9}"
                  f"  {','.join(L.get('tiers', []))}")
        print(f"\nprobed {roster['_probed']} -- {roster['_expires']}")
        print(f"dead: {', '.join(d['id'] for d in roster['dead'])}")
        return 0

    if not args.task:
        print("[FAIL] --task is required"); return 5
    tp = pathlib.Path(args.task)
    if not tp.is_absolute():
        tp = REPO / args.task
    if not tp.exists():
        print(f"[FAIL] task file not found: {tp}"); return 5
    task_text = tp.read_text(encoding="utf-8", errors="ignore")

    # A task that must RUN something cannot go to an HTTP lane. Catch this
    # before spending a call: three lanes independently returned BLOCKED on a
    # `gh run view` task because none of them has a shell.
    SHELL_MARKERS = ("gh run view", "gh api", "gh pr", "cargo ", "./gradlew",
                     "gradlew ", "adb ", "git log", "git ls-remote", "python scripts/")
    if args.tier != "shell":
        hits = [m for m in SHELL_MARKERS if m in task_text]
        if hits:
            print(f"[BLOCKED] task requires a shell (found {hits[:3]}) but tier="
                  f"'{args.tier}' routes to HTTP-only lanes that cannot execute.")
            print("[NEXT] re-dispatch with --tier shell, which routes to agy "
                  "(pass --add-dir <repo> and an exact --model), or run it yourself.")
            return 4

    # Selection function over measured properties -- NOT a fixed ranking.
    # Filter to lanes that can actually do the job, then order by what the
    # caller is optimising for. Re-derived every run from the current roster,
    # so a lane dying changes routing automatically.
    if args.lane:
        ladder = [L for L in lanes if L["id"] == args.lane]
        if not ladder:
            print(f"[FAIL] unknown lane {args.lane}"); return 4
    else:
        ladder = [L for L in lanes
                  if L.get("status") == "OK"
                  and args.tier in L.get("tiers", [])
                  and L.get("endpoint")                 # API-callable from here
                  and L["cost_class"] == "free"]        # never auto-spend metered
        need = len(build_prompt(task_text, args.files, args.mode))
        # Drop lanes whose context cannot hold the prompt (~3.5 chars/token).
        ladder = [L for L in ladder
                  if not L.get("context") or L["context"] * 3.5 > need]
        ladder.sort(key=lambda L: L.get("latency_s") or 999)

    if not ladder:
        print(f"[BLOCKED] no free lane satisfies tier='{args.tier}' at this prompt size. "
              f"Re-probe (lane_probe.py) in case the roster is stale, then escalate "
              f"deliberately: agy-gemini (shell-capable) -> agy-claude (metered) -> native verdict.")
        return 4

    prompt = build_prompt(task_text, args.files, args.mode)
    if args.dry_run:
        print(f"[INFO] tier={args.tier} prompt={len(prompt)} chars")
        print(f"[INFO] ladder: {' -> '.join(L['id'] for L in ladder[:args.max_lanes])}")
        return 0

    OUTDIR.mkdir(parents=True, exist_ok=True)
    stem = tp.stem
    attempts = []
    for L in ladder[:args.max_lanes]:
        try:
            text, dt, used = call(L, prompt, args.max_tokens, args.timeout)
            ok, reason = validate(text, args.mode)
            if ok:
                out = OUTDIR / f"{stem}__{L['id']}.md"
                out.write_text(
                    f"# delegate result\n\ntask: {args.task}\nlane: {L['id']}\n"
                    f"model: {used}\nlatency: {dt:.1f}s\ntier: {args.tier}\n\n---\n\n{text}\n",
                    encoding="utf-8")
                print(f"[PASS] {stem} via {L['id']} ({used}) {dt:.1f}s -> {out.relative_to(REPO)}")
                for a in attempts:
                    print(f"[INFO] earlier attempt {a}")
                return 0
            attempts.append(f"{L['id']}: {reason}")
        except urllib.error.HTTPError as e:
            body = ""
            try:
                body = e.read().decode()[:110].replace("\n", " ")
            except Exception:
                pass
            attempts.append(f"{L['id']}: HTTP{e.code} {body}")
        except Exception as e:
            attempts.append(f"{L['id']}: {type(e).__name__} {str(e)[:100]}")

    print(f"[BLOCKED] {stem} -- {len(attempts)} lane(s) failed, no usable output")
    for a in attempts:
        print(f"   {a}")
    print("[NEXT] escalate: agy (--add-dir + exact --model) -> agy-claude -> native verdict")
    return 3


if __name__ == "__main__":
    sys.exit(main())
