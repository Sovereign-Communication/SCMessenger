# Android Development Setup Guide

Status: Active  
Last updated: 2026-03-07  
Validates: Requirements 5.9

This guide covers setting up your development environment for SCMessenger Android development.

## Prerequisites

### Required Software

1. **Android Studio** (latest stable version)
   - Download from: https://developer.android.com/studio
   - Includes Android SDK, emulator, and build tools

2. **Java JDK 11 or higher**
   - Check version: `java -version`
   - Download from: https://adoptium.net/ (recommended)

3. **Rust toolchain**
   - Install: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
   - Add Android targets:
     ```bash
     rustup target add aarch64-linux-android
     rustup target add armv7-linux-androideabi
     rustup target add x86_64-linux-android
     rustup target add i686-linux-android
     ```

4. **Android NDK r26b**
   - Install via Android Studio: Tools → SDK Manager → SDK Tools → NDK
   - Or download directly: https://developer.android.com/ndk/downloads

### Environment Variables

Add to your shell profile (`~/.bashrc`, `~/.zshrc`, or `~/.profile`):

```bash
# Android SDK
export ANDROID_HOME=$HOME/Android/Sdk  # Linux
# export ANDROID_HOME=$HOME/Library/Android/sdk  # macOS
# export ANDROID_HOME=%LOCALAPPDATA%\Android\Sdk  # Windows

export PATH=$PATH:$ANDROID_HOME/emulator
export PATH=$PATH:$ANDROID_HOME/platform-tools
export PATH=$PATH:$ANDROID_HOME/tools
export PATH=$PATH:$ANDROID_HOME/tools/bin

# Android NDK
export NDK_HOME=$ANDROID_HOME/ndk/26.3.11579264
export PATH=$PATH:$NDK_HOME
```

Reload your shell: `source ~/.bashrc` (or restart terminal)

## Project Setup

### 1. Clone Repository

```bash
git clone https://github.com/YOUR_ORG/SCMessenger.git
cd SCMessenger
```

### 2. Build Rust Core

```bash
# Build for all Android architectures
cargo build --release --target aarch64-linux-android
cargo build --release --target armv7-linux-androideabi
cargo build --release --target x86_64-linux-android
cargo build --release --target i686-linux-android
```

### 3. Copy Native Libraries

```bash
# Libraries are automatically copied by the build script
# Or manually:
mkdir -p android/app/src/main/jniLibs/{arm64-v8a,armeabi-v7a,x86_64,x86}
cp target/aarch64-linux-android/release/libscmessenger_mobile.so android/app/src/main/jniLibs/arm64-v8a/
cp target/armv7-linux-androideabi/release/libscmessenger_mobile.so android/app/src/main/jniLibs/armeabi-v7a/
cp target/x86_64-linux-android/release/libscmessenger_mobile.so android/app/src/main/jniLibs/x86_64/
cp target/i686-linux-android/release/libscmessenger_mobile.so android/app/src/main/jniLibs/x86/
```

### 4. Open in Android Studio

1. Open Android Studio
2. File → Open → Select `SCMessenger/android` directory
3. Wait for Gradle sync to complete
4. Build → Make Project

## Building

### Debug Build

```bash
cd android
./gradlew assembleDebug
```

Output: `android/app/build/outputs/apk/debug/app-debug.apk`

### Release Build

```bash
cd android
./gradlew assembleRelease
```

Output: `android/app/build/outputs/apk/release/app-release.apk`

### Android App Bundle (for Play Store)

```bash
cd android
./gradlew bundleRelease
```

Output: `android/app/build/outputs/bundle/release/app-release.aab`

## Running

### On Emulator

1. Create emulator in Android Studio: Tools → Device Manager → Create Device
2. Start emulator
3. Run from Android Studio: Run → Run 'app'

Or via command line:

```bash
cd android
./gradlew installDebug
adb shell am start -n com.scmessenger.android/.MainActivity
```

### On Physical Device

1. Enable Developer Options on device:
   - Settings → About phone → Tap "Build number" 7 times
2. Enable USB Debugging:
   - Settings → Developer options → USB debugging
3. Connect device via USB
4. Verify connection: `adb devices`
5. Install: `./gradlew installDebug`

## Testing

### Unit Tests

```bash
cd android
./gradlew test
```

### Instrumentation Tests

```bash
cd android
./gradlew connectedAndroidTest
```

### Specific Test

```bash
cd android
./gradlew test --tests "com.scmessenger.android.data.MeshRepositoryTest"
```

## Debugging

### Logcat

```bash
# View all logs
adb logcat

# Filter by tag
adb logcat -s SCMessenger

# Filter by priority (V=Verbose, D=Debug, I=Info, W=Warn, E=Error)
adb logcat *:E

# Save to file
adb logcat > android_log.txt
```

### Android Studio Debugger

1. Set breakpoints in Kotlin code
2. Run → Debug 'app'
3. Use Debug panel to step through code

### Native Debugging (Rust)

1. Build with debug symbols: `cargo build --target aarch64-linux-android`
2. Use `lldb` or `gdb` with NDK tools
3. Attach to running process

## Common Issues

### Issue: NDK not found

```
Error: NDK not configured
```

**Solution**: Set `NDK_HOME` environment variable or configure in `local.properties`:

```properties
# android/local.properties
ndk.dir=/path/to/ndk/26.3.11579264
```

### Issue: Rust library not found

```
Error: java.lang.UnsatisfiedLinkError: dlopen failed: library "libscmessenger_mobile.so" not found
```

**Solution**: Rebuild Rust core and copy libraries to `jniLibs/`

### Issue: Gradle sync failed

```
Error: Could not resolve all dependencies
```

**Solution**: 
1. Check internet connection
2. Invalidate caches: File → Invalidate Caches / Restart
3. Clean build: `./gradlew clean`

### Issue: Emulator won't start

**Solution**:
1. Check virtualization is enabled in BIOS
2. Ensure no other emulators are running
3. Try cold boot: Device Manager → Cold Boot Now

## Code Style

### Kotlin Style Guide

Follow [Kotlin Coding Conventions](https://kotlinlang.org/docs/coding-conventions.html):

- Use 4 spaces for indentation
- Maximum line length: 120 characters
- Use camelCase for variables and functions
- Use PascalCase for classes

### Linting

```bash
cd android
./gradlew ktlintCheck
```

Fix automatically:

```bash
./gradlew ktlintFormat
```

## Project Structure

```
android/
├── app/
│   ├── src/
│   │   ├── main/
│   │   │   ├── java/com/scmessenger/android/
│   │   │   │   ├── MainActivity.kt
│   │   │   │   ├── data/           # Data layer (repositories)
│   │   │   │   ├── ui/             # UI layer (composables)
│   │   │   │   └── viewmodel/      # ViewModels
│   │   │   ├── jniLibs/            # Native libraries
│   │   │   ├── res/                # Resources
│   │   │   └── AndroidManifest.xml
│   │   └── test/                   # Unit tests
│   └── build.gradle                # App-level build config
├── build.gradle                    # Project-level build config
└── gradle.properties               # Gradle properties
```

## Additional Resources

- [Android Developer Guide](https://developer.android.com/guide)
- [Jetpack Compose Documentation](https://developer.android.com/jetpack/compose)
- [Kotlin Documentation](https://kotlinlang.org/docs/home.html)
- [UniFFI Documentation](https://mozilla.github.io/uniffi-rs/)
- [SCMessenger Architecture](../ARCHITECTURE.md)
- [SCMessenger Testing Guide](../TESTING_GUIDE.md)

## Getting Help

- Check [Troubleshooting Guide](../troubleshooting/BUILD_ISSUES.md)
- Search [GitHub Issues](https://github.com/YOUR_ORG/SCMessenger/issues)
- Ask in [GitHub Discussions](https://github.com/YOUR_ORG/SCMessenger/discussions)
