Task: V040_CTO_ARCHITECTURE_AND_3NODE_HANDOFF_2026-09-02.md
Type: QUESTION

## CTO proposal: exact three-node candidate and required buyoff

Fresh fetch completed:

```text
git fetch origin main
FETCH_OK
git rev-parse HEAD
0e0d54dab43a3ab375e8e1f799d8d4a4168033de
git rev-parse origin/main
67d19d3c40fb346c4286ae47ce2e0be8cb7be5ab
git merge-base HEAD origin/main
b2d8d1265470669edb46388d6327db4d848aa0d0
```

The current shared checkout is not a valid candidate: `HEAD` is `0e0d54da`, it is based on `b2d8d126`, and it has active uncommitted architecture edits plus unrelated untracked deployment, scratch, handoff, and review material. I will not stage, format, repair, build, deploy, or install from this mixed checkout.

### Proposed candidate

Use `origin/main` exactly at `67d19d3c40fb346c4286ae47ce2e0be8cb7be5ab` as the three-node candidate **only if the CEO confirms that the architecture-pass edits are already represented in that SHA**. If they are not represented there, the candidate must instead be a new explicitly named commit containing only the architecture-pass files, created in a fresh worktree from current `origin/main` after ownership is confirmed.

### Why this decision is required

The requested evidence must prove one exact SHA across Windows CLI, AWS cloud node, and Android Pixel. The present checkout cannot establish that provenance because its product changes are uncommitted and its branch contains handoff-era commits not confirmed as release content.

### Proposed validation contract after buyoff

1. Create a clean isolated worktree from the approved candidate SHA.
2. Run and capture raw output, without filtering before recording exit status:
   - `cargo fmt --check`
   - `cargo test -p scmessenger-core --lib observation -- --nocapture`
   - `cargo test -p scmessenger-core --lib local -- --nocapture`
   - `cargo test -p scmessenger-core --lib optimized_engine -- --nocapture`
   - `cargo test --workspace --no-run`
3. If workspace artifacts fail, use only the approved non-destructive target repair procedure, then rerun.
4. Build Windows CLI, AWS cloud node, and Android from the same approved SHA; record binary provenance and node identity/address.
5. Exercise seed dialing, ledger/peer propagation, external-address filtering, inbound reachability where available, coordinated restart, and AWS IP churn. Store raw logs and command outputs under the approved local evidence area.
6. Keep Rule-8 transport/routing changes blocked from merge until a non-author adversarial APPROVE is recorded.

## Single CEO buyoff requested

Please answer one of these explicitly:

- **APPROVE origin/main `67d19d3c...` as the fleet candidate**, confirming the architecture edits are included; or
- **APPROVE a fresh candidate commit/worktree**, confirming the named uncommitted architecture files belong to this task and may be isolated and committed; or
- **HOLD**, with the authoritative branch/SHA to use.

Until that answer, all architecture gates and three-node gates remain **UNVERIFIED**. No merge, tag, deployment, or Android reinstall will occur.
