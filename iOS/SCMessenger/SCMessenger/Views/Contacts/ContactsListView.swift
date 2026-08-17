//
//  ContactsListView.swift
//  SCMessenger
//
//  Contacts list view
//

import SwiftUI
import VisionKit
import Vision

@MainActor
struct ContactsListView: View {
    @Environment(MeshRepository.self) private var repository
    @State private var viewModel: ContactsViewModel?
    @State private var showingAddContact: Bool = false
    @State private var pendingChatConversation: Conversation?
    @State private var navigateToPendingChat: Bool = false
    @State private var nearbyPrefilledPeer: NearbyPeer? = nil
    @State private var quickConnectError: String? = nil
    @State private var pendingDeletePeerId: String? = nil
    @State private var pendingDeleteDisplayName: String = ""
    @State private var editingContact: Contact? = nil
    @State private var editNickname: String = ""
    @State private var contactToVerify: Contact? = nil
    @State private var contactToBlock: Contact? = nil
    @State private var actionError: String?

    var body: some View {
        List {
            // MARK: Nearby Peers — discovered on the mesh, not yet added
            let nearby = (viewModel?.nearbyPeers ?? []).filter { $0.hasFullIdentity }
            if !nearby.isEmpty {
                Section {
                    ForEach(nearby) { peer in
                        NearbyPeerRow(peer: peer) {
                            if peer.hasFullIdentity {
                                quickConnect(peer)
                            } else {
                                nearbyPrefilledPeer = peer
                                showingAddContact = true
                            }
                        }
                    }
                } header: {
                    Label("Nearby", systemImage: "antenna.radiowaves.left.and.right")
                        .foregroundStyle(Color.accentColor)
                }
            }

            // MARK: Saved contacts
            Section {
                ForEach(viewModel?.filteredContacts ?? [], id: \.peerId) { contact in
                    NavigationLink(
                        value: Conversation(
                            peerId: contact.peerId,
                            peerNickname: conversationDisplayName(for: contact)
                        )
                    ) {
                        ContactRow(contact: contact)
                    }
                    .contextMenu {
                        Button {
                            editingContact = contact
                            editNickname = contact.localNickname ?? contact.nickname ?? ""
                        } label: {
                            Label("Edit Nickname", systemImage: "pencil")
                        }

                        Button {
                            contactToVerify = contact
                        } label: {
                            Label("Verify Safety Number", systemImage: "checkmark.shield")
                        }

                        Button(role: .destructive) {
                            pendingDeletePeerId = contact.peerId
                            pendingDeleteDisplayName = conversationDisplayName(for: contact)
                        } label: {
                            Label("Delete", systemImage: "trash")
                        }

                        Button(role: .destructive) {
                            contactToBlock = contact
                        } label: {
                            Label("Block Contact", systemImage: "hand.raised.fill")
                        }
                    }
                }
                .onDelete { offsets in
                    let contacts = viewModel?.filteredContacts ?? []
                    guard let index = offsets.first, contacts.indices.contains(index) else { return }
                    let contact = contacts[index]
                    pendingDeletePeerId = contact.peerId
                    pendingDeleteDisplayName = conversationDisplayName(for: contact)
                }
            } header: {
                if !(viewModel?.filteredContacts ?? []).isEmpty {
                    Text("Contacts")
                }
            }
        }
        .navigationDestination(for: Conversation.self) { conversation in
            ChatView(conversation: conversation)
        }
        .navigationDestination(isPresented: $navigateToPendingChat) {
            if let conversation = pendingChatConversation {
                ChatView(conversation: conversation)
            }
        }
        .searchable(text: Binding(
            get: { viewModel?.searchText ?? "" },
            set: { viewModel?.searchText = $0 }
        ))
        .navigationTitle("Contacts")
        .toolbar {
            ToolbarItem(placement: .navigationBarTrailing) {
                Button {
                    nearbyPrefilledPeer = nil
                    showingAddContact = true
                } label: {
                    Image(systemName: "plus")
                }
            }
        }
        .sheet(isPresented: $showingAddContact, onDismiss: {
            nearbyPrefilledPeer = nil
            viewModel?.loadContacts()
            if pendingChatConversation != nil {
                navigateToPendingChat = true
            }
        }) {
            AddContactView(
                pendingChatConversation: $pendingChatConversation,
                prefilledPeer: nearbyPrefilledPeer
            )
        }
        .onAppear {
            if viewModel == nil {
                viewModel = ContactsViewModel(repository: repository)
                viewModel?.loadContacts()
            }
        }
        .alert(
            "Nearby Connect Failed",
            isPresented: Binding(
                get: { quickConnectError != nil },
                set: { if !$0 { quickConnectError = nil } }
            ),
            actions: {
                Button("OK", role: .cancel) { quickConnectError = nil }
            },
            message: {
                Text(quickConnectError ?? "Unknown error")
            }
        )
        .alert(
            "Delete Contact?",
            isPresented: Binding(
                get: { pendingDeletePeerId != nil },
                set: { if !$0 { pendingDeletePeerId = nil } }
            ),
            actions: {
                Button("Delete", role: .destructive) {
                    guard let peerId = pendingDeletePeerId else { return }
                    do {
                        try viewModel?.removeContact(peerId: peerId)
                        viewModel?.loadContacts()
                        actionError = nil
                    } catch {
                        actionError = "Failed to delete contact: \(error.localizedDescription)"
                    }
                    pendingDeletePeerId = nil
                }
                Button("Cancel", role: .cancel) {
                    pendingDeletePeerId = nil
                }
            },
            message: {
                Text("Remove \(pendingDeleteDisplayName) from contacts?")
            }
        )
        .alert(
            "Edit Nickname",
            isPresented: Binding(
                get: { editingContact != nil },
                set: { if !$0 { editingContact = nil } }
            ),
            actions: {
                TextField("Nickname", text: $editNickname)
                Button("Save") {
                    guard let contact = editingContact else { return }
                    let trimmed = editNickname.trimmingCharacters(in: .whitespacesAndNewlines)
                    do {
                        try viewModel?.setLocalNickname(peerId: contact.peerId, nickname: trimmed.isEmpty ? nil : trimmed)
                        viewModel?.loadContacts()
                        actionError = nil
                    } catch {
                        actionError = "Failed to save nickname: \(error.localizedDescription)"
                    }
                    editingContact = nil
                }
                Button("Cancel", role: .cancel) {
                    editingContact = nil
                }
            },
            message: {
                if let contact = editingContact {
                    if let federatedNick = contact.nickname, !federatedNick.isEmpty {
                        Text("Set local nickname (federated: @\(federatedNick))")
                    } else {
                        Text("Set local nickname for \(contact.peerId.prefix(16))...")
                    }
                } else {
                    Text("Set local nickname")
                }
            }
        )
        .sheet(isPresented: Binding(
            get: { contactToVerify != nil },
            set: { if !$0 { contactToVerify = nil } }
        )) {
            if let peerId = contactToVerify?.peerId, let viewModel {
                VerifySafetyNumberSheet(peerId: peerId, viewModel: viewModel)
            }
        }
        .confirmationDialog(
            "Block Contact?",
            isPresented: Binding(
                get: { contactToBlock != nil },
                set: { if !$0 { contactToBlock = nil } }
            ),
            titleVisibility: .visible
        ) {
            Button("Block", role: .destructive) {
                guard let contact = contactToBlock else { return }
                do {
                    try repository.blockPeer(peerId: contact.peerId, reason: "blocked_from_contacts")
                    actionError = nil
                } catch {
                    actionError = "Failed to block contact: \(error.localizedDescription)"
                }
                contactToBlock = nil
            }
            Button("Cancel", role: .cancel) {
                contactToBlock = nil
            }
        } message: {
            if let contact = contactToBlock {
                Text("Messages from \(conversationDisplayName(for: contact)) will be blocked. You can unblock them from Settings.")
            }
        }
        .alert(
            "Contact Action Failed",
            isPresented: Binding(
                get: { actionError != nil },
                set: { if !$0 { actionError = nil } }
            ),
            actions: {
                Button("OK", role: .cancel) { actionError = nil }
            },
            message: {
                Text(actionError ?? "Unknown contact action error")
            }
        )
    }

    private func quickConnect(_ peer: NearbyPeer) {
        guard let viewModel else { return }
        guard let publicKey = peer.publicKey?.trimmingCharacters(in: .whitespacesAndNewlines),
              publicKey.count == 64 else {
            nearbyPrefilledPeer = peer
            showingAddContact = true
            return
        }

        var notesParts: [String] = []
        if let blePeerId = peer.blePeerId?.trimmingCharacters(in: .whitespacesAndNewlines),
           !blePeerId.isEmpty {
            notesParts.append("ble_peer_id:\(blePeerId)")
        }
        if let libp2p = peer.libp2pPeerId?.trimmingCharacters(in: .whitespacesAndNewlines),
           !libp2p.isEmpty {
            notesParts.append("libp2p_peer_id:\(libp2p)")
        }
        if !peer.listeners.isEmpty {
            notesParts.append("listeners:\(peer.listeners.joined(separator: ","))")
        }

        let contact: Contact = Contact(
            peerId: peer.peerId,
            nickname: peer.nickname,
            localNickname: nil,
            publicKey: publicKey,
            addedAt: UInt64(Date().timeIntervalSince1970),
            lastSeen: nil,
            notes: notesParts.isEmpty ? nil : notesParts.joined(separator: ";"),
            lastKnownDeviceId: nil,
            verifiedAt: nil,
            isTombstone: false
        )

        do {
            try viewModel.addContact(contact)
            if !peer.listeners.isEmpty {
                let dialPeerId: String
                if let libp2pPeerId = peer.libp2pPeerId?.trimmingCharacters(in: .whitespacesAndNewlines),
                   !libp2pPeerId.isEmpty {
                    dialPeerId = libp2pPeerId
                } else {
                    dialPeerId = peer.peerId
                }
                Task {
                    await repository.connectToPeer(dialPeerId, addresses: peer.listeners)
                }
            }
            let displayName: String
            if let nickname = peer.nickname?.trimmingCharacters(in: .whitespacesAndNewlines),
               !nickname.isEmpty {
                displayName = nickname
            } else {
                displayName = String(peer.peerId.prefix(8))
            }
            pendingChatConversation = Conversation(peerId: peer.peerId, peerNickname: displayName)
            navigateToPendingChat = true
        } catch {
            quickConnectError = error.localizedDescription
        }
    }

    private func conversationDisplayName(for contact: Contact) -> String {
        let local = contact.localNickname?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        if !local.isEmpty { return local }
        let federated = contact.nickname?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        if !federated.isEmpty { return federated }
        return String(contact.peerId.prefix(8)) + "..."
    }

}

// MARK: - Nearby Peer Row

struct NearbyPeerRow: View {
    let peer: NearbyPeer
    let onAdd: () -> Void

    var body: some View {
        HStack(spacing: Theme.spacingMedium) {
            ZStack {
                Circle()
                    .fill(Color.green.opacity(0.15))
                    .frame(width: 44, height: 44)
                Image(systemName: "dot.radiowaves.left.and.right")
                    .foregroundStyle(Color.green)
            }

            VStack(alignment: .leading, spacing: 2) {
                Text(peer.displayName)
                    .font(Theme.titleMedium)
                if peer.hasFullIdentity {
                    Text("● Identity verified")
                        .font(.system(.caption2))
                        .foregroundStyle(Color.green)
                } else {
                    Text(peer.peerId.prefix(16))
                        .font(.system(.caption, design: .monospaced))
                        .foregroundStyle(Theme.onSurfaceVariant)
                }
            }

            Spacer()

            Button {
                onAdd()
            } label: {
                Label("Add", systemImage: "person.badge.plus")
                    .font(.system(size: 20))
                    .foregroundStyle(Color.accentColor)
            }
            .buttonStyle(.plain)
        }
        .padding(.vertical, 4)
    }
}

struct ContactRow: View {
    let contact: Contact

    var body: some View {
        let local = contact.localNickname?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        let primaryName = !local.isEmpty ? local : (contact.nickname ?? "Unknown")
        HStack(spacing: Theme.spacingMedium) {
            Circle()
                .fill(Theme.primaryContainer)
                .frame(width: 44, height: 44)
                .overlay {
                    Text(primaryName.prefix(1).uppercased())
                        .font(Theme.titleMedium)
                        .foregroundStyle(Theme.onPrimaryContainer)
                }

            VStack(alignment: .leading, spacing: 4) {
                Text(primaryName)
                    .font(Theme.titleMedium)
                if contact.localNickname != nil, let federated = contact.nickname, !federated.isEmpty {
                    Text("@\(federated)")
                        .font(.caption)
                        .foregroundStyle(Theme.onSurfaceVariant)
                }

                Text(contact.peerId.prefix(8))
                    .font(.system(.caption, design: .monospaced))
                    .foregroundStyle(Theme.onSurfaceVariant)
            }
        }
        .padding(.vertical, 4)
    }
}

@MainActor
struct AddContactView: View {
    @Environment(\.dismiss) private var dismiss
    @Environment(MeshRepository.self) private var repository
    @Binding var pendingChatConversation: Conversation?

    /// Pre-fill from a nearby peer when launched from the Nearby section.
    var prefilledPeer: NearbyPeer? = nil

    @State private var nickname: String = ""
    @State private var publicKey: String = ""
    @State private var peerId: String = ""
    @State private var listeners: [String] = []
    @State private var blePeerId: String = ""
    @State private var libp2pPeerId: String = ""
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
                Section("Contact Information") {
                    Button(action: pasteIdentity) {
                        Label("Paste Identity Export", systemImage: "doc.on.clipboard")
                    }

                    Button(action: { showingQrScanner = true }) {
                        Label("Scan Contact QR", systemImage: "qrcode.viewfinder")
                    }
                    .disabled(!canUseQrScanner)
                    if !canUseQrScanner {
                        Text("QR scanning is unavailable on this device.")
                            .font(Theme.bodySmall)
                            .foregroundStyle(.secondary)
                    }

                    TextField("Nickname", text: $nickname)
                    TextField("Public Key", text: $publicKey)
                        .font(.system(.body, design: .monospaced))
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()

                    if !peerId.isEmpty {
                        Text("ID: \(peerId)")
                            .font(.caption)
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
                    Button("Add Contact") {
                        addContact(andChat: false)
                    }
                    .disabled(publicKey.isEmpty)

                    Button("Add & Chat") {
                        addContact(andChat: true)
                    }
                    .disabled(publicKey.isEmpty)
                }
            }
            .navigationTitle("Add Contact")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") {
                        dismiss()
                    }
                }
            }
            .onAppear {
                if let peer = prefilledPeer, peerId.isEmpty {
                    peerId = peer.peerId
                    if let pk = peer.publicKey { publicKey = pk }
                    if let nick = peer.nickname, !nick.isEmpty { nickname = nick }
                    if let ble = peer.blePeerId, !ble.isEmpty { blePeerId = ble }
                    if let lpid = peer.libp2pPeerId, !lpid.isEmpty { libp2pPeerId = lpid }
                    listeners = peer.listeners
                }
            }
            .sheet(isPresented: $showingQrScanner) {
                if canUseQrScanner {
                    ContactQrScannerSheet(
                        onScan: { payload in
                            showingQrScanner = false
                            importQrPayload(payload)
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

    private func pasteIdentity() {
        guard let string = UIPasteboard.general.string else { return }
        applyImportedContact(raw: string, invalidFormatMessage: "Invalid format")
    }

    private func addContact(andChat: Bool) {
        let finalPublicKey = publicKey.trimmingCharacters(in: .whitespacesAndNewlines)
        var finalPeerId = peerId.trimmingCharacters(in: .whitespacesAndNewlines)
        if finalPeerId.isEmpty { finalPeerId = String(finalPublicKey.prefix(16)) }

        if finalPublicKey.isEmpty {
            self.error = "Public key cannot be empty"
            return
        }
        if finalPublicKey.count != 64 {
            self.error = "Public key must be exactly 64 hex characters (got \(finalPublicKey.count))"
            return
        }
        let hexCharacterSet: CharacterSet = CharacterSet(charactersIn: "0123456789abcdefABCDEF")
        if !finalPublicKey.unicodeScalars.allSatisfy({ hexCharacterSet.contains($0) }) {
            self.error = "Public key contains invalid characters (must be hex: 0-9, a-f)"
            return
        }

        // Store nearby route hints for sendMessage routing.
        var notesParts: [String] = []
        if !blePeerId.isEmpty {
            notesParts.append("ble_peer_id:\(blePeerId)")
        }
        if !libp2pPeerId.isEmpty {
            let addrs = listeners.joined(separator: ",")
            notesParts.append("libp2p_peer_id:\(libp2pPeerId)")
            notesParts.append("listeners:\(addrs)")
        }
        let notesValue: String? = notesParts.isEmpty ? nil : notesParts.joined(separator: ";")

        // UNIFIED ID FIX: Canonicalize peerId to public_key_hex before storage.
        // resolveIdentity handles all input formats (libp2p_peer_id, identity_id, public_key_hex).
        let canonicalPeerId: String
        do {
            if let resolved = try repository.ironCore?.resolveIdentity(anyId: finalPeerId) {
                canonicalPeerId = resolved
            } else {
                // Fallback: publicKey is already validated as canonical hex
                canonicalPeerId = finalPublicKey
            }
        } catch {
            canonicalPeerId = finalPublicKey
        }

        let contact: Contact = Contact(
            peerId: canonicalPeerId,
            nickname: nickname,
            localNickname: nil,
            publicKey: finalPublicKey,
            addedAt: UInt64(Date().timeIntervalSince1970),
            lastSeen: nil,
            notes: notesValue,
            lastKnownDeviceId: nil,
            verifiedAt: nil,
            isTombstone: false
        )

        do {
            try repository.addContact(contact)

            if !listeners.isEmpty {
                let peerIdForDial = libp2pPeerId.isEmpty ? finalPeerId : libp2pPeerId
                Task {
                    await repository.connectToPeer(peerIdForDial, addresses: listeners)
                }
            }

            if andChat {
                let normalizedNickname = nickname.trimmingCharacters(in: .whitespacesAndNewlines)
                let displayName = normalizedNickname.isEmpty ? String(canonicalPeerId.prefix(8)) + "..." : normalizedNickname
                pendingChatConversation = Conversation(peerId: canonicalPeerId, peerNickname: displayName)
            } else {
                pendingChatConversation = nil
            }

            dismiss()
        } catch {
            self.error = error.localizedDescription
        }
    }

    private func importQrPayload(_ raw: String) {
        applyImportedContact(raw: raw, invalidFormatMessage: "Invalid QR format")
    }

    private func applyImportedContact(raw: String, invalidFormatMessage: String) {
        guard let payload = parseImportedContactPayload(raw: raw) else {
            error = invalidFormatMessage
            return
        }

        if let key = payload.publicKey { publicKey = key }
        if let nick = payload.nickname { nickname = nick }
        if let pid = payload.peerId { peerId = pid }
        if let lpid = payload.libp2pPeerId { libp2pPeerId = lpid }
        if let addrs = payload.listeners { listeners = addrs }

        if !(payload.peerId ?? "").isEmpty && (payload.publicKey ?? "").isEmpty {
            error = "Identity ID was found, but public key is missing in this payload."
            return
        }
        error = nil
    }

    private func parseImportedContactPayload(raw: String) -> ImportedContactPayload? {
        guard let data = raw.data(using: .utf8),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            return nil
        }

        let publicKey = ((json["public_key"] as? String)
            ?? (json["publicKey"] as? String)
            ?? (json["publicKeyHex"] as? String))?
            .trimmingCharacters(in: .whitespacesAndNewlines)

        let nickname = (json["nickname"] as? String)?
            .trimmingCharacters(in: .whitespacesAndNewlines)

        // UNIFIED ID FIX: "peer_id" is libp2p Peer ID (network routable), NOT identity_id
        let peerId = ((json["peer_id"] as? String)
            ?? (json["libp2p_peer_id"] as? String)
            ?? (json["libp2pPeerId"] as? String)
            ?? (json["peerId"] as? String))?
            .trimmingCharacters(in: .whitespacesAndNewlines)

        let libp2pPeerId = ((json["libp2p_peer_id"] as? String)
            ?? (json["libp2pPeerId"] as? String)
            ?? (json["peer_id"] as? String))?
            .trimmingCharacters(in: .whitespacesAndNewlines)
        let normalizedLibp2pPeerId = libp2pPeerId.flatMap {
            PeerIdValidator.isLibp2pPeerId($0) ? $0 : nil
        }

        let listeners = (
            (json["listeners"] as? [String] ?? []) +
                (json["external_addresses"] as? [String] ?? []) +
                (json["connection_hints"] as? [String] ?? [])
        )
        .map { $0.replacingOccurrences(of: " (Potential)", with: "").trimmingCharacters(in: .whitespacesAndNewlines) }
        .filter { !$0.isEmpty }
        .reduce(into: [String]()) { acc, value in
            if !acc.contains(value) { acc.append(value) }
        }

        return ImportedContactPayload(
            peerId: peerId?.isEmpty == false ? peerId : nil,
            publicKey: publicKey?.isEmpty == false ? publicKey : nil,
            nickname: nickname?.isEmpty == false ? nickname : nil,
            libp2pPeerId: normalizedLibp2pPeerId,
            // Never auto-dial a listener without a validated libp2p route ID.
            listeners: normalizedLibp2pPeerId == nil || listeners.isEmpty ? nil : listeners
        )
    }

    private func conversationDisplayName(for contact: Contact) -> String {
        let local = contact.localNickname?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        if !local.isEmpty { return local }
        let federated = contact.nickname?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        if !federated.isEmpty { return federated }
        return String(contact.peerId.prefix(8)) + "..."
    }
}

private struct ImportedContactPayload {
    let peerId: String?
    let publicKey: String?
    let nickname: String?
    let libp2pPeerId: String?
    let listeners: [String]?
}

@available(iOS 16.0, *)
private struct ContactQrScannerSheet: UIViewControllerRepresentable {
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
