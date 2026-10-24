# Evidence snapshot and source binding

Status: Proposed planning evidence; no backup execution
Last updated: 2026-10-24 UTC

```json
{
  "captured_at": "2026-10-24T08:51:11.425252+00:00",
  "github_login": "Treystu",
  "main": "941e669da7b1fe9c0c5350dcd010e0ee5c6ecd58",
  "live_ref_counts": {
    "origin": 1292,
    "backup": 19,
    "treystu": 1292,
    "pixiegirlchristy": 99
  },
  "unproven_count": 29,
  "unproven_dispositions": {
    "PROVE_EXISTING_LIVE_REF": 24,
    "PRESERVE_NEW_REF_AFTER_APPROVAL": 5
  },
  "missing_count": 1000,
  "missing_states": {
    "SAME_LIVE_REF_LOCAL_OBJECT_MISSING": 998,
    "LIVE_REF_ADVANCED": 2
  },
  "dirty_count": 7,
  "changed_dirty_ids": [],
  "new_comments_count": 0,
  "no_content_reads": true
}
```

All source SHA256 values below bind local inputs used to derive this plan. Original complete inventory is published in 67 linked comments. A refreshed live-ref list proves only that instant; no perpetual retention is inferred.

| Input | SHA256 | Bytes |
|---|---|---|
| `backup-plan/backup-live-refs.txt` | `80b2c567c6ac2db862e48e9ff6e72129ca33064f6c498e7dceb6dbd898700e54` | 1558 |
| `backup-plan/dirty-seven.json` | `32f7ce1d6121ca1a538d9876432611fce023db688ca6192263ff1a1ccaa7322c` | 6867 |
| `backup-plan/input-summary.json` | `49a784598fd2a388d9519de6e6ab2a7a8bc9792ad0d1ef68ae176a810c2be75f` | 615 |
| `backup-plan/missing-tips-1000.json` | `bb19703572fbc5f3caa69c4d5bece34d4fb406e80968c8d0b8c8b8bfad15a8b8` | 327674 |
| `backup-plan/new-claims.json` | `37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` | 3 |
| `backup-plan/origin-live-refs.txt` | `641b17aa5975c6b3bf3ab59bad435a0b8405e38f5590eeec28afaffffa91f1ac` | 104251 |
| `backup-plan/pixiegirlchristy-live-refs.txt` | `bba99371968e6604a00fd27d013d80e51e5f6862edf67b2881142bfb11a53ad8` | 8153 |
| `backup-plan/treystu-live-refs.txt` | `641b17aa5975c6b3bf3ab59bad435a0b8405e38f5590eeec28afaffffa91f1ac` | 104251 |
| `backup-plan/unproven-29.json` | `a8d5590e922dc81f758d5053e046ede70c67cd56682c0fca8a51b8ba9d99ef87` | 14259 |
| `backup-plan-objects/object-frontiers.json` | `e3b51466ee8df72269db957f55c426b7169e240dc4441029ed3980d07d1279c9` | 6452932 |
| `phase1-inventory.md` | `3f90bd7b1671809c2a97b22a412cf87a9f8dd6afa9e6d67a56f00b0cf252c663` | 3433354 |
| `phase2-backup-batch-a-20261023-jev.json` | `981efd38bb04ef07ff3340fc9b6ad12f12e3700c46bacf781e61ad854821f38e` | 7455 |
| `phase2-backup-batch-a.json` | `72ed0475563c07e87222ac29538a07db933cc6564a93192b5557acbe16ca6813` | 60606 |
| `phase2-reconciliation.json` | `b21219fc907e81820cc6474ca0fe65ad42e1585867127a74dde5edd029c27b9e` | 127171 |

Commands: `gh api user --jq .login`; `git ls-remote <each of four known remotes>`; `git --no-optional-locks status --porcelain=v1 --untracked-files=all` and `git rev-parse HEAD` in the seven inventoried dirty trees; paginated new #602 comments since prior claims capture; read-only graph commands in object-proof. No fsck re-inventory or filesystem sweep was performed.
