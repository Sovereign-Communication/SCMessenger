plugins {
    kotlin("multiplatform")
    id("org.jetbrains.compose")
}

kotlin {
    // JVM target for Compose Desktop (runs on Linux via JVM)
    jvm("desktop") {
        withJava()
    }

    // Android target
    androidTarget {
        compilations.all {
            kotlinOptions {
                jvmTarget = "17"
            }
        }
    }

    sourceSets {
        val commonMain by getting {
            dependencies {
                // Compose Multiplatform core
                implementation(compose.runtime)
                implementation(compose.foundation)
                implementation(compose.material3)
                implementation(compose.ui)

                // Coroutines
                implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.7.3")

                // Koin for KMP DI (replaces Hilt in shared module)
                implementation("io.insert-koin:koin-core:3.5.0")
            }
        }

        val commonTest by getting {
            dependencies {
                implementation(kotlin("test"))
                implementation("org.jetbrains.kotlinx:kotlinx-coroutines-test:1.7.3")
            }
        }

        val androidMain by getting {
            dependencies {
                implementation("io.insert-koin:koin-android:3.5.0")
            }
        }

        // Compose Desktop runs on the JVM target
        val desktopMain by getting {
            dependsOn(commonMain)
            dependencies {
                implementation(compose.desktop.currentOs)
            }
        }
    }
}

// Desktop application distribution configuration
compose.desktop {
    application {
        mainClass = "com.scmessenger.shared.desktop.MainKt"

        nativeDistributions {
            targetFormats(
                org.jetbrains.compose.desktop.application.dsl.TargetFormat.Deb,
                org.jetbrains.compose.desktop.application.dsl.TargetFormat.Rpm,
                org.jetbrains.compose.desktop.application.dsl.TargetFormat.TarGz
            )
            packageName = "SCMessenger"
            packageVersion = "0.4.0"
            description = "SCMessenger Encrypted Delay-Tolerant Messaging"
            copyright = "© 2024-2026 SCMessenger Contributors"

            // XDG desktop conventions
            linux {
                iconFile.set(project.file("src/desktopMain/resources/icon.png"))
                menuGroup = "Network;RemoteAccess;"
                appCategory = "Network"
            }

            jvmArgs += listOf("-Xmx512m", "-Djava.awt.headless=false")
        }
    }
}
