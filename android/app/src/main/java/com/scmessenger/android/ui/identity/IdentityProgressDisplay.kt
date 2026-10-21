package com.scmessenger.android.ui.identity

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.scmessenger.android.ui.viewmodels.IdentityProgressStage
import kotlinx.coroutines.delay

/**
 * Honest progress display for identity creation.
 *
 * Shows:
 *  - A spinner + "Creating your identity…"
 *  - Real elapsed time (updated every second)
 *  - Optional sub-stage detail from the progress callback
 *
 * Does NOT claim specific phases or show fake ETA countdowns.
 * The elapsed timer starts when this composable first renders.
 */
@Composable
fun IdentityProgressDisplay(
    currentStage: IdentityProgressStage,
    subStageDetail: String? = null,
    modifier: Modifier = Modifier
) {
    // Real elapsed time counter — starts when composable enters composition
    var elapsedSeconds by remember { mutableLongStateOf(0L) }
    val startTimeMs = remember { System.currentTimeMillis() }

    LaunchedEffect(Unit) {
        while (true) {
            elapsedSeconds = (System.currentTimeMillis() - startTimeMs) / 1000L
            delay(1000L)
        }
    }

    val elapsedText = when {
        elapsedSeconds < 1L -> "Starting…"
        elapsedSeconds < 60L -> "${elapsedSeconds}s elapsed"
        else -> "${elapsedSeconds / 60}m ${elapsedSeconds % 60}s elapsed"
    }

    Card(
        modifier = modifier,
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surfaceVariant
        )
    ) {
        Column(
            modifier = Modifier.fillMaxWidth().padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            // Header: spinner + status
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                CircularProgressIndicator(
                    modifier = Modifier.size(20.dp),
                    strokeWidth = 2.dp
                )
                Text(
                    text = "Creating your identity…",
                    style = MaterialTheme.typography.titleSmall,
                    fontWeight = FontWeight.Bold
                )
            }

            // Elapsed time
            Text(
                text = elapsedText,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )

            // Sub-stage detail from progress callback (if any)
            if (!subStageDetail.isNullOrBlank()) {
                Text(
                    text = subStageDetail,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.primary,
                    fontWeight = FontWeight.SemiBold
                )
            }

            Spacer(modifier = Modifier.height(4.dp))

            // Single honest status line
            Text(
                text = "This may take a moment on first setup. Your device is generating cryptographic keys and starting the secure mesh service.",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.7f)
            )
        }
    }
}
