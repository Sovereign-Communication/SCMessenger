# SCMessenger legacy handoff classification manifest

**Date:** 2026-09-24
**Snapshot:** all files present below the owner `HANDOFF/` directory at manifest generation time.
**Change boundary:** documentation only; no legacy file was edited, moved, deleted, or migrated.

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This owner-local manifest records 1605 discovered handoff paths for SCMessenger. It is an inventory, not a migration.

## Classification rule

1. Enumerate every regular file recursively below `HANDOFF/`.
2. Read UTF-8 bytes and inspect both the repository-relative path and document body for a foreign-product alias.
3. Classify `quarantined` when either path or body contains a foreign-product alias; preserve the original bytes and do not use the file as evidence.
4. Otherwise run the owner-local scope gate on the actual file. Classify `owner-valid` only on a pass; classify `blocked-by-metadata` on any gate failure or unreadable bytes.
5. Do not mass-migrate. A future migration requires owner approval of this manifest, a deterministic transformation rule, and a fresh gate run on the resulting bytes.

The exact raw path register is kept in the OC operations index. Foreign-named quarantined paths are represented here by stable IDs so this owner handoff does not import a foreign alias.

## Counts

- `owner-valid`: **3**
- `quarantined`: **133**
- `blocked-by-metadata`: **1469**

## Path ledger

| ID / path | Classification | Reason | SHA-256 |
|---|---|---|---|
| `HANDOFF/.MORPH_LITE_QUICKREF.txt` | `blocked-by-metadata` | `gate-fail` | `175e21df2e93ad64b1921ae606d179161c1385589cc9d82e5e3d650023442aea` |
| `HANDOFF/ACTIVE_LEDGER.md` | `blocked-by-metadata` | `gate-fail` | `88c5e61ff042e21a88474709a3b1e122d4444e6796ff9915cf624ec22d0c498a` |
| `HANDOFF/AGENTS_ANDROID.md` | `blocked-by-metadata` | `gate-fail` | `79f0aa40cfeb83868123f344bc07f38e933a3eea286c91689f97df94e893f8e4` |
| `HANDOFF/AGENT_HANDOFF_GUIDANCE.md` | `blocked-by-metadata` | `gate-fail` | `ecd70599b43e1c1b87c3357b86cffb1ad180f52486d23a5bb033ddf129501748` |
| `HANDOFF/ALPHA_TEST_LUCAS_JOSH_SETUP.md` | `blocked-by-metadata` | `gate-fail` | `6f1d85d9ef4eb24f127e154bd0890fa2560e07bd4e52a9976d18485c00ecae58` |
| `HANDOFF/ALPHA_TEST_SESSION_FINDINGS_2026-07-19.md` | `blocked-by-metadata` | `gate-fail` | `cf3b3d3917f691c18482c06611dc855f65919ca9faf2a6fe5d298fbf85b5f947` |
| `HANDOFF/ANTIGRAVITY_WIRING_IMPLEMENTATION_PROMPT.md` | `blocked-by-metadata` | `gate-fail` | `1f71a9adae0b24ee8147fa5e8a5808556c881e2c01a67b05325bd1da21181ecd` |
| `HANDOFF/API_RESET_EXECUTION_CHARTER_2026-08-28.md` | `blocked-by-metadata` | `gate-fail` | `64d448027f4c9df3a4589ab1890cc6a13d096a032814a417e5af70c31d6fb63e` |
| `HANDOFF/BLUETOOTH_DISCOVERY_PARITY_PLAN.md` | `blocked-by-metadata` | `gate-fail` | `bfe759644d625c34b05e3fcb91692ee362981325606048c24f82c782b12826bb` |
| `HANDOFF/BOB_KICKOFF_PROMPT.md` | `blocked-by-metadata` | `gate-fail` | `66444e9295f9440b5c2fc358bf4e07b2184a123501cbd2162d2d6db7c854eef6` |
| `SCM-Q-64766b89ce1b` (path hash `64766b89ce1bec4b95d2ded785a161abab862a41e9b2e15e5d18ae32572b1a69`) | `quarantined` | `foreign-alias-in-bytes` | `eebf720aa6f681135502981fb9409705c30841d97f942440aa450279ad004bb5` |
| `HANDOFF/BRANCH_SPACE_CLEARING_SOP.md` | `blocked-by-metadata` | `gate-fail` | `963b42803ccffa897fcfda9c9dc7dfabd856eacb2e239d3834c94846cfc5d0fa` |
| `HANDOFF/CELL_RELAY_CUSTODY_FIX_2026-09-10.md` | `blocked-by-metadata` | `gate-fail` | `95f49b7e683af6dbcde9be82f0582ceccfa4d4e714c6ee029c24d64f04e6bbee` |
| `HANDOFF/CEO_RULINGS_2026-08-16.md` | `blocked-by-metadata` | `gate-fail` | `d2fbb74df920aef01f9b479cf2014006ac153a66a3ed148735481539de37eae9` |
| `SCM-Q-e3c067f8faac` (path hash `e3c067f8faacec9162597f50325b6251ab9bbf97c5e42ff249502b7677706101`) | `quarantined` | `foreign-alias-in-bytes` | `ab9cd249a2abc2549dad11ce5b0b4cb5e1087546e34074327bfe083c84ec9ae0` |
| `HANDOFF/CLAUDE_CODE_PROTOCOL.md` | `blocked-by-metadata` | `gate-fail` | `df50ec9e969e0e2449f3ff6b8ae7e5799b242b5c5866823b624269430afdab80` |
| `HANDOFF/CLAUDE_CODE_README.md` | `blocked-by-metadata` | `gate-fail` | `5afea7e12c8ab409058595dd6bb2dd443007a07e4e43ca59d5bd32b978f4bbaa` |
| `HANDOFF/CLI_DISCOVERY_VERIFICATION_REPORT.md` | `blocked-by-metadata` | `gate-fail` | `74a768390819639d65407b02a90546800bff918b7b0a5d5e22d8899274d92bc3` |
| `HANDOFF/CLI_DRIVER_DISCOVERY_QUICKSTART.md` | `blocked-by-metadata` | `gate-fail` | `eaf28b29ff86359e73337c46027cfb85f9578a1da6d84772c40c18498d94b569` |
| `HANDOFF/CLI_PARITY_PLAN.md` | `blocked-by-metadata` | `gate-fail` | `a89e71148bc281c1422a340e6ddad53ec0b934f1ffbfb716a552bc67c7ed5b4f` |
| `HANDOFF/COMMIT_CHECKPOINT_PROMPT.md` | `blocked-by-metadata` | `gate-fail` | `48db7ba79537593909f78e7508d1b0522f1f003dc0d655ed9a48192a585b9b2d` |
| `HANDOFF/COORDINATION_FALLBACK_PROTOCOL_2026-08-10.md` | `blocked-by-metadata` | `gate-fail` | `07181a29f796b3f3aabc6633cdacf81ef69c57db2c9e6bfd13b5b4611358627b` |
| `HANDOFF/CTO_DISPATCH_PLAN_2026-08-16.md` | `blocked-by-metadata` | `gate-fail` | `700bf82f7cbf1e9d1315911ea0ba25f636134375cde926dc572293181ed3f5bb` |
| `HANDOFF/CTO_DISPATCH_PLAN_2026-08-20.md` | `blocked-by-metadata` | `gate-fail` | `8fc31889171b3ecb3329ed8529803f58e0b2f24e57bef765602bba8738046383` |
| `SCM-Q-56d3b2e99fa4` (path hash `56d3b2e99fa475325bb7b24a95596099d28b523d1c618511e78cd038dab9493e`) | `quarantined` | `foreign-alias-in-bytes` | `4dceaea001951c3a1c84620c844c78c78c6b48a089f2ed2ce417862bb6a9959c` |
| `SCM-Q-da07a044ac66` (path hash `da07a044ac663c388cadbbe41c3c2f11106b17031fc2a038ee482ad97471d0ff`) | `quarantined` | `foreign-alias-in-bytes` | `af88309d1dfd98cee1176303bf92484777cae86fd5055fbf0d8e2ded3ad8aeef` |
| `HANDOFF/D4_NODE_REBUILD_RUNBOOK.md` | `blocked-by-metadata` | `gate-fail` | `f5217787525dffd30b4333f13816def422390e6a41b1e2fe2cdfda528e5701bb` |
| `HANDOFF/DEAD_CODE_TRIAGE_RESULTS.md` | `blocked-by-metadata` | `gate-fail` | `5886d7b4288572571b3b12847641b8b420f495806fde029763406e684a74d613` |
| `HANDOFF/DESKTOP_BRIDGE_WIRING_SPEC.md` | `blocked-by-metadata` | `gate-fail` | `fedbfa0fd4cf27c3ac13631c78c2c6232b9a414b922458d6e667a84d826ce165` |
| `HANDOFF/DISCOVERY_ISSUE_DIAGNOSIS.md` | `blocked-by-metadata` | `gate-fail` | `568c0a118cb7e081d1248f4481b7bd8aa9159b9718618c8adaa93093c14271ba` |
| `HANDOFF/DISCOVERY_STATUS_SUMMARY.md` | `blocked-by-metadata` | `gate-fail` | `8d4399a133646516ed8d2f79beb5077e90525ccb96baa64cbd15ea9dd4037daa` |
| `HANDOFF/DISCOVERY_TESTING_COMPLETE.md` | `blocked-by-metadata` | `gate-fail` | `8529a1145df1feb4e4e1ee2e43ad5c6d49d64b4cd5cb0a01222d5e131f43b913` |
| `HANDOFF/ESCALATIONS/ACK_ESC-FC74D953.md` | `blocked-by-metadata` | `gate-fail` | `37bc385d382bd65e374aa9fb3da01e1ad722dc6f26087195b85867ff5e02c0a5` |
| `HANDOFF/ESCALATIONS/ESCALATION_claude-code_2026-06-10T18-59-30Z_review-required.md` | `blocked-by-metadata` | `gate-fail` | `f486b2c4f93d99b87714d409531500721e8dd175b94d548624a843371d9a38b5` |
| `HANDOFF/ESCALATION_PROTOCOL.md` | `blocked-by-metadata` | `gate-fail` | `11b2e7482d2268dd3bde6f87c3cb2fc956df175179aff0bb4956df4407920338` |
| `HANDOFF/FARM_PORT_STRATEGY_ANALYSIS.md` | `blocked-by-metadata` | `gate-fail` | `cb2696b8a64b6388c0c493ad25b58085b3cf6ab9c78f8e5605663b6ef3ff802c` |
| `SCM-Q-54eb5cbc58ab` (path hash `54eb5cbc58ab3e5662f4a28895bdf13d91e3a46691af64e6ba022677cf6f80a7`) | `quarantined` | `foreign-alias-in-bytes` | `997dbe95b7441e77abaf1a93e9ddca863943e4435bdbcddfcede809d90ec73b7` |
| `HANDOFF/FARM_SIM_PHASE_2_3_EXECUTION_SUMMARY.md` | `blocked-by-metadata` | `gate-fail` | `8db7efaa6ce00bf4b13d33dcf40ccd7487d974cda088f7724561aae6f4c789f2` |
| `SCM-Q-482742535a3b` (path hash `482742535a3bcded1a7456a602cf5badb5e952e00148478c1edcb786635d94da`) | `quarantined` | `foreign-alias-in-bytes` | `9e852cbed902c3fc80827fb812a1bb261fb80a5130c5121fc13c2a865f467a04` |
| `HANDOFF/FARM_UNIFICATION_STRATEGY.md` | `blocked-by-metadata` | `gate-fail` | `a62fa09f99a134ed3a26a659f191bcf859f0fc1d17dcfec37f6bf9bc6c4f25a9` |
| `HANDOFF/FINAL_HANDOFF_2026-08-04.md` | `blocked-by-metadata` | `gate-fail` | `2d7860452b098b599bf08927a1cf7020913f3d0dd05a9a7d6752e1a4dbc07640` |
| `HANDOFF/GEMINI_ORCHESTRATOR_QUICKSTART.md` | `blocked-by-metadata` | `gate-fail` | `f3b8f6948664193f3f110b15e0ca55069086c9ae620b948c03cbb3a327d4049b` |
| `HANDOFF/GITHUB_CI_CD_AUDIT_FINDINGS.md` | `blocked-by-metadata` | `gate-fail` | `b49966539b344af8cc5541236bf41e0fabf5b668efde59b1f9044cc2f4fd52a8` |
| `HANDOFF/GPT_PRIMARY_HANDOFF_2026-07-29.md` | `blocked-by-metadata` | `gate-fail` | `b9219ecfa52ad3c8e605b0a06963d0d9ec20a8d9b42b0b545391cf68c3585010` |
| `HANDOFF/IMMEDIATE_NEXT_STEPS.md` | `blocked-by-metadata` | `gate-fail` | `7d08fb05b28439c1afb8b84872c3a049a2cf66701252022fcfda3da4ed2aba57` |
| `HANDOFF/IMPLEMENTATION_NOTES_P1_DIAL_POLICY.md` | `blocked-by-metadata` | `gate-fail` | `5b58a5ebaea1f4f7f31c434ed05377c0c9a2e22bf1c9541e88c139406ea0910f` |
| `HANDOFF/IMPLEMENTATION_NOTES_P3.md` | `blocked-by-metadata` | `gate-fail` | `add4de81949688b68fa4304eaf526be89cf59340fbc8a025360d21c9df095516` |
| `HANDOFF/INFRASTRUCTURE_REDESIGN_2026-07-18.md` | `blocked-by-metadata` | `gate-fail` | `a00482281d31b3a971c4ea42900ddb2b7ba1b5089c6340218230e467efb1d628` |
| `HANDOFF/IN_PROGRESS/A-05_IOS_RECEIPT_UNIFICATION.md` | `blocked-by-metadata` | `gate-fail` | `9c958b253f4f5898f08513e93a404e6d7811c6aee6b1ef1093a47b9756b3a615` |
| `HANDOFF/IN_PROGRESS/ANDROID_RELAY_INBOUND_EVIDENCE_2026-08-10_CELLULAR.md` | `blocked-by-metadata` | `gate-fail` | `91db722df31f82b78783be5075b273d12f0f4580349ed55a5ca52b0cf2b8f258` |
| `HANDOFF/IN_PROGRESS/CORE_BLOCK_GATE_IDENTIFIER_FIX.md` | `blocked-by-metadata` | `gate-fail` | `83483b20735e7c6559420271878370e25ff5d56fcc00c69fb1ce7108e6e59a1d` |
| `HANDOFF/IN_PROGRESS/D1_DESKTOP_BRIDGE_UNIFFI_VERIFICATION.md` | `blocked-by-metadata` | `gate-fail` | `23dbb72735ba91c126ccce6fbab0c6c2477bfe3bfbed8576246cbc8637d4db74` |
| `HANDOFF/IN_PROGRESS/DISPATCH_WAVE_CRYPTO_CODER.md` | `blocked-by-metadata` | `gate-fail` | `2d241c7318f9b872052af05b568abb5c6a597b6b0ad8ee87622383b5a34fa6a9` |
| `HANDOFF/IN_PROGRESS/FARM_SIM_PHASE_2_3_COMPREHENSIVE_TESTING.md` | `blocked-by-metadata` | `gate-fail` | `6be80deb2e648a99f6c32658a5694e7bb1ba9336c0e8d59caf319735c1affa7d` |
| `HANDOFF/IN_PROGRESS/GITHUB_V1_0_0_RUNNER_SETUP.md` | `blocked-by-metadata` | `gate-fail` | `5556ee34bb9385ba80b7d27c17cab2f2616619a495cddb690ad2ffb6421de633` |
| `HANDOFF/IN_PROGRESS/PQC09_HYBRID_ONION_INVESTIGATION.md` | `blocked-by-metadata` | `gate-fail` | `f5b4a04d57ba4e496c03cc06d2328550fe2bb1bd24708d57d735c441186fa368` |
| `HANDOFF/IN_PROGRESS/U2_TOPIC_CONSTANTS_CENTRALIZE.md` | `blocked-by-metadata` | `gate-fail` | `69a4610bed2d7256d0a0fed7de4e025fd954b48f97bd2ed8b94cae207f1d13d3` |
| `HANDOFF/LAUNCH_NEXT.md` | `blocked-by-metadata` | `gate-fail` | `6f2a5b085fa4f8cfc82fa23e7d1c0b0a85b19e69ed6dd5a884c00fa5c72235ea` |
| `SCM-Q-9a5e2503cf79` (path hash `9a5e2503cf79302cc2a5ce445f7ed8583e0a13da729f9275620d8f0866ab2a58`) | `quarantined` | `foreign-alias-in-bytes` | `441cdd178bc9bd64e734e44a0588b1f30e2ff14bd24347fdf9d4f7b1acd21eb7` |
| `SCM-Q-2c820602aa3e` (path hash `2c820602aa3ea26b13fbfac255b435d9dcb6e2d1c82ac38252b3ce00f41d1e02`) | `quarantined` | `foreign-alias-in-bytes` | `97554c698190b157f0be85a1a43a1df9427c1c353f7251aa2cd76356eec7ece6` |
| `HANDOFF/MDNS_FIX_COMPLETE.md` | `blocked-by-metadata` | `gate-fail` | `92da83398b1f475047049fd0dd14799c1cf5e4da57745824b4cd9d69425a1f11` |
| `HANDOFF/MORPH_LITE_HANDOFF.md` | `blocked-by-metadata` | `gate-fail` | `b6a3c050555f3e924f013f30c538bef4ca9daf5549cfe31e00a3b4553a884526` |
| `HANDOFF/NEXT_ORCHESTRATOR_PROMPT.md` | `blocked-by-metadata` | `gate-fail` | `1a43fffb07f523ec405a8e9944bd6b4a678251f5ba6377129e5d236bc3209f2a` |
| `HANDOFF/NEXT_STEPS_COMPLETION_PLAN.md` | `blocked-by-metadata` | `gate-fail` | `30d89f4220a08875dc05456cc9613dc67792e76c60b47e3826e1c79536fb290c` |
| `HANDOFF/ORCHESTRATION_COMPLETION_REPORT.md` | `blocked-by-metadata` | `gate-fail` | `7f2ab0dbbfac81009e45ed47903c8c826735b641ce571bee9ac0d95e161ae00d` |
| `HANDOFF/ORCHESTRATION_CONTROL_PLANE_V2_DRAFT_PR.md` | `blocked-by-metadata` | `gate-fail` | `b4285ec6e54ff99666c47d36e976ff3d4bc8a78dbdd49226b46f0e6bd42e4578` |
| `HANDOFF/ORCHESTRATION_IMPLEMENTATION_GUIDE.md` | `blocked-by-metadata` | `gate-fail` | `326f7f47d463ea66d527d1bdc2df0817b8b5a32874b0ebd7538068222a13545f` |
| `HANDOFF/ORCHESTRATION_LEDGER_P1_P4_P6.md` | `blocked-by-metadata` | `gate-fail` | `abb0f407d31546d3f946f7382008bb8193c028f0185e1b698f036296ac9a3611` |
| `HANDOFF/ORCHESTRATION_SUMMARY.md` | `blocked-by-metadata` | `gate-fail` | `2f7c2a37aef8c1de78cd845acf778301f0cf5f8d770ce8e62fd0419807580fc2` |
| `HANDOFF/ORCHESTRATION_SUMMARY.txt` | `blocked-by-metadata` | `gate-fail` | `ecb9de69569ed04c0c3ac24d6939dc2a94859415bfbfd96d4e8dd5f94cf73eea` |
| `HANDOFF/ORCHESTRATION_TOKEN_REDUCTION_PLAN.md` | `blocked-by-metadata` | `gate-fail` | `acb94aa045d47410e5a503e8ed43c782c06b5fa950c4c2bb1adc421d078b72f1` |
| `HANDOFF/ORCHESTRATION_TOKEN_STRATEGY.md` | `blocked-by-metadata` | `gate-fail` | `11dba0b2406dcd2b4a1ff21cf03ffe08f1e24da5f4c83a408f8a801d5ccf9042` |
| `HANDOFF/ORCHESTRATOR_AUDIT_EXECUTIVE_SUMMARY.md` | `blocked-by-metadata` | `gate-fail` | `298c57f6e48c09407fa8067437ea173063105c443b62fdc92167039ad38b0fd9` |
| `SCM-Q-95d9cd75a24f` (path hash `95d9cd75a24f9ca97dc682114a57b9225a7e9c0e8543a2a6043b2e2d2f2e1d40`) | `quarantined` | `foreign-alias-in-bytes` | `f9b13dd3253907245b87ae69a64b497a5fda22a5c19737666a29d366ad22487f` |
| `HANDOFF/ORCHESTRATOR_LOG.md` | `blocked-by-metadata` | `gate-fail` | `37ee38eae8fda731e32ab53161b5331f94c35b786fcdeb9e6ada9746dae2e388` |
| `HANDOFF/ORCHESTRATOR_SESSION.md` | `blocked-by-metadata` | `gate-fail` | `6f502443488e879685738f78618bf58746687c169e1a3af4a4c98b6c37812a0c` |
| `HANDOFF/ORCHESTRATOR_START_HERE.md` | `blocked-by-metadata` | `gate-fail` | `7be0be3aec0313ec6a696c317ccb0ea0da43804f6e92a1374a0b542aa441ab81` |
| `HANDOFF/ORCHESTRATOR_STATUS.md` | `blocked-by-metadata` | `gate-fail` | `ed6950b0d6f1eb654ac90efadf5066abda788bb0b5f56dd90676077a28a1daa1` |
| `SCM-Q-0c60c9d444e4` (path hash `0c60c9d444e434be46737ffc1f7c642efd832680fa5ea8eed932ef94ea229f04`) | `quarantined` | `foreign-alias-in-bytes` | `65bc32fd7704b694aadb9364b42e53e1f6c6d1244c544ecb651477cf246072b0` |
| `HANDOFF/ORCHESTRATOR_TAKEOVER_2026-08-09.md` | `blocked-by-metadata` | `gate-fail` | `b016e878bbf595873a0bfc0421c4358eae77bf70bc7d14932938f2420f515c3f` |
| `HANDOFF/ORCHESTRATOR_TAKEOVER_2026-08-09_WINDOWS_LANE.md` | `blocked-by-metadata` | `gate-fail` | `bb382f43e3086f37e5afcd4d9cc2a919d142344cbe613e3f4474a3f373f50ee7` |
| `HANDOFF/ORCHESTRATOR_TAKEOVER_2026-08-10_WINDOWS_LANE.md` | `blocked-by-metadata` | `gate-fail` | `40825a8887403d9cc3e9a20c9197e2b9a111c6988f6b91cb5573b54f5811f6f4` |
| `HANDOFF/ORCHESTRATOR_TAKEOVER_2026-08-28.md` | `blocked-by-metadata` | `gate-fail` | `5f96401e98ab1241fb85bb6ae86010320d166306b8cd4cc9322f7a6b0d4ce654` |
| `HANDOFF/ORCHESTRATOR_TOKEN_AUDIT_AND_REDESIGN.md` | `blocked-by-metadata` | `gate-fail` | `e69d8ff97f87468a1feba7fda02fed8743f73cfe0038841799c1535e0fb7621f` |
| `HANDOFF/P1_DIAL_POLICY_CHECKLIST.md` | `blocked-by-metadata` | `gate-fail` | `24d158d5c5feab7996d365b0f8fe41c4cd25d438a053ecb8d736aaa3eeda3002` |
| `HANDOFF/P1_DIAL_POLICY_DESIGN_RATIONALE.md` | `blocked-by-metadata` | `gate-fail` | `dd11b074ae32031582cf7e5918f9923df5bb78c53e204917c6dbb1cc88658e15` |
| `HANDOFF/P1_DIAL_POLICY_DIFFS.md` | `blocked-by-metadata` | `gate-fail` | `ec23105b62ab31df5983ec5c858e20a6e6ac226673e672cd6dd76ac95a10e392` |
| `HANDOFF/PEER_AUDIT_AND_33DA1982_CUTOVER_2026-09-11.md` | `blocked-by-metadata` | `gate-fail` | `cf871c692223bf5a73af71157ced0c435773b8d33acaf77e14775303f77b0548` |
| `HANDOFF/PEER_ID_TRIAD_UNIFICATION_2026-09-10.md` | `blocked-by-metadata` | `gate-fail` | `432c976a6ca5b3cb6876a0d07b2ceef729880c0c57e7f9200a6b0f1f876f971d` |
| `SCM-Q-5b2ac3f13c25` (path hash `5b2ac3f13c25d1f119ab5da7945b2d62990d3c4b5fbab4bb30fa2d14b00c3d7b`) | `quarantined` | `foreign-alias-in-bytes` | `08f534049c41a98e59daa2f1a06b77df663d6308de74ffd7d01468ed1626c6ec` |
| `HANDOFF/POST_GREEN_PLAN.md` | `blocked-by-metadata` | `gate-fail` | `2912b4c187f0f918e7f8a86fe9eaac2c9879e850f6903aa17fbe8116c4150f73` |
| `HANDOFF/PR139_FIVE_NODE_GATE_STATUS_2026-08-13.md` | `blocked-by-metadata` | `gate-fail` | `38e04e0c1f3d0f4732c9473c415ae5c0b164bc175b57b8af709c9268ddfe9ac0` |
| `HANDOFF/PR93_COMMENT_RESOLUTION.md` | `blocked-by-metadata` | `gate-fail` | `83e749da81669cb69218e279054a0d001ffb2771d4bb56c68ca7d8658cfbd55e` |
| `HANDOFF/PROOF_TWO_ENDPOINT_DELIVERY_2026-07-20.md` | `blocked-by-metadata` | `gate-fail` | `836c9c95ae4d1a848a9c547efb14615c29e7e6f96fdf080950830e00b3dd163c` |
| `HANDOFF/QWENFreeQuotaUsageOverview.md` | `blocked-by-metadata` | `gate-fail` | `a82d26c1fcba7e809722276597bc2ac6d9fb9f030eb6f0a481838f09b666ecb8` |
| `SCM-Q-178ab7ca8471` (path hash `178ab7ca847106c9611325018b7de1f440082a03714ac380f9f4a6272583ddcd`) | `quarantined` | `foreign-alias-in-bytes` | `e2e9910a5489054ff88fed6c4c9003d92f7c6fabb5d672d81d821934f6291690` |
| `SCM-Q-86ffa1383645` (path hash `86ffa13836451483edeb87b2aa2267ab07d74b7f9ed0cd0fd9613dcafbc7aa83`) | `quarantined` | `foreign-alias-in-bytes` | `03dc292bcd65b4ebd96ae8a44a60339f37e1ca99b5ef49504b9d1732e810b3d4` |
| `HANDOFF/RELEASE_READINESS_FIXES_DRAFT.md` | `blocked-by-metadata` | `gate-fail` | `533f957245ceb9fd0130f62c53c75b0a4bd549af3c66d7588d2b5dd6bdb49ef3` |
| `HANDOFF/REPLY_2026-06-05_21-25_PT_OPTION_B.md` | `blocked-by-metadata` | `gate-fail` | `ebcbe238127e6ff5d5920fe0dc08a566da1d2bdfe51448b1c616100acca87186` |
| `HANDOFF/REPLY_2026-06-06_01-15_PT_P0_025_RETEST_GO.md` | `blocked-by-metadata` | `gate-fail` | `771f5ce1e6a9c2dfcd5b68b5b796783cd3e0135083bf14add9aa3a72bd23d53a` |
| `HANDOFF/REPLY_2026-06-06_01-45_PT_P0_025_RETEST_RESULT.md` | `blocked-by-metadata` | `gate-fail` | `15dae888351d340861ed5ed8efdd1fddc329c6c8355bc4a5f48e72a9d707ad77` |
| `HANDOFF/REPLY_2026-06-07_20-25_PT_AGY_HANDOFF_TO_CLAUDE_CODE.md` | `blocked-by-metadata` | `gate-fail` | `eb54b460f3b665d407e3b7af9ca2a3250cdaf8ca252741f1f3b527ba279c6d6c` |
| `HANDOFF/RESUME_STATE_2026-07-04.md` | `blocked-by-metadata` | `gate-fail` | `fc44c5e1abc72ae086a55fa95db6cfdb3e27f6463305a09ce37bd2a5159f7247` |
| `HANDOFF/RUNBOOK_PIXEL_PASSIVE_VERIFICATION_2026-09-11.md` | `blocked-by-metadata` | `gate-fail` | `d0c07a42f0074ad63c4cdac90327cbd257c5c7780b289c50376410eace7e2189` |
| `HANDOFF/SCMORC_KICKOFF_PROMPT.md` | `blocked-by-metadata` | `gate-fail` | `7ddd2d1407bac2abbd093bf3d5949287a7d8db58be83390a4d242c50c65d7fe2` |
| `SCM-Q-200437eb46fd` (path hash `200437eb46fd1371390fadd92476ec8e1e3ac56fa689399709a01ac1b482821c`) | `quarantined` | `foreign-alias-in-bytes` | `6d0ba62c27ca0bd4a3ed29ed5e503d817ec4e4b73c652cf7b651e82a1ceb8b5e` |
| `HANDOFF/SESSION_HANDOFF_2026-07-06_scmorc.md` | `blocked-by-metadata` | `gate-fail` | `47857b8fe9b1b4cb830ef4665595e7cf2052af60073b37721bf396a3b4bc3c12` |
| `SCM-Q-5c9e856e38c4` (path hash `5c9e856e38c47466ac368d4aeeced0c77c10c2742a6915f297f6903dbde5a746`) | `quarantined` | `foreign-alias-in-bytes` | `13a44cfd09e4166cfaf5e823fc39408636f11f940b675c8dfedef4324cdbe711` |
| `SCM-Q-835e964b5dcd` (path hash `835e964b5dcd49ed7bbcb67a85b89d2822ea5e44b625d3b1614c6c7d349b4c83`) | `quarantined` | `foreign-alias-in-bytes` | `3600ff18cb81cff40b7edfd8aaa6c4e1023d974aa650be276ee040d30b518aa3` |
| `HANDOFF/SESSION_HANDOFF_2026-07-20_LUCAS_JOSH_ALPHA.md` | `blocked-by-metadata` | `gate-fail` | `ece0969c38f41b4c406b4577a113b40938a7801fe847881175e7f543f01dcda8` |
| `HANDOFF/SESSION_HANDOFF_2026-07-25.md` | `blocked-by-metadata` | `gate-fail` | `bea16e4733e10fe864bf04b69085fddee30d8a654fc2a79cfd93ce27ce740b02` |
| `HANDOFF/SESSION_HANDOFF_2026-08-04_IDENTITY_AND_RUN2.md` | `blocked-by-metadata` | `gate-fail` | `078b9170af4162fc6994370cd6691eef88b58d63ce7f5155094d1c17c7328e28` |
| `HANDOFF/SMALL_FIXES_STATUS.md` | `blocked-by-metadata` | `gate-fail` | `27f009c40553e2cf2213c07973cb5f9bdaae7dc6af646aa8bfbe7e65ff84bff7` |
| `HANDOFF/STATE/.hermes-tmp.94124` | `blocked-by-metadata` | `gate-fail` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `HANDOFF/STATE/2026-06-05_ANDROID_P0_024_P1_022_BUILD_VERIFIED.md` | `blocked-by-metadata` | `gate-fail` | `e04dbfb17c661afcf4ae1c0d48ef66de96f429eb087bde637850a4050cfe1adf` |
| `HANDOFF/STATE/2026-06-05_COLD_SWARM_BOOTSTRAP.md` | `blocked-by-metadata` | `gate-fail` | `3f9ffcae04bec762dc1c27551c8ac7561f9d24bfc4a7fcc299f0bf6e4f41d3f3` |
| `HANDOFF/STATE/2026-06-05_HERMES_P0_024_P0_025_DECISION_RECEIVED.md` | `blocked-by-metadata` | `gate-fail` | `1e34036c455d1fb25b8d00d1e5b5e3d7c80b89b80b131f5d65811a2af5b38d7a` |
| `HANDOFF/STATE/2026-06-05_NEARBY_DISCOVERY_PRODUCTION_PUSH.md` | `blocked-by-metadata` | `gate-fail` | `a1da2ca2edf5452123ace5d9174c456e5fa621a27a76355af1c23f628efe0dd0` |
| `HANDOFF/STATE/2026-06-05_ORCHESTRATION_INDEX.md` | `blocked-by-metadata` | `gate-fail` | `6dc629bd8e1e8f510ea69c2d411d360bc1f28d0131aa6dc8109d414909a693b9` |
| `HANDOFF/STATE/2026-06-05_ORCHESTRATOR_ROLE_PROTOCOL_INSTALLED.md` | `blocked-by-metadata` | `gate-fail` | `2d7324adf7590ad4f9213305be93f53856917650b73336be40d8f49e6a59d37d` |
| `HANDOFF/STATE/2026-06-05_QUOTA_LEDGER_REPAIR.md` | `blocked-by-metadata` | `gate-fail` | `608a8e50fc853a2695db014d0ec2aa222a794354c4fd622398df992269132c2b` |
| `HANDOFF/STATE/2026-06-05_UNIFFI_BINDING_RACE_FIX.md` | `blocked-by-metadata` | `gate-fail` | `9be0ad1e6c6980c17cdbc7fddab9c3d01d60263baf74b222c068aad28caf587b` |
| `HANDOFF/STATE/2026-06-06_01-50_PT_OVERNIGHT_BRIEF.md` | `blocked-by-metadata` | `gate-fail` | `fdbde411896a4398d7763b82894fa09e3adf70ecb5f308bbe26cde2d4259c704` |
| `HANDOFF/STATE/2026-06-06_HERMES_LOCAL_ONLY_SWITCH.md` | `blocked-by-metadata` | `gate-fail` | `97e004f2f364c354150c40035f76e7547cd0050ac2074eb6d8a0139117f17fd6` |
| `HANDOFF/STATE/2026-06-06_HERMES_P0_025_RETEST_GO.md` | `blocked-by-metadata` | `gate-fail` | `06b95a7b1efc174ed9cc0cf2ed227d0944efd0de52090a4db64a1876526bac2f` |
| `HANDOFF/STATE/2026-06-06_OVERNIGHT_BRIEF.md` | `blocked-by-metadata` | `gate-fail` | `2d8ae84bd3b5bf7b223a7a95c46307f3bfb2e5077387efefd2eabd08f9f25119` |
| `HANDOFF/STATE/2026-06-06_OVerseer_PHASE1_RECOVERED_AND_RESTORED.md` | `blocked-by-metadata` | `gate-fail` | `8ab9c33f6ee9cd534d17cdf6ca7ab05c4923aa5d834b8acb47ce7000e111343b` |
| `HANDOFF/STATE/2026-06-06_OVerseer_PHASE2_FIX_COMMITTED_RETEST_BLOCKED.md` | `blocked-by-metadata` | `gate-fail` | `bb22697c763cca35077617145e7057ecf19e0ac16d10e2fc4f77f23590fa6498` |
| `SCM-Q-125b069b8cd8` (path hash `125b069b8cd88c706d7060303e52f51ecce7e9d7871571d4b9992fbb2d44c462`) | `quarantined` | `foreign-alias-in-bytes` | `3b75c3c972bdba319b5596b3581a7d55853768ffffbaf235392006b410a602b6` |
| `HANDOFF/STATE/2026-06-08_ORCHESTRATOR_LIVE.md` | `blocked-by-metadata` | `gate-fail` | `29cbcdd196397eefdf27a142b51fe891a9bb59145787131e90067686b059fb07` |
| `HANDOFF/STATE/2026-06-08_SWEEP_RESULTS.md` | `blocked-by-metadata` | `gate-fail` | `a283bf477c3d7ef09d44256a7ddd1393d680813c5a07243c8374422c2f2f943b` |
| `SCM-Q-fbd0b786c922` (path hash `fbd0b786c9227cda9cc1ad6fe512952c293165350334974f87803634501e1c30`) | `quarantined` | `foreign-alias-in-bytes` | `80aa00ac0a1e35dbbe2c8339a252820418db508e393a3d6112504e715b7813d8` |
| `HANDOFF/STATE/2026-06-10_assembleDebug_pid` | `blocked-by-metadata` | `gate-fail` | `54540cea1e37da3c20705a4dc1867d2419c3cc7f91ed73a03d46cfc59c79d268` |
| `HANDOFF/STATE/2026-06-10_gradle_pid` | `blocked-by-metadata` | `gate-fail` | `d246838d28414818ab0035c2b818fbd1e1d2ecd722309fa909ce14a2db83831b` |
| `HANDOFF/STATE/ESCALATION_LEDGER.md` | `blocked-by-metadata` | `gate-fail` | `c821a2cbf7d08a83128ba3d43ba0b23e80d4780001cdb539b2f96bd0c8ab178d` |
| `HANDOFF/STATE/PLAN_VERIFICATION_2026-06-11.md` | `blocked-by-metadata` | `gate-fail` | `6231bfc895c6b6d46fe7481f62045d93478b0cf24f3fcb5c56e500d7f1f6414e` |
| `HANDOFF/STATE/agent1_complete.md` | `blocked-by-metadata` | `gate-fail` | `72e907087583cefba8236c3c104174707aa117f882f29583ee05a7bd97cfc476` |
| `HANDOFF/STATE/agent2_complete.md` | `blocked-by-metadata` | `gate-fail` | `1e049a48187be62726720b0af9e6ca7bbab8ca9485496e2985a09595e2decd25` |
| `HANDOFF/STATE/agent3_complete.md` | `blocked-by-metadata` | `gate-fail` | `b1a09f1f7565252a8debbbc58d4645f8315349a438a501e1bc5a2e8f6ad10951` |
| `HANDOFF/STATE/agent4_complete.md` | `blocked-by-metadata` | `gate-fail` | `fcc372b854329fe971459a4fd17198989c8364ef311aedb73e9434f872141abe` |
| `HANDOFF/STATE/phase1_complete.md` | `blocked-by-metadata` | `gate-fail` | `ef930f07d92921bdec46ad609604b9caf99a831fa9908d340a3e4ecbd3cf8726` |
| `HANDOFF/STORE_FORWARD_FIELD_TEST_2026-09-10.md` | `blocked-by-metadata` | `gate-fail` | `1a909d780e14e72fef492ad383d9005ddb778b77735082a27107caa17c0fd713` |
| `HANDOFF/SUBAGENT_RCA_BATCH_2026-09-11.md` | `blocked-by-metadata` | `gate-fail` | `8be40b40af4089ac4fbf48a3be2cff4e94ad4b0b2234341a03a4d2fa4bbbd6a8` |
| `HANDOFF/TASK_COMPLETION_VERIFICATION_SYSTEM.md` | `blocked-by-metadata` | `gate-fail` | `725dddcd0f537c39ac249a5e720f125420f4ccd8ec93ea46e7314b4988299486` |
| `HANDOFF/TASK_VERIFICATION_TEMPLATE.md` | `blocked-by-metadata` | `gate-fail` | `d3f96dae443f2bfc646bfa00fc2a4948e42c931bb2cb21798b13dfae93d5003c` |
| `HANDOFF/TELEGRAM_OUT_2026-06-05_21-22_PT.md` | `blocked-by-metadata` | `gate-fail` | `ee82dd1cbe643f18b255ef2f11ff3d7dda6cef8a257322cc7e304e69f324cd4a` |
| `HANDOFF/TELEGRAM_OUT_2026-06-06_01-00_PT.md` | `blocked-by-metadata` | `gate-fail` | `601ef1b6b8adbb9c9589b6b86a48155f0168945e09697ea387808e321389dbb2` |
| `HANDOFF/TELEGRAM_OUT_2026-06-06_01-30_PT.md` | `blocked-by-metadata` | `gate-fail` | `5917a569480f7b0c05a72633797dd77a277c3b4d0348ea0d859444cf9c21f33f` |
| `HANDOFF/TELEGRAM_OUT_2026-06-07_19-55_PT_ORCHESTRATOR_SANITY_HALT.md` | `blocked-by-metadata` | `gate-fail` | `5936713c3baa94d1b08a28621968d01f363d05e7adefca8114863b278fb518d6` |
| `HANDOFF/TELEGRAM_OUT_2026-06-07_22-05_PT_ORCHESTRATOR_HALT.md` | `blocked-by-metadata` | `gate-fail` | `47befab0225b61f75d6fe12e59e8643eff385b9888c02b2c330edc785c0e04df` |
| `HANDOFF/TELEGRAM_OUT_2026-06-07_22-17_PT_HERMES_AUDIT_RECOVERED.md` | `blocked-by-metadata` | `gate-fail` | `a562fa61b85b421d61ec216416fa0035d749ddb8e2f76f89e41124e80b5b81e5` |
| `HANDOFF/TELEGRAM_OUT_2026-06-07_22-37_PT_HALT_CLEANUP.md` | `blocked-by-metadata` | `gate-fail` | `d7e1827d43c40fc4f94e84aa7b14d1d2e192c57eb38aa1c2e2ae1d64963b2eaa` |
| `HANDOFF/TELEGRAM_OUT_2026-06-08_02-02_PT_SWEEP_IN_PROGRESS.md` | `blocked-by-metadata` | `gate-fail` | `82bb5e5762c72871173afe0ee1bd46842eb18d36dfd501ac4ce905b2a30998d5` |
| `HANDOFF/TELEGRAM_OUT_2026-06-08_02-15_PT_SWEEP_DONE.md` | `blocked-by-metadata` | `gate-fail` | `729542f219008f6215018836c87fe720e69255180d98e1f9af2caf24cbb9d7dc` |
| `HANDOFF/UNIFIED_V040_3NODE_DEPLOY_RUNBOOK_2026-09-10.md` | `blocked-by-metadata` | `gate-fail` | `f1be0ab7766f99663f1da54531dec9338eba0a7af32824ddfe2b3f7ee89a0c8a` |
| `HANDOFF/UNIFIED_V040_3NODE_PARITY_PLAN_2026-09-10.md` | `blocked-by-metadata` | `gate-fail` | `12d27fd2f4d944c57941613c5c609c5de7fe2d9907eb48e63ab776cf63955c18` |
| `HANDOFF/V040_3NODE_DEPLOY_EVIDENCE_238a8c53_2026-09-10.md` | `blocked-by-metadata` | `gate-fail` | `85796b5c2b4b373cc7360f9c1db159ba4ac4f00358d5900c9085b294ea175962` |
| `SCM-Q-5a45bbbac841` (path hash `5a45bbbac841fb04ef1580d0bd607976109f6255231e09d0b114662e4de7ae48`) | `quarantined` | `foreign-alias-in-bytes` | `d228cd4bda35fc6a05c1e9d37f5d3eca6027536136e93c8bf7bf82355a6174c3` |
| `HANDOFF/V040_3NODE_RCA_2026-09-09.md` | `blocked-by-metadata` | `gate-fail` | `d5bb975f59c24fc1b74269b6e3d6531a30e795d2bd354efc34e93c3a532ae034` |
| `HANDOFF/V040_3NODE_REDEPLOY_E8C8F52B_2026-09-10.md` | `blocked-by-metadata` | `gate-fail` | `75183060a44609eb7cf8da8087959c1e456ce1e924f66c75df34f0d67401a609` |
| `HANDOFF/V040_3NODE_VERIFY_REPORT_2026-09-11.md` | `blocked-by-metadata` | `gate-fail` | `6bdfcfda90b080cc82ffb37efee7434b229177580d328d8463704f9ba116b67c` |
| `HANDOFF/V040_COMPLETION_PLAN_2026-08-01.md` | `blocked-by-metadata` | `gate-fail` | `27c7b02ab2d5bd666142e0fd562347678f437424d2a25f7d901fa2e3179d5d31` |
| `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260908T120957Z_PREFLIGHT.md` | `blocked-by-metadata` | `gate-fail` | `d4a4dd40cc2b0b823d2467a0ed3221973e7b7fd859cc74f875ec10ddd7eaf20b` |
| `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260908T171520Z_NODE_READY.md` | `blocked-by-metadata` | `gate-fail` | `8f69123c02830763fbe00b7c0c3debbbab03033840de9b6fb400d6104c87ff14` |
| `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260908T174811Z_NODE_READY.md` | `blocked-by-metadata` | `gate-fail` | `fb4e9f7bdeee53090038731a2e21dfbe5564caabbce23ce38f3f8d35ed2ccd4b` |
| `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260908T224800Z_BLE01_SCANNER_FIX_LOCAL.md` | `blocked-by-metadata` | `gate-fail` | `4f0a6c6f004d1dc86f6d0da5d5d0b5f8b89c61766ced42b08f16b7bd6059dbcd` |
| `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260908T232000Z_T14_EXTERNAL_ADDR_FIX_LOCAL.md` | `blocked-by-metadata` | `gate-fail` | `1f238cbb2d7a235f0c58e20f11e21fdc5e80d4ad783722647de55fb6e87cad15` |
| `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260909T001500Z_T14_GOLIVE.md` | `blocked-by-metadata` | `gate-fail` | `a1504b7506bf98edbf5e42f2e535edfb6c167ee8bac4f4426502caba14ecdb9f` |
| `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260909T033500Z_PARITY_PREP_E3E4.md` | `blocked-by-metadata` | `gate-fail` | `61b7992e02d606a59f7a6b5d48b02128e97c0ec759b982a383ab5e102dcd81df` |
| `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260909T053500Z_TRANSPORT_VERIFY.md` | `blocked-by-metadata` | `gate-fail` | `76b6028915e46d34a121ea03075514bbf71a515cbabf7bee7ac385ec6a470a87` |
| `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260909T080000Z_D1_LIVE_CUTOVER.md` | `blocked-by-metadata` | `gate-fail` | `1ea04d1a80bf5e60a618b4a2a2461c239c677cad142c71d2e3f97d60b7012c1b` |
| `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260909T093500Z_DROPPHASE_RCA.md` | `blocked-by-metadata` | `gate-fail` | `8a9795c9e7fdea6b3db0a73cb97bcb701d7b70cad7bd1c2b55ecadeb160e53b1` |
| `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260909T111000Z_UNIFICATION_CODE_FIXES.md` | `blocked-by-metadata` | `gate-fail` | `80efd5175ab0cdc7f911c5be044a03bb6d0e489e25dfa09c428f21ad3cc89d81` |
| `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260909T172000Z_UNIFICATION_GATES_LIVE.md` | `blocked-by-metadata` | `gate-fail` | `9aab6939752ec61f4ec7235501a5021fa55a9b6640a3d8feb35a76a002835ac2` |
| `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260909T190000Z_PR279_FULLGREEN.md` | `blocked-by-metadata` | `gate-fail` | `e2ced022823a11a2cc15087286dbb76d788cbbb90458fbf5953edd7d25f25aba` |
| `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260909T225500Z_ANR_MAIN_FFI_FIX.md` | `blocked-by-metadata` | `gate-fail` | `be2e206114d0bbbe85abc979032556b590cb8b6db24d1d34a54f4796b0c52e05` |
| `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260910T021100Z_ANR_RECURRENCE_CONTROL.md` | `blocked-by-metadata` | `gate-fail` | `d69ae3539ccfd28b65e93da886210a74be7ce4936d894fe33ddb597aea7da9c3` |
| `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260910T051500Z_OPERATOR_TEST_RCA_FIX.md` | `blocked-by-metadata` | `gate-fail` | `1030860f4477bc5e3b7d7d460b4cd19e1378e309f5305710e4a48da5862d11a8` |
| `SCM-Q-ec7fc2fc2455` (path hash `ec7fc2fc24553d09f1ac30c46432535cda7d31dc1cc54a7656b00a6aaf424e70`) | `quarantined` | `foreign-alias-in-bytes` | `d19b220e2331540d70d1cef60d0c771abfdcc9c5fe656883d92f060657ad4fbd` |
| `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260911T060700Z_LOADPEERS_DELIVERY.md` | `blocked-by-metadata` | `gate-fail` | `30ab4a917e81133a23d439f38e3fd3a5a04c8275adb583aef716c2c36c1e8d21` |
| `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260911T180217Z_CELL_SELF_3NODE.md` | `blocked-by-metadata` | `gate-fail` | `4f4c573906525a80fb5d8df677b85e88d47952e41834946ed4bfc29f05f6f076` |
| `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260911T185000Z_CELL_PASS_RECEIPT.md` | `blocked-by-metadata` | `gate-fail` | `08b01f9c8852413ac9e13a52ed726bc9764721e48279d936f267a03d53305fa6` |
| `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260911T200027Z_CELL_27S.md` | `blocked-by-metadata` | `gate-fail` | `5c484a0c3dd8851b3d99263ce01f2348b15ec045bacbd62e6beddffd61f0fe22` |
| `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260911T201606Z_WIFIOFF_2NODE.md` | `blocked-by-metadata` | `gate-fail` | `8e6ad74e56b48e34e7c699905ec7361dff8782605986a0aab885a3681d71b0a7` |
| `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260911T202000Z_CELL_7MIN_FAIL.md` | `blocked-by-metadata` | `gate-fail` | `5232dcfd8b3d99005e1e11cc7352b14a59ce702e65f74d6b055abe524ccd632a` |
| `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260911T203500Z_CELL_STAND_DOWN.md` | `blocked-by-metadata` | `gate-fail` | `f9f52fbf93c31d00ce18f1acc1641cf1628fcb313a05819826891f933ee3b0ce` |
| `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260911T221938Z_EMU_3NODE.md` | `blocked-by-metadata` | `gate-fail` | `a519fcffc432b2d64edd8feaa0c8bc96a9436ab367e8da85525d80c298af7bdb` |
| `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260911T224500Z_HANG_HOLISTIC_RCA.md` | `blocked-by-metadata` | `gate-fail` | `a210caea6b725bccbfbbffd5b1810aff8efd2b22fb5a96b135f46bf2acc3409a` |
| `SCM-Q-751cadb632e1` (path hash `751cadb632e16f9c0a32443799a1bbc178fcef1fa99d7b2263f370d05b1f9eff`) | `quarantined` | `foreign-alias-in-bytes` | `8ab64a2cda72b3408de4145c55391a3044fc036a0220f880c71b0035d32c98e1` |
| `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260911T231500Z_NICKNAME_OWNERSHIP.md` | `blocked-by-metadata` | `gate-fail` | `178da109432f51be0c0fde39efc0e15c3afcf19bd4d41f150f635ca218558b88` |
| `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260917T211913Z_NODE_READY.md` | `blocked-by-metadata` | `gate-fail` | `256c1af2c732c2b6a1edc188997da410ae40ca4687eeb5b1c374ea437341bc86` |
| `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_TEMPLATE.md` | `blocked-by-metadata` | `gate-fail` | `71aa38722e0d6b3c5843a2cc9eec69aa4b5085ded2109ef7921cf26ebbc0ad4b` |
| `HANDOFF/V040_CTO_3NODE_BLE_CONTROLLER_PACKAGE_2026-09-08.md` | `blocked-by-metadata` | `gate-fail` | `5a0b5b4bdbd8d32928e556269492169de047da355bcc69b9f075d4f0a05b5e3e` |
| `HANDOFF/V040_CTO_3NODE_BLE_FINAL_HANDOFF_20260908T120957Z.md` | `blocked-by-metadata` | `gate-fail` | `317f279603f2a8976c9f1a137a22df92205a37b492b534cbe1726ecf9fda4ec6` |
| `HANDOFF/V040_CTO_BLE_ARCHITECTURE_2026-09-08.md` | `blocked-by-metadata` | `gate-fail` | `d568063f91851c975c5c1463f252a267eb846de9a561f26739539886fd55e52a` |
| `HANDOFF/V040_CTO_HANDOFF_SCMESSENGER_IDENTITY_TRANSPORT_WIFI_2026-09-21.md` | `blocked-by-metadata` | `gate-fail` | `d6328b775d380a626f37f0ee54c076641799d00fc8570263f5bf62b63ba6685a` |
| `SCM-Q-45453a7f0772` (path hash `45453a7f0772a954ea1ecdf09ade358d9a38cc25ea4e1c58b1f19df05fa43860`) | `quarantined` | `foreign-alias-in-bytes` | `9ab69f05a124dc69f97121acfc35b68358a5d81e9451c012aa1ca159ad432cf0` |
| `HANDOFF/V040_CTO_NEXTRUN_PACKAGE_2026-09-09.md` | `blocked-by-metadata` | `gate-fail` | `9efc646cdd7556023a04dc5c921221401df5bac217adebf0cac96048d084d9f7` |
| `SCM-Q-115a30503a08` (path hash `115a30503a081f1fd594a5a119e412021f268ee67b0122d2ac0eade576b88eac`) | `quarantined` | `foreign-alias-in-bytes` | `0835d7b930438678144ba4504b28a1aa331aa688ebfe46245a8186fc9ac2450c` |
| `HANDOFF/V040_DIAG_PLAN_2026-09-11.md` | `blocked-by-metadata` | `gate-fail` | `cf00cefc58c936a1441d7bbbefcc145aafff0196571dde1ff3ce360863b3c9ff` |
| `HANDOFF/V040_E8_ADVERTISEMENT_CONFIRMATION_SPEC_2026-09-09.md` | `blocked-by-metadata` | `gate-fail` | `1738a9a169203f6f975a6335a7ef48818b362724c9d44db04767ef2598ed004b` |
| `SCM-Q-28b3798ccf32` (path hash `28b3798ccf32c67d23b8e311ff2c9099b73858c6b7e4f2e1a0201cdbc4fdd2a1`) | `quarantined` | `foreign-alias-in-bytes` | `84ae954a28dc8a267f307bf24d480a5af1ec0c1472d8d243961827f452e5101b` |
| `SCM-Q-4765693b5667` (path hash `4765693b5667a790ef5100a5410092cc735a0b6667ee385e0961bad7d89125b6`) | `quarantined` | `foreign-alias-in-bytes` | `bcd3a1a3805e888743e53ca48cc47e9d7601d49c0092bc6a7ab854585651add4` |
| `SCM-Q-4d371a42f88b` (path hash `4d371a42f88b97f2e8bb77031cdc690c4d385e8c593261461d3814a14c0736fa`) | `quarantined` | `foreign-alias-in-bytes` | `20fcd04142fda60197cbd691abd12e2ab1cad1cd0b8054d9858d668b40b028cd` |
| `SCM-Q-aaa51755383a` (path hash `aaa51755383a44245e7929231253ed305fc420018a1f4ad3620d4dcaa8adff44`) | `quarantined` | `foreign-alias-in-path-and-bytes` | `639851d2b6e71e7175067051943faccedac87478fbd81748771303424d7c8c34` |
| `HANDOFF/V040_MERGE_EXECUTION_LOG_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `be3d2b5f5c2c0b0a39f4630ae3fe93f315e2944ec4345c39198a3715c2781d3d` |
| `HANDOFF/V040_ORCHESTRATION_COMPLETE.md` | `blocked-by-metadata` | `gate-fail` | `613d83259a807bf525ae5e2adac9bdd5db240e5ee91ea0b27a444c10ec98c1c3` |
| `HANDOFF/V040_PASSIVE_3NODE_DEPLOY_READINESS_2026-09-10.md` | `blocked-by-metadata` | `gate-fail` | `93624efed7ddf71ea854c8f02121ec6221e0b20f2ee2a003ef3ed77eb8bb3d84` |
| `SCM-Q-6abf312fb078` (path hash `6abf312fb0783f048436dec7d0a5cd51d703f1405340e121f82bafa0453aabcc`) | `quarantined` | `foreign-alias-in-bytes` | `a01d70b66d20a0086b6cca96b305647f6db4bdff911548a7cc3e3a4601c6aaf6` |
| `HANDOFF/V040_V050_UNIFIED_PLAN_2026-08-01.md` | `blocked-by-metadata` | `gate-fail` | `aeb90e4b26c6e644e42d385e75d7ddc71971afaa82e609b9872ae690975bcb88` |
| `SCM-Q-13eda9f5902d` (path hash `13eda9f5902dbf6badcb2e1e25fc1ca1e3585ae25a496686bf29579f8a391d79`) | `quarantined` | `foreign-alias-in-bytes` | `14b938186cf2f09b85fa8b3884cf751f5a8eda860c09b2ba08ec268b7fd092d7` |
| `HANDOFF/V1_0_0_EXECUTION_PLAN.md` | `blocked-by-metadata` | `gate-fail` | `e82726a05439c4f07f8dbca29e238d82d4a1e40b7618af055f897cc334f8c516` |
| `HANDOFF/V1_0_0_UNIFICATION_PLAN.md` | `blocked-by-metadata` | `gate-fail` | `cb61b17b595586e5df80324d9fda2550c476399174183b554423446904c25bc3` |
| `SCM-Q-7bcdf6cd03b5` (path hash `7bcdf6cd03b578732c0910fcaa4818f284019b4b049dad5b9b4d6d73f516aaea`) | `quarantined` | `foreign-alias-in-bytes` | `124b6a7455d7672afa427d1bfe5d75a79e52b9cf470622a9e94a1ae64bfc5318` |
| `SCM-Q-04fccd6e2dfb` (path hash `04fccd6e2dfbb282d74e839df45bb13b3c26451299c283ad41a9201049320d09`) | `quarantined` | `foreign-alias-in-bytes` | `653d88b363deb09517aec01793ee25b633ac57174e531dcc28e713c8a2e465f5` |
| `HANDOFF/WIRING_PATCH_MANIFEST.json` | `blocked-by-metadata` | `gate-fail` | `f75dc1654863aaca33b241d7c413dcfa6e9d24e738690bce4ed3eca87c56190b` |
| `HANDOFF/WIRING_PATCH_MANIFEST.md` | `blocked-by-metadata` | `gate-fail` | `cbfce8015a5be0c4042f5ba5bc385a9a7baa9079e06708ba2de56b867568bc8f` |
| `SCM-Q-ce67a98c004f` (path hash `ce67a98c004f8dd04d6ec1ee5ecad01c612bcbde5a59735923906a7d932f573b`) | `quarantined` | `foreign-alias-in-bytes` | `80d2010bfeb0d3d2d4036aca6f78ede0be1424b5356471fad95c22f3b01e1b7e` |
| `HANDOFF/archive/A-09_RELAY_DISCOVERY_DIAL_AMPLIFICATION.md` | `blocked-by-metadata` | `gate-fail` | `a328f19da818ebe28d2db8aee020789031e22b73eee2b38072213cf4b8ea80eb` |
| `HANDOFF/archive/ANDROID_FRESH_INSTALL_RUN2_READY.md` | `blocked-by-metadata` | `gate-fail` | `532419b4519d0c641f44fbffe883d6d1357290fc293ff3ec7b9fa8a1aa4f818b` |
| `HANDOFF/archive/ANDROID_REINSTALL_UPDATE_INBOX_BRIDGE_ALLOWLIST.md` | `blocked-by-metadata` | `gate-fail` | `10d2589479f081eb3928ac09b83ec3cefce4a7a940a5a63ca5e2707130ff4ed8` |
| `HANDOFF/archive/APP_SHARING_IOS_PARITY_CROSS_INSTALL_2026-08-05.md` | `blocked-by-metadata` | `gate-fail` | `a8fb2ddfc95474ed0f162cfa360d96c4adf34f4c918e5b820e3c54ad2e73d64d` |
| `HANDOFF/archive/AWS_CLOUD_NODE_PRELAUNCH_CHECKLIST.md` | `blocked-by-metadata` | `gate-fail` | `aa18514f615583da494cb1a769215426752e65b98494b7fb1f3563ff3452b693` |
| `HANDOFF/archive/BOOTSTRAP_TOPOLOGY_WIRING.md` | `blocked-by-metadata` | `gate-fail` | `1be1a40e43d0118d1415066af671e0b804350353a40c72a54621b40db3080068` |
| `SCM-Q-a19428f492ea` (path hash `a19428f492eabe692bbc238ab004c252ef745561a16d8d38db172c41cda5a5be`) | `quarantined` | `foreign-alias-in-bytes` | `3e514a34a78e5756c2a18401f6f3d5873836ff9554cb22eb44f6c158bf6ebccf` |
| `HANDOFF/archive/C-06_P1_18_relay_task_3_node_custo.md` | `blocked-by-metadata` | `gate-fail` | `ff04ecdbcaa1b39997e9ba094d5a70e5529a87b85d710a3083c828fe0d2fa121` |
| `HANDOFF/archive/CEO_CTO_COMMIT_CHECKPOINT_PROMPT.md` | `blocked-by-metadata` | `gate-fail` | `97b92c3a4a0fffca24f7bfeabe5a6a4ac68133960d94e960de13fef8ed2d83f4` |
| `HANDOFF/archive/CEO_CTO_SPINDOWN_AND_BOARD_PASS_3_2026-09-11.md` | `blocked-by-metadata` | `gate-fail` | `5234402b88ce72f32627b0831c582c4a09b50f20fa006705634854a6b16339f0` |
| `HANDOFF/archive/CLAUDE_CODE_SONNET_LOCKOUT_2026-08-04.md` | `blocked-by-metadata` | `gate-fail` | `648d6b50a0a86d11c576dea4fc0bae0af48d48015bb31e558bea57eae823ad20` |
| `HANDOFF/archive/COMMIT_CHECKPOINT_PROMPT_multi-repo-2026-09-11.md` | `blocked-by-metadata` | `gate-fail` | `297c5b7a7166889e4ed59a01aa6d169140291163fd1e06205a8cbe54e124429a` |
| `HANDOFF/archive/COMMIT_CHECKPOINT_PROMPT_push-enabled-2026-09-11.md` | `blocked-by-metadata` | `gate-fail` | `14e8d63801c4c515379bf806ce7239c2212e8a57acf8aa675046b8662e1ef7cf` |
| `HANDOFF/archive/COMMIT_CHECKPOINT_PROMPT_single-repo-2026-09-11.md` | `blocked-by-metadata` | `gate-fail` | `e59513fdcdcad1279489693226ededb45fa657a78b6f86e91ac60e209bed8904` |
| `HANDOFF/archive/CONTACT_PROVISIONING_FIX.md` | `blocked-by-metadata` | `gate-fail` | `4c3b4f48e1cc882c7b110e3bf95ad7d1f5100ba2393fb6eb9d72ed86c00ce60f` |
| `HANDOFF/archive/D-02_Android_Robolectric_wiring_ad.md` | `blocked-by-metadata` | `gate-fail` | `b96054653378ba664641ab93b53c672b85729910215c20af2e78bf06c8fe3628` |
| `HANDOFF/archive/D-03_iOS_XCTest_target_register_SC.md` | `blocked-by-metadata` | `gate-fail` | `ea8ace97202fb239726ed57f07090897f4ecb39ebf6a50623216f814c723f6d1` |
| `HANDOFF/archive/D-04_Emulator_instrumented_test_job.md` | `blocked-by-metadata` | `gate-fail` | `b80370f989503d06901b68e3260b27330c7fa56dfc0a1fd7c43cbac62d987efc` |
| `HANDOFF/archive/D-05_unwrap_panic_hardening_v1.md` | `blocked-by-metadata` | `gate-fail` | `25cad03cd73dc2c42cab0d8915b14dc3077b445deedf0553bc87581307d86d87` |
| `HANDOFF/archive/DELEGATE_SCOPED_FILES_ALLOWLIST_FIX.md` | `blocked-by-metadata` | `gate-fail` | `35ea38cd133a62e3c1214a54b07adb73b9a2d0760cd3d5933ee531b764195a17` |
| `HANDOFF/archive/DISPATCH_AUDIT_LEDGER_VISIBILITY_GAP_2026-08-05.md` | `blocked-by-metadata` | `gate-fail` | `74e28b091d1d213c6f88fc660ef4bcd5b2d1f2f4749a523ee0c66fcf0352644a` |
| `HANDOFF/archive/DISPATCH_AUDIT_TRANSPORT_FAILOVER_HICCUP_2026-08-05.md` | `blocked-by-metadata` | `gate-fail` | `2415ef8c6a236c405bbf587d48c4e8e7f0c68ce5efc5f0c6a466025d17aaf470` |
| `HANDOFF/archive/DISPATCH_DESIGN_TRUST_SCOPED_LAN_DISCLOSURE_2026-08-05.md` | `blocked-by-metadata` | `gate-fail` | `7d3b045d9e73618b42e095e87d98b18cb4fccc9131201f662435db5c70ccb6c2` |
| `HANDOFF/archive/DISPATCH_IMPL_ANDROID_MDNS_PARITY_2026-08-05.md` | `blocked-by-metadata` | `gate-fail` | `035d111bcbc967f6dd03a1af1e56f7107a064e7c3734199d68697c68f331645e` |
| `HANDOFF/archive/DISPATCH_IMPL_TRANSPORT_LIVENESS_P1_2026-08-05.md` | `blocked-by-metadata` | `gate-fail` | `e0cbd083d48d237a8cd5314b1fb73a54a47e5fc2b8dc3deb68324e868ab1d4ba` |
| `HANDOFF/archive/DISPATCH_REREVIEW_TRANSPORT_LIVENESS_2026-08-05.md` | `blocked-by-metadata` | `gate-fail` | `d0f3d046fbe267e8efb49bd9c2cb0ab741a9bff998908112748e8ba57d4167dc` |
| `HANDOFF/archive/DISPATCH_REVIEW_TRANSPORT_LIVENESS_2026-08-05.md` | `blocked-by-metadata` | `gate-fail` | `588e70ca28f9dabfc42cf5edc749fd1c6089799ec31bebc9d5450bec9d39f8b9` |
| `HANDOFF/archive/E01B_FABLE_DESIGN_HANDOFF.md` | `blocked-by-metadata` | `gate-fail` | `66daa49a0d4b4b49d82b022987905fb90ecddd5ea9a065d5fc568f123383c776` |
| `HANDOFF/archive/EXECUTE_PHASE_2_3_ON_INSTANCE.md` | `blocked-by-metadata` | `gate-fail` | `a1bb0da462066c56b62b79866af675656067f208685b9c1ffeab359d9f5c9888` |
| `HANDOFF/archive/FARM_SIM_PHASE_2_3_FINDINGS.md` | `blocked-by-metadata` | `gate-fail` | `326232f87d039bbcb0d0b7bf7d44d25a86159a9b93f5d1169de4e0eff074fd3b` |
| `HANDOFF/archive/GRACEFUL_AF_DIAL_POLICY.md` | `blocked-by-metadata` | `gate-fail` | `74a96c9fda0b2cefc29520484a51c90d611647609d26b3f71dd80079da798a6a` |
| `HANDOFF/archive/IDENTIFIER_GATE_FOLLOWUPS_2026-08-04.md` | `blocked-by-metadata` | `gate-fail` | `63f08c8ac42081e5b025ea5886c848f6d9144dbb8d6912b9ccbdff13a5834081` |
| `HANDOFF/archive/IDENTITY_BACKUP_UI_INTEGRATION_2026-08-12.md` | `blocked-by-metadata` | `gate-fail` | `19459fbce51a811333fd9448a7461134683e0298ec33ec06aa9c099b774829ce` |
| `HANDOFF/archive/LEDGER_ADDRESS_IDENTITY_MISATTRIBUTION_2026-08-09.md` | `blocked-by-metadata` | `gate-fail` | `8c81b26d3eb7f4699ff78a1c27f13c7a905edac160c33b8d932e605ef69427df` |
| `HANDOFF/archive/LEDGER_CHOKE_POINT_REFACTOR.md` | `blocked-by-metadata` | `gate-fail` | `1d951c5e3cb0030156ee44470919474c774d45b150d59adba2083edba7c06407` |
| `HANDOFF/archive/LEDGER_SHARING_ANDROID_NODE_VISIBILITY_2026-08-05.md` | `blocked-by-metadata` | `gate-fail` | `199705846f794bb96bf01bcab5f54c18a073839d7d57945d6322317b370080a2` |
| `HANDOFF/archive/ORCHESTRATE_FARM_SIM_FIX_AND_RETEST.md` | `blocked-by-metadata` | `gate-fail` | `5ff735cf05f4c4331ae2d5a6f321e9406fe39ab662661636c6a79a6d13d1497c` |
| `HANDOFF/archive/ORCHESTRATE_STRICT_HARDENING.md` | `blocked-by-metadata` | `gate-fail` | `f1b0a2a5d1c030e9f95fcc4f22e61b002628bd4e5820411e0c76857673318e21` |
| `HANDOFF/archive/P0_BLE_L2CAP_ACCEPT_SPIN_2026-08-08.md` | `blocked-by-metadata` | `gate-fail` | `f9c24ed92d85cbb5246b7af50673344491d023776c66782ec6fdd873c4b39087` |
| `HANDOFF/archive/P0_NO_MOBILE_BOOTSTRAP_MEANS_NO_OFF_LAN_RENDEZVOUS_2026-08-10.md` | `blocked-by-metadata` | `gate-fail` | `c22a3c2698b80b917af66bd368ba8d86169075405c79a1aa2e2a6ca4731032c9` |
| `HANDOFF/archive/P0_NO_RELAY_FALLBACK_FOR_ROAMING_PEERS_2026-08-10.md` | `blocked-by-metadata` | `gate-fail` | `5c9e7a44ebe570d742917ad51328df49cd9eeecbf17299a2b001a85a2a15588c` |
| `HANDOFF/archive/P0_REQUEST_RESPONSE_PANIC_KILLS_DESKTOP_ON_MESH_GROWTH_2026-08-09.md` | `blocked-by-metadata` | `gate-fail` | `23e70dc11670914a3b6223db11b0fb043dc1e8414d3a9f30973251fd1393b4f2` |
| `HANDOFF/archive/P0_UPNP_PANIC_KILLS_DESKTOP_NODE_2026-08-08.md` | `blocked-by-metadata` | `gate-fail` | `be1f64ac306866508ea4aa379fd20b240a848ac08ec03f35554719ace5c132d4` |
| `HANDOFF/archive/P1_AWS_RELAY_NOT_REMOTELY_MANAGEABLE_2026-08-09.md` | `blocked-by-metadata` | `gate-fail` | `10f3dd49a0eafd0f242b8c716fc433797daeb5239afbcfd3ccff701bf1c41754` |
| `SCM-Q-3a9de2044f08` (path hash `3a9de2044f08d7e98b0abca4b92005b2d91098aa157fe735285ca5920e5eabca`) | `quarantined` | `foreign-alias-in-bytes` | `2ad986113b97f3df10078078a830a86f583c67f1785509192e555a8306082cc3` |
| `HANDOFF/archive/P1_DISCLOSURE_CGNAT_AND_FAILOPEN_2026-08-09.md` | `blocked-by-metadata` | `gate-fail` | `6dfd1081d4e659dc3f9ea52ac78c305b62091542d3ac4e45e079ca6ca4d7b0af` |
| `HANDOFF/archive/P1_NESTED_CIRCUIT_ADDRESSES_STILL_FORMED_2026-08-10.md` | `blocked-by-metadata` | `gate-fail` | `a6d9676ba33015a56e5e1c3d2db0711addaaccea082e6d8823873d0d3ff0de9b` |
| `HANDOFF/archive/P1_PROMISCUOUS_DIAL_WASTES_BUDGET_ON_SELF_AND_CELLULAR_2026-08-09.md` | `blocked-by-metadata` | `gate-fail` | `00bad33fadcaf0d7021e0a1c3c571e45f7e057c914a9cde508d5c6b19645daf2` |
| `HANDOFF/archive/P1_PRUNE_CLAUDE_DATA_2026-08-08.md` | `blocked-by-metadata` | `gate-fail` | `d5858e266527c0086a2dbd2ac2b4c4bed75b23a859d2667c4b884242e35bfec2` |
| `HANDOFF/archive/P1_STALE_BUILD_PROVENANCE_INVALIDATES_SHA_CLAIMS_2026-08-09.md` | `blocked-by-metadata` | `gate-fail` | `245b33f9221435da935ba6ed2e9bf842a15953071fce84ab8449afc640a74b4e` |
| `HANDOFF/archive/P1_TEST_FIXTURE_ADDRESSES_LEAKED_INTO_LIVE_LEDGER_2026-08-09.md` | `blocked-by-metadata` | `gate-fail` | `b1d6a1c0a8acb434e1971d5d55bd066b84fd021d4e3219b3b4a624632260162d` |
| `HANDOFF/archive/P2_RESTORE_UPNP_ON_0_7_0_2026-08-09.md` | `blocked-by-metadata` | `gate-fail` | `815b40d6381b68aa8ef9ca7e57cc42d1d83d12dc1847928b91e95c102a93ede3` |
| `HANDOFF/archive/PANIC_libp2p_request_response_duplicate_disconnect.md` | `blocked-by-metadata` | `gate-fail` | `3553b979fdf09168f4628d59790b3898cec1ecb02db6f566b06fbd32efa95ff2` |
| `HANDOFF/archive/PQC_00_MASTER_PLAN.md` | `blocked-by-metadata` | `gate-fail` | `fa8e1acf935eeaee990812ddcf6a4c55361302f28b71bcbe7caae2f76d0e5453` |
| `HANDOFF/archive/PQC_09_HYBRID_ONION.md` | `blocked-by-metadata` | `gate-fail` | `cc9266d0963cff087e964f54f98f32939190035668207ed9824f7641d6099ecd` |
| `HANDOFF/archive/PQC_09_ONION_COMPILE_FIX.md` | `blocked-by-metadata` | `gate-fail` | `33717903bf91d8afa36f324dcedf3ce00dd47cdfb99146765ddba664467e9899` |
| `HANDOFF/archive/PQC_09_SECURITY_REVIEW_FIXES.md` | `blocked-by-metadata` | `gate-fail` | `c170ac3c4d614906588f6ff7862572c7412e872e62dfa2b326c8319385ffcb8a` |
| `HANDOFF/archive/PQC_10_MLDSA_MODULE_MISSING.md` | `blocked-by-metadata` | `gate-fail` | `e4899e4885eef56f71d5aaf857280ebd1b247cdf23cb95029a7bb6e728f2560e` |
| `HANDOFF/archive/PQC_SEEDING_SECURITY_HARDENING.md` | `blocked-by-metadata` | `gate-fail` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `HANDOFF/archive/PR136_IDENTITY_FIX_CI_STATUS_2026-08-04.md` | `blocked-by-metadata` | `gate-fail` | `efa193f0c40fbd62fb49d650656277d36af7c6a50f94d08bbcff440e47f1db3e` |
| `HANDOFF/archive/PROMISCUOUS_ACCEPT_UNROUTABLE_ADDR_2026-08-09.md` | `blocked-by-metadata` | `gate-fail` | `9192bf428975077a60276330ed3616b59a79bd75fef3150b3ff86884393c6eaa` |
| `SCM-Q-7d388770f32e` (path hash `7d388770f32e27a2298c4e1d03bd833d97908f6d7955722fde38d59e040cf9d0`) | `quarantined` | `foreign-alias-in-bytes` | `d75e8c18b378a2a40287d6e5c4c49e260083b76cca418c188d2fb025c500e109` |
| `HANDOFF/archive/QWEN_IDENTITY_CANONICALIZATION_CRITICAL.md` | `blocked-by-metadata` | `gate-fail` | `355aae94f052d301f0a7e8fcb24c639824aa742d2effbf3a0fa1aaef85be7941` |
| `HANDOFF/archive/QWEN_RUN2_CLOUD_NODE_VERIFICATION.md` | `blocked-by-metadata` | `gate-fail` | `d1da62345545479e11ce8b034476f517c269730d8568ba08de67188b22c93382` |
| `HANDOFF/archive/REGENERATE_LAKE_REGISTRY.md` | `blocked-by-metadata` | `gate-fail` | `ee19e429f9cd3598a2073cf81554e055bf7af040dc2af49f937e855160a29b0d` |
| `HANDOFF/archive/T-02_T1.md` | `blocked-by-metadata` | `gate-fail` | `f6de8b2259d408e87aac80d81aad08d66d4b544c272e7e346f86919d8af5a062` |
| `HANDOFF/archive/T-03_T1.md` | `blocked-by-metadata` | `gate-fail` | `b34c89631026dba7a5fcbf94272aa9976aa368b6cb65f9592eb5cf25ca0f5f5f` |
| `HANDOFF/archive/T-04_T2.md` | `blocked-by-metadata` | `gate-fail` | `471ebe3302bb0ee3e6f2422a143b7e37abf5e899688379a49f7b28f06efca7a5` |
| `HANDOFF/archive/TASK_KMP_COMPOSE_ARCHITECT.md` | `blocked-by-metadata` | `gate-fail` | `cc38f258c5e9ba63734e8b7f822e78f0bf96d07ea36374b6f395b9a03d8b0ed4` |
| `HANDOFF/archive/TASK_KMP_DEVOPS_PACKAGING.md` | `blocked-by-metadata` | `gate-fail` | `9442271612174553fa1efc50306b8db368a2753b011ac0b421e5dd5dbf6f9648` |
| `SCM-Q-868db4829b65` (path hash `868db4829b657d256eea50411a75c96f8e487a6767cce39454550414f7fde5c5`) | `quarantined` | `foreign-alias-in-bytes` | `d8880809b9746ca8188bbdb4973a4919cb523c390d7291470ce71a4b59ccc7f8` |
| `HANDOFF/archive/TASK_KMP_RUST_UNIFFI_LINUX.md` | `blocked-by-metadata` | `gate-fail` | `d005556719702a05bfadeed0e238a153b962905e99e90957f1e8d49d0bda43dc` |
| `HANDOFF/archive/TRANSPORT_BLE_LAN_HICCUP_VERIFICATION_2026-08-05.md` | `blocked-by-metadata` | `gate-fail` | `195eef46d968e2eb399386b1ac4095a75d3bc55b75715219cf11cc2da025418f` |
| `HANDOFF/archive/U2_TOPIC_CONSTANTS.md` | `blocked-by-metadata` | `gate-fail` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `HANDOFF/archive/U3_RETRY_POLICY.md` | `blocked-by-metadata` | `gate-fail` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `HANDOFF/archive/U6_IOS_RECEIPT_UNIFICATION.md` | `blocked-by-metadata` | `gate-fail` | `b761c3e904f82a5496ae6910e568af3b4de04ddf9c2f50c9b841ca77c4de3593` |
| `HANDOFF/archive/UNIFICATION_AUDIT_FINDINGS.md` | `blocked-by-metadata` | `gate-fail` | `a78c8a727b24313bec7441d476e3b2d9e6a5c7c5a8af3a5d8fcdbb7be07e1b7a` |
| `HANDOFF/archive/V040_FREE_LANE_DISPATCH_READY.md` | `blocked-by-metadata` | `gate-fail` | `0ccb4650b742e048597c9a68ba253864334059b2618f0e378bc5a651ede41814` |
| `HANDOFF/archive/V1_INSTALL_ARTIFACT_FOR_ALPHA_TESTERS.md` | `blocked-by-metadata` | `gate-fail` | `27d8e3aa59c12e1538120080106b5915318f221f1a29c21aabfeff1677abdc14` |
| `HANDOFF/archive/VERIFY_LEDGER_EXCHANGE.md` | `blocked-by-metadata` | `gate-fail` | `cb8598ca2c1a69dbb62ca08e5f0093f6eb89492eee925322088fb203a5e236e0` |
| `HANDOFF/archive/WAVE_B_FREEZE_STATUS.md` | `blocked-by-metadata` | `gate-fail` | `a674f727f3030c85e53f10a08ab8ccc71d47c0dbd6f5963bc1f1318e57e19652` |
| `HANDOFF/archive/WINDOWS_BLE_PANIC_TOKIO_SPAWN_OUTSIDE_RUNTIME.md` | `blocked-by-metadata` | `gate-fail` | `214999edc95cb77e456e57eb729a9cae9db8b29df7cdf2610f524e5f6fa31d31` |
| `HANDOFF/archive/_NEXT_ORCHESTRATE_KICKOFF.md` | `blocked-by-metadata` | `gate-fail` | `e0e0e7b91ef77a318d5da852accee470cd3ac7bcb09d6302e712c16f15d95712` |
| `HANDOFF/audit/AWS_RELAY_REBUILD_2026-08-04.md` | `blocked-by-metadata` | `gate-fail` | `0b96c25d2b3c5f14b16c95f53789c88b39373f9bd1c8efad3d94f94c32cc2e18` |
| `HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_INDEX.md` | `blocked-by-metadata` | `gate-fail` | `b27036184a7aa31f04cb14e23b7d9a815b683633c82620db5287d8529ddd4711` |
| `HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_iter0.md` | `blocked-by-metadata` | `gate-fail` | `ce6c8cf4cb410cb9e139336ff6b67e1d3e479c19e253ad71a8b19bd908796ce9` |
| `HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_iter1.md` | `blocked-by-metadata` | `gate-fail` | `e8bf52a3daf39e55e350a1e7d7c8cacd907930a3e80c4af01fe2a1c04613be05` |
| `HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_iter2.md` | `blocked-by-metadata` | `gate-fail` | `112706926a5307e1918ffbee705f50d0cb27368af8e52df6d4ed364662718bbc` |
| `HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_iter3.md` | `blocked-by-metadata` | `gate-fail` | `c6b1a2c595b8da5b419f2c5904d50415a6a34aa64020957b7b6194d4b3885a07` |
| `HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_iter4.md` | `blocked-by-metadata` | `gate-fail` | `f8cb4d3bab3379166af3cf7e939c203d4fcfc9752c51ecea0021552c02384755` |
| `HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_iter5.md` | `blocked-by-metadata` | `gate-fail` | `179fe5761c9518ba5a4e2e082a63dc74d955e6046cb5d444cdbe4bda432be814` |
| `HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_iter6.md` | `blocked-by-metadata` | `gate-fail` | `1db08b1df1e2b7dd2f3596738c79390763ba1e32661275fae3d1aa1999fe2b06` |
| `HANDOFF/audit/CELLULAR_PATH_TRIANGULATION_2026-09-15.md` | `blocked-by-metadata` | `gate-fail` | `626b15b9a25acd8435ecc2658883c844d992d036924b6ab7252468be4fea5f3e` |
| `HANDOFF/audit/D2_KEYSTORE_VERIFICATION_2026-09-13.md` | `blocked-by-metadata` | `gate-fail` | `108d6f1f4743f0b0bca736ef38deea199b15319418cd9091508b61f6c7012827` |
| `HANDOFF/audit/DIRECTIONAL_PARITY_DIAGNOSTIC.md` | `blocked-by-metadata` | `gate-fail` | `5fec69db3d9ca962357371c4d3ec5d314c229f04df73f3d6b8e64ede400fb5d1` |
| `HANDOFF/audit/FIVE_NODE_RUN_1_ANALYSIS.md` | `blocked-by-metadata` | `gate-fail` | `871180980dd2be5f59dea21b61fc8bfe9877fffe689671e22df048a7ded64309` |
| `HANDOFF/audit/HARDCODED_IP_SWEEP_2026-08-04.md` | `blocked-by-metadata` | `gate-fail` | `f929a549bbe9c8ef277d6e0dd39e8fa9cf2fc786769be51fcc0b3182c894164c` |
| `HANDOFF/audit/IDENTIFIER_PARITY_AUDIT_2026-09-15.md` | `blocked-by-metadata` | `gate-fail` | `9da4b3a13fa97588945698c8435726a31f9a9ae6ed4bb9c8edcbbd8dad52fc9d` |
| `HANDOFF/audit/IDENTITY_HASH_VS_PUBKEY_CONFLICT.md` | `blocked-by-metadata` | `gate-fail` | `14d1961c4c22a3ff383404e2c2b3aee8758f9b80f40368428eb02deb61edc7cc` |
| `HANDOFF/audit/IRON_CORE_COMPLETE_AUDIT_2026-08-08.md` | `blocked-by-metadata` | `gate-fail` | `3aed2f66bec3c8480e3aae669d23e7ad03ce1bdf8e0a073af2bfc2f9519e5782` |
| `HANDOFF/audit/MULTIDIMENSIONAL_AUDIT_2026-09-17.md` | `blocked-by-metadata` | `gate-fail` | `c682f88a58b91418a16b65bb6b1aef710010510a731202e7f99c5a7cb6b82198` |
| `HANDOFF/audit/ORCHESTRATION_AUDIT_QWEN_TAKEOVER_2026-08-04.md` | `blocked-by-metadata` | `gate-fail` | `db2109c5290ba37692c0d09d3732e3236dae501f49fbc707da4d08bef2fd7904` |
| `HANDOFF/audit/PR134_REMAINING_TASKS.md` | `blocked-by-metadata` | `gate-fail` | `bbf8252ece65384ddbb96a96a4ba761c7cff4088d3729b9fbfda9cb739cd4f0c` |
| `HANDOFF/audit/RCA_OLD_PIXEL_577fd171_2026-09-11.md` | `blocked-by-metadata` | `gate-fail` | `a18efd220026246f437c19773943c58d3ef84319d9d5e40a018d7139f8d9a8ce` |
| `HANDOFF/audit/RCA_STOP_RACE_AND_CELL_STORED_2026-09-11.md` | `blocked-by-metadata` | `gate-fail` | `8fe23d44dd6d935350856f6c27717480c82ee0e822866b129663af3aa98b2438` |
| `SCM-Q-3cf831a02330` (path hash `3cf831a023301d2eb293d231b27988979e856c4eebea373aa9d0ac2796f46b12`) | `quarantined` | `foreign-alias-in-path-and-bytes` | `4c2a05b6962a9f2069a422dbd15f31e3833719558e02a1d8d85546289cefec58` |
| `SCM-Q-8319c26406e5` (path hash `8319c26406e54fdba97b29e3bdd5f5e2d198dc1175bc03ae34eb335574efe673`) | `quarantined` | `foreign-alias-in-bytes` | `14bac999fc1ba56f296d225d604b85922835ee040196c8318b705f9eaf276e9a` |
| `HANDOFF/audit/SHADOW_AUDIT_V040_V050_ADVERSARIAL_REVIEW_2026-09-16.md` | `blocked-by-metadata` | `gate-fail` | `41f59cdcea029825edf1821c04c1abaf925a7c367107fe8575a44cf69e2302c9` |
| `HANDOFF/audit/STRICT_CELL_TRANSPORT_AUDIT_2026-09-11.md` | `blocked-by-metadata` | `gate-fail` | `4f860a6547e81ef574cc6e3bc933436833a9f64fb604a7ddab41e6e3b07c3ad5` |
| `HANDOFF/audit/T1_T2_CENSUS_DISPOSITION_2026-09-13.md` | `blocked-by-metadata` | `gate-fail` | `70b7bf9dbb182eee3006ca51a519aa4169d028570df23000c5414ead9a3a4112` |
| `HANDOFF/audit/T4_ROUTING_FEED_ANALYSIS_2026-09-13.md` | `blocked-by-metadata` | `gate-fail` | `318a8096a6fd10d9c52104c12e00edca84ab809d728540c3fc606051439acb1d` |
| `SCM-Q-53cecb2b2530` (path hash `53cecb2b2530664189252e2e62ce6f6a7ae1cec2f67d89fbb16dec235862d62e`) | `quarantined` | `foreign-alias-in-bytes` | `2a8290903cf73a4119483c439adfbe0a36be2aa363deb84e857fe12e3d955ab6` |
| `HANDOFF/audit/ble_gatt_restart_fix.md` | `blocked-by-metadata` | `gate-fail` | `ce839bee98f67413c778bfacbc7e74fbdf811018485be2996fb77d3749d4e13d` |
| `HANDOFF/audit/ble_wedge_rootcause.md` | `blocked-by-metadata` | `gate-fail` | `bf0c2349032bab2d55376f1b12c70886dce98a11e235b59ca42128ace30b4c4b` |
| `HANDOFF/audit/ble_wedge_rootcause_v2.md` | `blocked-by-metadata` | `gate-fail` | `99c4da198a19e45d163b5fb0fb1d93f42e91e38232185aef6ba3f59396548473` |
| `HANDOFF/audit/crit_iron_core.jsonl` | `blocked-by-metadata` | `gate-fail` | `58c0cc3e52a3a1af258b9195ccf752025c9f4949d78e065a9812d4e0eece9de4` |
| `HANDOFF/audit/crit_mobile_bridge.jsonl` | `blocked-by-metadata` | `gate-fail` | `525c85bc2e72a74dfc8887d31d03601ec45fb84b4585da63623f96730d8e2a16` |
| `HANDOFF/audit/doctrine_violation_inventory.md` | `blocked-by-metadata` | `gate-fail` | `0783d8c05d19b0b886aa0eddd22d7bbf3433fdd8d1021bef6f16626e46021f29` |
| `HANDOFF/audit/eprintln_to_tracing_inventory.md` | `blocked-by-metadata` | `gate-fail` | `0cb0fd9a8c21f4cd2843dac3d277352784ce79a9ab4d7bd3884dd3715f3de75c` |
| `HANDOFF/audit/loopback_dial_design.md` | `blocked-by-metadata` | `gate-fail` | `e3f2396d1d31648c58d1710bfe21d9646263cee76b54c0c389d8756ae7177692` |
| `HANDOFF/audit/ondatareceived_deadlock.md` | `blocked-by-metadata` | `gate-fail` | `67957026a31a29341e3167ed11b4ca6f2631e9093d9ba3316460110504deda7b` |
| `HANDOFF/audit/parity_evidence_audit.md` | `blocked-by-metadata` | `gate-fail` | `85b1cafee458c1a41d506ff5ec8f3be4f4959b063dfe19f6d6f5d3e4bab2ae8c` |
| `HANDOFF/audit/redaction_scan.md` | `blocked-by-metadata` | `gate-fail` | `ca530edaaa384e440cb948ad4dcd046819f2f15adf33a7708b6fa443e0730613` |
| `HANDOFF/audit/security_review_pr129.md` | `blocked-by-metadata` | `gate-fail` | `3207d9777788f52073beb4d4c22e7fa489c10979b8f18e8fbbfaed453c297d56` |
| `HANDOFF/audit/security_review_pr132.md` | `blocked-by-metadata` | `gate-fail` | `73f30d3b6948a911ffd5dd4380ef9b264f80c9e951d4a4e1d2b34e37156c7d41` |
| `HANDOFF/audit/windows_node_driver_log.md` | `blocked-by-metadata` | `gate-fail` | `bb8aa7b5a3d7285cecc5af290adcd04a93cb67c1398367b13632f6fc19941c3a` |
| `HANDOFF/backlog/AGENT_GUIDANCE_Philosophy_Enforcement.md` | `blocked-by-metadata` | `gate-fail` | `43963c5c30a82b6d862c04db18a6bdc02087e3b763142e667ae2e687b63c076b` |
| `HANDOFF/backlog/ANDROID_PIXEL_6A_AUDIT_2026-04-17.md` | `blocked-by-metadata` | `gate-fail` | `d2f4abc444e6280c4d633dc17a34e7cc2d4ef9fda1d4dd636db8b365a1f888f1` |
| `SCM-Q-4390fe189337` (path hash `4390fe189337bc1bbc3a26b69621c5c45030b87bb9772629f6f9fa4b4382d780`) | `quarantined` | `foreign-alias-in-path-and-bytes` | `baa4dde5c7946041718427070d2734ce54fd2ada322435a40fcae719ed828f04` |
| `HANDOFF/backlog/ORCHESTRATOR_001_Full_Ecosystem_Integration.md` | `blocked-by-metadata` | `gate-fail` | `f60883ba38bfe5a1fd5fd5bbc5996111e7d9b142cd06f9acaac7170a48f7ed7f` |
| `HANDOFF/backlog/P0_AUDIT_001_Retroactive_Task_Verification.md` | `blocked-by-metadata` | `gate-fail` | `2a6a0e75bf14c0b3b16f5ed3e52b9ff5e1bbca0ce6e5dfb570def03cb60ab256` |
| `HANDOFF/backlog/P0_BUILD_002_Integration_Test_Repair.md` | `blocked-by-metadata` | `gate-fail` | `2c2285139a447047fb0e06c3c615dcc59d473ef298a6b14496b5e4b4fdef6f22` |
| `HANDOFF/backlog/P1_ANDROID_021_Real_Device_Release_Smoke_Test.md` | `blocked-by-metadata` | `gate-fail` | `d1d71fc6f7192057126842b5aa72bb4b2098e0a0543eb959e23be16f87c49222` |
| `HANDOFF/backlog/P1_CORE_003_Mycorrhizal_Routing_Activation.md` | `blocked-by-metadata` | `gate-fail` | `febf4b597f174a2ca042271c1bea568ca3dd777e2d0af832a3f23b84efb8571b` |
| `HANDOFF/backlog/P1_IOS_002_NOTIFICATION_VERIFICATION.md` | `blocked-by-metadata` | `gate-fail` | `ed20cbc0487d80ece42ae4aa854d8ac9f7922484847d8d18ea8861922e4b16da` |
| `HANDOFF/backlog/P1_WASM_002_NOTIFICATION_VERIFICATION.md` | `blocked-by-metadata` | `gate-fail` | `44cdd061cf09c76c5fa04d1248eee60e61b5797ac8978bba3c1a80f32474f46f` |
| `HANDOFF/backlog/P2_ANDROID_FCM_001_Push_Notifications.md` | `blocked-by-metadata` | `gate-fail` | `330accb847ce03a1c7137c0062e2d53fd5f4caa24a6df2da05209b959a886147` |
| `HANDOFF/discovery/REPO_MAP.jsonl` | `blocked-by-metadata` | `gate-fail` | `b8d1645ed9d31be968faaee3636dd7a55780b21959be17351bfa5cd2c465e26f` |
| `HANDOFF/docs/SCHEMA_VERSIONING_MAP.md` | `blocked-by-metadata` | `gate-fail` | `03c9b74ea207baa91860537b2b93d716c634958e7e5202c3ec477dc11d647ea4` |
| `HANDOFF/done/2026-06-10_5_GATE_BUILD_VALIDATION_integration_branch.md` | `blocked-by-metadata` | `gate-fail` | `e4d478caaae61fc5f98895bec59de350f6a994f4ce507d70c30271c9728ff76c` |
| `HANDOFF/done/2026-06-10_PIXEL_6A_INSTALL_v0.3.4.md` | `blocked-by-metadata` | `gate-fail` | `164a8b5776e3d477be6ef383ffd6ed729190ca198c3b4a165fc231ba0aee5cfe` |
| `HANDOFF/done/2026-07-02_WINDOWS_AUTO_DISCOVERY_VERIFICATION_AND_NEARBY_DISMISS.md` | `blocked-by-metadata` | `gate-fail` | `d08421721a2f805d3b6edeed98610391696220b26fe3b11588b22d8e05c8a76e` |
| `HANDOFF/done/A-01_A3_Android_retry_suppression.md` | `blocked-by-metadata` | `gate-fail` | `6dd39994d88fe7a2e4d887d64ab05ad3b22f99dad43990bb9557593af2e5d7c7` |
| `HANDOFF/done/A-02_F1_confirmation_run_cargo_tes.md` | `blocked-by-metadata` | `gate-fail` | `cdd62b9c48d6964ded843c9eff69278ff6a6f7b41d195a364bf026333eda137d` |
| `HANDOFF/done/A-04_ANDROID_RECEIPT_UNIFICATION.md` | `blocked-by-metadata` | `gate-fail` | `184b6a7f789bb3e9aa8935f47e2b3f9f18536604b3822375df62ee9564be6bcf` |
| `HANDOFF/done/A09_SECURITY_DESIGN_AND_IMPL.md` | `blocked-by-metadata` | `gate-fail` | `8276208cb18a4042d2c58e9c2b2cf2e1719106d480269697ceaf7cba54f17e42` |
| `HANDOFF/done/ADD_TIER_SELECTION.md` | `blocked-by-metadata` | `gate-fail` | `48bca1b395f90edac3154ed61d581849efe16d893e82436a86d84cc27c2ab88b` |
| `HANDOFF/done/AND-CONTACTS-WIPE-001_P0_Android_Contacts_Recovery.md` | `blocked-by-metadata` | `gate-fail` | `e8deb13d3363b06601c7e2a5ba134e9d1f8e4fe64bc832cc8362a64739470ec5` |
| `HANDOFF/done/AND-SEND-BTN-001_P0_Send_Button_Fix.md` | `blocked-by-metadata` | `gate-fail` | `2b75b58388f6b26edcc1af7445865e3b1f9da6e735003b1307d4cc819270750d` |
| `HANDOFF/done/ANDROID_ADD_CONTACT_BUTTON_HIDDEN_AND_SAVE_FAILS.md` | `blocked-by-metadata` | `gate-fail` | `41caa680b791c1ac394c0ed70a44edf0483b7bbd6091e00c32912cef969bae6e` |
| `HANDOFF/done/ANDROID_ANR_RCA_2026-04-23.md` | `blocked-by-metadata` | `gate-fail` | `0e513eb0914941f656f26d64e25842cdf1e4af3a29aee607da8b5fb1cad29f94` |
| `HANDOFF/done/ANDROID_OVERHAUL_STATUS_2026-04-23.md` | `blocked-by-metadata` | `gate-fail` | `f37a5a628880ce01f57bb13a5b802d9b1d1020101c9aaafdb5f05cbc80bae168` |
| `HANDOFF/done/ANDROID_SWEEP_01_hardcoded_strings_contacts_settings.md` | `blocked-by-metadata` | `gate-fail` | `3b97e165f84f4b4b0cf5a09ab9e580ff6581a181a04156f6af07e491daaee5c2` |
| `HANDOFF/done/ANDROID_SWEEP_02_NEEDS_PLANNING_smart_transport_router_unused_params.md` | `blocked-by-metadata` | `gate-fail` | `6923f68e38259a192a8a08a0382470a9e89b18728393d1db7711645db4a84a92` |
| `HANDOFF/done/AUDIT_ANDROID_WINDOWS_INTEROP_PARITY_2026-05-20.md` | `blocked-by-metadata` | `gate-fail` | `047f30d69e289f36d4737d63acf5749b9893d7cffe5edfbc461554bcc3712d52` |
| `HANDOFF/done/B-01_SUITE_NEGOTIATION_VERIFY.md` | `blocked-by-metadata` | `gate-fail` | `d21bbf541ae4c9bf3b2e6764dde595b5ef351a56ed1d8dbb2af85aa0784f60fb` |
| `HANDOFF/done/BATCH_ANDROID_CONTACT_PERSISTENCE_RESIDUAL.md` | `blocked-by-metadata` | `gate-fail` | `eb91223e58138914b842bc7618f46fe2d0749cd63f10635f7f103f5a75f30869` |
| `HANDOFF/done/BATCH_ANDROID_GROUP_KOTLIN_WIRING.md` | `blocked-by-metadata` | `gate-fail` | `9713b264c2ec3b84e88f2a108616caa6af618c5df17c133adcc857963ebb40ae` |
| `HANDOFF/done/BATCH_ANDROID_WIRING_1.md` | `blocked-by-metadata` | `gate-fail` | `e6048da890e9cfbf4ae023fb065e8945645000a767e1fd67a685f9496db6f5a6` |
| `HANDOFF/done/BATCH_ANDROID_WIRING_2.md` | `blocked-by-metadata` | `gate-fail` | `02a6f800c108eba1ffb2fe03572f09bd57c578e9626ffdc0a1175f03c0a86bf5` |
| `HANDOFF/done/BATCH_ANDROID_WIRING_5.md` | `blocked-by-metadata` | `gate-fail` | `fc3b8e7e09f8be358ca59878c43501a1830dfea3cebd76839e1614b517b6e051` |
| `HANDOFF/done/BATCH_ANDROID_WIRING_6.md` | `blocked-by-metadata` | `gate-fail` | `eaaff0b661d3f46d19e649a8c2dfeb785a699386ae2efdc4b9de0106f371f81e` |
| `HANDOFF/done/BATCH_ANDROID_WIRING_B3B5.md` | `blocked-by-metadata` | `gate-fail` | `d67ee3381cc2ae3951d4efb3b53fc36898a7e6eaffe08d15bd4c0e523af49642` |
| `HANDOFF/done/BATCH_ANDROID_WIRING_D4.md` | `blocked-by-metadata` | `gate-fail` | `7fa13f7f53f06baff09fe21a6c4b2b546bc8adf9dade16b35d765d6cfefb9cb1` |
| `HANDOFF/done/BATCH_ANDROID_WIRING_P1.md` | `blocked-by-metadata` | `gate-fail` | `024211ba8b84221bce46ee72fb98da81c4db472b3b511213aaacb60408d76049` |
| `HANDOFF/done/BATCH_ANDROID_WIRING_P2.md` | `blocked-by-metadata` | `gate-fail` | `6da3707d70d8ace6ff3dd00fd18847b1c8fa2cabcd1ec957e94e11a616705e8d` |
| `HANDOFF/done/BATCH_ANDROID_WS14_3_NOTIFICATION_PARITY.md` | `blocked-by-metadata` | `gate-fail` | `6f35c9446f98b14c4b3d44b18c22a8b47b222a68eeb794da27b7a22359ec6ea0` |
| `HANDOFF/done/BATCH_CLI_WASM.md` | `blocked-by-metadata` | `gate-fail` | `895c54b50ef2843fbd5b98d702462813fcbed1151de3445a2fa5885da4fa0ac3` |
| `HANDOFF/done/BATCH_CORE_CROSS_B1B2B8.md` | `blocked-by-metadata` | `gate-fail` | `dc780e44a0a72305696cca4cd2ff155e6bce13daf0e6a392277ffcaad633811d` |
| `SCM-Q-0ec273587153` (path hash `0ec2735871531dfd452ee357072f0a92a8229196f3ab77bd3f62313c6816404c`) | `quarantined` | `foreign-alias-in-bytes` | `9c27ba437d4f8e9b5f1086c6ed9cd5fcd44e30ffc14dfba3a63148e265da9fcc` |
| `HANDOFF/done/BATCH_CORE_RUST_WIRING_C4_SUB01.md` | `blocked-by-metadata` | `gate-fail` | `f9cf2134dbcc185215f71da8cc22d940ec69b6a30f6d9ac6fe2c1622d5a9624d` |
| `HANDOFF/done/BATCH_CORE_RUST_WIRING_C4_SUB02.md` | `blocked-by-metadata` | `gate-fail` | `9cceb2aa710514e20f2f944fec905ea7d1ec10dd155c6a5bb22c4d864c0aa888` |
| `SCM-Q-788978c37360` (path hash `788978c37360ab4f8243b2a4db6b6a48c4a7055802c842a25cd5bfef8da4fddf`) | `quarantined` | `foreign-alias-in-bytes` | `31664bc9ee4e39bb5df40eb8afabb0044504caaf12dddc1f91f637fd79014a58` |
| `SCM-Q-c37a61ab6333` (path hash `c37a61ab633352bb6fa33afd9ffcc7bcd7511452cba664d482488afabb0e2846`) | `quarantined` | `foreign-alias-in-bytes` | `0f6c4e84f9f3d8481437ff0fa6564da9fa43ec12c2103ded02916efd4984c5d7` |
| `HANDOFF/done/BATCH_CORE_RUST_WIRING_C4_SUB05.md` | `blocked-by-metadata` | `gate-fail` | `3939df8a330d8cdf2894e4aa9b8f08bd11bce115acc3fe99bbdb05ae2b814d84` |
| `HANDOFF/done/BATCH_CORE_RUST_WIRING_C4_SUB06.md` | `blocked-by-metadata` | `gate-fail` | `e6c67594f223c569d2f4f8f6a1cd53379096d735acfe7a4de28866c990345e39` |
| `HANDOFF/done/BATCH_CORE_RUST_WIRING_C4_SUB07.md` | `blocked-by-metadata` | `gate-fail` | `1e923d2927fbea652b23b211f4ff64556f93d718ab4c223882ccd3168d9c825d` |
| `HANDOFF/done/BATCH_CORE_WASM_WIRING_3.md` | `blocked-by-metadata` | `gate-fail` | `4ec9ad3730e5f035ddaf8d0a575eb80dd240ff9a0077578ac93e11dda130eb76` |
| `HANDOFF/done/BATCH_CORE_WASM_WIRING_4.md` | `blocked-by-metadata` | `gate-fail` | `b5d0027f4d90acbf06fa721b7c0740ed13a9e400e79e5d905d1efa8e9eec8685` |
| `HANDOFF/done/BATCH_CORE_WASM_WIRING_5.md` | `blocked-by-metadata` | `gate-fail` | `592dcdc96218e2da46abd775d5141ca3759b701e2aeb9824f25f24ef00f6ddad` |
| `HANDOFF/done/BATCH_CORE_WIRING_B1B2.md` | `blocked-by-metadata` | `gate-fail` | `ac77f47bd6fe5d6c8383b5e53e7f5a4a60cb5ffbb133353ffe0cfc7ad39c7dac` |
| `HANDOFF/done/BATCH_RUST_GROUPA_RESUME_PREFETCH.md` | `blocked-by-metadata` | `gate-fail` | `8492fbe29f202070c11bb6534feb50385f050906bae283cd3d1ed6e80b950d00` |
| `HANDOFF/done/BATCH_RUST_GROUPC_NOTIFICATION_RELAY.md` | `blocked-by-metadata` | `gate-fail` | `ac4c3b4007a3724bc635555c4d66b555454a67b6d815d4d48be7a4d93a117355` |
| `HANDOFF/done/BATCH_RUST_GROUPG_MISC_CORE.md` | `blocked-by-metadata` | `gate-fail` | `cfc6b980049d17625ce43ed42eb39dc88c79945261ac63191925c377ce95ab06` |
| `HANDOFF/done/BATCH_S1_T1_FIX_ANDROID_BUILD.md` | `blocked-by-metadata` | `gate-fail` | `9abcfba41da684773741340f015bfa490d09bdd404fbe7bd6ac488548dbfcb3e` |
| `HANDOFF/done/BATCH_S1_T2_UNIFFI_BINDINGS.md` | `blocked-by-metadata` | `gate-fail` | `f2ddc9356f2dd9711e618096ec0620be9c42c17a1e3393a29c7cb638ba1551c8` |
| `HANDOFF/done/BATCH_S1_T3_CORE_INTEGRATION_AUDIT.md` | `blocked-by-metadata` | `gate-fail` | `b6dc3ba2c812b6d709844708bd3a48fce58e08cb7ebf33e774811a5bc6e83974` |
| `HANDOFF/done/BATCH_S1_T4_CI_PIPELINE.md` | `blocked-by-metadata` | `gate-fail` | `965b86360b108c8a212dfaa7369a84768d02acf45164ee0808e39c1abc27eb17` |
| `HANDOFF/done/BATCH_S2_T1_SWARM_BRIDGE.md` | `blocked-by-metadata` | `gate-fail` | `de3c3e7f620fca03ec3276ca1b1ee8c0f5e3b1d787a26f674eb3117a1e6c3149` |
| `HANDOFF/done/BATCH_S2_T2_TOPIC_MANAGER.md` | `blocked-by-metadata` | `gate-fail` | `0c2be1eeea6976b5c389ed9593ad8b4c20e6a1cf1f32bd1ca8eff77c1351d6a1` |
| `HANDOFF/done/BATCH_S2_T3_MESSAGE_DEDUP.md` | `blocked-by-metadata` | `gate-fail` | `99279d897b1a23b07fc1bf093656a406feb88d1636918ee32ecd3fa33b219f59` |
| `HANDOFF/done/BATCH_S2_T4_RELAY_BOOTSTRAP.md` | `blocked-by-metadata` | `gate-fail` | `6d6591f536d6b835e2bb80782e9080490b3ee7ef94e333cf40995b87b601506f` |
| `HANDOFF/done/BATCH_S2_T5_VPN_SERVICE.md` | `blocked-by-metadata` | `gate-fail` | `33e168e295289c1643654473bbacf273d203781667a73bb638331c782cd2ba8f` |
| `HANDOFF/done/BATCH_S3_T1_BLE_CORE_FORWARDING.md` | `blocked-by-metadata` | `gate-fail` | `9f9964267f266fd2b78c9d84cdedb4745ffc4323a5eb2d4c8e8980edbac7e30a` |
| `HANDOFF/done/BATCH_S3_T2_BLE_IDENTITY_HANDSHAKE.md` | `blocked-by-metadata` | `gate-fail` | `e95a07fce2ca314567417b5716470b91f1f452497fd69ab955d94594aeaffdde` |
| `HANDOFF/done/BATCH_S3_T3_BLE_QUOTA_AUTOADJUST.md` | `blocked-by-metadata` | `gate-fail` | `ca873e086988d0248e77021048b84a6a35dd3ebfb6c9cbe3f6ad097fbdbfc3c2` |
| `HANDOFF/done/BATCH_S3_T4_BLE_DEGRADATION.md` | `blocked-by-metadata` | `gate-fail` | `f61712ab0dfba737824e868e0437709733e7eb041150a7249355ca3d3b5885b0` |
| `HANDOFF/done/BATCH_S4_T1_ANR_ELIMINATION.md` | `blocked-by-metadata` | `gate-fail` | `020c86587a1f43d00b9a86d4355120c4cd6daa7f618f3c1a1fab010f0db89c91` |
| `HANDOFF/done/BATCH_S4_T2_NOTIFICATION_RELIABILITY.md` | `blocked-by-metadata` | `gate-fail` | `b3e3c75a863c332d1142503d3c2cb5361ac768961b787e0d8085db45970c56d5` |
| `HANDOFF/done/BATCH_S4_T3_DATA_PERSISTENCE.md` | `blocked-by-metadata` | `gate-fail` | `502cb52457c590fe352809b2c4f9bfd8fad3fad3576fddddb0b60840a3fb6998` |
| `HANDOFF/done/BATCH_S4_T4_IDENTITY_CACHE.md` | `blocked-by-metadata` | `gate-fail` | `b5f660bbda4cf7d9563f6d6bfa2f4862b6fc776018edb61fe93bfb5cedf70f5b` |
| `HANDOFF/done/BATCH_S5_T1_PRIVACY_COMPLIANCE.md` | `blocked-by-metadata` | `gate-fail` | `f17b601b6ff02caeeebebbaea07d323246eb8c99ccff0eb44268045b820bb835` |
| `HANDOFF/done/BATCH_S5_T2_CRASH_REPORTING.md` | `blocked-by-metadata` | `gate-fail` | `4d498c0fcd252d6226f2048ad4e898fad04971e173ee6f6eff439fae115d6550` |
| `HANDOFF/done/BATCH_S5_T3_ALPHA_BRANDING.md` | `blocked-by-metadata` | `gate-fail` | `1e87432f83339b43c2728f3fbefd11f94878692a82085f264d20c19d5bf6b5b9` |
| `HANDOFF/done/BATCH_S6_T1_IDENTITY_BACKUP.md` | `blocked-by-metadata` | `gate-fail` | `3e32ebd359c6b3a3b1295e7aac500356082c21a71c28e385757dc0ccf86ad870` |
| `HANDOFF/done/BATCH_S6_T2_CONTACT_QR_SHARING.md` | `blocked-by-metadata` | `gate-fail` | `308a2a87ae378c04ebbd576c2aeb51c0f066e3e23e95de655e8c73130739612d` |
| `HANDOFF/done/BATCH_S6_T3_DEEP_LINK_INVITE.md` | `blocked-by-metadata` | `gate-fail` | `a8dd8354de58cc762aa1f256e2e92d5fad8544beabd146afc3f82df8b8a8f8ec` |
| `HANDOFF/done/BATCH_S6_T4_FINAL_INTEGRATION_TEST.md` | `blocked-by-metadata` | `gate-fail` | `8ac8f8cdd215e4769402db7cbab6acfbb08a9b7ae9ba62cd7320beba886e0e88` |
| `HANDOFF/done/BATCH_SECURITY_CARGO_AUDIT_RESIDUAL.md` | `blocked-by-metadata` | `gate-fail` | `e5f3c821400b18a340ac39760f6dee6ce0cb7da00657d0321ea6a0e05425e6a9` |
| `HANDOFF/done/BATCH_WASM_B6.md` | `blocked-by-metadata` | `gate-fail` | `87420a6e6637da5fe47035b42b90d2ea6cd8a5de2b1962ed1f5ed1c72cba517f` |
| `HANDOFF/done/BATCH_WS13_6_TIGHT_PAIR_MIGRATION.md` | `blocked-by-metadata` | `gate-fail` | `e9b972a538b9565680cb2e50254f665da623b593d76785e98343b74f866cf2a7` |
| `HANDOFF/done/CI_RED_ON_MAIN_ALL_FEATURES.md` | `blocked-by-metadata` | `gate-fail` | `b8fc223c570645e3c0ade14a49dbb37f771430fdf76ee7e636c2dba8c8c7c7df` |
| `HANDOFF/done/CLEANUP_STALE_BATCH_P1_CORE_MYCO_ROUTING.md` | `blocked-by-metadata` | `gate-fail` | `6b656e6497030cb0131d8a08f8e622f47241cdafc5114614473c58e4125c2540` |
| `HANDOFF/done/CLIPPY_DEBT_cli_desktop_bridge_dwarnings.md` | `blocked-by-metadata` | `gate-fail` | `372c68b11a2e871fa4de03f97f665c0e9d0e767375a89b32f90bea7e360dbe2a` |
| `HANDOFF/done/CONTACT_LOOKUP_PUBKEY_FLAVOR_MISS_2026-08-09.md` | `blocked-by-metadata` | `gate-fail` | `9113ffa50b65bb9782e88b490f44d9928b510f04ed410fe18f42d10873eeea5a` |
| `HANDOFF/done/CORE_BLOCK_GATE_HARDENING.md` | `blocked-by-metadata` | `gate-fail` | `0ebedee5cd883fb5eed402c113208afa9584012f37b79ef3996fc39341f7dd78` |
| `HANDOFF/done/CORE_DAEMON_TEST_2026-04-22.md` | `blocked-by-metadata` | `gate-fail` | `caa26f87357a67c1eddfefbecaf959aa2ec1610d4413820942b89b4e513f4ac2` |
| `HANDOFF/done/CORE_DAEMON_TEST_20260423_1905.md` | `blocked-by-metadata` | `gate-fail` | `5ac2721a17ba3b898c56eace789ae34fe47f1b9650d1656f65d5c5ff33c5f47d` |
| `HANDOFF/done/CORE_INTEGRATION_BACKUP_ENVELOPE_TYPE.md` | `blocked-by-metadata` | `gate-fail` | `dab180b634c96b6cb664bc1721972e6d8d5afffd4ff74fecd7df76fbf3d432fd` |
| `HANDOFF/done/CORE_LIB_TEST_COMPILE_BREAK.md` | `blocked-by-metadata` | `gate-fail` | `dd1ebec833f4ff86e354ebaeb213186dffd95ecae48838d0b93e4b488fbd7cf6` |
| `HANDOFF/done/CORE_LIB_TEST_FINAL_2_FAILURES.md` | `blocked-by-metadata` | `gate-fail` | `a7b4054cc3f8db6bae8f5c48a9a6e8ae5cd611c5e32076502457d461f2bdf321` |
| `SCM-Q-2c13cb842333` (path hash `2c13cb84233308da8ce427ec2ea9719092f053b6304d4a56a8374e3fb3ee7c3a`) | `quarantined` | `foreign-alias-in-bytes` | `1ff4f7a888485d6a8f05e468983ddd7448696fc7eb9f409fb1853aa63dfcb0dc` |
| `HANDOFF/done/CORE_SWEEP_03_ble_gatt_traits_never_implemented.md` | `blocked-by-metadata` | `gate-fail` | `0f74e76945f0e3195d508285e5664b475a9409e1a5fd53fd73eee5e03c5d6ac4` |
| `HANDOFF/done/CORE_SWEEP_04_systemic_unix_epoch_expect_pattern.md` | `blocked-by-metadata` | `gate-fail` | `51c8b05ccacfef8cad6dab96b99a8ac46108ffb8bb509612897dcaaf739da429` |
| `HANDOFF/done/CRITICAL_ANDROID_FALSE_DELIVERY_FAILURE_NO_RECEIPT_ACK.md` | `blocked-by-metadata` | `gate-fail` | `bcdac5fab7333de4f265c96763ef8258a8a32d4b2e13fc5b913718c389a53335` |
| `HANDOFF/done/CRITICAL_OUTBOX_NEVER_FLUSHES_DESPITE_ACTIVE_CONNECTION.md` | `blocked-by-metadata` | `gate-fail` | `5d7a8fd0d4210a7e6970e952f929234d551b240228669c5ef1bfa2071a5b7663` |
| `HANDOFF/done/CRITICAL_RATCHET_SUBSYSTEM_NOT_WIRED_INTO_IRONCORE.md` | `blocked-by-metadata` | `gate-fail` | `3664c8cbfbb02999b952f8207ca58da87162e32a6974b2a2eddf62bdea2b2f38` |
| `SCM-Q-e4094201136f` (path hash `e4094201136fbe9fa349044cc674fdede1e832885f928ebaeda2f8fd03d5fe28`) | `quarantined` | `foreign-alias-in-bytes` | `8ffa162942adb7e24033bc90fe592cebabefe15244ca29446fd044892d24d7c1` |
| `HANDOFF/done/D-02_ANDROID_ROBOLECTRIC_WIRING.md` | `blocked-by-metadata` | `gate-fail` | `b9d56b75a8c84a206c868d525b6ecce8e75db0597fa2c7552123fa23051091af` |
| `HANDOFF/done/D-05_UNWRAP_PANIC_HARDENING.md` | `blocked-by-metadata` | `gate-fail` | `44388d8caf4e2d642126e6dac04a1201f1d613e9813758dbb3036f4c8b1a2108` |
| `SCM-Q-a0286fa34103` (path hash `a0286fa3410379911982f27648626d1d71ac3d930c808a27e7d786d2c60ae68c`) | `quarantined` | `foreign-alias-in-bytes` | `955530663c211786ee2fe1f8bd9c3e813afad82e38e2cf56aea67f2362852594` |
| `HANDOFF/done/DISPATCH_LAKE_OPENROUTER_DIRECT.md` | `blocked-by-metadata` | `gate-fail` | `cc9150277759a96b2cc8938062a24e3d3cb9336a19d55c7d658b16a707dc9132` |
| `HANDOFF/done/ESC_ANDROID_DNS_RESOLVER_FIX.md` | `blocked-by-metadata` | `gate-fail` | `44972125d4f8fbd62f6b1c8885caf39d83cb2b8c513f1857e4333f5f24b7c2f9` |
| `HANDOFF/done/F1_LEDGER_CONVERGENCE_FIX.md` | `blocked-by-metadata` | `gate-fail` | `d9a2217902535e8acc62bd49a15e2edc50af4017659693410892700376b8d955` |
| `HANDOFF/done/F2_MESHSTORE_PERSISTENCE.md` | `blocked-by-metadata` | `gate-fail` | `fc3511b6076b2945bbbe943ddea3b422ae46efbdd19103587f9936600bc3a9e0` |
| `HANDOFF/done/FABLE5_FOLLOWUP_F4_F7_F8.md` | `blocked-by-metadata` | `gate-fail` | `2d4681fc6d2af3c6ca0ebaca0c94b1deab68efada80a71e0fa5b6d98b36abac7` |
| `HANDOFF/done/FABLE_5_COMPREHENSIVE_AUDIT.md` | `blocked-by-metadata` | `gate-fail` | `dba0b6e74aefffd35be05d5ea67487f08d8e3210801e6165b750e3840bc19298` |
| `SCM-Q-577e2622fef8` (path hash `577e2622fef84cbc46fcd0de25663452cf5fa94b60844af0e1b4fd2f785d8198`) | `quarantined` | `foreign-alias-in-bytes` | `af1586a32d8e95d8d90722293d35fcab15ef02a171540716f1e271b2c831aba5` |
| `HANDOFF/done/FARM_WS_F1_FIX.md` | `blocked-by-metadata` | `gate-fail` | `b75f904d1a9005c9b5cd6407474fdb1c43730f8f27b13a9d0efc23422ee5b3bb` |
| `HANDOFF/done/FINAL_WIRING_AUDIT.md` | `blocked-by-metadata` | `gate-fail` | `ef1fef2dc1f999f5f4a291ac57d2df6d0821cb37deb140bcc32f4146db061f62` |
| `HANDOFF/done/FINAL_WIRING_AUDIT_V2.md` | `blocked-by-metadata` | `gate-fail` | `739728021fcff892500a3a60000bb2fe87df8978756a006f47228e1d39bb76e7` |
| `HANDOFF/done/FIX_DELEGATE_PARSER.md` | `blocked-by-metadata` | `gate-fail` | `d6f88c471a23b7d6e928941f135b253b8eea2409eaf53d55b4f32bf832a72135` |
| `HANDOFF/done/FIX_DOCKER_BOOTSTRAP_CONFIG.md` | `blocked-by-metadata` | `gate-fail` | `9203dc10d10593e75ae6d6e22fc6b5a6a2ac63925064a6d91c202d90ffe3aa50` |
| `HANDOFF/done/FIX_UNIFFI_HISTORY.md` | `blocked-by-metadata` | `gate-fail` | `7995268c8617685f40d3af3c1e80703af90f2b8bb9426118842150b7748ce437` |
| `HANDOFF/done/FOR_BETA_AUDIT_B2_MANIFEST_REANCHOR.md` | `blocked-by-metadata` | `gate-fail` | `4a4181d7ebf5314696cdc74a5f0796b4e3de27a8607ce56a34fe7e252923779f` |
| `SCM-Q-635fc271cbb5` (path hash `635fc271cbb5b3703fc0d05a621291b7a99bed0751631e1c98390f545927b8b7`) | `quarantined` | `foreign-alias-in-bytes` | `10d28134090b9fe8cec666e7c3d16a6e2ff298c1add62d4aa8d41210b71cbcd7` |
| `HANDOFF/done/HEAVY_LIFT_LIBP2P_UPGRADE_001.md` | `blocked-by-metadata` | `gate-fail` | `48d4b1cd6e1f2a3732bfdb2ff86f29a46374bd965a249c5a51591005f90c2e26` |
| `HANDOFF/done/INVESTIGATE_ANDROID_DIAL_NETWORKERROR.md` | `blocked-by-metadata` | `gate-fail` | `2d8eff4c15799a4b9a89779e4511c6a98aa1fe8c4724cb54db6aba9c568be2f2` |
| `HANDOFF/done/INVESTIGATE_IDENTITY_OPTIONAL_RELAY_MODE.md` | `blocked-by-metadata` | `gate-fail` | `25e159a4d6754a6fd821c3e134d36c32702ab95cbb0348c40e934dd291ceae91` |
| `HANDOFF/done/INVESTIGATE_OUTBOX_RETRY_MECHANISM.md` | `blocked-by-metadata` | `gate-fail` | `be205d03ee47b3cca96a7fc1542270c42df9788bb1b7d2e945aa27c535660265` |
| `HANDOFF/done/IN_PROGRESS_P0_ANDROID_024_IDENTITY_GENERATION_REGRESSION.md` | `blocked-by-metadata` | `gate-fail` | `852f503406796519344baebfd550a84e432c20faf236ce6f57b9d89e4f522553` |
| `HANDOFF/done/IN_PROGRESS_P0_ANDROID_IDENTITY_GENERATION_ARGON2_OPT_LEVEL_FIX.md` | `blocked-by-metadata` | `gate-fail` | `0149ec0a95d301cf5c020bf08ff20593ef102d4e87ff1910dc9c866eb3b25090` |
| `SCM-Q-1e0dff48b516` (path hash `1e0dff48b5164901c9dde82c6faa6dc8660d9add302ca8953bbc71873980cf66`) | `quarantined` | `foreign-alias-in-bytes` | `f0b5b0ed8b6e2783c0947ce52d51f38174120655d1feb5dc7fe1ea468bcb4b33` |
| `HANDOFF/done/IN_PROGRESS_claude_slot2_status.md` | `blocked-by-metadata` | `gate-fail` | `0579f3a8265f28aee5c7d86c70fe718e59edba24dd1a03e9a5b4f56135e7d3f6` |
| `SCM-Q-6fc2aa803d82` (path hash `6fc2aa803d828416c93483426943ab3fd92e63b93456d6a3082ab2a983aa79be`) | `quarantined` | `foreign-alias-in-bytes` | `33fb9bbf56a592dcae118ede8a6421dbe4cd212fe46c662de040c342fa00233a` |
| `HANDOFF/done/IN_PROGRESS_task_agy_android_stability_complete_handoff_2026-06-07.md` | `blocked-by-metadata` | `gate-fail` | `15929e316e69dabebd308ce68bf4631fb7deac111a69c8c5e2c507fb76e33606` |
| `HANDOFF/done/IN_PROGRESS_task_ironcore_full.md` | `blocked-by-metadata` | `gate-fail` | `d7a0d2e3102c2be400714f00660a436ba35c9f4e63cab12abceafb344dad1a57` |
| `HANDOFF/done/IN_PROGRESS_task_swarm_fixes.md` | `blocked-by-metadata` | `gate-fail` | `cf1e3cb18a5d553dfd9bca4e7e4c16a0cc0ca4bbc0dbb76fb2a41408720299f5` |
| `HANDOFF/done/IN_PROGRESS_task_wire_ConnectionQualityIndicator.md` | `blocked-by-metadata` | `gate-fail` | `ca65c9670d326acfbb62d5e1da3ea967ad07a6adb69a1b231e8306a28bed3a28` |
| `HANDOFF/done/IN_PROGRESS_task_wire_getRetryDelay.md` | `blocked-by-metadata` | `gate-fail` | `4a2309595b55a7186b8bbbfc176b328ff24f0414f7f206bca08e72428850766f` |
| `HANDOFF/done/IN_PROGRESS_task_wire_isStorageStateCritical.md` | `blocked-by-metadata` | `gate-fail` | `3afe61155516e460e42de105ca53fc455a77589ee2b4bbb3130055a1095e3704` |
| `HANDOFF/done/IN_PROGRESS_task_wire_notifyForeground.md` | `blocked-by-metadata` | `gate-fail` | `d12e58a6cca3ed5ad64c4ca99c68780dfd4c2fdf16fd1d7f26c0929a852a46d0` |
| `HANDOFF/done/IN_PROGRESS_task_wire_setRotationInterval.md` | `blocked-by-metadata` | `gate-fail` | `9ac65aeb906e40e73264f66a8fbcfeaceb01bb9cab1fd83bce1856370393cd42` |
| `HANDOFF/done/IN_PROGRESS_v030_build_session.md` | `blocked-by-metadata` | `gate-fail` | `becd587e80b45fdd1189b3a1ad4dfd19fddc2003de9ef0c8ef5d447ae66a8e3d` |
| `HANDOFF/done/IN_PROGRESS_v030_rebuild.md` | `blocked-by-metadata` | `gate-fail` | `9f197c0e613f345628a6cb8a3567cb495d29c63831435a9b36e88b20fdf5e8c2` |
| `HANDOFF/done/IN_PROGRESS_v030_release.md` | `blocked-by-metadata` | `gate-fail` | `3b729e6b09952f76c5c3a5370e60115990a260b252d6ad16d158ec1a1e29bf84` |
| `HANDOFF/done/KMP_UNBLOCK_DESKTOP_BRIDGE.md` | `blocked-by-metadata` | `gate-fail` | `36312d380d2659b3298d9e81037ca732a61581be99599c705a7c98420c35b946` |
| `HANDOFF/done/LAN_TRANSPORT_TEST.md` | `blocked-by-metadata` | `gate-fail` | `fd9b9817dde1ebdf73b9309eafc68697463320de8ec9048353d7107d78b64edd` |
| `HANDOFF/done/MICROBATCH_ANDROID_KOTLIN_WIRING.md` | `blocked-by-metadata` | `gate-fail` | `3357ce74ad1620f5d06f7e5ca6c1b9b4e0c81e2523350a924ea1e0aa2ec5a2c2` |
| `HANDOFF/done/MICROBATCH_CORE_RUST_WIRING_MISC.md` | `blocked-by-metadata` | `gate-fail` | `85fad48ab01eeefb70029660d3188415b0f870a0244e0eef0f653c62c76480a3` |
| `HANDOFF/done/MICRO_RUST_CLIPPY_CLEANUP_001.md` | `blocked-by-metadata` | `gate-fail` | `2d8166083c134ac4052f5cacc1f6e8ff9ccd1ebe4f0a6d1c3a63f03d4dc682be` |
| `HANDOFF/done/MICRO_RUST_CLIPPY_CLEANUP_002.md` | `blocked-by-metadata` | `gate-fail` | `8e0799ca52afb8a37bf8df77a3a445debcc08cefb38d442e6b37a5927edc2fff` |
| `HANDOFF/done/NETWORKERROR_OBSERVABILITY_GAP.md` | `blocked-by-metadata` | `gate-fail` | `4d76d79b7004c62923f7b39bdb6b215b5fda57e94dd6028cdf376271f31ff134` |
| `HANDOFF/done/NETWORKERROR_OBSERVABILITY_GAP_COMPILE_FIX.md` | `blocked-by-metadata` | `gate-fail` | `0f65a08085d9fb137eec1643ec552e43fc2bffd6d1c973351b13905b2a83958f` |
| `HANDOFF/done/NETWORKERROR_PART_A.md` | `blocked-by-metadata` | `gate-fail` | `47a199d1c920117c91f42110815f92169069191077348d5cf5710886405d1e49` |
| `HANDOFF/done/NETWORKERROR_PART_A_FULL.md` | `blocked-by-metadata` | `gate-fail` | `a8908f01fada83c9770dd7eddc1f498df15ebbd4d14b9bbc5c94a93ac818b0e2` |
| `HANDOFF/done/NETWORKERROR_PART_B_FULL.md` | `blocked-by-metadata` | `gate-fail` | `ecf80db8c41a076fdec5eed849e423712618790e9c0cb00d9588041d32b6867d` |
| `HANDOFF/done/NETWORKERROR_PART_C.md` | `blocked-by-metadata` | `gate-fail` | `85a9ecdb90098bd707144e669285b8afda7e01068401a9cec00a423eb9b67fee` |
| `SCM-Q-b7bd7b339f26` (path hash `b7bd7b339f264930476f111ee5334d696f95696d076525a1b312877f871bd95c`) | `quarantined` | `foreign-alias-in-bytes` | `9f707bd917fd15ce7c97bb35776ae3d69483ce1457f58c986b2d6800e2e0a101` |
| `HANDOFF/done/NEXT_ITER_02_Adversarial_Review_Sprint_Diff.md` | `blocked-by-metadata` | `gate-fail` | `c64f9480193362e6627216f3a19691b6fde94d16efadc55bdb4a4a1d680ec85f` |
| `HANDOFF/done/NEXT_ITER_03_Docs_Sync_And_Residual_Debt.md` | `blocked-by-metadata` | `gate-fail` | `04e15087c08d965ff50e66e8c60c51b91fce248f3c6889bdfbb3a1790842528c` |
| `HANDOFF/done/NEXT_ITER_04_Live_Device_Retest_Pairing.md` | `blocked-by-metadata` | `gate-fail` | `c1ba370f40f36756ce84f37a1cb6fa7d7b8193ec90928f2a3c40487c9594c080` |
| `HANDOFF/done/ONION_FFI_RPC_SURFACE_UNGATED.md` | `blocked-by-metadata` | `gate-fail` | `22447ed088e8ffa84c2f3313616b8012f3875d2af48f1c6d961a8d18d0cbab3d` |
| `HANDOFF/done/ONION_GATING_PART_A_FULL.md` | `blocked-by-metadata` | `gate-fail` | `2bfe2795a676e06234512033d2b24ec715497bfc9962e73fab0c92ae32dfb0d7` |
| `HANDOFF/done/ONION_GATING_PART_B_FULL.md` | `blocked-by-metadata` | `gate-fail` | `691eefd9ccbef2625538de07b39aaf156fbbab6e49b583524562cb225420d473` |
| `HANDOFF/done/ONION_GATING_PART_C.md` | `blocked-by-metadata` | `gate-fail` | `7bdc565379342ab8a265e4793cc8935a53c77e084b1a7c5204dfc5bab995ef28` |
| `HANDOFF/done/ONION_GATING_TASK.md` | `blocked-by-metadata` | `gate-fail` | `7d02dc61b416d2760a5898f3cbe01270015fcf19e26418cb8ecd9431ffff0280` |
| `HANDOFF/done/ORCH_E2E_GITIGNORE_LINE_REFS_2026-08-04.md` | `blocked-by-metadata` | `gate-fail` | `22c9730e80313ae2a478c79ee621f491fd893b40c9e04018a8ef8a3b7155dc36` |
| `HANDOFF/done/OUTBOX_FLUSH_ON_CONNECT_RETRY.md` | `blocked-by-metadata` | `gate-fail` | `39f5eb8a72cff49eb36c438d29a9ea70a6420b8077dd12db000d13cc3915c7fc` |
| `HANDOFF/done/P0_ANDROID_001_JAVA_HOME_Configuration.md` | `blocked-by-metadata` | `gate-fail` | `b53ce6df15852290358c9fcecff0dc12f9af737240ada97e777a50b9246c908d` |
| `HANDOFF/done/P0_ANDROID_001_TRANSPORT_IDENTITY_AND_CONTACT_RECOVERY.md` | `blocked-by-metadata` | `gate-fail` | `6748613e8a805e03b522586a3465f6b3147b30cb34c8452d2f4b19055aa7e15a` |
| `HANDOFF/done/P0_ANDROID_002_ANR_Crash_Resolution.md` | `blocked-by-metadata` | `gate-fail` | `918c705eb1648f9da4a6963cc8b8f27e8f2d549255dfab822ed8a170f6eb30a0` |
| `HANDOFF/done/P0_ANDROID_003_BLE_SCAN_STABILIZATION.md` | `blocked-by-metadata` | `gate-fail` | `ec36d7b850260eec4da1e5778cf3aae22c9a01305e3cb5731c7c51b14b687eaa` |
| `HANDOFF/done/P0_ANDROID_003_Build_Fix.md` | `blocked-by-metadata` | `gate-fail` | `a1cf85f871dfc55d34bc6c7dc10990cbbe884099088bc0c4e10c9e1dcd5ead26` |
| `HANDOFF/done/P0_ANDROID_005_MESSAGE_ID_TRACKING_FIX.md` | `blocked-by-metadata` | `gate-fail` | `f1360345cea2e1bea5d3b6a453322fdaf5af70b514801538640d2dd92c3f4ccc` |
| `HANDOFF/done/P0_ANDROID_006_COROUTINE_CANCELLATION_FIX.md` | `blocked-by-metadata` | `gate-fail` | `dc52c47294e57aa5cd828a26274ddd4319e493d3b239ee9344af5c66944c7c3b` |
| `HANDOFF/done/P0_ANDROID_007_NETWORK_DIAGNOSTICS.md` | `blocked-by-metadata` | `gate-fail` | `518b72115454f3090aa9c72c5b7fa6a20c4c2c22f48992e20dc748329552eb50` |
| `HANDOFF/done/P0_ANDROID_008_Kotlin_Compile_Fixes.md` | `blocked-by-metadata` | `gate-fail` | `a826477fb78acb99df0c8a9cd3c63e0152a2fb0804d3c5caf13820442803121a` |
| `HANDOFF/done/P0_ANDROID_009_Pixel_6a_Runtime_Validation.md` | `blocked-by-metadata` | `gate-fail` | `5f341ed62a3f3ede36005feb089d4fc5d7090bb0b9fecfd5336eace1609213f3` |
| `HANDOFF/done/P0_ANDROID_009_Settings_Screen_Regression_Debug.md` | `blocked-by-metadata` | `gate-fail` | `d0a23b087035db3285620b212600a14df5402449005ddeea2349aefd699c0af0` |
| `HANDOFF/done/P0_ANDROID_010_Logcat_Full_RCA.md` | `blocked-by-metadata` | `gate-fail` | `ec760a62d6ab7c0cca757349c9fd224fb6941d66b55406d6fd0594f8a9fec70b` |
| `HANDOFF/done/P0_ANDROID_011_Identity_Regression_Tests.md` | `blocked-by-metadata` | `gate-fail` | `1f668ce1088aabcf79f91d31574739487dcbec07c43bb0c902d7bc1411bbcffa` |
| `HANDOFF/done/P0_ANDROID_012_Pixel6a_Validation.md` | `blocked-by-metadata` | `gate-fail` | `b051a4680279a27fa648bde58e190f0c60b9e4a804869f667a14d801eb3a80b7` |
| `HANDOFF/done/P0_ANDROID_015_Nickname_Regression_RCA.md` | `blocked-by-metadata` | `gate-fail` | `9cff1dfda66ac60596eadc00d9821ba908b8b0234cc1ca2b94983e755debe371` |
| `HANDOFF/done/P0_ANDROID_016_Settings_Screen_42s_Hang.md` | `blocked-by-metadata` | `gate-fail` | `3a4ea0890ca9fbb52f79359f276293e3912790a9c2c357136d241da9821b30f2` |
| `HANDOFF/done/P0_ANDROID_017_Settings_ANR_Debug.md` | `blocked-by-metadata` | `gate-fail` | `f798eebd9e905bac37a9c70afb78753b1b9e8db7f7290d7f18e823ea98ec08e8` |
| `HANDOFF/done/P0_ANDROID_018_Notification_Channel_Init_and_Manifest_Cleanup.md` | `blocked-by-metadata` | `gate-fail` | `9857b95336dd431089bd40b2644c50d5626c1be792c7aa3eea4a19590306c13b` |
| `HANDOFF/done/P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH.md` | `blocked-by-metadata` | `gate-fail` | `e306700dfe9af220c61d11811bb80fb268fed3a2416575b1356b6ae7c6d98df7` |
| `HANDOFF/done/P0_ANDROID_026_Foreground_Service_Android_12_Crash_Fix.md` | `blocked-by-metadata` | `gate-fail` | `5a6a22c37ce24aef06891ac605fefc3b1e936258bcd4da300f7ac7297306fddb` |
| `HANDOFF/done/P0_ANDROID_ANR_BatteryReceiver_Synchronous_FFI_Call.md` | `blocked-by-metadata` | `gate-fail` | `bf58c299178a24a4f31595530ad11b2d95dba287d0831d46e439c8e1d2943370` |
| `HANDOFF/done/P0_ANDROID_IDENTITY_001_ANR_FIXES.md` | `blocked-by-metadata` | `gate-fail` | `de526c746215e34f6998f6677563b2d59cf46fa432da85b9856d7d64aedb820f` |
| `HANDOFF/done/P0_ANDROID_IDENTITY_002_NICKNAME_AND_STATE.md` | `blocked-by-metadata` | `gate-fail` | `2b62b5e212bcf813a024b6aac7a8fe1e5e2ba0a6572ee6be50b61f222d7b4b5a` |
| `HANDOFF/done/P0_ANDROID_IDENTITY_003_PERSISTENCE_HARDENING.md` | `blocked-by-metadata` | `gate-fail` | `347cd75a8597d1ac8974261e72d8167c4156320fe1632c68bddae40e04279412` |
| `HANDOFF/done/P0_ANDROID_PLAYSTORE_001_Compliance_and_Assets.md` | `blocked-by-metadata` | `gate-fail` | `9bf09d1c588bf28c5867d8e7955aad4812628d1696bede1615980160f45a77be` |
| `HANDOFF/done/P0_ANDROID_STABILITY_001_ANR_Battery_Fixes.md` | `blocked-by-metadata` | `gate-fail` | `2c061ab921e1a540172fa898beffa06ce3b507823482ad518c7a8bbb99213254` |
| `HANDOFF/done/P0_ANTI_ABUSE_001_Critical_Controls_Implementation.md` | `blocked-by-metadata` | `gate-fail` | `b2cd87ae8db99636692210752e10a618a031b8d27e6623723a3829017d5fe260` |
| `HANDOFF/done/P0_AUDIT_001_Retroactive_Task_Verification.md` | `blocked-by-metadata` | `gate-fail` | `2a6a0e75bf14c0b3b16f5ed3e52b9ff5e1bbca0ce6e5dfb570def03cb60ab256` |
| `HANDOFF/done/P0_BUILD_001_Core_Integration_Test_Fix.md` | `blocked-by-metadata` | `gate-fail` | `db7994fe63bf3d4eb9962efedfec5f842c8ea32e7518f0d4c071663b7ca505c2` |
| `HANDOFF/done/P0_BUILD_003_Core_Test_Stabilization.md` | `blocked-by-metadata` | `gate-fail` | `896f5881d6c019fad0c335ca3e90d9fc3b6d666422e23e96b1a2ddd76d3a9ea2` |
| `HANDOFF/done/P0_BUILD_004_Cargo_Clippy_Lint_Cleanup.md` | `blocked-by-metadata` | `gate-fail` | `c8aa9feb42f7f4756274b8b887fa161f310b490bfde4e2c9789cec5aa2d7eb20` |
| `HANDOFF/done/P0_BUILD_004_REPO_BLOAT_AUDIT_OPTIMIZATION.md` | `blocked-by-metadata` | `gate-fail` | `86a2215ea4ec6702ec1c1e90dbcd869d49cf699cd6afc0b0ebbd4fb1230e5d8f` |
| `HANDOFF/done/P0_BUILD_004_Windows_Test_Stabilization.md` | `blocked-by-metadata` | `gate-fail` | `8cca83a24d2297effaa1cb09d4cc65c30c25de955cba707c7420afaf6ae03503` |
| `HANDOFF/done/P0_BUILD_005_Android_Test_Config_Audit.md` | `blocked-by-metadata` | `gate-fail` | `a62444d5ac90985bd3732afa0f664c455e31ebf23baecbccf9cfa8f7110fdf46` |
| `HANDOFF/done/P0_CLI_001_Daemon_Smoke_Test_FINDINGS.md` | `blocked-by-metadata` | `gate-fail` | `035d9eb6641813924551e1d1fc5a2203e2b1649a92c424fe59cb67a119330f27` |
| `HANDOFF/done/P0_CLI_002_LAN_Message_Test.md` | `blocked-by-metadata` | `gate-fail` | `46bc82a0b24e00c060caed8a2f440f064b2e900b1af92872a2bafca2d57f3a80` |
| `HANDOFF/done/P0_COMPILE_GATE_VERIFICATION.md` | `blocked-by-metadata` | `gate-fail` | `1a4e6b0ca523927990be00fbc958cfe765452c6be502bc4dc4d79c2034f936da` |
| `HANDOFF/done/P0_CORE_swarm_rs_Test_Module_Broken_Imports_Blocking_Compile_Gate.md` | `blocked-by-metadata` | `gate-fail` | `f386206b5e09a4c711a0d071238892f972dd05f723c06570942318aeefba8ace` |
| `HANDOFF/done/P0_DEEPLINK_PARSES_BUT_NEVER_DIALS_2026-08-18.md` | `blocked-by-metadata` | `gate-fail` | `510c0d87a0ca5aec3222c9a8de3bfa1f74d2a27e05ff434cb5c90086ec4c1541` |
| `HANDOFF/done/P0_DESKTOP_BRIDGE_Missing_Linux_Cfg_Gate_On_ble_Module.md` | `blocked-by-metadata` | `gate-fail` | `acebd4d71d59c04ae2045cf6615ae72539682f989695701c25fa99fa19f268a3` |
| `HANDOFF/done/P0_DUAL_BIND_TCP_AND_WS_ON_SAME_PORT_2026-08-10.md` | `blocked-by-metadata` | `gate-fail` | `1511112820d1f8620f8303c45ca98116f528a4d49f5af6e00724fdebb778dec1` |
| `HANDOFF/done/P0_IDENTITY_002_Implementation_Summary_2026-04-24.md` | `blocked-by-metadata` | `gate-fail` | `ea4acdfb7fddba6486019f65cf91e5612d2654e76b5ec14b5e90997bf46c0ec2` |
| `HANDOFF/done/P0_IDENTITY_002_Unified_Infallible_ID_Strategy.md` | `blocked-by-metadata` | `gate-fail` | `925a4f4d3a192020bb68a67322dd514dac1f861bb424c71dc2016e942b88f348` |
| `HANDOFF/done/P0_IDENTITY_002d_Remaining_Validation_and_UDL.md` | `blocked-by-metadata` | `gate-fail` | `8a4e211310345889c2fe5793a63ea41ece3590d5d106792c59443cb440ccd145` |
| `HANDOFF/done/P0_IMPLEMENTATION_001_Unused_Code_Activation.md` | `blocked-by-metadata` | `gate-fail` | `c5ca09db9da99428978a725d44be18762cd396a39ed3d5fc506a84f7e80f2473` |
| `HANDOFF/done/P0_IOS_001_Field_Binary_Deployment.md` | `blocked-by-metadata` | `gate-fail` | `a79a5220f8c4428107b049d6aa2adf90954f62fc56ac7e918679c0c4b999ead0` |
| `HANDOFF/done/P0_IOS_002_Parity_and_Cross_OS_Unification.md` | `blocked-by-metadata` | `gate-fail` | `f90bc5dadd6f2010b61ef32701491051ad02e7fef271392e8cc1a06d44378112` |
| `HANDOFF/done/P0_IOS_002_Parity_and_Cross_OS_Unification_COMPLETED` | `blocked-by-metadata` | `gate-fail` | `c48179f5246e3cc7b6572fe9172fff21506ccf80bfd34bb21fd0b2db4e294b7b` |
| `HANDOFF/done/P0_IOS_002_Parity_and_Cross_OS_Unification_FINDINGS.md` | `blocked-by-metadata` | `gate-fail` | `63ea795bf726cc4e3ad2eb45d059bdbfbf796bbb44fe69222d8e12ed90fd9c1a` |
| `HANDOFF/done/P0_IOS_003_iOS_Parity_Gap_Remediation.md` | `blocked-by-metadata` | `gate-fail` | `1fe4fbcf66078a11b80f1ee009c6f2d309bf6192967900f64c620a95f64ad96f` |
| `HANDOFF/done/P0_JSONRPC_PARITY_EXPANSION_001.md` | `blocked-by-metadata` | `gate-fail` | `2f657126de2b16cd095ad73e804125f6e8147e567886e1d74940b62f963e4efa` |
| `HANDOFF/done/P0_NETWORK_002_RELAY_CONNECTIVITY_FIX.md` | `blocked-by-metadata` | `gate-fail` | `c3383cfe607ff4dd07de3908afa1e64cad7b83e6b1533827b353942a9c46eb0a` |
| `HANDOFF/done/P0_SECURITY_001_Bounded_Retention_Enforcement.md` | `blocked-by-metadata` | `gate-fail` | `6015d48bd94aa3ffa8aa7c2337bdc660430afcd5cc55115450170631d1afbd6f` |
| `HANDOFF/done/P0_SECURITY_002_Anti_Abuse_Controls.md` | `blocked-by-metadata` | `gate-fail` | `ca25f9733e50202547ac28361590d3b288b6162c875a2533e59f934d3beda1f4` |
| `HANDOFF/done/P0_SECURITY_003_Forward_Secrecy_Implementation.md` | `blocked-by-metadata` | `gate-fail` | `3181bb97dbeb3d9af02ed7162e52252759a8de24bb84d688d5f17a0a6bf837ad` |
| `HANDOFF/done/P0_SECURITY_004_Forward_Secrecy_Implementation.md` | `blocked-by-metadata` | `gate-fail` | `8d3bf26d36225e994376ff324e9c820c8823304a1ee8b747ac0389bbb31d10c1` |
| `HANDOFF/done/P0_SECURITY_004_Identity_Backup_Encryption.md` | `blocked-by-metadata` | `gate-fail` | `216a02760f81b6dded4a4ae873984a73fc98f22d879720833aa4cb383734a98f` |
| `HANDOFF/done/P0_SECURITY_005_Audit_Logging_System.md` | `blocked-by-metadata` | `gate-fail` | `a3e0ae58b61cd666daa34ed7f0324917f4e9f6e0a9ebd1fe8e3ae9d164603224` |
| `HANDOFF/done/P0_SECURITY_006_Consent_Gate_Enforcement.md` | `blocked-by-metadata` | `gate-fail` | `e9c52b33c2a77601770091ce82cc60e5eab921569f233fc435bd85a472dd7c61` |
| `HANDOFF/done/P0_TRANSPORT_001_CLI_Android_LAN_Unification_SUMMARY.md` | `blocked-by-metadata` | `gate-fail` | `54ca6e4d1d1ac5ff7d9e64b3858eecc29961801ff57e2a3900a044d80e28a3c4` |
| `HANDOFF/done/P0_WASM_003_Core_Cfg_Gating.md` | `blocked-by-metadata` | `gate-fail` | `b3aab3a7a53f50d0b77633a824fc008940c906c128cfd726c1a97f5c7979b6c5` |
| `HANDOFF/done/P0_WINDOWS_001_CLI_Integration_Test_Stabilization.md` | `blocked-by-metadata` | `gate-fail` | `19accbaed5289f5a6b2c579c5fdab0e6cf988a9795eac2c7c2bcd4c961908a0a` |
| `HANDOFF/done/P1-05_Build_Provenance_Stamps.md` | `blocked-by-metadata` | `gate-fail` | `40881545a39878a58f0c7dfaacd640fe305aa9748bc5fd93c86607b5b0fdc012` |
| `HANDOFF/done/P1-11_Listen_Side_Adaptive_Port_Selection.md` | `blocked-by-metadata` | `gate-fail` | `45056a4355f2eebd9ff45fdbaf61e99a85320308ceb5852febde0fab46fcff57` |
| `HANDOFF/done/P1-12_Advertise_Dial_Remember_Adaptive_Port_Selection.md` | `blocked-by-metadata` | `gate-fail` | `56daaf467062017c61719db762251e95744b39fd2f7e37620d0ff4147b704482` |
| `HANDOFF/done/P1-13_Hardcode_Sweep_Retire_9001_9002_9010.md` | `blocked-by-metadata` | `gate-fail` | `42b99a712fca2cd48cbc8930e41c6869fe29175a975a979d450998e768aff3fc` |
| `HANDOFF/done/P1-19_Phase_1_Exit_Review.md` | `blocked-by-metadata` | `gate-fail` | `16f7239d435418e1e4c48a36842cf78499e33a1e5e6b30171c98ff4219ab4e21` |
| `HANDOFF/done/P1_06_FOLLOWUP_mDNS_selfloopback_unit_tests.md` | `blocked-by-metadata` | `gate-fail` | `856bcd9315dc8df143107ac7c5097cb50150fa28056369ec4efe6cc87c36f387` |
| `HANDOFF/done/P1_ANDROID_003_Identity_Import_UI.md` | `blocked-by-metadata` | `gate-fail` | `05860ad06b8306dad703e828f9912c59e29d694ac47de0ada5bf9ffa23d26280` |
| `HANDOFF/done/P1_ANDROID_013_Bootstrap_Reliability.md` | `blocked-by-metadata` | `gate-fail` | `2674067f0b5cd8c2361022c8adeaa61a058bde685b2b8c5fb26876e7988e93ed` |
| `HANDOFF/done/P1_ANDROID_022_Accessibility_and_Polish_Audit.md` | `blocked-by-metadata` | `gate-fail` | `05fe919f1542257e5f187052172820e26dda8e233ef868660544866f5a82582e` |
| `HANDOFF/done/P1_ANDROID_023_Deep_Linking_for_Identity_Sharing.md` | `blocked-by-metadata` | `gate-fail` | `841e1a6c14ea20bfb4be007b8d37ff36f87c82c93132e7ce1afa36c54743a9e9` |
| `HANDOFF/done/P1_ANDROID_024_Manual_Dark_Light_Theme_Toggle.md` | `blocked-by-metadata` | `gate-fail` | `8d120155ed5b68e47be6c28d76f49675d71cd3ffd7acb3909c5c0b59642ef526` |
| `HANDOFF/done/P1_ANDROID_025_Google_Play_Services_Defensive_Check.md` | `blocked-by-metadata` | `gate-fail` | `3dbb586c9d8979cd9532602fc745239948f0e3139d4dabdf342101d459f54629` |
| `HANDOFF/done/P1_ANDROID_CONTACTS_FAB_REAPPEAR.md` | `blocked-by-metadata` | `gate-fail` | `aa02001f69cac8c16c6171905deec3f4b5d87c72c498dc83bc9a7acd3398ec3e` |
| `HANDOFF/done/P1_ANDROID_CRASH_TRIAGE.md` | `blocked-by-metadata` | `gate-fail` | `57546e1f2cced47606dc80e495f1f57d1e381a352fce5645f8c3b77a1de7edd8` |
| `HANDOFF/done/P1_ANDROID_Inbound_Libp2p_Listener_Not_Externally_Reachable.md` | `blocked-by-metadata` | `gate-fail` | `083ec31ea2beb80b5ba0662f8d44dcd70b49f675c46c18818818f355d4a2d426` |
| `HANDOFF/done/P1_ANDROID_LAN_DISCOVERY_REPAIR.md` | `blocked-by-metadata` | `gate-fail` | `50e4aabd64b4484ba372d2e450a65b5b036b37919cd145ec3e2cebc8e0a014e1` |
| `HANDOFF/done/P1_ANDROID_LAN_Discovery_Not_Feeding_Bootstrap_Peer_Count.md` | `blocked-by-metadata` | `gate-fail` | `d30f26e8a4b9af836d8f019e6d53b8f4b094144297951f2924625dfbe74a5faa` |
| `HANDOFF/done/P1_ANDROID_QR_Code_Identity_Export_Extremely_Slow.md` | `blocked-by-metadata` | `gate-fail` | `6ccfe447c209d0a15657ea97683096dda914b7e487299629e7d5c6b8a4f252d8` |
| `HANDOFF/done/P1_ANDROID_RELEASE_001_Build_Verification_and_Dashboard.md` | `blocked-by-metadata` | `gate-fail` | `07fb1da866188c518885f909903a869bc4d6423e3a424dc253bfa91e1978f649` |
| `HANDOFF/done/P1_ANDROID_TransportManager_LAN_Discovery_Never_Starts.md` | `blocked-by-metadata` | `gate-fail` | `e494bb404e8f0f6adec2d4edf1d5fefdd514a40f2eb1e48d8649db8999c8f513` |
| `HANDOFF/done/P1_ANDROID_UI_001_Chat_Empty_Loading.md` | `blocked-by-metadata` | `gate-fail` | `f53178842f9a2731d041d183978129c956b3c1d9a332c24956724896c8f06902` |
| `HANDOFF/done/P1_ANDROID_Unit_Tests_Force_Disabled_Since_2026-06-06.md` | `blocked-by-metadata` | `gate-fail` | `727f1b8b9b199c90a6ff183b779e5e68c869cc27f78cc1467a990cbebd0e61ce` |
| `HANDOFF/done/P1_ANDROID_mDNS_Self_Loopback_Discovery.md` | `blocked-by-metadata` | `gate-fail` | `e36ff1cc383019fe192cb214951bcb1f0c7ee8194c1b39329d6f9ec94f97d9f3` |
| `HANDOFF/done/P1_CLI_BLE_Outbound_TX_Path_Missing.md` | `blocked-by-metadata` | `gate-fail` | `0a52298be7624ca141015d6e823298794f3e09cfa0c7cd743022098052ec8b7d` |
| `HANDOFF/done/P1_CLI_Transport_Negotiation_Failure_On_Android_Inbound_Dial.md` | `blocked-by-metadata` | `gate-fail` | `614bb50cc916ecfd4a617a0743c76704715a5949d22d7dd2604fe37e49d4bc3c` |
| `HANDOFF/done/P1_CORE_001_Drift_Protocol_Activation.md` | `blocked-by-metadata` | `gate-fail` | `24e9736d7e7d94a05abf327762133106076031b0a0db9749220057c507cf1769` |
| `HANDOFF/done/P1_CORE_003_Privacy_Modules_Activation.md` | `blocked-by-metadata` | `gate-fail` | `e24d6837183dc55155aeab22a34e622de2255cf9c878a785a38516e9e8b5391c` |
| `HANDOFF/done/P1_CORE_004_Mobile_Receipt_Wiring.md` | `blocked-by-metadata` | `gate-fail` | `8a0107b7afde674b01f949f0bcc2f9b83214ead0a162b47c6040950c7e81f900` |
| `HANDOFF/done/P1_CORE_004_Outbox_Flush_Completion.md` | `blocked-by-metadata` | `gate-fail` | `3632e0017608fd7847237ac74639f95089a02e7bae3d3932cf7dac1bef49963d` |
| `SCM-Q-f450857842d1` (path hash `f450857842d1de69c33dc877b178a5336d8aa151fce68a647af24899bb69d70d`) | `quarantined` | `foreign-alias-in-bytes` | `3358eca267eac71f8e250b4c98636991f7995ee94e53685bf5aacfebdfcfd95b` |
| `HANDOFF/done/P1_CORE_005_Warnings_Cleanup.md` | `blocked-by-metadata` | `gate-fail` | `2a8e5ca786b003aaef73b5770389671b1099aa76c5629aed56c71df23b208fe7` |
| `HANDOFF/done/P1_CORE_BLE_GATT_Traits_Dead_And_Malformed_UUID.md` | `blocked-by-metadata` | `gate-fail` | `5efe500fcdc03738338019ad1764a707964f5c00645f180d2640429f724d75c2` |
| `HANDOFF/done/P1_CORE_Rate_Limited_Negotiation_Failure_Signal.md` | `blocked-by-metadata` | `gate-fail` | `bc5976c6a6a148364d3fc7d6ac2b6fc55828982eafe588cbb740ac41141243d2` |
| `HANDOFF/done/P1_DOCS_WiFi_Aware_T12c_Ledger_Correction.md` | `blocked-by-metadata` | `gate-fail` | `3ad965ddd46b6d8b72e47dcb51ee2ad76bb6a2810143620045c848f283d4df15` |
| `HANDOFF/done/P1_GEMINI_FLASH_021_CLI_Identity_Info_Expect_Panic.md` | `blocked-by-metadata` | `gate-fail` | `443ffbd31a8a877b056d09f9d04ee2da45fc34b741603b1466f99bf086c574f3` |
| `SCM-Q-dbdfabd9bfb8` (path hash `dbdfabd9bfb8e435624b80f5250d411fe066f604b91fad7c7dfbce73fdff9a4e`) | `quarantined` | `foreign-alias-in-bytes` | `02ee4f89ec2c0eb5c8d567b0629c6b7283890bacd12959c05cb6b3c83c0625f4` |
| `HANDOFF/done/P1_GEMINI_FLASH_023_Core_History_Retention_Corrupt_Record_Panic.md` | `blocked-by-metadata` | `gate-fail` | `86987001ba70b735ba07402edd8d4c2690deb8d0c1be7529de65848810c90b4c` |
| `HANDOFF/done/P1_GRACEFUL_DIAL_POLICY.md` | `blocked-by-metadata` | `gate-fail` | `bc71d4550a6815fd8181db3e435f7ec5e592362b84f80bb66ecfab028f0dc5b1` |
| `HANDOFF/done/P1_ROUTING_ENGINE_NEVER_LEARNS_PEERS_2026-08-10.md` | `blocked-by-metadata` | `gate-fail` | `e3a77c54c7971c2083720c5c222487fadcb212596d48d0d30ad6274bbb133c6b` |
| `HANDOFF/done/P1_WASM_001_IndexedDB_Persistence.md` | `blocked-by-metadata` | `gate-fail` | `db8dbd8022bf20dacd6e37ab5964f6e5dd9b291d7734b8f95521544c158d17db` |
| `HANDOFF/done/P2_ANDROID_BLE_MAC_Rotation_Breaks_Session_Continuity.md` | `blocked-by-metadata` | `gate-fail` | `cddd1b2dc46a582b5371999abe95e24bdb44e04b269cde8468c6de250e91f7ca` |
| `HANDOFF/done/P2_ANDROID_HARDCODED_STRINGS_CONTACTS_SETTINGS.md` | `blocked-by-metadata` | `gate-fail` | `0ce4dbf07e67f158e9e1da1d867083ba6737a5245f8f1765ffe51ff4cef1bec8` |
| `HANDOFF/done/P2_ANDROID_IDENTITY_QR_PRERENDER.md` | `blocked-by-metadata` | `gate-fail` | `b8de277e11d565baaf15ba661dad590aca2019571e93cb25c6252787ad26a0bb` |
| `HANDOFF/done/P2_ANDROID_IDENTITY_SCROLL_FIX.md` | `blocked-by-metadata` | `gate-fail` | `00a6b23dc68c666d3acb76b300e418fd0b65085a0a9e88df3cdb27b4d1c08975` |
| `HANDOFF/done/P2_BUILD_001_Core_Integration_Test_Fixes.md` | `blocked-by-metadata` | `gate-fail` | `748d863631bb4e2f89fd433a5e827e34eca440cbe62fca35c01ee39ca98eff9c` |
| `HANDOFF/done/P2_CLI_Orphaned_History_And_Contacts_Modules.md` | `blocked-by-metadata` | `gate-fail` | `6a31246b2da6d4fbef2f67de8364bb7cd27914c54ef0f7d100548099d2058093` |
| `HANDOFF/done/P2_CORE_Bootstrap_Test_Depends_On_Unset_Env_Var.md` | `blocked-by-metadata` | `gate-fail` | `c678b5f2e793b4359ac6fa657a6135dd898e99b66a4745a9cadec142a7f22374` |
| `HANDOFF/done/P2_IOS_ContactManagerFix_TryBang_FFI_Crash_Risk.md` | `blocked-by-metadata` | `gate-fail` | `1df68ffe114f7866e24c7dc29f28b92dad844f761a250ca2e8cb3e03492557cc` |
| `SCM-Q-af86c27b7cf7` (path hash `af86c27b7cf7c407d2b0b0d507e0c3365156e00f7d7fbf136f108b6623825e55`) | `quarantined` | `foreign-alias-in-bytes` | `6d088cafb9240d786ac052778a23db5ff039c378d4b2b8d6424389444e89a41c` |
| `HANDOFF/done/P2_OUTBOX_FLUSH_ON_RECONNECT.md` | `blocked-by-metadata` | `gate-fail` | `decfc5343e1d53e9e45849295187fbdbedeb7ca3e48464799e305bdaa1c265ae` |
| `HANDOFF/done/P3_ANDROID_NEEDS_PLANNING_IDENTITY_STATE_NO_REGRESS_FROM_READY.md` | `blocked-by-metadata` | `gate-fail` | `b629e1db991019721869e332d84227bb133a4ebaa3ae81f3a9ace112166780df` |
| `HANDOFF/done/P3_ANDROID_NEEDS_PLANNING_SMARTTRANSPORTROUTER_DEAD_PARAMS.md` | `blocked-by-metadata` | `gate-fail` | `a1d8762d96751d754c8a65211bd75a2b81b6004abddf16f9642f9c96ebe8f441` |
| `HANDOFF/done/P3_ANDROID_RETRY_SUPPRESSION.md` | `blocked-by-metadata` | `gate-fail` | `97706d7404ebf2ae8da5d7d9e6bf2851af956b16c3ecb1d324fe4902a1e5bfa0` |
| `SCM-Q-7f1e77ab001c` (path hash `7f1e77ab001c9507c1796bb91521a8f3d82e4e42b8aea707380e015f9cdc126d`) | `quarantined` | `foreign-alias-in-bytes` | `4fe33b48ad8673b35aeb230007e5f7d638e069ed883b0bbe46c96a1776466dbc` |
| `HANDOFF/done/P4_ANDROID_RECEIPT_UNIFICATION.md` | `blocked-by-metadata` | `gate-fail` | `1bc56cc07a7183d7c7ad66486cfe835ba8b470fa653e8521245b86d21af78e9b` |
| `HANDOFF/done/P6_FFI_SNAPSHOT_DRIFT.md` | `blocked-by-metadata` | `gate-fail` | `034952ea00d6f708138a491b201dd85cef1c702bb6dc1e814debd91f47d2e6da` |
| `HANDOFF/done/PQC_01_DEPENDENCY_MLKEM.md` | `blocked-by-metadata` | `gate-fail` | `c18689484b975ae45c32fd41041f1db2d1edc3bcb0e5fb3ff6ce54b4f4f48bcc` |
| `HANDOFF/done/PQC_02_ENVELOPE_V2.md` | `blocked-by-metadata` | `gate-fail` | `c33833153d447cd75a0e589dc5bb43fd498af481109863c804528ad3079c719b` |
| `HANDOFF/done/PQC_03_IDENTITY_V2_KEYBUNDLE.md` | `blocked-by-metadata` | `gate-fail` | `f1c456951fe7f72f97902d911cf82f230df32d99e417eb17fe7791840395f504` |
| `HANDOFF/done/PQC_04_SUITE_NEGOTIATION.md` | `blocked-by-metadata` | `gate-fail` | `a938e2b226f12bcbf84be070291168f39e09479262efe242387b3e3545477b2b` |
| `HANDOFF/done/PQC_05_HYBRID_KEM_MODULE.md` | `blocked-by-metadata` | `gate-fail` | `2d6b131d829cc46dca2976b592e65992f515e19ceece85bf666013b9ac774acc` |
| `HANDOFF/done/PQC_06_HYBRID_SESSION_INIT.md` | `blocked-by-metadata` | `gate-fail` | `e6d39ce25dfb59fb172aec2c02fdfef9743c9429a043ecb3ed305cd1504ba68e` |
| `HANDOFF/done/PQC_07_CADENCE_TEST_COVERAGE.md` | `blocked-by-metadata` | `gate-fail` | `bf109312f99ac70d8917bcd5fb11a3836fc6bd6583a07cdfb0225f589ca9c368` |
| `HANDOFF/done/PQC_07_COMPILE_FIX.md` | `blocked-by-metadata` | `gate-fail` | `5ab51ae7d2860d77639e077914ac9de183aaff3ea45def5bb0b4de549100f73e` |
| `HANDOFF/done/PQC_07_FORCE_RATCHET_SAME_DEFECT.md` | `blocked-by-metadata` | `gate-fail` | `89eae21b8d1ee805ea4e19d3f5491132d00be64386718d3418cb7d2d93aefa57` |
| `SCM-Q-c26c4e348d5e` (path hash `c26c4e348d5e966499581b21baabd8f358b3513b947350aed4610ce038d19326`) | `quarantined` | `foreign-alias-in-bytes` | `c32207950ddc4640be0ae772d2a15645e3ad1e2649f1a5f1ba4f565505e922d0` |
| `HANDOFF/done/PQC_07_PQ_REFRESH_WITHOUT_DH_CROSSING.md` | `blocked-by-metadata` | `gate-fail` | `41c95a0efbde7b9a7878082793e962ae6f5e2b3f5dfe86cf5cb6ff94c0288450` |
| `HANDOFF/done/PQC_07_PQ_SECRET_NEVER_MIXED_INTO_ROOT_KEY.md` | `blocked-by-metadata` | `gate-fail` | `cd3ff7f069cafe1a1339ab3868c6829f6506b35b6712736f801d0c722af0c4d3` |
| `HANDOFF/done/PQC_07_RATCHET_COMPILE_FIX_V2.md` | `blocked-by-metadata` | `gate-fail` | `29f5782fb633c962217bc5cf38e796033d76db5095c11161337f731c22f3df16` |
| `HANDOFF/done/PQC_07_WIRE_RATCHET_STEP.md` | `blocked-by-metadata` | `gate-fail` | `5214ab6ca7d7b5aa846196ebfdc80feb9e6a17407473ec50f1a4d138f9c9bd28` |
| `HANDOFF/done/PQC_08_COMPILE_FIX.md` | `blocked-by-metadata` | `gate-fail` | `f8fd7c3aeb63e0747c94a802a4dcb727491ea9c48b9481d9cba0997b48fab008` |
| `HANDOFF/done/PQC_08_LEGACY_PATH_RETIREMENT.md` | `blocked-by-metadata` | `gate-fail` | `a1e8091b4a1cfcbc43d5226c821ae6c23bc04cce4f7380c4bb1198cde57656c4` |
| `HANDOFF/done/PQC_08_TEST_CALLSITE_FIX.md` | `blocked-by-metadata` | `gate-fail` | `f84b2fb525695a36e03e76530265302dcc303d46ff142aa57ccd23fb21767d91` |
| `HANDOFF/done/PQC_08_V2_NO_SESSION_BUG.md` | `blocked-by-metadata` | `gate-fail` | `e98747a694daa7abdebafe34157b4e8919378c08f0eefc1dae5b47eed134bff1` |
| `HANDOFF/done/PQC_10_MLDSA_IDENTITY_SIGNATURES.md` | `blocked-by-metadata` | `gate-fail` | `1d4d37f85693d56a703860566051cfc603554e4b2e4c7cec7a33749ffdd6c53f` |
| `HANDOFF/done/PQC_11_RELAY_INVITE_HYBRID_AUTH.md` | `blocked-by-metadata` | `gate-fail` | `7535f8a51f7a648de50972b602db883958f85e4803f3616efa1ed3924f1398c0` |
| `HANDOFF/done/PQC_12_TRANSPORT_TLS_PQ.md` | `blocked-by-metadata` | `gate-fail` | `bbf3ba537d184684b460eaa7dd4a6a6d18a66156406e5edd932f884efa7c1857` |
| `SCM-Q-bfd12bae37de` (path hash `bfd12bae37def005bb5d191ced23b1b01006aa1ecebfb7aacac70a4221f989ac`) | `quarantined` | `foreign-alias-in-bytes` | `cf6beedb0ec3e4e117ce9426fa36c5be7a4704e8520dd88115e65e4274ca75a0` |
| `HANDOFF/done/PQC_14_DOCS_AND_RISK_REGISTER.md` | `blocked-by-metadata` | `gate-fail` | `2cac05e6b75629b6c8dea877aa55608a61bbc94d1d7987be69bb5c9b680e3f93` |
| `HANDOFF/done/PQC_RATCHET_SKIPPED_KEYS_NOT_PERSISTED.md` | `blocked-by-metadata` | `gate-fail` | `24dad6d3f2e9e2499e372985655d501979a581b8c914f8918e4e3369aba42158` |
| `HANDOFF/done/PQC_REVIEW_CHECKPOINT_05_06_07.md` | `blocked-by-metadata` | `gate-fail` | `ecb53d307691d62b91fc973fca3ea76669ba47b837d133069736bc035f182bab` |
| `HANDOFF/done/PQC_REVIEW_PARTB_RETRY.md` | `blocked-by-metadata` | `gate-fail` | `cb07febf9a608f45bfd012a60a5dcf5a62b4b5b783236893e34edf873a159193` |
| `HANDOFF/done/PQC_SUITE_NEGOTIATION_TEST_FAILURES.md` | `blocked-by-metadata` | `gate-fail` | `5c18b9a9fe43c853b2240a321af4d9b44c968c6fc376f822da6c557922f89307` |
| `HANDOFF/done/PROVE_SECOND_REAL_ENDPOINT_DELIVERY.md` | `blocked-by-metadata` | `gate-fail` | `13ee31ef91f6176a7759ef97d044d0acfeb6d219295c9c0ed159f9b40179ced8` |
| `HANDOFF/done/REVIEW_ANDROID_IDENTITY_FIXES.md` | `blocked-by-metadata` | `gate-fail` | `7bba063e53e60a4e413760bfa8427ef232408d0d3825ec3387227a1d59ea05e3` |
| `HANDOFF/done/REVIEW_ANDROID_IDENTITY_FIXES_COMPLETED.md` | `blocked-by-metadata` | `gate-fail` | `a65fb863044c44fdae4d137a3f6f5529502f7c370260845175e5d9ece196124f` |
| `HANDOFF/done/RUST_MOBILE_BRIDGE_BOOTSTRAP_FIX.md` | `blocked-by-metadata` | `gate-fail` | `d76e7e1ba1afa686b028a1192a320ff0a7dc688c03765608d70668341b130d72` |
| `HANDOFF/done/SWARM_ARCHITECTURE_GUIDANCE.md` | `blocked-by-metadata` | `gate-fail` | `cd643bde9dac13a98a9f2ece8bf4cd7c5eeca27fe3db5fd0b2777d88a960946f` |
| `HANDOFF/done/SWARM_ARCHITECTURE_GUIDANCE_COMPLETION.md` | `blocked-by-metadata` | `gate-fail` | `fc6b710eeaa05f93f8a60b5ce9981f50a7d0d78d95efcdbf7063bb0d0889795a` |
| `HANDOFF/done/T1_DUAL_FLAVOR_BLOCK_DISPATCH_2026-08-06.md` | `blocked-by-metadata` | `gate-fail` | `e602ec4d85c1f7e1d8b5fc00f018801ccaef7967fd8f9cd19d57c5f83d1b99a3` |
| `HANDOFF/done/TASK_CI_IOS_BINDINGS_DRIFT_FIX.md` | `blocked-by-metadata` | `gate-fail` | `24220cc2fa7b5bd92b319d72dc89b1a42c0b3dd5925d3ea3452779fd01614a7f` |
| `HANDOFF/done/TASK_CI_IOS_MACOS_RUNNER_FIX.md` | `blocked-by-metadata` | `gate-fail` | `00793e4f99974fd7b1befd33e690d86f9248328788d239041a6f902467f5d12f` |
| `HANDOFF/done/TASK_DELEGATE_DIFF_MODE.md` | `blocked-by-metadata` | `gate-fail` | `1e4d20d3f59dfb7b74ced85401ca357ab65e27fd2729e9b33eccb4f1c4586241` |
| `HANDOFF/done/TASK_DELEGATE_VERIFY_LOOP.md` | `blocked-by-metadata` | `gate-fail` | `4dfcee89eaf1b2b474a00ab2e54205c664186a81f5f8bd08022f7f56e13e4b3e` |
| `HANDOFF/done/TASK_INFRA_CLAUDE_PATH_FIX.md` | `blocked-by-metadata` | `gate-fail` | `219444e7250d84ef0e3038958c2b0b21713eec48939e41edc9ee9ef528bfc8b7` |
| `HANDOFF/done/TASK_VERSION_BUMP_0_3_5.md` | `blocked-by-metadata` | `gate-fail` | `a1d3136928fc725fd1e2640f2214375d42f1e09b52157868b0eab3775f65d2e9` |
| `HANDOFF/done/TEST_AGENT_RESPONSE.md` | `blocked-by-metadata` | `gate-fail` | `7def5563ec0923b1533007ae5d7ede2d01087058ce77fb3195729b4a1a1057c5` |
| `HANDOFF/done/U1_OUTBOX_OPEN_DEFAULT_HELPER.md` | `blocked-by-metadata` | `gate-fail` | `ab53b9310baa78fff97ece872ba3925de138c0667ca5d8c7f759fd5445951ad8` |
| `HANDOFF/done/U1_outbox_open_default.md` | `blocked-by-metadata` | `gate-fail` | `586197d2581966abf58a597bc38a7a5752c5603391bc06b9e56a587a6a2f8342` |
| `HANDOFF/done/U2_TOPIC_CONSTANTS_UNIFICATION.md` | `blocked-by-metadata` | `gate-fail` | `6493ec66ff3e63f43ea56605273aa16c3cb243871683930de8e711bd3850574d` |
| `HANDOFF/done/U3_RETRY_POLICY_CORE.md` | `blocked-by-metadata` | `gate-fail` | `5ba60a46bc196f60e3992467cd15cb7125b655f933eb936ed9ce550ec52b72bc` |
| `HANDOFF/done/U4_RECEIPT_ENCODING_UNIFIED.md` | `blocked-by-metadata` | `gate-fail` | `27f0dbed5d99bc71ce19412c2bf680ddbc6927448646cd9ead768154c2c2d82c` |
| `HANDOFF/done/U5_ANDROID_RECEIPT_UNIFICATION.md` | `blocked-by-metadata` | `gate-fail` | `0f68cdcdcf41d532e732c4fff10d68232e065fd7c9bcf91f70f669afbe01efdd` |
| `HANDOFF/done/U7_SCHEMA_DRIFT_AUDIT.md` | `blocked-by-metadata` | `gate-fail` | `e2301bd756b1ddd4c845957b0683dfd2cedff4c60546cae20f9db32cfb5ea9f3` |
| `HANDOFF/done/UNIFY_GEMINI_DOCS.md` | `blocked-by-metadata` | `gate-fail` | `f85befb4eb20d7432ef27c174a8237d90c695737d463b685ffd7268b58da71c0` |
| `HANDOFF/done/VERIFICATION_REPORT_P0_AUDIT_001.md` | `blocked-by-metadata` | `gate-fail` | `690fe8f5e007f250b628df08488ce9c9faf83ea846a8d3877ddf9d174efbaf75` |
| `HANDOFF/done/WIFI_AWARE_HARDCODED_PMK.md` | `blocked-by-metadata` | `gate-fail` | `1b3b1fadf42d82206f0587d73757a4c4177be2e54ce4b99822522553fbb755ec` |
| `HANDOFF/done/Z-02_Regenerate_one_self_contained.md` | `blocked-by-metadata` | `gate-fail` | `3643512cd904fbd32d9cc22c9648703cf358024fbf5a4c728b604ced5ebfb4ac` |
| `HANDOFF/done/Z-03_Rebuild_HANDOFF_ACTIVE_LEDGER.md` | `blocked-by-metadata` | `gate-fail` | `debf87291c1adc2b3fa84bc876f7d375ab9a491c1970924a90c1a4ea4cfd84e2` |
| `HANDOFF/done/[META]_ORCHESTRATOR_ROLE_PROTOCOL_v1.md` | `blocked-by-metadata` | `gate-fail` | `f6c2a77bdfec4cf7412b873114756d18b5564ae72b2a24b574d97ce841083ca2` |
| `HANDOFF/done/[META]_ORCHESTRATOR_WORKER_POOL_WARMUP.md` | `blocked-by-metadata` | `gate-fail` | `54e0d7aa21ab3248b4bf760a55dadcae49276297d4f66119e4ae354945d7b96e` |
| `HANDOFF/done/[META]_QUOTA_LEDGER_REPAIR.md` | `blocked-by-metadata` | `gate-fail` | `6e399650e8f664f2a19a5c6a5ffa535af250617d5d55c9da71914f72e661bb58` |
| `SCM-Q-0cf0d4f71f93` (path hash `0cf0d4f71f9399dd8f71af8e1ba47b428dfa0be74a18a40084b50974e7dbc7cf`) | `quarantined` | `foreign-alias-in-bytes` | `b48288acb55c0b30c461cf97c398b9637880805fcb56d242a30d3b616b588879` |
| `HANDOFF/done/[VALIDATED]/[STALE]_[VALIDATED]_phase_2_platform_clients.md` | `blocked-by-metadata` | `gate-fail` | `4c27ba81d8636e8cee8b71e17c8c45cf36c98eb048ca6f3da74999d9d1538c5f` |
| `HANDOFF/done/[VALIDATED]/[STALE]_[VALIDATED]_task_p1a_illegalstate_crash_audit.md` | `blocked-by-metadata` | `gate-fail` | `250d98055316f746093b7f25c3bc211e888ca4d6764fd0d4a4f2919faac40a21` |
| `HANDOFF/done/[VALIDATED]/[STALE]_[VALIDATED]_task_recovery_session_2026-05-14.md` | `blocked-by-metadata` | `gate-fail` | `9e243c80c17c70ef8498a55e111ebfdbc39ece94b93bb390aff431fdea14abc0` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_MICRO_RUST_RELAY_ONION_ENABLE_001.md` | `blocked-by-metadata` | `gate-fail` | `270c22aca8638ca6554fb84f7e6a1e1f3db5db5c4dd2d008a748374e9c4efdb5` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P0_ANDROID_024_DISPATCH.md` | `blocked-by-metadata` | `gate-fail` | `2166282cfe8d8ab6d4f6d9c1f0bf44fbd8e84edef7d931c8678d19cd9e597313` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P0_ANDROID_AGY_HANDOFF_2026-06-07_Identity_Stability_Bundle.md` | `blocked-by-metadata` | `gate-fail` | `5be49cc2ffa829b8649bd60112e9bad2708ac29a5161c15e9bc47d35f256248b` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P0_AUDIT_HERMES_HANDOVER_2026-06-07_Post_Session_State_Audit.md` | `blocked-by-metadata` | `gate-fail` | `2c861b7b2b96def975afd962a3446efea794dcd81e5e418578d7187898854f16` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P0_CLI_023_ContactManager_Shared_Backend_Key_Collision.md` | `blocked-by-metadata` | `gate-fail` | `6d35c56c051a40c4ad1252db6cfb5331a5d802a13465e0ffa29aea2a63c1f20e` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P0_CLI_027_Drift_Protocol_Still_Dormant_At_0_2_1.md` | `blocked-by-metadata` | `gate-fail` | `102729d27e4901de55ce222b670439a2a793f7e19ddffe13a02d5d9fec71c13f` |
| `SCM-Q-92c557d6bbbd` (path hash `92c557d6bbbd36d8508769a7b36461df6cbe37f429966b0f8b63756007c8b121`) | `quarantined` | `foreign-alias-in-bytes` | `7388ae870b49bba4f99a3d6bd78653161f5b1d725caa522c9dbfd89f8cc30fba` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P0_RELEASE_001_v0.2.1_Complete_Notes.md` | `blocked-by-metadata` | `gate-fail` | `3f22a1397c524615e17d303b6ccf026cd38bb919874441a53c750ee9fc857617` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P0_SECURITY_007_Identity_Backup_Encryption_V2.md` | `blocked-by-metadata` | `gate-fail` | `635b76977838c1cf6c4a0d2947514697ec7fb0158726704b501d435e82802b57` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P0_SECURITY_008_Audit_Log_Identity_Ops.md` | `blocked-by-metadata` | `gate-fail` | `d461554ec2f9800cfbf388d0aeedab046e629a2eff3b69b33d41e778a2d4e572` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P0_SECURITY_009_Sled_Compaction_And_Monitoring.md` | `blocked-by-metadata` | `gate-fail` | `fcaf34377e16222ed433717817423aeee11f7c835700c66caa77144afe80697b` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P0_SECURITY_010_Api_Level_Consent_Gate.md` | `blocked-by-metadata` | `gate-fail` | `bd2594c4bf2f9908fcf3a17b4d77be3c6a47c5c2ff463dac566849517bbee95b` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P0_SETUP_001_Workstation_Cleanup_And_Model_Install.md` | `blocked-by-metadata` | `gate-fail` | `97ca2588acec66a6619bc7a5006eb3f6f83a31615372064bc1bac846ef3938b6` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_ANDROID_AUDIT_LOG_VIEWER_001.md` | `blocked-by-metadata` | `gate-fail` | `059b27ee28576d8aaece9d4dfbbee20c6876733558bc63d584ed4914a84f7fc7` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_ANDROID_MESSAGE_SEARCH_UI_001.md` | `blocked-by-metadata` | `gate-fail` | `55cd090486746d8318a722cbe2dd1821c302a94dbac0856f5d1b0202dc07b959` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_ANDROID_PLAY_READINESS_AUDIT_001.md` | `blocked-by-metadata` | `gate-fail` | `8b945bce2bf8341a7a9358b5164a985fb3f2d876e8354fd2c320411244788b80` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_CLI_025_Identify_Protocol_Spam_From_Relay_Peer.md` | `blocked-by-metadata` | `gate-fail` | `271d90dcd82735e60bbb6209928502fa12df5f2204e99235e25e444ea6fdfc23` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_CLI_026_External_Address_Omits_LAN_Interface.md` | `blocked-by-metadata` | `gate-fail` | `af24b559270b52364e183b97b8027367deec320208f23ec3dffec2c23bbc8a79` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_CLI_028_Config_Listen_Port_Stale_vs_Actual_Port_9101.md` | `blocked-by-metadata` | `gate-fail` | `562c393bd42eeb614c5fe463f176a08ad306edd1484dde01f56ec52294e36ba7` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_CLI_029_Running_Binary_Cannot_Be_Killed_Or_Replaced_For_Build.md` | `blocked-by-metadata` | `gate-fail` | `da0a509335b76f0bba27f0af6bdeda620ad370258ca4411cfe0e150a8439edb5` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_CLI_030_Discovery_Peers_Transport_Hardcoded_As_TCP_LAN.md` | `blocked-by-metadata` | `gate-fail` | `16891cb7747c04a75cb00545d0a883af2d0f0ed545c46f1c6e1a5fba192bb5ac` |
| `SCM-Q-e7a51b191033` (path hash `e7a51b191033db2a8e65d2391e3c36d932d0b325a4f58a1ac9d6bdff6433f33e`) | `quarantined` | `foreign-alias-in-bytes` | `c41aeaadeae2629af16356285b57110bf71b561cdd85f5a5416daa7f201a24e0` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_CLI_032_Control_API_Missing_GET_Contacts_Endpoint.md` | `blocked-by-metadata` | `gate-fail` | `63e4d84b321558471e94dff88f94dbf8e08b598ccc3ed30e1d8deda8d7b3fa02` |
| `SCM-Q-74911f47ab72` (path hash `74911f47ab72830afcd84a95950b28808549aea1f9fc97cc318ae8a56e9cd055`) | `quarantined` | `foreign-alias-in-path-and-bytes` | `267cae5124401386d9b36e4d11ff5102a165e71bd56e584c261fe2745c247c83` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_CORE_001_Drift_Protocol_Production_Wire.md` | `blocked-by-metadata` | `gate-fail` | `be626f7bb95f9387fd01101f883b75bb911308aa3d5c313a819e50f82c078cfb` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_CORE_002_Mycorrhizal_Routing_Production_Wire.md` | `blocked-by-metadata` | `gate-fail` | `31ca9a421af3bcd8f40051022ae2903320358eea50416fd1755c29dc0c9c1585` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_CORE_003_Privacy_Modules_Production_Wire.md` | `blocked-by-metadata` | `gate-fail` | `314955d5a047179733f7b33c623780bfcda58aa173d3f8cd2cd32da2d346b654` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_CORE_004_LZ4_Compression_Production_Wire.md` | `blocked-by-metadata` | `gate-fail` | `047a37a879f604cec4ea418f6758ca7ac88e61e976e7b5433a3a1ea001706135` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_GEMINI_FLASH_001_Permission_Loop_State_Machine.md` | `blocked-by-metadata` | `gate-fail` | `b1fefc70198b8bde9dfc4361e302e7dd7d61835f3fef8a970b55ebe23a571a49` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_GEMINI_FLASH_002_Relay_Peer_Contacts_Filter.md` | `blocked-by-metadata` | `gate-fail` | `2472ab64fc2cd4af0e5b96a15c7a173d1d5d671edb70be401bfbf7c9157a031a` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_GEMINI_FLASH_004_Android_Backup_Exclusion_Rules.md` | `blocked-by-metadata` | `gate-fail` | `6d6974d654a6a967f5812feea07b793821490ea39327ef32ebfd41374bfffb18` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_GEMINI_FLASH_005_CLI_GET_Delete_Contacts_Endpoint.md` | `blocked-by-metadata` | `gate-fail` | `b52811cf4cde4dc5fa14bbec7c52fe5e0b58f7e05660443c06d8cc9571a202dd` |
| `SCM-Q-b423686be389` (path hash `b423686be38925348a0b50916e38fe1949e10131195ae8687bcb1833d39e3c4f`) | `quarantined` | `foreign-alias-in-bytes` | `e747226b944ec1144349e2d1f2d0b21295954a5bc863099ad9dfc1b0ba40e0c4` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_GEMINI_FLASH_007_Docs_Path_Normalization.md` | `blocked-by-metadata` | `gate-fail` | `9ccda6777a94424cb9807d793315211de629ad72f17a937ca8364f6150173ab4` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_GEMINI_FLASH_008_Active_Ledger_Stale_Banner.md` | `blocked-by-metadata` | `gate-fail` | `30a66dd493686355ca42e676a1dc5294056500f7c79d9093183f2ef8a66af1dd` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_GEMINI_FLASH_009_Repo_Root_Helper_Script.md` | `blocked-by-metadata` | `gate-fail` | `5980b58e8f779644c063112e881fd426c972d667aa3f614b0271a140be4b2098` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_GEMINI_FLASH_010_Quota_Governor_Runbook.md` | `blocked-by-metadata` | `gate-fail` | `b69d658dfedd88c304411ae33adb70b93284d724c9b027321998e03136e72fa1` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_GEMINI_FLASH_011_ContactItem_Nickname_Overflow.md` | `blocked-by-metadata` | `gate-fail` | `908e943d167ddf4f6a5bdf311d2bf607d983a49ad3ca79cd243d5b8a56427f1d` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_GEMINI_FLASH_013_iOS_Verify_Test_Hardening.md` | `blocked-by-metadata` | `gate-fail` | `828921175e013133ffbcb1b34900d1c9413cb3356a58ebde5803e962229e2207` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_GEMINI_FLASH_014_Remove_Deprecated_Read_Variant.md` | `blocked-by-metadata` | `gate-fail` | `705f73c5d136d57fc3dffac53e31f9f1d52b008c6dd05e7d02e776862cdac991` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_GEMINI_FLASH_015_WiFi_Aware_Discovery_Counter.md` | `blocked-by-metadata` | `gate-fail` | `ae828c059e3af3c9d7cf63576112e08800dc362c3a14ec7e61ff571ce649c9aa` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_GEMINI_FLASH_016_Write_Ollama_Config.md` | `blocked-by-metadata` | `gate-fail` | `76d7f8870b019305e8fead5c7d4d28962bf75b33749870830920ff21516dc9cc` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_GEMINI_FLASH_017_Verify_Task_Completion_Script.md` | `blocked-by-metadata` | `gate-fail` | `f329e612bbd38ed0103214516fa843b139566562a36c12f99b2e478cf5d86a7e` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_GEMINI_FLASH_018_Compile_Gate_History_Ledger.md` | `blocked-by-metadata` | `gate-fail` | `2eeea95df6e6befc4e517ca9a59421cd4fbd8aad73aa934323adbe8643569fc0` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_GEMINI_FLASH_019_Corrected_Routing_Claim_Notes.md` | `blocked-by-metadata` | `gate-fail` | `dde24fc90ecafd9243e678dccb1e2b6f33f0ca3efb321c52427adde248f1e17a` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_GEMINI_FLASH_020_Nightly_CI_Workflow.md` | `blocked-by-metadata` | `gate-fail` | `dacb3c94f0a98276a3e9f62e36f7f24043d96c24d245b5358133352cffb2f7cc` |
| `SCM-Q-6a3183fa3769` (path hash `6a3183fa376981021b8e14f8ea8d81a7d4bc583fc098208bc99cd7a4d6767234`) | `quarantined` | `foreign-alias-in-bytes` | `fdb3e77705bc53d6d8957b87644dccbaffe69c88e22874cf0ba0cd808da13c82` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_IOS_002_Notification_Permission_Flow.md` | `blocked-by-metadata` | `gate-fail` | `85d0807dba760bebe6cbad31dc00a57645c32fe1bdc6b87e5363639ce70859da` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_IOS_003_Background_Mode_BLE_Multipeer.md` | `blocked-by-metadata` | `gate-fail` | `e73f463e00a2ab3a97a3c22a14ca36cccce58a350315d59935a462f7d5e94ef0` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_PLATFORM_001_Outbox_Flush_PeerDiscovered.md` | `blocked-by-metadata` | `gate-fail` | `cf62b3ee85e282ee9f7687814d0b1c5d0cd380fbeefa6784aa040147d53fc996` |
| `SCM-Q-33905ea60bae` (path hash `33905ea60baeb85c8023fb977367b355d721405cb7bb77d67e993de22ed71243`) | `quarantined` | `foreign-alias-in-bytes` | `ad6d6346a3dbd1a29459972b3f8e1acd2bbfeaab4d0ad53327bdd79bae6fbbdb` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_WASM_003_End_To_End_Test_CLI_Local_Authority.md` | `blocked-by-metadata` | `gate-fail` | `5eb33de1dbf8dd31d5274389660406eb4122990534ad9ac8ff738348d0562a29` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P1_WASM_004_Comprehensive_WASM_Feature_Extension_Suite.md` | `blocked-by-metadata` | `gate-fail` | `e182af1871ef8354bdb488c784d159a97a0ded5fd73f66fe797b6027a8bc5d94` |
| `HANDOFF/done/[VALIDATED]/[VALIDATED]_P2_ANDROID_IDENTITY_QR_PRERENDER_AND_SCROLL.md` | `blocked-by-metadata` | `gate-fail` | `af5e147a9c68922288fdd36431786d24e789c3aeeb633b972af6baeead107300` |
| `SCM-Q-3c06e3d0373a` (path hash `3c06e3d0373abcabaa2fccd455bcc9e1539022d3ad20c653326451568e83e768`) | `quarantined` | `foreign-alias-in-path-and-bytes` | `ab0cd70e270107858302dd3df478204842601c8d42384a5d0dbc088b2e25364b` |
| `HANDOFF/done/[VALIDATED]_BATCH_RUST_GROUPB_DSPY_MODULES.md` | `blocked-by-metadata` | `gate-fail` | `e0867efdd5dbd4294e9f2042ee028a65ec10c1f4d86c4858b521d37a126fc23b` |
| `HANDOFF/done/[VALIDATED]_BATCH_S1_T1_FIX_ANDROID_BUILD.md` | `blocked-by-metadata` | `gate-fail` | `5319393475fd98cbb577b9a0f7beccfbd9d63eaa094b9c205e2428bc6210c714` |
| `HANDOFF/done/[VALIDATED]_FIX_ANDROID_BUILD_001_MSVC_Target_Alignment.md` | `blocked-by-metadata` | `gate-fail` | `9c478f6dc774881ffe9f9cea7a883fec1588208a5f1f257979bb75d5696a74f0` |
| `HANDOFF/done/[VALIDATED]_FOR_BETA_SWEEP_B2_CORE_TRANSPORT_ROUTING.md` | `blocked-by-metadata` | `gate-fail` | `16dd14a30f5cb3a2d5263c14a8e74a7ce8242d6256a70c373b0313e28aa88464` |
| `HANDOFF/done/[VALIDATED]_IN_PROGRESS_P0_NETWORK_001_BOOTSTRAP_FALLBACK_IMPLEMENTATION.md` | `blocked-by-metadata` | `gate-fail` | `6d9adfc25169c1d521af0ae14ef91627302135d34f46722eaaad43a19d1038ee` |
| `HANDOFF/done/[VALIDATED]_MICRO_004_Install_Windows_GNU_Target.md` | `blocked-by-metadata` | `gate-fail` | `2b0255b5d78b374a8d9112c716762bcf0bf3784a7710d4fe5370b69be31a9db5` |
| `HANDOFF/done/[VALIDATED]_MICRO_ANDROID_NOTIFICATION_STRINGS_001.md` | `blocked-by-metadata` | `gate-fail` | `f5197cfd353a89292119522f36ca2bf86108f9825dad1f47ad0c51f090fc7637` |
| `HANDOFF/done/[VALIDATED]_MICRO_ANR_001_MeshRepository_RelayIdentity_SafeReturn.md` | `blocked-by-metadata` | `gate-fail` | `ca7e69da3b2edfd272f9adc1fe9cd828a1dcbb45751f15cf89adb6f4f21dc23e` |
| `HANDOFF/done/[VALIDATED]_MICRO_ANR_002_MeshRepository_EmptyMessageId_SafeReturn.md` | `blocked-by-metadata` | `gate-fail` | `0e88336de264ae8d509c5ec6c3987b1eb1c52e0d267f7f1463dba52a9a60d07a` |
| `HANDOFF/done/[VALIDATED]_MICRO_ANR_003_ForegroundService_RunningState_SafeReturn.md` | `blocked-by-metadata` | `gate-fail` | `42d2798d18df44d764ba25631b84330c7fac2792df8310db6e4bca646bc731a4` |
| `HANDOFF/done/[VALIDATED]_MICRO_DEPRECATION_001_BleGattServer_API31_ExecutorOverload.md` | `blocked-by-metadata` | `gate-fail` | `013787e94ff6f15cb4a27668452ace891eb125a44d2dec9859d01507b49b9815` |
| `HANDOFF/done/[VALIDATED]_MICRO_DEPRECATION_002_MdnsServiceDiscovery_API28_GateFix.md` | `blocked-by-metadata` | `gate-fail` | `42a0e1980765a105c2faf461f13006ec86190b50eef6a8638600f532ee51e817` |
| `HANDOFF/done/[VALIDATED]_P0_ANDROID_019_Auto_Backup_Stale_Data_Fix.md` | `blocked-by-metadata` | `gate-fail` | `dd6753a92e801c078613e479364a06d0a5f88435f04c21043b66613730da8b0e` |
| `HANDOFF/done/[VALIDATED]_P0_ANDROID_020_Permission_Request_Loop_Fix.md` | `blocked-by-metadata` | `gate-fail` | `a8b8d266f6911b6e1f68da77075101cb612e0f521dd26fbdbaf26d0047ed7ea6` |
| `HANDOFF/done/[VALIDATED]_P0_ANDROID_022_Relay_Peer_Contacts_Filter.md` | `blocked-by-metadata` | `gate-fail` | `50a77ea247f8ddc6991ce52a33e3ac5988fa09baee780b4f2ed89817e8fb32c3` |
| `HANDOFF/done/[VALIDATED]_P0_ANDROID_024_Identity_Generation_Reentrant_Guard.md` | `blocked-by-metadata` | `gate-fail` | `457b39a99d190ac744ec9650998958e1af37184420edc0657e17595367492045` |
| `HANDOFF/done/[VALIDATED]_P0_BUILD_001_Workspace_Test_Gate_Restoration.md` | `blocked-by-metadata` | `gate-fail` | `16443de99c92772d9772baac962c4aac5cb847414935c47ff2a3e6f0f28413c5` |
| `HANDOFF/done/[VALIDATED]_P0_BUILD_002_Workspace_Unification_And_Android_Build_Setup.md` | `blocked-by-metadata` | `gate-fail` | `b6c6afa911a414d79cc2a6da156c8ea07e9c64796bb03baab2e36d6b69dc7ebe` |
| `HANDOFF/done/[VALIDATED]_P0_CORE_001_Drift_Protocol_Completion.md` | `blocked-by-metadata` | `gate-fail` | `a2a4bac6bda8e873668f3f16978b01ac021afe99f9c981291ad4d280dbb78962` |
| `HANDOFF/done/[VALIDATED]_P0_IDENTITY_001_Unified_ID_System.md` | `blocked-by-metadata` | `gate-fail` | `e968899abd14cfaeef99aaaa84a9f0b85942ad5a682424cf2633a8bf945a7285` |
| `HANDOFF/done/[VALIDATED]_P0_TRANSPORT_001_CLI_Android_LAN_Unification.md` | `blocked-by-metadata` | `gate-fail` | `69cceb92d647a880fb83e213eed4397e61cf82fff3442f45367d1baaaa6280bc` |
| `HANDOFF/done/[VALIDATED]_P1_ANDROID_022_BLE_Stale_Cache_Cleanup.md` | `blocked-by-metadata` | `gate-fail` | `78b971f7fe9abdc3d236ae909df2fc828216c6b91b913c984e5414faa083423d` |
| `HANDOFF/done/[VALIDATED]_P1_ANDROID_023_History_Persistence_Regression_Test.md` | `blocked-by-metadata` | `gate-fail` | `0c62bd39e4dccc8c1460f30023f9d5814af423556fe5a43bf9eb298bd53e2146` |
| `HANDOFF/done/[VALIDATED]_P1_ANDROID_DIAGNOSTICS_EXPORT_001.md` | `blocked-by-metadata` | `gate-fail` | `8ecbe0b5654b34e15ab33869242bbdc5bb8719091bd54df69a0c0ab076cb687e` |
| `HANDOFF/done/[VALIDATED]_P1_ANDROID_Identity_Generation_From_Settings_Missing_Entropy_And_Hangs_30s.md` | `blocked-by-metadata` | `gate-fail` | `59f5b7cf66c96a200e84ce46fa38c9e95b7bb3a82921256ab896c4753a516ec9` |
| `HANDOFF/done/[VALIDATED]_P1_BUILD_003_Gradle_Rust_Android_Build_Parallelization_And_Incremental_Fix.md` | `blocked-by-metadata` | `gate-fail` | `3c01ed5b77a6d066748beb9dd6e50a7e799a5592e3acd5dfffc70cb01dcfb019` |
| `HANDOFF/done/[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses.md` | `blocked-by-metadata` | `gate-fail` | `1425f5f2570c4d8cef5c7ef0ae2d62636715adf203918426ef8cae0545721737` |
| `HANDOFF/done/[VALIDATED]_P1_GEMINI_FLASH_003_BLE_Stale_Peer_Cache_Cleanup.md` | `blocked-by-metadata` | `gate-fail` | `0c32edbb063f0daeedd28272ecb0ac725e075fb685386248e30a022652b08c43` |
| `HANDOFF/done/[VALIDATED]_P1_GEMINI_FLASH_012_Remove_Unused_Arc_Import_WASM.md` | `blocked-by-metadata` | `gate-fail` | `0d2fe98b26f7de3e24edfe812a8664207691e48a2babc1238b36e81ddef1d770` |
| `HANDOFF/done/[VALIDATED]_phase_3_security_hardening.md` | `blocked-by-metadata` | `gate-fail` | `1bf3a43f126eea287db24b48084f03c347368b861b3ca00eb988d5181dc5bcd0` |
| `SCM-Q-8aca1752c927` (path hash `8aca1752c92760f0c9e296ca325cf18168814c4c9750b4ac5ae3fe1450346d3c`) | `quarantined` | `foreign-alias-in-bytes` | `5f53289b9794afa05591fb7fb57cc3cb8830de8378e3e60dbfc17ca12cf9e36d` |
| `HANDOFF/done/[VALIDATED]_task_fire_drill_audit.md` | `blocked-by-metadata` | `gate-fail` | `d28e1347c5a0d62f494762aacb25aa917a63832e7b7d6dac01cb8f553ee555fa` |
| `HANDOFF/done/[VALIDATED]_task_p0_android_play_readiness.md` | `blocked-by-metadata` | `gate-fail` | `988ceee644e50055c2c1590bae8b21d9d1c507716c2c2eddf4bd423d3c1e4d5c` |
| `HANDOFF/done/[VALIDATED]_task_p0_deprecation_api_migration.md` | `blocked-by-metadata` | `gate-fail` | `43fda3ea8a1023899d4f95f565eb4e26d98235e1f28c145b4a079e539e902543` |
| `SCM-Q-78568524d693` (path hash `78568524d6934bd22035aad94415754f0fd497b14a041492a560cc36aa1a946a`) | `quarantined` | `foreign-alias-in-path-and-bytes` | `fec4cba38a6435e15f34e01c5b39c691ab08b51c7b54615cfa904885a7b182f2` |
| `HANDOFF/done/[VALIDATED]_task_p0_theme_regression_fix.md` | `blocked-by-metadata` | `gate-fail` | `2f77aca0eb2a498a3a28f78d383d12b3266175fdc1d63c047bfa16f71ff594c0` |
| `HANDOFF/done/[VALIDATED]_task_p1_android_hardening.md` | `blocked-by-metadata` | `gate-fail` | `30ade6962ad5515961f3c9c96b19b368ad0d9fac22fb4f2e0af3a835a38a3917` |
| `HANDOFF/done/[VALIDATED]_task_p1_android_hardening_remnants.md` | `blocked-by-metadata` | `gate-fail` | `d4513d1f1bfba48c4e2efe719f7132a076e285f3ec8e5de1c737597d6402d34a` |
| `HANDOFF/done/[VALIDATED]_task_p1_ironcore_placeholders.md` | `blocked-by-metadata` | `gate-fail` | `732d455188a89eb94afd971aadcdb1dfa9e284d87575d6041927f0361ca13a29` |
| `HANDOFF/done/[VALIDATED]_task_p1_meshvpn_enable.md` | `blocked-by-metadata` | `gate-fail` | `0b070f57482b0168cd43dce107054fe18f02f6918739c423e6d050c95d720e83` |
| `HANDOFF/done/[VALIDATED]_task_p1_multidevice_blocking.md` | `blocked-by-metadata` | `gate-fail` | `d515caac9ee5be1735a76864ae525b94fca8e35b61ec3bd7e2b9e2af81fd8139` |
| `HANDOFF/done/[VALIDATED]_task_p1_mycorrhizal_routing_activation.md` | `blocked-by-metadata` | `gate-fail` | `fd03553f117caf5a6fb1647bc3e675aafe4be990a6794d46040d6e47d89fa3c0` |
| `HANDOFF/done/[VALIDATED]_task_p1_network_detector_debounce.md` | `blocked-by-metadata` | `gate-fail` | `d9b12f721a130286260427c844f3a6cdae3224b6942bfd10f36443e9dafb0696` |
| `HANDOFF/done/[VALIDATED]_task_p1_nickname_datastore_fallback.md` | `blocked-by-metadata` | `gate-fail` | `9ffd0ca231179391a76168db23ed75009031dbeb7382397b46fca7ec57494137` |
| `HANDOFF/done/[VALIDATED]_task_p1_wasm_jsonrpc_expansion.md` | `blocked-by-metadata` | `gate-fail` | `1d15d1caf2097e5334b2df3111d8077a1f5caf070fe9bbb2a1c4456fd216a7c5` |
| `HANDOFF/done/[VALIDATED]_task_p1b_notification_channel_dedup.md` | `blocked-by-metadata` | `gate-fail` | `89ff68acdf4f4fc24971a0e1e80db889d99c2b0af6c05650685af9a0ee07dc1e` |
| `HANDOFF/done/ios-compile/IOS_COMPILE_FIX_2ERRORS.md` | `blocked-by-metadata` | `gate-fail` | `dbb8a4fe930472c7f485d63c0106e9e7e228f3685e70d9dffb91fbeaa0c01cd0` |
| `HANDOFF/done/ios-compile/IOS_COMPILE_FIX_MDNS.md` | `blocked-by-metadata` | `gate-fail` | `57293e82faea63109c72c105ae60b245eccdbf4caebb4f8e6175f47cc4445804` |
| `HANDOFF/done/phase_1a_compilation_baseline.md` | `blocked-by-metadata` | `gate-fail` | `36e8e8648709818dd29eac2b855a62bd2624e363e62e86aff1d0c7b83f73ac9b` |
| `HANDOFF/done/phase_1b_core_module_wiring.md` | `blocked-by-metadata` | `gate-fail` | `ef7d62a418e7b0450944bf8a45a0ff390050b3ea5924fbaaa8d72e456231749c` |
| `SCM-Q-ec1e039fb72c` (path hash `ec1e039fb72c419c0c4018ede6138b6caae423e632e243ad35d33e85ecd7cf1f`) | `quarantined` | `foreign-alias-in-bytes` | `e6a8b76c726fa11a34a5ab726e274eccabf1e8300c4848cd5e0ed2c292a50c90` |
| `HANDOFF/done/phase_4_documentation_polish.md` | `blocked-by-metadata` | `gate-fail` | `d4160fef953ecb4d083af065f88d1766f14b286aefcbb9a4c389168c89f09e97` |
| `HANDOFF/done/phase_full_wiring_verification_2026-04-28.md` | `blocked-by-metadata` | `gate-fail` | `82ab09976c465711ad272bf00b49025c93445d3daea62c835762f1debbbeb98b` |
| `HANDOFF/done/task_000_MASTER_AUDIT.md` | `blocked-by-metadata` | `gate-fail` | `285fdca232c5eba9854ebce104348dac4b9710100ed4d92a24123bbdb339e9d6` |
| `HANDOFF/done/task_000_MASTER_AUDIT_REPORT.md` | `blocked-by-metadata` | `gate-fail` | `ead034c68bec5f7b7061dcf4cbd18a62af56c346b306274740d64f4618be9fb9` |
| `HANDOFF/done/task_P0_SWARM_TRIAGE_AND_LEDGER.md` | `blocked-by-metadata` | `gate-fail` | `7fb6b550bb0083855a7e2aa9e24e2b590d313229065ba967eb76556ef9d03086` |
| `HANDOFF/done/task_agent_ironcore_stub.md` | `blocked-by-metadata` | `gate-fail` | `fc4abcb696d27ea3506b9cf2970d5fec308fccf7f09dbb89504fbafed4aa41a6` |
| `HANDOFF/done/task_android_build_verify.md` | `blocked-by-metadata` | `gate-fail` | `3d3d58a1d2518c71a3df8851b5664b6a1ffb126cf2e633d352db86465645bbd2` |
| `HANDOFF/done/task_android_full_fix.md` | `blocked-by-metadata` | `gate-fail` | `747aadf325a1b562186dcdbef5638c91e348626df58efe535a290cdb841a06ab` |
| `HANDOFF/done/task_cli_swarm_stats.md` | `blocked-by-metadata` | `gate-fail` | `822386c2731451bf6c7e8b08d013f5db1831b8f8ae64f171fd7b354bdc89de96` |
| `HANDOFF/done/task_fix_windows_linker.md` | `blocked-by-metadata` | `gate-fail` | `25c6ee333fa2b8b7e7c8dcfce6779773339a55bcc857088dfc5b6394d05742d0` |
| `HANDOFF/done/task_security_tooling.md` | `blocked-by-metadata` | `gate-fail` | `46ac4f92c0eb7b893b08bc6803bb11f4107f334fbd37b85bdd247c9d579eb917` |
| `HANDOFF/done/task_test_compile_verify.md` | `blocked-by-metadata` | `gate-fail` | `8548da9de3ca5dd1870b9ade6941208bd2a9ce5b3c82dc9d453f7bbafec8ff2b` |
| `HANDOFF/done/task_verify_uniffi_bindings.md` | `blocked-by-metadata` | `gate-fail` | `2a011ab4395cc4c692729a2689559736a32996da7b0510d58168accc9cfea1bd` |
| `HANDOFF/done/task_wire_ContactDetailScreen.md` | `blocked-by-metadata` | `gate-fail` | `da2b5269411fa8532179241ebed0b897642b12602790af2a4ba6fce3a8c1a87d` |
| `HANDOFF/done/task_wire_ErrorState.md` | `blocked-by-metadata` | `gate-fail` | `fc198109211636aa8dc75cc637539722a9771186acf2890742ec5fd71d33317f` |
| `HANDOFF/done/task_wire_IdenticonFromHex.md` | `blocked-by-metadata` | `gate-fail` | `59518eca8bca182126c2bdf993c0d8de41ca26a3ee114750ab9104167f664962` |
| `HANDOFF/done/task_wire_InfoBanner.md` | `blocked-by-metadata` | `gate-fail` | `d1183cf12481d5f1ebeb95c8e280a754826eae692ffe2bb8fa1a04ccb608081f` |
| `HANDOFF/done/task_wire_LabeledCopyableText.md` | `blocked-by-metadata` | `gate-fail` | `afc7c1cc0e7f05bf450da5dcb6d7f8d1ab47e461f03e5c47a8938e8a62f7abf3` |
| `HANDOFF/done/task_wire_MeshSettingsScreen.md` | `blocked-by-metadata` | `gate-fail` | `4d9a1d8d66686e51cce50d1b6d771ff2292e52cf0b06da3d0ec4287256c31a7e` |
| `HANDOFF/done/task_wire_MessageInput.md` | `blocked-by-metadata` | `gate-fail` | `d676e2e4ba043b422d08d6096e73c39d892f3e7005ed1b51bdf95b6933fffe9c` |
| `HANDOFF/done/task_wire_PeerListScreen.md` | `blocked-by-metadata` | `gate-fail` | `edffb85b18a175e8aba92bfb88bd5e4d382cea448a75e50915d9f860ebe04616` |
| `HANDOFF/done/task_wire_PowerSettingsScreen.md` | `blocked-by-metadata` | `gate-fail` | `d9c382dacadae7b7b37845364949216fb3868eb3e8bfdf15508ff1c189ecf859` |
| `HANDOFF/done/task_wire_TopologyScreen.md` | `blocked-by-metadata` | `gate-fail` | `82931d2aac289be83cb13be47aa5852977bdb089382e69417980ebab36604570` |
| `HANDOFF/done/task_wire_TruncatedCopyableText.md` | `blocked-by-metadata` | `gate-fail` | `68d2c7af44e1ef6bab9ab39f3251ee610ad4f70baca6f0d330d1ffac42b542d4` |
| `HANDOFF/done/task_wire_WarningBanner.md` | `blocked-by-metadata` | `gate-fail` | `a3a478341ae90cb0c30803ea3645f2d4b2abdfaffaa5192e3b69f837d8c4c4e7` |
| `HANDOFF/done/task_wire_abusive_peer_burst_is_rate_limited_but_other_peer_still_passes.md` | `blocked-by-metadata` | `gate-fail` | `389df1edb42f52b82faaffbd56a0763553a3c1feb04461022da0bc5e3c700174` |
| `HANDOFF/done/task_wire_acquireWakeLock.md` | `blocked-by-metadata` | `gate-fail` | `76250abdadf1b7aa079989577e5c8951526064ad87be20fc629c28206a585105` |
| `HANDOFF/done/task_wire_active_paths.md` | `blocked-by-metadata` | `gate-fail` | `477f368436d7c12e8231adc45f49338ebfd60d8094e53eaa45a4d1aba2cc9c73` |
| `HANDOFF/done/task_wire_add_discovered_peer.md` | `blocked-by-metadata` | `gate-fail` | `348932847d25be2c96db7327862cf32c1d5920a097e5ca83c6393d1c159f1b50` |
| `HANDOFF/done/task_wire_add_kad_address.md` | `blocked-by-metadata` | `gate-fail` | `29181433a8533a3da928c814c7333e93f5fc0322182c5ea496d33b7739115d50` |
| `HANDOFF/done/task_wire_add_rtc_connection.md` | `blocked-by-metadata` | `gate-fail` | `c663e5b06dbae30b5505628fe03bcff43b7b7441c64593871b5df14a9883579b` |
| `HANDOFF/done/task_wire_add_step.md` | `blocked-by-metadata` | `gate-fail` | `ba34739e8022f47be3fc18edad554bc73a1fcb8712256d8b017e0fc0e02bb78b` |
| `HANDOFF/done/task_wire_add_websocket.md` | `blocked-by-metadata` | `gate-fail` | `83e63c4935cef5ddae11a77c0ae430a068f2a6aee0ebe7391f8f07e540dd4427` |
| `HANDOFF/done/task_wire_advertise_service.md` | `blocked-by-metadata` | `gate-fail` | `7c48d295111740370c5abcfa36e15b660c3e41e8346466a46a283e5072f98d64` |
| `HANDOFF/done/task_wire_all_connections.md` | `blocked-by-metadata` | `gate-fail` | `1fb0e6c8fea9b17fe9fea3956bec819957137ef8476ee18a2765c9bdabf46b25` |
| `HANDOFF/done/task_wire_android_ui_B4.md` | `blocked-by-metadata` | `gate-fail` | `2164e64ff471d7d218859effa6942cf3733725d2a25a44e589e2c521848a0b4e` |
| `HANDOFF/done/task_wire_annotate_identity.md` | `blocked-by-metadata` | `gate-fail` | `7077cc336fc972ec47f66ee2869cc09cb5f2bbf7f1705fe4507bcaf256ae6ca8` |
| `HANDOFF/done/task_wire_applyAdvertiseSettings.md` | `blocked-by-metadata` | `gate-fail` | `884723382114314d0823c4a87aeb8d033a29be6799762948cf2c096d90359137` |
| `HANDOFF/done/task_wire_applyScanSettings.md` | `blocked-by-metadata` | `gate-fail` | `0c580051b15bff3755a8e7dc195c0f28f51bebd1838bc14d81e8ac4199433347` |
| `HANDOFF/done/task_wire_apply_policy_config.md` | `blocked-by-metadata` | `gate-fail` | `9ae7f3c51804b5db297ae422ea485a064d1aded6d17d64e390c49c07e3b62d4c` |
| `HANDOFF/done/task_wire_attemptBleRecovery.md` | `blocked-by-metadata` | `gate-fail` | `ef0a78449a914b32fbc1920f1a2acf05381e6938c29731439de4e7585f022804` |
| `HANDOFF/done/task_wire_audit_count.md` | `blocked-by-metadata` | `gate-fail` | `3bac9c1667537aaea0e71d54dceaf9e906cb6f3855f13fcce18663b263180d09` |
| `HANDOFF/done/task_wire_autoSubscribeToPeerTopics.md` | `blocked-by-metadata` | `gate-fail` | `7f6c53f9fbff9590b8732daaf7bb21620183340055373cc4a9a9b952253fed7e` |
| `HANDOFF/done/task_wire_best_relays.md` | `blocked-by-metadata` | `gate-fail` | `46247807773727f754023f9241330d814610700a3cb90a044248753b252050bd` |
| `HANDOFF/done/task_wire_blake3_hash.md` | `blocked-by-metadata` | `gate-fail` | `141e3f99c59541d6715583b29a81af05649cfc84b0dff8a673b1c862a0f1895f` |
| `HANDOFF/done/task_wire_blocked_only_peer_ids.md` | `blocked-by-metadata` | `gate-fail` | `79b5ecdcede92717ac728a408a21714a41fc6167564b7488fb71936b137ebdc7` |
| `HANDOFF/done/task_wire_buildForegroundServiceNotification.md` | `blocked-by-metadata` | `gate-fail` | `0504a08dcc0584df3d39465fffa3dcf55bf84441e070381e4a911adea9c2869a` |
| `HANDOFF/done/task_wire_build_optimization_pipeline.md` | `blocked-by-metadata` | `gate-fail` | `23f8f6a241c7540d069c4641544dceba320d8321a86fc4464f8bfe533bcc8920` |
| `HANDOFF/done/task_wire_build_security_audit_pipeline.md` | `blocked-by-metadata` | `gate-fail` | `e0e4e4eb52c709269529a22d6d8c319b8ab42e8430b6a5cc5abcbca8169f740e` |
| `HANDOFF/done/task_wire_calculate_dynamic_ttl.md` | `blocked-by-metadata` | `gate-fail` | `8a215051faaaec206706436cee79a3f73ce97f54c8a60851ad322b8bd1b3cbfa` |
| `HANDOFF/done/task_wire_can_bootstrap_others.md` | `blocked-by-metadata` | `gate-fail` | `f3462c0c02ded8450be6d1ff8852de92122e4d6a1c3098955c59d7c9a96c8c0a` |
| `HANDOFF/done/task_wire_can_forward_for_wasm.md` | `blocked-by-metadata` | `gate-fail` | `c49a340059ee5152b21b0cdf33840dbb2c5712357d6c9b8c967355732e1ecf52` |
| `HANDOFF/done/task_wire_can_reach_destination.md` | `blocked-by-metadata` | `gate-fail` | `9db8c8106366618a0fe697c10699141dd3c74114eb29e7ff825e184b7b03c243` |
| `HANDOFF/done/task_wire_chain_ratchet_produces_distinct_keys.md` | `blocked-by-metadata` | `gate-fail` | `c7460a7a5fb0b18c0cc8b3eb2669dd928f60e9e2dd2ed13327a8d3f89f92e288` |
| `HANDOFF/done/task_wire_cheap_heuristics_reject_invalid_payload_shapes.md` | `blocked-by-metadata` | `gate-fail` | `86e3400fdaa52a957e46677f156af864e8d67e01d580d513b9589dc9e170e155` |
| `HANDOFF/done/task_wire_checkAndRecordMessage.md` | `blocked-by-metadata` | `gate-fail` | `1697847a2dedb07654679cc444b19a5f76d8635363e69faa759d1abd384094b7` |
| `HANDOFF/done/task_wire_cleanup_stale_connections.md` | `blocked-by-metadata` | `gate-fail` | `43c570f453dd6e64841734de65c631eceb18cc070cdbf852068c61f7740f3577` |
| `HANDOFF/done/task_wire_clearAllHistory.md` | `blocked-by-metadata` | `gate-fail` | `4b0ad25acb32c2f40ce9efdc2aa44fec208e95e1388f404197de20273c9f6017` |
| `HANDOFF/done/task_wire_clearAllRequestNotifications.md` | `blocked-by-metadata` | `gate-fail` | `e86a43a14d792eb9e5477b55fd100cf835bb494436690f6efbb0f954cb6075b0` |
| `HANDOFF/done/task_wire_clearAnrEvents.md` | `blocked-by-metadata` | `gate-fail` | `fa8dded7d124c0928d2b1273cc29ce46954a4c4a769e8b1649b8f7a5b1f1687e` |
| `HANDOFF/done/task_wire_clearInput.md` | `blocked-by-metadata` | `gate-fail` | `470480bcd289f66217f2564c622fb8fd010bf3d18835ebaa682ae27366fd8f3e` |
| `HANDOFF/done/task_wire_clearMessageNotifications.md` | `blocked-by-metadata` | `gate-fail` | `ae9d16920bdfb4a50f4e55d99ef257607f0b3e45d917a49025e1d104929205ff` |
| `HANDOFF/done/task_wire_clearPeerCache.md` | `blocked-by-metadata` | `gate-fail` | `f5baa96b61bf03fae3e629323a995d1c0276e2548b3bfe54ee982bcea52a2676` |
| `HANDOFF/done/task_wire_clearSearch.md` | `blocked-by-metadata` | `gate-fail` | `4f3cd3015acdd797079282d22f861aaed6c0d1560ddfe4fead96f02c1e73daca` |
| `HANDOFF/done/task_wire_clear_unreachable_peer.md` | `blocked-by-metadata` | `gate-fail` | `36162aaa2d4b48197873f6e611f0017000c81226e74023914407c98c08515a3a` |
| `HANDOFF/done/task_wire_close_all_notifications.md` | `blocked-by-metadata` | `gate-fail` | `11f09ce116cc7a268682cf23150dd760aeef336e851f118eaba7701c4489d2eb` |
| `HANDOFF/done/task_wire_compute_ble_adjustment.md` | `blocked-by-metadata` | `gate-fail` | `95147282a54ceda3f7903ef008b04568e42ab0fed6550457af437d23e369531c` |
| `HANDOFF/done/task_wire_compute_relay_adjustment.md` | `blocked-by-metadata` | `gate-fail` | `c87fd69a2781f297fc2161174d86611ba851d4913053be669b39dbe8110eca06` |
| `HANDOFF/done/task_wire_contact_new_has_no_last_known_device_id.md` | `blocked-by-metadata` | `gate-fail` | `d7091a340279e5ab3c3985b2c0258cfd9c256b9363d8945ceeac28d6e2645ed6` |
| `HANDOFF/done/task_wire_contact_roundtrips_through_serde_with_default_device_id.md` | `blocked-by-metadata` | `gate-fail` | `4578c3425ea97a660f7957eb03a30bfc35f62c951825ae0ffc0cf31710a55b1d` |
| `HANDOFF/done/task_wire_converge_delivered_for_message_removes_matching_pending_records.md` | `blocked-by-metadata` | `gate-fail` | `2288003fd2a160354d3bc75df459f0e22a13b60eec41f54ae13c327256db5cee` |
| `HANDOFF/done/task_wire_convergence_marker_accepts_when_custody_exists_locally.md` | `blocked-by-metadata` | `gate-fail` | `7df9db2448c4165739d157471b679aaf3681e018fb600f8147fd5e6e6269f3c5` |
| `HANDOFF/done/task_wire_convergence_marker_rejects_invalid_shape.md` | `blocked-by-metadata` | `gate-fail` | `31df5956db367137a5ead71c46031563f1207de9221a50a211cca99b409db79a` |
| `HANDOFF/done/task_wire_convergence_marker_requires_local_tracking_context.md` | `blocked-by-metadata` | `gate-fail` | `f0c720dc46017e236bca5039bcb4508ef429e0794f3f6a11f8b65ae087f450fa` |
| `HANDOFF/done/task_wire_count_with_peer.md` | `blocked-by-metadata` | `gate-fail` | `8a038c9749c197a5a382f8171b2cdc9df87c3a735081abf6f3eaf8b8878f0dc3` |
| `HANDOFF/done/task_wire_create_basic.md` | `blocked-by-metadata` | `gate-fail` | `693e57208d05b5b375c59d1370dba23d1fc86b4d6b727386bc0d831bd3843637` |
| `HANDOFF/done/task_wire_create_cot.md` | `blocked-by-metadata` | `gate-fail` | `e80adf04efe6d09673f76cbe539f8b07e5d30d0c9088184f6ac10b04db53e7ba` |
| `HANDOFF/done/task_wire_create_multihop.md` | `blocked-by-metadata` | `gate-fail` | `e7002756eb6144bf52234568b3fe39dc8aaf357e42baafa4c55ccbe338f759b4` |
| `HANDOFF/done/task_wire_create_optimizer.md` | `blocked-by-metadata` | `gate-fail` | `a2c1387fa5ffbfecae87de5ee59cb123b16cffa9fc1d3fddedf95334b243076b` |
| `HANDOFF/done/task_wire_create_receiver_session.md` | `blocked-by-metadata` | `gate-fail` | `2155837dab12ebc32559e108eaa6f54f2a2296aa8d9bc9c9a0fcdf8be11fd9eb` |
| `HANDOFF/done/task_wire_currentCount.md` | `blocked-by-metadata` | `gate-fail` | `6a8b99b36067c24b77718603fbbd4d35127c45ee0d04091a0721a5e459e181c1` |
| `HANDOFF/done/task_wire_current_discovery_phase.md` | `blocked-by-metadata` | `gate-fail` | `a407a725d6e87ae58e336a9e90f6b3c764298b65cef78f3f3ca62a28bddc4943` |
| `HANDOFF/done/task_wire_custody_audit_persists_across_restart.md` | `blocked-by-metadata` | `gate-fail` | `d277767c22a9cfec3bc447bd3e7bc2f6bb57640cb83c3148adb04cd9f9e1088f` |
| `HANDOFF/done/task_wire_custody_deduplicates_same_destination_and_message_id.md` | `blocked-by-metadata` | `gate-fail` | `f78923088762cf612e008ec05c4da90c20c7d60121d399df7e0f80dc15d48975` |
| `HANDOFF/done/task_wire_custody_transitions_are_recorded.md` | `blocked-by-metadata` | `gate-fail` | `bf4672f6b6b5c98843e6011d0fc645de8ceb7dcb8d3c0b28d9c4420e4ee261b0` |
| `HANDOFF/done/task_wire_decode_rejects_short_buffer.md` | `blocked-by-metadata` | `gate-fail` | `b08af1f832d294d88d97093d02cdce8adf32cead41f74c3ae18c41c815df8807` |
| `HANDOFF/done/task_wire_default_settings.md` | `blocked-by-metadata` | `gate-fail` | `0ca3897453c397eebe48118064531628202a324a38d25f39d7755fb69cf1ee75` |
| `HANDOFF/done/task_wire_derive_key_always_32_bytes.md` | `blocked-by-metadata` | `gate-fail` | `7e50f9639a555b64fb90e313658a92f85f10fd73ca257d7c7ad5e80c3605d359` |
| `HANDOFF/done/task_wire_detect_browser.md` | `blocked-by-metadata` | `gate-fail` | `5d32c0461326e0c91e836f1076b5bcaf059684b373bf7d2403b432a186088389` |
| `HANDOFF/done/task_wire_disableTransport.md` | `blocked-by-metadata` | `gate-fail` | `2cf87c4253202e0b668837b1dac91eda872425f87e20d5431c8d555b60b90680` |
| `HANDOFF/done/task_wire_disable_location_background.md` | `blocked-by-metadata` | `gate-fail` | `a40808e5bb80812e88fda90dfce84137f044307480c46ff8d81fc3064d87695e` |
| `HANDOFF/done/task_wire_disabled_notifications_suppress_delivery.md` | `blocked-by-metadata` | `gate-fail` | `f85a354d927e98edc218b94a24bebe4659ee3e4e70c037374aee97da8badfe02` |
| `HANDOFF/done/task_wire_drain_received_messages.md` | `blocked-by-metadata` | `gate-fail` | `435999d7883690a3058744d041600739e03cb0d3baf1707e98b86ce2079eb05a` |
| `HANDOFF/done/task_wire_drift_activate.md` | `blocked-by-metadata` | `gate-fail` | `f11c0db5ea3c6868404a16d39eac0fecbc3a2271ebb0cfd2a3eb2d467fcf546d` |
| `HANDOFF/done/task_wire_drift_deactivate.md` | `blocked-by-metadata` | `gate-fail` | `fb0f46777777ecd823b9b80d27c3e28f390268446a481bf51f01532e0d52970d` |
| `HANDOFF/done/task_wire_drift_network_state.md` | `blocked-by-metadata` | `gate-fail` | `5daf8f5fd4b71c6d24b552beb7244cba43385cd55d3c81a1f8429fd3ae0954fa` |
| `HANDOFF/done/task_wire_drift_store_size.md` | `blocked-by-metadata` | `gate-fail` | `7f07477685731f5af5e29abf234352604c20d7d0e961d4c326645e9182372ba3` |
| `HANDOFF/done/task_wire_duplicate_window_suppresses_immediate_replay_then_expires.md` | `blocked-by-metadata` | `gate-fail` | `a2a8b9f0f0c0b09927cbb146eae2ed94fc9b03ad254c47b1e1711255b1ef6e38` |
| `HANDOFF/done/task_wire_duplicates_are_suppressed.md` | `blocked-by-metadata` | `gate-fail` | `4e42b98c91925f5c1d7afc220c9e5a8a20e7a29869e8a67a7c9dc6fd458427a0` |
| `HANDOFF/done/task_wire_ed25519_conversion_produces_32_bytes.md` | `blocked-by-metadata` | `gate-fail` | `931718e87fc78c522185076e33ebf0af67b521dc4c3d18f5ba369f7bfdd83ed6` |
| `HANDOFF/done/task_wire_emergency_recover.md` | `blocked-by-metadata` | `gate-fail` | `4ab454c7ab1d4aa7b16d932311f3d5d7beac0dcd86d9d35672b215ba85087ae6` |
| `HANDOFF/done/task_wire_enableTransport.md` | `blocked-by-metadata` | `gate-fail` | `9d7b20c97f6cab1cd9f0a9a0baa72497335888debd4c83edb889fe426dbf6299` |
| `HANDOFF/done/task_wire_encrypt_xchacha20.md` | `blocked-by-metadata` | `gate-fail` | `86a2548fbbf6fbb3e997a8427392aba0ca179a7259778743a7be21f14e2bf6f6` |
| `HANDOFF/done/task_wire_evaluate_all_tracked.md` | `blocked-by-metadata` | `gate-fail` | `424e2f659f139036d627709114a2f06837107c4f460024b2e652c7ca5fd526d5` |
| `HANDOFF/done/task_wire_expire_old_observations.md` | `blocked-by-metadata` | `gate-fail` | `f28a6049979004f794ea437dc52aa2d1df52587ce4bbb68178f4539298002424` |
| `HANDOFF/done/task_wire_explicit_request_overrides_known_contact_inference.md` | `blocked-by-metadata` | `gate-fail` | `6b68f2c328a89620cd418ac1e22224d1390dc10ed7a92c9439f394043e38e40d` |
| `HANDOFF/done/task_wire_exportDiagnosticsAsync.md` | `blocked-by-metadata` | `gate-fail` | `bdc4b901c0a07d4bfd53e556e4d4fa9a368986842862e4b46a0dafabce0a06bb` |
| `HANDOFF/done/task_wire_export_audit_log.md` | `blocked-by-metadata` | `gate-fail` | `e33726231554c534c1a1efb999b547b976dc66c25c08c237e91e4cf13e0a407f` |
| `HANDOFF/done/task_wire_federated_nickname.md` | `blocked-by-metadata` | `gate-fail` | `c7902db8db5dcfeb38a08ef9d2f504234150514548db2184fc5cbe3600a787a9` |
| `HANDOFF/done/task_wire_filterMessagesByTopic.md` | `blocked-by-metadata` | `gate-fail` | `b79607e0c606262f0658481b5cf8747433d61114ae8ee5de2543ab9350d0a480` |
| `HANDOFF/done/task_wire_find_by_nickname.md` | `blocked-by-metadata` | `gate-fail` | `fdc80e9b2d000b49e9ecc41cc4b892ffe0e5ae6e35f9fe01ae5e44d5608836fb` |
| `HANDOFF/done/task_wire_find_by_public_key.md` | `blocked-by-metadata` | `gate-fail` | `8eb57fd2b5411f94bdffba08e5a41884466a7762fd3831a341e99ad9dfab4100` |
| `HANDOFF/done/task_wire_for_local_peer_prefers_explicit_custody_dir_override.md` | `blocked-by-metadata` | `gate-fail` | `41f2cb809bd16d54489f1c4e944af87ab63ba7cf177d15b4cb801ead1ab80976` |
| `HANDOFF/done/task_wire_forceRestartScanning.md` | `blocked-by-metadata` | `gate-fail` | `2fe5a9198852bf3b74e0acf1c7f6b4267ad86db698ca109e32450cae7d2bde39` |
| `HANDOFF/done/task_wire_force_ratchet.md` | `blocked-by-metadata` | `gate-fail` | `d6333ca426b5d83be9121b9ba8d5f1ed09401deb864b97155e7838202da82fb8` |
| `HANDOFF/done/task_wire_foreground_direct_messages_follow_foreground_toggle.md` | `blocked-by-metadata` | `gate-fail` | `4e5a3aaaba616c67e28ea2c27f1bca8bd088e634958729b11190ef1251eb9d83` |
| `HANDOFF/done/task_wire_formatReportForUser.md` | `blocked-by-metadata` | `gate-fail` | `3efa733b1d0a05ec1a493105d2d5ab094dfa47ceae560b7c43272cc2fac3df74` |
| `HANDOFF/done/task_wire_formatted_time.md` | `blocked-by-metadata` | `gate-fail` | `4c85bd16ef41bdad64ca8e05fc1e7c584e71d735a6a76c4f80e896b14d282761` |
| `HANDOFF/done/task_wire_fromValue.md` | `blocked-by-metadata` | `gate-fail` | `6613c8844f469d7119d65cf8fd02cbb6b8143d2ea36010695fe625f2f9fd0e22` |
| `HANDOFF/done/task_wire_generate_cover_traffic_if_due.md` | `blocked-by-metadata` | `gate-fail` | `ac0ce28facb0bd4d50b741eefc430b46a744e5920ddad3ffb461bbd8d7022385` |
| `HANDOFF/done/task_wire_getActiveTransports.md` | `blocked-by-metadata` | `gate-fail` | `05951e8e88f2d976d076fa00fbe2e382f7c6a48cb64f07d05442f1b1fb9ac8d8` |
| `HANDOFF/done/task_wire_getAllAnrEvents.md` | `blocked-by-metadata` | `gate-fail` | `3b9820fb4867d623ff41f3a25018da328492ae51ab9e5f99488853052313b70f` |
| `HANDOFF/done/task_wire_getAnrStats.md` | `blocked-by-metadata` | `gate-fail` | `826b70c25febbac57814ffbf98c23bb0feb97d565f0c235899c3a773109056db` |
| `HANDOFF/done/task_wire_getAvailableTransports.md` | `blocked-by-metadata` | `gate-fail` | `e5459f9b301934c06e41cd8181a797f1ea52da65f35455cc7944911200cc1d5a` |
| `HANDOFF/done/task_wire_getAvailableTransportsSorted.md` | `blocked-by-metadata` | `gate-fail` | `d1bd2801aa3e91dbe4d4433645b6873014ad1c7d2b61a8dc065264e6d6539062` |
| `HANDOFF/done/task_wire_getBlockedCount.md` | `blocked-by-metadata` | `gate-fail` | `d68294df2eddebe51712a5435fb32d28abea8b1db2de71f221f3fb1d70da118f` |
| `HANDOFF/done/task_wire_getBootstrapNodesForSettings.md` | `blocked-by-metadata` | `gate-fail` | `85720b6313322da949e21b3658477d36d7821f468bf20452993df21b3b0193a6` |
| `HANDOFF/done/task_wire_getDedupStats.md` | `blocked-by-metadata` | `gate-fail` | `496081d1b2437543238450fed89161aa666b749ef33a32be2b9849578fcdac89` |
| `HANDOFF/done/task_wire_getHealthStatus.md` | `blocked-by-metadata` | `gate-fail` | `ba7d4ce330203bc740b2f0810af469dfdfc79ead744c9b9c49cc9c1126b3562d` |
| `HANDOFF/done/task_wire_getHealthyRelays.md` | `blocked-by-metadata` | `gate-fail` | `e6ac814b40c9f11f61e28dcd22cde460c43bb81ae54541f55b02673ada46aa18` |
| `HANDOFF/done/task_wire_getInboxCount.md` | `blocked-by-metadata` | `gate-fail` | `fc7b7cc6f400f7a2c70467c826f65d031d653a78285a4df0a8d03e3cc756209d` |
| `HANDOFF/done/task_wire_getKnownTopicsList.md` | `blocked-by-metadata` | `gate-fail` | `dada43f925d784fe76f481cec2f30fb55286c66988a24b1438bc4468cf0f92bb` |
| `HANDOFF/done/task_wire_getLastFailure.md` | `blocked-by-metadata` | `gate-fail` | `dcc7d93b99977ed4ddf38fcbfb1b510001bf8baf72fe8285fb65fbb6876291eb` |
| `HANDOFF/done/task_wire_getLastFailureReason.md` | `blocked-by-metadata` | `gate-fail` | `d22cb78ad18e2a7eef6cd9725b883012ce10337d8fcd283abd54193780335341` |
| `HANDOFF/done/task_wire_getMessage.md` | `blocked-by-metadata` | `gate-fail` | `e37dfac9e5fa7305a67f98962b56d437a058ac6c8a659f118ae31c74e2454f01` |
| `HANDOFF/done/task_wire_getNetworkDiagnosticsReport.md` | `blocked-by-metadata` | `gate-fail` | `d29fa0d1c2c4ea5dec2c16a3ab3c756d0514644fddb9e435b7a02e13d421497e` |
| `HANDOFF/done/task_wire_getNetworkDiagnosticsSnapshot.md` | `blocked-by-metadata` | `gate-fail` | `06113800644c13bb6731330329d7287edbad87c7c85cf76cd8d1dd6bafd19f48` |
| `HANDOFF/done/task_wire_getNetworkFailureSummary.md` | `blocked-by-metadata` | `gate-fail` | `7d16a65056278fb17b48bda373fddfb6fb51c77d5b3589a4b732c667cc34e600` |
| `HANDOFF/done/task_wire_getNotificationStats.md` | `blocked-by-metadata` | `gate-fail` | `9dda55a66309f76c5f59852a506549214664a1fa88c97b6330ffb143a4fa5765` |
| `HANDOFF/done/task_wire_getOpenCircuits.md` | `blocked-by-metadata` | `gate-fail` | `60b57aa80b739985f3fee77884763459e7e1b055ce49232a3f45467cdf06bc56` |
| `HANDOFF/done/task_wire_getSubscribedTopicsList.md` | `blocked-by-metadata` | `gate-fail` | `be07bd048ea91fc207e4280a8bc3529f8342d41065a5525fe626465516134ba9` |
| `HANDOFF/done/task_wire_getTotalAnrEvents.md` | `blocked-by-metadata` | `gate-fail` | `f6ff04e88ea18eeda11ae92afdf57580e8a35010c30ff4afce171a7bac521adc` |
| `HANDOFF/done/task_wire_getTransportHealthSummary.md` | `blocked-by-metadata` | `gate-fail` | `4eecb89edc1ad4a341782fd8a768a76cd4dc5136e253928accad699757fc2025` |
| `HANDOFF/done/task_wire_get_activity.md` | `blocked-by-metadata` | `gate-fail` | `e728d7f49f0dbae3c884e6167d3fc6ef4460fde50192d2c9d8016b4d779a9d85` |
| `HANDOFF/done/task_wire_get_all_connection_stats.md` | `blocked-by-metadata` | `gate-fail` | `5b824995b8c1f28594733f4ba2079ed1c1e38f7a1fd9c7248eeff4e9239316f8` |
| `HANDOFF/done/task_wire_get_all_relay_stats.md` | `blocked-by-metadata` | `gate-fail` | `eb595ce2d13f8ca8cb4d86f1850fe502e9b894ccb784933d5ef97a21a75268f3` |
| `HANDOFF/done/task_wire_get_audit_events_since.md` | `blocked-by-metadata` | `gate-fail` | `1a495f977f0a242685d0049a0a4b387e2ab40668b79a133e5b4e5d56a4fc0620` |
| `HANDOFF/done/task_wire_get_audit_log.md` | `blocked-by-metadata` | `gate-fail` | `95ce4407d7a3a9b7c87876e9f24fa650a43f2b6e2df79b8ca1aaaa189bd26c20` |
| `HANDOFF/done/task_wire_get_available_paths.md` | `blocked-by-metadata` | `gate-fail` | `08fdc7ee4663393e80d0addb808dd96a42c7267793726ffa96176d72206a1619` |
| `HANDOFF/done/task_wire_get_best_forwarding_path.md` | `blocked-by-metadata` | `gate-fail` | `48533bbebe617a58fb6f3903b91a99a46e58c4ebc3b2e1dcb71ec0b5457607de` |
| `HANDOFF/done/task_wire_get_bootstrap_candidates.md` | `blocked-by-metadata` | `gate-fail` | `0235319e3b8a713b4948ef902f5ccc87035510c7ced9ed6beac3b58a98fb7fe7` |
| `HANDOFF/done/task_wire_get_browser_options.md` | `blocked-by-metadata` | `gate-fail` | `6179e8e50bad3330c966617e891290f301212b0b5b22fdf5595d50d4d8c2e59c` |
| `HANDOFF/done/task_wire_get_contact_manager.md` | `blocked-by-metadata` | `gate-fail` | `9cccf8dd0b1543ee5eeebe7e32257b20382f6d431989954883b6ed5f1d8cecae` |
| `HANDOFF/done/task_wire_get_daemon_socket_url.md` | `blocked-by-metadata` | `gate-fail` | `7ebc9ac4075f0ed9a668f8091deefb4a09f8c2a0025e226c7919f62586356fb3` |
| `HANDOFF/done/task_wire_get_default_settings.md` | `blocked-by-metadata` | `gate-fail` | `753b706898f207e4982eb7a4929aef7db5c9a3747ce40383b3028fc6ade7355f` |
| `HANDOFF/done/task_wire_get_enhanced_peer_reputation.md` | `blocked-by-metadata` | `gate-fail` | `4019a41d510e4574323f5de88f247a16fa3cfe5997fddf53c1095f192d4027c6` |
| `HANDOFF/done/task_wire_get_fallback_relays.md` | `blocked-by-metadata` | `gate-fail` | `1b67689efe10939f32947b00b48ba3a88ba6254154769417f918f5e8c7da20fe` |
| `HANDOFF/done/task_wire_get_forwarding_capability.md` | `blocked-by-metadata` | `gate-fail` | `b05b96dec41c4f6398a2175f5c7c8b223b137f02084e43abd73ae64f82e1ca12` |
| `HANDOFF/done/task_wire_get_healthy_connections.md` | `blocked-by-metadata` | `gate-fail` | `614512861cf9e8c81f5a595e7ac395e36bddee21770ed3e0239b4e1203c4102c` |
| `HANDOFF/done/task_wire_get_healthy_relays.md` | `blocked-by-metadata` | `gate-fail` | `2c6ecb2fc5f8c5ec9dc408bdcbd0fc5003c1464cd32e10e85b47aba7ada8b235` |
| `HANDOFF/done/task_wire_get_history_manager.md` | `blocked-by-metadata` | `gate-fail` | `7e0719052d90c793d57f6aea17a9ec8968b5ced94ffd652a60630c5061d79535` |
| `HANDOFF/done/task_wire_get_history_via_api.md` | `blocked-by-metadata` | `gate-fail` | `3dc1c59d3178913d67172ee6f158566dec6796c74d50b6622b7060c2febe17cf` |
| `HANDOFF/done/task_wire_get_hole_punch_status.md` | `blocked-by-metadata` | `gate-fail` | `5cfbc344cebf21c26277a02bffbfef52efb074847c0dc7d886c25954a3ec8ddd` |
| `HANDOFF/done/task_wire_get_identity_from_daemon.md` | `blocked-by-metadata` | `gate-fail` | `4e65fb69dfeb372514a7e89345c900513e629ccd6424119ca25c05b9b341e2ed` |
| `HANDOFF/done/task_wire_get_identity_wire_shape.md` | `blocked-by-metadata` | `gate-fail` | `2badada0df86c6f8a4a2e5f6816511a1f62a2a1e7a04a9dcbc13de5d05dec3e6` |
| `HANDOFF/done/task_wire_get_iron_core_mode.md` | `blocked-by-metadata` | `gate-fail` | `48bfb3e6b046f2809a8e9a9ee7ab6666d1e558c88f803bb05c9374f297158744` |
| `HANDOFF/done/task_wire_get_last_profile.md` | `blocked-by-metadata` | `gate-fail` | `8f5d7ae3034e6fa1ae5ad2385b4b7cc6372a14239085471366847c1da7935f7b` |
| `HANDOFF/done/task_wire_get_overrides.md` | `blocked-by-metadata` | `gate-fail` | `795934fe084775fc4b3ff68f5ccd2b961fa49a476ed4e916f5594ddc06d9a9bb` |
| `HANDOFF/done/task_wire_get_peer_reputation.md` | `blocked-by-metadata` | `gate-fail` | `465d06784d58abc18a44404e6ae664bdb7572a896d41d252fbb354edd1311179` |
| `HANDOFF/done/task_wire_get_permission.md` | `blocked-by-metadata` | `gate-fail` | `8d0b0ea91f01d71665dc451ad8f6fa22b1079f0714f0a062776f330ad09fce47` |
| `HANDOFF/done/task_wire_get_privacy_config.md` | `blocked-by-metadata` | `gate-fail` | `d5b1a3944588bdd62ccd1ae6bc18d484aa34b6e38f9c07ed287adb6173563cc7` |
| `HANDOFF/done/task_wire_get_registration_state_info.md` | `blocked-by-metadata` | `gate-fail` | `cf4df2f858c6d6b8b3542402893d3d63a4a66d7d28caa822c36fa21b106f290f` |
| `HANDOFF/done/task_wire_get_settings.md` | `blocked-by-metadata` | `gate-fail` | `4280a3cba5024879581792c2ea797c6355a3a6d6497ca1b2a1676c4641f82bd5` |
| `HANDOFF/done/task_wire_get_signable_data.md` | `blocked-by-metadata` | `gate-fail` | `b84f99ce5b6f992a6d7a5ba43be78a071d11d7bdb9ff1aec70902dbdd84bc43b` |
| `HANDOFF/done/task_wire_get_signature.md` | `blocked-by-metadata` | `gate-fail` | `48523ce9662d5e317c0706deaaca5fabb8c3a490b01ceef6fd99b6178b0f1c1b` |
| `HANDOFF/done/task_wire_get_swarm_bridge.md` | `blocked-by-metadata` | `gate-fail` | `40a0d5f298f9dbdaac99b50d15feecc8f9e75cdd948a8a0d6c6c243efc771ad4` |
| `HANDOFF/done/task_wire_get_unhealthy_connections.md` | `blocked-by-metadata` | `gate-fail` | `3026a832f2a3f43b106025373605a07c0c0005ffb96a7b48ee14c4e543df90b4` |
| `HANDOFF/done/task_wire_handleBleFailure.md` | `blocked-by-metadata` | `gate-fail` | `20f6f9b9f70f4402f256f7f4138d46740b9fb03d5d57384594124af9bbff0316` |
| `HANDOFF/done/task_wire_handleScanFailure.md` | `blocked-by-metadata` | `gate-fail` | `aeb35c64688a3e2a1943b5e7ba3fb161a9b3dee2ce749f2abeef6a4435be6d4e` |
| `HANDOFF/done/task_wire_hasDnsFailures.md` | `blocked-by-metadata` | `gate-fail` | `36eb61f6d87d743ab2169968814ba5d4bfcd974f591c5065bc089e2df6c8e684` |
| `HANDOFF/done/task_wire_hasPortBlocking.md` | `blocked-by-metadata` | `gate-fail` | `7229c0b66ccc9d4db379170356fb01fc79de9228e6fd8cb4e847ddfde8a189b9` |
| `HANDOFF/done/task_wire_incrementAttemptCount.md` | `blocked-by-metadata` | `gate-fail` | `ea5212adde73cde251c00e85442440bb248f71ea79656e6fa0f74d3fb411a5b1` |
| `HANDOFF/done/task_wire_initialize_identity_from_daemon.md` | `blocked-by-metadata` | `gate-fail` | `b8170bdc8d9a283e1f1c82e63ccfca3f91f9592ee8924365a463c23f3becb765` |
| `HANDOFF/done/task_wire_isAtMaxDelay.md` | `blocked-by-metadata` | `gate-fail` | `fee6d1651a84ed6f1af729c27e3472e9996e57514c4825d798e5223ba058ce53` |
| `HANDOFF/done/task_wire_isPortLikelyBlocked.md` | `blocked-by-metadata` | `gate-fail` | `3b86d90fa9f0e692ba083f2c2965365453bd868873e7a2bb144a1eae394fe527` |
| `HANDOFF/done/task_wire_isServiceHealthy.md` | `blocked-by-metadata` | `gate-fail` | `d49bf56cf9497d1385fde115e35005bd0ca0a9c5b4ade0ecab80929118419463` |
| `HANDOFF/done/task_wire_is_ble_available.md` | `blocked-by-metadata` | `gate-fail` | `340840b947cd60c812991da5dd6d3ba8c8638d28de77b22a29ec46b6867579bb` |
| `HANDOFF/done/task_wire_is_permission_granted.md` | `blocked-by-metadata` | `gate-fail` | `22b89affd6405f286a961e5e9d25c2c1c9d971fa0a866d96a79811a4b837ed1b` |
| `HANDOFF/done/task_wire_is_prefetch_complete.md` | `blocked-by-metadata` | `gate-fail` | `94559695b33ca58b885bb8782ce3891219b6184530ae45b93144d0112f4affb4` |
| `HANDOFF/done/task_wire_is_prefetch_in_progress.md` | `blocked-by-metadata` | `gate-fail` | `ca0ca367652443973c78b2f930b8de9206cc5c6b0ab7517720c2b44be753169d` |
| `HANDOFF/done/task_wire_jsonrpc_get_identity.md` | `blocked-by-metadata` | `gate-fail` | `19b934312debe7c7e522adbf336a609b7eea2b55692ab94e566abe239ff00b36` |
| `HANDOFF/done/task_wire_jsonrpc_send_message_roundtrip.md` | `blocked-by-metadata` | `gate-fail` | `f681ab8fc24f29ad1509dc4a341f0ec65668fefbc6ce07246ce2a947d25fc8ef` |
| `HANDOFF/done/task_wire_known_contact_defaults_to_direct_message.md` | `blocked-by-metadata` | `gate-fail` | `29f7dedb8d9cecb6f554e8ee8fd17984f886462908f6a9f5ff862fd1d56d79ad` |
| `HANDOFF/done/task_wire_list_endpoints.md` | `blocked-by-metadata` | `gate-fail` | `ed39e8fde6ba859b396b6cd42831f1527bad88cd80bf1614152759c2cfa99ad5` |
| `HANDOFF/done/task_wire_loadConversation.md` | `blocked-by-metadata` | `gate-fail` | `41fde470e6d6cf99cd472c8936c415d940037863c2fe864e66b9e6942a591da5` |
| `HANDOFF/done/task_wire_loadMoreMessages.md` | `blocked-by-metadata` | `gate-fail` | `5284107ca14d8fb97ee14c71bc68d598426668df0e6dfa8fef9c16a0d1a745ca` |
| `HANDOFF/done/task_wire_loadPendingOutboxAsync.md` | `blocked-by-metadata` | `gate-fail` | `657d6733f44453034e0f3e82db1ad4f8dba352391d1ba7af401037faba4f20a3` |
| `HANDOFF/done/task_wire_load_device_id.md` | `blocked-by-metadata` | `gate-fail` | `5f9d2f45c3a836336af537cff31206dcaa4255466a8b3577a070a1dcfe7511a6` |
| `HANDOFF/done/task_wire_load_seniority_timestamp.md` | `blocked-by-metadata` | `gate-fail` | `d8a7518f6f4717ba46ef0f64e4d2e1c5764376f3814f687fcbbe2859ffd8b8d7` |
| `HANDOFF/done/task_wire_logMessageDeliveryAttempt.md` | `blocked-by-metadata` | `gate-fail` | `eb793681ba6f1c460990ab8d502b5d46bd5bb068575a51360ccd6228d46165f2` |
| `HANDOFF/done/task_wire_markCorrupted.md` | `blocked-by-metadata` | `gate-fail` | `17a278035deaca64622ae5c6e22e2bd32000f77ac0156fadfa685f3a140aa133` |
| `HANDOFF/done/task_wire_mark_path_failed.md` | `blocked-by-metadata` | `gate-fail` | `ae6c61202ea05a39003a38528f9952881c424a8c9e8b702461ed738349d9513c` |
| `HANDOFF/done/task_wire_mark_refresh_failed.md` | `blocked-by-metadata` | `gate-fail` | `5ae7a2a0ce58a8d98bf7ee5d4e6cd0c086342edcbc53b58a0ada4ffa00b3bd7d` |
| `HANDOFF/done/task_wire_micro_add_step.md` | `blocked-by-metadata` | `gate-fail` | `a987e52b9a9e4f0a869b33932a54e1609052c889c2857f1678b668d6b7bd24fb` |
| `HANDOFF/done/task_wire_micro_build_rust_feature_pipeline_dsflash.md` | `blocked-by-metadata` | `gate-fail` | `0ae0b0e378d1a433d614ad544b050fa3ffcf33899623863e65ab5cbed50f9105` |
| `HANDOFF/done/task_wire_micro_build_security_audit_pipeline.md` | `blocked-by-metadata` | `gate-fail` | `908ac8ea01d0def0eb141045154429bfadae71804db0ce08125775d811e8e413` |
| `HANDOFF/done/task_wire_micro_get_signature.md` | `blocked-by-metadata` | `gate-fail` | `958470e93f20a469dd4a696dd7cac9601a95cf1fd48127f8a0b17dbaf81cac34` |
| `HANDOFF/done/task_wire_micro_overall_score.md` | `blocked-by-metadata` | `gate-fail` | `4a70b60ac0912d1ec0208e28608222663294a82c25bf228b7f48619774348cdc` |
| `HANDOFF/done/task_wire_micro_start_refresh.md` | `blocked-by-metadata` | `gate-fail` | `5005efdb642717827b2cbbc6fcb82ca9fd01fe57bfd9d4402792907058befb97` |
| `HANDOFF/done/task_wire_negative_cache_stats.md` | `blocked-by-metadata` | `gate-fail` | `45b82d57a1266e28df2d8ba09ae1b5a79cd81b4f5a38d75ae472fd63baa5ee11` |
| `HANDOFF/done/task_wire_new_sync.md` | `blocked-by-metadata` | `gate-fail` | `bd9c0073a91124f22f1b05d3e56caa7789707d66f46187314ed9105396de3720` |
| `HANDOFF/done/task_wire_next_refresh_hint.md` | `blocked-by-metadata` | `gate-fail` | `4b06d3b19544314f868a17f4f15a6bcf5a82916f622f3ef851f2fc1126b2d1c5` |
| `HANDOFF/done/task_wire_nonce_length_invariant.md` | `blocked-by-metadata` | `gate-fail` | `67938254884d01df6bd4afa7e6ca584091bac67e87fd42b56e91dcb2660ed151` |
| `HANDOFF/done/task_wire_normal_low_volume_usage_is_unaffected.md` | `blocked-by-metadata` | `gate-fail` | `5b2dfdcd60e2c456218d15a018a3d01f4a259f6366b00fb37e301bc561db7d76` |
| `HANDOFF/done/task_wire_notif_mesh_topology.md` | `blocked-by-metadata` | `gate-fail` | `1c74af017eb0193d3222ca4c842f030a3bee57f31747fd66f92f0410fca82254` |
| `HANDOFF/done/task_wire_notification_roundtrip_for_ui_state.md` | `blocked-by-metadata` | `gate-fail` | `e7108152622762c41a5cdf6d71a2141aea582681552514a1cf503dac3b019756` |
| `HANDOFF/done/task_wire_notification_serialization.md` | `blocked-by-metadata` | `gate-fail` | `2556dd86cc03673d1768930aa31992d55abe02165e0b16136971e8a28f64df2a` |
| `HANDOFF/done/task_wire_notifyBackground.md` | `blocked-by-metadata` | `gate-fail` | `4880965923c56b13f1ba4b564c8e2e312da60784957953e001b7d9523bd2e40f` |
| `HANDOFF/done/task_wire_observeNetworkStats.md` | `blocked-by-metadata` | `gate-fail` | `45b6b9109c20342c60d995256cecad9e221de6559d627583384461b98e946f95` |
| `HANDOFF/done/task_wire_observePeers.md` | `blocked-by-metadata` | `gate-fail` | `ecbfe6523fef401e159113187b83c7ec4190e6fad7d24c593ffa69c723e196e6` |
| `HANDOFF/done/task_wire_onAnr.md` | `blocked-by-metadata` | `gate-fail` | `6ac78ef7c3b54143944c2650edb993bd211ad7b72e6ad51bad4aae748bd55181` |
| `HANDOFF/done/task_wire_onBind.md` | `blocked-by-metadata` | `gate-fail` | `1eba0061e949f6d5a68884dd42319b389356195a2d68980c58a846e8bdbb8403` |
| `HANDOFF/done/task_wire_onBleDataReceived.md` | `blocked-by-metadata` | `gate-fail` | `70e6cf3983e01990edbc150e6a013601d5b9d1e80c47c654850ff05bfef06990` |
| `HANDOFF/done/task_wire_onDiscoveryStarted.md` | `blocked-by-metadata` | `gate-fail` | `208aa6cd8e9f6af0302bdb66c8cf1eaf4f9ed2308aa7b9cbaa9ae37654c991dd` |
| `HANDOFF/done/task_wire_onDiscoveryStopped.md` | `blocked-by-metadata` | `gate-fail` | `ab0dd86ec5ef3fd0ae235182a4ef5a8f81b7de91e8eba18bc74a7f03963cb8de` |
| `HANDOFF/done/task_wire_onPeerDisconnected.md` | `blocked-by-metadata` | `gate-fail` | `ee49929417006d86d086e5cf1dac0de633ae0c24c088a550a228980cfe8144d5` |
| `HANDOFF/done/task_wire_onPeerIdentified.md` | `blocked-by-metadata` | `gate-fail` | `de260088f26735c357b873bdc190a811c2a5ff98d8a2e5f22f7554fee924790b` |
| `HANDOFF/done/task_wire_onReceiptReceived.md` | `blocked-by-metadata` | `gate-fail` | `9871f93b527a3193dbe18efd72c588bff64d1924f88387ed04951ce6d31ecaca` |
| `HANDOFF/done/task_wire_onRegistrationFailed.md` | `blocked-by-metadata` | `gate-fail` | `75ecbefdddabb2c95fc4ce97222a31ff93911cddcf5ce31d45796d2680badf97` |
| `HANDOFF/done/task_wire_onResolveFailed.md` | `blocked-by-metadata` | `gate-fail` | `ebf688f19470a1eb5db7f27496fe2180fdcb5da98040580db47dd19c4ffb7a8d` |
| `HANDOFF/done/task_wire_onScanFailed.md` | `blocked-by-metadata` | `gate-fail` | `84d7fccdfee70f1f4e0ee9760343fc46dc7b054bd313bd9be109fa62ff633669` |
| `HANDOFF/done/task_wire_onScanResult.md` | `blocked-by-metadata` | `gate-fail` | `c8e2086c9f6e6ee47d99ae14f196072d138ce99391d4c6d7870b2a52b2018e3d` |
| `HANDOFF/done/task_wire_onServiceFound.md` | `blocked-by-metadata` | `gate-fail` | `462556abda30fe6dcdb0f4902115fa3b46a0ca542ba7c2d548c8e62e8c3cb818` |
| `HANDOFF/done/task_wire_onServiceLost.md` | `blocked-by-metadata` | `gate-fail` | `a196925bda75bd88503b1c7f1f99054887bd6116a1c240ecc57d1c1a60d404f2` |
| `HANDOFF/done/task_wire_onServiceRegistered.md` | `blocked-by-metadata` | `gate-fail` | `e825c50c6f81204d701811332eec4d5ade9ff608ac5b818a0706e56f142cea09` |
| `HANDOFF/done/task_wire_onServiceResolved.md` | `blocked-by-metadata` | `gate-fail` | `e52c1491fe2f1ec2ec513bbae980911ef7113ec990ab5615f6a0557da6573d41` |
| `HANDOFF/done/task_wire_onServiceUnregistered.md` | `blocked-by-metadata` | `gate-fail` | `a059cbcd2646fe9a9e4cd2ea3c9f8a9874e56c74054a644c1c026fb6eaabeee6` |
| `HANDOFF/done/task_wire_onStartDiscoveryFailed.md` | `blocked-by-metadata` | `gate-fail` | `a63c7a8b63313fb5a9fcd11886ae081118aef52ee7956bb5b85461477f0554dc` |
| `HANDOFF/done/task_wire_onStartFailure.md` | `blocked-by-metadata` | `gate-fail` | `d5d78e28ee11705d25abcf92ea288ea771e977251beda89974c57705d98c402b` |
| `HANDOFF/done/task_wire_onStartSuccess.md` | `blocked-by-metadata` | `gate-fail` | `2493eca599589703ab171a6c2dfa1ed175c40f060a5e6015aa373da9cb9c2556` |
| `HANDOFF/done/task_wire_onStopDiscoveryFailed.md` | `blocked-by-metadata` | `gate-fail` | `0a8f39f2a88bef929f165b7d9603a0ca8ea14f4c06f11b30dcb72c4c09244d3e` |
| `HANDOFF/done/task_wire_onUnregistrationFailed.md` | `blocked-by-metadata` | `gate-fail` | `9587d9c770a240393875d482ba18d2da32b3837a4f7fc94fbd5e5431e7ab62b1` |
| `HANDOFF/done/task_wire_on_battery_changed.md` | `blocked-by-metadata` | `gate-fail` | `b0b81c29e278f749a5d47e6ced9fe3f680d2331a4da21e79e93c1bbacaf73767` |
| `HANDOFF/done/task_wire_on_ble_data_received.md` | `blocked-by-metadata` | `gate-fail` | `b4b21cc12e5e2fb36648e8eb758cb5e412eb61cafbd3b8e7dc214b8dff2b4908` |
| `HANDOFF/done/task_wire_on_entering_background.md` | `blocked-by-metadata` | `gate-fail` | `38b37bf81fc04da9e25ad381456d059240fe6b7f91dfadab6b9084102a470ab8` |
| `HANDOFF/done/task_wire_on_entering_foreground.md` | `blocked-by-metadata` | `gate-fail` | `43760cdf55cad828c2aa06662db36d2e2ce8d42d59b4006fce40e22f69dc8dff` |
| `HANDOFF/done/task_wire_on_motion_changed.md` | `blocked-by-metadata` | `gate-fail` | `89b93ff184519ca31c0c522da5eb1deb9986333a4d450c63f6ec0c5623c4737a` |
| `HANDOFF/done/task_wire_on_network_changed.md` | `blocked-by-metadata` | `gate-fail` | `8d92d0850f0364edfdf7db18df9d63d6a88562d05e918fed91d27e0b6cf81073` |
| `HANDOFF/done/task_wire_on_read.md` | `blocked-by-metadata` | `gate-fail` | `ba658b3e108d81cb59298c18e1accddc2a945defcc52bef7c4587146bca24748` |
| `HANDOFF/done/task_wire_on_write.md` | `blocked-by-metadata` | `gate-fail` | `cbd7b3c39a778e6bedb28c3022ec597cd10120ad4556a8c6fdf6b3e7c30b212d` |
| `HANDOFF/done/task_wire_overall_score.md` | `blocked-by-metadata` | `gate-fail` | `a7193e36edc22c25f61c4ef1a30a256eda933869158b56356483009f084a5ba3` |
| `HANDOFF/done/task_wire_override_ble_advertise_interval.md` | `blocked-by-metadata` | `gate-fail` | `c3c80089b0e33f944ca0b03172d82b0b1a060646c4e7043119ad71cbb0fcc8e3` |
| `HANDOFF/done/task_wire_override_relay_priority_threshold.md` | `blocked-by-metadata` | `gate-fail` | `2f55a409afb89a5d26e2d22f6e7b27869a110f1f6ce154c72a886ae8dfdb61d6` |
| `HANDOFF/done/task_wire_parse_response.md` | `blocked-by-metadata` | `gate-fail` | `5d1d1e58e6cdc52c37ad2e3afe49759e0c4c00400cb00f6b0ee5158edd71e1b2` |
| `HANDOFF/done/task_wire_peel_onion_layer.md` | `blocked-by-metadata` | `gate-fail` | `36ce70618bd89da61a5d1dd00edad7f491f2c83d739dd8484cce75ca9cfac056` |
| `HANDOFF/done/task_wire_peer_id_public_key_extraction_roundtrips_for_ed25519_peers.md` | `blocked-by-metadata` | `gate-fail` | `f526c7a73e197506468155c986a036a2cb2dea82dee52a805500b37653b84250` |
| `HANDOFF/done/task_wire_peer_rate_limit_multiplier.md` | `blocked-by-metadata` | `gate-fail` | `dcb60354cdf85b78f115f92a789c9037ba3e223206677fce6bdffc0dc285ad4c` |
| `HANDOFF/done/task_wire_peer_spam_score.md` | `blocked-by-metadata` | `gate-fail` | `eb83e91329217749a05b8a43449b295065c55f004197c5b826e42829d3e74b9a` |
| `HANDOFF/done/task_wire_peers_needing_reconnect.md` | `blocked-by-metadata` | `gate-fail` | `8cfd605810580f0fdba6c14a0d4b7dcff459e488d71cd161da906db08a80cd83` |
| `HANDOFF/done/task_wire_prefetch_manager_mut.md` | `blocked-by-metadata` | `gate-fail` | `476b18aede9a789492f13cc2ec803d03227594fd1d25645d5ff3a1463b996c50` |
| `HANDOFF/done/task_wire_prefetch_stats.md` | `blocked-by-metadata` | `gate-fail` | `6d981fce1b8ef84f6d5246748073e90f208e8cec5ba3177027408f22623ea30a` |
| `HANDOFF/done/task_wire_prepare_onion_message.md` | `blocked-by-metadata` | `gate-fail` | `ece154a71d18cac2524fbd864fcce0bb8ad70ed75e59e9508f30429844c0291b` |
| `HANDOFF/done/task_wire_primeRelayBootstrapConnectionsLegacy.md` | `blocked-by-metadata` | `gate-fail` | `9aaee0138cb684762f706add1ed5cab79f467d9b0955c2555574e7d880ef5f56` |
| `SCM-Q-accb304a5469` (path hash `accb304a54698fba407424cda79ba887577674c95250422e14c3424702a094d5`) | `quarantined` | `foreign-alias-in-bytes` | `2ace0946547fabe4ec68eccf87712741301a990e07cac08206dfbc3334f598a6` |
| `SCM-Q-c9696b6bdb09` (path hash `c9696b6bdb093662632712a260ea4edad58fc9a6dea4776cd0fbb6a3084f4305`) | `quarantined` | `foreign-alias-in-bytes` | `3e87b79597803bfd72a189a52a86894b9cf985fbdead640986a1bbf10420ef17` |
| `SCM-Q-2fe8526c737d` (path hash `2fe8526c737d5ac3b6845b196af4c8ed12f2bff43ddbd463f7e32fef32fa3ca3`) | `quarantined` | `foreign-alias-in-bytes` | `90bda0f8c2784c9a04ceb0f18d087d48bbd003d051de5e0e1a51094490ab6c44` |
| `SCM-Q-58fd05b2fef2` (path hash `58fd05b2fef2f2fb69f1c756fa3991bbc7b590e57762a4d306724d6187c21478`) | `quarantined` | `foreign-alias-in-bytes` | `f6204a27d15ebd8a8899ab9f138b15ca8226e6800464f09645a5b78c925c17cd` |
| `SCM-Q-0683c58f7dff` (path hash `0683c58f7dffd11b505ce1fd6189a4e9a5caa6b99b69078ac42d4ea90eb6509f`) | `quarantined` | `foreign-alias-in-bytes` | `9383a896a9536d8ed4745338ef8164c764af48accd88bfc974eedbfe545a480b` |
| `SCM-Q-966f8fb08fb3` (path hash `966f8fb08fb31c17ac62ba2b84439246d5be4d62a9d53ef0a583a8e00fa83028`) | `quarantined` | `foreign-alias-in-bytes` | `40203d452cdae78421ce1fac351732d02cd0cac10a44b992fbe91e8ea4c0a451` |
| `SCM-Q-88b8480669ed` (path hash `88b8480669ed2751657d76df8feba3815647986ac08ee349896de7fcee28436e`) | `quarantined` | `foreign-alias-in-bytes` | `b7f79a0bb1bfa3604a7b011071fefe9e4fef149b32e143b53be94545538f8546` |
| `HANDOFF/done/task_wire_provideMeshRepository.md` | `blocked-by-metadata` | `gate-fail` | `ba3952e47946812bd69cc80c15b3df725424a09ac64e5643e1d73d3b0e772632` |
| `HANDOFF/done/task_wire_providePreferencesRepository.md` | `blocked-by-metadata` | `gate-fail` | `c541c63c01d2d13f2d80bd8fd15f073b1b5d068dcb043f996f10c2fb6d9ecad1` |
| `HANDOFF/done/task_wire_prune_below.md` | `blocked-by-metadata` | `gate-fail` | `7b0ac1987964f43a22f968eafd6579b2a1d62ef2bddab746a78bfaa377923518` |
| `HANDOFF/done/task_wire_random_port.md` | `blocked-by-metadata` | `gate-fail` | `0b383948c6b424d73a6c622d5640e34b860332f87c802104b165d19251a5de76` |
| `HANDOFF/done/task_wire_ratchet_has_session.md` | `blocked-by-metadata` | `gate-fail` | `f11a5640cfbabb24d91e03c501328a7f125995df4a9c84b0aec95c625b89bc63` |
| `HANDOFF/done/task_wire_ratchet_reset_session.md` | `blocked-by-metadata` | `gate-fail` | `194c30f65d449af2347a87542dc7cfe03096d18bf31da9cdda61f601157b94e2` |
| `HANDOFF/done/task_wire_ratchet_session_count.md` | `blocked-by-metadata` | `gate-fail` | `a595a83373ce27a86353c0bc6d14efc70cfb302abf19907bf8c9080e4d6d030c` |
| `HANDOFF/done/task_wire_read_with_timeout.md` | `blocked-by-metadata` | `gate-fail` | `0e4a268d9445c4c2fde3367e005535164caaf41a78be83888bc43fb8cd4ae124` |
| `HANDOFF/done/task_wire_recordAnrEvent.md` | `blocked-by-metadata` | `gate-fail` | `0fdb3af14b2b4fb9ff56cc7dcf5384f7915a0fc5d5d0cf3ec053418282bdd889` |
| `HANDOFF/done/task_wire_recordConnectionFailure.md` | `blocked-by-metadata` | `gate-fail` | `3d1610256131c88369fc46602f6158dbcf9d60f3bf5512357e4c52a8838fa3ac` |
| `HANDOFF/done/task_wire_recordTransportEvent.md` | `blocked-by-metadata` | `gate-fail` | `16a5a1c9048d5f95f0973bf2a257b310dec41d4948c444e70542f7e967f60b93` |
| `HANDOFF/done/task_wire_recordUiTiming.md` | `blocked-by-metadata` | `gate-fail` | `70b7f21ee7a4f94815a34b1672c1e0da75b365a4dc405762e896edfc31a72159` |
| `HANDOFF/done/task_wire_refresh_delegate_routes.md` | `blocked-by-metadata` | `gate-fail` | `a11b1d963c10c5aaf2c67d87e8cb3b6cde96df42765bd9acd03a0a07ebeee57c` |
| `HANDOFF/done/task_wire_register_endpoint.md` | `blocked-by-metadata` | `gate-fail` | `fcab46c89bcf6a1e78547d08fef3cae0db19b7a3d8464358fa183eb45b2a4559` |
| `HANDOFF/done/task_wire_register_path.md` | `blocked-by-metadata` | `gate-fail` | `98610ea9e6593b6e410f688ca862ee30d8343852dd316950c1f499c62849af0c` |
| `HANDOFF/done/task_wire_register_state_change_callback.md` | `blocked-by-metadata` | `gate-fail` | `18f586d9fe72d175a023c0b4a381a60b3f743548319ae8ed216b4b784182750e` |
| `HANDOFF/done/task_wire_registration_payload_canonical_bytes_are_stable.md` | `blocked-by-metadata` | `gate-fail` | `fb1f5100d23c1fdc7cb380f75643efd3e6a631c1f7380a975d7ea09d77e72e9b` |
| `HANDOFF/done/task_wire_registration_transitions_for_identity.md` | `blocked-by-metadata` | `gate-fail` | `6e85933dddef0cc597fa3cd258e131bf4fa73ca125e9a0fff4c556cc1b944f2f` |
| `HANDOFF/done/task_wire_relay_discovery_mut.md` | `blocked-by-metadata` | `gate-fail` | `681b18d77bab3218b5d735f7562e4d253d7c8ffcb0db0c85dd24e29a8fb7ab9c` |
| `HANDOFF/done/task_wire_relay_jitter_delay.md` | `blocked-by-metadata` | `gate-fail` | `76180a6175ff8254d4f4482422ca30e22a2c2dedfab8ae0bb721c9cc7560ccd5` |
| `HANDOFF/done/task_wire_relay_request_carries_ws13_metadata_when_set.md` | `blocked-by-metadata` | `gate-fail` | `728563dced73a8ff9ecfa1bb7f5f10fc0f900d8df85b32b4bdc665a75e409224` |
| `HANDOFF/done/task_wire_relay_request_missing_ws13_fields_deserialize_with_defaults.md` | `blocked-by-metadata` | `gate-fail` | `14f28c6ca5b4398d71682bfbae4b001eb707764eea807c208a6a43311db97f51` |
| `HANDOFF/done/task_wire_remove_rtc_connection.md` | `blocked-by-metadata` | `gate-fail` | `84ac1e266df57b2cc548d3f85bed450cac5c3f83a65630825a4b5f8e9487c365` |
| `HANDOFF/done/task_wire_remove_websocket.md` | `blocked-by-metadata` | `gate-fail` | `c7de9541f7a446088d392a4b7e0c9a4e3660d03502ab616728ffd386d496ffcf` |
| `HANDOFF/done/task_wire_request_permission.md` | `blocked-by-metadata` | `gate-fail` | `7a43a1e1e1b4c88e7cfc93af99742da563453bfb3d68cb7a2ff7fa7c69240937` |
| `HANDOFF/done/task_wire_resetHealth.md` | `blocked-by-metadata` | `gate-fail` | `8078ca647bf4ef803efdc9b24437f1f3e33e1a64809c70ad6a17fa4bd860d02a` |
| `HANDOFF/done/task_wire_resetNotificationStats.md` | `blocked-by-metadata` | `gate-fail` | `50a00abdffbbc527b98b529eab23ea831660fe64fa3d1b15f2b593aba677e7bc` |
| `HANDOFF/done/task_wire_resetServiceStats.md` | `blocked-by-metadata` | `gate-fail` | `85d91d78c48f032411d16b174a7afb7d94d7f2a255ec81685588a601efecbac1` |
| `HANDOFF/done/task_wire_reset_circuit_breakers.md` | `blocked-by-metadata` | `gate-fail` | `7a6be4de70e87076f673903f6f20fc62693469f92b810202fce8c0063151c1e2` |
| `HANDOFF/done/task_wire_resolveDeliveryState.md` | `blocked-by-metadata` | `gate-fail` | `8fe1799519bd7719168aef4034975365cc61740e44545b9445e4f619d4d1bad5` |
| `HANDOFF/done/task_wire_routing_tick.md` | `blocked-by-metadata` | `gate-fail` | `13b64f4a0f1f78327a45ffd7f709d92f0345a9d51acb9a94eb24d08dc963bddb` |
| `HANDOFF/done/task_wire_run_optimization.md` | `blocked-by-metadata` | `gate-fail` | `6b979850815bea3217dd75fac618b11b2dde95f011bb60717acb370c39638955` |
| `HANDOFF/done/task_wire_scan_for_advertisements.md` | `blocked-by-metadata` | `gate-fail` | `9717fe2207aab1472d54aad6623ee78b95df56f3c88060e03605e395e26f60d9` |
| `HANDOFF/done/task_wire_searchContacts.md` | `blocked-by-metadata` | `gate-fail` | `0fd611d9b5c6a0084dfac59ba596ef2ef39ce5ab8b6ffcbc9d4d17716ff132ab` |
| `HANDOFF/done/task_wire_sendBlePacket.md` | `blocked-by-metadata` | `gate-fail` | `d0f3afc0fec8560fc7cfa1b5601fe4a34a6ac1d69d83f591d41ba39329fc45fb` |
| `HANDOFF/done/task_wire_send_ble_packet.md` | `blocked-by-metadata` | `gate-fail` | `0ec5e015e65f288d4d29ab1b2de3a3fa4d3746a4ed3540ff697c284e3037824f` |
| `HANDOFF/done/task_wire_send_message_status.md` | `blocked-by-metadata` | `gate-fail` | `f8ed207cf70c3643a03f44208d5ca381d114d953bd945360353df8679c46990d` |
| `HANDOFF/done/task_wire_send_prepared_envelope.md` | `blocked-by-metadata` | `gate-fail` | `27ba68eac7981b14dc00801296db9e01e7cba4f42a80c751d6d4b6d847fc9c36` |
| `HANDOFF/done/task_wire_setBleComponents.md` | `blocked-by-metadata` | `gate-fail` | `9faf4ae09c76ca4abc9ce8d69f8a19e28cbea3406ac5a9c5e9c129adda473ec1` |
| `HANDOFF/done/task_wire_setContactNickname.md` | `blocked-by-metadata` | `gate-fail` | `ce486bd71120171958f6d7bbfb8dfb633b30e7089f68b10422821490d6952f00` |
| `HANDOFF/done/task_wire_setPeer.md` | `blocked-by-metadata` | `gate-fail` | `07b2be567c09015024c0d0bd472f3d70cbdca381377531350330621196a4ba6c` |
| `HANDOFF/done/task_wire_set_cover_traffic.md` | `blocked-by-metadata` | `gate-fail` | `27f33c046fd47a76de0055a9e755c8548f763f0fd858656d9bc3741af59cd444` |
| `HANDOFF/done/task_wire_set_daemon_socket_url.md` | `blocked-by-metadata` | `gate-fail` | `2f483d7e6e9267e8e0ca8d0cae23781fee696685b35cecca1c2bc56a518e136c` |
| `HANDOFF/done/task_wire_set_delegate.md` | `blocked-by-metadata` | `gate-fail` | `a620af28ca57182dcb9947bf34ae6179f123c59596b8fffbddda1db3924801e3` |
| `HANDOFF/done/task_wire_set_iron_core_mode.md` | `blocked-by-metadata` | `gate-fail` | `ed8ad097079965aaa3d3410c67d5bc7eb9549a392158ba3451336f85d769c221` |
| `HANDOFF/done/task_wire_set_notes.md` | `blocked-by-metadata` | `gate-fail` | `34575762a927692e3f2dd92de1441ca31d455221af87f853c65f7233694dccb2` |
| `HANDOFF/done/task_wire_set_privacy_config.md` | `blocked-by-metadata` | `gate-fail` | `a3c70be6db1c945688df5d0801757b0674bc26d7de4a983c9cf6e6f9f5b43009` |
| `HANDOFF/done/task_wire_set_reputation_manager.md` | `blocked-by-metadata` | `gate-fail` | `4b27bbcf0d467e2edea870274f8449e3a9b401cea697f730cdc625f805cb001d` |
| `HANDOFF/done/task_wire_shouldRetryMessage.md` | `blocked-by-metadata` | `gate-fail` | `1835083508439116131675f3030913f933458f66fffa83ecdecc37c6889e34f6` |
| `HANDOFF/done/task_wire_shouldUseTransport.md` | `blocked-by-metadata` | `gate-fail` | `3c75bd15eec844374935e7d20010457192482f7f6e1e35419fe84d3eeb446ff2` |
| `HANDOFF/done/task_wire_should_advance.md` | `blocked-by-metadata` | `gate-fail` | `7f1bf2bc6f3d26bd61f4cd4f9eca9ffdf1816642f70e6115b841dc28e8cc9140` |
| `HANDOFF/done/task_wire_showMeshStatusNotification.md` | `blocked-by-metadata` | `gate-fail` | `9a8798ad61775363f08a5bcf7b5c048bcb27ee634136b263d02210a582437639` |
| `HANDOFF/done/task_wire_showPeerDiscoveredNotification.md` | `blocked-by-metadata` | `gate-fail` | `7e119cad358f2478ecc62b4d36bb8a5e47a4401dcf5b4475ef4c3fca5df4d3d9` |
| `HANDOFF/done/task_wire_show_permission_guidance.md` | `blocked-by-metadata` | `gate-fail` | `508f45189bb28a50742d30900b8d8d3c6a63f9942fa924b97b17b4fe14ad4ad0` |
| `HANDOFF/done/task_wire_signed_deregistration_request_rejects_same_source_and_target_device.md` | `blocked-by-metadata` | `gate-fail` | `41b31e0cf5a72c003ba42661b9c0079936e445fe0e94e00518e925df5acc2bde` |
| `HANDOFF/done/task_wire_signed_deregistration_request_verifies_against_matching_public_key.md` | `blocked-by-metadata` | `gate-fail` | `ae5f87193d9ebc3a357639f3754a222d5357f756c22dbd98aae5f49739bf3745` |
| `HANDOFF/done/task_wire_signed_registration_request_rejects_malformed_identity_id.md` | `blocked-by-metadata` | `gate-fail` | `3da800e6d8cef80dac8c7e726abbcc50c5985fa3578536144e08d6698378adf1` |
| `HANDOFF/done/task_wire_signed_registration_request_rejects_tampered_payload.md` | `blocked-by-metadata` | `gate-fail` | `45f76b63adfa42454155b5ef676f8a007dde25981607f7c45a19186396bd4b03` |
| `HANDOFF/done/task_wire_signed_registration_request_verifies_against_matching_public_key.md` | `blocked-by-metadata` | `gate-fail` | `1c89c9903f11212e2b95469b17f647fb97c068957d74faf49cbcde8700b4f29c` |
| `HANDOFF/done/task_wire_startAll.md` | `blocked-by-metadata` | `gate-fail` | `cd009b8317109d868103205dc5b9bd92b8f3fe340a7bfdefd6dbb56ec708f8be` |
| `HANDOFF/done/task_wire_start_hole_punch.md` | `blocked-by-metadata` | `gate-fail` | `2062e408786444b9b27b663cf8644adebf353257a2bf43543564926bf95b2fab` |
| `HANDOFF/done/task_wire_start_receive_loop.md` | `blocked-by-metadata` | `gate-fail` | `5b969b3575b4e90118ce031be3a82a400af3322ee11b36d62cd7857104ef9650` |
| `HANDOFF/done/task_wire_start_refresh.md` | `blocked-by-metadata` | `gate-fail` | `bdd5a65cf1f8c05b936a9097070f6b514dd8b29548b1dbda35fbb1e9ebd69ab1` |
| `HANDOFF/done/task_wire_stop_swarm.md` | `blocked-by-metadata` | `gate-fail` | `26a58d156c67c9f6fa22114fcd11a35cc2b54158469d2731af98c7c7026acb67` |
| `HANDOFF/done/task_wire_storage_pressure_emergency_mode_rejects_non_critical_and_recovers.md` | `blocked-by-metadata` | `gate-fail` | `5c54fcd438a550bb8afbc20bb925bb0efaa27998aa6894f407e6ea153dcfa8c9` |
| `HANDOFF/done/task_wire_storage_pressure_purge_prioritizes_non_identity_then_identity.md` | `blocked-by-metadata` | `gate-fail` | `8eeefa1b144bbdd8a9e0e6aeb84432517a4918b3af314793112adfa79552ac1f` |
| `HANDOFF/done/task_wire_storage_pressure_purge_records_audit_transition_before_delete.md` | `blocked-by-metadata` | `gate-fail` | `4d331b4c91e1a816a32ebf4b93ebefa6b41b7c5c427b8e92b6d25e6b82cff571` |
| `HANDOFF/done/task_wire_storage_pressure_quota_bands_follow_locked_policy.md` | `blocked-by-metadata` | `gate-fail` | `8edcf3a938008fe2026f3ca51c62252de06453295caa0e00b71a7ce1522082b4` |
| `HANDOFF/done/task_wire_storage_pressure_state_uses_synthetic_snapshot_when_probe_unavailable.md` | `blocked-by-metadata` | `gate-fail` | `97d1f1355092f943041b6c049365d27600c2653b313ee0d563aa8764cab785f7` |
| `HANDOFF/done/task_wire_testLedgerRelayConnectivity.md` | `blocked-by-metadata` | `gate-fail` | `a09876eed75998e8e275d748ca868ae6e9a4fb4e95282c847d13ad24b056c7c4` |
| `HANDOFF/done/task_wire_timeout_budget_summary.md` | `blocked-by-metadata` | `gate-fail` | `ea69f3f7815ee0da85e80ddec292c80152bf421d37cf7c4f31aabc7ba9727215` |
| `HANDOFF/done/task_wire_toLogString.md` | `blocked-by-metadata` | `gate-fail` | `12a4c48c804298d9f655c1c6cba6100c298649bd8ab85360e9934cce999d2522` |
| `HANDOFF/done/task_wire_token_bucket_refills_after_elapsed_time.md` | `blocked-by-metadata` | `gate-fail` | `ea4d924132ea03855bbfda01530a5e0e59f900c796b20904c6c9838ed4d30e54` |
| `HANDOFF/done/task_wire_touch_endpoint.md` | `blocked-by-metadata` | `gate-fail` | `1c13150a4fbcb0a9b117b666bd1566448e58db29a661e10e50e0655d8cf2a630` |
| `HANDOFF/done/task_wire_transport_type_to_routing_transport.md` | `blocked-by-metadata` | `gate-fail` | `b8b25808000bdd04080196cf324b9157f4756c89cf1e3e9ef0349d398394c784` |
| `HANDOFF/done/task_wire_try_enable_bluetooth.md` | `blocked-by-metadata` | `gate-fail` | `1effeeeb784a299869606831f19d2034a4e8ce9b56b9f58593ecaabc41751b49` |
| `HANDOFF/done/task_wire_unknown_method_error.md` | `blocked-by-metadata` | `gate-fail` | `86727a7b6c689ed22e45b2ec45ba861e19981838fec106e81515ceed0f60918c` |
| `HANDOFF/done/task_wire_unknown_sender_defaults_to_direct_message_request.md` | `blocked-by-metadata` | `gate-fail` | `cec8b2de9d3dac7bd06ea7eb90090f76426e28887ccca6294e3880fde65a6e63` |
| `HANDOFF/done/task_wire_unregister_endpoint.md` | `blocked-by-metadata` | `gate-fail` | `7db24c034e086931e9b5c27104d8ce6d080f2fb7b0f0b5e960e24b7bfe18e2c5` |
| `HANDOFF/done/task_wire_updateBatteryFloor.md` | `blocked-by-metadata` | `gate-fail` | `e559ba302501d80623730546fee3e954bf852ec295399216c71520aed8a3b096` |
| `HANDOFF/done/task_wire_updateContactDeviceId.md` | `blocked-by-metadata` | `gate-fail` | `49edef619d28d90ccceb556818973b4a29e733318eaa92dd53396173e9b6756e` |
| `HANDOFF/done/task_wire_updateDiscoveryMode.md` | `blocked-by-metadata` | `gate-fail` | `eafe5bd8b9a17803281b1df7375b693ec90ad181f6a65d4ff792f62650430fb0` |
| `HANDOFF/done/task_wire_updateHeartbeat.md` | `blocked-by-metadata` | `gate-fail` | `5c19c641f47d2f3e03101d55ed7e8ac8b35396ba81d53b220b9dd79e2e417876` |
| `HANDOFF/done/task_wire_updateInputText.md` | `blocked-by-metadata` | `gate-fail` | `2ffa30180e4a05ddfe665ab2ca34f03ad71075c42a5ec8098db9142584675922` |
| `HANDOFF/done/task_wire_updateMaxRelayBudget.md` | `blocked-by-metadata` | `gate-fail` | `6d9796221f2d56f3d47891c4465045eb0ce13e9da935124cd44b953fbcc1e676` |
| `HANDOFF/done/task_wire_update_keepalive.md` | `blocked-by-metadata` | `gate-fail` | `e38d201668f8c52842d6683f14456bf641fc5a460d753b0b936aeaef2476b5e2` |
| `HANDOFF/done/task_wire_update_last_known_device_id_can_clear.md` | `blocked-by-metadata` | `gate-fail` | `1ec2b8adf5a07bcac123f520c526a146d663584e395a95a7c02c309a46b4c6ae` |
| `HANDOFF/done/task_wire_update_last_known_device_id_ignores_invalid_values.md` | `blocked-by-metadata` | `gate-fail` | `2c841db39570ba3fef898e46cdf47b074c6884b2d47bb103c2b4633c2e6ee952` |
| `HANDOFF/done/task_wire_update_last_known_device_id_persists_and_is_readable.md` | `blocked-by-metadata` | `gate-fail` | `2f5ed6a05c1786b10ee7c4569ffe91ded2d3bb281986c360a6cd6fc3e244a150` |
| `HANDOFF/done/task_wire_update_last_known_device_id_trims_valid_uuid.md` | `blocked-by-metadata` | `gate-fail` | `3f84b5b1a738768aa8cf9256675fbbc186af1427ea3ead21dce9c3f729691bce` |
| `HANDOFF/done/task_wire_update_settings.md` | `blocked-by-metadata` | `gate-fail` | `2f841eb86353c76326720022cce0ffdcc44e6a629b861279d5fbe96e17f8f571` |
| `HANDOFF/done/task_wire_validate_audit_chain.md` | `blocked-by-metadata` | `gate-fail` | `ac94563e5c539ea9bdd581e5f72a731e22c7cf2141bc9be0fc1a0606843fbfe7` |
| `HANDOFF/done/task_wire_validate_settings.md` | `blocked-by-metadata` | `gate-fail` | `b3339c020502a63334b13796613c02af306bcab8d06492258ce60cb9e6a2e0b2` |
| `HANDOFF/done/task_wire_verify_registration_message_rejects_peer_identity_mismatch.md` | `blocked-by-metadata` | `gate-fail` | `35a50c8f36c5a648e1291f165f2f46f7f3f59070dbf090c725bef956ba8222cf` |
| `HANDOFF/dreaming/SKILL.md` | `blocked-by-metadata` | `gate-fail` | `f6734896c3be7c010c4d616aad2e410fb8213a01a844d28f03d774515424b0b0` |
| `HANDOFF/freebuff/CLARIFICATION_REQUEST_2026-08-31.md` | `blocked-by-metadata` | `gate-fail` | `662b86f72fbecf360a67f7816a90abf335e7b45b4d0cfcc2d8132f0b0f07168c` |
| `SCM-Q-b35c8948af7d` (path hash `b35c8948af7d77690c7f48e897b669939b64b2c50c0adcf756b22bfe845ec7da`) | `quarantined` | `foreign-alias-in-bytes` | `95e7891af2c239b38118cdf787530a7d6990c8e2f89224a56452835a3f5fcd36` |
| `HANDOFF/freebuff/done/README.md` | `blocked-by-metadata` | `gate-fail` | `bba0043a00a6c8717513ca0398708b0b2e69f4cefadec48827bfb0d6e6ca0a4c` |
| `HANDOFF/freebuff/done/V040_T5_DOCS_SYNC_GATE_IS_RED.md` | `blocked-by-metadata` | `gate-fail` | `042cb550377f686f87df0469eaa5a5cdb4b67081ee632a81b8d1fc0e1866c113` |
| `HANDOFF/freebuff/inbox/AUDIT_CANONICAL_OUTLIER_FINAL_TRACKING_2026-09-19.md` | `blocked-by-metadata` | `gate-fail` | `26e20641a4bf6c7ab28777a0189b9e32624ef1395ffa45411a5329b4bba95471` |
| `HANDOFF/freebuff/inbox/AUDIT_CANONICAL_OUTLIER_iter0_DONE_2026-09-19.md` | `blocked-by-metadata` | `gate-fail` | `0f0f23146c8c4afa0b90b98526e6f2ce0613e444e8c4dd3ac4ce54f1b5df08b9` |
| `HANDOFF/freebuff/inbox/AUDIT_CANONICAL_OUTLIER_iter1_DONE_2026-09-19.md` | `blocked-by-metadata` | `gate-fail` | `a8597da097a2347e682dcfe77eca6474db20e8fa981dd8d0aba4173f700625c6` |
| `HANDOFF/freebuff/inbox/AUDIT_CANONICAL_OUTLIER_iter2_DONE_2026-09-19.md` | `blocked-by-metadata` | `gate-fail` | `0ebf7b70586456b176858d01f77896a2345b63d384f5903201c8bf7bc21abe01` |
| `HANDOFF/freebuff/inbox/AUDIT_CANONICAL_OUTLIER_iter3_DONE_2026-09-19.md` | `blocked-by-metadata` | `gate-fail` | `1db7e81589f2db28ff6d3fbf5b618a488e3f236ae91bcaaf21a958ed2955514e` |
| `HANDOFF/freebuff/inbox/AUDIT_CANONICAL_OUTLIER_iter4_DONE_2026-09-19.md` | `blocked-by-metadata` | `gate-fail` | `705a5ebf2d034c298246775f83db56918d683d99279543cc3e994cdf3747cf9d` |
| `HANDOFF/freebuff/inbox/AUDIT_CANONICAL_OUTLIER_iter5_DONE_2026-09-19.md` | `blocked-by-metadata` | `gate-fail` | `a0445a73c6474ffa2565138fa83534ae11ec38287cb523679912d6282fabf174` |
| `HANDOFF/freebuff/inbox/AUDIT_CANONICAL_OUTLIER_iter6_DONE_2026-09-19.md` | `blocked-by-metadata` | `gate-fail` | `69dd759fb72f10f6d4f38ac41ca2f0d8c4a5e2034564bfb56cd029add45330f0` |
| `HANDOFF/freebuff/inbox/BRIEF_2026-08-31_state_pacing_and_next.md` | `blocked-by-metadata` | `gate-fail` | `a0583079b6330459ed3bfae51a0051fe8304ef712518751f22bf6943a2ea5228` |
| `HANDOFF/freebuff/inbox/PR289_WORKAHEAD_AUDIT_VERDICT_2026-09-14.md` | `blocked-by-metadata` | `gate-fail` | `eea5c70acbf806ed4a31a0d6beb283ed9d3bf72442c56490957552a34d49dba6` |
| `HANDOFF/freebuff/inbox/README.md` | `blocked-by-metadata` | `gate-fail` | `acd535b64b5e5afaaee02f6d97aaefb78717717741df0cf75c930b4c0a272f35` |
| `HANDOFF/freebuff/inbox/RULE8_PR262_PR263_VERDICT_OPUS.md` | `blocked-by-metadata` | `gate-fail` | `968a5b4d83fd10d0b8dc04f6748f3d8e1dd7b15356d11d928a2e2b9aea5765bd` |
| `HANDOFF/freebuff/inbox/RULE8_PR267_VERDICT.md` | `blocked-by-metadata` | `gate-fail` | `24782aaff888c71164154973031ba398c4fc16bb6c277bc661dce23a1cff1341` |
| `HANDOFF/freebuff/inbox/RULING_2026-08-31_FDHT_inventory_correction.md` | `blocked-by-metadata` | `gate-fail` | `5f2639e34ba2606e74a910252139161157e63519913129dfdc7291379d167de2` |
| `HANDOFF/freebuff/inbox/RULING_2026-08-31_T12_acceptance2_correction.md` | `blocked-by-metadata` | `gate-fail` | `d6261e8ffa568520ac6e5164e1ec6bfd499d3940dfa952e1a7da6ffb4556df0b` |
| `HANDOFF/freebuff/inbox/RULING_2026-08-31_T12_verification_invalid.md` | `blocked-by-metadata` | `gate-fail` | `b45098c27397111bcfe646848476c8c333982c2016c39ac96f61595fc39b7f27` |
| `HANDOFF/freebuff/inbox/RULING_2026-08-31_T13_FDHT_A_and_F7_B.md` | `blocked-by-metadata` | `gate-fail` | `99a4fbc88e74129b317a0596beb18c331ad28c6da9db9f4c703c3da7c8904e12` |
| `HANDOFF/freebuff/inbox/RULING_2026-08-31_T1_greenlight.md` | `blocked-by-metadata` | `gate-fail` | `6d1a4fa3929e09543787fa8eb73d68f53319bac0ea9ebca473d9b5dc3b93cd86` |
| `HANDOFF/freebuff/inbox/RULING_2026-08-31_T2_disk_cleared_ack.md` | `blocked-by-metadata` | `gate-fail` | `0ff4a12fe539dde0338883a6fd57495b6f9482998e0ae45890850995d299941c` |
| `HANDOFF/freebuff/inbox/RULING_2026-08-31_T2_disk_space.md` | `blocked-by-metadata` | `gate-fail` | `8173084cbe85804f1121b352349f4897111464d38055c1114ef5bfbb2234b3f8` |
| `HANDOFF/freebuff/inbox/RULING_2026-08-31_T2_merged_port_T1.md` | `blocked-by-metadata` | `gate-fail` | `0c54eae04aa4c94ec8777bbc79f16537b1c21bc5e4b64e924cb167032dc2e3ee` |
| `HANDOFF/freebuff/inbox/RULING_2026-08-31_clarification_response.md` | `blocked-by-metadata` | `gate-fail` | `930ac339baf2937639ba559019235ca68f45b0ab707b24f64c0845a9991c0c61` |
| `HANDOFF/freebuff/inbox/RULING_2026-09-01_PR267_REJECTED_my_ruling_was_wrong.md` | `blocked-by-metadata` | `gate-fail` | `01bc541eea6e44576d2c2d3a574307dd3f2bde9103eb2925795928440bcad8ee` |
| `HANDOFF/freebuff/inbox/V040_267_RECHECK_TRIAGED_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `4d786dda76e52aba4297e48657a2fe6733e22716251d4481676b86c38ef1a838` |
| `HANDOFF/freebuff/inbox/V040_272_CI_RED_FIXED_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `941c34857423daebf12adff0f14934759963ae3f6a87766686cd033e72b0cacf` |
| `HANDOFF/freebuff/inbox/V040_272_R2_TRIAGED_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `c2dbd57f5909045ccf293247f3dbfdfe068fc8443beeb93edcf02e9f19fa22cb` |
| `HANDOFF/freebuff/inbox/V040_272_REVIEW_TRIAGED_2026-09-02.md` | `blocked-by-metadata` | `gate-fail` | `4ba84a580ad7c5ed472eb1dfe4e7d4243bd329fc284132ed3576ae615a36abe2` |
| `HANDOFF/freebuff/inbox/V040_3NODE_BASELINE_AND_UPDATE_PLAN_2026-09-19.md` | `blocked-by-metadata` | `gate-fail` | `eb2a819d2ad96592f416061b77e74b1d95f4227ecf1f0962e59b286f8e1265cd` |
| `HANDOFF/freebuff/inbox/V040_3NODE_ROLLOUT_PLAN_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `c67f1f55f30c6c8203a95f12383a2626c51491af7bd21b4e0184d5aa8d9be08a` |
| `HANDOFF/freebuff/inbox/V040_3NODE_VALIDATION_RESULTS_b0f7ac4e_2026-09-06.md` | `blocked-by-metadata` | `gate-fail` | `3151bb279b826fed0c8945ee0856e8395d172f8cc7c4fee22ba7d8477da30791` |
| `HANDOFF/freebuff/inbox/V040_ARCH_VERIFY_DONE_E97C3F82_2026-09-04.md` | `blocked-by-metadata` | `gate-fail` | `577e1f88c3dde44d8374e05c26af597dcd7abd24969e6fb32bc4938c5785afed` |
| `HANDOFF/freebuff/inbox/V040_AWS_REBUILD_CEO_RULING_2026-09-04.md` | `blocked-by-metadata` | `gate-fail` | `fd78b39514ae5d3d060ac5a23d16b11d7fed19c4babe2ebe253f98f22ba3ca80` |
| `HANDOFF/freebuff/inbox/V040_AWS_REBUILD_DONE_E97C3F82_2026-09-04.md` | `blocked-by-metadata` | `gate-fail` | `aef9c5a5dc34890d8d7d1a8463d6bd09a3cc9a5946487b297bb8bab750e54aef` |
| `HANDOFF/freebuff/inbox/V040_AWS_REBUILD_PLAN_E97C3F82_2026-09-04.md` | `blocked-by-metadata` | `gate-fail` | `acc21a3df15d76869923b0cfa0f14b069ec65160cf3607831d2c6aa98e90f8dc` |
| `SCM-Q-f0c801e10314` (path hash `f0c801e103144d740be4dceb4f1a19a3de7a428a2a557d585bcd11d216f4430d`) | `quarantined` | `foreign-alias-in-bytes` | `a7a6c34aa1bf918cb95b1ee753c2ec5664cb30f1cb37482efadb2be41d33428f` |
| `HANDOFF/freebuff/inbox/V040_CANDIDATE_272_DELTA_ADDENDUM_REQUEST_44fee3c4_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `98ea3a541883e231c74b5d9bd249c742792478af573a947ca15cd79681b53939` |
| `HANDOFF/freebuff/inbox/V040_CANDIDATE_MERGE_CONFLICT_BRIEF_2026-09-04.md` | `blocked-by-metadata` | `gate-fail` | `0a62466be9ce1ec815c4dc2ae680e0e4b6025d7616353723545727957d42c049` |
| `SCM-Q-c0933c15db02` (path hash `c0933c15db02d4286222a42b663febe00c428215bf6f532051604fb18ec9c1e2`) | `quarantined` | `foreign-alias-in-bytes` | `5150ebbc61f6205e41459ae625edb784788ff5f77793ed716d302dd3d1a972a8` |
| `HANDOFF/freebuff/inbox/V040_CEO_BUYOFF_3NODE_CANDIDATE_2026-09-02.md` | `blocked-by-metadata` | `gate-fail` | `aaa17036546ec4e190a7fbebe28508deaa74e846fb57d71a0aa0183f7424f0d0` |
| `HANDOFF/freebuff/inbox/V040_CEO_CTO_SYNC_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `e46ad08a6b9bc5eb12ad73b801f62ed4af06ede6043006b511f3854f03bd4e9c` |
| `HANDOFF/freebuff/inbox/V040_CEO_DIRECTIVE_REVIEWS_ITERATE_TO_APPROVE_MULTI_TRANSPORT_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `d276b9e8275653da15a40e9ad2c6ccbf70fac1dc59fd7bc9ba467f0b27d4a971` |
| `HANDOFF/freebuff/inbox/V040_CEO_HANDOFF_2026-09-01.md` | `blocked-by-metadata` | `gate-fail` | `eb55ef327a9a5c0a93474c072076fb0ca0322b38c046b83c8b335d05cc30014e` |
| `HANDOFF/freebuff/inbox/V040_CEO_INPUT_272_REVIEW_DISPATCH_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `997aadf38b7a72ec8c120ab7a52b41e7c7ef0c7084a0a085156136e039cd9dc3` |
| `HANDOFF/freebuff/inbox/V040_CEO_RELAUNCH_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `9326e0c48511571733a7833171896dd3aa4a828900bb407122a2a752a9613b89` |
| `HANDOFF/freebuff/inbox/V040_COORDINATOR_HANDOFF_POINTER_2026-09-06.md` | `blocked-by-metadata` | `gate-fail` | `eaef18d0cc1a17727d9f49cab96cc7be4cc81ee4c7468f18cd71c51248500f04` |
| `HANDOFF/freebuff/inbox/V040_CTO_ARCHITECTURE_AND_3NODE_HANDOFF_2026-09-02_blocked.md` | `blocked-by-metadata` | `gate-fail` | `9eea5f2844345038da0702b10dac0721a9edfc46b32f88b1f4cebf33048e74f5` |
| `SCM-Q-f4980f3ebb45` (path hash `f4980f3ebb452b1f07419c041819a9a62f637bab1813efb06dec6ce00f660dda`) | `quarantined` | `foreign-alias-in-bytes` | `9b480dd7435f6d5d6a9b967f4a704b669a1df98d1b4e2987f7e7482ebbdc8298` |
| `HANDOFF/freebuff/inbox/V040_CTO_CEO_BUYOFF_3NODE_CANDIDATE_2026-09-02.md` | `blocked-by-metadata` | `gate-fail` | `008387172f82eed9da9a1623827dbe18be7f01f6a7f63c1e3fc6db2d3917ff0d` |
| `HANDOFF/freebuff/inbox/V040_CTO_CI_TOOLCHAIN_PIN_REQUEST_2026-09-04.md` | `blocked-by-metadata` | `gate-fail` | `0a2446000abb40046af1d27b306b469b590c96b06374af5fc9deb1adfaaa890c` |
| `SCM-Q-418c91281b6e` (path hash `418c91281b6e2a3dfa2677f505db757e2bc9b941e9689981e5bc8f6da54f0d88`) | `quarantined` | `foreign-alias-in-bytes` | `0c2b4f244314f643eb2691f0492948d2f53681e3f13c692c23f5696cfb0021bf` |
| `SCM-Q-1357c63c6fcd` (path hash `1357c63c6fcda2c49c83479988b55a198ca9eb33a30e144c534fe9c15f6ccce1`) | `quarantined` | `foreign-alias-in-bytes` | `b9282f4ac5639e55319a1981cc02d2fd031c8bb12878d2c75e32ff370388d5e6` |
| `HANDOFF/freebuff/inbox/V040_CTO_TRACKING_PING_2026-09-02.md` | `blocked-by-metadata` | `gate-fail` | `97721df7c1bb6d5eb064206d1f4f66cfdd767c3a1332dbcc57dd0b97a73daf73` |
| `HANDOFF/freebuff/inbox/V040_CTO_safe_clear_proposal_2026-09-02.md` | `blocked-by-metadata` | `gate-fail` | `247efc28d06e21890ffef812b207aa12670880811d76482656272fcaf09c82a6` |
| `HANDOFF/freebuff/inbox/V040_FINAL_APPROVE_DONE_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `b6318cdde2bde3c8120a6bcd39fae29531c193757d566117f9a9bfe29f5fc7ca` |
| `HANDOFF/freebuff/inbox/V040_FLEET_REDEPLOY_b0f7ac4e_PROGRESS_2026-09-06.md` | `blocked-by-metadata` | `gate-fail` | `cef64533d3ce03227d76f01765890742019f1ddb1dd7502d972f9b16d94d10a9` |
| `SCM-Q-640965f53edc` (path hash `640965f53edc6cb2d65594c8509ff20d0c81173450ff8adfc624ae41a3a9e62a`) | `quarantined` | `foreign-alias-in-path-and-bytes` | `d4cf9c521680b2569f7acca361658550a406019f714e3e58cbdd1315e7629a19` |
| `HANDOFF/freebuff/inbox/V040_MERGE_APPROVAL_RECORDS_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `e13b7d8bfcadb0fc9569ec82197b70ae6751f54e4d5483130b78f991900b0988` |
| `SCM-Q-376de493b92b` (path hash `376de493b92b588d3d1edf11267c75ec9adef85e8946433b9fe567f5b9ee2e3d`) | `quarantined` | `foreign-alias-in-bytes` | `8982445a6efe490fc6d44882080dc32af400f3ee82ecd19f8383fdb878413b78` |
| `SCM-Q-75c6b7c62ec9` (path hash `75c6b7c62ec95d551fb5e48e954603f72957bdda49d3d774324dc2684e35fbce`) | `quarantined` | `foreign-alias-in-bytes` | `82cf7294c89f713782d956ff67c60eb01c199515992181e1cde40a850f7772cf` |
| `HANDOFF/freebuff/inbox/V040_NIMBLE_PEER_FIX_DONE_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `b8b8888c8bc34dacbb49c6c670d70d710fa87bc67b808a0cd1192d0f72dc65ed` |
| `HANDOFF/freebuff/inbox/V040_NIMBLE_RECYCLE_FIX_DONE_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `1d6d0fbd1c4f2132da175c314c668916e21ee79d31dc7dc81d5c9b3ade58342a` |
| `HANDOFF/freebuff/inbox/V040_POST_VALIDATION_MERGE_PLAN_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `f06bd291e1f81f63021ae68492d415e350355ee06db6845cd39f69f8385643d6` |
| `HANDOFF/freebuff/inbox/V040_PR274_MERGE_DECISION_CEO_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `fd4f2f62d978645b5611b4a47241d247e42a32b4daf23e3d1ddbd9a7f434a06e` |
| `HANDOFF/freebuff/inbox/V040_PR_DISPOSITION_2026-09-02.md` | `blocked-by-metadata` | `gate-fail` | `f2df926599d3266f8828efe02325ebbf8b08914f7611e11effa137f4026afedf` |
| `HANDOFF/freebuff/inbox/V040_QWEN_REVIEWS_DISPATCHED_2026-09-01.md` | `blocked-by-metadata` | `gate-fail` | `8aa10de4b897b24a930ee2d5dc3098b8b6214bd519fc15fc1be1601f164e1b16` |
| `HANDOFF/freebuff/inbox/V040_RCA_RERUN_ROOT_CAUSES_2026-09-06.md` | `blocked-by-metadata` | `gate-fail` | `d9ff246e8995820d48c8b910b2cd2c3d1303b0e007d95d19672ebb4486220804` |
| `HANDOFF/freebuff/inbox/V040_REVALIDATION_GATES_DONE_2026-09-07.md` | `blocked-by-metadata` | `gate-fail` | `6419eadba99ff5985a4d40a0884e687756291d6c93719d13f61f6b5c1762e709` |
| `SCM-Q-2388977736d0` (path hash `2388977736d0f0b5cb13ef93411f10e6b8deffd5d7a3216f63c3403c33a98eca`) | `quarantined` | `foreign-alias-in-bytes` | `0776aa4f165142c200edf9a742e71ae3404ebe7e72f14486decdc22297f02d4a` |
| `HANDOFF/freebuff/inbox/V040_SCOPING_UDP_QUIC_ADMISSION_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `2e0f927c2f42c60758a462bccd609d9ecc9ed83b8ddc9422730b3cd4d679d120` |
| `HANDOFF/freebuff/inbox/V040_T12_done_2026-08-31.md` | `blocked-by-metadata` | `gate-fail` | `f6d838670d2c42451f9c40d4f520b0cb5beef3cd96c00c1c563e3872e8796a1e` |
| `HANDOFF/freebuff/inbox/V040_T13_A_REWORK_done_2026-09-01.md` | `blocked-by-metadata` | `gate-fail` | `5b8328177634edebe1b7598100cd6d6fad0ff3b3332b7f116d86e337ee382c68` |
| `HANDOFF/freebuff/inbox/V040_T13_A_done_2026-09-01.md` | `blocked-by-metadata` | `gate-fail` | `e43f6231557150fdb558e9e5da3625494ea887ab7c4f71143599c0798aa57704` |
| `HANDOFF/freebuff/inbox/V040_T13_B_done_2026-09-01.md` | `blocked-by-metadata` | `gate-fail` | `b6f043f42a783e083ebc7f72d3816ad7888b756f65a909334014338f686fd42d` |
| `HANDOFF/freebuff/inbox/V040_T13_F7_HINT_COLLISION_PROPOSAL_2026-08-31.md` | `blocked-by-metadata` | `gate-fail` | `8b19b1b4fe433ee3f9bd5209606fb56dc02e7fccdd87959ef897c6628a3b7105` |
| `HANDOFF/freebuff/inbox/V040_T13_FDHT_KADEMLIA_DISCLOSURE_PROPOSAL_2026-08-31.md` | `blocked-by-metadata` | `gate-fail` | `862fc1d4bd8d1c51e8a8046366e4ef5728d9fcb75f83ec22c5d43378ede04886` |
| `HANDOFF/freebuff/inbox/V040_T14_EPHEMERAL_done_2026-09-01.md` | `blocked-by-metadata` | `gate-fail` | `1834d77d82856d8cdf8c814b6659f8910ba03228b7d55dac1180b0d03d71cef7` |
| `HANDOFF/freebuff/inbox/V040_T14_done_2026-09-01.md` | `blocked-by-metadata` | `gate-fail` | `72f576053cbde043a41e2e36d9479bd61997a5f26305d8afba958085e217e39c` |
| `HANDOFF/freebuff/inbox/V040_T1_HALF2_done_2026-08-31.md` | `blocked-by-metadata` | `gate-fail` | `dc5595c2499fcba67c73a340ef112a4ac067e875170bfcdd11151f14dd624ac4` |
| `HANDOFF/freebuff/inbox/V040_T1_preflight_ready_2026-08-31.md` | `blocked-by-metadata` | `gate-fail` | `c95398e3c849710a7950fe12e54c719028d6ff4bcf8311c1d465b7240e6ab17f` |
| `SCM-Q-e98527485a22` (path hash `e98527485a22fbf4a6df5212326eaae3e8aca3b9d1ad1b61f8af5caa86d150f8`) | `quarantined` | `foreign-alias-in-bytes` | `b771e56148fb8e44a0f69e439b37d9b8bc46db36fea731b6058473450edbe83c` |
| `HANDOFF/freebuff/inbox/V040_T2_disk_cleared_2026-08-31.md` | `blocked-by-metadata` | `gate-fail` | `264d3326f45cc2a9a564c088309ebcb1861aed9dc0390ccc2a9912907999ad43` |
| `HANDOFF/freebuff/inbox/V040_T2_disk_space_question_2026-08-31.md` | `blocked-by-metadata` | `gate-fail` | `3242b0b5e922b6acf703d1b3400ca5e138c255a52dfc4e0f92f058a07f38c208` |
| `HANDOFF/freebuff/inbox/V040_T2_done_2026-08-31.md` | `blocked-by-metadata` | `gate-fail` | `a37b9f631534ea989f8c12d3383672ac933f1cea831bdd9aac56f43d5f487b52` |
| `HANDOFF/freebuff/inbox/V040_T4_done_2026-08-31.md` | `blocked-by-metadata` | `gate-fail` | `7cf338e380496e6c6c3562c4f21f9383b49059ecad843e7480d837b088bd069a` |
| `HANDOFF/freebuff/inbox/V040_T5_done_2026-08-31.md` | `blocked-by-metadata` | `gate-fail` | `a0c4d2fc09317282b20806f933f0eb2caaeeb2b7ee9a019310d5b65e19b982a8` |
| `HANDOFF/freebuff/inbox/V040_T8_done_2026-09-01.md` | `blocked-by-metadata` | `gate-fail` | `8251050850e46f6e5c968f4bf3ed0dc5ffb32e6d3d81d5e2b71ed9c1d633f6d4` |
| `HANDOFF/freebuff/inbox/V040_candidate_commit_2026-09-02.md` | `blocked-by-metadata` | `gate-fail` | `9ae80180d86f00dd44ac9df087a987746cbc377518f2399b97157ad3bb7d95d9` |
| `HANDOFF/freebuff/inbox/V040_cleanup_record_2026-09-02.md` | `blocked-by-metadata` | `gate-fail` | `ac026dd8d1ef70ec242bd5d7a751391cb09a26f9f39514deb8de7072aaf6771d` |
| `HANDOFF/freebuff/queue/AUDIT_CANONICAL_OUTLIERS_V040_V050_V100_2026-09-19.md` | `blocked-by-metadata` | `gate-fail` | `905675e9c7f9642d16a46d58dcf6a100df10a87149281cf311bc76923e7d288b` |
| `HANDOFF/freebuff/queue/V040_3NODE_VALIDATION_RUNBOOK_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `4e1607842c5f1cabe61eb959cc37710b82e8d0e2e9199dcfd57d1e08d4694e2a` |
| `HANDOFF/freebuff/queue/V040_ARCH272_ROUTING_FEED_FINDING_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `69fcf875a5ff3fdb89493238be9975974a2743fc7a63bd45ff375a5d6ef2b331` |
| `HANDOFF/freebuff/queue/V040_BEACH_JOIN_CONTINUATION_2026-09-05.md` | `blocked-by-metadata` | `gate-fail` | `f8f04ce1c3c30883464f3aeda05516b1e8059a6e53c02e0aea1e4ea17da51a35` |
| `HANDOFF/freebuff/queue/V040_BEACH_JOIN_PHASE0_1_2026-09-20.md` | `blocked-by-metadata` | `gate-fail` | `b18245d15f6b220e0afc0fc54d1bf817d4a8f2dc9da39bbb4ae87380235f7b5d` |
| `HANDOFF/freebuff/queue/V040_CEO_CLEANUP_MERGE_DIRECTIVE_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `368fc27182d2b07f41142826b7e3bbc14bd50cc4bc283ce17fa01239bcf80d5b` |
| `HANDOFF/freebuff/queue/V040_CTO_ARCHITECTURE_AND_3NODE_HANDOFF_2026-09-02.md` | `blocked-by-metadata` | `gate-fail` | `a5985ae072c00de254e07874d5491c05a5ce805f45538ee30ab3d2e765c3a15b` |
| `HANDOFF/freebuff/queue/V040_CTO_HANDOFF_2026-09-01.md` | `blocked-by-metadata` | `gate-fail` | `1e5cc8039db15b570b90fbdb5cbaf1151c726a77783392b9822d6f67ce70c958` |
| `HANDOFF/freebuff/queue/V040_CTO_PROMPT_2026-09-02.md` | `blocked-by-metadata` | `gate-fail` | `ff48b3846d9cc4ba9e1cbbb015049221149b0f7f3d20bdece6c92042c2fbc584` |
| `HANDOFF/freebuff/queue/V040_CTO_TRACKING_PROTOCOL.md` | `blocked-by-metadata` | `gate-fail` | `faaebc4420078957f608b0e53ec73936a83f6ad082765a2aefb293ef57c4c7b8` |
| `HANDOFF/freebuff/queue/V040_REVIEW_DISPATCH_267_272_FINAL_APPROVE_QWEN_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `53a0dc278e0a11368368595316f99d0110909c92904ff376fb0b2deee9804f6f` |
| `HANDOFF/freebuff/queue/V040_REVIEW_DISPATCH_267_FDHT_QWEN_2026-09-01.md` | `blocked-by-metadata` | `gate-fail` | `034c6d873a8499f7f291ad507e6bec67b97f67d16a1552c47b070afab72f3fe6` |
| `HANDOFF/freebuff/queue/V040_REVIEW_DISPATCH_267_RECHECK_QWEN_2026-09-02.md` | `blocked-by-metadata` | `gate-fail` | `0524135833e8a21ebcf12f0aca25eaf4412388cfa40cedb62c8292bd5908bfa5` |
| `HANDOFF/freebuff/queue/V040_REVIEW_DISPATCH_267_RECHECK_QWEN_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `a8d18847e2d2d5fdefeb2f35bc754215cf90a1bdcbacd2bf1d621479e9137797` |
| `HANDOFF/freebuff/queue/V040_REVIEW_DISPATCH_268_269_QWEN_2026-09-01.md` | `blocked-by-metadata` | `gate-fail` | `5ebf7f51789ee86dda462a98d23384aaf19e717a6dd804e457fc9161a69d2d26` |
| `HANDOFF/freebuff/queue/V040_REVIEW_DISPATCH_268_270_CONFIRM_APPROVE_QWEN_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `5be36b5396a0baebd41deed6839468a1119dd2dc39f96292d25ccc8390f3e92e` |
| `HANDOFF/freebuff/queue/V040_REVIEW_DISPATCH_270_EPHEMERAL_QWEN_2026-09-01.md` | `blocked-by-metadata` | `gate-fail` | `fdb6345f9632ce4f75151f550f6122b0348825e5346f2e3d4be03b13d3fa274a` |
| `HANDOFF/freebuff/queue/V040_REVIEW_DISPATCH_272_ARCH_QWEN_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `ca4e56d5ea8a99d798a3f15d5c3ab0c7279ed57a91ca95bf7bdd6ba232df4d1a` |
| `HANDOFF/freebuff/queue/V040_REVIEW_DISPATCH_272_CANDIDATE_QWEN_2026-09-02.md` | `blocked-by-metadata` | `gate-fail` | `f283647b528027142723131dd68486e49592b84b9d6a4d0b304f49d703791b61` |
| `HANDOFF/freebuff/queue/V040_REVIEW_DISPATCH_272_RECHECK_QWEN_2026-09-02.md` | `blocked-by-metadata` | `gate-fail` | `3cd547b37ba4e52a5e6fc06f036fe3ac33de330d50dd3dc190e704423f2b5cac` |
| `HANDOFF/freebuff/queue/V040_REVIEW_DISPATCH_273_NIMBLE_PEER_QWEN_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `2556b48caf6011bc927196a649df71cb122da3069b6a7ea356923a56b51523a1` |
| `HANDOFF/freebuff/queue/V040_REVIEW_DISPATCH_276_OUTBOX_FIX_QWEN_2026-09-04.md` | `blocked-by-metadata` | `gate-fail` | `e752836f9736922a8fc562db878b24926bab105cef834e5d2a7594da1a31add4` |
| `HANDOFF/freebuff/queue/V040_T10_FFI_SURFACE_GATE_PASSES_VACUOUSLY.md` | `blocked-by-metadata` | `gate-fail` | `ab1a5af3d9e659e2db23067ab63c8354d7367ef5712d3bd9d1e8d141fb4a09b8` |
| `HANDOFF/freebuff/queue/V040_T11_CANONICAL_DOC_RECONCILE.md` | `blocked-by-metadata` | `gate-fail` | `d7ac64b482533084d88ce29b627709e434d8b195a9c3e5c296d361c967d348fd` |
| `HANDOFF/freebuff/queue/V040_T12_CI_CONCURRENCY_AND_PATH_FILTERS.md` | `blocked-by-metadata` | `gate-fail` | `91c1ef3142f73115ba10ce4a731e1acb1de512ef90108f6fe49be31009fa4312` |
| `HANDOFF/freebuff/queue/V040_T13_RULE8_FOLLOWUPS_262_263.md` | `blocked-by-metadata` | `gate-fail` | `3b327b53056be6774ed8604804c640727c505115bc87da453366782dfac7f3bf` |
| `HANDOFF/freebuff/queue/V040_T14_EPHEMERAL_PORT_ADVERTISED_AS_EXTERNAL.md` | `blocked-by-metadata` | `gate-fail` | `d9ee517ace077498e09a6210a7ec6400010a77037fec8709b79c38d3ef48968c` |
| `HANDOFF/freebuff/queue/V040_T14_PREEXISTING_DHT_BUGS_TICKET_2026-09-01.md` | `blocked-by-metadata` | `gate-fail` | `a53c7c1dfca657bdc20929e04c6a65d3706ae9482cbb024e530d2981b441a551` |
| `HANDOFF/freebuff/queue/V040_T1_NODE_BOOT_SEED_DIAL.md` | `blocked-by-metadata` | `gate-fail` | `3bd2711f12c3b402c817a2970df038bfb645100ecd933c652e8a9d2790702ad9` |
| `HANDOFF/freebuff/queue/V040_T2_UNIFY_PEER_LEDGER_STORES.md` | `blocked-by-metadata` | `gate-fail` | `a40f6585010bcd92a7098f4c75c5385a84cd794d9621c303463f5dace4fabcd3` |
| `HANDOFF/freebuff/queue/V040_T4_ROUTING_FEED_ON_CONNECTION_ESTABLISHED.md` | `blocked-by-metadata` | `gate-fail` | `b4341429b1231b82e81eba4a091fda187a0ac59a736bbc5445bc73c8dd26b812` |
| `SCM-Q-c8e561c52fab` (path hash `c8e561c52fabf5886cd1311e335ad5f71bc31a2097762594030e0754eed97bcd`) | `quarantined` | `foreign-alias-in-path-and-bytes` | `43f90e053d77ae6f8c517fe9a6349a8657050eba3ca4a38599d75a0b7176d3d6` |
| `HANDOFF/freebuff/queue/V040_T7_ANDROID_PARITY_STAGING.md` | `blocked-by-metadata` | `gate-fail` | `c0f74ea961ffca65a3f30bc70c3844e337ddd07ea77b6783568b6eb717c712ad` |
| `HANDOFF/freebuff/queue/V040_T8_RESTORE_DIAGNOSTICS_FORMATTER_TEST.md` | `blocked-by-metadata` | `gate-fail` | `55d841923807bf1555010936255705080263bf1d603b1935c848f3d7840eead8` |
| `HANDOFF/freebuff/queue/V040_T9_PR_QUEUE_BURNDOWN.md` | `blocked-by-metadata` | `gate-fail` | `382fc5656ab23ab8e01bb4cee85bee32e912fae8351e55d196de0385fbf750fa` |
| `HANDOFF/freebuff/queue/V040_T_AND06_KOTLIN_COLLAPSE.md` | `blocked-by-metadata` | `gate-fail` | `c66b76a6ce4f50264ed15fe42266194d79d6e4579d8f5dc4856862ba08445d57` |
| `HANDOFF/freebuff/queue/V040_T_AND06_UNIFI_CUTOVER.md` | `blocked-by-metadata` | `gate-fail` | `908dbfce0f3eeb47724b4ab6f4d752f6cb00c46e53f1786d11fc5cb5a1ff1bc4` |
| `HANDOFF/freebuff/queue/V040_T_CONN_LIMITS_MULTIPORT.md` | `blocked-by-metadata` | `gate-fail` | `eb48cc0833bffe05f920c7f05ea723846b68033a242a0aa74427e0bda9102759` |
| `HANDOFF/freebuff/queue/V040_T_LEDGER_IP_CHURN_AUTONOMOUS.md` | `blocked-by-metadata` | `gate-fail` | `cf942b4704cf7738fefa9dd8d3844b1e097f68b190280d17f21c57061d750931` |
| `SCM-Q-9fe725dac6e6` (path hash `9fe725dac6e6bcd694dc101f5f573104e4f7422ee187c6a67d1b2c7876d0cb28`) | `quarantined` | `foreign-alias-in-bytes` | `89c98798413bf816f3540226ce6ea5a8bfae58f8be0b2d042dca3b86d2046187` |
| `SCM-Q-a81fa91958e2` (path hash `a81fa91958e23e4491789930673afad1962674022264b7ded4712e129948cd3b`) | `quarantined` | `foreign-alias-in-bytes` | `b435069a0d352131b581cdea56779f7088cdce97f7f220e75db71da3e2f942a5` |
| `HANDOFF/freebuff/queue/V050_WP1_IDENTITY_UNIFICATION_2026-09-21.md` | `blocked-by-metadata` | `gate-fail` | `d0ade2ddae8b3de732f1ec2c9262f830634e0ae28ce5b247272344a1fe245265` |
| `HANDOFF/freebuff/queue/V050_WP2_ROUTING_FEED_ALL_TRANSPORTS_2026-09-21.md` | `blocked-by-metadata` | `gate-fail` | `acfc6130084affe71fd1599b2ed134820eca53a54a72b747aaf4f36690867af9` |
| `HANDOFF/freebuff/queue/V050_WP3_INBOUND_COMPLETENESS_2026-09-21.md` | `blocked-by-metadata` | `gate-fail` | `afea7ec78f61d71debe8bdcee412b6fd8a19476f0ac8c47a4dea1d5fa22f2bc5` |
| `HANDOFF/freebuff/queue/V050_WP4_DELIVERY_TRUTH_2026-09-21.md` | `blocked-by-metadata` | `gate-fail` | `6a5597a87b2d51c325687418b3b5bf365cd8cc87830383253354c64d345afaee` |
| `HANDOFF/gpt/AWS_RELAY_CURRENT_ADDRESS.md` | `blocked-by-metadata` | `gate-fail` | `71092c8c7c96423102ab41d4c29ecd9c6ee1625653947481bf4028f4ded04d45` |
| `HANDOFF/gpt/CTO_TO_CAO.md` | `blocked-by-metadata` | `gate-fail` | `936a9dc02ef88d7e5b0c2fbd971ce21306718834087951a4bf88176798abcbe6` |
| `HANDOFF/gpt/CTO_TO_OPERATOR_2026-08-24_APPLE_INSTALL_PACKET.md` | `blocked-by-metadata` | `gate-fail` | `2d2f3ee1915af5d458dcc26387baccf36706fb4d42028a01184b41013f058c5b` |
| `HANDOFF/gpt/GPT_CODEX_TAKEOVER_2026-08-02.md` | `blocked-by-metadata` | `gate-fail` | `785f1feef5545998a7eb62e9b8fc41be1613d20d7a6d96b05090a1e88c686bb7` |
| `HANDOFF/gpt/GPT_DISPATCH_VALIDATION_2026-08-01.md` | `blocked-by-metadata` | `gate-fail` | `8f904c86d0e2d52a0cbe112c095a3628fced347d4d72a9ef9d71ea434ca29cc7` |
| `HANDOFF/gpt/GPT_GO_IOS_MACOS_POST_PR138_2026-08-05.md` | `blocked-by-metadata` | `gate-fail` | `0e232a74cbe2b4bc684643a3bfbee457fcacbb614cbf484a4704c028ce030989` |
| `HANDOFF/gpt/GPT_IOS_LANE_COMPLETION_2026-07-28.md` | `blocked-by-metadata` | `gate-fail` | `a834e53de49a0bc8dd4d06b683594cca21dfea18c5d874822404c613593d75ee` |
| `HANDOFF/gpt/GPT_IOS_LANE_FINDINGS_2026-07-28.md` | `blocked-by-metadata` | `gate-fail` | `5a2775b018af6833bf888a7998e28a739beedaf261a4788e7c4011d4ca3a04cc` |
| `HANDOFF/gpt/GPT_IOS_LANE_KICKOFF.md` | `blocked-by-metadata` | `gate-fail` | `faaedf1221402fa652c8895aa69371ccc42e810c6051badd6dc4cffa0e0ddb1b` |
| `HANDOFF/gpt/GPT_MAC_IOS_LOG_PULL_REQUEST_2026-08-08.md` | `blocked-by-metadata` | `gate-fail` | `62db2679933e025e3972614daf2141865ba60e320bd00e99047881262715f038` |
| `HANDOFF/gpt/GPT_MAC_PR139_TAKEOVER_2026-08-07.md` | `blocked-by-metadata` | `gate-fail` | `533e81722339c2d294fd7a2320fb1885c45f04f3952c3d7dc778e3ab80360995` |
| `HANDOFF/gpt/GPT_PLANNING_040_050.md` | `blocked-by-metadata` | `gate-fail` | `3a05bfb99c24f6bf031554d86ca5df51b75aea5c222064b570682ffef8697a3c` |
| `HANDOFF/gpt/GPT_PLANNING_040_050_VERDICT.md` | `blocked-by-metadata` | `gate-fail` | `ff17fcfedc28f149ffaf3790362a04d9c5da0bafcbf7e1cb655a034d0c0e2fb5` |
| `HANDOFF/gpt/GPT_REVIEW_SEEDING_FIXES.md` | `blocked-by-metadata` | `gate-fail` | `54005a02f926c73a3c83038dbabd5e40cd97070bc11483e9b993efd6ed23f57c` |
| `HANDOFF/gpt/GPT_SEEDING_REVIEW_MINIMUM_UNBLOCK.md` | `blocked-by-metadata` | `gate-fail` | `35257d5aca3e302b71a4b306c808119ded982b9ef19ded2a4f3276d65dde8170` |
| `HANDOFF/gpt/GPT_SEEDING_REVIEW_ORCHESTRATOR_REQUEST.md` | `blocked-by-metadata` | `gate-fail` | `a27b5c5a31ef0d50947bcdfa30ab7c6e2558d4dd78933cef510112c599b8c525` |
| `HANDOFF/gpt/GPT_SEEDING_REVIEW_ORCHESTRATOR_RESPONSE.md` | `blocked-by-metadata` | `gate-fail` | `0c3bc2b4d563fad5806eecc867eae861c614ecdfdb8a8eb5424cdac6b669e9b4` |
| `SCM-Q-f2557841a72d` (path hash `f2557841a72db2e461db8000746af0283afc0de6963ea9b7b81bdc4a8519ef81`) | `quarantined` | `foreign-alias-in-bytes` | `4dd16b9d44a7d648d5701edc0312856ee3ff8e67c984185ab032f781051bb26a` |
| `HANDOFF/gpt/GPT_SEEDING_REVIEW_RESPONSE_STAGE_1A.md` | `blocked-by-metadata` | `gate-fail` | `050773cd2f2c8859f77fce1b5a12645a0b005479dbb4917fe2b5de315f095ab9` |
| `HANDOFF/gpt/GPT_SEEDING_REVIEW_RESPONSE_STAGE_1B.md` | `blocked-by-metadata` | `gate-fail` | `1fe4f61d37e159d9771bff092abe46897d50e69df3ecf5ca3033aa0dc3dba632` |
| `HANDOFF/gpt/GPT_SEEDING_REVIEW_STAGE_1A.md` | `blocked-by-metadata` | `gate-fail` | `22e7b1a42bb2e90550b48ea210b4eefb6f15ead227ac5c74eadc4af6f4d2ab20` |
| `HANDOFF/gpt/GPT_SEEDING_REVIEW_STAGE_1B.md` | `blocked-by-metadata` | `gate-fail` | `66b8d892b5a71687c34bde692061dbc177a4578b5751037117952036d7798002` |
| `HANDOFF/gpt/GPT_SEEDING_REVIEW_TERMINAL_REQUEST.md` | `blocked-by-metadata` | `gate-fail` | `4352cd49dcaef679aa4cd744e0a6c46e57e687fe2397c4fb93d59e0512d1d964` |
| `HANDOFF/gpt/GPT_SEEDING_REVIEW_V2A_PRECHECK.md` | `blocked-by-metadata` | `gate-fail` | `1147f94f7aa42f87a5ed284ede3e1cc67415dbcff8f0d650bfe0fc85bf01b61d` |
| `HANDOFF/gpt/GPT_SOL_ULTRA_JOSH_040_READINESS_FINDINGS_2026-07-29.md` | `blocked-by-metadata` | `gate-fail` | `6535f8b34d3eea7b3d34ff46f2d0a5cadd891624517073d8851f94569b9a8f68` |
| `HANDOFF/gpt/GPT_SOL_ULTRA_PQC07_RATCHET_DESYNC_2026-08-01.md` | `blocked-by-metadata` | `gate-fail` | `dba493b3324a98a322b47b9e75cdec6fd5ec094a5b96288b711f688f1de1472a` |
| `HANDOFF/gpt/GPT_TAKEOVER_2026-08-01_WINDOWS_WINDDOWN.md` | `blocked-by-metadata` | `gate-fail` | `290350c5e13092ac412e6cf181d56ed9c6ced65cda02e881b3cceec8a9ed5d2a` |
| `HANDOFF/gpt/GPT_WAIT_FOR_PR136_BEFORE_FINAL_IOS_MACOS_BUILD_2026-08-04.md` | `blocked-by-metadata` | `gate-fail` | `848ffdf0c9d8c76e735099be70f559cd851c47d8d663d93072b2e5d875875ce4` |
| `HANDOFF/gpt/IOS_ANDROID_BIDIRECTIONAL_DEBUG_2026-08-02.md` | `blocked-by-metadata` | `gate-fail` | `0e31fb0c235130658fe9c3be7920103e94c26342ce2f1e5eca6cc8477203eea0` |
| `HANDOFF/gpt/IOS_ANDROID_PAIRED_CAPTURE_REQUEST_2026-08-02.md` | `blocked-by-metadata` | `gate-fail` | `59a63efecf26725186affead180651850bc3eef1a8559402666bc5423ee7ce15` |
| `HANDOFF/gpt/IOS_MACOS_PR139_STATUS_2026-08-07.md` | `blocked-by-metadata` | `gate-fail` | `466787492e3c46994cd68cf8e2da49255df0b9bfceca58f9bf2481299db372e6` |
| `HANDOFF/gpt/IOS_MACOS_PR139_STATUS_2026-08-09.md` | `blocked-by-metadata` | `gate-fail` | `b07383ba0d1d44124b189b04ee3a24eff1e7f8b9088173dfcfcabb22ad7c4f51` |
| `HANDOFF/gpt/MAC_WINDOWS_BLE_PARITY_QUEUE_2026-08-11.md` | `blocked-by-metadata` | `gate-fail` | `d60723918612d10bc7fc23a88496084ac65cc82f0c97b259f6ce03c859e59b81` |
| `HANDOFF/gpt/ORCHESTRATOR_DISPATCH_PLAN_2026-08-01.md` | `blocked-by-metadata` | `gate-fail` | `0c81d6c8c164c65b6a979638cfcc4969bcd336b4f44f063cd8b288757d961e8b` |
| `HANDOFF/gpt/PR139_ORCHESTRATION_STATE_2026-08-10.md` | `blocked-by-metadata` | `gate-fail` | `bfd42e677a85a4b8f19d2ad2d18d8c5d731240cd78ac99b9e04279456d26adb4` |
| `HANDOFF/gpt/QWEN_WAKE_EXECUTE_RELEASE_READINESS_2026-07-29.md` | `blocked-by-metadata` | `gate-fail` | `9855ef3389f6a930d104c957c486cd7e7b5504b679c8d8ce355cf9602abe28a5` |
| `HANDOFF/gpt/WINDOWS_5NODE_TEST_ANDROID_STATE_2026-08-03.md` | `blocked-by-metadata` | `gate-fail` | `bd3cd8d91d894d3026ebc9040697cd2c9e464660c7b0556fba43386c046696d8` |
| `HANDOFF/gpt/WINDOWS_ALL_NODES_FRESH_5NODE_GO_2026-08-03.md` | `blocked-by-metadata` | `gate-fail` | `3688f43b5832832a34aea753c1a25dca6a0f0bb4cf49806334c197c178bc37f6` |
| `HANDOFF/gpt/WINDOWS_ANDROID_LOG_FINDINGS_2026-08-02.md` | `blocked-by-metadata` | `gate-fail` | `5172536035150fb376ba7cd63d906f9fc00fd8ae92d69ee93f28e534956ebb06` |
| `HANDOFF/gpt/WINDOWS_BLE_FORWARD_GAP_2026-08-02.md` | `blocked-by-metadata` | `gate-fail` | `9cd449d714952dbea56a0fc96cac97d2f0f0c87a546a0bfd81ad3944e9ceecf5` |
| `HANDOFF/gpt/WINDOWS_BLE_ONDATARECEIVED_NEVER_RETURNS_2026-08-02.md` | `blocked-by-metadata` | `gate-fail` | `a1e5c8e9cca078210dd29cd0ffaf8d3dcd962f1e3b843ae69f17ed982ca53a9d` |
| `HANDOFF/gpt/WINDOWS_BRANCH_UNIFICATION_PROPOSAL_2026-08-03.md` | `blocked-by-metadata` | `gate-fail` | `8e57b6eeeabd0736955871ef3d07e8a5970fee0a56d0261b56af464a62a874a4` |
| `HANDOFF/gpt/WINDOWS_COORDINATION_PROTOCOL_2026-08-02.md` | `blocked-by-metadata` | `gate-fail` | `9fbe8ef1037f96c88093ba006d1a8f921f03bad7030f0514370a798371544531` |
| `HANDOFF/gpt/WINDOWS_CORRECTION_AND_BLE_FINDING_2026-08-02.md` | `blocked-by-metadata` | `gate-fail` | `398c7175e6f2f90d7cdc49cd89c8bcf7fa4aa2b8aa8edd18379bef37989e4307` |
| `HANDOFF/gpt/WINDOWS_CORRELATION_ANSWER_2026-08-03.md` | `blocked-by-metadata` | `gate-fail` | `9f254aa031e41f51cacd0d6b9f5d7f82680fd0599b88fb89b91277c1d385c1ad` |
| `HANDOFF/gpt/WINDOWS_GATE_EVIDENCE_AND_BRIDGE_RCA_2026-08-11.md` | `blocked-by-metadata` | `gate-fail` | `42cfc7d98c8e120376edb8072f6be641b0d77175700cf2fa0c89451b0f889f28` |
| `HANDOFF/gpt/WINDOWS_IDENTITY_UNIFICATION_MANDATE_2026-08-03.md` | `blocked-by-metadata` | `gate-fail` | `efbabfdc56027c75824b282c305d54c7a576fa68e6b635a7a68c5f770d5c1f6d` |
| `HANDOFF/gpt/WINDOWS_IOS_BUILD_PARITY_REQUEST_2026-08-03.md` | `blocked-by-metadata` | `gate-fail` | `fc1d0d681f8891a5c72c5a36c52231b9b66a411dae307743514ebef0c5e11dae` |
| `HANDOFF/gpt/WINDOWS_IOS_OWNERSHIP_AND_PARITY_2026-08-03.md` | `blocked-by-metadata` | `gate-fail` | `c55a4d3f27b3adcbe80af24db49cf155a8f012f1103def47d755af041e358a1f` |
| `HANDOFF/gpt/WINDOWS_LOG_BUNDLE_PROTOCOL_2026-08-03.md` | `blocked-by-metadata` | `gate-fail` | `7bb9fff1e11c187c5364813c9b98a1dca6b3bb62a68864b2420b9ac9a6d014ef` |
| `HANDOFF/gpt/WINDOWS_LOG_SYNC_WINDOW_2026-08-02.md` | `blocked-by-metadata` | `gate-fail` | `cf2c1f077d4e707f3b8cf17fc328f92873a12b971d3aedd1582e34ac5ab228f5` |
| `HANDOFF/gpt/WINDOWS_PR133_TRACKING_2026-08-03.md` | `blocked-by-metadata` | `gate-fail` | `40c10adcfdd9efb8a56bd192886ca5294c1c4b1234ba84d37c222b2791b28549` |
| `HANDOFF/gpt/WINDOWS_PR139_RESUME_STATE_2026-08-11.md` | `blocked-by-metadata` | `gate-fail` | `96a395062e14c7b01ebadfaf0e1ed4916fe96ad872ff03e1f8aa6c9c9790ccd9` |
| `HANDOFF/gpt/WINDOWS_PR139_RESUME_STATE_2026-08-11B.md` | `blocked-by-metadata` | `gate-fail` | `80276feeb282c220588833dc9ee5d74dbedfc6b581f249f37bad9fcba52fcccc` |
| `HANDOFF/gpt/WINDOWS_PR139_UNIFIED_BRANCH_AND_CHANNELS_2026-08-10.md` | `blocked-by-metadata` | `gate-fail` | `a1588e540c9cf1920b0500bae5dbf7d385e9249416fa4bb7f5f67f14620893ca` |
| `HANDOFF/gpt/WINDOWS_QWEN_CONSOLIDATION_REQUEST_2026-08-02.md` | `blocked-by-metadata` | `gate-fail` | `1273977b52f8cb5ab00709011d852a52593205f1464c02f3d8ed731c25569c2c` |
| `HANDOFF/gpt/WINDOWS_REQUEST_IOS_LOGS_REGRESSION_2026-08-02.md` | `blocked-by-metadata` | `gate-fail` | `48efef49df57b5a4e68f8904d224e9713bb96f17ae1f60fbe8386c7ea64b9049` |
| `HANDOFF/gpt/WINDOWS_REQUEST_IOS_LOG_EXCHANGE_2026-08-02.md` | `blocked-by-metadata` | `gate-fail` | `9319621f4dd5ccf9ac9c984c8708f165711627e21f226be535669aae82df43b8` |
| `HANDOFF/gpt/WINDOWS_REQUEST_IOS_UPDATE_CHRISTY_2026-08-02.md` | `blocked-by-metadata` | `gate-fail` | `3265ec042de69c603acafbc4b88f663175167c4794df34a0b18fe0894a754574` |
| `HANDOFF/gpt/WINDOWS_REQUEST_PLAN_CONFIRMATION_2026-08-02.md` | `blocked-by-metadata` | `gate-fail` | `c35d8e759f7df82faac78e74855d9e26e96b9d336247f6bb5d8f1ab0e9093379` |
| `HANDOFF/gpt/WINDOWS_REQUEST_RELEASE_READINESS_AND_UNIFICATION_2026-07-29.md` | `blocked-by-metadata` | `gate-fail` | `bd46aeaef355a70f4c0f0eff6ba205f0474a29b819a0ae53aae32f0e3a18df6b` |
| `HANDOFF/gpt/WINDOWS_ROOT_CAUSE_SWARM_DEAD_2026-08-02.md` | `blocked-by-metadata` | `gate-fail` | `838c95d4ea43027ad40fdeebfb8e1014eda68339c34c002894c9003d54b4936d` |
| `HANDOFF/gpt/WINDOWS_RUN2_READINESS_2026-08-03.md` | `blocked-by-metadata` | `gate-fail` | `d7c0a76698765dbb12a14d5e8d3e4b467488dfa4e98b51efe6b6462285514356` |
| `HANDOFF/gpt/WINDOWS_STATUS_SYNC_2026-08-02.md` | `blocked-by-metadata` | `gate-fail` | `df28192dc61a2d5cae88194b31059a8a6bb9a7a07d1e91ea4303d82316ad991c` |
| `HANDOFF/gpt/WINDOWS_V040_V050_FOUR_NODE_PARITY_KICKOFF_2026-08-21.md` | `blocked-by-metadata` | `gate-fail` | `08c69b7c400605c1231386074486d4bdc2155b4ffcc69bb53a5e006a3a12d4aa` |
| `HANDOFF/gpt/WINDOWS_WIPE_DISCLOSURE_PARITY_NOW_2026-08-03.md` | `blocked-by-metadata` | `gate-fail` | `5b12a9c8f55f324b2acf3fe849f70ac55871cdd16797b4c7f21bbfee1b1925f3` |
| `SCM-Q-0fc71aace06e` (path hash `0fc71aace06e82d321f13a5bf27b2933fa057417fc49fd38dbd56bd7cea3e518`) | `quarantined` | `foreign-alias-in-path-and-bytes` | `83c25fae88a1078b513c25097121b584120c1e4b03947d16c5413430779949ff` |
| `SCM-Q-8522a41e48b1` (path hash `8522a41e48b199b977ee984a537dfa806783c218abf86882c845c7584106d073`) | `quarantined` | `foreign-alias-in-path-and-bytes` | `c625fd8405673a8772e294939ec73e1ea41f625967d6970993ce646b2cc24500` |
| `SCM-Q-b99949133f9e` (path hash `b99949133f9e8dae1c64e5cc4265f1b88cdc5f273f8512567cf80159b5d2aa79`) | `quarantined` | `foreign-alias-in-path-and-bytes` | `07a5d2c94270bd85a0a13b41bcf5fe28e6a223e31bb92b7ee743737116f49945` |
| `HANDOFF/implementation_plan.md` | `blocked-by-metadata` | `gate-fail` | `fcf920f33b2cee3c206dbf57d66281c2b95357e45876795051ae6bde17b881a5` |
| `HANDOFF/implementation_plan_5.9.26.md` | `blocked-by-metadata` | `gate-fail` | `15ffad3edd8b84af35196e8679b0edea5572e2778311caad0f6f45873f936423` |
| `HANDOFF/logs/PAIRED_WINDOW_2230-2245Z.log` | `blocked-by-metadata` | `gate-fail` | `ca4865966666f642bb7d06fcea930559c825164015a0db5743c94497fd541565` |
| `HANDOFF/logs/android_all_buffers.log` | `blocked-by-metadata` | `gate-fail` | `344cbe8b170e3022044f327ea7652916ea0dedb61a87ff67423a732c63cf3373` |
| `HANDOFF/logs/android_ble_1045.log` | `blocked-by-metadata` | `gate-fail` | `48b06a9d50ebacb25a2401283e970606e9f767ac002a197780d4b99a1f415754` |
| `HANDOFF/logs/android_hist_2219.log` | `blocked-by-metadata` | `gate-fail` | `f27d2c743bae110eebd744eebb3f16d0461e8e87afeb74f830aabbb4a6171055` |
| `HANDOFF/logs/android_live_1043.log` | `blocked-by-metadata` | `gate-fail` | `a3c63c0f58b07d11cde522211ecc6e932fc8c0cd3df80ad36e59d4329b863915` |
| `HANDOFF/logs/android_live_window.log` | `blocked-by-metadata` | `gate-fail` | `f5ae1cc2dbcb876f44c2d0cf250b2728199efccd2371945c1306702449e9582e` |
| `HANDOFF/logs/android_msgflow_1046.log` | `blocked-by-metadata` | `gate-fail` | `73f91f75d67c774e1bce5787cc354d7d48fcd3a075c905d89ca79eabcc637d57` |
| `HANDOFF/logs/build_cli.log` | `blocked-by-metadata` | `gate-fail` | `9e179b053eebcc047874f9a702b07369cf70cf9751e8aa4eadf293224a319873` |
| `HANDOFF/logs/chat/2026-08-11.jsonl` | `blocked-by-metadata` | `gate-fail` | `5d44040ae065eefad5f1ab53d7daf5aa20691dbf44dbaf90e82161a53167a9a6` |
| `HANDOFF/logs/chat/2026-08-13.jsonl` | `blocked-by-metadata` | `gate-fail` | `6146d46b0076edb9423924a042935784cd901ef21dc482e27fb2562f0482171d` |
| `HANDOFF/logs/cloud_node_2026-08-02.log` | `blocked-by-metadata` | `gate-fail` | `770d70d512f90ac3061e00794b13e7aa157b627ddf7f992004711b39c72f2fa4` |
| `HANDOFF/plans/BEACH_JOIN_AUDIT_AND_PLAN_2026-09-05.md` | `blocked-by-metadata` | `gate-fail` | `53dff7e1a4fafbdce443b34f9099586bdfd8bdb7558131a0233fe86fe4748332` |
| `HANDOFF/plans/BLE_OFFLINE_DEMO_PLAYBOOK.md` | `blocked-by-metadata` | `gate-fail` | `1ef0875b9ad302c81aba7eb615cb7e4c255a8ae5ec722e430b3d8bf510c6c531` |
| `HANDOFF/plans/BLOCK_DUAL_FLAVOR_DESIGN_2026-08-06.md` | `blocked-by-metadata` | `gate-fail` | `a718721b914be6ead10533eeb0c4769b301f6c44349704bfb3c5426a20297c73` |
| `HANDOFF/plans/FARM_FINAL_PLAN.md` | `blocked-by-metadata` | `gate-fail` | `113a870fbf0a06335a69b5e18b46510feb624a3b5a8d35c401b229686e660be2` |
| `HANDOFF/plans/FIVE_NODE_RUN_2_PLAN.md` | `blocked-by-metadata` | `gate-fail` | `bdc10f4ecc93ad44982e09808e4b2276e18a6c6cbb6abd8c6a103ae0ea8d1cd3` |
| `HANDOFF/plans/FIVE_NODE_UNIFIED_TEST_PLAN_2026-08-09.md` | `blocked-by-metadata` | `gate-fail` | `be672ef23b35c4b097d5a74131253724e6c5153fa1ea155c8643223104628db1` |
| `SCM-Q-16e1cd291fdd` (path hash `16e1cd291fdd07672064b7b81c20c7bb24bf22bd76932b44bb725a5cd2d9e92f`) | `quarantined` | `foreign-alias-in-bytes` | `61308413737d9c6009bed1a7a4b492e2a5d1fc888291a738b0ad28eaa985c7c5` |
| `SCM-Q-30fdfdbbb6e8` (path hash `30fdfdbbb6e828417a9ace5f58ed8cc8c503b358c6615ad6ac978ee89eb6b284`) | `quarantined` | `foreign-alias-in-bytes` | `7279b7c4719ea9c404e594a52f0f100034df4689842762b80fbd4f6104ce3072` |
| `SCM-Q-29ce5e3712bb` (path hash `29ce5e3712bb35f1af9a2414e96a3f45807ef32b3ce9cfe9d4f8595fb4b4765d`) | `quarantined` | `foreign-alias-in-bytes` | `2e6a4c34a52d743b2cb9211ed4bfbce19c583fdb3826116cc586f319d5ffc06e` |
| `HANDOFF/plans/ITERATION_2_NAT_TRAVERSAL_TEST_2026-08-10.md` | `blocked-by-metadata` | `gate-fail` | `e67ba07a77942e4c94dc0ff15929aa6a27cd7e3469eb9ae83357344c5dbd22d5` |
| `HANDOFF/plans/MEETING_MODE_DESIGN.md` | `blocked-by-metadata` | `gate-fail` | `7e45f1ca833cf2360b01bd47561f693eee26962af6fc08e4a34212a61ae0d912` |
| `SCM-Q-9bf2bfdad8c3` (path hash `9bf2bfdad8c3dc687b78534e37cbf45e82c565aa5a783f8d4e24a9b5b2b59271`) | `quarantined` | `foreign-alias-in-bytes` | `0236d86e61031f46925101e1afc29c9890285ca109e51e556502668821a83e80` |
| `SCM-Q-6ef47d2cfcac` (path hash `6ef47d2cfcac1857314fa915146e751e8d447d14a7a1fd1c944ad3da462f5b3c`) | `quarantined` | `foreign-alias-in-bytes` | `57eefabe81cec847e882474b3db782ca55604d0077dc3f7767e7b3d35c387812` |
| `HANDOFF/plans/P1-10_adaptive_port_selection_design.md` | `blocked-by-metadata` | `gate-fail` | `a0403c3ca2fab11d9a0693339efdf1a76d3c046e45779534dbb7a391597cd7bf` |
| `HANDOFF/plans/P1-15_transport_matrix_audit.md` | `blocked-by-metadata` | `gate-fail` | `abee54eb0defe338c4bbb0093b65653fe936d0e0d532b310492a6a9bb652a90d` |
| `HANDOFF/plans/P1-17_windows_wifi_direct_design.md` | `blocked-by-metadata` | `gate-fail` | `a34a67658100a8807534ed6b4462fbdac863e9594ee43887646db8352f0fb62e` |
| `SCM-Q-f1e358380760` (path hash `f1e358380760175d5dd063170cf1dda9cfd884f672116a8c69291cdb0136aac2`) | `quarantined` | `foreign-alias-in-bytes` | `acb4a17e491a5e049b610f6176fc67d5276e7c935f9237aed0cd736534806b8a` |
| `HANDOFF/plans/PR_MERGE_UNIFY_PLAN_2026-08-04.md` | `blocked-by-metadata` | `gate-fail` | `d7a0a0b1b2b3ee7596223a26f882d12c21f8ea6fbbebad165141f9b80e94a0b7` |
| `HANDOFF/plans/QWEN_ITERATION_PLAN_2026-07-08.md` | `blocked-by-metadata` | `gate-fail` | `b393417a19f6fac8496b9f2856c242c0cbbf0aa9da29d560e3ba163cf9d2f5ff` |
| `HANDOFF/plans/RETICULUM_AUDIT_2026-09-21.md` | `blocked-by-metadata` | `gate-fail` | `cb38ef466fbc533517651a16aa4587fac6decb894941802181ea3344147c440a` |
| `HANDOFF/plans/RFC1918_MESH_ROUTING_SPEC_2026-08-08.md` | `blocked-by-metadata` | `gate-fail` | `66a843f809ef00f7a24f7bda2e886aea9836f270222354ab8a8f665c9799c187` |
| `HANDOFF/plans/RFC1918_ON_RFC1918_DISCLOSURE_DESIGN_2026-08-06.md` | `blocked-by-metadata` | `gate-fail` | `527bdd606210cf67ffc03f6a4b627739cb4ade6dde644926998bfffed486644b` |
| `HANDOFF/plans/T1_BLOCK_FLAVOR_FIX_DESIGN_2026-08-05.md` | `blocked-by-metadata` | `gate-fail` | `9163516c0295febf902eefdff9bb5e69e1bef614855f283dc807c534c8410a42` |
| `HANDOFF/plans/TRUST_SCOPED_LAN_DISCLOSURE_DESIGN_2026-08-05.md` | `blocked-by-metadata` | `gate-fail` | `1349a56452103e5df6a9a53e9e8dfb1fc2ec14abb4efb9bc2913049f78cb96a7` |
| `SCM-Q-bb927964f3c8` (path hash `bb927964f3c8ce698006fb75317928a6498b0915412df55026215f6134b753e1`) | `quarantined` | `foreign-alias-in-bytes` | `af137a3b5d8132d614d92f682af7ba9358095e55ae53a01c68e4da9f5d3a7d75` |
| `HANDOFF/plans/UNIFICATION_V3_DELIVERY_CONVERGENCE_PLAN.md` | `blocked-by-metadata` | `gate-fail` | `b5dbb4a916846e9fa3e3d057856f2346d91196cbb7dacf5bf8299fa5d1fdff38` |
| `SCM-Q-645616499e5e` (path hash `645616499e5ef1d767ea538a3cbdf4c15defc2e783678b16c44d20e121547eb8`) | `quarantined` | `foreign-alias-in-bytes` | `b5698fbac593285486d58e0b3c7a3e24bbac9f758d4c87cc7ef365431c992f87` |
| `HANDOFF/plans/UNIFIED_MASTER_MERGED_PLAN_2026-08-10.md` | `blocked-by-metadata` | `gate-fail` | `09fc30eed7774b2f827bcefaf9a13a7723eecc357eec3d0aa9d4d0a9a129a3b4` |
| `HANDOFF/plans/V040_COMPLETION_PLAN.md` | `blocked-by-metadata` | `gate-fail` | `96755b009a66fb30b9b3cd378b0b1ecda8b28e5de7eed695e199c66d1999297d` |
| `HANDOFF/plans/V040_ORCHESTRATION_PLAN.md` | `blocked-by-metadata` | `gate-fail` | `da192f6947d1ae579b72ccf4926a32e599ed9d8ed15463cb8a2af0ccc5160352` |
| `SCM-Q-ff9d1b59f2dd` (path hash `ff9d1b59f2dd138d72f8cb01a86c21aa6060e3066c7551f34cf790cf8c0bd29a`) | `quarantined` | `foreign-alias-in-bytes` | `d56629615bd304546760181158feb5434c3e33036dc7a1b7111deac1886ff4ec` |
| `HANDOFF/plans/V040_RELEASE_NOTES_DRAFT.md` | `blocked-by-metadata` | `gate-fail` | `fb48eedd0ea6fdc5d04614060178a235931fe6ee68b92f299bce737f6f4f1778` |
| `HANDOFF/plans/V040_V050_FIVE_NODE_GATE_PLAN_2026-08-05.md` | `blocked-by-metadata` | `gate-fail` | `f6c092aa645a9ba984737db930fdb0a02e5f340e52b7e4e86b576fc2da762d82` |
| `SCM-Q-4c9ce1c21428` (path hash `4c9ce1c21428949c0e055f6cf07a8e8ae156b13f0a3df1d555b07e32a626960e`) | `quarantined` | `foreign-alias-in-bytes` | `26c3f4054e2ba65fa1dd18f6086a96eda919a389c759185bac3ae7690759d01f` |
| `HANDOFF/plans/android_emulator_setup_plan.md` | `blocked-by-metadata` | `gate-fail` | `b5ca8316a58651b6e6aec4fc3f535197e87bdfa6921b565b2d1d64c9abd95af1` |
| `HANDOFF/plans/cli_orphaned_modules_cleanup_plan.md` | `blocked-by-metadata` | `gate-fail` | `572aacd136c575f8541971dc7c7441b4aa744c28b4e2d38c2719b40ef7eecb99` |
| `HANDOFF/plans/core_ble_gatt_traits_cleanup_plan.md` | `blocked-by-metadata` | `gate-fail` | `ec400eac3f8ee2062a2ae101bd389b8add4d95a8b9e0f5334e5247b2aed86256` |
| `HANDOFF/plans/identity_state_regression_fix_plan.md` | `blocked-by-metadata` | `gate-fail` | `0271d4cb9bb5e40e00ba2e649174f04ccbf121c9af43c64606326d68790e4847` |
| `SCM-Q-b03d9e39a53b` (path hash `b03d9e39a53b05a6cd8a4c4d883557d45844b4c01646af30cf9700b7ece8ebf4`) | `quarantined` | `foreign-alias-in-bytes` | `0cd1bce7314458d3b2f91232ae6a5f4d7299564e99433d5dca229d99d475d63d` |
| `HANDOFF/plans/smart_transport_router_params_cleanup_plan.md` | `blocked-by-metadata` | `gate-fail` | `bd4710614b0543fc94d4d1da5440fda011a3830972979b860917a121e14eff47` |
| `HANDOFF/research/2026-06-05_DYNAMIC_PORT_DISCOVERY_RESEARCH.md` | `blocked-by-metadata` | `gate-fail` | `73ae233c04585ce2aed94f2bff6f9f7f76cb07962b70f4a3e692011f5613289b` |
| `SCM-Q-afcb3720a59a` (path hash `afcb3720a59ab479d25639b7ee33766cc74ba3761f45cf9912934ca86f53607b`) | `quarantined` | `foreign-alias-in-bytes` | `bcc659c9b4f8ba3233f3e52a73423d0997b84b9467f6d453839e5d2a9beac7c9` |
| `SCM-Q-6d495db7e6e3` (path hash `6d495db7e6e3492400868768cc8193ed6d29f5ff993c4f9987f568ca6b4b1943`) | `quarantined` | `foreign-alias-in-bytes` | `e80b3b5ec4d1374a93bd8fb538f17d5db54e6d4dbb6ca1e831604e8be41ce6b0` |
| `HANDOFF/results/alice-peers.json` | `blocked-by-metadata` | `gate-fail` | `d5a5eaa70b60faf467ddb24bd2d2a87e4646f4225e417146251f5440489aa525` |
| `HANDOFF/results/alice.log` | `blocked-by-metadata` | `gate-fail` | `4778b4d07ddc67fb2885776f49e81ceb840834b8706028aaa3f6690d82765e14` |
| `HANDOFF/results/bob-peers.json` | `blocked-by-metadata` | `gate-fail` | `902ae938a96915b5c4567ad5901a4828beb963b1cbb640219baaa88b74418744` |
| `HANDOFF/results/bob.log` | `blocked-by-metadata` | `gate-fail` | `b58d9aa13ac2f2ebcf3bfd772f7974218d0b1ddb392606f968bf61d1869a25b8` |
| `HANDOFF/results/carol-peers.json` | `blocked-by-metadata` | `gate-fail` | `30f1a2faa9ec36cf0ac753f118d9353340c234ba38fe53ecade4ef8a339656e7` |
| `HANDOFF/results/carol.log` | `blocked-by-metadata` | `gate-fail` | `411fbb62909faf74f1bcef24e483b1e7ddd005c6e72196870d5da80aa58866b2` |
| `HANDOFF/results/david-peers.json` | `blocked-by-metadata` | `gate-fail` | `19907636021969d85f553425307b1f7582928002cc90fb68f157c2ef69dcfbdd` |
| `HANDOFF/results/david.log` | `blocked-by-metadata` | `gate-fail` | `bd0029539ec206dac68f0de2b001ac674906d88beeba6ce913d4b8bcd3bd2cde` |
| `HANDOFF/results/docker-stats.txt` | `blocked-by-metadata` | `gate-fail` | `938aeb549269636e6cca003854ca4f6e5cf0dfa6528177d912ebaf16cabb3877` |
| `HANDOFF/results/eve-peers.json` | `blocked-by-metadata` | `gate-fail` | `2c6e5bde91872a2c27b46558a203ad7a2e6d474b82f96598bcd2800abb733a61` |
| `HANDOFF/results/eve.log` | `blocked-by-metadata` | `gate-fail` | `bb470959c07f3db39f16e5722326b37978c3ff170772a621a24df31274edc0d2` |
| `HANDOFF/results/relay1-peers.json` | `blocked-by-metadata` | `gate-fail` | `a77a22bcd0673ebf2dc4232068a24ab338d2b156509b30135ec33c622b7a7b02` |
| `HANDOFF/results/relay1.log` | `blocked-by-metadata` | `gate-fail` | `c7aa6588f5493f49b0942cf8c239c55d4f45e18291e485d164bb55b557dcf7d3` |
| `HANDOFF/results/relay2-peers.json` | `blocked-by-metadata` | `gate-fail` | `7e7a05d195dae25a95e3cbadd2edd849d6a62094b5dc85be471a8b7900fbfc9e` |
| `HANDOFF/results/relay2.log` | `blocked-by-metadata` | `gate-fail` | `49feac8df09177994f873a2ccea2b6610b2717141dfbcc7a34253643d1852255` |
| `HANDOFF/results/stress-test-summary.log` | `blocked-by-metadata` | `gate-fail` | `3b8db4bb80b01ed897ab14cd03cf5dc73bacc470be11ab5b3e910069916a7f5e` |
| `HANDOFF/retired/BATCH_AND_DELIVERY_001.md` | `blocked-by-metadata` | `gate-fail` | `4304b75103caa96d3cc3e24c16371ce1e4eeb9fa0217f199d21fb65c8033e067` |
| `HANDOFF/retired/BATCH_AND_NO_ROUTE_001.md` | `blocked-by-metadata` | `gate-fail` | `441943687ecf72589daf072caac6806d1196c5a899100b6e5c55a7500110267b` |
| `HANDOFF/retired/FOR_BETA_SWEEP_B2_POST_REJECT.md` | `blocked-by-metadata` | `gate-fail` | `30442602c92795fdad2728fe72d6bb3f786430970ab43b96c8fb791d520367b4` |
| `SCM-Q-d2b6926c1d38` (path hash `d2b6926c1d38dec494f9cae8cea962344664f3caee81161ffdd8e9bc1b583355`) | `quarantined` | `foreign-alias-in-bytes` | `9f0303fcff630dd5f84f09f46b86d560984a10aa2a2b55a8c13276f0bd6fc90a` |
| `HANDOFF/retired/[VALIDATED]_task_p1_multidevice_blocking.md` | `blocked-by-metadata` | `gate-fail` | `81896bfd0c7fa1c207e66aa9d1285ae98c131832793e496f7579e087185bfbb9` |
| `HANDOFF/retired/dupes_2026-07-17/A-04_U5_Android_receipt_unificatio.md` | `blocked-by-metadata` | `gate-fail` | `e5a9a8ab489d10e3760822106d451f48c395145a8818cac0702d2c01db5b09ad` |
| `HANDOFF/retired/dupes_2026-07-17/A-05_U6_iOS_receipt_unification.md` | `blocked-by-metadata` | `gate-fail` | `7c91a4204e43986d68ccfb17272fef3f105e44451364c48c141c1afa95f09485` |
| `SCM-Q-a8b658c820fd` (path hash `a8b658c820fd98ca00d987b8b8437654d9bf5f8415016e1fa2ed9aad55666811`) | `quarantined` | `foreign-alias-in-bytes` | `0bc48c08bfdafe964c363a24e29d20c28ee1ea7a68f62fb5595e38660dbedfbf` |
| `HANDOFF/retired/dupes_2026-07-17/E-02_PQC_07_FORCE_RATCHET_SAME_DEFE.md` | `blocked-by-metadata` | `gate-fail` | `b93eadf921f9ac455642e2204739624b77d8ab852e5b0fb2cc31ccecb9630c93` |
| `HANDOFF/retired/dupes_2026-07-17/E-04_PQC_07_WIRE_RATCHET_STEP_wire.md` | `blocked-by-metadata` | `gate-fail` | `1187df82371c1538387c15ebce586421adf287dc27d8ccbbb39c855e06d0ff6d` |
| `HANDOFF/retired/dupes_2026-07-17/U2_topic_constants.md` | `blocked-by-metadata` | `gate-fail` | `0e3a16ff82c8a9c258e8afcd6e01a681a1a548f6c20255efc7a9db994a7fefb7` |
| `HANDOFF/retired/dupes_2026-07-17/U3_retry_policy.md` | `blocked-by-metadata` | `gate-fail` | `532975df9f2c3ef93c66ed3911a81b76703d0c079352c349ee87bb67f6f3168b` |
| `HANDOFF/retired/dupes_2026-07-17/U4_receipt_encoding.md` | `blocked-by-metadata` | `gate-fail` | `35ffd8aad81e2f5cac23d083bd5b4fb423835a73027d2c78ff22361e4dbf0a30` |
| `HANDOFF/retired/task_wire_force_state_for_test.md` | `blocked-by-metadata` | `gate-fail` | `d14d82e3ca106905340c6c20bac1ee2bf0d352b9268a9d262200aa7cc2889b0a` |
| `HANDOFF/review/A2_DURABLE_DELIVERY_DESIGN_PANEL_R1_2026-08-10.md` | `blocked-by-metadata` | `gate-fail` | `f8a8ea22044a6478ec03f34eb90092f492875b7dc029234c3330bd042a45b3ca` |
| `HANDOFF/review/A2_DURABLE_DELIVERY_DESIGN_PANEL_R2_2026-08-10.md` | `blocked-by-metadata` | `gate-fail` | `c98ea0047f8403f23e79a9fad0bc5f810f0630ab7f490460d5cb2de23c9ead08` |
| `HANDOFF/review/A3_ROUTING_PEER_SEEN_ANALYSIS_2026-08-10.md` | `blocked-by-metadata` | `gate-fail` | `b1f3b8f007cca6ee62aba2f0df932db002837da4faa4b5b48b921eb9038d147b` |
| `HANDOFF/review/ADVERSARIAL_RFC1918_DISCLOSURE_2026-08-06.md` | `blocked-by-metadata` | `gate-fail` | `9d76c1313dfc71841a311d7401cb29b0fbfa2112e7f8a7021b4c34fe0ef62fd5` |
| `HANDOFF/review/ANDROID_LEDGER_VISIBILITY_ROOT_CAUSE_2026-08-09.md` | `blocked-by-metadata` | `gate-fail` | `55bc85df0614c77dc5a85c7d215fa2d6c9b1447db96339ec859e403e55614972` |
| `HANDOFF/review/B1_DNS_HARDENING_ATTEMPT1.patch` | `blocked-by-metadata` | `gate-fail` | `3d5598d2382166a1695721f510add2e7e53da2fb309a3d5d2a9b838176861c8a` |
| `HANDOFF/review/B1_DNS_HARDENING_REVIEW.md` | `blocked-by-metadata` | `gate-fail` | `40900e26ee0c1e6bcc8e39c889baad515ce3223448ba682e9d7c9e3cb3f93f8b` |
| `HANDOFF/review/B1_DNS_HARDENING_STATUS.md` | `blocked-by-metadata` | `gate-fail` | `72e33287ffa2720eafd1f9095b5961cf55d4243a57b926fcd94a851946c74865` |
| `HANDOFF/review/CORE_BLOCK_GATE_ADVERSARIAL_REVIEW_2026-08-04.md` | `blocked-by-metadata` | `gate-fail` | `bb8b8aaa8af830fbd6337a44a82827482f89c6b46bdc225fb907e0d11c1c83cc` |
| `SCM-Q-56ed5b4054e3` (path hash `56ed5b4054e33da918d96de9dc3721fcc14482155579331a74799b145d809234`) | `quarantined` | `foreign-alias-in-path-and-bytes` | `20ae1506627079a08c03c8bb62a7735fbc6319a2d37bc5a3b2a0d23c10c92499` |
| `HANDOFF/review/E00_WIRING_IMPL.md` | `blocked-by-metadata` | `gate-fail` | `6218abdb7243fa3cfa391add6ec3728d06e198b89eb938b07f60d5bc94e77fa1` |
| `SCM-Q-7499b3bf685f` (path hash `7499b3bf685f92ead39ff2c0b86be892f0cae81771f34aa3c5745f69c898b92c`) | `quarantined` | `foreign-alias-in-bytes` | `012c43dbe242029b2530dcf740956de81c406612514bde54a52d3edd4109b87a` |
| `HANDOFF/review/E01a_attempt_constraints.md` | `blocked-by-metadata` | `gate-fail` | `43e60766a4b00352c4afedc51c9e52cbe805bb0e2c198465404fc6491048337a` |
| `HANDOFF/review/E02_FORCE_RATCHET_REMOVAL.md` | `blocked-by-metadata` | `gate-fail` | `02d80c81924050905e937164a249ad29fc172718d7542067d857121e4838b53a` |
| `HANDOFF/review/FOR_ALPHA_FIX_B2_MANIFEST_DEFECTS.md` | `blocked-by-metadata` | `gate-fail` | `752e2e3c8cc72df8087c90c526a932addcdc0e9df42ecd40a9db2ba346751e85` |
| `HANDOFF/review/FOR_ALPHA_WIRE_B2_MANIFEST_REANCHOR.md` | `blocked-by-metadata` | `gate-fail` | `d3f8c25d6ddc3386d50e62b1cf0b854db787edaf16ce5e65f38a051e2a9f1fa3` |
| `HANDOFF/review/IN_PROGRESS_task_security_tooling.md` | `blocked-by-metadata` | `gate-fail` | `ace4ae5c366d6f095f29df6d8078718403f69a08599829072990a5f091f94a89` |
| `HANDOFF/review/LEDGER_SEEDING_ADVERSARIAL_REVIEW_2026-07-25.md` | `blocked-by-metadata` | `gate-fail` | `fa1c111b8932a957248abf0a3eda8a9e956165ffdf0abb8bfbfcc90142858e55` |
| `HANDOFF/review/LEDGER_VISIBILITY_AUDIT_QWENPAID_2026-08-05.md` | `blocked-by-metadata` | `gate-fail` | `d624ebb11906905768991705226a6a187800ebf38d63810c1fdf70ee9ab7a1cd` |
| `HANDOFF/review/LEDGER_VISIBILITY_AUDIT_QWENPAID_2026-08-09.md` | `blocked-by-metadata` | `gate-fail` | `97f9fc981816eb366b2a5c7eea79fb19b0edc05e4a964c8b7ff917ca3048f5ec` |
| `SCM-Q-de0a89378f35` (path hash `de0a89378f352a02e6fb714b69db6fcb019fdd5ec359cc63552079edda6d0798`) | `quarantined` | `foreign-alias-in-bytes` | `9a5520b7d06f28f61841a9de22dc8ac52e1d603c591e80852e7b377a30fc63b5` |
| `HANDOFF/review/OUTBOX_FLUSH_ATTEMPT_296LINES.patch` | `blocked-by-metadata` | `gate-fail` | `1c9ce5764217c55dec24f68dfc226ccd7a8ecd54608b7dc8fb905a8dc3ef9750` |
| `HANDOFF/review/P1_VERIFICATION_GATE.md` | `blocked-by-metadata` | `gate-fail` | `a742fbcb3d199500134b1dcc9f1f7882b65643e34bac9fef43a5b9326f38e8ca` |
| `HANDOFF/review/P2_VERIFICATION_GATE.md` | `blocked-by-metadata` | `gate-fail` | `419ad02e62f49c907f1ccd4a01bab8c3f71261f33970682779e049d42d0d6cba` |
| `HANDOFF/review/P3_VERIFICATION_GATE.md` | `blocked-by-metadata` | `gate-fail` | `9ced31f5494379d47b4b2e8932df837f95aca580e38c67b7fe8e37d4a207ad51` |
| `HANDOFF/review/P4_VERIFICATION_GATE.md` | `blocked-by-metadata` | `gate-fail` | `4b1f9400f249fa41363cbe8ee0efc5fa1b4bfe146f7716a0ea0013b47f80aedf` |
| `HANDOFF/review/P6_STATUS_PENDING_P5.md` | `blocked-by-metadata` | `gate-fail` | `8433cb2d6059814540249353e897206b42561baf65baffee81d9c3b83565d5ab` |
| `HANDOFF/review/PHASE0B_MSGREQ_GATE_REVIEW_QWENPAID_2026-08-04.md` | `blocked-by-metadata` | `gate-fail` | `6bdc55d46183239309785c834c5dea444fe36694153e9a3b34adfa7e9d8cdd3a` |
| `HANDOFF/review/PQC_05_06_07_ADVERSARIAL_REVIEW.md` | `blocked-by-metadata` | `gate-fail` | `672349243fa08469c4ac6e4b6ab25adc59278246e071d3130faa7bc2380ac7ac` |
| `HANDOFF/review/PQC_07_ATTEMPT2_ENCRYPT_WIRING.patch` | `blocked-by-metadata` | `gate-fail` | `8858a15c25bb63345aa37f1c3eef18dbf5649a0790c53380a565c5b8443ec43b` |
| `HANDOFF/review/PQC_07_ATTEMPT2_FUSION_LITE_VERDICT.json` | `blocked-by-metadata` | `gate-fail` | `e2ba4a436b0e40f7e6db518eeaa4ee5933060ac0ffe52009cfbf034a4e66613c` |
| `HANDOFF/review/PQC_07_ATTEMPT2_MIX_PQ_SECRET_DESIGN.patch` | `blocked-by-metadata` | `gate-fail` | `62f83ccc9c2f95b38616660aca07dcb1ac8f5cac78f3b0edafa366a0351042a0` |
| `HANDOFF/review/PQC_07_ATTEMPT3_DRAFT.md` | `blocked-by-metadata` | `gate-fail` | `584c0e7ecd745df280c823134fcd7a566440ff60df6c8e922b26b59f4b8c8182` |
| `HANDOFF/review/PQC_07_ATTEMPT3_REVIEW_VERDICT.md` | `blocked-by-metadata` | `gate-fail` | `c2a9f5a60ef508f567153c44b6037abccb575adca61816a4d84c76f3680fe3f0` |
| `HANDOFF/review/PQC_07_ATTEMPT4_DRAFT_UNREVIEWED.md` | `blocked-by-metadata` | `gate-fail` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `SCM-Q-0a9d739b4b0b` (path hash `0a9d739b4b0b8591fd418cd1cb8ed183cef09ef801b4bbbc672d08bd3aa0300e`) | `quarantined` | `foreign-alias-in-bytes` | `0884f56cb74c4bb61ad0e255dce60b55484074ba0f1ac81060a09e149a294f6b` |
| `HANDOFF/review/RECEIPT_GAP_ANALYSIS_2026-08-10.md` | `blocked-by-metadata` | `gate-fail` | `ba7225c89f537b98fd17b22a31b05f820bf806976634e622017de96b786ef063` |
| `SCM-Q-07200f0b4f2a` (path hash `07200f0b4f2ab5172a1cb566ee9162ee0d1da429307fe5680597cd7ee77fbdf6`) | `quarantined` | `foreign-alias-in-bytes` | `34ce7bba10adf10f2acbd718d05452550fedc51097e88aed46c0eed13b5151d8` |
| `SCM-Q-469ead2c9ccd` (path hash `469ead2c9ccd40cfcf280fffa5bf42c74e61fea660117f50d7fa4998877c5b37`) | `quarantined` | `foreign-alias-in-bytes` | `a4abb394bd958cfba141578a3304624b39d7565e3d4f47858c954a066f9b2154` |
| `SCM-Q-7b77803ae942` (path hash `7b77803ae942527180ca85d86791dd2671639433e8a4b10e1f9e4764f3c07210`) | `quarantined` | `foreign-alias-in-bytes` | `f58ff9fe8980bc0ed350abc2ad999a211da32c1c6ab273955736a8b54a9a39f1` |
| `HANDOFF/review/SCOPE_INVENTORY_2026-09-24.md` | `owner-valid` | `gate-pass-after-write` | `c2f7bb35e15b51c11fc18ae9f71521a83d3936d480a630f1b09c4c1468a48045` |
| `HANDOFF/review/SCOPE_OWNERSHIP_RCA_2026-09-24.md` | `owner-valid` | `gate-pass` | `fa5de7201e07073297783fb4a4687fe6b847d381a7725e3e9cbc7dc0ec8f8b91` |
| `HANDOFF/review/T02_AWARE_PORT_TLV.md` | `blocked-by-metadata` | `gate-fail` | `adadd8bad169ad808672aaf7e9f60172f7236c4eab11a03751cce24f8c3d219e` |
| `HANDOFF/review/TRANSPORT_FAILOVER_AUDIT_QWENPAID_2026-08-05.md` | `blocked-by-metadata` | `gate-fail` | `a3609d6e00fa1f92e463b59c99697948f1fd4416f3edf6bf9d542e6686cee944` |
| `HANDOFF/review/TRANSPORT_LIVENESS_REREVIEW_PASS_QWENPAID_2026-08-05.md` | `blocked-by-metadata` | `gate-fail` | `066e943371711c60b2f79cf2ed2f9c9748930e02dcf9b2be05d98d45b4966982` |
| `HANDOFF/review/U7_SCHEMA_DRIFT_AUDIT_REPORT.md` | `blocked-by-metadata` | `gate-fail` | `a35a84c134492de936e2579d6c90986e909db1ae34fbeb6d6e71b0b4e294e5c5` |
| `HANDOFF/review/V040_BASELINE_FREEZE.md` | `blocked-by-metadata` | `gate-fail` | `89be9b872dde9022eea46fe8ad5690501528404f7a228fca3e09581e72cd9c45` |
| `SCM-Q-ddd8472851a7` (path hash `ddd8472851a7427a09518d682db8851ec4cd6a2ddb446d9b59a9e1f241759887`) | `quarantined` | `foreign-alias-in-path-and-bytes` | `024b53f3116d241b4bbc8ca4518b4763180d51d7df2e05f49dfe94526b295867` |
| `SCM-Q-82847a83ce61` (path hash `82847a83ce612cc6838ee574abc71a773c2121480c23995e3f5dae377dc47471`) | `quarantined` | `foreign-alias-in-path-and-bytes` | `0342ca36c6bc65d0278f64e3cfe4ad8a00b95f8eb9368d6313f7bcfecea824e6` |
| `HANDOFF/review/V040_CANDIDATE_272_FINAL_APPROVE_QWEN_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `9cccce8919b86e89524b23557d943a6dfb3aec06df1c6b2986d4463adc1d55db` |
| `HANDOFF/review/V040_CANDIDATE_272_REVIEW_QWEN_2026-09-02.md` | `blocked-by-metadata` | `gate-fail` | `e2d823570e1380bdf6ddb494153c96a7b140c66296b1e24cf4acac56039ce813` |
| `HANDOFF/review/V040_CANDIDATE_272_REVIEW_QWEN_R2_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `6889b78ba9a8984029ae65aa5783826551eab5b7ac64cf750466ef7dee044183` |
| `HANDOFF/review/V040_CANDIDATE_272_VERDICT_REPIN_85cb4c67_2026-09-07.md` | `blocked-by-metadata` | `gate-fail` | `587012a8ed140dbfaa05e0aa611c755e8449a6eae86ceafddfcb6c8027b90d25` |
| `HANDOFF/review/V040_CANDIDATE_272_VERDICT_REPIN_b0f7ac4e_2026-09-06.md` | `blocked-by-metadata` | `gate-fail` | `0e195c9c71692c642fef92deaa2d4d8d7ae19022f7babaefee78462ce6b039bb` |
| `HANDOFF/review/V040_D10B_POISON_LISTENER_GUARD_REVIEW_2026-09-10.md` | `blocked-by-metadata` | `gate-fail` | `8c6fbb9603fe392c19266400b79a06bc2b61c1acfeb5fb21929ba3ae3db703ac` |
| `HANDOFF/review/V040_D10_RESERVATION_BASE_REVIEW_PACKET_2026-09-10.md` | `blocked-by-metadata` | `gate-fail` | `794f7accafac08e313a1e64fe2896496ae21803adaadaee3a0ee598047f87e33` |
| `HANDOFF/review/V040_D10_REVIEWER_DISPATCH_PACKET_2026-09-10.md` | `blocked-by-metadata` | `gate-fail` | `8fb96a84b4a7efcb6b63de8a89258c8d84d761ce49a72ab61e729e34f28d9769` |
| `HANDOFF/review/V040_D2_BLIND_B_ADVERSARIAL_VERDICT_2026-09-13.md` | `blocked-by-metadata` | `gate-fail` | `71519dce664eddbfadc4f4f7d201b1fb72e37c4d43fbdf72cc556dc3bcc14dec` |
| `HANDOFF/review/V040_F6_BLIND_B_ADVERSARIAL_VERDICT_2026-09-13.md` | `blocked-by-metadata` | `gate-fail` | `431eb0641cb2548ecf82bab5d907a8d18df3045d8a36f56e7fda0dba73ec267e` |
| `HANDOFF/review/V040_FINDING_DISPOSITIONS.md` | `blocked-by-metadata` | `gate-fail` | `29445aed3166d74585abf66fbd25f8e19d6cab7ac7706d4fbd899eada3284705` |
| `HANDOFF/review/V040_MULTI_TRANSPORT_STORE_FORWARD_RULE8_REVIEW_2026-09-14.md` | `blocked-by-metadata` | `gate-fail` | `59bdc1b8f96fa46ec96df01ce12c63f8f039eb99c384ab8e14aa93a42d97018b` |
| `HANDOFF/review/V040_NIMBLE_PEER_REVIEW_QWEN_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `de46900e666dc3cdf9e173edbfb57d0f51faf6ed502fa3cb68a42349751e0f39` |
| `HANDOFF/review/V040_NIMBLE_RECYCLE_REVIEW_QWEN_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `d03ccc45f2e4eb91b1b6144313063fc42ee54444e4f5c1727403df28cdefc444` |
| `SCM-Q-c9c38ba11a48` (path hash `c9c38ba11a484ab8bba05ea34303842913b5e2e983e097fe1b62a65861aa388e`) | `quarantined` | `foreign-alias-in-bytes` | `f885efdb2de0c085373dd87b8a5c4827b638aed782bbbe7bd663852957b2abc6` |
| `HANDOFF/review/V040_OUTBOX_FIX_REVIEW_QWEN_2026-09-05.md` | `blocked-by-metadata` | `gate-fail` | `e5de7e5fd3832af5bf01635bd7b362f185b5a69894164e2540a0a52a11343c9d` |
| `SCM-Q-aac02112a3a6` (path hash `aac02112a3a60c42e7fc354cdc3ce04f084ecd8531776c455162bc0eb47273de`) | `quarantined` | `foreign-alias-in-bytes` | `ad478dec28ca18c94022ffd30d6dc0fbeadeb5f9918330a14530bda8eeebce86` |
| `HANDOFF/review/V040_PR278_ANDROID_ANR_REVIEW_APPROVED_bfe6bc0b_2026-09-07.md` | `blocked-by-metadata` | `gate-fail` | `c731611c2cd623b610f8e93c0e9413d2bf02947500389101e8f81609efc0d506` |
| `SCM-Q-1e883a3cd53a` (path hash `1e883a3cd53a1ae3aaccdc76d5fd1678052caa52e7c23ac07f0026bc5e70792d`) | `quarantined` | `foreign-alias-in-path-and-bytes` | `21200ce288c3322c89c173d94107575e63f3265acbb140eb319de8fc6cbb0ec1` |
| `SCM-Q-d381abd12f5e` (path hash `d381abd12f5e2841b5b6a654813d587b332134a77cc8146e6eb006f8c0e02c60`) | `quarantined` | `foreign-alias-in-bytes` | `b28a26d28bf5c2514f1624464b9a0894a6d7c7e31e8e66ee850dd227ee8d7cd8` |
| `HANDOFF/review/V040_S3_RELEASE_MECHANICS.md` | `blocked-by-metadata` | `gate-fail` | `db1bf1a83ba8acc7ce72a64bba7d3a853e5bfbfcca06da85b417ea60bce2d511` |
| `HANDOFF/review/V040_S4_DELIVERY_PROOF_RUNBOOK.md` | `blocked-by-metadata` | `gate-fail` | `56068f3f00fcff9b4dce5133456fb136a7b2718b00406727d2dbc5b93d972cfb` |
| `HANDOFF/review/V040_S5_JOSH_WAN_RUNBOOK.md` | `blocked-by-metadata` | `gate-fail` | `854078d8a884af6c366fa3fa94794c01f62145e8fd1031764e92e5a502ef6f3c` |
| `SCM-Q-7e30d7e59bc3` (path hash `7e30d7e59bc326cefb6849b3d083ad18a44bec6814ad78be4e7b947fbc18d857`) | `quarantined` | `foreign-alias-in-path-and-bytes` | `4558cfcb403ac20300bd56296b96fca695ca4b5af00c918a0a86427d360cfb78` |
| `HANDOFF/review/V040_T13_F7_REVIEW_QWEN_2026-09-01.md` | `blocked-by-metadata` | `gate-fail` | `7eb213f1da64b566b52e27967a8e68e09cd7889a5e30e4e9db0b49290d08681b` |
| `SCM-Q-4d45391c100b` (path hash `4d45391c100b0e65f322fd5a97cb51d3d655a30d6f4d51ffc0305a8a8e7b4fb1`) | `quarantined` | `foreign-alias-in-bytes` | `03e0a2820c7a0255bf883ff8c27cc72550c6460a8d3de7b6687b04d658e9d1ae` |
| `SCM-Q-d487c14efddd` (path hash `d487c14efdddfcb0085d17d448790c41d61e22b01531048c6ad47e7cfb61dde9`) | `quarantined` | `foreign-alias-in-bytes` | `fe228abf6cb2c9826b5e4fd2b9d0f3e2c2b6f8d21860b450c3c39fdcf7a611a9` |
| `SCM-Q-444579f287b6` (path hash `444579f287b6b605557d5f02ef35006c42bbadbab155a9299da3e80233fcb8c4`) | `quarantined` | `foreign-alias-in-bytes` | `c69462ef7637ae48c90531ec6dcccd176be41db602ac41255e844162b39237d4` |
| `HANDOFF/review/V040_T13_FDHT_FINAL_APPROVE_QWEN_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `541e54d974dba38ddf3cbee94ba420cf2b7207135dd23f40b85f154a63627b63` |
| `HANDOFF/review/V040_T13_FDHT_RECHECK_QWEN_2026-09-03.md` | `blocked-by-metadata` | `gate-fail` | `7f1349f0dda44a99136a71301d4d484f6bcf739e9716cffe2b22e5c12691ba78` |
| `HANDOFF/review/V040_T13_FDHT_REVIEW_QWEN_2026-09-01.md` | `blocked-by-metadata` | `gate-fail` | `ae60191cc8643882b413c2fd023d92f68e207e2a071e625dc6093ee78933a74b` |
| `SCM-Q-63be034705e7` (path hash `63be034705e77051f6487bc01f9449ce08d669232036e02bbc83823f76b46dcd`) | `quarantined` | `foreign-alias-in-path-and-bytes` | `14a16972443735f5249f2a6cadc7742afee710d6c0690e0e74cbde4998ce5478` |
| `HANDOFF/review/V040_T14_EPHEMERAL_REVIEW_QWEN_2026-09-01.md` | `blocked-by-metadata` | `gate-fail` | `2c6e9a5f3f912cb4bc5afb66cb5a9b589ef86dfa4735ca5088c0a9d547331da1` |
| `HANDOFF/review/V040_T14_PREEXISTING_REVIEW_QWEN_2026-09-01.md` | `blocked-by-metadata` | `gate-fail` | `7eb213f1da64b566b52e27967a8e68e09cd7889a5e30e4e9db0b49290d08681b` |
| `HANDOFF/review/V040_T1T2_BLIND_B_ADVERSARIAL_VERDICT_2026-09-13.md` | `blocked-by-metadata` | `gate-fail` | `9a541880b05270e2b364c9f017960f38754235677a33a284671b757a55d74fa1` |
| `HANDOFF/review/V040_T4_BLIND_B_ADVERSARIAL_VERDICT_2026-09-13.md` | `blocked-by-metadata` | `gate-fail` | `7e237ea24f68399274957362945894247344a9fcaf01e77c02f3e72a0785dc1b` |
| `HANDOFF/review/WINDOWS_ANDROID_PROBE_2026-08-10.md` | `blocked-by-metadata` | `gate-fail` | `c2e94632f27a5d34653b91ad70faeb169d8edd76e23960bf8b1df0c2099dae98` |
| `HANDOFF/review/WINDOWS_INSTALL_ROUNDTRIP_RCA_2026-09-24.md` | `owner-valid` | `gate-pass` | `09cc47b48f5fc455493c1776ff6be099e326c267a88578c9798f154b335f2e04` |
| `HANDOFF/salvage/agent-ae6817c8e4bd576e2.untracked.txt` | `blocked-by-metadata` | `gate-fail` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `HANDOFF/salvage/agent-aeb9e43e7b5943b69.untracked.txt` | `blocked-by-metadata` | `gate-fail` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `HANDOFF/scmessenger_rust_audit_optimization_plan.md` | `blocked-by-metadata` | `gate-fail` | `e114fd241b2262e739bb3ed92b5a844c40291dab02d59f642ef407fde86c255f` |
| `HANDOFF/scmessenger_rust_implementation_prompt.md` | `blocked-by-metadata` | `gate-fail` | `4bf8d65df4c22e8737b6e315eb644862ce791570f2688f6b94381feac0b53f15` |
| `HANDOFF/scmessenger_rust_status.md` | `blocked-by-metadata` | `gate-fail` | `0a476f306fbdd9ea7cedd5eb57aeb13ab4c98ee2ebfe9bebfdb892aed19b281f` |
| `HANDOFF/todo/ANDROID_CI_APK_SIGNATURE_BLOCKS_INPLACE_UPGRADE_2026-08-09.md` | `blocked-by-metadata` | `gate-fail` | `111f14ec9dfb7982e7838ed2052e11d21edcf933d691a3ac98a09b58cc191c8a` |
| `HANDOFF/todo/ANDROID_FFI_IN_COMPOSITION_BUILD_KILLER_2026-09-10.md` | `blocked-by-metadata` | `gate-fail` | `addaacea6d7e09375bca9205ea967052d25326eef931e9119863a1dc63d0ba4d` |
| `HANDOFF/todo/ANDROID_INBOUND_CRYPTOERROR_2026-08-09.md` | `blocked-by-metadata` | `gate-fail` | `dd78930cc76de2eddb14536f916d85496b84a4ab333b80a858bae72d5155d4eb` |
| `HANDOFF/todo/AND_STOP_START_FLOOD_AND_CANCEL_2026-09-23.md` | `blocked-by-metadata` | `gate-fail` | `a128f11a8a04419ea90ce8d4a53d83ab873dbd418c6fa8dcd3696fd33313b6ba` |
| `SCM-Q-30e0dbb21dc4` (path hash `30e0dbb21dc4b0565c387c58a8d608f8c8558e253c46cc64523f35b1b753ec46`) | `quarantined` | `foreign-alias-in-bytes` | `1a75afcb9af5e45b15d81106cc02c5cfb2c7da664e8ebab8e875f1d641af873c` |
| `HANDOFF/todo/CODEBASE_UNIFICATION_PLAN.md` | `blocked-by-metadata` | `gate-fail` | `286fcf8220dc0981e977f538789a19bc8848b415c98dba84c481b27be83bad64` |
| `HANDOFF/todo/CORE_DIAL_CANDIDATE_DOUBLE_CIRCUIT_PRUNE_2026-09-10.md` | `blocked-by-metadata` | `gate-fail` | `7d34fae849afdd7c5f9c6ac53a2b2b4a298998ebc410108e917ae82bdafbc1a5` |
| `SCM-Q-13482fa6420f` (path hash `13482fa6420fea2ec0ec1a25181117a516cf18cf2e4aff3a9c85b2de62a150aa`) | `quarantined` | `foreign-alias-in-bytes` | `04509b460040d70c8729d6c49c68772df17a570477b043828df9401768c8a489` |
| `SCM-Q-a6ab291e6486` (path hash `a6ab291e6486bd85d43710c51df5767fc106433ad7ce8161bd481c36e23307de`) | `quarantined` | `foreign-alias-in-bytes` | `2cfc134648a7c066acf771f3409bb90753e288f2532c607a668e66176ab161ba` |
| `HANDOFF/todo/D3_SELF_RECIPIENT_POISON_GUARD.md` | `blocked-by-metadata` | `gate-fail` | `221202a39879138654abcf86333832f22f795d6066a58cd6be4dc77d1ece5fb5` |
| `HANDOFF/todo/D4_MOBILE_BRIDGE_HISTORY_FLAVOR_MATCHING_2026-08-30.md` | `blocked-by-metadata` | `gate-fail` | `6711be6cbb63949cf78f9cd635b99770b26209890f47ba7bdc397de5addde035` |
| `HANDOFF/todo/D5_D6_BOOTSTRAP_NODES_PERSISTENCE.md` | `blocked-by-metadata` | `gate-fail` | `d0cd6ab19466cd17fb058743a02d57ee1ef90f6c198827165213a444f83b9ca7` |
| `HANDOFF/todo/D8_CUSTODY_DELIVERY_RECEIPT_STATUS.md` | `blocked-by-metadata` | `gate-fail` | `d89845e59303ae9b239f16e5a7f461852fba5e445f634576fb71522dec23f1aa` |
| `SCM-Q-c8b603aec24a` (path hash `c8b603aec24a79d93918ec0a8a75bd024c7a658b65124af19a962e8d00956149`) | `quarantined` | `foreign-alias-in-bytes` | `d7ef28670fddccd0910823daf610e2a1d609a258d259e160c4d8c2807b23af61` |
| `HANDOFF/todo/DEPENDENCY_DEBT_TOOLCHAIN_UPGRADE_2026-08-28.md` | `blocked-by-metadata` | `gate-fail` | `64983220b71d2ccad13cc77b680b057de6f58ac6464f2cd87f32520e2d22c6d7` |
| `SCM-Q-424d4812ea96` (path hash `424d4812ea968fb61dddc03dc7201e25d13fd752335b37506df03efaf326cbb6`) | `quarantined` | `foreign-alias-in-bytes` | `af427edb16eb8b100bfd1b2612b2e0ff6ee14accb75892c14dccaa7b4c58d5f0` |
| `HANDOFF/todo/INBOX_2026-08-11T042629Z_0fba9fce75be.md` | `blocked-by-metadata` | `gate-fail` | `082711000ed07ca8c52f2a2b490e5351b6909596e1e3a87a8ae114d935325f2d` |
| `HANDOFF/todo/INBOX_2026-08-11T042629Z_1e8b7f5589dc.md` | `blocked-by-metadata` | `gate-fail` | `eb478a855d74c54628cfb804731423fa439f49229f8bfb637cc82dd98a6c9131` |
| `HANDOFF/todo/INBOX_2026-08-11T042730Z_44bc1aa2459b.md` | `blocked-by-metadata` | `gate-fail` | `7ae42f673ace218da77c2dad32c724a54bf33114778d645c399d926913400339` |
| `HANDOFF/todo/INBOX_2026-08-11T042834Z_759c6034ba12.md` | `blocked-by-metadata` | `gate-fail` | `5a064e4fb150b140ad25708377ea65985f0126f6d6db2c07cc56faf7166b69cc` |
| `HANDOFF/todo/INBOX_2026-08-11T042934Z_fbe0b5016e86.md` | `blocked-by-metadata` | `gate-fail` | `c96cc5abd384b1e48378666e45d53d199d5381882a9ba123a729092065288c27` |
| `HANDOFF/todo/INBOX_2026-08-11T043034Z_e27f503e7b52.md` | `blocked-by-metadata` | `gate-fail` | `86bfe668801cf84233c50048de58ac4384a89d25e0192d3dbec2d7c9ed353f74` |
| `HANDOFF/todo/INBOX_2026-08-11T043332Z_3007a47066c5.md` | `blocked-by-metadata` | `gate-fail` | `a08932709f90cb39242e6e39c0be19cd9adda5543c00da2c10d91cfd73277993` |
| `HANDOFF/todo/INBOX_2026-08-11T043333Z_e7a8b366ff27.md` | `blocked-by-metadata` | `gate-fail` | `b5447e13299b70bb2973c38f2c57c403011efa0e5921caad15eae014e47e464b` |
| `HANDOFF/todo/INBOX_2026-08-11T043434Z_b594ceab6132.md` | `blocked-by-metadata` | `gate-fail` | `4405337a9ac112fbb112bdc73a82dd6f6951e898e30340aa993648a9b83fc215` |
| `HANDOFF/todo/INBOX_2026-08-11T043633Z_281197e9ec9e.md` | `blocked-by-metadata` | `gate-fail` | `f4c6d6900feae7e24fb0d322f5d6b07e9df44552196d59eadc24509f7bcd0fc9` |
| `HANDOFF/todo/INBOX_2026-08-11T043734Z_c193811a57a6.md` | `blocked-by-metadata` | `gate-fail` | `cfc553e65fa36a29ef64fed4df698cee428cc4d331900c89f6433c87ea447ba1` |
| `SCM-Q-54ed827c950f` (path hash `54ed827c950faeab42a4a5f30de7f1714c63aa53878c867980db158a09907c68`) | `quarantined` | `foreign-alias-in-bytes` | `d987e9cb7fffe87664fffd7186c6f7806b252bfea15700ee70eb7c7452387baa` |
| `HANDOFF/todo/P0_ANDROID_FINITE_RETRY_ABANDONMENT_2026-08-10.md` | `blocked-by-metadata` | `gate-fail` | `c374543c977219ccfa107274bcb730dfa7a3d5ae19a4799e9ed7367f00c9c1e2` |
| `HANDOFF/todo/P0_ANDROID_SELF_RATCHET_RESET_2026-08-10.md` | `blocked-by-metadata` | `gate-fail` | `1f5710ad85a30588959b26ffd36e6e8235c02296f7a819a8bca9f83e51be831a` |
| `HANDOFF/todo/P0_SCMESSENGER_WIFI_DELIVERY_IDENTITY_TRANSPORT_CANONICAL_2026-09-21.md` | `blocked-by-metadata` | `gate-fail` | `ffbd449f4e867d6adb06c1f57a87280ed0aafa560eda369013d0b7187b1746dd` |
| `HANDOFF/todo/P0_SEND_CRYPTO_FAILS_VS_DELIVERED_2026-08-30.md` | `blocked-by-metadata` | `gate-fail` | `daf1cb7bfc1b68e4dc2fc877dd3b1288d61af8629603a77546c790c124c4941c` |
| `HANDOFF/todo/P1_ANDROID_CHAT_ORDER_CROSS_CLOCK_2026-09-17.md` | `blocked-by-metadata` | `gate-fail` | `cb93dd70c6471e8e1a41935d014cdb956f943f1ac5ff3e67ed22ebe2e7eeb2b4` |
| `HANDOFF/todo/P1_ANDROID_COMPOSE_CRASH_RECURRENCE_2026-09-15.md` | `blocked-by-metadata` | `gate-fail` | `f5f9d612ba97ddd2754fbe7e8d35e5d4268cf28e1efcaa0ec73cff661d229ae2` |
| `HANDOFF/todo/P1_ANDROID_UNIFFI_CURVE_CHECK_RELOCATION.md` | `blocked-by-metadata` | `gate-fail` | `d1fea76dc69d3da8d0f58a2d2c6adf96f65c00aed677ac94943a2c7fee0726c7` |
| `HANDOFF/todo/P1_ASYNC_DELIVERY_RECEIPTS_DO_NOT_CONVERGE_2026-08-10.md` | `blocked-by-metadata` | `gate-fail` | `bedd61cae78d9b3ef6641c6f758b4f61939f9295cdaf25d5531a9e52cb5965dc` |
| `HANDOFF/todo/P1_ASYNC_DELIVERY_RECEIPTS_DO_NOT_CONVERGE_LIVE_RCA_2026-08-25.md` | `blocked-by-metadata` | `gate-fail` | `c41dff091c079e0f4a727370cc4aeb22c83f36b39b17bf2c24f50af3adcb7789` |
| `HANDOFF/todo/P1_CLI_OUTBOX_CANONICAL_DRAIN_AND_SLED_UNIFICATION_2026-09-16.md` | `blocked-by-metadata` | `gate-fail` | `ced2d89d32a7556c265726ff27ba1d049960b4f091eb9d61cbebb5113efa1295` |
| `HANDOFF/todo/P1_CLI_SEND_CANONICAL_IDENTIFIER_PARITY_2026-09-16.md` | `blocked-by-metadata` | `gate-fail` | `8df4ef9ffb79b3ef93695f0d7844d39921a1c831b7646a7b967d0336341c8a9d` |
| `HANDOFF/todo/P1_CONTACT_RECOVERY_WRITES_PEERID_AS_PUBLIC_KEY_2026-08-10.md` | `blocked-by-metadata` | `gate-fail` | `520ae6a0084d378761ef7be84c87089efb4cff760fc82c2f5144f7001bd3d986` |
| `HANDOFF/todo/P1_CORE_IDENTITY_SPOOF_AND_WASM_TOPIC_PARITY_2026-09-16.md` | `blocked-by-metadata` | `gate-fail` | `0d45c4b42229c0fc17b530c3de02ba8016b3e5084b26f9be76922ac48ab3f906` |
| `HANDOFF/todo/P1_DOCKER_CONTROL_API_SECURITY_HARDENING_2026-09-16.md` | `blocked-by-metadata` | `gate-fail` | `248cf3feedb4ec46682337efdd1d075feff7f6e2fbc104cf9fb160459d82b30a` |
| `HANDOFF/todo/P1_GHOST_GUARD_OWN_TOPIC_MESSAGE_LOSS_2026-09-16.md` | `blocked-by-metadata` | `gate-fail` | `462a8b2d3963c3e26bd9f23f364727f977acbf789a440a5ac745d0d3925dcd6c` |
| `HANDOFF/todo/P1_RELEASE_SIGNING_GATE_FAIL_CLOSED_2026-09-16.md` | `blocked-by-metadata` | `gate-fail` | `55c4a89fd0867ff1f99d82dc0a912646fb20a5b564653d72baedde48c27687a1` |
| `HANDOFF/todo/P1_ROUTING_ENGINE_NEVER_LEARNS_PEERS_2026-08-10.md` | `blocked-by-metadata` | `gate-fail` | `2fbf22c5c0f4bc934da1d29d9c7b9de58fbd9580ce9ea4bce7a1c028b473e359` |
| `HANDOFF/todo/P1_SECURITY_RUSTLS_RUSTSEC_2026_0285_TRACKING.md` | `blocked-by-metadata` | `gate-fail` | `6ecb90a71b2422536263866d7495a3fcf880a2abe37e46b7a9bf653d40efdca7` |
| `HANDOFF/todo/P1_SWARM_CHANNEL_BACKPRESSURE_DEADLOCK_2026-09-16.md` | `blocked-by-metadata` | `gate-fail` | `74f08007a2a33873f3b5ddb9f200ab43f3b97927f01f9121fcf76b6d9678e33a` |
| `HANDOFF/todo/P1_WINDOWS_NODE_SILENT_WEDGE_2026-09-15.md` | `blocked-by-metadata` | `gate-fail` | `8ed85d445874e9c0c0797beb2afdc2c5d42eb964e77c208b212be9f4751d853a` |
| `HANDOFF/todo/P2_ANDROID_KOTLIN_WARNING_TRIAGE_2026-09-14.md` | `blocked-by-metadata` | `gate-fail` | `939c0f825cf3fd06de8c37b77d34b96ded78125d0c76bd3675f776fa744a454f` |
| `HANDOFF/todo/P2_NON_SWARM_TRANSPORT_ROUTING_FEED.md` | `blocked-by-metadata` | `gate-fail` | `7e0b7c165bae10688dec0809b6c78b98ce9e47e2f9abe034a12de660d9394332` |
| `HANDOFF/todo/P2_WIRE_ENVELOPE_TRUNCATION_2026-08-10.md` | `blocked-by-metadata` | `gate-fail` | `3730adbd7ddf3a4388e05214f2dced34ea717fe469133045de92e587ed1f9bf9` |
| `HANDOFF/todo/P3_DOC_LEDGER_MIGRATION_F6_NOTES.md` | `blocked-by-metadata` | `gate-fail` | `22cb13b14adeadd90fede38ec21d1386f254db80d90885c4e1e196ed41bad641` |
| `HANDOFF/todo/PQC_FULL_INTEGRATION_REVIEW_2026-08-28.md` | `blocked-by-metadata` | `gate-fail` | `99d0951dc0999ec3ee5b101841ea9ed573a33a9963dbb063967c5391b8196b87` |
| `HANDOFF/todo/RCA_DELIVERY_ACK_IMPLEMENTATION_PLAN_2026-08-25.md` | `blocked-by-metadata` | `gate-fail` | `0a703f6ae4df70f0110e87d1e68f80b1391a3d6cbafb7d678c51995c8728a019` |
| `HANDOFF/todo/RECEIPT_MARKER_ID_FLAVOR_MISMATCH_2026-08-09.md` | `blocked-by-metadata` | `gate-fail` | `70aa96dda8fc90d07d3df40dd16c11d51bcb669150ef729f8d8a800f73c3205a` |
| `HANDOFF/todo/REJECTED/OLD_BLOCKED_BY_QUOTA.md` | `blocked-by-metadata` | `gate-fail` | `afcb542bee26915d01f29655cde17d65ef47605da9ff1bf1fc26f48cbb75aa1b` |
| `HANDOFF/todo/REJECTED/OLD_MICRO_BATCH_2026_05_14.md` | `blocked-by-metadata` | `gate-fail` | `0f69c81178a22494485e920ef8d3b23b4b057d5a801aa552aae3ea26c878eb2f` |
| `HANDOFF/todo/REJECTED/REJECTION_task_epic_wiring_draft.md` | `blocked-by-metadata` | `gate-fail` | `828814a8928945dcbf600d2f9721c16c8af83fc508f07044533b0620b1aed48f` |
| `HANDOFF/todo/REJECTED/[STALE]_BATCH_MICRO_084716.md` | `blocked-by-metadata` | `gate-fail` | `2f94825c34ee693bf636129cd8e7b496c7b4d57ad52a08aaedcee5adbb938684` |
| `HANDOFF/todo/REJECTED/task_epic_wiring_draft.md` | `blocked-by-metadata` | `gate-fail` | `d41634ded45b040ce339286d72829e1d8ce742d6534cc3fe6cf2c423778abc8f` |
| `HANDOFF/todo/RULE8_REVIEW_PR262_LEDGER_UNIFICATION.md` | `blocked-by-metadata` | `gate-fail` | `11af47317f1ffe7a1fb906f43c1025b852bb3f47dadc6ae9d71f3c6127cd3430` |
| `SCM-Q-770f050631fd` (path hash `770f050631fd02d89ecf085b5857ca8ea76d1f12652a1bf3bdf6ca75b751cbba`) | `quarantined` | `foreign-alias-in-bytes` | `8980980af987c6f134e1d85546c620b85aaf1aabdaf33da56ac9991ac72db2a0` |
| `HANDOFF/todo/V040_LEDGER_SEEDING_AND_GOSSIP.md` | `blocked-by-metadata` | `gate-fail` | `42c1a64d391154b98cb724beb4a2435a9736c0c15c43b4a65101338398bda10a` |
| `HANDOFF/todo/WAVE_H_HUMAN_GATES.md` | `blocked-by-metadata` | `gate-fail` | `9475c829e8e232e40f2e40f4d4f603e1a27a75d723b822ce5bb7cd56d203feea` |
| `HANDOFF/todo/_QUEUE.md` | `blocked-by-metadata` | `gate-fail` | `b48f0e1ebab6b3f4005f66b3b957bdc78ee51dde72eccdd8c85438bc04feec7e` |

## Disposition

- No quarantined file is copied, committed, published, or treated as owner evidence.
- No blocked-by-metadata file is repaired in this pass; its owner must supply metadata and approve a deterministic migration.
- The owner-valid rows are the only rows eligible for later owner-local publication, subject to fresh staged-index and CI checks.
- Manifest path: `HANDOFF/review/SCOPE_INVENTORY_2026-09-24.md`.
