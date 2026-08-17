//
//  ChatViewModel.swift
//  SCMessenger
//
//  ViewModel for chat/messaging
//

import Foundation
import Combine

@MainActor
@Observable
final class ChatViewModel {
    private weak var repository: MeshRepository?
    private var cancellables: Set<AnyCancellable> = Set<AnyCancellable>()

    let conversation: Conversation
    var messages: [MessageRecord] = []
    var messageText: String = ""
    var error: String?

    init(conversation: Conversation, repository: MeshRepository) {
        self.conversation = conversation
        self.repository = repository
        loadMessages()
        subscribeToNewMessages()
        subscribeToMessageEvents()
    }

    func loadMessages() {
        do {
            let fetched: [MessageRecord] = try repository?.getConversation(peerId: conversation.peerId) ?? []
            messages = fetched.sorted(by: { lhs, rhs in
                let t1: UInt64 = lhs.senderTimestamp > 0 ? lhs.senderTimestamp : lhs.timestamp
                let t2: UInt64 = rhs.senderTimestamp > 0 ? rhs.senderTimestamp : rhs.timestamp
                if t1 == t2 { return lhs.timestamp < rhs.timestamp }
                return t1 < t2
            })
        } catch {
            self.error = error.localizedDescription
        }
    }

    func sendMessage() {
        let content: String = messageText.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !content.isEmpty else { return }

        messageText = ""
        error = nil

        let now: UInt64 = UInt64(Date().timeIntervalSince1970)
        let optimisticMessage: MessageRecord = MessageRecord(
            id: UUID().uuidString,
            direction: .sent,
            peerId: conversation.peerId,
            content: content,
            timestamp: now,
            senderTimestamp: now,
            delivered: false,
            status: .queued,
            hidden: false
        )
        messages.append(optimisticMessage)
        messages.sort { lhs, rhs in
            let t1: UInt64 = lhs.senderTimestamp > 0 ? lhs.senderTimestamp : lhs.timestamp
            let t2: UInt64 = rhs.senderTimestamp > 0 ? rhs.senderTimestamp : rhs.timestamp
            if t1 == t2 { return lhs.timestamp < rhs.timestamp }
            return t1 < t2
        }

        Task { @MainActor [weak self] in
            guard let self else { return }

            do {
                try await repository?.sendMessage(peerId: conversation.peerId, content: content)
                self.error = nil
            } catch let error as IronCoreError {
                switch error {
                case .CryptoError(let message):
                    self.error = "Crypto Error: \(message)"
                case .NetworkError(let message):
                    self.error = "Network Error: \(message)"
                case .StorageError(let message):
                    self.error = "Storage Error: \(message)"
                case .NotInitialized(let message):
                    self.error = "Not Initialized: \(message)"
                case .InvalidInput:
                    self.error = "Could not encrypt message — this contact may have an invalid public key. Try re-adding them using their identity export."
                case .Internal(let message):
                    self.error = "Internal Error: \(message)"
                case .Blocked(let message):
                    self.error = "Blocked: \(message)"
                case .AlreadyRunning(let message):
                    self.error = "Already Running: \(message)"
                case .ConsentRequired(let message):
                    self.error = "Consent Required: \(message)"
                default:
                    self.error = "Unknown IronCore Error"
                }
                self.loadMessages()
            } catch {
                self.error = error.localizedDescription
                self.loadMessages()
            }
        }
    }

    func statusGlyph(for status: MessageStatus) -> String {
        switch status {
        case .queued: return "clock"
        case .inCustody: return "arrow.triangle.2.circlepath"
        case .sent: return "checkmark"
        case .delivered: return "checkmark.circle"
        }
    }

    private var reloadDebounceTask: Task<Void, Never>?

    private func scheduleReload() {
        reloadDebounceTask?.cancel()
        reloadDebounceTask = Task { @MainActor [weak self] in
            try? await Task.sleep(nanoseconds: 80_000_000)
            guard !Task.isCancelled else { return }
            self?.loadMessages()
        }
    }

    private func subscribeToNewMessages() {
        repository?.messageUpdates
            .filter { [weak self] message in
                message.peerId == self?.conversation.peerId
            }
            .sink { [weak self] _ in
                self?.scheduleReload()
            }
            .store(in: &cancellables)
    }

    private func subscribeToMessageEvents() {
        MeshEventBus.shared.messageEvents
            .receive(on: DispatchQueue.main)
            .sink { [weak self] event in
                guard let self else { return }
                switch event {
                case .sent, .delivered, .failed:
                    // The event ID is authoritative and may differ from the
                    // optimistic UUID used by the chat UI. Reload the current
                    // conversation so the repository record is reflected.
                    self.scheduleReload()
                case .received:
                    break
                }
            }
            .store(in: &cancellables)
    }
}
