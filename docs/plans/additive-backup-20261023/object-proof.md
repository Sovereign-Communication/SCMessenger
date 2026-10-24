# Object graph methods and proof limits

Status: Proposed planning evidence; no backup execution
Last updated: 2026-10-24 UTC

rev-list --topo-order --parents over inventory commits; transitive candidate-descendant propagation; maximal candidates have no candidate descendant. Independent rev-list from frontiers verifies full commit-union coverage. rev-list --objects --no-object-names from frontiers and locally available captured live tips computes exact object-ID reachability.

- Live remote lists are captured evidence, not current read-back; revalidate exact refs immediately before execution and deletion.
- Missing local live tips make negative coverage incomplete; positive available-tip ancestry remains valid for that captured ref.
- Object ID reachability proves object inclusion only; no supersession or feature equivalence is inferred from subjects, patch equivalence or merges.
- No object contents, file bytes or secret values were inspected or emitted. Secret screening is mandatory before any new shared remote preservation.
- Existing remote coverage is not permission to copy private content to a public repo.
- No dirty/untracked/ignored filesystem state is captured by this proof.
- A PROVE disposition still needs fresh remote SHA read-back, retrieval/restore validation and Overseer acknowledgement; no deletion is authorized.

## primary

```json
{
  "clone": "primary",
  "inventory_source": "tmp\\wip-closer-20261023\\final-shared\\unreachable-final.json",
  "inventory_sha256": "403bea0e85638b211b29355909bbeb08491487d9bf8e38ce89d78eea1f32455a",
  "inventory_object_counts": {
    "blob": 9777,
    "commit": 97,
    "tree": 9910
  },
  "commit_count": 97,
  "transitive_maximal_frontier_count": 79,
  "frontiers": [
    "085cc1245c69447391102f21606c73ebe7487a40",
    "0ca9da537e2a290c46f48060d2d804ece1da08cb",
    "107aa38ea7c470b8d1e5a27b4398e7315d918ec4",
    "12c7a6374597eb3e665d97ac87f2a128c65ad87a",
    "1423e1451100b4d88075d485440ec9adf1d6e76b",
    "1513e8bd9eea5f109b9150497489c94243432312",
    "1a208f726842129cd3ac9a13a639e9af2698ec4e",
    "1c6acd02b1dbdd6400351d71bcbd7ae96e91f4be",
    "1cfa3e4623f5fd6541ee30214e7b889b99f1613e",
    "1d6554048dfb7d4c1a0d449b5a313eb47d830945",
    "212228d33a691da45d57e0f3433f6a608421c171",
    "220297013832373a3e64b0aa6ca0e9dff8428d78",
    "247c7abb5e9c25b040a02a42cb648013c914835d",
    "27cdd538b26dc6808db8faba386c0c57c59971e9",
    "2c65f99461dce267dc87fcbc74b1543799648419",
    "2cce65a030f59eae5694d9eccb753f4915497f3d",
    "2d1a79e20916f55013cd054154ffe6262d4088ed",
    "2ec2397a61cd83691a4939d83931fe6440053482",
    "32e6c69c829386cd8e70892dc0a4c9c20fb9fea8",
    "35c49318b643a2a51d692144054546c143f727e0",
    "36783aa2741c928db05e9abb08d155f575917231",
    "3b2c02abee048e9a16b60e753cc14178b4ee3ebe",
    "3c20c6933e20a9ad610570f804968e2343685dfa",
    "3d4da02437051d5c6aac1d090c0699d26ea3039f",
    "3f66eb66b8053e65dd7da2b0889d6a49ce861e78",
    "41b4ae60b7828a088be87dfc5e607f930d11c2d6",
    "41d0c85b280c1f73d73d8becaff86884006817ee",
    "41fb858050695c1f97a33f035fb4e2c1bb325aa6",
    "44760862c93de83cac98ee3a9db4c3fe91f5f131",
    "4a8d7614b5c084ea2098250e15e2ef6c883aced1",
    "568b17bb9ec2245417d33cc89597bdef543784d2",
    "576f16085e0606d1ca03fa0e733355bc4cc24470",
    "5811dfc1fca7268b1459803c09015992c764b441",
    "5959180af9e921fd3ef41bb962bc9c688151f0ed",
    "67678b9158b0e5f6185b3dcd8e849b82b3a12a3c",
    "69cfba52c58d4591dc98fc1c22a73080f71f8bba",
    "6c2408acb96d1cc015b61b9635833f2256cce3e2",
    "6d144c123fc916447e8638ebcb6920609fdcd48d",
    "6e8b04ab048cb9f9dfb29afecf872b5a69bf1e85",
    "6f5669995182e661e8fe5cf753b9a346dfa7a713",
    "6fe7e9b6525aefff01d67290f8c6f2b9066c2a81",
    "723b748698502096bad64508a828cc9e628df5a3",
    "7baf1ebf7177615f0b282b0b65fb4ee4847f0963",
    "7f888bbd12762e9f508b793ae49392a08faa01b6",
    "80603842e7f0968ccd5eaee52cb994a03eff8e75",
    "8a3a6d6e453db46ef2ecc4ebba0d82036a3d1752",
    "8e8790b74ca1f547c5df7154f25a3568d151ed90",
    "916dab120406ff2e52debfe8a63e7e86f35f3286",
    "9b3980b49f27fab75571dfd597958111ab410e3e",
    "9ed7cd401e299d287e7f24a7e664e60d4ba6face",
    "a023e6cb954704cbf27713a102d5111825e7fcd7",
    "a038e29fd0199a85d9807fd12478f41386f28817",
    "a1b9631376c606a02d221fde285a6b77d6d7600a",
    "a787d8f9b52cd987c9bd73bb3619bd9b74b2112a",
    "ad7ca774e30a115c147b393398fe503bd82f3cb3",
    "af6453924fb5d78cda67b2996041570b5029d5d9",
    "b526846e1d65afe7fba1cdadf9cd797ab6956706",
    "b84cd5f0017bfdc1fc88dfcdc54bd54f4976d86b",
    "be8d980d6f803551e4eccffc5d8d4ee9bf226501",
    "bf01319af6e6dfbba8c20ed93076af266498653e",
    "c1e4543842830a64f8c58b1d0d479993220cac4f",
    "c5d6a4edced7370459e1c9748b53c6fe072cceb2",
    "c9c50f6265c08073f232c10b2a517ed95b0ef886",
    "ca992f1b4820fa3a7d267abf040dbedc4c136140",
    "da40b92d1a4da07de7e40e6e327ca0ddebe3ee0c",
    "dbed5380643fb589bc356b31fe582470abdf10ae",
    "e02625e13b088d68a9e72adc3e335a43a17d2cac",
    "e2820fc60c1c2404afffa87f0f602005073a4cb3",
    "e4aa34e2dca4850b6eac182a250b532392383371",
    "ec670a530c9b305f64ea599de4642aef7f1edd0e",
    "f164a2c7f433cde165c1487ed5618a01f7e88c0e",
    "f358a6e2aeacb97e8aaa499c8f4172ac96f744f4",
    "f39d6d20ac4ccd9ce8b9a2ee61931cabf523765f",
    "f64f988744d2fe9ce4b259af19d89769f8a3a468",
    "f7059821911dfd705136daac200d3efb8e5bb5d5",
    "fbb9757d6321fd5a540596f912402e75f706f7c4",
    "fdf2a45e9ded1535a01792eef2401059007c14ef",
    "fede49ab7aa476b852452ccaf510c1ae36ad7a21",
    "ffd983d141be72ae3483bd2e0008b7e15dc30f9f"
  ],
  "frontier_union_covers_all_inventory_commits": true,
  "frontier_closure_total_object_count": 43363,
  "classification_counts": {
    "PRESERVE_ORPHAN_OBJECT_OWNER_SECRET_SCREEN": 19416,
    "PRESERVE_FRONTIER_CLOSURE": 367,
    "PROVE_EXISTING_LIVE_REF": 1
  },
  "frontier_covered_inventory_counts": {
    "blob": 72,
    "commit": 97,
    "tree": 199
  },
  "live_covered_inventory_counts": {
    "commit": 1
  },
  "orphan_counts": {
    "blob": 9705,
    "tree": 9711
  }
}
```

## second

```json
{
  "clone": "second",
  "inventory_source": "tmp\\wip-closer-20261023\\final-oc\\unreachable-final.json",
  "inventory_sha256": "7f5a1edad7806cc7377b09f0e686ea4b5b415933e291886f52a8007b02c17c64",
  "inventory_object_counts": {
    "blob": 5,
    "commit": 1,
    "tree": 8
  },
  "commit_count": 1,
  "transitive_maximal_frontier_count": 1,
  "frontiers": [
    "bd09c49ee208af023861d0874e913104f7fc2e69"
  ],
  "frontier_union_covers_all_inventory_commits": true,
  "frontier_closure_total_object_count": 3785,
  "classification_counts": {
    "PROVE_EXISTING_LIVE_REF": 14
  },
  "frontier_covered_inventory_counts": {
    "blob": 5,
    "commit": 1,
    "tree": 8
  },
  "live_covered_inventory_counts": {
    "blob": 5,
    "commit": 1,
    "tree": 8
  },
  "orphan_counts": {}
}
```

## Executed read-only graph commands

```json
[
  {
    "command": [
      "git",
      "-C",
      "C:\\Users\\SCM\\Documents\\GitHub\\SCMessenger",
      "rev-list",
      "--topo-order",
      "--parents",
      "--stdin"
    ],
    "stdin_line_count": 97,
    "exit": 0,
    "stdout_sha256": "1529f7d13de42319f693ac316219eb150d884b7375345c742e1a5e1391a135bd",
    "stdout_lines": 2726,
    "stderr": ""
  },
  {
    "command": [
      "git",
      "-C",
      "C:\\Users\\SCM\\Documents\\GitHub\\SCMessenger",
      "rev-list",
      "--stdin"
    ],
    "stdin_line_count": 79,
    "exit": 0,
    "stdout_sha256": "251e36807f2495f1d1e027ac773d99adb94376d10ca1999beea50feb5b339e1c",
    "stdout_lines": 2726,
    "stderr": ""
  },
  {
    "command": [
      "git",
      "-C",
      "C:\\Users\\SCM\\Documents\\GitHub\\SCMessenger",
      "rev-list",
      "--objects",
      "--no-object-names",
      "--stdin"
    ],
    "stdin_line_count": 79,
    "exit": 0,
    "stdout_sha256": "5ed0a2fdb3fd044896c8a8d01b90611d04abe3c6af9950a51cb32e454da82740",
    "stdout_lines": 43363,
    "stderr": ""
  },
  {
    "command": [
      "git",
      "-C",
      "C:\\Users\\SCM\\Documents\\GitHub\\SCMessenger",
      "cat-file",
      "--batch-check=%(objectname) %(objecttype)"
    ],
    "stdin_line_count": 981,
    "exit": 0,
    "stdout_sha256": "47e4ffa996e3240ed8c81684949dfb5ac0b6ab7531f5fbb2db452759310df917",
    "stdout_lines": 981,
    "stderr": ""
  },
  {
    "command": [
      "git",
      "-C",
      "C:\\Users\\SCM\\Documents\\GitHub\\SCMessenger",
      "rev-list",
      "--objects",
      "--no-object-names",
      "--stdin"
    ],
    "stdin_line_count": 914,
    "exit": 0,
    "stdout_sha256": "5360e99f8572798b7d8cdfa8a6db39f697abaeabaf468eb04af8a76655968588",
    "stdout_lines": 53231,
    "stderr": ""
  },
  {
    "command": [
      "git",
      "-C",
      "C:\\Users\\SCM\\Documents\\GitHub\\SCMessenger",
      "rev-list",
      "--objects",
      "--no-object-names",
      "--stdin"
    ],
    "stdin_line_count": 17,
    "exit": 0,
    "stdout_sha256": "e78ea5fd3ef21ea8d5890ab0d55265a202cb1ab42081c88ffd0632960d4d7983",
    "stdout_lines": 43059,
    "stderr": ""
  },
  {
    "command": [
      "git",
      "-C",
      "C:\\Users\\SCM\\Documents\\GitHub\\SCMessenger",
      "rev-list",
      "--objects",
      "--no-object-names",
      "--stdin"
    ],
    "stdin_line_count": 914,
    "exit": 0,
    "stdout_sha256": "5360e99f8572798b7d8cdfa8a6db39f697abaeabaf468eb04af8a76655968588",
    "stdout_lines": 53231,
    "stderr": ""
  },
  {
    "command": [
      "git",
      "-C",
      "C:\\Users\\SCM\\Documents\\GitHub\\SCMessenger",
      "rev-list",
      "--objects",
      "--no-object-names",
      "--stdin"
    ],
    "stdin_line_count": 97,
    "exit": 0,
    "stdout_sha256": "499ff1b4682fb0c5df3f3bec498597d380d6340d70a89f2a84f5d21c1f726a2a",
    "stdout_lines": 35263,
    "stderr": ""
  },
  {
    "command": [
      "git",
      "-C",
      "C:\\Users\\SCM\\Documents\\GitHub\\SCMessenger",
      "rev-list",
      "--objects",
      "--no-object-names",
      "--stdin"
    ],
    "stdin_line_count": 1,
    "exit": 0,
    "stdout_sha256": "fc38db79362fd72b9fb648c5ed6c615d31dea53c877bdbb3d553d0aea2b67f7a",
    "stdout_lines": 42302,
    "stderr": ""
  },
  {
    "command": [
      "git",
      "-C",
      "C:\\Users\\SCM\\Documents\\GitHub\\OC\\runtime\\worktrees\\scmessenger-main",
      "rev-list",
      "--topo-order",
      "--parents",
      "--stdin"
    ],
    "stdin_line_count": 1,
    "exit": 0,
    "stdout_sha256": "79897eb518d5dac79fd39f44f96bf2e64a4caef02e1f4b7f02d86ffacdbb0364",
    "stdout_lines": 2,
    "stderr": ""
  },
  {
    "command": [
      "git",
      "-C",
      "C:\\Users\\SCM\\Documents\\GitHub\\OC\\runtime\\worktrees\\scmessenger-main",
      "rev-list",
      "--stdin"
    ],
    "stdin_line_count": 1,
    "exit": 0,
    "stdout_sha256": "d2949898747c1b6dd69efcaeed9ff3ae73bf462b9810bf0a217d3d126cf80c56",
    "stdout_lines": 2,
    "stderr": ""
  },
  {
    "command": [
      "git",
      "-C",
      "C:\\Users\\SCM\\Documents\\GitHub\\OC\\runtime\\worktrees\\scmessenger-main",
      "rev-list",
      "--objects",
      "--no-object-names",
      "--stdin"
    ],
    "stdin_line_count": 1,
    "exit": 0,
    "stdout_sha256": "c5fb032e0a3dfefa8c54176d28c189d3fe7de9df8b4d48d73dc614955e00a859",
    "stdout_lines": 3785,
    "stderr": ""
  },
  {
    "command": [
      "git",
      "-C",
      "C:\\Users\\SCM\\Documents\\GitHub\\OC\\runtime\\worktrees\\scmessenger-main",
      "cat-file",
      "--batch-check=%(objectname) %(objecttype)"
    ],
    "stdin_line_count": 951,
    "exit": 0,
    "stdout_sha256": "d48ed57e57910975406b4cf4e8f684297988eee8297214c34fbe2e940efad73f",
    "stdout_lines": 951,
    "stderr": ""
  },
  {
    "command": [
      "git",
      "-C",
      "C:\\Users\\SCM\\Documents\\GitHub\\OC\\runtime\\worktrees\\scmessenger-main",
      "rev-list",
      "--objects",
      "--no-object-names",
      "--stdin"
    ],
    "stdin_line_count": 281,
    "exit": 0,
    "stdout_sha256": "56c20d4142b52f92f79def7c1bd2e64f41dca35e9291a7085f6333cda93f2385",
    "stdout_lines": 42320,
    "stderr": ""
  },
  {
    "command": [
      "git",
      "-C",
      "C:\\Users\\SCM\\Documents\\GitHub\\OC\\runtime\\worktrees\\scmessenger-main",
      "rev-list",
      "--objects",
      "--no-object-names",
      "--stdin"
    ],
    "stdin_line_count": 1,
    "exit": 0,
    "stdout_sha256": "f1df1894100b33cc17fdd5df137a0a3aa50f03e5b7b49a4fbf260d5f507f0caf",
    "stdout_lines": 42301,
    "stderr": ""
  }
]
```
