package com.scmessenger.android.data

import uniffi.api.IronCoreException

/**
 * The core dial guard reports a dial it deliberately did not dispatch
 * (target is self / our own address / rate-limited probe) as the typed
 * [IronCoreException.DialSkipped]. That is neither a success nor a failure
 * and carries NO connectivity evidence: callers must not book failures,
 * backoff, dead-marking, recordSuccess or a Connected state against it.
 *
 * An already-connected skip (exact socket / peer id) is returned by the core
 * as a normal success, so it never reaches this type.
 */
object DialSkip {
    fun isSkipped(error: Throwable): Boolean = error is IronCoreException.DialSkipped
}
