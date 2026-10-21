package com.scmessenger.shared.model

/**
 * Platform-agnostic service state matching the core ServiceState enum.
 */
enum class ServiceState {
    STOPPED, STARTING, RUNNING, ERROR;

    val isRunning: Boolean get() = this == RUNNING
    val isStopped: Boolean get() = this == STOPPED
}
