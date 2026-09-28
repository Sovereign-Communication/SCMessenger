<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

TARGET: android\app\src\main\java\com\scmessenger\android\di\AppModule.kt

SYSTEM DIRECTIVE: COMPREHENSIVE DEAD-END RESOLUTION
The function 'provideMeshRepository' is defined in 'android\app\src\main\java\com\scmessenger\android\di\AppModule.kt' but is NEVER called anywhere in the repository. Your job is to fully integrate it into the overarching application loop. It may need to be wired into 1, 2, or 3+ different locations.

================================================================================
INTEGRATION SUMMARY
================================================================================

ANALYSIS:
---------
The 'provideMeshRepository' function in AppModule.kt is a Hilt dependency
injection provider method. It is annotated with @Provides and @Singleton,
and is installed in SingletonComponent. While not explicitly called in source
code, it is automatically invoked by Hilt's generated code when components
need a MeshRepository instance.

WIRING VERIFICATION:
--------------------
MeshRepository is properly injected into the following classes:
- AndroidPlatformBridge
- TopicManager
- ContactsViewModel
- ChatViewModel
- ConversationsViewModel
- DashboardViewModel
- IdentityViewModel
- SettingsViewModel
- MainViewModel
- MeshServiceViewModel
- MainActivity
- MeshForegroundService

All these classes have MeshRepository as a constructor parameter or @Inject
field, which means Hilt automatically calls provideMeshRepository() to provide
the dependency.

The provideMeshRepository function is properly configured:
- Annotated with @Provides and @Singleton
- In a @Module that's @InstallIn(SingletonComponent::class)
- Takes @ApplicationContext as a parameter
- Returns MeshRepository instance

BUILD STATUS:
-------------
Android: BUILD SUCCESSFUL
Rust: cargo check --workspace completed successfully

STATUS: SUCCESS_STOP


--- CLOSEOUT EVIDENCE ---
VERIFIED WIRED: DI @Provides @Singleton at AppModule.kt:26. Invoked by Hilt DI container at runtime.
Verified by: orchestrator-TRP-2026-05-03
