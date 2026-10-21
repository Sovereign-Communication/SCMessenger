package com.scmessenger.shared.viewmodel

import com.scmessenger.shared.model.Contact
import com.scmessenger.shared.model.ServiceState
import com.scmessenger.shared.platform.PlatformNetworking
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

/**
 * Shared ViewModel for managing contacts.
 * Uses expect/actual PlatformNetworking for platform-specific peer discovery.
 */
open class ContactsViewModel(
    private val networking: PlatformNetworking
) {
    private val scope = CoroutineScope(Dispatchers.Main + SupervisorJob())

    private val _contacts = MutableStateFlow<List<Contact>>(emptyList())
    val contacts: StateFlow<List<Contact>> = _contacts.asStateFlow()

    private val _selectedContact = MutableStateFlow<Contact?>(null)
    val selectedContact: StateFlow<Contact?> = _selectedContact.asStateFlow()

    private val _isLoading = MutableStateFlow(false)
    val isLoading: StateFlow<Boolean> = _isLoading.asStateFlow()

    fun selectContact(contact: Contact) {
        _selectedContact.value = contact
    }

    fun clearSelection() {
        _selectedContact.value = null
    }

    fun loadContacts() {
        scope.launch {
            _isLoading.value = true
            try {
                val peers = networking.getDiscoveredPeers()
                _contacts.value = peers.map { peerId ->
                    Contact(
                        id = peerId,
                        publicKey = peerId,
                        lastSeen = System.currentTimeMillis()
                    )
                }
            } catch (e: Exception) {
                // TODO: Error handling
            } finally {
                _isLoading.value = false
            }
        }
    }
}
