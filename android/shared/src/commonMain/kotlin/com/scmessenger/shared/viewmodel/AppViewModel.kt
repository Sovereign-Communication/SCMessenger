package com.scmessenger.shared.viewmodel

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
 * Shared ViewModel for overall app state.
 * Manages service lifecycle, connection status, and settings.
 */
open class AppViewModel(
    private val networking: PlatformNetworking,
    private val storagePath: String = ""
) {
    private val scope = CoroutineScope(Dispatchers.Main + SupervisorJob())

    private val _serviceState = MutableStateFlow(ServiceState.STOPPED)
    val serviceState: StateFlow<ServiceState> = _serviceState.asStateFlow()

    private val _unreadCount = MutableStateFlow(0)
    val unreadCount: StateFlow<Int> = _unreadCount.asStateFlow()

    private val _connectionStatus = MutableStateFlow("Disconnected")
    val connectionStatus: StateFlow<String> = _connectionStatus.asStateFlow()

    fun startService() {
        if (_serviceState.value.isRunning) return
        scope.launch {
            _serviceState.value = ServiceState.STARTING
            try {
                networking.start(storagePath)
                _serviceState.value = ServiceState.RUNNING
                _connectionStatus.value = "Connected"
            } catch (e: Exception) {
                _serviceState.value = ServiceState.ERROR
                _connectionStatus.value = "Error: ${e.message}"
            }
        }
    }

    fun stopService() {
        if (_serviceState.value.isStopped) return
        scope.launch {
            try {
                networking.stop()
                _serviceState.value = ServiceState.STOPPED
                _connectionStatus.value = "Disconnected"
            } catch (e: Exception) {
                _serviceState.value = ServiceState.ERROR
            }
        }
    }

    fun toggleService() {
        if (_serviceState.value.isRunning) stopService() else startService()
    }
}
