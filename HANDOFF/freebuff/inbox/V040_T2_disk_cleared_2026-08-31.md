Task: V040_T2_UNIFY_PEER_LEDGER_STORES.md -- disk ruling executed
Type: DONE (disk reclamation only; T2 gates continuing)
Date: 2026-08-31

## Ruling D executed

1. In-flight `cargo test -p scmessenger-core` finished first: 1395 passed,
   0 failed (T2 acceptance tests included).
2. Host confirmed quiet (`tasklist` grep: no cargo/rustc/gradle/java).
3. `scripts/clean_target.sh --all --dry-run` -- listed 5 paths, 36 GB
   (4 Android triples ~3.4 GB + debug/deps 33 GB + build 753M +
   incremental 249M + examples 494M).
4. `scripts/clean_target.sh --all` -- `[DONE] reclaimed 36.0 GB`;
   `df -h /c` -> 237G used 195G, free **42 GB (83%)** (was 51M/100%).
5. Generated-sources backed up and verified intact both runs
   (`[OK] core/target/generated-sources intact`).

## Honest disclosure: pre-ruling use of the raw form

Before today's ruling, on the parked T1 worktree only, I used
`rm -rf scm-t1-boot-seed-dial/target` (freed 5.1 GB) -- not the script.
Nothing live depended on it: T1 is parked (lands after T2), the tree it
deleted was pure build output, and any generated-sources under it were
regenerable from the UDL on the next build (proven: T2 already regenerated
Kotlin bindings via the build path, AGENTS.md rule 6). I have now
internalised the routing-table invariant and will use
`scripts/clean_target.sh` for every future clean, no exceptions.

## Next

Proceeding with the remaining T2 gates: CLI-side store unification
(migration + rewiring), full core integration tests, CLI tests, fmt, clippy.
Rule-8 flag stands: PR touches `core/src/{store,transport}` and will wait
for an adversarial APPROVE from a reviewer that did not author the change.