package com.scmessenger.android.service

import android.content.Context
import androidx.work.CoroutineWorker
import androidx.work.WorkerParameters
import com.scmessenger.android.data.MeshRepository
import dagger.hilt.EntryPoint
import dagger.hilt.InstallIn
import dagger.hilt.android.EntryPointAccessors
import dagger.hilt.components.SingletonComponent
import timber.log.Timber

/**
 * WorkManager worker that periodically executes the Rust core's maintenance cycle.
 * This sweeps/syncs delay-tolerant outbox envelopes and handles peer-mesh routing cleanup.
 */
class MeshSyncWorker(
    context: Context,
    params: WorkerParameters
) : CoroutineWorker(context, params) {

    @EntryPoint
    @InstallIn(SingletonComponent::class)
    interface MeshSyncWorkerEntryPoint {
        fun getMeshRepository(): MeshRepository
    }

    override suspend fun doWork(): Result {
        Timber.i("MeshSyncWorker: background maintenance cycle triggered")
        // The mesh must keep running unless the user stopped it: re-ensure on
        // every periodic run, before (and independent of) the maintenance cycle.
        MeshAutoRestart.ensure(applicationContext, MeshStartTrigger.WORKER)
        return try {
            val entryPoint = EntryPointAccessors.fromApplication(
                applicationContext,
                MeshSyncWorkerEntryPoint::class.java
            )
            val meshRepository = entryPoint.getMeshRepository()
            val meshService = meshRepository.getMeshService()
            if (meshService != null && meshService.isRunning()) {
                val core = meshService.getCore()
                if (core != null) {
                    // Run maintenance cycle with a 25-second budget
                    val report = core.runMaintenanceCycle(25000u)
                    Timber.i("MeshSyncWorker: maintenance cycle report: $report")
                } else {
                    Timber.w("MeshSyncWorker: IronCore instance not available")
                }
            } else {
                Timber.w("MeshSyncWorker: MeshService is not running or initialized")
            }
            Result.success()
        } catch (e: Exception) {
            Timber.e(e, "MeshSyncWorker: maintenance cycle failed")
            Result.failure()
        }
    }
}
