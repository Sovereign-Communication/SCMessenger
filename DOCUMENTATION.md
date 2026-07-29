# SCMessenger Documentation Hub

Status: Active
Last updated: 2026-07-25
Applies to: v0.3.5 (alpha, working toward v1.0.0)

This is the documentation entrypoint. Start here.

Historical material -- completed plans, dated audits, and past session reports --
lives under `docs/historical/` and is deliberately not listed below. For
per-document lifecycle status (`Active`, `Planned`, `Historical`, `Superseded`),
see the [Document Status Index](docs/DOCUMENT_STATUS_INDEX.md).

## Start here

- [README](README.md) -- what SCMessenger is and how to run a node
- [Current State](docs/CURRENT_STATE.md) -- verified implementation status
- [Known Limitations](docs/V1_KNOWN_LIMITATIONS.md) -- what does not work yet
- [Architecture](docs/ARCHITECTURE.md) -- system design and security model
- [Module Map](docs/ARCHITECTURE_MODULE_MAP.md) -- per-module file map of the core
- [Repository Layout](docs/REPO_LAYOUT.md) -- where files are supposed to live

## Install and run

- [Install Guide](docs/INSTALL.md) -- full contributor setup
- [Simple Android Install](docs/SIMPLE_INSTALL_ANDROID.md)
- [Simple iOS Install](docs/SIMPLE_INSTALL_IOS.md)
- Platform setup: [Android](docs/platform/ANDROID_SETUP.md) |
  [iOS](docs/platform/IOS_SETUP.md) | [WASM](docs/platform/WASM_SETUP.md) |
  [CLI](docs/platform/CLI_SETUP.md)
- CLI per-OS guides: [Windows](docs/CLI_WINDOWS.md) |
  [Linux](docs/CLI_LINUX.md) | [macOS](docs/CLI_MACOS.md)
- [Docker Quickstart](docs/platform/DOCKER_QUICKSTART.md)
- [Joining the Mesh](docs/BOOTSTRAP.md) -- peer discovery, ledger exchange, and
  supplying a cold-start peer address
- [Deployment](docs/DEPLOYMENT.md)

## Protocol and internals

- [Protocol Specification](docs/PROTOCOL.md) -- the wire contract
- [Nature-Inspired Mesh Philosophy](docs/NATURE_INSPIRED_MESH_PHILOSOPHY.md) -- WBE 3/4 biological scaling laws & metabolic mesh architecture
- [Post-Quantum Protocol Specification](docs/PQC_HYBRID_PROTOCOL.md) -- ML-KEM-768 & ML-DSA-65 hybrid cryptographic suite
- [Transport Architecture](docs/TRANSPORT_ARCHITECTURE.md)
- [NAT Traversal Guide](docs/NAT_TRAVERSAL_GUIDE.md)
- [Ephemeral Messaging](docs/ephemeral_messaging.md)
- [Identity and Blocking](docs/IDENTITY_BLOCKING_IMPLEMENTATION.md)
- [Platform Support Matrix](docs/PLATFORM_SUPPORT_MATRIX.md)
- [Feature Parity](docs/FEATURE_PARITY.md) -- cross-platform parity status

## Contributing

- [CONTRIBUTING.md](CONTRIBUTING.md) -- workflow, style, review gates
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Feature Workflow](docs/FEATURE_WORKFLOW.md) -- adding a cross-platform feature
- [Testing Guide](docs/TESTING_GUIDE.md) -- gates and test inventory
- [Claude Reference](docs/CLAUDE_REFERENCE.md) -- build/test command reference and
  core module map
- [AGENTS.md](AGENTS.md) -- contract for automated contributors
- Troubleshooting: [Build Issues](docs/troubleshooting/BUILD_ISSUES.md) |
  [CI Failures](docs/troubleshooting/CI_FAILURES.md) |
  [Runtime Issues](docs/troubleshooting/RUNTIME_ISSUES.md)

## Security

- [SECURITY.md](SECURITY.md) -- policy and private reporting channel
- [Quantum Readiness Audit](docs/QUANTUM_READINESS_AUDIT.md)
- [Privacy Policy](docs/privacy_policy.md)
- Open audits: [Unfinished Code Audit](docs/audits/UNFINISHED_CODE_AUDIT.md) |
  [Public Key Validation](docs/audits/PUBLIC_KEY_VALIDATION_AUDIT_2026-03-18.md)
- [Dependency Audit (2026-07-22)](docs/DEPENDENCY_AUDIT_2026-07-22.md)
- [Stubs and Unimplemented](docs/STUBS_AND_UNIMPLEMENTED.md)

## Planning and status

Read **`HANDOFF/todo/_QUEUE.md` status-correction headers first** for what to dispatch next.
Sequencing scope: [v1.0.0 Execution Plan](HANDOFF/V1_0_0_EXECUTION_PLAN.md) (Section 0A).
Release milestones: [Milestone Release Plan](HANDOFF/plans/MILESTONE_RELEASE_PLAN.md).
Farm validator: [Farm Final Plan](HANDOFF/plans/FARM_FINAL_PLAN.md).

- [Remaining Work Tracking](REMAINING_WORK_TRACKING.md) -- session backlog narrative
- [v1.0.0 Execution Plan](HANDOFF/V1_0_0_EXECUTION_PLAN.md) -- the road to 1.0
- [Dispatch queue](HANDOFF/todo/_QUEUE.md) -- live pick list
- [Edge-Case Readiness Matrix](docs/EDGE_CASE_READINESS_MATRIX.md)
- [Release Readiness (2026-07-02)](docs/release-readiness-2026-07-02.md)
- [CHANGELOG.md](CHANGELOG.md)
- [Document Status Index](docs/DOCUMENT_STATUS_INDEX.md)

## Operations

- [Node Operator Guide](docs/RELAY_OPERATOR_GUIDE.md) -- running a publicly
  reachable node (not a distinct role; every node is a full relay)
- [Log Extraction Standard](docs/ops/LOG_EXTRACTION_STANDARD.md) -- mandatory for
  iOS/Android log capture
- [Log Extraction Quick Reference](docs/ops/LOG_EXTRACTION_QUICK_REF.md)
- [Log Rotation](docs/ops/LOG_ROTATION_INFO.md)
- [iOS Log Quickstart](docs/ops/QUICKSTART_IOS_LOGS.md)
- [GCP Deploy Guide](docs/ops/GCP_DEPLOY_GUIDE.md)
- [Android Release Signing](docs/ANDROID_RELEASE_SIGNING.md)
- [Build Reproducibility](docs/BUILD_REPRODUCIBILITY.md)

## Internal tooling

These document the AI-assisted development workflow used on this repository. They
are not product documentation and are not required to build, run, or contribute
to SCMessenger.

- [Orchestration Protocol](docs/ORCHESTRATION.md) -- canonical cross-mode protocol
- [Orchestration Playbook](docs/ORCHESTRATION_PLAYBOOK.md)
- [Lake Registry](docs/orchestration/SCM_UNIFIED_LAKE_ORCHESTRATION.md)
- [Orchestrator Directive](docs/orchestration/ORCHESTRATOR_DIRECTIVE.md)
- [AI Standards](docs/AI_STANDARDS.md)
- [Script Hygiene Guidelines](docs/SCRIPT_HYGIENE_GUIDELINES.md)

Per-workstream execution notes from the 2026-03 to 2026-04 alpha cycle were moved
to `docs/historical/EXECUTION_NOTES_ARCHIVE.md`.

## Support

See [SUPPORT.md](SUPPORT.md). GitHub Discussions is not enabled on this
repository; use issues, or the private security channel described in
[SECURITY.md](SECURITY.md) for vulnerabilities.

## Documentation governance

1. Execution truth comes from the Active docs listed above.
2. Backlog updates go to `REMAINING_WORK_TRACKING.md`.
3. Superseded status and audit reports move to `docs/historical/` rather than
   being duplicated as new "final" docs.
4. Use `iOS/` (uppercase-I) in all path references.
5. Run `./scripts/docs_sync_check.sh` (Unix / Git Bash) or
   `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/docs_sync_check.ps1`
   (Windows) before finalizing a change; fix failures in the same run.
6. If a change touches code, bindings, scripts, or platform wiring, run the
   relevant build verification from `.claude/rules/build.md` and record the
   result.
