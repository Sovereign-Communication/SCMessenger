# Android Emulator Setup Plan

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Active
Last updated: 2026-07-09

This document outlines the plan for setting up and running the Android emulator locally on a Windows host using Intel/AMD hardware virtualization with Windows Hyper-V / Windows Hypervisor Platform (WHPX).

## Package Requirements

The following Android SDK packages are required for the emulator to function correctly:
1. `emulator`: The Android emulator runner itself.
2. `platform-tools`: Includes `adb`, which is required for communication with the virtual device.
3. `system-images;android-34;google_apis;x86_64`: The system image for Android 14 (API 34) with Google APIs, matching the target platform of the SCMessenger Android application.

## Creating a Virtual Device (AVD)

To create the virtual device using `avdmanager`:
1. Ensure the system image is installed:
   ```cmd
   sdkmanager.bat "system-images;android-34;google_apis;x86_64"
   ```
2. Create the virtual device named `SCMessenger_Emulator` with a specified device profile (e.g., `pixel_6`):
   ```cmd
   avdmanager.bat create avd -n SCMessenger_Emulator -k "system-images;android-34;google_apis;x86_64" -d "pixel_6" --force
   ```

## Running the Emulator

The emulator can be started in standard (GUI) mode or headless mode.

### Command-line Flags
- `-avd SCMessenger_Emulator`: Specifies the name of the AVD to start.
- `-no-snapshot`: Starts the device from a clean boot instead of loading from a quick-boot snapshot.
- `-gpu host`: Uses the host machine's graphics hardware for rendering, which increases performance.
- `-no-window`: Runs the emulator headlessly without a graphical user interface window.
- `-no-audio`: Disables audio output/input support.

### Starting Standard (GUI) Mode
```cmd
emulator.exe -avd SCMessenger_Emulator -gpu host -no-snapshot
```

### Starting Headless Mode (for CI/automated testing)
```cmd
emulator.exe -avd SCMessenger_Emulator -no-window -no-audio -gpu host -no-snapshot
```

## Troubleshooting: Hyper-V and WHPX Lockouts

If you encounter performance issues or errors regarding virtualization:
1. **Verify Windows Features**:
   Ensure `Hyper-V` and `Windows Hypervisor Platform` are enabled in Windows Features:
   - Run `OptionalFeatures.exe`.
   - Check `Hyper-V` and `Windows Hypervisor Platform`.
   - Restart the host machine.
2. **Verify User Group Membership**:
   Ensure the current user is a member of the `Hyper-V Administrators` group if required.
3. **Check WHPX Status via Emulator**:
   Run the emulator with `-verbose` to check if WHPX is successfully detected and initialized:
   ```cmd
   emulator.exe -avd SCMessenger_Emulator -verbose
   ```
   Look for `WHPX: Status: Connected` or similar log lines.
4. **Antivirus / Co-existence Issues**:
   Other virtualization software (like VirtualBox or VMware) can conflict with Hyper-V. Disable them or configure them to run in Hyper-V fallback mode.
