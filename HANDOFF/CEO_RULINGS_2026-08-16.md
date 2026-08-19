# CEO Rulings -- 2026-08-16

Status: Active
From: CEO
To: CTO
Re: `HANDOFF/CTO_STATE.md` §7 "OPEN -- do not guess"

**Why this file exists.** These rulings were sent twice over session messaging
and queued against a session that was not running, so the CTO recorded "no CEO
reply" while the answers sat undelivered. The repo is the reliable channel;
session messages are not. Decisions that block work go here from now on.

---

## §7.1 -- Android sources deleted in `ebf5411b`: RESOLVED

**Operator ruling (2026-08-16):** the only deliberate deletions were on
`feature/josh-build-single-transport`, the paranoid/stripped-down variant. None
of it was meant to reach the mainline. **Any Android deletion on main is
unintentional.** The restore was correct. Do not revert it.

**Follow-on defect found while confirming this -- the restore was incomplete.**

`ebf5411b` is on `origin/main` and ~15 active branches; it removed 1,658 lines
across 20 files. All 8 deleted source files are restored and verified present.
`AndroidManifest.xml` was **not** restored -- 106 lines now against 158 before --
and three components are missing their registrations:

| Component | Source file | Manifest entry |
|---|---|---|
| MeshVpnService | present | **MISSING** |
| BootReceiver | present | **MISSING** |
| ShareReceiver | present | **MISSING** |

An undeclared Service or BroadcastReceiver cannot be started by Android. These
compile, pass lint, and are dead at runtime -- the class `CLAUDE.md` opens with.
Read consequences: no mesh restart after reboot; VPN transport cannot start;
share-to-app and APK sharing dead, which touches D2.

Six further files are short of their pre-deletion length -- MeshForegroundService
-15, PerformanceMonitor -17, SettingsScreen -19, SettingsViewModel -18. **These
are not being called defects.** Line count is a signal, not a verdict, and some
files legitimately moved on (MeshApplication +9, build.gradle +113 versus
pre-deletion), so a blanket restore-to-before would be wrong. Needs a per-hunk
read against `ebf5411b`.

**Priority:** the three manifest registrations land before the tag. The six
partially-stripped files are the CTO's to sequence; anything deferred goes to
`POST_TAG_QUEUE.md` Section 3, not into a commit message.

## §7.2 -- Josh single-transport: CLOSED

Operator already ruled: not the v0.4.0 default, ships as v0.3.9 if at all. No
CEO input needed. Do not spend further on it.

**Isolation of the variant:** operator ruling 2026-08-16 -- **isolate when safe,
no rush.** This is explicitly *not* pre-tag work. After the tag, the josh variant
moves somewhere it cannot reach main by accident. Logged as S4-11.

## §7.3 -- README framing: APPROVED. Ship it.

The honest-first tone is right and is to be **kept, not softened** before the
tag. For a security product over-claiming is fatal and under-claiming is
recoverable. "The cryptography has been reviewed only by the people and tools
that wrote it. That is not a credential" is the most valuable sentence on the
page -- it is what makes every other claim believable. Keep the three-sentence
threat model including what the product does *not* protect against.

Verified rather than taken on trust, per the CTO's own §8 lesson:

- All 12 referenced paths resolve. No broken links.
- `LICENSE` is genuinely the Unlicense.
- ML-KEM-768 and ML-DSA are really present in `core/src/crypto/`. Nothing in the
  crypto table is unbacked.

**One substantive edit required before tag, and it is not about tone.** The
opening paragraph asserts in present tense that "the transports race, so a
message takes whichever one is actually working" and "if there is no internet,
phones in range still talk." Two P0s currently in `HANDOFF/archive/` say that is
not yet true: `P0_NO_RELAY_FALLBACK_FOR_ROAMING_PEERS` (Open, observed live) and
`P0_NO_MOBILE_BOOTSTRAP_MEANS_NO_OFF_LAN_RENDEZVOUS` (Open). The status section
below is honest, but that opening sits above the caveat and reads as shipped
behaviour.

Mechanism is the CTO's call -- hedge to intent language, or hold the wording and
let D4 prove it before the tag. What must not happen is that sentence shipping
as present-tense fact while those two P0s are open. It is the one line on the
page that could later read as a false claim rather than an honest limitation,
which would undo what the rest of the page earns.

## §7.4 -- Dependency-deferral trigger: CONFIRMED

Logged as **S4-1** in `POST_TAG_QUEUE.md`.

- **Trigger:** first working day after the v0.4.0 tag. Not "post-tag" vaguely.
- **Method:** the 13 dependabot PRs are batch-merged as **one decision**, not
  thirteen. (#64, #65, #67, #69, #99, #100, #102, #103, #106, #107, #108, #141,
  #142)
- **Scope:** 7 vulnerabilities on the default branch, 3 high.
- **Expiry:** if still parked 60 days after the tag it returns to the CEO for a
  keep/kill ruling. Permanent deferral is a decision, made in the open.

The CTO's framing -- right to defer for shipping, wrong to leave deferred long on
a security product -- is the framing in the register. Agreed as written.

---

## Still outstanding from the CEO to the CTO

1. **Six-P0 disposition**, `POST_TAG_QUEUE.md` Section 2. Tag-blocking.
2. **Three manifest registrations** above. Tag-blocking.
3. Per-hunk read of the six partially-stripped Android files. Sequence at CTO
   discretion; defer explicitly if cosmetic.

Nothing else is awaited from the CEO. If a decision is needed and messaging is
unreliable, write the question into `HANDOFF/CTO_STATE.md` §7 and it will be
answered here.

---

## Addendum -- CEO review of CTO delivery, 2026-08-16 (later)

Reviewed `scripts/check_wiring.py`, `AGENTS.md` Rule 15, and
`docs/security/W1_FIX_VALIDATION_2026-08-16.md`. Verified by running the gate,
not by reading its description.

**`check_wiring.py` is accepted and is the right response.** It was asked to fix
three manifest registrations; it built the gate that catches the entire class.
The gate is self-validating -- run against the current tree it reports exactly
the three components the CEO found by hand:

```
[C3_MANIFEST_MISSING] service/BootReceiver.kt:23      - BootReceiver
[C3_MANIFEST_MISSING] service/MeshVpnService.kt:21    - MeshVpnService
[C3_MANIFEST_MISSING] utils/ShareReceiver.kt:31       - ShareReceiver
```

A gate proven by the defect that motivated it is worth more than one proven by
its own test suite. Exit code 1. Wire it into CI before the tag.

**The fix is still outstanding.** `AndroidManifest.xml` remains 106 lines against
158 pre-deletion; all three are still unregistered. Gate built, defect confirmed,
repair not yet applied. Tag-blocking, unchanged.

**The gate found more than the CEO did, and one item may be D4-blocking.**

`JoinMeshScreen` has zero callers. Verified independently: no reference anywhere
in `android/app/src/main` outside its own file, and `MeshApp.kt` registers no
join route. The whole QR join wizard behind it is dead --  `QrScannerView`,
`ParsingView`, `ConnectingView`, `SuccessView`, `ErrorView`.

**Open question for the CTO, treat as D4-relevant until answered:** if the QR
wizard is the intended way two strangers pair, then D4 cannot pass on the current
Android build and this belongs beside the six P0s rather than in a cleanup pile.
If pairing is meant to happen another way -- contact provisioning, mDNS
auto-discovery -- then this is dead code and defers to S4. **Do not sweep it
until that question is answered.** This is a reachability question about the
north-star flow, not a tidiness question.

**The restore-did-not-rewire pattern repeats one level up.** `DiagnosticsScreen`,
`ApkShareDialog`, and `ApkShareManager` were restored as files after `ebf5411b`
and have zero callers; `Screen.Diagnostics ("diagnostics")` is navigated to but
never registered in the NavHost, so it lands nowhere. `SecurityUtils`, which
`ebf5411b` *added*, also has zero callers. Restoring a file is not restoring a
feature -- same lesson as the manifest, one layer higher.

**Ruling on the rest of the C1/C4 output: defer to S4, do not fix before tag.**
Dead code does not break what works, and a large sweep during a ship sprint is
how regressions arrive. Log the inventory in `POST_TAG_QUEUE.md` Section 3.
Exception: anything the JoinMeshScreen answer promotes to D4.

**`AGENTS.md` Rule 15 (no silent truncation): endorsed without reservation.**
Recording that `scripts/pr_scope.sh` printed `[OK] clear of
core/src/{crypto,transport}` while six merge-blocked files sat past an API cap --
and reported 100 commits where git counts 204 -- is the project catching its own
tooling lying about security-gated files. "Limits belong on what you DO, never on
what you can SEE" is now a standing rule and applies to CEO reporting too.

**One doc-sync gap:** `check_wiring.py` cites "AGENTS.md Rule 16 (Reachability &
Wiring Enforcement)". No Rule 16 exists in `AGENTS.md`; the new rule landed as
15. Either write Rule 16 or fix the citation -- a gate pointing at a rule that
does not exist undermines both.

**W1 fix validation (PR #172): accepted as evidence.** It cites file and line for
each claim and answers the right adversarial questions -- is the bypass closed,
did the removal break anything, is there a regression on legitimate reconnects.
That is the standard. Note it is a security-module change under
`core/src/transport/`, so it still needs adversarial review sign-off before merge
per the merge-block rule; this validation does not substitute for it.

**Priority order, CEO ruling:**

1. Answer the JoinMeshScreen reachability question -- cheap, and it decides whether D4 is provable at all
2. Three manifest registrations -- tag-blocking
3. Six-P0 disposition -- tag-blocking
4. Wire `check_wiring.py` into CI -- before tag, so this class cannot recur
5. Everything else in the gate output -- S4
