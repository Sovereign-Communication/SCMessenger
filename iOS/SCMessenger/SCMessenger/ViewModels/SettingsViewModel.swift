//
//  SettingsViewModel.swift
//  SCMessenger
//
//  ViewModel for settings management
//  Mirrors: android/.../ui/viewmodels/SettingsViewModel.kt
//

import Foundation
import os

@MainActor
@Observable
final class SettingsViewModel {
    private weak var repository: MeshRepository?

    var settings: MeshSettings?
    var isLoading = false
    var isSaving = false
    var isApplyingTransportSettings = false
    var error: String?
    var successMessage: String?
    private var lastSettingUpdateTime: Date = .distantPast
    private let settingDebounceInterval: TimeInterval = 0.5  // 500ms
    private var pendingSettingsUpdates = 0
    private var settingsUpdateTask: Task<Void, Never>?

    init(repository: MeshRepository) {
        self.repository = repository
        self.loadNickname()
    }

    private func debouncedUpdateSettings(_ update: @escaping () -> Void) {
        let now = Date()
        if now.timeIntervalSince(lastSettingUpdateTime) < settingDebounceInterval {
            os_log("Settings update throttled")
            return
        }
        lastSettingUpdateTime = now
        update()
    }

    // MARK: - Identity

    var nickname: String = ""

    func loadNickname() {
        if let name = repository?.getNickname() {
            self.nickname = name
        } else {
            // A relay-only installation has no identity yet.  Showing a fake
            // default name makes Settings look initialized and can accidentally
            // create an identity with an unintended nickname.
            self.nickname = ""
        }
    }

    func updateNickname(_ name: String) {
        Task { @MainActor in
            do {
                try await repository?.setNickname(name)
                self.nickname = name
                successMessage = "Nickname updated"
            } catch {
                self.error = "Failed to update nickname: \(error.localizedDescription)"
            }
        }
    }

    func getIdentityExportString() async -> String {
        return await repository?.getIdentityExportString() ?? "{}"
    }

    // MARK: - Identity Backup (passphrase-encrypted)

    var backupExportResult: Result<String, Error>?
    var isExportingBackup = false

    /// Export a passphrase-encrypted identity backup (identity key + ratchet
    /// sessions + contacts). Distinct from `getIdentityExportString()`, which
    /// exports the public identity card with no encryption.
    ///
    /// The generated UniFFI surface follows this target's MainActor isolation.
    /// Yield first so the progress state is rendered before the synchronous
    /// encryption call begins.
    func exportIdentityBackup(passphrase: String) {
        guard let core = repository?.ironCore else {
            backupExportResult = .failure(MeshError.notInitialized("IronCore not initialized"))
            return
        }
        isExportingBackup = true
        Task { @MainActor [weak self] in
            await Task.yield()
            let result: Result<String, Error>
            do {
                result = .success(try core.exportIdentityBackup(passphrase: passphrase))
            } catch {
                result = .failure(error)
            }
            self?.finishBackupExport(result)
        }
    }

    private func finishBackupExport(_ result: Result<String, Error>) {
        isExportingBackup = false
        backupExportResult = result
    }

    func clearBackupExportResult() {
        backupExportResult = nil
    }

    var backupImportResult: Result<Void, Error>?
    var isImportingBackup = false

    /// Import an identity backup using a user-supplied passphrase. This mirrors
    /// `exportIdentityBackup`'s progress yield and completes repository-level
    /// identity publication after a successful import.
    func importIdentityBackup(backup: String, passphrase: String) {
        guard let core = repository?.ironCore else {
            backupImportResult = .failure(MeshError.notInitialized("IronCore not initialized"))
            return
        }
        isImportingBackup = true
        Task { @MainActor [weak self] in
            await Task.yield()
            let result: Result<Void, Error>
            do {
                try core.importIdentityBackup(backup: backup, passphrase: passphrase)
                if core.getIdentityInfo().initialized {
                    core.grantConsent()
                }
                result = .success(())
            } catch {
                result = .failure(error)
            }
            self?.finishBackupImport(result)
        }
    }

    private func finishBackupImport(_ result: Result<Void, Error>) {
        if case .success = result {
            repository?.finalizeImportedIdentity()
            loadNickname()
        }
        isImportingBackup = false
        switch result {
        case .success:
            successMessage = "Identity restored successfully"
        case .failure(let error):
            self.error = "Failed to restore identity: \(error.localizedDescription)"
        }
        backupImportResult = result
    }

    func clearBackupImportResult() {
        backupImportResult = nil
    }

    // MARK: - Settings Lifecycle

    func loadSettings() {
        isLoading = true
        do {
            settings = try repository?.loadSettings()
            error = nil
        } catch {
            self.error = error.localizedDescription
        }
        isLoading = false
    }

    func saveSettings() {
        guard let settings = settings else { return }
        enqueueSettingsUpdate(settings)
    }

    private func enqueueSettingsUpdate(_ nextSettings: MeshSettings) {
        let previousTask = settingsUpdateTask
        pendingSettingsUpdates += 1
        isSaving = true
        isApplyingTransportSettings = true

        settingsUpdateTask = Task { @MainActor [weak self] in
            await previousTask?.value
            guard let self else { return }
            defer {
                self.pendingSettingsUpdates -= 1
                self.isSaving = self.pendingSettingsUpdates > 0
                self.isApplyingTransportSettings = self.pendingSettingsUpdates > 0
            }

            guard let repository = self.repository else {
                self.error = MeshError.notInitialized("MeshRepository unavailable").localizedDescription
                return
            }

            do {
                try await repository.saveAndApplyTransportSettings(nextSettings)
                self.successMessage = "Settings saved"
                self.error = nil
            } catch {
                self.error = error.localizedDescription
            }
        }
    }

    // MARK: - Relay

    func toggleRelay() {
        guard var currentSettings = settings else { return }
        currentSettings.relayEnabled = !currentSettings.relayEnabled
        settings = currentSettings
        debouncedUpdateSettings { self.saveSettings() }
    }

    // MARK: - Supported iOS Transport Toggles

    func updateBleEnabled(_ enabled: Bool) {
        guard var currentSettings = settings else { return }
        currentSettings.bleEnabled = enabled
        settings = currentSettings
        enqueueSettingsUpdate(currentSettings)
    }

    func updateInternetEnabled(_ enabled: Bool) {
        guard var currentSettings = settings else { return }
        currentSettings.internetEnabled = enabled
        settings = currentSettings
        enqueueSettingsUpdate(currentSettings)
    }

    // MARK: - Discovery & Mesh Settings

    func updateDiscoveryMode(_ mode: DiscoveryMode) {
        guard var currentSettings = settings else { return }
        currentSettings.discoveryMode = mode
        settings = currentSettings
        debouncedUpdateSettings { self.saveSettings() }
    }

    func updateBatteryFloor(_ floor: UInt8) {
        guard var currentSettings = settings else { return }
        currentSettings.batteryFloor = floor
        settings = currentSettings
        debouncedUpdateSettings { self.saveSettings() }
    }

    // MARK: - Privacy / Onion Routing

    func updateOnionRouting(_ enabled: Bool) {
        guard var currentSettings = settings else { return }
        currentSettings.onionRouting = enabled
        settings = currentSettings
        debouncedUpdateSettings { self.saveSettings() }
    }

    // MARK: - BLE Privacy

    var isBleRotationEnabled: Bool {
        return repository?.blePrivacyEnabled ?? true
    }

    var bleRotationInterval: TimeInterval {
        return repository?.blePrivacyInterval ?? 900
    }

    func toggleBleRotation(enabled: Bool) {
        repository?.blePrivacyEnabled = enabled
    }

    func updateBleRotationInterval(_ interval: TimeInterval) {
        repository?.blePrivacyInterval = interval
    }

    // MARK: - App Preferences (mirrors Android PreferencesRepository)

    var isNotificationsEnabled: Bool {
        get { settings?.notificationsEnabled ?? true }
        set {
            guard var currentSettings = settings else { return }
            currentSettings.notificationsEnabled = newValue
            settings = currentSettings
            debouncedUpdateSettings { self.saveSettings() }
            if newValue {
                Task {
                    _ = await NotificationManager.shared.requestPermissionIfNeeded()
                }
            }
        }
    }

    var notifyDmEnabled: Bool {
        get { settings?.notifyDmEnabled ?? true }
        set {
            guard var currentSettings = settings else { return }
            currentSettings.notifyDmEnabled = newValue
            settings = currentSettings
            debouncedUpdateSettings { self.saveSettings() }
        }
    }

    var notifyDmRequestEnabled: Bool {
        get { settings?.notifyDmRequestEnabled ?? true }
        set {
            guard var currentSettings = settings else { return }
            currentSettings.notifyDmRequestEnabled = newValue
            settings = currentSettings
            debouncedUpdateSettings { self.saveSettings() }
        }
    }

    var notifyDmInForeground: Bool {
        get { settings?.notifyDmInForeground ?? false }
        set {
            guard var currentSettings = settings else { return }
            currentSettings.notifyDmInForeground = newValue
            settings = currentSettings
            debouncedUpdateSettings { self.saveSettings() }
        }
    }

    var notifyDmRequestInForeground: Bool {
        get { settings?.notifyDmRequestInForeground ?? true }
        set {
            guard var currentSettings = settings else { return }
            currentSettings.notifyDmRequestInForeground = newValue
            settings = currentSettings
            debouncedUpdateSettings { self.saveSettings() }
        }
    }

    var soundEnabled: Bool {
        get { settings?.soundEnabled ?? true }
        set {
            guard var currentSettings = settings else { return }
            currentSettings.soundEnabled = newValue
            settings = currentSettings
            debouncedUpdateSettings { self.saveSettings() }
        }
    }

    var badgeEnabled: Bool {
        get { settings?.badgeEnabled ?? true }
        set {
            guard var currentSettings = settings else { return }
            currentSettings.badgeEnabled = newValue
            settings = currentSettings
            debouncedUpdateSettings { self.saveSettings() }
        }
    }

    // MARK: - Privacy Feature Preferences
    //
    // These preferences persist via UserDefaults and are forwarded to the Rust
    // core privacy module (cover traffic, padding, timing obfuscation) via
    // MeshSettings once the UniFFI surface exposes per-feature toggles.
    // Until then the stored value is the source of truth consulted by the UI.

    var isCoverTrafficEnabled: Bool {
        get { UserDefaults.standard.object(forKey: "privacy_cover_traffic") as? Bool ?? false }
        set {
            UserDefaults.standard.set(newValue, forKey: "privacy_cover_traffic")
            guard var s = settings else { return }
            s.coverTrafficEnabled = newValue
            settings = s
            debouncedUpdateSettings { self.saveSettings() }
        }
    }

    var isMessagePaddingEnabled: Bool {
        get { UserDefaults.standard.object(forKey: "privacy_message_padding") as? Bool ?? false }
        set {
            UserDefaults.standard.set(newValue, forKey: "privacy_message_padding")
            guard var s = settings else { return }
            s.messagePaddingEnabled = newValue
            settings = s
            debouncedUpdateSettings { self.saveSettings() }
        }
    }

    var isTimingObfuscationEnabled: Bool {
        get { UserDefaults.standard.object(forKey: "privacy_timing_obfuscation") as? Bool ?? false }
        set {
            UserDefaults.standard.set(newValue, forKey: "privacy_timing_obfuscation")
            guard var s = settings else { return }
            s.timingObfuscationEnabled = newValue
            settings = s
            debouncedUpdateSettings { self.saveSettings() }
        }
    }

    // MARK: - Service Control

    var isServiceRunning: Bool {
        return repository?.serviceState == .running
    }

    func startService() {
        repository?.start()
    }

    func stopService() {
        repository?.stopMeshService()
    }

    // MARK: - Info Counts

    func getContactCount() -> UInt32 {
        return (try? repository?.getContactCount()) ?? 0
    }

    func getMessageCount() -> UInt32 {
        return (try? repository?.getMessageCount()) ?? 0
    }

    func getConnectionPathState() -> ConnectionPathState {
        return repository?.getConnectionPathState() ?? .disconnected
    }

    func getNatStatus() -> String {
        return repository?.getNatStatus() ?? "unknown"
    }

    func exportDiagnostics() -> String {
        return repository?.exportDiagnostics() ?? "{}"
    }

    // MARK: - Auto-Adjust (mirrors Android PowerSettingsScreen)

    var isAutoAdjustEnabled: Bool {
        get { repository?.isAutoAdjustEnabled ?? (UserDefaults.standard.object(forKey: "auto_adjust_enabled") as? Bool ?? true) }
        set { repository?.setAutoAdjustEnabled(newValue) }
    }

    // MARK: - Factory Reset (mirrors Android resetAllData)

    /// Wipe all local data and return the app to first-run state.
    func resetAllData() async {
        await repository?.resetAllData()
        nickname = ""
        successMessage = nil
        error = nil
    }
}
