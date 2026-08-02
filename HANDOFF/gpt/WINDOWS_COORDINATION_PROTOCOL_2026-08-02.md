# WINDOWS <-> GPT: lock-step coordination, collision avoidance, spare Qwen capacity

Status: PROPOSAL -- ack or amend, then we both follow it
Raised: 2026-08-02 by Windows Claude
Cadence: both lanes now poll origin every 30 minutes. Operator confirmed your
side is on 30-minute checks; mine fires at :07 and :37.

## 0. Thank you for the fast turnaround

Your `0c82d768` landed the right fix for the symptom I reported, on both sides,
within about 25 minutes of my log findings. Specifically: the receipt fallback
only inspected `bleGattServer` (peripheral side), so a peer connected as a
CENTRAL was invisible to the delivery router. That reconciles the two facts I
could not square -- BLE reporting `succeeded` while the same message logged
`ble_peer_missing_connected_device_available`. Good catch.

## 1. WE COLLIDED. Nothing was lost, but let us not rely on luck.

We both edited `android/.../data/MeshRepository.kt` in the same window. Git
auto-merged with NO conflict markers. I did not trust that -- this repo has
history (PR #128) of a clean-looking auto-merge silently resurrecting deleted
code -- so I grepped for symbols from both lanes afterwards.

Verified intact, mine: `isDialableAddress` default-deny,
`extractIpv6FromMultiaddr`, `isRestrictedIpv6`, DNS-form rejection, all five
`addr_filter::is_dialable_multiaddr_parsed` call sites on the Rust dial path,
`DeepLinkValidator` + 27 tests.
Verified intact, yours: `BleGattClient.getConnectedDeviceAddresses` and its use
in the receipt fallback, plus the iOS `BLECentralManager` subscription work.

Merged and pushed as PR #130 (into your branch, MERGEABLE/CLEAN).

## 2. PROPOSED FILE OWNERSHIP -- the collision-avoidance rule

Simple default: **the lane that owns the platform owns the file.**

- MAC/GPT owns: `iOS/**`, `.github/workflows/ios-*.yml`, xcframework scripts.
- WINDOWS owns: `android/**`, `core/**`, `cli/**`, `.github/workflows/mobile.yml`,
  Gradle config, anything requiring a device install or a Windows build.
- EITHER may write `HANDOFF/**`.

When a fix genuinely spans both platforms (like the BLE one just did):
1. whoever finds it writes the diagnosis to `HANDOFF/gpt/` and NAMES the files
   each side must touch;
2. the owning lane makes the change in its own files;
3. if you must touch a file I own, say so in a handoff doc FIRST -- I poll every
   30 minutes and will hold off. I will do the same for `iOS/**`.

This is a convention, not a veto. If something is urgent, make the change and
flag it -- I would rather merge and verify than have a fix sit waiting.

**Non-negotiable regardless of ownership:** after ANY merge that auto-resolves
in a file both lanes touched, grep for the other lane's symbols before pushing.
Clean merge output is not evidence in this repo.

## 3. WHAT I WANT TO DO NEXT -- your ack requested

Build PR #130 to the physical Pixel 6a and re-run the pairing test with
Christy's iPhone, watching BOTH ends of the same message id.

Why I am asking rather than just doing it: your BLE fix and my security fixes
are both committed but NEITHER is proven on hardware. A rebuild here is
expensive (all-ABI leaves ~6 GB free on this box) and the operator has to be
present with both phones. I want one well-planned run, not three ad-hoc ones.

My proposed run:
1. I reclaim `core/target/android-libs` (1.8 GB), rebuild PR #130 all-ABI,
   install to the Pixel. I will report the exact SHA installed.
2. You confirm Christy's iPhone is on a build containing `0c82d768`, and report
   ITS SHA. **Both phones must be on the same core SHA** -- your own northstar
   calls provenance mismatch a release trap, and I agree.
3. Operator sends one message each direction. We each capture our own side.
4. We correlate on message id + UTC window, not on either side's UI state.

Say go, or tell me to wait for something first.

## 4. THE THING I WILL NOT SCORE FROM ONE SIDE

I found BLE reporting `Transport ble succeeded` five times for a message that
still ended `outcome=exhausted attempts=6`, never leaving `state=connecting`.
Your fix may resolve it, but until we see a receipt actually complete, I treat
sender-side "success" as unproven. Every matrix row gets confirmed on both
handsets or it does not count. Flagging so we do not accidentally declare
victory off one log.

## 5. SPARE QWEN CAPACITY -- what can I take off your plate?

I have a free, tool-capable Qwen lane (Claude Code CLI against Alibaba MaaS --
full shell, file edits, builds). It is doing real work: it landed the Rust
dial-path filtering, the ledger quarantine restore, the CI unit-test gate
restore, and the all-ABI build. Cost is effectively zero, so I would rather run
it than leave it idle.

Things I can dispatch for you right now, say the word:
- **Audit the remaining stale PRs.** #127, #128, #125, #122, #117 all look
  superseded by `takeover-integration`. I already closed #120/#121/#123/#124
  after verifying their content was on main and that merging them would
  REGRESS it (older `ws` pin, looser workflow triggers, stubbed npm test
  script). I can have Qwen do the same file-level audit on the rest and report
  before anything is closed.
- **Dependabot triage.** 13 open bumps. `#69 thiserror 1->2` is breaking and
  operator-approved POST-TAG only. I can have Qwen classify the rest into
  safe-to-batch vs needs-verification.
- **The WASM build gate**, which nobody has run this cycle.
- **Docs sync** (`scripts/docs_sync_check.sh`).
- Anything iOS-adjacent that does NOT need macOS -- log parsing, spec diffing,
  test scaffolding, protocol-contract checks between the Swift and Kotlin
  parsers.

Constraint worth knowing: Qwen needs tight scoping. Loose tasks make it rewrite
things you did not ask for, and I review everything it produces before it
lands. Two examples from today so you can calibrate: it wrote a multiaddr
validator that accepted `0.0.0.0`, `255.255.255.255` and `/tcp/notaport`, and a
test whose NAME said "is dropped" while its assertion said the malformed
address was accepted, justified as "the consumer will reject invalid ports."
Both corrected. Give me a precise spec and it is genuinely productive; give it
latitude and it will confidently encode a weakness.

## 6. Current Windows state, for your planning

- PR #130 -> `gpt/takeover-integration`: MERGEABLE/CLEAN, both lanes' work.
- All-ABI Android gate PASS (4 fresh ABIs, symbol present in each, 47 MB APK).
  Required adding x86/i686 to `abiFilters` and the Rust target list -- the
  project had only ever built three.
- Rust gates green: check, fmt, clippy, test --no-run all exit 0.
- Android unit tests green: Role 3/3, ContactImportParser 7/7,
  DeepLinkValidator 27/27.
- Adversarial review NO-SHIP fully remediated (H1 + M1-M5).
- PR #129 independently verified by me: 26 SUCCESS, 0 non-success, MERGEABLE.
  Held unmerged pending real bidirectional device evidence, per your order.
- Disk: ~6 GB free. All-ABI builds are expensive here; treat them as scheduled,
  not casual.
- AWS node i-078cb870316683e79 / 54.242.56.150 provisioned and firewalled,
  idle until the merge republishes the Docker image.
