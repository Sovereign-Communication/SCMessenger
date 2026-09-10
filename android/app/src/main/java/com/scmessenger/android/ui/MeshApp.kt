package com.scmessenger.android.ui

import androidx.compose.foundation.layout.*
import com.scmessenger.android.R
import androidx.compose.ui.res.stringResource
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Block
import androidx.compose.material.icons.automirrored.filled.Chat
import androidx.compose.material.icons.filled.People
import androidx.compose.material.icons.filled.Router
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.navigation.NavHostController
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import com.scmessenger.android.ui.contacts.AddContactScreen
import com.scmessenger.android.ui.contacts.ContactDetailScreen
import com.scmessenger.android.ui.dashboard.PeerListScreen
import com.scmessenger.android.ui.dashboard.TopologyScreen
import com.scmessenger.android.ui.identity.IdentityScreen
import com.scmessenger.android.ui.screens.*
import com.scmessenger.android.ui.screens.RequestsInboxScreen
import com.scmessenger.android.ui.viewmodels.MainViewModel
import com.scmessenger.android.ui.viewmodels.DeepLinkData
import com.scmessenger.android.ui.settings.MeshSettingsScreen
import com.scmessenger.android.ui.settings.PowerSettingsScreen
import com.scmessenger.android.ui.join.JoinMeshScreen
import com.scmessenger.android.data.MeshRepository
import dagger.hilt.EntryPoint
import dagger.hilt.InstallIn
import dagger.hilt.android.EntryPointAccessors
import dagger.hilt.components.SingletonComponent
import androidx.compose.ui.platform.LocalContext

@EntryPoint
@InstallIn(SingletonComponent::class)
interface MeshAppEntryPoint {
    fun getMeshRepository(): MeshRepository
}

/**
 * Root composable for the SCMessenger app.
 *
 * Sets up the navigation graph and bottom navigation bar.
 */
@Composable
fun MeshApp(mainViewModel: MainViewModel = hiltViewModel()) {
    val hasIdentity by mainViewModel.hasIdentity.collectAsState()
    val showOnboarding by mainViewModel.showOnboarding.collectAsState()
    val isStorageLow by mainViewModel.isStorageLow.collectAsState()
    val availableStorageMB by mainViewModel.availableStorageMB.collectAsState()
    val navController = rememberNavController()
    val pendingDeepLink by mainViewModel.pendingDeepLink.collectAsState()
    val pendingRequestsInbox by mainViewModel.pendingRequestsInbox.collectAsState()

    LaunchedEffect(Unit) {
        mainViewModel.refreshIdentityState()
    }

    // Navigate to AddContact when a deep link arrives and identity is ready
    LaunchedEffect(pendingDeepLink, hasIdentity) {
        if (pendingDeepLink != null && hasIdentity) {
            navController.navigate(Screen.AddContact.route) {
                launchSingleTop = true
            }
        }
    }

    // Issue #6: Debounce hasIdentity changes to prevent transient false from causing
    // navigation jumps. We track the "stable" identity state to avoid flicker.
    var hasStableIdentity by remember { mutableStateOf(hasIdentity) }

    LaunchedEffect(hasIdentity) {
        // Debounce: only update stable identity after a short delay
        // This prevents transient false values from triggering navigation jumps
        kotlinx.coroutines.delay(300L)
        hasStableIdentity = hasIdentity
    }

    // Navigate to add contact when deep link arrives
    LaunchedEffect(pendingDeepLink, hasStableIdentity) {
        if (pendingDeepLink != null && hasStableIdentity) {
            navController.navigate(Screen.AddContact.route) {
                launchSingleTop = true
            }
        }
    }

    // Navigate to requests inbox when ACTION_OPEN_REQUESTS is triggered
    LaunchedEffect(pendingRequestsInbox, hasStableIdentity) {
        if (pendingRequestsInbox != null && hasStableIdentity) {
            navController.navigate(Screen.RequestsInbox.route) {
                launchSingleTop = true
                popUpTo(navController.graph.startDestinationId) {
                    saveState = true
                }
            }
        }
    }

    // Navigation guard: only trigger when stable identity state changes
    LaunchedEffect(hasStableIdentity) {
        val currentRoute = navController.currentBackStackEntry?.destination?.route
        val allowedRoutes = roleBasedBottomNavItems(hasStableIdentity).map { it.route }.toSet()
        if (currentRoute != null && currentRoute !in allowedRoutes && !currentRoute.startsWith("chat/")) {
            navController.navigate(startDestinationForRole(hasStableIdentity)) {
                launchSingleTop = true
            }
        }
    }

    Column(modifier = Modifier.fillMaxSize()) {
        if (isStorageLow) {
            com.scmessenger.android.ui.components.StorageWarningBanner(availableMB = availableStorageMB)
        }

        if (showOnboarding) {
            OnboardingScreen(
                onOnboardingComplete = { mainViewModel.refreshIdentityState() },
                viewModel = mainViewModel,
                modifier = Modifier.weight(1f)
            )
        } else {
            // Track current route so the outer Scaffold can host a
            // per-screen FAB without nesting a Scaffold inside the
            // Contacts screen. Nested Scaffolds previously hid the "+" FAB.
            val navBackStackEntry by navController.currentBackStackEntryAsState()
            val currentRoute = navBackStackEntry?.destination?.route
            val isContactsScreen = currentRoute == Screen.Contacts.route

            Scaffold(
                modifier = Modifier.weight(1f),
                bottomBar = { MeshBottomBar(navController = navController, hasIdentity = hasIdentity) },
                floatingActionButton = {
                    if (isContactsScreen) {
                        FloatingActionButton(
                            onClick = { navController.navigate(Screen.AddContact.route) }
                        ) {
                            Icon(
                                imageVector = Icons.Default.Add,
                                contentDescription = stringResource(R.string.app_content_desc_add_contact)
                            )
                        }
                    }
                }
            ) { paddingValues ->
                MeshNavHost(
                    navController = navController,
                    hasIdentity = hasIdentity,
                    onIdentityChanged = { mainViewModel.refreshIdentityState() },
                    bottomPadding = paddingValues
                )
            }
        }
    }
}

@Composable
fun MeshNavHost(
    navController: NavHostController,
    hasIdentity: Boolean,
    onIdentityChanged: () -> Unit,
    bottomPadding: PaddingValues = PaddingValues()
) {
    NavHost(
        navController = navController,
        startDestination = startDestinationForRole(hasIdentity)
    ) {
        if (hasIdentity) {
            composable(Screen.Conversations.route) {
                // P0_LAYOUT_FIX: apply bottomPadding so the bottom NavigationBar
                // (rendered by the outer Scaffold) doesn't overlap the last
                // message row. Previously only SettingsScreen got this padding,
                // so Contacts/Conversations were partially covered by the navbar.
                Box(modifier = Modifier.padding(bottomPadding)) {
                    ConversationsScreen(
                        onNavigateToChat = { peerId ->
                            navController.navigate("chat/$peerId")
                        }
                    )
                }
            }

            composable(Screen.Contacts.route) {
                // P0_LAYOUT_FIX: see ConversationsScreen comment. Without this
                // the search bar and last list row were covered by the navbar.
                Box(modifier = Modifier.padding(bottomPadding)) {
                    ContactsScreen(
                        onNavigateToChat = { peerId ->
                            navController.navigate("chat/$peerId")
                        },
                        onNavigateToAddContact = {
                            navController.navigate(Screen.AddContact.route)
                        },
                        onNavigateToContactDetail = { contactId ->
                            navController.navigate("contact/$contactId")
                        }
                    )
                }
            }

            composable(
                route = "contact/{contactId}",
                arguments = listOf(androidx.navigation.navArgument("contactId") { type = androidx.navigation.NavType.StringType })
            ) { backStackEntry ->
                val contactId = backStackEntry.arguments?.getString("contactId") ?: return@composable
                // P0_LAYOUT_FIX: contact detail is pushed via the bottom nav so it
                // also needs the bottom padding.
                Box(modifier = Modifier.padding(bottomPadding)) {
                    ContactDetailScreen(
                        contactId = contactId,
                        onNavigateBack = { navController.popBackStack() },
                        onNavigateToChat = { peerId ->
                            navController.navigate("chat/$peerId")
                        },
                        onNavigateToVerify = { peerId ->
                            navController.navigate("verify_safety_number/$peerId")
                        }
                    )
                }
            }

            composable(
                route = "verify_safety_number/{contactId}",
                arguments = listOf(androidx.navigation.navArgument("contactId") { type = androidx.navigation.NavType.StringType })
            ) { backStackEntry ->
                val contactId = backStackEntry.arguments?.getString("contactId") ?: return@composable
                Box(modifier = Modifier.padding(bottomPadding)) {
                    com.scmessenger.android.ui.contacts.VerifySafetyNumberScreen(
                        contactId = contactId,
                        onNavigateBack = { navController.popBackStack() }
                    )
                }
            }

            composable(Screen.AddContact.route) {
                val mainVm: MainViewModel = hiltViewModel()
                val deepLinkData = remember { mainVm.consumeDeepLink() }
                AddContactScreen(
                    onNavigateBack = { navController.popBackStack() },
                    onContactAdded = { navController.popBackStack() },
                    prefilledPeerId = deepLinkData?.peerId ?: "",
                    prefilledPublicKey = deepLinkData?.publicKey ?: "",
                    prefilledNickname = deepLinkData?.nickname ?: "",
                    prefilledLibp2pPeerId = deepLinkData?.libp2pPeerId,
                    prefilledListeners = deepLinkData?.listeners ?: emptyList()
                )
            }
        }

        composable(Screen.Dashboard.route) {
            // P0_LAYOUT_FIX: Dashboard is on the bottom nav, so apply padding.
            Box(modifier = Modifier.padding(bottomPadding)) {
                DashboardScreen(
                    onNavigateToPeerList = {
                        navController.navigate("peer_list")
                    },
                    onNavigateToTopology = {
                        navController.navigate("topology")
                    },
                    onNavigateToJoinMesh = {
                        navController.navigate(Screen.JoinMesh.route)
                    }
                )
            }
        }

        composable(
            route = "peer_list"
        ) {
            PeerListScreen(
                onNavigateBack = { navController.popBackStack() }
            )
        }

        composable(
            route = "topology"
        ) {
            TopologyScreen(
                onNavigateBack = { navController.popBackStack() }
            )
        }

        composable(Screen.Settings.route) {
            Box(modifier = Modifier.padding(bottomPadding)) {
                SettingsScreen(
                    onNavigateToIdentity = {
                        navController.navigate(Screen.Identity.route)
                    },
                    onNavigateToDiagnostics = {
                        navController.navigate(Screen.Diagnostics.route)
                    },
                    onNavigateToBlockedPeers = {
                        navController.navigate(Screen.BlockedPeers.route)
                    },
                    onNavigateToMeshSettings = {
                        navController.navigate("mesh_settings")
                    },
                    onNavigateToPowerSettings = {
                        navController.navigate("power_settings")
                    }
                )
            }
        }

        composable(
            route = "mesh_settings"
        ) {
            MeshSettingsScreen(
                onNavigateBack = { navController.popBackStack() }
            )
        }

        composable(
            route = "power_settings"
        ) {
            PowerSettingsScreen(
                onNavigateBack = { navController.popBackStack() }
            )
        }

        composable(Screen.Identity.route) {
            IdentityScreen(
                onNavigateBack = {
                    onIdentityChanged()
                    navController.popBackStack()
                }
            )
        }



        composable(Screen.Diagnostics.route) {
            DiagnosticsScreen(
                onNavigateBack = { navController.popBackStack() }
            )
        }

        composable(Screen.JoinMesh.route) {
            val context = LocalContext.current
            val repository = remember(context) {
                EntryPointAccessors.fromApplication(
                    context.applicationContext,
                    MeshAppEntryPoint::class.java
                ).getMeshRepository()
            }
            JoinMeshScreen(
                repository = repository,
                onJoinSuccess = { navController.popBackStack() },
                onCancel = { navController.popBackStack() }
            )
        }

        composable(Screen.BlockedPeers.route) {
            BlockedPeersScreen(
                onNavigateBack = { navController.popBackStack() }
            )
        }

        composable(Screen.RequestsInbox.route) {
            RequestsInboxScreen(
                onNavigateBack = { navController.popBackStack() }
            )
        }

        if (hasIdentity) {
            composable(
                route = "chat/{peerId}",
                arguments = listOf(androidx.navigation.navArgument("peerId") { type = androidx.navigation.NavType.StringType })
            ) { backStackEntry ->
                val peerId = backStackEntry.arguments?.getString("peerId") ?: return@composable
                ChatScreen(
                    conversationId = peerId,
                    onNavigateBack = { navController.popBackStack() }
                )
            }
        }
    }
}

/**
 * Bottom navigation bar.
 */
@Composable
fun MeshBottomBar(navController: NavHostController, hasIdentity: Boolean) {
    val navBackStackEntry by navController.currentBackStackEntryAsState()
    val currentRoute = navBackStackEntry?.destination?.route

    // Hide bottom bar on Chat screen
    if (currentRoute?.startsWith("chat/") == true) return

    NavigationBar {
        roleBasedBottomNavItems(hasIdentity).forEach { screen ->
            NavigationBarItem(
                icon = { Icon(screen.icon, contentDescription = screen.label) },
                label = { Text(screen.label) },
                selected = currentRoute == screen.route,
                onClick = {
                    navController.navigate(screen.route) {
                        popUpTo(navController.graph.startDestinationId) {
                            saveState = true
                        }
                        launchSingleTop = true
                        restoreState = true
                    }
                }
            )
        }
    }
}

/**
 * Screen definitions for navigation.
 */
sealed class Screen(val route: String, val label: String, val icon: androidx.compose.ui.graphics.vector.ImageVector) {
    object Conversations : Screen("conversations", "Chats", androidx.compose.material.icons.Icons.AutoMirrored.Filled.Chat)
    object Contacts : Screen("contacts", "Contacts", androidx.compose.material.icons.Icons.Default.People)
    object AddContact : Screen("add_contact", "Add Contact", androidx.compose.material.icons.Icons.Filled.Add)
    object Dashboard: Screen("dashboard", "Mesh", androidx.compose.material.icons.Icons.Filled.Router)
    object Settings : Screen("settings", "Settings", androidx.compose.material.icons.Icons.Default.Settings)
    object Identity : Screen("identity", "Identity", androidx.compose.material.icons.Icons.Default.Settings)
    object Diagnostics : Screen("diagnostics", "Diagnostics", androidx.compose.material.icons.Icons.Default.Settings)
    object BlockedPeers : Screen("blocked_peers", "Blocked Peers", androidx.compose.material.icons.Icons.Filled.Block)
    object RequestsInbox : Screen("requests_inbox", "Requests", androidx.compose.material.icons.Icons.Default.Add)
    object JoinMesh : Screen("join_mesh", "Join Mesh", androidx.compose.material.icons.Icons.Filled.Router)

    companion object {
        val fullRoleBottomNavItems = listOf(Conversations, Contacts, Dashboard, Settings)
        val relayOnlyBottomNavItems = listOf(Dashboard, Settings)
    }
}

internal fun roleBasedBottomNavItems(hasIdentity: Boolean): List<Screen> =
    if (hasIdentity) Screen.fullRoleBottomNavItems else Screen.relayOnlyBottomNavItems

internal fun startDestinationForRole(hasIdentity: Boolean): String =
    if (hasIdentity) Screen.Conversations.route else Screen.Dashboard.route
