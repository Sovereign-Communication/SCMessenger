# Implementation Plan: Repository Production Readiness and Hygiene

## Overview

This implementation plan transforms the SCMessenger repository into a production-ready codebase with reliable CI/CD, automated multi-platform releases, comprehensive non-regression protection, and world-class repository hygiene. The approach prioritizes fixing failing GitHub Actions workflows first, then establishing non-regression protection, followed by platform-specific improvements in order: Android/Windows/Core → WASM → iOS.

**Implementation Strategy**:
1. **Phase 1: CI/CD Reliability** - Fix failing workflows, optimize for free tier (2000 min/month)
2. **Phase 2: Non-Regression Protection** - Pre-commit hooks, property-based tests, branch protection
3. **Phase 3: Platform Builds** - Android/Windows/Core first, then WASM, then iOS
4. **Phase 4: Security & Quality** - Dependency scanning, linting, secret detection
5. **Phase 5: Release Automation** - Multi-platform release pipeline
6. **Phase 6: Documentation & Community** - Comprehensive docs, contribution infrastructure

## Tasks

### Phase 1: CI/CD Reliability System (Highest Priority)

- [x] 1. Fix failing GitHub Actions workflows and establish baseline reliability
  - [x] 1.1 Audit current workflow failures and identify root causes
    - Run all existing workflows manually via workflow_dispatch
    - Document each failure with error messages and affected jobs
    - Categorize failures: environment setup, dependency issues, test failures, timeouts
    - _Requirements: 1.2, 1.4_
  
  - [x] 1.2 Fix environment configuration issues (ANDROID_HOME, NDK, Xcode)
    - Update `.github/workflows/ci.yml` to properly configure Android SDK and NDK r26b
    - Add environment validation steps before build jobs
    - Pin Xcode version for iOS builds using `xcode-select`
    - _Requirements: 1.3, 7.4_
  
  - [x] 1.3 Implement path-based job filtering to optimize CI minutes
    - Add `dorny/paths-filter@v2` action to detect changed file paths
    - Create path filter configuration for core, android, ios, wasm, docs
    - Update all jobs to conditionally execute based on path filter outputs
    - _Requirements: 1.9, 7.1_
  
  - [x] 1.4 Configure aggressive dependency caching
    - Add `Swatinem/rust-cache@v2` with matrix-specific cache keys
    - Configure Gradle caching for Android builds
    - Configure CocoaPods caching for iOS builds
    - Set `cache-on-failure: true` and `save-if: ${{ github.ref == 'refs/heads/main' }}`
    - _Requirements: 1.6, 7.2_
  
  - [x] 1.5 Add retry logic for transient network failures
    - Wrap network-dependent steps with `nick-fields/retry@v2`
    - Configure 3 retry attempts with 10-minute timeout per attempt
    - Apply to: cargo test, gradle build, pod install, npm install
    - _Requirements: 1.8_
  
  - [x] 1.6 Set job timeouts and optimize workflow execution
    - Add `timeout-minutes: 30` to all jobs
    - Optimize test execution: run unit tests in parallel, integration tests with `--jobs 1`
    - Skip expensive steps (integration tests) on doc-only changes
    - _Requirements: 1.1, 1.5_

- [x] 2. Checkpoint - Verify all CI workflows pass consistently
  - Ensure all tests pass, ask the user if questions arise.

### Phase 2: Non-Regression Protection (Top Priority)

- [x] 3. Implement comprehensive pre-commit hooks for local enforcement
  - [x] 3.1 Create pre-commit hook script with formatting and linting checks
    - Create `.git/hooks/pre-commit` script running cargo fmt, cargo clippy, unit tests
    - Make hook executable and test locally
    - Add hook installation instructions to CONTRIBUTING.md
    - _Requirements: 3.6, 6.1, 6.2_
  
  - [x] 3.2 Create commit-msg hook for conventional commit enforcement
    - Create `.git/hooks/commit-msg` script validating commit message format
    - Enforce pattern: `^(feat|fix|docs|style|refactor|test|chore|perf|ci|build|revert)(\(.+\))?: .{1,72}`
    - Provide helpful error messages with examples
    - _Requirements: 10.6_
  
  - [x]* 3.3 Add pre-commit hook installation script
    - Create `scripts/install_hooks.sh` to copy hooks to `.git/hooks/`
    - Make hooks executable automatically
    - _Requirements: 3.6_

- [x] 4. Establish property-based testing for parsers and cryptographic operations
  - [x] 4.1 Add proptest dependency and configure test framework
    - Add `proptest = "1.4"` to core/Cargo.toml dev-dependencies
    - Create `core/tests/property/` directory for property tests
    - _Requirements: 3.4, 13.9_
  
  - [x] 4.2 Implement property test for message serialization round-trip
    - **Property 1: Message serialization round-trip consistency**
    - **Validates: Requirements 3.5, 13.1**
    - Write proptest for: `parse(serialize(msg)) == msg` for all Message types
    - Test with 1000+ random inputs covering edge cases
    - _Requirements: 3.5, 13.1_
  
  - [x] 4.3 Implement property test for encryption round-trip
    - **Property 2: Encryption/decryption round-trip consistency**
    - **Validates: Requirements 3.4, 13.4**
    - Write proptest for: `decrypt(encrypt(plaintext, key), key) == plaintext`
    - Test with random plaintexts (0-1024 bytes) and keys
    - _Requirements: 3.4, 13.4_
  
  - [x] 4.4 Implement property test for identity export/import round-trip
    - **Property 3: Identity backup round-trip consistency**
    - **Validates: Requirements 13.3**
    - Write proptest for: `import(export(identity)) == identity`
    - Test with random identity configurations
    - _Requirements: 13.3_
  
  - [ ]* 4.5 Add property tests for cryptographic key serialization
    - **Property 4: Key serialization round-trip consistency**
    - **Validates: Requirements 13.4**
    - Write proptest for: `deserialize(serialize(key)) == key`
    - _Requirements: 13.4_

- [x] 5. Configure branch protection and test coverage tracking
  - [x] 5.1 Enable branch protection rules for main branch
    - Configure via GitHub UI: prevent force pushes, prevent deletion
    - Require status checks to pass before merging (ci.yml jobs)
    - Document branch protection settings in docs/CONTRIBUTING.md
    - _Requirements: 3.7, 3.8, 3.9_
  
  - [x] 5.2 Add code coverage tracking with tarpaulin
    - Create `.tarpaulin.toml` configuration file with 80% line coverage threshold
    - Add coverage job to `.github/workflows/ci.yml`
    - Generate HTML and LCOV reports in `target/coverage/`
    - _Requirements: 3.1_
  
  - [ ]* 5.3 Add regression test registry and documentation
    - Create `core/tests/regression/` directory
    - Document regression test naming convention: `issue_<number>_<description>.rs`
    - Add template for regression tests
    - _Requirements: 3.11_

- [x] 6. Checkpoint - Verify non-regression protection is active
  - Ensure all tests pass, ask the user if questions arise.

### Phase 3: Platform-Specific Build System (Android/Windows/Core → WASM → iOS)

- [x] 7. Implement Android build and test infrastructure
  - [x] 7.1 Configure Android CI job with proper environment setup
    - Add Android job to `.github/workflows/ci.yml` with path filtering
    - Use `android-actions/setup-android@v3` and `nttld/setup-ndk@v1` (NDK r26b)
    - Configure ANDROID_HOME and NDK environment variables
    - _Requirements: 1.3, 7.3_
  
  - [x] 7.2 Add Android build and test steps
    - Build debug APK: `cd android && ./gradlew assembleDebug`
    - Run unit tests: `./gradlew test`
    - Run instrumentation tests: `./gradlew connectedAndroidTest` (if emulator available)
    - _Requirements: 3.2_
  
  - [ ]* 7.3 Add ktlint for Kotlin code quality enforcement
    - Add ktlint plugin to `android/app/build.gradle`
    - Create lint job in `.github/workflows/lint.yml`
    - Run `./gradlew ktlintCheck` in CI
    - _Requirements: 6.3_

- [x] 8. Implement Windows and Linux CLI build infrastructure
  - [x] 8.1 Add CLI build matrix for Linux, macOS, Windows
    - Create build matrix in `.github/workflows/ci.yml`
    - Matrix: `os: [ubuntu-latest, macos-14, windows-latest]`
    - Build CLI binary: `cargo build --release --bin scmessenger-cli`
    - _Requirements: 2.1, 7.1_
  
  - [ ]* 8.2 Add CLI binary verification and smoke tests
    - Verify binary exists after build
    - Run basic smoke test: `./scmessenger-cli --version`
    - _Requirements: 2.1_

- [x] 9. Implement WASM build and optimization infrastructure
  - [x] 9.1 Add WASM CI job with wasm-pack
    - Add WASM job to `.github/workflows/ci.yml` with path filtering
    - Install wasm-pack: `curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh`
    - Build WASM: `cd wasm && wasm-pack build --target web --release`
    - _Requirements: 2.4, 11.7_
  
  - [x] 9.2 Add wasm-opt for bundle size optimization
    - Install wasm-opt (from binaryen package)
    - Optimize WASM: `wasm-opt -Oz pkg/scmessenger_wasm_bg.wasm -o pkg/scmessenger_wasm_bg.wasm`
    - Report bundle size before/after optimization
    - _Requirements: 11.7, 11.8_
  
  - [ ]* 9.3 Add JavaScript linting for WASM bindings
    - Add eslint to `wasm/package.json`
    - Create `.eslintrc.js` configuration
    - Run `npm run lint` in CI
    - _Requirements: 6.5_

- [x] 10. Implement iOS build infrastructure
  - [x] 10.1 Add iOS CI job with Xcode configuration
    - Add iOS job to `.github/workflows/ci.yml` (runs-on: macos-latest)
    - Pin Xcode version: `sudo xcode-select -s /Applications/Xcode_15.0.app`
    - Install CocoaPods: `gem install cocoapods`
    - _Requirements: 1.3, 7.4_
  
  - [x] 10.2 Add iOS build and test steps
    - Install pods: `cd iOS && pod install`
    - Build workspace: `xcodebuild -workspace SCMessenger.xcworkspace -scheme SCMessenger -configuration Debug build`
    - Run tests: `xcodebuild test -workspace SCMessenger.xcworkspace -scheme SCMessenger -destination 'platform=iOS Simulator,name=iPhone 15'`
    - _Requirements: 3.2_
  
  - [ ]* 10.3 Add swiftlint for Swift code quality enforcement
    - Install swiftlint: `brew install swiftlint`
    - Create `.swiftlint.yml` configuration
    - Run `swiftlint lint --strict` in CI
    - _Requirements: 6.4_

- [ ] 11. Checkpoint - Verify all platform builds pass
  - Ensure all tests pass, ask the user if questions arise.

### Phase 4: Security and Code Quality Systems

- [ ] 12. Implement dependency security scanning
  - [ ] 12.1 Create weekly security scan workflow
    - Create `.github/workflows/security.yml` with weekly cron schedule
    - Install cargo-audit and cargo-deny
    - Run `cargo audit --json > audit-report.json`
    - _Requirements: 4.1, 4.3, 9.2_
  
  - [ ] 12.2 Add cargo-deny configuration for license compliance
    - Create `deny.toml` with license allowlist (MIT, Apache-2.0, BSD-3-Clause)
    - Add license denylist (GPL-2.0, GPL-3.0, AGPL-3.0)
    - Run `cargo deny check licenses` in security workflow
    - _Requirements: 4.2, 4.5, 4.10_
  
  - [ ] 12.3 Add automated GitHub issue creation for vulnerabilities
    - Parse audit-report.json and extract vulnerabilities
    - Use `actions/github-script@v7` to create issues for HIGH/CRITICAL vulnerabilities
    - Add security label and severity classification
    - _Requirements: 4.4, 9.9_
  
  - [ ]* 12.4 Configure dependabot for automated dependency updates
    - Create `.github/dependabot.yml` configuration
    - Enable for Cargo, Gradle, CocoaPods, npm
    - Set update schedule to weekly
    - _Requirements: 4.6_

- [ ] 13. Implement secret scanning and security hardening
  - [ ] 13.1 Add gitleaks for secret detection
    - Create `.gitleaks.toml` configuration with rules for API keys, private keys, tokens
    - Add gitleaks job to `.github/workflows/security.yml`
    - Run `gitleaks detect --no-git --source . --report-path gitleaks_report.json`
    - _Requirements: 9.3, 9.4, 10.5_
  
  - [ ] 13.2 Create unsafe Rust code audit script
    - Create `scripts/audit_unsafe.sh` to find unsafe blocks
    - Check for `// SAFETY:` comments within 5 lines before each unsafe block
    - Run in CI and fail if SAFETY comments missing
    - _Requirements: 9.5_
  
  - [ ] 13.3 Add platform security configuration validation
    - Create `scripts/verify_platform_security.sh`
    - Check Android ProGuard enabled: `grep "minifyEnabled true" android/app/build.gradle`
    - Check iOS ATS configured: `grep "NSAppTransportSecurity" iOS/SCMessenger/Info.plist`
    - Check for hardcoded passwords in code
    - _Requirements: 9.6, 9.11, 9.12_

- [ ] 14. Implement multi-language linting and code quality enforcement
  - [ ] 14.1 Create comprehensive linting workflow
    - Create `.github/workflows/lint.yml` triggered on pull requests
    - Add jobs for: rust, kotlin, swift, javascript
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_
  
  - [ ] 14.2 Configure Rust linting with clippy and rustfmt
    - Add rustfmt check: `cargo fmt --all -- --check`
    - Add clippy check: `cargo clippy --workspace -- -D warnings`
    - Add custom checks: reject unwrap() and println! in library code
    - _Requirements: 6.1, 6.2, 6.7, 6.8_
  
  - [ ] 14.3 Create clippy configuration for complexity limits
    - Create `.clippy.toml` with cognitive-complexity-threshold: 15
    - Set too-many-arguments-threshold: 7
    - Add disallowed-methods for std::env::set_var
    - _Requirements: 6.9_

- [x] 15. Checkpoint - Verify security and quality checks pass
  - Ensure all tests pass, ask the user if questions arise.

### Phase 5: Release Automation System

- [x] 16. Implement version management and synchronization
  - [x] 16.1 Create version sync script
    - Create `scripts/sync_version.sh` to read version from Cargo.toml
    - Update Android build.gradle versionName and versionCode
    - Update iOS Info.plist CFBundleShortVersionString and CFBundleVersion
    - Update wasm/package.json version
    - _Requirements: 8.1, 8.2, 8.3, 8.4_
  
  - [x] 16.2 Create version validation script
    - Create `scripts/validate_tag.sh` to verify tag matches Cargo.toml version
    - Validate semantic versioning format
    - Check for pre-release tags (alpha, beta, rc)
    - _Requirements: 8.5, 8.9_
  
  - [x] 16.3 Create changelog generation script
    - Create `scripts/generate_changelog.sh` to extract commits since last tag
    - Group commits by type: feat, fix, docs, chore
    - Format with commit message, author, PR number
    - Highlight breaking changes at the top
    - _Requirements: 8.6, 8.7, 8.8_

- [x] 17. Implement multi-platform release pipeline
  - [x] 17.1 Create release workflow with build matrix
    - Create `.github/workflows/release.yml` triggered on version tags (v*)
    - Add CLI build matrix: Linux (x86_64), macOS (x86_64, ARM64), Windows (x86_64)
    - Build release binaries: `cargo build --release --bin scmessenger-cli --target ${{ matrix.target }}`
    - _Requirements: 2.1, 7.1_
  
  - [x] 17.2 Add Android release build with signing
    - Decode keystore from GitHub Secrets: `echo "${{ secrets.ANDROID_KEYSTORE_BASE64 }}" | base64 -d > release.keystore`
    - Build signed APK and AAB: `cd android && ./gradlew assembleRelease bundleRelease`
    - Configure signing with KEYSTORE_PASSWORD, KEYSTORE_ALIAS, KEY_PASSWORD from secrets
    - _Requirements: 2.2, 11.1, 11.2_
  
  - [x] 17.3 Add iOS release build with signing
    - Import certificate and provisioning profile from GitHub Secrets
    - Create keychain and import certificate: `security import certificate.p12 -k build.keychain`
    - Build archive: `xcodebuild -workspace SCMessenger.xcworkspace -scheme SCMessenger -configuration Release archive`
    - Export IPA: `xcodebuild -exportArchive -archivePath build/SCMessenger.xcarchive -exportPath build -exportOptionsPlist ExportOptions.plist`
    - _Requirements: 2.3, 11.4, 11.5_
  
  - [x] 17.4 Add WASM release build with optimization
    - Build WASM: `cd wasm && wasm-pack build --target web --release`
    - Optimize: `wasm-opt -Oz pkg/scmessenger_wasm_bg.wasm -o pkg/scmessenger_wasm_bg.wasm`
    - Generate both ES module and UMD formats
    - _Requirements: 2.4, 11.7, 11.8_
  
  - [x] 17.5 Add artifact verification and checksum generation
    - Generate SHA256 checksums for all artifacts
    - Create SHA256SUMS.txt file
    - Verify checksums before upload
    - _Requirements: 2.7_
  
  - [x] 17.6 Add GitHub Release creation with artifacts
    - Download all artifacts from build jobs
    - Generate changelog from commits
    - Create GitHub Release using `softprops/action-gh-release@v2`
    - Attach all artifacts and SHA256SUMS.txt
    - Mark as prerelease if tag contains alpha/beta/rc
    - _Requirements: 2.5, 2.6, 2.10_

- [x] 18. Create platform signing setup scripts
  - [x] 18.1 Create Android signing setup script
    - Create `scripts/setup_android_signing.sh` to generate release keystore
    - Generate keystore with keytool (RSA 2048-bit, 10000 days validity)
    - Base64-encode keystore for GitHub Secrets
    - Document keystore backup procedures
    - _Requirements: 11.1, 11.10, 11.11_
  
  - [x] 18.2 Create iOS signing setup script
    - Create `scripts/setup_ios_signing.sh` to export certificate and provisioning profile
    - Export certificate from Keychain as .p12
    - Base64-encode certificate and provisioning profile
    - Document certificate rotation procedures
    - _Requirements: 11.4, 11.10_
  
  - [x] 18.3 Create deployment documentation
    - Create `docs/DEPLOYMENT.md` with platform-specific release procedures
    - Document GitHub Secrets configuration
    - Document key generation and rotation procedures
    - Include troubleshooting guide for signing issues
    - _Requirements: 11.10_

- [x] 19. Checkpoint - Test release pipeline end-to-end
  - Ensure all tests pass, ask the user if questions arise.

### Phase 6: Build Reproducibility and Repository Hygiene

- [-] 20. Implement build reproducibility system
  - [ ] 20.1 Create rust-toolchain.toml with pinned versions
    - Create `rust-toolchain.toml` pinning Rust 1.75.0
    - Add components: rustfmt, clippy
    - Add all required targets: Linux, macOS, Windows, Android, iOS, WASM
    - _Requirements: 7.1, 7.2_
  
  - [ ] 20.2 Pin Android build dependencies
    - Update `android/build.gradle` with pinned versions
    - Pin Kotlin version, Gradle plugin, Hilt, Compose
    - Pin compileSdk, targetSdk, minSdk
    - _Requirements: 7.3_
  
  - [ ] 20.3 Create Docker build environment for Linux
    - Create `docker/build.Dockerfile` with Rust 1.75.0-slim base
    - Install build dependencies (build-essential, pkg-config, libssl-dev)
    - Set up non-root builder user
    - _Requirements: 7.7_
  
  - [ ]* 20.4 Create build verification script
    - Create `scripts/verify_build.sh` to test reproducibility
    - Build twice and compare checksums
    - Document reproducibility status
    - _Requirements: 7.5, 7.10_

- [-] 21. Implement repository hygiene system
  - [ ] 21.1 Create comprehensive .gitignore files
    - Update root .gitignore with build artifacts (target/, build/, .gradle/)
    - Add IDE files (.idea/, .vscode/, *.swp)
    - Add OS files (.DS_Store, Thumbs.db)
    - Add secrets (*.keystore, *.p12, *.mobileprovision, .env)
    - _Requirements: 10.1, 10.2, 10.3_
  
  - [ ] 21.2 Create repository hygiene workflow
    - Create `.github/workflows/hygiene.yml` triggered on pull requests
    - Check for tracked files matching .gitignore patterns
    - Check for trailing whitespace
    - Check for case-colliding paths
    - Check for nested .git directories
    - _Requirements: 10.4, 10.7, 10.8, 10.10_
  
  - [ ] 21.3 Create CODEOWNERS file for documentation
    - Create `.github/CODEOWNERS` with ownership patterns
    - Document maintainer responsibilities
    - Add CODEOWNERS syntax validation to hygiene workflow
    - _Requirements: 10.11_

- [x] 22. Checkpoint - Verify build reproducibility and hygiene
  - Ensure all tests pass, ask the user if questions arise.

### Phase 7: Documentation and Community Infrastructure

- [ ] 23. Create comprehensive documentation system
  - [ ] 23.1 Update README.md with accurate build instructions
    - Add prerequisites for all platforms
    - Add build commands for each platform
    - Add testing instructions
    - Add links to platform-specific guides
    - _Requirements: 5.1_
  
  - [ ] 23.2 Create CONTRIBUTING.md with contribution guidelines
    - Add development setup instructions
    - Add commit message format guidelines
    - Add PR submission process
    - Add code style requirements
    - Add testing requirements
    - _Requirements: 5.2, 12.2_
  
  - [ ] 23.3 Update SECURITY.md with vulnerability reporting procedures
    - Add contact information for security reports
    - Add severity classification guidelines
    - Add response time expectations
    - _Requirements: 5.3, 9.1_
  
  - [ ] 23.4 Create platform-specific setup guides
    - Create `docs/platform/ANDROID_SETUP.md`
    - Create `docs/platform/IOS_SETUP.md`
    - Create `docs/platform/WASM_SETUP.md`
    - Create `docs/platform/CLI_SETUP.md`
    - _Requirements: 5.9_
  
  - [ ] 23.5 Create troubleshooting guides
    - Create `docs/troubleshooting/BUILD_ISSUES.md`
    - Create `docs/troubleshooting/CI_FAILURES.md`
    - Create `docs/troubleshooting/RUNTIME_ISSUES.md`
    - _Requirements: 5.10_
  
  - [ ] 23.6 Create documentation sync check script
    - Create `scripts/docs_sync_check.sh` to validate documentation
    - Check for required header fields (Status, Last updated)
    - Validate markdown links
    - Check for machine-local paths
    - Require doc updates when code changes (configurable)
    - _Requirements: 5.4, 5.7, 5.11_
  
  - [ ]* 23.7 Generate API documentation
    - Create `scripts/generate_docs.sh`
    - Generate Rust docs: `cargo doc --workspace --no-deps --document-private-items`
    - Generate Android KDoc: `cd android && ./gradlew dokkaHtml`
    - Generate iOS docs with jazzy
    - _Requirements: 5.6_

- [ ] 24. Implement community engagement infrastructure
  - [ ] 24.1 Create issue templates
    - Create `.github/ISSUE_TEMPLATE/bug_report.yml` with structured fields
    - Create `.github/ISSUE_TEMPLATE/feature_request.yml`
    - Create `.github/ISSUE_TEMPLATE/security_vulnerability.yml`
    - _Requirements: 12.3, 12.4_
  
  - [ ] 24.2 Create pull request template
    - Create `.github/pull_request_template.md` with checklist
    - Include sections: Description, Related Issue, Type of Change, Platform, Testing
    - Add checklist: style guidelines, self-review, docs updated, tests added
    - _Requirements: 12.4_
  
  - [ ] 24.3 Create CODE_OF_CONDUCT.md
    - Define community standards and expected behavior
    - Define unacceptable behavior
    - Define enforcement procedures
    - Add contact information for conduct violations
    - _Requirements: 12.1_
  
  - [ ] 24.4 Create SUPPORT.md routing guide
    - Route questions to GitHub Discussions
    - Route bugs to GitHub Issues
    - Route security issues to security email
    - _Requirements: 12.5_
  
  - [ ] 24.5 Create auto-labeling workflow
    - Create `.github/workflows/auto-label.yml`
    - Label PRs based on changed files: rust, android, ios, wasm, cli, docs, ci, tests
    - Use `actions/github-script@v7` to add labels
    - _Requirements: 12.6_
  
  - [ ] 24.6 Create stale issue management workflow
    - Create `.github/workflows/stale.yml` with weekly schedule
    - Mark issues stale after 60 days of inactivity
    - Close stale issues after 7 days
    - Exempt pinned, security, and roadmap issues
    - _Requirements: 12.12_
  
  - [ ]* 24.7 Create governance and roadmap documentation
    - Create `docs/GOVERNANCE.md` with decision-making process
    - Create `docs/ROADMAP.md` with planned features
    - _Requirements: 12.10, 12.11_

- [x] 25. Final checkpoint - Comprehensive system verification
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- **Tasks marked with `*` are optional** and can be skipped for faster MVP delivery
- **Each task references specific requirements** for traceability back to requirements.md
- **Checkpoints ensure incremental validation** at reasonable breaks in the workflow
- **Property tests validate universal correctness properties** for parsers and cryptographic operations
- **Priority order**: CI/CD fixes → Non-regression protection → Platform builds (Android/Windows/Core → WASM → iOS) → Security/Quality → Release automation → Documentation
- **Free tier optimization**: Path-based filtering, aggressive caching, conditional job execution to stay within 2000 CI minutes/month
- **Truly sovereign**: All infrastructure uses GitHub-native features, no external services

## Implementation Guidance

### CI/CD Optimization Tips
- Use path filtering aggressively to avoid unnecessary builds
- Cache Rust target/, Gradle ~/.gradle, CocoaPods caches
- Run jobs in parallel where possible
- Skip expensive integration tests on doc-only changes
- Monitor CI minute usage weekly

### Security Best Practices
- Never commit secrets to git history
- Use GitHub Secrets for all sensitive data
- Rotate signing keys every 2 years
- Respond to HIGH vulnerabilities within 7 days
- Run security scans weekly, not on every PR

### Testing Strategy
- Property-based tests for parsers, serializers, crypto operations
- Unit tests for business logic
- Integration tests with `--jobs 1` to avoid race conditions
- Regression tests for all fixed bugs
- Target 80% code coverage for core modules

### Release Process
1. Update version in Cargo.toml
2. Run `./scripts/sync_version.sh`
3. Commit and push
4. Create and push tag: `git tag v0.2.2 && git push origin v0.2.2`
5. GitHub Actions builds and creates release automatically
6. Download artifacts and verify checksums
7. Optionally upload to app stores

### Platform-Specific Notes
- **Android**: Requires keystore in GitHub Secrets, ProGuard enabled for release
- **iOS**: Requires certificate and provisioning profile in GitHub Secrets
- **WASM**: Use wasm-opt for size optimization, target <150KB gzipped
- **CLI**: Strip debug symbols, enable LTO for smaller binaries
