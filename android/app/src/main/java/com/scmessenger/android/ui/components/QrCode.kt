package com.scmessenger.android.ui.components

import android.graphics.Bitmap
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Card
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.unit.dp
import com.google.zxing.BarcodeFormat
import com.google.zxing.qrcode.QRCodeWriter
import timber.log.Timber

/**
 * Renders [data] as a QR code inside a Card, or nothing if encoding fails
 * (e.g. the payload is too large for a QR code's capacity).
 */
@Composable
fun QrCodeImage(
    data: String,
    contentDescription: String?,
    modifier: Modifier = Modifier,
    size: Int = 512
) {
    val bitmap = remember(data, size) {
        try {
            generateQrCodeBitmap(data, size)
        } catch (e: Exception) {
            Timber.e(e, "Failed to generate QR code")
            null
        }
    }

    bitmap?.let {
        Card(modifier = modifier) {
            Image(
                bitmap = it.asImageBitmap(),
                contentDescription = contentDescription,
                modifier = Modifier
                    .size(256.dp)
                    .padding(16.dp)
            )
        }
    }
}

/** Generate a black-and-white QR code bitmap from string data. */
fun generateQrCodeBitmap(data: String, size: Int): Bitmap {
    val writer = QRCodeWriter()
    val bitMatrix = writer.encode(data, BarcodeFormat.QR_CODE, size, size)

    val width = bitMatrix.width
    val height = bitMatrix.height

    // Fill an IntArray in Kotlin and hand it to the bitmap in one call,
    // instead of up to 262,144 (512x512) individual JNI setPixel calls on
    // the main thread.
    val pixels = IntArray(width * height)
    for (y in 0 until height) {
        val rowOffset = y * width
        for (x in 0 until width) {
            pixels[rowOffset + x] =
                if (bitMatrix[x, y]) android.graphics.Color.BLACK else android.graphics.Color.WHITE
        }
    }

    return Bitmap.createBitmap(pixels, width, height, Bitmap.Config.RGB_565)
}
