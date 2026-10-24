# Final publication manifest

Status: Frozen planning publication; execution remains gated
Last updated: 2026-10-24 UTC

The approved 20 files remain unchanged. This manifest additionally binds the approved-manifest, independent review, Jev decision and rule-6 investigation receipts. It omits itself to avoid a self-hash cycle; its own digest is recorded in the #602 publication/read-back receipt and the Git commit. Every listed file will be read back from GitHub and compared.

```json
{
  "scope": "Only SCMessenger planning documents and bound review/gate/investigation receipts",
  "approved_plan_manifest_sha256": "2f55ee9ae82a70062d2a2f93b11549585e232e8cd5f15fa5bc506e6992d0a659",
  "actual_jev_verdict": "investigate",
  "rule6_resolution": "PROCEED_WITH_DOCUMENTED_NOTE_UNDER_JEV_V1_RULE_6",
  "base_commit": "941e669da7b1fe9c0c5350dcd010e0ee5c6ecd58",
  "files": [
    {
      "path": "docs/plans/additive-backup-20261023/approved-manifest.md",
      "sha256": "ea2757bf93f9d1d3ac5c097db7fca8f8bc71d0088dfe586ecfade5bfa1b74594",
      "bytes": 4185
    },
    {
      "path": "docs/plans/additive-backup-20261023/ci-evidence.md",
      "sha256": "7abd144576556a8575120efe5b4119a9c0bd354c3ec33b660d816fc04de906f5",
      "bytes": 10392
    },
    {
      "path": "docs/plans/additive-backup-20261023/commits.md",
      "sha256": "a0413fe3c3a6dd38c348e92bad4f2612cab0be63c3dd0581734caae0afcfc741",
      "bytes": 15601
    },
    {
      "path": "docs/plans/additive-backup-20261023/dirty-worktrees.md",
      "sha256": "f958009da8f9a2623a94cb480e6c3a2cbf8b8bdb407b937c8573be7e9540c9e2",
      "bytes": 4400
    },
    {
      "path": "docs/plans/additive-backup-20261023/evidence.md",
      "sha256": "432914ea70c964f705f17c83e3782c080e343833630c0782e80227594574fbca",
      "bytes": 3005
    },
    {
      "path": "docs/plans/additive-backup-20261023/gate.md",
      "sha256": "758c7973c5a46b8d0990a4aace921beaa8fdda4aa1e1ca9f3607d5cae48745d2",
      "bytes": 37398
    },
    {
      "path": "docs/plans/additive-backup-20261023/investigation.md",
      "sha256": "934e742e288a11da25e2f3a989c5c4c6750d94802c98be623aae0e6783672432",
      "bytes": 5023
    },
    {
      "path": "docs/plans/additive-backup-20261023/live-backup.md",
      "sha256": "1a3e6de85e91d7929b4e7a74717e0ff992a9f5834980f1579d34508663ab63a2",
      "bytes": 1850
    },
    {
      "path": "docs/plans/additive-backup-20261023/live-origin.md",
      "sha256": "a3bb06276d0f03700565379e17bf8a95b1d73803fa17807087eb4866a965ddb4",
      "bytes": 104543
    },
    {
      "path": "docs/plans/additive-backup-20261023/live-pixiegirlchristy.md",
      "sha256": "dff59098bc440d7523480765e0fc15804f062d96d4b0c11a076fb887f0dc590e",
      "bytes": 8455
    },
    {
      "path": "docs/plans/additive-backup-20261023/live-treystu.md",
      "sha256": "7406c197af1c8ab77eb7d940e9b9fda896498e57fb3870b27cb67b939964d4a6",
      "bytes": 104544
    },
    {
      "path": "docs/plans/additive-backup-20261023/missing-tips.md",
      "sha256": "ad3189b73d39cf16b4264f069d9e30a1bcbe68f8ccb875c4cd119d4216aae97f",
      "bytes": 240355
    },
    {
      "path": "docs/plans/additive-backup-20261023/noncommit-refs.md",
      "sha256": "caf30b573b1a82efd673498f80c2c2f60520fa06254addb4ed359cd4b3fc22e8",
      "bytes": 3190
    },
    {
      "path": "docs/plans/additive-backup-20261023/object-proof.md",
      "sha256": "97ae1216b4e3b2f0b4b08b3d6a79401bba7cbeb885ad878627900350acf3cd57",
      "bytes": 12473
    },
    {
      "path": "docs/plans/additive-backup-20261023/objects-01.md",
      "sha256": "6f616e36029444b1c4304078c73f99a8daa2a7cefa926c45f65b2ef121e7e7f2",
      "bytes": 512033
    },
    {
      "path": "docs/plans/additive-backup-20261023/objects-02.md",
      "sha256": "0be15498f76e9a768c158afef9952b55b17f2f1ac1b4886992dfd096715bc385",
      "bytes": 512000
    },
    {
      "path": "docs/plans/additive-backup-20261023/objects-03.md",
      "sha256": "f973c07340bf7dffc1ac095901ad4cd4eaea67427363db60bf6a3094c52013b6",
      "bytes": 509661
    },
    {
      "path": "docs/plans/additive-backup-20261023/objects-04.md",
      "sha256": "2efd4d8db8efd947909e71916e13b610ccb259f3fa5af3167ed416fa9c812c75",
      "bytes": 511138
    },
    {
      "path": "docs/plans/additive-backup-20261023/objects-05.md",
      "sha256": "c9f1d298eb2aed65f157db702f9813ae00efe617be0a1cda50f165f1381e7a4d",
      "bytes": 485088
    },
    {
      "path": "docs/plans/additive-backup-20261023/objects-index.md",
      "sha256": "47dea8e83371e2733cf14cedde451d729f6a4be9ab437d0d404410f48d297b5f",
      "bytes": 618
    },
    {
      "path": "docs/plans/additive-backup-20261023/README.md",
      "sha256": "d3eb7b123156f6e874444e3ba9b33b7c591f777cf149f9130fde632d09a23164",
      "bytes": 24150
    },
    {
      "path": "docs/plans/additive-backup-20261023/review.md",
      "sha256": "d196ecceb55917f7217ee23fcb08c141d2651b099cde534db3fd5915bb32ee98",
      "bytes": 2925
    },
    {
      "path": "docs/plans/additive-backup-20261023/unproven-refs.md",
      "sha256": "bb9b9c31712f24fc107dbbabf2f7fb7f3e8dd313598aaf220801d36a47441955",
      "bytes": 9657
    },
    {
      "path": "docs/DOCUMENT_STATUS_INDEX.md",
      "sha256": "50212e34b00545964cf3b4bccb3156b03637a5889e6ba927b9056a8b4343c5f8",
      "bytes": 35008
    }
  ]
}
```

## Complete document links

- [docs/plans/additive-backup-20261023/approved-manifest.md](approved-manifest.md): `ea2757bf93f9d1d3ac5c097db7fca8f8bc71d0088dfe586ecfade5bfa1b74594`
- [docs/plans/additive-backup-20261023/ci-evidence.md](ci-evidence.md): `7abd144576556a8575120efe5b4119a9c0bd354c3ec33b660d816fc04de906f5`
- [docs/plans/additive-backup-20261023/commits.md](commits.md): `a0413fe3c3a6dd38c348e92bad4f2612cab0be63c3dd0581734caae0afcfc741`
- [docs/plans/additive-backup-20261023/dirty-worktrees.md](dirty-worktrees.md): `f958009da8f9a2623a94cb480e6c3a2cbf8b8bdb407b937c8573be7e9540c9e2`
- [docs/plans/additive-backup-20261023/evidence.md](evidence.md): `432914ea70c964f705f17c83e3782c080e343833630c0782e80227594574fbca`
- [docs/plans/additive-backup-20261023/gate.md](gate.md): `758c7973c5a46b8d0990a4aace921beaa8fdda4aa1e1ca9f3607d5cae48745d2`
- [docs/plans/additive-backup-20261023/investigation.md](investigation.md): `934e742e288a11da25e2f3a989c5c4c6750d94802c98be623aae0e6783672432`
- [docs/plans/additive-backup-20261023/live-backup.md](live-backup.md): `1a3e6de85e91d7929b4e7a74717e0ff992a9f5834980f1579d34508663ab63a2`
- [docs/plans/additive-backup-20261023/live-origin.md](live-origin.md): `a3bb06276d0f03700565379e17bf8a95b1d73803fa17807087eb4866a965ddb4`
- [docs/plans/additive-backup-20261023/live-pixiegirlchristy.md](live-pixiegirlchristy.md): `dff59098bc440d7523480765e0fc15804f062d96d4b0c11a076fb887f0dc590e`
- [docs/plans/additive-backup-20261023/live-treystu.md](live-treystu.md): `7406c197af1c8ab77eb7d940e9b9fda896498e57fb3870b27cb67b939964d4a6`
- [docs/plans/additive-backup-20261023/missing-tips.md](missing-tips.md): `ad3189b73d39cf16b4264f069d9e30a1bcbe68f8ccb875c4cd119d4216aae97f`
- [docs/plans/additive-backup-20261023/noncommit-refs.md](noncommit-refs.md): `caf30b573b1a82efd673498f80c2c2f60520fa06254addb4ed359cd4b3fc22e8`
- [docs/plans/additive-backup-20261023/object-proof.md](object-proof.md): `97ae1216b4e3b2f0b4b08b3d6a79401bba7cbeb885ad878627900350acf3cd57`
- [docs/plans/additive-backup-20261023/objects-01.md](objects-01.md): `6f616e36029444b1c4304078c73f99a8daa2a7cefa926c45f65b2ef121e7e7f2`
- [docs/plans/additive-backup-20261023/objects-02.md](objects-02.md): `0be15498f76e9a768c158afef9952b55b17f2f1ac1b4886992dfd096715bc385`
- [docs/plans/additive-backup-20261023/objects-03.md](objects-03.md): `f973c07340bf7dffc1ac095901ad4cd4eaea67427363db60bf6a3094c52013b6`
- [docs/plans/additive-backup-20261023/objects-04.md](objects-04.md): `2efd4d8db8efd947909e71916e13b610ccb259f3fa5af3167ed416fa9c812c75`
- [docs/plans/additive-backup-20261023/objects-05.md](objects-05.md): `c9f1d298eb2aed65f157db702f9813ae00efe617be0a1cda50f165f1381e7a4d`
- [docs/plans/additive-backup-20261023/objects-index.md](objects-index.md): `47dea8e83371e2733cf14cedde451d729f6a4be9ab437d0d404410f48d297b5f`
- [docs/plans/additive-backup-20261023/README.md](README.md): `d3eb7b123156f6e874444e3ba9b33b7c591f777cf149f9130fde632d09a23164`
- [docs/plans/additive-backup-20261023/review.md](review.md): `d196ecceb55917f7217ee23fcb08c141d2651b099cde534db3fd5915bb32ee98`
- [docs/plans/additive-backup-20261023/unproven-refs.md](unproven-refs.md): `bb9b9c31712f24fc107dbbabf2f7fb7f3e8dd313598aaf220801d36a47441955`
- [docs/DOCUMENT_STATUS_INDEX.md](../../DOCUMENT_STATUS_INDEX.md): `50212e34b00545964cf3b4bccb3156b03637a5889e6ba927b9056a8b4343c5f8`
