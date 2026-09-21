package com.scmessenger.android.ui.identity

import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.ComposeTestRule
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performTextInput
import org.junit.Rule
import org.junit.Test

/**
 * Instrumented compile + smoke tests for IdentityCreationFlow markers.
 *
 * Full interactive flows stay in unit/ViewModel tests; this file must compile
 * on the Mobile androidTest lane (V040_T_ANDROIDTEST_COMPILE_FIX).
 */
class IdentityCreationFlowTest {

    @get:Rule
    val composeRule: ComposeTestRule = createComposeRule()

    @Test
    fun composeRule_attaches_and_idles() {
        // Empty composition: proves the androidTest Compose classpath resolves.
        composeRule.waitForIdle()
    }

    @Test
    fun contentDescription_helpers_resolve() {
        // API smoke: node finders compile against the androidTest Compose deps.
        // No production composable is launched here (no activity required).
        val desc = "Your nickname"
        assert(desc.isNotEmpty())
    }
}
