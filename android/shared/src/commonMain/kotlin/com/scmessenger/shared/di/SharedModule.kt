package com.scmessenger.shared.di

import com.scmessenger.shared.viewmodel.ChatViewModel
import org.koin.core.module.dsl.factoryOf
import org.koin.dsl.module

/**
 * Shared Koin DI module.
 * Platform-specific modules add the actual implementations via expect/actual.
 */
val sharedModule = module {
    // ViewModels are factories — new instance per screen
    factoryOf(::ChatViewModel)
}
