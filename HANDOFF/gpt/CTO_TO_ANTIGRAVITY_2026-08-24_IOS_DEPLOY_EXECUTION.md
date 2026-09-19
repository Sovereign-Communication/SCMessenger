# CTO TO ANTIGRAVITY: iOS DEPLOY EXECUTION -- 2026-08-24

Status: ACTIVE -- execute on the MacBook NOW.
Mandate: INSTALL-ONLY per AW-BILAT-0003. No code edits, no project-file
changes, no signing/bundle-ID/entitlement modifications, no merges, no tag
operations. If any step below cannot be completed verbatim, STOP and report.

# THE FROZEN ANCHOR

    Tag:   v0.4.0-rc.1
    SHA:   134e06d225e29a77a67206d12128a370ebeb2d73
    Verify: git rev-parse v0.4.0-rc.1^{commit}   # must print the SHA above;
            # if it differs, STOP -- a newer anchor was cut; ask for the new SHA

NOTE FOR THE OPERATOR: the tag was moved ONCE (with your authorization) from
4077bc39 to 134e06d2 for a release-pipeline fix. The delta is docs + one
shell script -- ZERO application-code delta, so the app binary is identical.
If the MacBook already checked out the tag before the move, `git fetch --tags
--force` and re-checkout; if it already BUILT from 4077bc39, that build is
fine for Christy's trip -- rebuild from the new SHA only if there is slack
time before she leaves.

Every node (Christy's iPhone, N1-N4) installs from this exact SHA. Do not
build from main, from a branch, or from a stale fetch.

# STEPS (MacBook)

    cd <SCMessenger clone>
    git fetch origin --tags --force
    git rev-parse v0.4.0-rc.1^{commit}     # confirm anchor above
    git checkout v0.4.0-rc.1               # detached HEAD is correct

Then Xcode:
    - open the iOS project from the iOS/ directory (use whatever project file
      exists there -- do NOT create/rename anything)
    - destination: Christy's device
    - signing: existing team/profiles on this machine only
    - Product > Run  (builds, installs, launches)

CLI alternative for the build step:
    xcodebuild -project iOS/<existing>.xcodeproj -scheme <existing scheme> \
      -destination 'platform=iOS,name=<device name>' build
    ...then install via Window > Devices and Simulators.

# EVIDENCE (required, ~3 minutes, paste into trip log / report back)

1. Output of `git rev-parse v0.4.0-rc.1^{commit}`
2. The xcodebuild "BUILD SUCCEEDED" line(s)
3. App About/Settings showing version 0.4.0 (build 9)
4. One screenshot of the app launched on her device

Full logs travel over the mesh once peers exist; redacted summaries only in
PRs (log-exchange protocol).

# WHAT CHRISTY CAN TEST SOLO

Identity creation, backup phrase flow, UI, nearby/mesh scanning.
MESSAGING NEEDS A PEER -- see peer status below; coordinate with the operator
before she leaves if peers are not up yet.

# PROHIBITIONS (all standing directives apply)

- No pushes, no force-anything, no tag moves (CTO owns the tag).
- No dependency/toolchain upgrades "while you are in there".
- No editing generated UniFFI sources.
- If provisioning fails: STOP, report -- do not improvise around Apple gating.

# PEER STATUS (for the operator, not blockers to install)

- N4 AWS relay rebuild at the tag SHA: proven path, key ~/.ssh/scm-node-key.pem,
  ec2-user@54.226.67.101, container scm-node, identity persists at
  /opt/scm-relay-data.
- N1/N2 Android: signed APK/AAB arrive as DRAFT release assets once the
  release pipeline completes on the final anchor (draft ruling intact).
- N3 Windows CLI: build locally from the tag checkout.
