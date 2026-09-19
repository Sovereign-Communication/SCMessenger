package com.scmessenger.android.ui.components

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import kotlin.math.absoluteValue

/**
 * Generates a deterministic identicon (visual avatar) from a public key or identity hash.
 *
 * Creates a geometric pattern with consistent colors based on the input bytes.
 * Each unique identity gets a unique, recognizable pattern.
 *
 * Algorithm:
 * - Use first bytes to determine base color
 * - Use subsequent bytes to generate geometric shapes
 * - Create a symmetric, visually distinct pattern
 */
@Composable
fun Identicon(
    data: ByteArray,
    size: Dp = 48.dp,
    modifier: Modifier = Modifier
) {
    // Colors/pattern are cheap (6 values) so no per-frame memoization is
    // needed here; the peer-id and hex wrappers memoize the byte array
    // itself (string-keyed) which is the real per-recomposition win.
    val colors = generateColors(data)
    val pattern = generatePattern(data)

    Box(
        modifier = modifier
            .size(size)
            .clip(CircleShape)
            .background(colors.first())
    ) {
        Canvas(modifier = Modifier.size(size)) {
            val centerX = this.size.width / 2
            val centerY = this.size.height / 2
            val radius = this.size.width / 3

            // Draw geometric pattern based on hash
            pattern.forEachIndexed { index, value ->
                val angle = (index * 60f) * (Math.PI / 180.0)
                val x = centerX + (radius * Math.cos(angle).toFloat())
                val y = centerY + (radius * Math.sin(angle).toFloat())

                val shapeRadius = (value.absoluteValue % 30) + 10f
                val color = colors[index % colors.size]

                drawCircle(
                    color = color,
                    radius = shapeRadius,
                    center = Offset(x, y)
                )
            }

            // Draw center circle
            drawCircle(
                color = colors.last(),
                radius = radius / 2,
                center = Offset(centerX, centerY)
            )
        }
    }
}

/**
 * Generate a consistent color palette from data bytes.
 */
private fun generateColors(data: ByteArray): List<Color> {
    if (data.isEmpty()) {
        return listOf(Color.Gray, Color.LightGray, Color.DarkGray)
    }

    val hue = (data[0].toInt() and 0xFF) / 255f * 360f
    val saturation = if (data.size > 1) {
        (data[1].toInt() and 0xFF) / 255f * 0.5f + 0.5f
    } else {
        0.7f
    }

    val primary = Color.hsv(hue, saturation, 0.9f)
    val secondary = Color.hsv((hue + 120) % 360, saturation, 0.8f)
    val tertiary = Color.hsv((hue + 240) % 360, saturation, 0.7f)
    val accent = Color.hsv((hue + 60) % 360, saturation * 0.7f, 1.0f)

    return listOf(primary, secondary, tertiary, accent)
}

/**
 * Generate pattern values from data bytes.
 */
private fun generatePattern(data: ByteArray): List<Int> {
    if (data.isEmpty()) {
        return List(6) { 50 }
    }

    return List(6) { index ->
        val byteIndex = index % data.size
        data[byteIndex].toInt() and 0xFF
    }
}

/**
 * Convenience function for generating identicon from hex string.
 */
@Composable
fun IdenticonFromHex(
    hexString: String,
    size: Dp = 48.dp,
    modifier: Modifier = Modifier
) {
    val bytes = remember(hexString) {
        try {
            hexString.chunked(2)
                .map { it.toInt(16).toByte() }
                .toByteArray()
        } catch (e: Exception) {
            ByteArray(0)
        }
    }

    Identicon(data = bytes, size = size, modifier = modifier)
}

/**
 * Convenience function for generating identicon from peer ID string.
 */
@Composable
fun IdenticonFromPeerId(
    peerId: String,
    size: Dp = 48.dp,
    modifier: Modifier = Modifier
) {
    // Cache the byte array so rapid peer churn does not reallocate per frame.
    val bytes = remember(peerId) { peerId.toByteArray() }
    Identicon(data = bytes, size = size, modifier = modifier)
}

/**
 * Generate identicon bitmap for notifications (non-Composable).
 */
fun generateIdenticonBitmap(data: ByteArray, sizePx: Int): android.graphics.Bitmap {
    val colors = generateColors(data)
    val pattern = generatePattern(data)

    val bitmap = android.graphics.Bitmap.createBitmap(sizePx, sizePx, android.graphics.Bitmap.Config.ARGB_8888)
    val canvas = android.graphics.Canvas(bitmap)
    val paint = android.graphics.Paint(android.graphics.Paint.ANTI_ALIAS_FLAG)

    // Draw background circle
    paint.color = colors.first().toArgb()
    canvas.drawCircle(sizePx / 2f, sizePx / 2f, sizePx / 2f, paint)

    // Draw pattern
    val centerX = sizePx / 2f
    val centerY = sizePx / 2f
    val radius = sizePx / 3f

    pattern.forEachIndexed { index, value ->
        val angle = (index * 60f) * (Math.PI / 180.0)
        val x = centerX + (radius * Math.cos(angle).toFloat())
        val y = centerY + (radius * Math.sin(angle).toFloat())

        val shapeRadius = (value.absoluteValue % 30) + 10f
        val color = colors[index % colors.size]

        paint.color = color.toArgb()
        canvas.drawCircle(x, y, shapeRadius, paint)
    }

    // Draw center circle
    paint.color = colors.last().toArgb()
    canvas.drawCircle(centerX, centerY, radius / 2, paint)

    return bitmap
}

/**
 * Convert Compose Color to ARGB int.
 */
private fun Color.toArgb(): Int {
    return android.graphics.Color.argb(
        (alpha * 255).toInt(),
        (red * 255).toInt(),
        (green * 255).toInt(),
        (blue * 255).toInt()
    )
}
