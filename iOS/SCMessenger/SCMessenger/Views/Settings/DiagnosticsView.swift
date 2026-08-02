import SwiftUI

@MainActor
struct DiagnosticsView: View {
    @Environment(MeshRepository.self) private var repository
    @State private var logText: String = ""
    @State private var showingExportSheet: Bool = false
    @State private var exportItems: [Any] = []
    @State private var autoRefreshTimer: Timer?
    @State private var isAutoRefreshing: Bool = false

    var body: some View {
        VStack(spacing: 0) {
            VStack(alignment: .leading, spacing: 4) {
                Text("Delivery states: pending -> stored -> forwarding -> delivered")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Text("Tester note: export after reproducing install/first-message issues and include permission prompts seen on device.")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }
            .padding(.horizontal, 12)
            .padding(.top, 6)

            // Technical info header
            HStack {
                VStack(alignment: .leading, spacing: 2) {
                    Text("File: \(URL(fileURLWithPath: repository.diagnosticsLogPath()).lastPathComponent)")
                    Text("Size: \(logFileSize)  •  Lines: \(logLineCount)")
                }
                .font(.system(size: 10, design: .monospaced))
                .foregroundStyle(.secondary)
                Spacer()
                Toggle("Live", isOn: $isAutoRefreshing)
                    .font(.caption)
                    .fixedSize()
                    .onChange(of: isAutoRefreshing) { _, enabled in
                        if enabled {
                            startAutoRefresh()
                        } else {
                            stopAutoRefresh()
                        }
                    }
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 6)
            .background(Color(uiColor: .systemGroupedBackground))

            ScrollViewReader { proxy in
                ScrollView {
                    if logText.isEmpty {
                        VStack(spacing: 20) {
                            Image(systemName: "doc.text.magnifyingglass")
                                .font(.largeTitle)
                                .foregroundStyle(.secondary)
                            Text("No Diagnostic Logs")
                                .font(.headline)
                            Text("Start the Mesh Service or tap Trace to generate logs.")
                                .font(.subheadline)
                                .multilineTextAlignment(.center)
                        }
                        .padding(.top, 100)
                        .foregroundStyle(.secondary)
                    } else {
                        Text(logText)
                            .font(.system(.caption, design: .monospaced))
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .padding()
                            .id("logContent")
                    }
                    Color.clear.frame(height: 1).id("bottom")
                }
                .background(Color(uiColor: .secondarySystemBackground))
                .onChange(of: logText) {
                    if isAutoRefreshing {
                        withAnimation { proxy.scrollTo("bottom", anchor: .bottom) }
                    }
                }
            }

            Divider()

            VStack(spacing: 10) {
                HStack(spacing: 12) {
                    Button {
                        refreshLogs()
                    } label: {
                        Label("Refresh", systemImage: "arrow.clockwise")
                    }
                    .buttonStyle(.bordered)

                    Button {
                        addManualTrace()
                    } label: {
                        Label("Trace", systemImage: "bolt.fill")
                    }
                    .buttonStyle(.bordered)

                    Button(role: .destructive) {
                        clearLogs()
                    } label: {
                        Label("Clear", systemImage: "trash")
                    }
                    .buttonStyle(.bordered)
                }

                Button {
                    prepareExport()
                } label: {
                    Label("Export Diagnostics Bundle", systemImage: "square.and.arrow.up")
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.borderedProminent)
            }
            .padding()
        }
        .navigationTitle("Diagnostics")
        .navigationBarTitleDisplayMode(.inline)
        .onAppear {
            refreshLogs()
        }
        .onDisappear {
            stopAutoRefresh()
        }
        .sheet(isPresented: $showingExportSheet) {
            ShareSheet(items: exportItems)
        }
    }

    private func refreshLogs() {
        let snapshot = repository.diagnosticsSnapshot(limit: 500)
        logText = snapshot.isEmpty ? "" : snapshot
    }

    private func addManualTrace() {
        repository.appendDiagnostic("manual_trace t=\(Int(Date().timeIntervalSince1970))")
        refreshLogs()
    }

    private func clearLogs() {
        repository.clearDiagnostics()
        logText = ""
    }

    private func prepareExport() {
        let jsonMetrics = repository.exportDiagnostics()
        let recentLogs = repository.diagnosticsSnapshot(limit: 5000)
        let missingPermissions = [
            "Bluetooth (peer discovery and nearby transport)",
            "Local Network (peer connectivity in local mesh)",
            "Notifications (optional delivery alerts)"
        ]
        let bundleText: String = """
        === SCMessenger Diagnostics Bundle (v0.4.0) ===
        Generated: \(Date())
        Version: 0.4.0

        --- Runtime Summary ---
        connection_path_state: \(repository.getConnectionPathState())
        nat_status: \(repository.getNatStatus())

        --- Delivery State Guide ---
        pending: queued locally, first route attempt in progress.
        stored: queued for retry while recipient is unreachable.
        forwarding: active retry is running now.
        delivered: recipient delivery receipt confirmed.

        --- Reliability Notes For Testers ---
        1) First send may remain pending/stored while routes warm up.
        2) Stored/forwarding are expected on unstable links; retries continue automatically.
        3) Report cases that never reach delivered after network recovery.

        --- Permissions Rationale ---
        \(missingPermissions.joined(separator: "\n"))

        --- Node State Metrics ---
        \(jsonMetrics)

        --- Recent Application Logs ---
        \(recentLogs.isEmpty ? "(no logs)" : recentLogs)
        """
        exportItems = [bundleText]
        showingExportSheet = true
    }

    private func startAutoRefresh() {
        autoRefreshTimer = Timer.scheduledTimer(withTimeInterval: 2.0, repeats: true) { _ in
            Task { @MainActor in
                refreshLogs()
            }
        }
    }

    private func stopAutoRefresh() {
        autoRefreshTimer?.invalidate()
        autoRefreshTimer = nil
    }

    private var logFileSize: String {
        let path = repository.diagnosticsLogPath()
        guard let attrs = try? FileManager.default.attributesOfItem(atPath: path),
              let size = attrs[.size] as? Int64 else { return "0 B" }
        let formatter: ByteCountFormatter = ByteCountFormatter()
        formatter.countStyle = .file
        return formatter.string(fromByteCount: size)
    }

    private var logLineCount: Int {
        logText.isEmpty ? 0 : logText.components(separatedBy: .newlines).count
    }
}

struct ShareSheet: UIViewControllerRepresentable {
    let items: [Any]

    func makeUIViewController(context: Context) -> UIActivityViewController {
        UIActivityViewController(activityItems: items, applicationActivities: nil)
    }

    func updateUIViewController(_ uiViewController: UIActivityViewController, context: Context) {}
}
