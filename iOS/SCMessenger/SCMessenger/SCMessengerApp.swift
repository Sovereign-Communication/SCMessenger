//
//  SCMessengerApp.swift
//  SCMessenger
//
//  Main application entry point
//

import SwiftUI

@main
@MainActor
struct SCMessengerApp: App {
    private let installChoiceKey: String = "hasCompletedInstallModeChoice"

    // Repository - single source of truth
    @State private var meshRepository: MeshRepository = MeshRepository()

    // Background service
    @State private var backgroundService: MeshBackgroundService?
    @State private var didRunSetup: Bool = false
    @State private var isAppReady: Bool = false
    @State private var showOnboarding: Bool = false

    init() {
        // Initialize background service after repository
        // Will be set in onAppear
    }

    var body: some Scene {
        WindowGroup {
            Group {
                if !isAppReady {
                    VStack(spacing: 12) {
                        ProgressView()
                        Text("Preparing SCMessenger…")
                            .foregroundStyle(.secondary)
                    }
                } else if showOnboarding {
                    OnboardingFlow {
                        showOnboarding = false
                    }
                } else {
                    MainTabView()
                }
            }
            .environment(meshRepository)
            .onAppear { setupApp() }
            .onChange(of: meshRepository.identityInfo?.initialized) { _, _ in
                refreshOnboardingGate()
            }
            .onChange(of: meshRepository.identityHydrationState) { _, _ in
                refreshOnboardingGate()
            }
            .onReceive(NotificationCenter.default.publisher(for: UIApplication.didEnterBackgroundNotification)) { _ in
                handleEnteringBackground()
            }
            .onReceive(NotificationCenter.default.publisher(for: UIApplication.willTerminateNotification)) { _ in
                meshRepository.checkpointPersistentState(reason: "app_terminating")
            }
            .onReceive(NotificationCenter.default.publisher(for: UIApplication.willEnterForegroundNotification)) { _ in
                handleEnteringForeground()
            }
        }
    }

    private func setupApp() {
        if didRunSetup { return }
        didRunSetup = true

        // Initialize background service
        backgroundService = MeshBackgroundService(meshRepository: meshRepository)
        backgroundService?.registerBackgroundTasks()

        // Initialize + start repository so identity/service state is hydrated at launch.
        do {
            try meshRepository.initialize()
            NotificationManager.shared.configure(repository: meshRepository)
            meshRepository.start()
            meshRepository.setNotificationAppInForeground(true)
            if meshRepository.notificationSettingsEnabled() {
                Task {
                    _ = await NotificationManager.shared.requestPermissionIfNeeded()
                }
            }
            refreshOnboardingGate()
        } catch {
            print("[ERROR] Failed to initialize repository: \(error)")
            // Keep the app navigable after a recoverable startup failure so the
            // relay-only identity flow in Settings remains available.
            refreshOnboardingGate()
        }
        isAppReady = true
    }

    private func handleEnteringBackground() {
        // Commit the address book before iOS suspends the process. This makes
        // the Keychain recovery snapshot current even if the next action is a
        // direct device update rather than a normal foreground relaunch.
        meshRepository.checkpointPersistentState(reason: "app_backgrounding")
        meshRepository.setNotificationAppInForeground(false)
        backgroundService?.onEnteringBackground()
    }

    private func handleEnteringForeground() {
        meshRepository.setNotificationAppInForeground(true)
        backgroundService?.onEnteringForeground()
        refreshOnboardingGate()
    }

    private func refreshOnboardingGate() {
        var installChoiceCompleted: Bool = UserDefaults.standard.bool(forKey: installChoiceKey)
        let hasIdentity: Bool = meshRepository.hasVerifiedIdentity
        if hasIdentity && !installChoiceCompleted {
            UserDefaults.standard.set(true, forKey: installChoiceKey)
            UserDefaults.standard.set(true, forKey: "hasCompletedOnboarding")
            installChoiceCompleted = true
        }
        showOnboarding = !installChoiceCompleted && !hasIdentity
    }
}
