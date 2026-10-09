package com.scmessenger.android.service

import android.app.ActivityManager
import android.app.ApplicationExitInfo
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import androidx.core.app.NotificationCompat
import androidx.work.BackoffPolicy
import androidx.work.CoroutineWorker
import androidx.work.ExistingWorkPolicy
import androidx.work.ForegroundInfo
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.OutOfQuotaPolicy
import androidx.work.WorkManager
import androidx.work.WorkerParameters
import com.scmessenger.android.R
import com.scmessenger.android.utils.FileLoggingTree
import com.scmessenger.android.utils.NotificationHelper
import timber.log.Timber
import java.util.concurrent.TimeUnit

/** Where an ensure request came from. [logName] is the `trigger=` value in `[MESH-START]`. */
enum class MeshStartTrigger(val logName: String) {
    PROCESS_START("process_start"),
    WORKER("worker"),
    BOOT("boot"),
    PACKAGE_REPLACED("package_replaced"),
    NETWORK("network"),
    ACTIVITY("activity")
}

/** Outcome of one ensure attempt. */
sealed class MeshEnsureResult {
    object Started : MeshEnsureResult()
    object AlreadyRunning : MeshEnsureResult()
    object UserStopped : MeshEnsureResult()
    data class Blocked(val reason: String) : MeshEnsureResult()
}

/**
 * Lifecycle lines for the start path, prefixed `[MESH-START]` / `[PROC-EXIT]`.
 * Same durability contract as [com.scmessenger.android.data.MeshStopLog]:
 * emitted through Timber, then flushed to the file log synchronously.
 */
object MeshStartLog {
    @Volatile
    internal var sink: (String) -> Unit = ::defaultSink

    fun emit(line: String) {
        kotlin.runCatching { sink(line) }
    }

    private fun defaultSink(line: String) {
        // WARN so the release file tree (WARN+) keeps the line.
        Timber.tag("MeshStart").w(line)
        Timber.forest().filterIsInstance<FileLoggingTree>().forEach { it.flushPending() }
    }
}

/**
 * Process-independent copy of the user-stop latch. [MeshForegroundService.userStoppedForSession]
 * is in-memory only; without this, a process revived by WorkManager after the
 * user pressed Stop would read "not stopped" and resurrect the mesh.
 */
object UserStopStore {
    private const val PREFS = "mesh_lifecycle"
    private const val KEY = "user_stopped"

    /** Seed the in-memory latch from disk and mirror every later write back. Call first in Application.onCreate. */
    fun install(context: Context) {
        val prefs = context.applicationContext.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
        MeshForegroundService.userStoppedForSession = prefs.getBoolean(KEY, false)
        MeshForegroundService.userStopPersist = { stopped ->
            kotlin.runCatching { prefs.edit().putBoolean(KEY, stopped).apply() }
        }
    }
}

internal object MeshAutoRestart {
    private const val ENSURE_WORK_NAME = "com.scmessenger.mesh.ensure"
    internal const val ENSURE_NOTIFICATION_ID = 1002

    enum class Decision { START, ALREADY_RUNNING, USER_STOPPED }

    /**
     * Pure rule. The user's stop always wins. A live service short-circuits
     * every trigger except ACTIVITY, which keeps its historical behaviour of
     * always delivering ACTION_ENSURE (idempotent; refreshes the notification).
     */
    fun decide(trigger: MeshStartTrigger, userStopped: Boolean, serviceAlive: Boolean): Decision = when {
        userStopped -> Decision.USER_STOPPED
        serviceAlive && trigger != MeshStartTrigger.ACTIVITY -> Decision.ALREADY_RUNNING
        else -> Decision.START
    }

    fun formatLine(trigger: MeshStartTrigger, result: MeshEnsureResult): String {
        val r = when (result) {
            MeshEnsureResult.Started -> "started"
            MeshEnsureResult.AlreadyRunning -> "already_running"
            MeshEnsureResult.UserStopped -> "user_stopped"
            is MeshEnsureResult.Blocked -> "blocked reason=${result.reason}"
        }
        return "[MESH-START] trigger=${trigger.logName} result=$r"
    }

    /** Testable core: decide, run [start] if needed, log, return. */
    fun ensure(
        trigger: MeshStartTrigger,
        userStopped: Boolean,
        serviceAlive: Boolean,
        start: () -> Unit,
        log: (String) -> Unit = MeshStartLog::emit
    ): MeshEnsureResult {
        val result = when (decide(trigger, userStopped, serviceAlive)) {
            Decision.USER_STOPPED -> MeshEnsureResult.UserStopped
            Decision.ALREADY_RUNNING -> MeshEnsureResult.AlreadyRunning
            Decision.START -> try {
                start()
                MeshEnsureResult.Started
            } catch (t: Throwable) {
                MeshEnsureResult.Blocked(blockReason(t))
            }
        }
        // Network callbacks fire constantly; do not log the no-op outcome.
        if (!(trigger == MeshStartTrigger.NETWORK && result == MeshEnsureResult.AlreadyRunning)) {
            log(formatLine(trigger, result))
        }
        return result
    }

    internal fun blockReason(t: Throwable): String {
        val name = t.javaClass.simpleName.ifEmpty { "Throwable" }
        val msg = t.message?.take(80)?.replace('\n', ' ')
        return if (msg == null) name else "$name:$msg"
    }

    /** Android entry point. A blocked direct start falls back to the expedited worker. */
    fun ensure(context: Context, trigger: MeshStartTrigger, allowWorkerFallback: Boolean = true): MeshEnsureResult {
        val app = context.applicationContext
        val result = ensure(
            trigger = trigger,
            userStopped = MeshForegroundService.userStoppedForSession,
            serviceAlive = MeshForegroundService.serviceAlive,
            start = {
                val intent = Intent(app, MeshForegroundService::class.java)
                    .apply { action = MeshForegroundService.ACTION_ENSURE }
                app.startForegroundService(intent)
                MeshForegroundService.fgsStartFailed.value = false
            }
        )
        if (result is MeshEnsureResult.Blocked) {
            MeshForegroundService.fgsStartFailed.value = true
            if (allowWorkerFallback) scheduleEnsureWorker(app)
        }
        return result
    }

    /**
     * Never-give-up fallback: an expedited job with a connectedDevice
     * foreground notification; retried with backoff until it succeeds.
     */
    fun scheduleEnsureWorker(context: Context) {
        kotlin.runCatching {
            val request = OneTimeWorkRequestBuilder<MeshEnsureWorker>()
                .setExpedited(OutOfQuotaPolicy.RUN_AS_NON_EXPEDITED_WORK_REQUEST)
                .setBackoffCriteria(BackoffPolicy.EXPONENTIAL, 30, TimeUnit.SECONDS)
                .build()
            WorkManager.getInstance(context).enqueueUniqueWork(ENSURE_WORK_NAME, ExistingWorkPolicy.KEEP, request)
        }.onFailure { Timber.w(it, "Could not schedule MeshEnsureWorker") }
    }
}

/** Expedited worker that re-ensures the mesh; Result.retry() while blocked. */
class MeshEnsureWorker(context: Context, params: WorkerParameters) : CoroutineWorker(context, params) {

    override suspend fun getForegroundInfo(): ForegroundInfo {
        val notification = NotificationCompat.Builder(applicationContext, NotificationHelper.CHANNEL_MESH_STATUS)
            .setContentTitle(applicationContext.getString(R.string.mesh_service_notification_title))
            .setContentText(applicationContext.getString(R.string.notification_mesh_resuming))
            .setSmallIcon(R.drawable.ic_notification)
            .setOngoing(true)
            .setSilent(true)
            .build()
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            ForegroundInfo(
                MeshAutoRestart.ENSURE_NOTIFICATION_ID,
                notification,
                ServiceInfo.FOREGROUND_SERVICE_TYPE_CONNECTED_DEVICE
            )
        } else {
            ForegroundInfo(MeshAutoRestart.ENSURE_NOTIFICATION_ID, notification)
        }
    }

    override suspend fun doWork(): Result {
        if (MeshForegroundService.userStoppedForSession) {
            MeshStartLog.emit(MeshAutoRestart.formatLine(MeshStartTrigger.WORKER, MeshEnsureResult.UserStopped))
            return Result.success()
        }
        // Holding our own foreground state makes the mesh start below legal.
        kotlin.runCatching { setForeground(getForegroundInfo()) }
            .onFailure { Timber.w(it, "MeshEnsureWorker: setForeground refused") }
        return when (MeshAutoRestart.ensure(applicationContext, MeshStartTrigger.WORKER, allowWorkerFallback = false)) {
            is MeshEnsureResult.Blocked -> Result.retry()
            else -> Result.success()
        }
    }
}

/** `[PROC-EXIT]` diagnostics from ActivityManager.getHistoricalProcessExitReasons (API 30+). */
internal object ProcessExitLog {
    fun reasonName(reason: Int): String = when (reason) {
        1 -> "EXIT_SELF"
        2 -> "SIGNALED"
        3 -> "LOW_MEMORY"
        4 -> "CRASH"
        5 -> "CRASH_NATIVE"
        6 -> "ANR"
        7 -> "INITIALIZATION_FAILURE"
        8 -> "PERMISSION_CHANGE"
        9 -> "EXCESSIVE_RESOURCE_USAGE"
        10 -> "USER_REQUESTED"
        11 -> "USER_STOPPED"
        12 -> "DEPENDENCY_DIED"
        13 -> "OTHER"
        14 -> "FREEZER"
        15 -> "PACKAGE_STATE_CHANGE"
        16 -> "PACKAGE_UPDATED"
        else -> "UNKNOWN($reason)"
    }

    fun format(reason: Int, description: String?, timestampMs: Long, importance: Int, pid: Int, status: Int): String =
        "[PROC-EXIT] reason=${reasonName(reason)} status=$status importance=$importance pid=$pid " +
            "ts=$timestampMs desc=\"${description.orEmpty().replace('\n', ' ')}\""

    fun logRecent(context: Context, max: Int = 5) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.R) return
        kotlin.runCatching {
            val am = context.getSystemService(Context.ACTIVITY_SERVICE) as ActivityManager
            val exits: List<ApplicationExitInfo> = am.getHistoricalProcessExitReasons(context.packageName, 0, max)
            if (exits.isEmpty()) MeshStartLog.emit("[PROC-EXIT] none recorded")
            exits.forEach {
                MeshStartLog.emit(format(it.reason, it.description, it.timestamp, it.importance, it.pid, it.status))
            }
        }.onFailure { Timber.w(it, "[PROC-EXIT] unavailable") }
    }
}
