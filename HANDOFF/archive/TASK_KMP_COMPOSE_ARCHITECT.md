# TASK: KMP Scaffolding & Compose Multiplatform Architecture

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Agent Role
Agent 2: Compose Multiplatform Architect

## Context (Compressed)
SCMessenger has an Android client (`android/`) using Kotlin + Jetpack Compose:
- UniFFI-generated `uniffi.api` package consumed by Kotlin
- Architecture: `MeshRepository` → ViewModels → Compose UI
- BLE/WiFi transport managers, foreground service, notification channels
- Gradle 8.13, AGP 8.13.2, Kotlin 1.9.20, Hilt DI

Goal: Add a KMP Compose Multiplatform desktop target for Ubuntu (linuxX64) with 100% feature parity.

## Your Mission
Design and implement the KMP scaffolding and Compose Multiplatform desktop UI layer.

### Specific Tasks
1. **Refactor `settings.gradle` / `build.gradle`** to add KMP support:
   - Add `org.jetbrains.kotlin.multiplatform` plugin
   - Add `org.jetbrains.compose` plugin (Compose Multiplatform)
   - Create `shared/` module with `commonMain`, `androidMain`, `linuxX64Main` source sets
   - Keep existing `android/app/` module intact (refactor to depend on `shared/`)

2. **Desktop Compose UI**: Create `linuxX64Main` source set with:
   - Main window (Compose Desktop `Window`)
   - System tray integration (use `Tray` composable from Compose Desktop)
   - Native notifications (bind to Rust FFI notification bridge)
   - Master-Detail layout: Contact list | Chat view | Settings

3. **Shared KMP layer** (`commonMain`):
   - Expect/actual pattern for platform-specific networking (desktop uses libp2p directly, Android uses BLE/WiFi)
   - Shared ViewModels for chat, contacts, settings
   - Kotlin Coroutines + StateFlow for reactive UI

4. **Dependency injection**: Adapt Hilt for KMP or replace with Koin (KMP-compatible) for shared module.

### Output Format
- `shared/` directory with KMP module structure
- Updated Gradle files (`settings.gradle`, `build.gradle`, `shared/build.gradle.kts`)
- Desktop entry point: `fun main()` in `linuxX64Main`
- Compose UI files for at least: MainWindow, ContactList, ChatView, SystemTray
- Verification: Gradle sync succeeds, `./gradlew :shared:compileKotlinLinuxX64` passes

### Constraints
- Must NOT break existing `./gradlew :app:assembleDebug` for Android
- Compose Multiplatform version must be compatible with Kotlin 1.9.20+ (or upgrade Kotlin if needed)
- Desktop window must honor XDG desktop conventions
- System tray must show connection status, unread count, quick-action menu
