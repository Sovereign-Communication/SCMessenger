package com.scmessenger.android.service

import android.content.Context
import androidx.work.WorkManager
import com.scmessenger.android.MeshApplication
import timber.log.Timber

/**
 * Idempotent open/close pair for one in-process OS registration (for example a
 * ConnectivityManager.NetworkCallback). [open] does nothing while already open
 * and [close] does nothing while already closed, so teardown may run more than
 * once without a double-unregister. If [onOpen] throws, the resource stays closed.
 *
 * Only for registrations that die with the process. Work persisted in WorkManager
 * survives process death, so it must be cancelled unconditionally: see [RevivalWork].
 */
internal class ManagedResource(
    private val onOpen: () -> Unit,
    private val onClose: () -> Unit
) {
    private var opened = false

    @Synchronized
    fun open() {
        if (opened) return
        onOpen()
        opened = true
    }

    @Synchronized
    fun close() {
        if (!opened) return
        opened = false
        onClose()
    }
}

/** The WorkManager calls the revival chain needs. Faked in JVM tests. */
internal interface RevivalWorkBackend {
    fun enqueuePeriodicSync()
    fun cancelPeriodicSync()
    fun cancelEnsureWorker()
}

/**
 * Periodic re-ensure (MeshSyncWorker) and the expedited ensure worker.
 *
 * Start re-enqueues the periodic work (KEEP, so repeats are no-ops). User Stop
 * cancels both, and cancels unconditionally: after a process death the
 * in-memory state is empty, but the periodic work persisted in WorkManager and
 * must still be cancelled.
 */
internal class RevivalWork(private val backend: RevivalWorkBackend) {
    fun onMeshStarted() {
        backend.enqueuePeriodicSync()
    }

    fun onUserStopped() {
        backend.cancelPeriodicSync()
        backend.cancelEnsureWorker()
    }
}

/** WorkManager-backed [RevivalWorkBackend]. Failures are logged, never thrown. */
internal class WorkManagerRevivalBackend(context: Context) : RevivalWorkBackend {
    private val appContext = context.applicationContext

    override fun enqueuePeriodicSync() {
        kotlin.runCatching {
            WorkManager.getInstance(appContext).enqueueUniquePeriodicWork(
                MeshApplication.MESH_SYNC_WORK_NAME,
                MeshApplication.MESH_SYNC_WORK_POLICY,
                MeshApplication.buildMeshSyncWorkRequest()
            )
        }.onFailure { Timber.w(it, "Could not enqueue periodic mesh sync") }
    }

    override fun cancelPeriodicSync() {
        kotlin.runCatching {
            WorkManager.getInstance(appContext).cancelUniqueWork(MeshApplication.MESH_SYNC_WORK_NAME)
        }.onFailure { Timber.w(it, "Could not cancel periodic mesh sync") }
    }

    override fun cancelEnsureWorker() {
        kotlin.runCatching {
            WorkManager.getInstance(appContext).cancelUniqueWork(MeshAutoRestart.ENSURE_WORK_NAME)
        }.onFailure { Timber.w(it, "Could not cancel mesh ensure worker") }
    }
}

/** Revival work bound to WorkManager for [context]. */
internal fun revivalWorkFor(context: Context): RevivalWork = RevivalWork(WorkManagerRevivalBackend(context))
