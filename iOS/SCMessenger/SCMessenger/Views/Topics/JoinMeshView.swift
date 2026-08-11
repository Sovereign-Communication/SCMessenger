//
//  JoinMeshView.swift
//  SCMessenger
//
//  View for joining mesh topics
//

import SwiftUI
import VisionKit
import Vision

@MainActor
struct JoinMeshView: View {
    @Environment(\.dismiss) private var dismiss
    @Environment(MeshRepository.self) private var repository

    @State private var topicManager: TopicManager?
    @State private var topicName: String = ""
    @State private var autoSubscribe: Bool = true
    @State private var error: String?
    @State private var showingQrScanner: Bool = false

    private var canUseQrScanner: Bool {
        if #available(iOS 16.0, *) {
            return DataScannerViewController.isSupported && DataScannerViewController.isAvailable
        }
        return false
    }

    var body: some View {
        NavigationStack {
            Form {
                Section("Join Mesh Topic") {
                    TextField("Topic Name", text: $topicName)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()

                    Toggle("Auto-subscribe to messages", isOn: $autoSubscribe)
                }

                Section("Join via QR") {
                    Button("Scan Join Bundle QR") {
                        showingQrScanner = true
                    }
                    .disabled(!canUseQrScanner)
                    if !canUseQrScanner {
                        Text("QR scanning is unavailable on this device. Use manual join.")
                            .font(Theme.bodySmall)
                            .foregroundStyle(.secondary)
                    }
                }

                if let error = error {
                    Section {
                        Text(error)
                            .foregroundStyle(.red)
                            .font(Theme.bodySmall)
                    }
                }

                Section {
                    Button("Join") {
                        joinMesh()
                    }
                    .disabled(topicName.isEmpty)
                }

                Section("Subscribed Meshes") {
                    ForEach(topicManager?.listTopics() ?? [], id: \.self) { topic in
                        HStack {
                            Text(topic)
                                .font(Theme.bodyMedium)
                            Spacer()
                            Button {
                                leaveTopic(topic)
                            } label: {
                                Image(systemName: "xmark.circle.fill")
                                    .foregroundStyle(.red)
                            }
                        }
                    }
                }

                Section {
                    Text("Topics allow you to join specific mesh networks and receive messages from all participants in that topic.")
                        .font(Theme.bodySmall)
                        .foregroundStyle(Theme.onSurfaceVariant)
                }
            }
            .navigationTitle("Join Mesh")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") {
                        dismiss()
                    }
                }
            }
            .onAppear {
                if topicManager == nil {
                    topicManager = TopicManager(meshRepository: repository)
                }
            }
            .sheet(isPresented: $showingQrScanner) {
                if canUseQrScanner {
                    QRCodeScannerSheetInline(
                        onScan: { payload in
                            showingQrScanner = false
                            joinFromBundle(payload)
                        },
                        onFailure: { message in
                            error = message
                        }
                    )
                } else {
                    Text("QR scanning is unavailable on this device.")
                        .padding()
                }
            }
        }
    }

    private func joinMesh() {
        Task {
            do {
                try await topicManager?.subscribe(to: topicName)
                error = nil
                topicName = ""
            } catch {
                self.error = error.localizedDescription
            }
        }
    }

    private func leaveTopic(_ topic: String) {
        Task {
            do {
                try await topicManager?.unsubscribe(from: topic)
                error = nil
            } catch {
                self.error = error.localizedDescription
            }
        }
    }

    private func joinFromBundle(_ raw: String) {
        struct JoinBundle: Decodable {
            // swiftlint:disable:next identifier_name
            let bootstrap_peers: [String]
            let topics: [String]
        }

        guard let data = raw.data(using: .utf8),
              let bundle = try? JSONDecoder().decode(JoinBundle.self, from: data) else {
            error = "Invalid join bundle QR data"
            return
        }

        if bundle.bootstrap_peers.isEmpty {
            error = "Join bundle has no bootstrap peers"
            return
        }

        Task {
            // Keep the bootstrap path durable. Seeds remain unproven until
            // transport Identify confirms the peer, but they must be
            // available on the next app launch.
            _ = repository.importSeedAddresses(bundle.bootstrap_peers)

            for addr in bundle.bootstrap_peers {
                await repository.connectToPeer("", addresses: [addr])
            }

            for topic in bundle.topics {
                do {
                    try await topicManager?.subscribe(to: topic)
                } catch {
                    self.error = "Failed subscribing to topic \(topic): \(error.localizedDescription)"
                }
            }
        }
    }
}

@available(iOS 16.0, *)
private struct QRCodeScannerSheetInline: UIViewControllerRepresentable {
    var onScan: (String) -> Void
    var onFailure: (String) -> Void

    func makeUIViewController(context: Context) -> DataScannerViewController {
        let controller: DataScannerViewController = DataScannerViewController(
            recognizedDataTypes: [.barcode(symbologies: [.qr])],
            qualityLevel: .balanced,
            recognizesMultipleItems: false,
            isHighFrameRateTrackingEnabled: false,
            isHighlightingEnabled: true
        )
        controller.delegate = context.coordinator
        return controller
    }

    func updateUIViewController(_ uiViewController: DataScannerViewController, context: Context) {
        do {
            try uiViewController.startScanning()
        } catch {
            onFailure("Unable to start camera scanner: \(error.localizedDescription)")
        }
    }

    func makeCoordinator() -> Coordinator {
        Coordinator(onScan: onScan)
    }

    final class Coordinator: NSObject, DataScannerViewControllerDelegate {
        private let onScan: (String) -> Void

        init(onScan: @escaping (String) -> Void) {
            self.onScan = onScan
        }

        func dataScanner(
            _ dataScanner: DataScannerViewController,
            didTapOn item: RecognizedItem
        ) {
            if case let .barcode(barcode) = item, let payload = barcode.payloadStringValue {
                onScan(payload)
            }
        }
    }
}
