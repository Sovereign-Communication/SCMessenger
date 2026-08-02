//
//  MainTabView.swift
//  SCMessenger
//
//  Main tab navigation
//

import SwiftUI
import UIKit
import OSLog

enum MessagesRoute: Hashable {
    case conversation(Conversation)
    case requestsInbox
}

@MainActor
struct MainTabView: View {
    @Environment(MeshRepository.self) private var repository
    @State private var identityInitialized: Bool = false
    @State private var selectedTab: MainTab = .mesh
    @State private var messagesPath: [MessagesRoute] = []

    private enum MainTab: Hashable {
        case messages
        case contacts
        case mesh
        case settings
    }

    var body: some View {
        TabView(selection: $selectedTab) {
            if identityInitialized {
                NavigationStack(path: $messagesPath) {
                    ConversationListView(onOpenRequestsInbox: openRequestsInbox)
                        .navigationDestination(for: MessagesRoute.self) { route in
                            switch route {
                            case .conversation(let conversation):
                                ChatView(conversation: conversation)
                            case .requestsInbox:
                                RequestsInboxView(onOpenConversation: openConversation)
                            }
                        }
                }
                .tabItem {
                    Label("Messages", systemImage: "message")
                }
                .tag(MainTab.messages)

                NavigationStack {
                    ContactsListView()
                }
                .tabItem {
                    Label("Contacts", systemImage: "person.2")
                }
                .tag(MainTab.contacts)
            }

            NavigationStack {
                MeshDashboardView()
            }
            .tabItem {
                Label("Mesh", systemImage: "network")
            }
            .tag(MainTab.mesh)

            NavigationStack {
                SettingsView(
                    onIdentityChanged: refreshIdentityRole
                )
            }
            .tabItem {
                Label("Settings", systemImage: "gear")
            }
            .tag(MainTab.settings)
        }
        .onAppear {
            repository.start()
            repository.setNotificationAppInForeground(true)
            refreshIdentityRole()
        }
        .onChange(of: repository.identityHydrationState) { _, _ in
            refreshIdentityRole()
        }
        .onChange(of: repository.identityInfo) { _, _ in
            refreshIdentityRole()
        }
        .onReceive(NotificationCenter.default.publisher(for: UIApplication.willEnterForegroundNotification)) { _ in
            repository.setNotificationAppInForeground(true)
            refreshIdentityRole()
        }
        .onReceive(NotificationCenter.default.publisher(for: UIApplication.didEnterBackgroundNotification)) { _ in
            repository.setNotificationAppInForeground(false)
        }
        .onReceive(NotificationCenter.default.publisher(for: .notificationRouteRequested)) { notification in
            guard let userInfo = notification.userInfo else { return }
            handleNotificationRoute(userInfo: userInfo)
        }
    }

    private func refreshIdentityRole() {
        identityInitialized = repository.hasVerifiedIdentity
        if !identityInitialized && (selectedTab == .messages || selectedTab == .contacts) {
            selectedTab = .mesh
        }
    }

    private func handleNotificationRoute(userInfo: [AnyHashable: Any]) {
        selectedTab = .messages
        messagesPath.removeAll()
        if (userInfo["routeTarget"] as? String) == "requests" {
            openRequestsInbox()
            return
        }
        guard let peerId = userInfo["conversationId"] as? String ?? userInfo["senderPeerId"] as? String else {
            return
        }
        let conversation: Conversation = Conversation(
            peerId: peerId,
            peerNickname: repository.displayNameForPeer(peerId: peerId)
        )
        openConversation(conversation)
    }

    private func openConversation(_ conversation: Conversation) {
        selectedTab = .messages
        messagesPath = [.conversation(conversation)]
    }

    private func openRequestsInbox() {
        selectedTab = .messages
        messagesPath = [.requestsInbox]
    }
}

@MainActor
struct ConversationListView: View {
    @Environment(MeshRepository.self) private var repository
    @State private var conversations: [Conversation] = []
    @State private var requestCount: Int = 0
    @State private var conversationToDelete: Conversation?
    @State private var showingDeleteConfirmation: Bool = false
    private let logger: Logger = Logger(subsystem: "com.scmessenger", category: "ConversationList")
    let onOpenRequestsInbox: () -> Void

    var body: some View {
        List {
            if requestCount > 0 {
                Section {
                    Button(action: onOpenRequestsInbox) {
                        HStack(spacing: Theme.spacingMedium) {
                            Image(systemName: "person.crop.circle.badge.exclamationmark")
                                .font(.title2)
                                .foregroundStyle(Theme.onPrimaryContainer)
                            VStack(alignment: .leading, spacing: 4) {
                                Text("Message Requests")
                                    .font(Theme.titleMedium.weight(.semibold))
                                Text("\(requestCount) pending request\(requestCount == 1 ? "" : "s")")
                                    .font(Theme.bodySmall)
                                    .foregroundStyle(Theme.onSurfaceVariant)
                            }
                            Spacer()
                            Image(systemName: "chevron.right")
                                .foregroundStyle(Theme.onSurfaceVariant)
                        }
                    }
                    .buttonStyle(.plain)
                }
            }

            ForEach(conversations) { conversation in
                NavigationLink(value: MessagesRoute.conversation(conversation)) {
                    ConversationRow(conversation: conversation)
                }
                .swipeActions(edge: .trailing, allowsFullSwipe: false) {
                    Button(role: .destructive) {
                        conversationToDelete = conversation
                        showingDeleteConfirmation = true
                    } label: {
                        Label("Delete", systemImage: "trash")
                    }
                }
            }
        }
        .navigationTitle("Messages")
        .toolbar {
            ToolbarItem(placement: .navigationBarTrailing) {
                Button(action: {}) {
                    Image(systemName: "square.and.pencil")
                }
            }
        }
        .task {
            loadConversations()
            loadRequestCount()
        }
        .onReceive(repository.messageUpdates) { _ in
            loadConversations()
            loadRequestCount()
        }
        .onReceive(MeshEventBus.shared.peerEvents) { event in
            switch event {
            case .identityDiscovered:
                // Refresh display names when federated identity metadata changes.
                loadConversations()
            default:
                break
            }
        }
        .confirmationDialog(
            "Delete Conversation?",
            isPresented: $showingDeleteConfirmation,
            titleVisibility: .visible
        ) {
            Button("Delete", role: .destructive) {
                if let conv = conversationToDelete {
                    deleteConversation(conv)
                }
            }
            Button("Cancel", role: .cancel) { }
        } message: {
            if let conv = conversationToDelete {
                Text("Delete all messages with \(conv.peerNickname)? This cannot be undone.")
            }
        }
    }

    private func deleteConversation(_ conversation: Conversation) {
        do {
            try repository.clearConversation(peerId: conversation.peerId)
        } catch {
            logger.error("Failed to clear conversation for \(conversation.peerId, privacy: .private): \(error.localizedDescription, privacy: .public)")
        }
        conversations.removeAll { $0.id == conversation.id }
    }

    private func loadConversations() {
        // Load conversations from repository
        do {
            let contacts = try repository.getContacts()
            let requestPeerIds = Set(repository.getMessageRequests().map(\.peerId))
            let deduped = deduplicateContactsByPublicKey(contacts)
            conversations = deduped
                .filter { !requestPeerIds.contains($0.peerId) }
                .map { contact in
                let local = contact.localNickname?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
                let displayName = !local.isEmpty
                    ? local
                    : (contact.nickname ?? String(contact.peerId.prefix(8)) + "...")
                let recentMsgs = (try? repository.getConversation(peerId: contact.peerId, limit: 25)) ?? []
                let lastMsg = recentMsgs.max(by: {
                    let t0 = $0.senderTimestamp > 0 ? $0.senderTimestamp : $0.timestamp
                    let t1 = $1.senderTimestamp > 0 ? $1.senderTimestamp : $1.timestamp
                    if t0 == t1 { return $0.timestamp < $1.timestamp }
                    return t0 < t1
                })
                let lastTime = lastMsg.map { Date(timeIntervalSince1970: Double($0.senderTimestamp > 0 ? $0.senderTimestamp : $0.timestamp)) }
                return Conversation(
                    peerId: contact.peerId,
                    peerNickname: displayName,
                    lastMessage: lastMsg?.content,
                    lastMessageTime: lastTime
                )
            }
            .sorted { ($0.lastMessageTime ?? Date.distantPast) > ($1.lastMessageTime ?? Date.distantPast) }
        } catch {
            logger.error("Failed to load conversations: \(error.localizedDescription, privacy: .public)")
        }
    }

    private func loadRequestCount() {
        requestCount = repository.getMessageRequests().count
    }

    private func deduplicateContactsByPublicKey(_ input: [Contact]) -> [Contact] {
        var byKey: [String: Contact] = [:]
        var passthrough: [Contact] = []

        for contact in input {
            let normalizedKey = contact.publicKey.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
            guard normalizedKey.count == 64 else {
                passthrough.append(contact)
                continue
            }

            if let current = byKey[normalizedKey] {
                byKey[normalizedKey] = preferredContact(current, contact)
            } else {
                byKey[normalizedKey] = contact
            }
        }

        return Array(byKey.values) + passthrough
    }

    private func preferredContact(_ a: Contact, _ b: Contact) -> Contact {
        let aLocal = a.localNickname?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        let bLocal = b.localNickname?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        let aName = !aLocal.isEmpty ? aLocal : (a.nickname?.trimmingCharacters(in: .whitespacesAndNewlines) ?? "")
        let bName = !bLocal.isEmpty ? bLocal : (b.nickname?.trimmingCharacters(in: .whitespacesAndNewlines) ?? "")

        if !aName.isEmpty && bName.isEmpty { return a }
        if !bName.isEmpty && aName.isEmpty { return b }
        if a.peerId.hasPrefix("12D3Koo") && !b.peerId.hasPrefix("12D3Koo") { return a }
        if b.peerId.hasPrefix("12D3Koo") && !a.peerId.hasPrefix("12D3Koo") { return b }
        return a
    }
}

@MainActor
struct RequestsInboxView: View {
    @Environment(MeshRepository.self) private var repository
    @State private var requests: [MessageRequestThread] = []
    @State private var error: String?
    let onOpenConversation: (Conversation) -> Void

    var body: some View {
        List {
            if let error = error {
                Section {
                    Text(error)
                        .foregroundStyle(.red)
                        .font(Theme.bodySmall)
                }
            }

            if requests.isEmpty {
                ContentUnavailableView(
                    "No Message Requests",
                    systemImage: "tray",
                    description: Text("Unknown senders will appear here until you accept them.")
                )
            } else {
                ForEach(requests) { request in
                    Button {
                        onOpenConversation(request.conversation)
                    } label: {
                        ConversationRow(conversation: request.conversation)
                    }
                    .buttonStyle(.plain)
                    .swipeActions(edge: .trailing, allowsFullSwipe: true) {
                        Button("Accept") {
                            do {
                                try repository.acceptMessageRequest(peerId: request.peerId)
                                error = nil
                                loadRequests()
                            } catch {
                                self.error = error.localizedDescription
                            }
                        }
                        .tint(.green)
                    }
                }
            }
        }
        .navigationTitle("Message Requests")
        .task {
            loadRequests()
        }
        .onReceive(repository.messageUpdates) { _ in
            loadRequests()
        }
    }

    private func loadRequests() {
        requests = repository.getMessageRequests()
    }
}

struct ConversationRow: View {
    let conversation: Conversation

    var body: some View {
        HStack(spacing: Theme.spacingMedium) {
            Circle()
                .fill(Theme.primaryContainer)
                .frame(width: 50, height: 50)
                .overlay {
                    Text(conversation.peerNickname.prefix(1).uppercased())
                        .font(Theme.titleMedium)
                        .foregroundStyle(Theme.onPrimaryContainer)
                }

            VStack(alignment: .leading, spacing: 4) {
                Text(conversation.peerNickname)
                    .font(Theme.titleMedium)

                if let lastMessage = conversation.lastMessage {
                    Text(lastMessage)
                        .font(Theme.bodySmall)
                        .foregroundStyle(Theme.onSurfaceVariant)
                        .lineLimit(1)
                }
            }

            Spacer()

            VStack(alignment: .trailing, spacing: 4) {
                if let lastTime = conversation.lastMessageTime {
                    Text(formatMessageDate(lastTime))
                        .font(Theme.labelSmall)
                        .foregroundStyle(Theme.onSurfaceVariant)
                }

                if conversation.unreadCount > 0 {
                    Text("\(conversation.unreadCount)")
                        .font(Theme.labelSmall)
                        .foregroundStyle(.white)
                        .padding(.horizontal, 8)
                        .padding(.vertical, 4)
                        .background(Theme.onPrimaryContainer)
                        .clipShape(Capsule())
                }
            }
        }
        .padding(.vertical, 4)
    }

    private func formatMessageDate(_ date: Date) -> String {
        let formatter: DateFormatter = DateFormatter()
        formatter.dateStyle = Calendar.current.isDateInToday(date) ? .none : .short
        formatter.timeStyle = .short
        return formatter.string(from: date)
    }
}

struct ChatView: View {
    @Environment(MeshRepository.self) private var repository
    let conversation: Conversation
    @State private var viewModel: ChatViewModel?
    @State private var scrollProxy: ScrollViewProxy?
    @State private var isAtBottom: Bool = true
    @State private var lastMessageId: String?

    var body: some View {
        VStack(spacing: 0) {

            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(spacing: 0) {
                        ForEach(viewModel?.messages ?? [], id: \.id) { message in
                            MessageBubble(message: message)
                                .id(message.id)
                        }
                        // Invisible anchor at the bottom for auto-scroll
                        Color.clear
                            .frame(height: 1)
                            .id("bottom")
                    }
                }
                .scrollDismissesKeyboard(.interactively)
                .onAppear {
                    scrollProxy = proxy
                    // Scroll to bottom when first opened
                    DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
                        proxy.scrollTo("bottom", anchor: .bottom)
                    }
                }
                .onChange(of: viewModel?.messages.count) { _, newCount in
                    // Only auto-scroll when a *new* message arrives (count increases),
                    // NOT on delivery-state updates which keep the same count.
                    guard let newCount, newCount > 0 else { return }
                    let newLastId = viewModel?.messages.last?.id
                    guard newLastId != lastMessageId else { return }
                    lastMessageId = newLastId
                    if isAtBottom {
                        withAnimation(.easeOut(duration: 0.2)) {
                            proxy.scrollTo("bottom", anchor: .bottom)
                        }
                    }
                }
            }

            MessageInputBar(
                text: Binding(
                    get: { viewModel?.messageText ?? "" },
                    set: { viewModel?.messageText = $0 }
                ),
                onSend: {
                    viewModel?.sendMessage()
                    withAnimation(.easeOut(duration: 0.2)) {
                        scrollProxy?.scrollTo("bottom", anchor: .bottom)
                    }
                }
            )

            if let error = viewModel?.error {
                Text(error)
                    .foregroundStyle(.white)
                    .padding()
                    .background(.red)
                    .cornerRadius(8)
                    .padding()
                    .onTapGesture {
                        viewModel?.error = nil
                    }
            }
        }
        .navigationTitle(conversation.peerNickname)
        .navigationBarTitleDisplayMode(.inline)
        .onAppear {
            if viewModel == nil {
                viewModel = ChatViewModel(conversation: conversation, repository: repository)
            }
            repository.setNotificationActiveConversation(peerId: conversation.peerId)
        }
        .onDisappear {
            repository.setNotificationActiveConversation(peerId: nil)
        }
    }
}

/// Zero-Status Architecture: displays only message content (text)
/// and sender-assigned timestamp (`senderTimestamp`, the time the message was saved
/// to local storage for sending). No delivery status indicators.
struct MessageBubble: View {
    let message: MessageRecord

    private var isSent: Bool {
        message.direction == .sent
    }

    var body: some View {
        HStack {
            if isSent { Spacer() }

            VStack(alignment: isSent ? .trailing : .leading, spacing: 2) {
                Text(message.content)
                    .font(Theme.bodyMedium)

                // senderTimestamp: the time the message was saved to local storage for sending
                let msgDate = Date(timeIntervalSince1970: Double(message.senderTimestamp))
                Text(formatMessageDate(msgDate))
                    .font(Theme.labelSmall)
                    .foregroundStyle(isSent ? Theme.onPrimaryContainer.opacity(0.8) : Theme.onSurface.opacity(0.8))
            }
            .padding(Theme.spacingMedium)
            .background(isSent ? Theme.primaryContainer : Theme.surfaceVariant)
            .foregroundStyle(isSent ? Theme.onPrimaryContainer : Theme.onSurface)
            .cornerRadius(Theme.cornerRadiusMedium)

            if !isSent { Spacer() }
        }
        .padding(.horizontal, Theme.spacingMedium)
        .padding(.vertical, Theme.spacingSmall)
    }

    private func formatMessageDate(_ date: Date) -> String {
        let formatter: DateFormatter = DateFormatter()
        formatter.dateStyle = Calendar.current.isDateInToday(date) ? .none : .short
        formatter.timeStyle = .short
        return formatter.string(from: date)
    }
}

struct MessageInputBar: View {
    @Binding var text: String
    let onSend: () -> Void

    var body: some View {
        HStack(spacing: Theme.spacingSmall) {
            TextField("Message", text: $text, axis: .vertical)
                .textFieldStyle(.roundedBorder)
                .lineLimit(1...4)

            Button(action: onSend) {
                Image(systemName: "arrow.up.circle.fill")
                    .font(.title2)
                    .foregroundStyle(text.isEmpty ? .gray : Theme.onPrimaryContainer)
            }
            .disabled(text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
        }
        .padding(Theme.spacingMedium)
        .background(Theme.surface)
    }
}
