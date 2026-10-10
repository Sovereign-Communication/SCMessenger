//
//  JoinMeshView.swift
//  SCMessenger
//
//  View for joining mesh topics
//

import SwiftUI
import VisionKit
import Vision
import CoreImage.CIFilterBuiltins

@MainActor
struct JoinMeshView: View {
    @Environment(\.dismiss) private var dismiss
    @Environment(MeshRepository.self) private var repository

    @State private var topicManager: TopicManager?
    @State private var topicName: String = ""
    @State private var autoSubscribe: Bool = true
    @State private var error: String?
    @State private var showingQrScanner: Bool = false
    @State private var statusMessage: String?
    @State private var isRedeeming: Bool = false
    @State private var myInvite: String?
    @State private var inviteUnavailable: Bool = false
    /// In-flight redeem / invite-creation tasks, cancelled when the view disappears.
    @State private var redeemTask: Task<Void, Never>?
    @State private var inviteTask: Task<Void, Never>?

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

                Section("Join with an invite") {
                    Button("Scan Invite QR") {
                        showingQrScanner = true
                    }
                    .disabled(!canUseQrScanner || isRedeeming)
                    Button("Paste invite") {
                        redeemInvite(UIPasteboard.general.string)
                    }
                    .disabled(isRedeeming)
                    if !canUseQrScanner {
                        Text("QR scanning is unavailable on this device. Paste the invite instead.")
                            .font(Theme.bodySmall)
                            .foregroundStyle(.secondary)
                    }
                    if let statusMessage = statusMessage {
                        Text(statusMessage)
                            .font(Theme.bodySmall)
                            .foregroundStyle(.secondary)
                    }
                }

                Section("Invite someone") {
                    Button("Show my invite") {
                        showMyInvite()
                    }
                    if let invite = myInvite {
                        if let image = Self.qrImage(for: invite) {
                            Image(uiImage: image)
                                .interpolation(.none)
                                .resizable()
                                .scaledToFit()
                                .frame(maxWidth: .infinity, minHeight: 220)
                                .accessibilityLabel("QR code for your SCMessenger invite")
                        }
                        Button("Copy invite") {
                            UIPasteboard.general.string = invite
                        }
                        ShareLink(item: invite) {
                            Text("Share invite")
                        }
                        Text("Another device can scan this code to join through you. It expires in one hour.")
                            .font(Theme.bodySmall)
                            .foregroundStyle(.secondary)
                    } else if inviteUnavailable {
                        Text("This node has no reachable address yet. Discovery is still running; try again shortly.")
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
            .onDisappear {
                redeemTask?.cancel()
                redeemTask = nil
                inviteTask?.cancel()
                inviteTask = nil
            }
            .sheet(isPresented: $showingQrScanner) {
                if canUseQrScanner {
                    QRCodeScannerSheetInline(
                        onScan: { payload in
                            showingQrScanner = false
                            redeemInvite(payload)
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

    /// Redeem a signed `SCI1:` invite through core (replaces the unsigned JSON
    /// join bundle). A seed that cannot be dialed yet is not a failure: the
    /// scheduler keeps retrying it.
    private func redeemInvite(_ raw: String?) {
        isRedeeming = true
        error = nil
        statusMessage = "Verifying invite..."
        redeemTask?.cancel()
        redeemTask = Task {
            let outcome = await repository.redeemInvite(raw)
            isRedeeming = false
            switch outcome {
            case let .success(_, imported, _):
                statusMessage = "Invite accepted. \(imported) nodes added. Connecting now; discovery keeps going in the background."
            case let .failure(reason):
                statusMessage = nil
                error = reason.userMessage
            }
        }
    }

    private func showMyInvite() {
        myInvite = nil
        inviteUnavailable = false
        inviteTask?.cancel()
        inviteTask = Task {
            if let invite = await repository.createInvite() {
                myInvite = invite
            } else {
                inviteUnavailable = true
            }
        }
    }

    private static func qrImage(for text: String) -> UIImage? {
        let filter = CIFilter.qrCodeGenerator()
        filter.message = Data(text.utf8)
        filter.correctionLevel = "L"
        guard let output = filter.outputImage else { return nil }
        let scaled = output.transformed(by: CGAffineTransform(scaleX: 8, y: 8))
        let context = CIContext()
        guard let cgImage = context.createCGImage(scaled, from: scaled.extent) else { return nil }
        return UIImage(cgImage: cgImage)
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
