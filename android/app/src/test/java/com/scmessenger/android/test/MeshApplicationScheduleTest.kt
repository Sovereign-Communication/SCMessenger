package com.scmessenger.android.test

import androidx.work.ExistingPeriodicWorkPolicy
import androidx.work.NetworkType
import com.scmessenger.android.MeshApplication
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import java.util.concurrent.TimeUnit

/**
 * Verifies the periodic [com.scmessenger.android.service.MeshSyncWorker]
 * request built at app-start (`MeshApplication.schedulePeriodicMaintenance`)
 * has the scheduling contract background sync depends on: a stable unique
 * work name (so re-scheduling on every process start doesn't pile up
 * duplicate jobs), a 15-minute cadence, and a battery-not-low constraint.
 */
class MeshApplicationScheduleTest {

    @Test
    fun `mesh sync work request uses the expected unique name and policy`() {
        assertEquals("com.scmessenger.mesh.maintenance", MeshApplication.MESH_SYNC_WORK_NAME)
        assertEquals(ExistingPeriodicWorkPolicy.KEEP, MeshApplication.MESH_SYNC_WORK_POLICY)
    }

    @Test
    fun `mesh sync work request runs every 15 minutes`() {
        val request = MeshApplication.buildMeshSyncWorkRequest()
        val expectedIntervalMillis = TimeUnit.MINUTES.toMillis(MeshApplication.MESH_SYNC_INTERVAL_MINUTES)

        assertEquals(15L, MeshApplication.MESH_SYNC_INTERVAL_MINUTES)
        assertEquals(expectedIntervalMillis, request.workSpec.intervalDuration)
    }

    @Test
    fun `mesh sync work request does not require network but does require battery not low`() {
        val request = MeshApplication.buildMeshSyncWorkRequest()
        val constraints = request.workSpec.constraints

        assertEquals(NetworkType.NOT_REQUIRED, constraints.requiredNetworkType)
        assertTrue(
            "constraints must require battery-not-low so sync doesn't drain a low battery",
            constraints.requiresBatteryNotLow()
        )
    }
}
