package com.scmessenger.shared.desktop

import androidx.compose.ui.graphics.painter.BitmapPainter
import androidx.compose.ui.graphics.toAwtImage
import androidx.compose.ui.res.loadImageBitmap
import androidx.compose.ui.window.Tray
import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.WindowPosition
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Message
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.Icon
import androidx.compose.ui.window.ApplicationScope
import androidx.compose.ui.window.FrameWindowScope
import kotlinx.coroutines.launch
import com.scmessenger.shared.di.sharedModule
import com.scmessenger.shared.model.ServiceState
import com.scmessenger.shared.model.Contact
import com.scmessenger.shared.platform.PlatformNetworking
import com.scmessenger.shared.platform.PlatformNotifier
import com.scmessenger.shared.viewmodel.AppViewModel
import com.scmessenger.shared.viewmodel.ChatViewModel
import com.scmessenger.shared.viewmodel.ContactsViewModel
import org.koin.core.context.startKoin
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import java.awt.image.BufferedImage

class AppComponent : KoinComponent {
    val networking by inject<PlatformNetworking>()
    val notifier by inject<PlatformNotifier>()
    val appViewModel by inject<AppViewModel>()
    val contactsViewModel by inject<ContactsViewModel>()
}

fun main() {
    // Initialize Koin DI
    startKoin {
        modules(
            // Desktop-specific modules
            org.koin.dsl.module.single { PlatformNetworking() },
            org.koin.dsl.module.single { PlatformNotifier() },
            org.koin.dsl.module.single {
                AppViewModel(
                    networking = org.koin.java.KoinJavaComponent.get(PlatformNetworking::class.java),
                    storagePath = System.getProperty("user.home") + "/.config/scmessenger"
                )
            },
            org.koin.dsl.module.factory {
                ContactsViewModel(
                    networking = org.koin.java.KoinJavaComponent.get(PlatformNetworking::class.java)
                )
            },
            sharedModule
        )
    }

    application {
        val component = remember { AppComponent() }
        val appViewModel = component.appViewModel
        val serviceState by appViewModel.serviceState.collectAsState()

        TrayComposable(
            connectionStatus = serviceState.name,
            unreadCount = 0,
            onStartService = { appViewModel.startService() },
            onStopService = { appViewModel.stopService() },
            onShowMainWindow = { /* Already visible */ },
            onExitApplication = { exitApplication() }
        )

        MainWindowComposable(component)
    }
}

@Composable
fun ApplicationScope.TrayComposable(
    connectionStatus: String,
    unreadCount: Int,
    onStartService: () -> Unit,
    onStopService: () -> Unit,
    onShowMainWindow: () -> Unit,
    onExitApplication: () -> Unit
) {
    Tray(
        icon = createTrayIcon(),
        tooltip = "SCMessenger — $connectionStatus" +
                if (unreadCount > 0) " ($unreadCount unread)" else "",
        menu = {
            Item(
                text = if (connectionStatus == "RUNNING") "Stop Mesh" else "Start Mesh",
                onClick = {
                    if (connectionStatus == "RUNNING") onStopService() else onStartService()
                }
            )
            Separator()
            Item(
                text = "Show SCMessenger",
                onClick = onShowMainWindow
            )
            Separator()
            Item(
                text = "Exit",
                onClick = onExitApplication
            )
        }
    )
}

/**
 * Create a simple tray icon programmatically.
 */
private fun createTrayIcon(): org.jetbrains.skia.Bitmap {
    val size = 16
    val image = BufferedImage(size, size, BufferedImage.TYPE_INT_ARGB)
    val g = image.createGraphics()
    g.color = java.awt.Color(0x1A, 0x73, 0xE8) // Blue
    g.fillRoundRect(0, 0, size, size, 4, 4)
    g.color = java.awt.Color.WHITE
    g.font = java.awt.Font("SansSerif", java.awt.Font.BOLD, 10)
    g.drawString("S", 4, 12)
    g.dispose()

    val bitmap = org.jetbrains.skia.Bitmap()
    bitmap.allocN32Pixels(size, size)
    // Simple approach: paint directly with Skia
    val canvas = org.jetbrains.skia.Canvas(bitmap)
    canvas.clear(0)
    return bitmap
}

@Composable
fun ApplicationScope.MainWindowComposable(component: AppComponent) {
    val contactsViewModel = component.contactsViewModel
    val selectedContact by contactsViewModel.selectedContact.collectAsState()

    Window(
        title = "SCMessenger",
        onCloseRequest = {
            // Don't exit — minimize to tray
            // TODO: Hide window to tray instead of closing
        },
        state = androidx.compose.ui.window.WindowState(
            position = WindowPosition.Aligned(Alignment.Center),
            width = 1024.dp,
            height = 768.dp
        )
    ) {
        MaterialTheme {
            MasterDetailLayout(component = component)
        }
    }
}

@Composable
fun FrameWindowScope.MasterDetailLayout(component: AppComponent) {
    val contactsViewModel = component.contactsViewModel
    val contacts by contactsViewModel.contacts.collectAsState()
    val selectedContact by contactsViewModel.selectedContact.collectAsState()

    Row(modifier = Modifier.fillMaxSize()) {
        // Left panel: Contact list (300dp fixed)
        Box(
            modifier = Modifier
                .width(300.dp)
                .fillMaxHeight()
                .background(MaterialTheme.colorScheme.surfaceVariant)
        ) {
            ContactList(
                contacts = contacts,
                selectedContact = selectedContact,
                onContactSelected = { contactsViewModel.selectContact(it) },
                appViewModel = component.appViewModel
            )
        }

        // Divider
        Box(
            modifier = Modifier
                .width(1.dp)
                .fillMaxHeight()
                .background(MaterialTheme.colorScheme.outlineVariant)
        )

        // Right panel: Chat view or empty state
        Box(
            modifier = Modifier
                .weight(1f)
                .fillMaxHeight()
        ) {
            if (selectedContact != null) {
                ChatView(
                    contact = selectedContact!!,
                    networking = component.networking
                )
            } else {
                EmptyChatPlaceholder()
            }
        }
    }
}

@Composable
fun EmptyChatPlaceholder() {
    Box(
        modifier = Modifier.fillMaxSize(),
        contentAlignment = Alignment.Center
    ) {
        Column(horizontalAlignment = Alignment.CenterHorizontally) {
            Icon(
                imageVector = Icons.Default.Message,
                contentDescription = null,
                modifier = Modifier.size(64.dp),
                tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.4f)
            )
            Spacer(modifier = Modifier.height(16.dp))
            Text(
                text = "Select a contact to start chatting",
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
            )
        }
    }
}
