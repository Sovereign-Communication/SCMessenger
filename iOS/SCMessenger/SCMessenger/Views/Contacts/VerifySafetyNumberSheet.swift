//
//  VerifySafetyNumberSheet.swift
//  SCMessenger
//
//  Safety-number verification: derives a Signal-style numeric fingerprint
//  from our identity and the contact's public key, so the user can compare
//  it with the contact out-of-band (in person, over a trusted channel)
//  before marking the contact as verified.
//

import SwiftUI

@MainActor
struct VerifySafetyNumberSheet: View {
    let peerId: String
    let viewModel: ContactsViewModel
    @Environment(\.dismiss) private var dismiss
    @State private var actionError: String?

    private var contact: Contact? {
        viewModel.contacts.first { $0.peerId == peerId }
    }

    /// `nil` means "no contact" or "identity not initialized yet";
    /// `Some("")` means the underlying Rust `safetyNumber()` rejected a
    /// malformed key - distinct from a real (matching) safety number, so it
    /// must never be rendered or allowed to back a verification action (S5).
    private var safetyNumberRaw: String? {
        guard let contact else { return nil }
        return viewModel.computeSafetyNumber(theirPublicKeyHex: contact.publicKey)
    }

    private var safetyNumber: String? {
        guard let raw = safetyNumberRaw, !raw.isEmpty else { return nil }
        return raw
    }

    private var safetyNumberIsInvalid: Bool {
        safetyNumberRaw?.isEmpty == true
    }

    var body: some View {
        NavigationStack {
            VStack(spacing: Theme.spacingMedium) {
                if safetyNumberIsInvalid {
                    Text("Safety number unavailable — key data invalid")
                        .foregroundStyle(.secondary)
                } else if let contact, let safetyNumber {
                    Text("Compare this number with \(displayName(for: contact)), in person or through another trusted channel. If they match, your conversation is secure from eavesdropping.")
                        .font(Theme.bodySmall)
                        .foregroundStyle(.secondary)
                        .multilineTextAlignment(.center)
                        .padding(.horizontal, Theme.spacingLarge)

                    if let image = QRCodeGenerator.image(from: safetyNumber) {
                        Image(uiImage: image)
                            .interpolation(.none)
                            .resizable()
                            .scaledToFit()
                            .frame(maxWidth: 280, maxHeight: 280)
                    }

                    Text(safetyNumber)
                        .font(.system(.title3, design: .monospaced))
                        .multilineTextAlignment(.center)
                        .padding(.horizontal, Theme.spacingLarge)

                    verificationStatus(for: contact)

                    if let actionError {
                        Text(actionError)
                            .font(Theme.bodySmall)
                            .foregroundStyle(.red)
                    }

                    if contact.verifiedAt != nil {
                        Button("Clear Verification", role: .destructive) {
                            do {
                                try viewModel.unverifyContact(peerId: peerId)
                                actionError = nil
                            } catch {
                                actionError = "Failed to clear verification: \(error.localizedDescription)"
                            }
                        }
                        .buttonStyle(.bordered)
                    } else {
                        Button {
                            do {
                                try viewModel.markContactVerified(peerId: peerId)
                                actionError = nil
                            } catch {
                                actionError = "Failed to mark as verified: \(error.localizedDescription)"
                            }
                        } label: {
                            Label("Mark as Verified", systemImage: "checkmark.shield.fill")
                        }
                        .buttonStyle(.borderedProminent)
                    }
                } else if contact == nil {
                    Text("Contact not found")
                        .foregroundStyle(.secondary)
                } else {
                    Text("Your identity isn't initialized yet")
                        .foregroundStyle(.secondary)
                }
            }
            .padding(Theme.spacingLarge)
            .navigationTitle("Verify Safety Number")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Done") { dismiss() }
                }
            }
        }
    }

    private func displayName(for contact: Contact) -> String {
        let local: String = contact.localNickname?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        if !local.isEmpty { return local }
        let federated: String = contact.nickname?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        if !federated.isEmpty { return federated }
        return String(contact.peerId.prefix(16))
    }

    @ViewBuilder
    private func verificationStatus(for contact: Contact) -> some View {
        HStack(spacing: Theme.spacingSmall) {
            Image(systemName: contact.verifiedAt != nil ? "checkmark.shield.fill" : "shield")
                .foregroundStyle(contact.verifiedAt != nil ? Color.green : .secondary)
            if let verifiedAt = contact.verifiedAt {
                Text("Verified on \(formattedDate(verifiedAt))")
                    .font(Theme.bodySmall.weight(.medium))
            } else {
                Text("Not verified")
                    .font(Theme.bodySmall.weight(.medium))
                    .foregroundStyle(.secondary)
            }
        }
    }

    private func formattedDate(_ timestamp: UInt64) -> String {
        let date: Date = Date(timeIntervalSince1970: TimeInterval(timestamp))
        let formatter: DateFormatter = DateFormatter()
        formatter.dateStyle = .medium
        formatter.timeStyle = .short
        return formatter.string(from: date)
    }
}
