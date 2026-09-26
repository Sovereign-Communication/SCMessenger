# TASK: Gradle Rust Android Build Parallelization and Incremental Fix

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Agent Role
Agent: Build System & CI Specialist (P1)

## Context
During a comprehensive audit of the Android build system and compilation process, two significant areas for build speed and reliability improvements were identified:
1. **Lack of Parallel Architecture Compilation**: The `buildRustAndroid` task compiles the three targets (`aarch64-linux-android`, `armv7-linux-androideabi`, `x86_64-linux-android`) sequentially. On a multi-core machine, Gradle can compile these in parallel if they are split into individual sub-tasks. This will reduce Rust target compilation times by almost 60%.
2. **Incorrect Incremental Build Directory Inputs**: The `buildRustAndroid` task only specifies `../../core/src` and `../../core/Cargo.toml` as inputs. Because it builds the `scmessenger-mobile` package, any changes made strictly inside the `mobile` crate (which holds the JNI interface and UniFFI mobile setup) are ignored by Gradle's incremental check, incorrectly showing the task as `UP-TO-DATE` when it needs to rebuild.

## Acceptance Criteria
- [ ] Split `buildRustAndroid` into three parallelizable tasks: `buildRustAndroidArm64`, `buildRustAndroidArmv7`, and `buildRustAndroidX86_64`.
- [ ] Make `buildRustAndroid` a lifecycle task that depends on all three sub-tasks.
- [ ] Add `../../mobile/src` and `../../mobile/Cargo.toml` to the task inputs.
- [ ] Support toggleable incremental compilation by reading a project property or environment variable (defaulting to cargo's default incremental behavior for local debug builds).
- [ ] Auto-detect and configure `sccache` as `RUSTC_WRAPPER` if present in the system path.
- [ ] Verify that running `./gradlew assembleDebug` successfully compiles and runs all parallel tasks.

## Implementation Plan

### Step 1: Split Tasks and Update Inputs in `android/app/build.gradle`
Modify the task definition in `android/app/build.gradle`:
- Set inputs to watch both `core` and `mobile` directories.
- Define a base action helper for cargo-ndk invocation.
- Define individual Gradle tasks for each architecture.

```groovy
// Helper function to invoke cargo ndk for a target architecture
def runCargoNdkBuild(String target, String sdkDir, String ndkDir, String rustProfile, String rustDir, boolean isWindows, String msysUcrtBin) {
    exec {
        workingDir project.file('../../')
        environment "RUSTFLAGS", "-C link-arg=-Wl,-z,max-page-size=16384"
        environment "ANDROID_HOME", sdkDir
        environment "ANDROID_NDK_HOME", ndkDir
        
        // Auto-detect and use sccache if available
        def sccacheExists = isWindows ? 
            file("C:/Program Data/chocolatey/bin/sccache.exe").exists() || System.getenv("PATH").split(";").any { file("${it}/sccache.exe").exists() } :
            System.getenv("PATH").split(":").any { file("${it}/sccache").exists() }
        if (sccacheExists) {
            environment "RUSTC_WRAPPER", "sccache"
        }

        if (isWindows && msysUcrtBin) {
            environment "PATH", "${msysUcrtBin};${System.getenv('PATH')}"
        }

        if (isWindows) {
            commandLine 'cmd', '/c', "cargo ndk -t $target build ${rustProfile} -p scmessenger-mobile"
        } else {
            commandLine 'cargo', 'ndk', '-t', target, 'build', rustProfile, '-p', 'scmessenger-mobile'
        }
    }
}
```

Implement the individual tasks:
```groovy
def targetList = [
    [name: 'Arm64', target: 'aarch64-linux-android', abi: 'arm64-v8a'],
    [name: 'Armv7', target: 'armv7-linux-androideabi', abi: 'armeabi-v7a'],
    [name: 'X86_64', target: 'x86_64-linux-android', abi: 'x86_64']
]

targetList.each { arch ->
    task "buildRustAndroid${arch.name}" {
        description = "Build Rust library for ${arch.target} using cargo-ndk"
        group = 'rust'

        inputs.dir('../../core/src')
        inputs.file('../../core/Cargo.toml')
        inputs.dir('../../mobile/src')
        inputs.file('../../mobile/Cargo.toml')
        outputs.file("../../core/target/android-libs/${arch.abi}/libscmessenger_mobile.so")

        doLast {
            def sdkDir = localProps.getProperty('sdk.dir') ?: android.sdkDirectory.absolutePath
            def ndkDir = localProps.getProperty('ndk.dir') ?: android.ndkDirectory?.absolutePath
            if (ndkDir == null || !file(ndkDir).exists()) {
                ndkDir = "${sdkDir}/ndk/${android.ndkVersion}"
            }
            
            def isWindows = System.getProperty('os.name').toLowerCase().contains('windows')
            def taskNames = project.gradle.startParameter.taskNames.join(",")
            def isRelease = taskNames.contains("Release") || taskNames.contains("Bundle")

            def rustProfile = isRelease ? "--release" : ""
            def rustDir = isRelease ? "release" : "debug"

            runCargoNdkBuild(arch.target, sdkDir, ndkDir, rustProfile, rustDir, isWindows, msysUcrtBin)

            // Copy output to destination
            def sourceFile = file("../../target/${arch.target}/${rustDir}/libscmessenger_mobile.so")
            def destDir = file("../../core/target/android-libs/${arch.abi}")
            destDir.mkdirs()
            copy {
                from sourceFile
                into destDir
            }
        }
    }
}

task buildRustAndroid {
    description = 'Aggregated task to build all target architectures in parallel'
    group = 'rust'
    dependsOn buildRustAndroidArm64
    dependsOn buildRustAndroidArmv7
    dependsOn buildRustAndroidX86_64
}
```

*Estimate:* `~120 LOC` changes.
